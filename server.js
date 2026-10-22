import { createServer } from 'node:http';
import { createReadStream, createWriteStream } from 'node:fs';
import { chmod, mkdtemp, rm, unlink } from 'node:fs/promises';
import { networkInterfaces, tmpdir } from 'node:os';
import { extname, join } from 'node:path';
import { randomBytes, randomUUID, timingSafeEqual } from 'node:crypto';
import { isIPv4 } from 'node:net';
import { Transform } from 'node:stream';
import { pipeline } from 'node:stream/promises';
import QRCode from 'qrcode';

const PORT = Number(process.env.HANDOFF_PORT ?? 4173);
if (!Number.isInteger(PORT) || PORT < 1024 || PORT > 65535) {
  throw new Error('HANDOFF_PORT must be a number from 1024 to 65535.');
}
if (process.env.HANDOFF_HOST && !isIPv4(process.env.HANDOFF_HOST)) {
  throw new Error('HANDOFF_HOST must be an IPv4 address.');
}
const MAX_FILE_BYTES = 512 * 1024 * 1024;
const MAX_SESSION_BYTES = 2 * 1024 * 1024 * 1024;
const MAX_SESSION_FILES = 200;
const SESSION_TOKEN = randomBytes(32).toString('base64url');
const sessionDirectory = await mkdtemp(join(tmpdir(), 'handoff-'));
await chmod(sessionDirectory, 0o700);

const sharedFiles = new Map();
let sessionBytes = 0;
let reservedBytes = 0;
let reservedFiles = 0;

function getLanAddresses() {
  const interfaces = networkInterfaces();
  const entries = Object.entries(interfaces).flatMap(([name, addresses]) =>
    (addresses ?? [])
      .filter((address) => address.family === 'IPv4' && !address.internal && !address.address.startsWith('169.254.'))
      .map((address) => ({ name, address: address.address }))
  );
  const preferred = entries.filter(({ name }) =>
    !/^(docker|br-|veth|virbr|tailscale|wg|tun|podman)/i.test(name)
  );
  return (preferred.length > 0 ? preferred : entries).map(({ address }) => address);
}

const lanAddresses = getLanAddresses();
const primaryAddress = process.env.HANDOFF_HOST ?? lanAddresses[0] ?? null;

function safeFilename(value) {
  const leaf = String(value ?? 'file').replaceAll('\\', '/').split('/').at(-1) ?? 'file';
  const clean = Array.from(leaf.replace(/[\u0000-\u001f\u007f]/g, '').trim()).slice(0, 180).join('');
  return clean || 'file';
}

function contentDisposition(filename) {
  const ascii = filename.replace(/[^\x20-\x7e]|[\\"]/g, '_');
  return `attachment; filename="${ascii}"; filename*=UTF-8''${encodeURIComponent(filename)}`;
}

function send(res, status, body, contentType = 'application/json; charset=utf-8') {
  if (res.destroyed || res.headersSent) return;
  res.writeHead(status, {
    'Content-Type': contentType,
    'Cache-Control': 'no-store',
    'X-Content-Type-Options': 'nosniff',
    'Referrer-Policy': 'no-referrer',
    'Content-Security-Policy': "default-src 'self'; img-src 'self'; style-src 'self'; script-src 'self'; connect-src 'self'; base-uri 'none'; frame-ancestors 'none'"
  });
  res.end(body);
}

function sendJson(res, status, payload) {
  send(res, status, JSON.stringify(payload));
}

function sameToken(candidate) {
  if (!candidate) return false;
  const received = Buffer.from(candidate);
  const expected = Buffer.from(SESSION_TOKEN);
  return received.length === expected.length && timingSafeEqual(received, expected);
}

function isLoopbackRequest(req) {
  const address = req.socket.remoteAddress ?? '';
  return address === '127.0.0.1' || address === '::1' || address.startsWith('::ffff:127.');
}

function originIsLocal(req) {
  const origin = req.headers.origin;
  if (!origin) return true;
  return origin === `http://${req.headers.host}`;
}

function getMimeType(filename) {
  const extension = extname(filename).toLowerCase();
  return new Map([
    ['.7z', 'application/x-7z-compressed'],
    ['.avif', 'image/avif'],
    ['.csv', 'text/csv; charset=utf-8'],
    ['.gif', 'image/gif'],
    ['.heic', 'image/heic'],
    ['.jpeg', 'image/jpeg'],
    ['.jpg', 'image/jpeg'],
    ['.json', 'application/json'],
    ['.m4a', 'audio/mp4'],
    ['.mp3', 'audio/mpeg'],
    ['.mp4', 'video/mp4'],
    ['.pdf', 'application/pdf'],
    ['.png', 'image/png'],
    ['.svg', 'image/svg+xml'],
    ['.txt', 'text/plain; charset=utf-8'],
    ['.webp', 'image/webp'],
    ['.zip', 'application/zip']
  ]).get(extension) ?? 'application/octet-stream';
}

async function listFiles() {
  return [...sharedFiles.values()]
    .sort((left, right) => right.createdAt - left.createdAt)
    .map(({ id, name, size, createdAt }) => ({ id, name, size, createdAt }));
}

function routeToken(pathname) {
  const match = pathname.match(/^\/(?:share|api)\/([A-Za-z0-9_-]+)(?:\/|$)/);
  return match?.[1] ?? null;
}

async function handleUpload(req, res, url) {
  const filename = safeFilename(url.searchParams.get('name'));
  const declaredBytes = Number(req.headers['content-length']);

  if (!Number.isSafeInteger(declaredBytes) || declaredBytes < 0) {
    req.resume();
    return sendJson(res, 411, { error: 'The browser did not provide a file size.' });
  }
  if (declaredBytes > MAX_FILE_BYTES) {
    req.resume();
    return sendJson(res, 413, { error: 'Each file must be 512 MB or smaller.' });
  }
  if (sharedFiles.size + reservedFiles >= MAX_SESSION_FILES) {
    req.resume();
    return sendJson(res, 413, { error: 'This room already has 200 files.' });
  }
  if (sessionBytes + reservedBytes + declaredBytes > MAX_SESSION_BYTES) {
    req.resume();
    return sendJson(res, 413, { error: 'This room has reached its 2 GB limit.' });
  }

  const id = randomUUID();
  const filePath = join(sessionDirectory, id);
  reservedBytes += declaredBytes;
  reservedFiles += 1;
  let receivedBytes = 0;
  const sizeLimit = new Transform({
    transform(chunk, encoding, callback) {
      receivedBytes += chunk.length;
      if (receivedBytes > declaredBytes || receivedBytes > MAX_FILE_BYTES) {
        callback(new Error('The upload exceeded its declared size.'));
        return;
      }
      callback(null, chunk);
    }
  });

  try {
    await pipeline(req, sizeLimit, createWriteStream(filePath, { flags: 'wx', mode: 0o600 }));
    if (receivedBytes !== declaredBytes) throw new Error('The upload ended before the full file arrived.');
    const createdAt = Date.now();
    sharedFiles.set(id, { id, name: filename, size: receivedBytes, createdAt, filePath });
    sessionBytes += receivedBytes;
    sendJson(res, 201, { id, name: filename, size: receivedBytes, createdAt });
  } catch (error) {
    await unlink(filePath).catch(() => {});
    if (error.code === 'ECONNRESET' || req.destroyed) return;
    sendJson(res, 400, { error: 'The file could not be received. Try again.' });
  } finally {
    reservedBytes -= declaredBytes;
    reservedFiles -= 1;
  }
}

const staticFiles = new Map([
  ['/', { path: 'index.html', type: 'text/html; charset=utf-8' }],
  ['/app.js', { path: 'app.js', type: 'text/javascript; charset=utf-8' }],
  ['/style.css', { path: 'style.css', type: 'text/css; charset=utf-8' }]
]);

const server = createServer(async (req, res) => {
  const url = new URL(req.url ?? '/', 'http://handoff.local');
  const token = routeToken(url.pathname);

  if (req.method === 'GET' && url.pathname === '/') {
    if (!isLoopbackRequest(req)) return sendJson(res, 404, { error: 'Open Handoff on the host device or use a room link.' });
    res.writeHead(302, { Location: `/share/${SESSION_TOKEN}`, 'Cache-Control': 'no-store' });
    res.end();
    return;
  }

  if (req.method === 'GET' && url.pathname.startsWith('/share/')) {
    if (!sameToken(token)) return sendJson(res, 404, { error: 'This transfer room is closed.' });
    const page = staticFiles.get('/');
    res.writeHead(200, {
      'Content-Type': page.type,
      'Cache-Control': 'no-store',
      'X-Content-Type-Options': 'nosniff',
      'Referrer-Policy': 'no-referrer',
      'Content-Security-Policy': "default-src 'self'; img-src 'self'; style-src 'self'; script-src 'self'; connect-src 'self'; base-uri 'none'; frame-ancestors 'none'"
    });
    createReadStream(new URL(`./public/${page.path}`, import.meta.url)).pipe(res);
    return;
  }

  if (url.pathname.startsWith('/api/')) {
    if (!sameToken(token)) return sendJson(res, 404, { error: 'This transfer room is closed.' });
    if (!originIsLocal(req)) return sendJson(res, 403, { error: 'This request did not come from the transfer room.' });

    if (req.method === 'GET' && url.pathname === `/api/${SESSION_TOKEN}/session`) {
      const shareUrl = primaryAddress ? `http://${primaryAddress}:${PORT}/share/${SESSION_TOKEN}` : '';
      return sendJson(res, 200, { shareUrl, hasLanAddress: Boolean(primaryAddress), maxFileBytes: MAX_FILE_BYTES });
    }
    if (req.method === 'GET' && url.pathname === `/api/${SESSION_TOKEN}/qr.svg`) {
      if (!primaryAddress) return sendJson(res, 503, { error: 'Connect to a local network before creating a room code.' });
      try {
        const shareUrl = `http://${primaryAddress}:${PORT}/share/${SESSION_TOKEN}`;
        const svg = await QRCode.toString(shareUrl, {
          type: 'svg',
          width: 240,
          margin: 1,
          errorCorrectionLevel: 'M',
          color: { dark: '#20251e', light: '#f4f1e8' }
        });
        return send(res, 200, svg, 'image/svg+xml; charset=utf-8');
      } catch {
        return sendJson(res, 500, { error: 'The connection code could not be created.' });
      }
    }
    if (req.method === 'GET' && url.pathname === `/api/${SESSION_TOKEN}/files`) {
      return sendJson(res, 200, { files: await listFiles(), totalBytes: sessionBytes });
    }
    if (req.method === 'POST' && url.pathname === `/api/${SESSION_TOKEN}/files`) {
      return handleUpload(req, res, url);
    }

    const downloadMatch = url.pathname.match(new RegExp(`^/api/${SESSION_TOKEN}/files/([0-9a-f-]{36})$`));
    if (req.method === 'GET' && downloadMatch) {
      const sharedFile = sharedFiles.get(downloadMatch[1]);
      if (!sharedFile) return sendJson(res, 404, { error: 'That file is no longer in this room.' });
      res.writeHead(200, {
        'Content-Type': getMimeType(sharedFile.name),
        'Content-Length': sharedFile.size,
        'Content-Disposition': contentDisposition(sharedFile.name),
        'Cache-Control': 'no-store',
        'X-Content-Type-Options': 'nosniff',
        'Referrer-Policy': 'no-referrer'
      });
      createReadStream(sharedFile.filePath).pipe(res);
      return;
    }
    return sendJson(res, 404, { error: 'That transfer action is not available.' });
  }

  if (req.method === 'GET' && staticFiles.has(url.pathname)) {
    const file = staticFiles.get(url.pathname);
    res.writeHead(200, {
      'Content-Type': file.type,
      'Cache-Control': 'no-store',
      'X-Content-Type-Options': 'nosniff',
      'Referrer-Policy': 'no-referrer',
      'Content-Security-Policy': "default-src 'self'; img-src 'self'; style-src 'self'; script-src 'self'; connect-src 'self'; base-uri 'none'; frame-ancestors 'none'"
    });
    createReadStream(new URL(`./public/${file.path}`, import.meta.url)).pipe(res);
    return;
  }

  sendJson(res, 404, { error: 'Page not found.' });
});

server.headersTimeout = 15_000;
server.requestTimeout = 30 * 60 * 1000;
server.maxHeadersCount = 50;
server.once('error', async (error) => {
  console.error(`Handoff could not open port ${PORT}:`, error.message);
  await rm(sessionDirectory, { recursive: true, force: true });
  process.exit(1);
});

server.listen(PORT, '0.0.0.0', () => {
  const localUrl = `http://localhost:${PORT}/share/${SESSION_TOKEN}`;
  console.log(`Handoff is ready on this device: ${localUrl}`);
  if (lanAddresses.length === 0) {
    if (process.env.HANDOFF_HOST) {
      console.log(`For nearby devices, scan the QR code or open http://${primaryAddress}:${PORT}/share/${SESSION_TOKEN}`);
    } else {
      console.log('No local network address was found. Connect to Wi-Fi, then restart Handoff.');
    }
  } else {
    console.log(`For nearby devices, scan the QR code or open http://${primaryAddress}:${PORT}/share/${SESSION_TOKEN}`);
  }
  console.log('Close this process to end the room and clear its temporary files.');
});

let shuttingDown = false;
async function shutdown() {
  if (shuttingDown) return;
  shuttingDown = true;
  server.close();
  server.closeAllConnections();
  await rm(sessionDirectory, { recursive: true, force: true });
  process.exit(0);
}

process.once('SIGINT', shutdown);
process.once('SIGTERM', shutdown);

process.once('uncaughtException', async (error) => {
  console.error('Handoff stopped after an unexpected error:', error.message);
  await rm(sessionDirectory, { recursive: true, force: true });
  process.exit(1);
});
