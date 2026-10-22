const token = window.location.pathname.match(/^\/share\/([A-Za-z0-9_-]+)$/)?.[1];
const api = `/api/${token}`;

const qrCode = document.querySelector('#qr-code');
const qrLoading = document.querySelector('#qr-loading');
const roomLink = document.querySelector('#room-link');
const copyButton = document.querySelector('#copy-link');
const networkNote = document.querySelector('#network-note');
const filePicker = document.querySelector('#file-picker');
const chooseButton = document.querySelector('#choose-files');
const dropZone = document.querySelector('#drop-zone');
const fileList = document.querySelector('#file-list');
const fileCount = document.querySelector('#file-count');
const totalSize = document.querySelector('#total-size');
const transferStatus = document.querySelector('#transfer-status');
let uploadRunning = false;

const formatBytes = (bytes) => {
  if (bytes < 1024) return `${bytes} B`;
  const units = ['KB', 'MB', 'GB'];
  let value = bytes;
  let unit = -1;
  do {
    value /= 1024;
    unit += 1;
  } while (value >= 1024 && unit < units.length - 1);
  return `${value.toFixed(value >= 10 ? 0 : 1)} ${units[unit]}`;
};

function showStatus(message, kind = '') {
  transferStatus.textContent = message;
  transferStatus.className = `transfer-status${kind ? ` ${kind}` : ''}`;
}

function renderFiles(files) {
  fileList.replaceChildren();
  fileList.setAttribute('aria-busy', 'false');
  fileCount.textContent = `${files.length} ${files.length === 1 ? 'file' : 'files'} in this room`;

  if (files.length === 0) {
    const empty = document.createElement('p');
    empty.className = 'empty-state';
    empty.textContent = 'Nothing here yet. Add a file to begin.';
    fileList.append(empty);
    return;
  }

  for (const file of files) {
    const row = document.createElement('article');
    row.className = 'file-row';

    const marker = document.createElement('span');
    marker.className = 'file-marker';
    marker.setAttribute('aria-hidden', 'true');

    const details = document.createElement('div');
    details.className = 'file-details';
    const name = document.createElement('p');
    name.className = 'file-name';
    name.textContent = file.name;
    name.title = file.name;
    const meta = document.createElement('p');
    meta.className = 'file-meta';
    meta.textContent = `${formatBytes(file.size)} · ${new Date(file.createdAt).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}`;
    details.append(name, meta);

    const download = document.createElement('a');
    download.className = 'download-link';
    download.href = `${api}/files/${encodeURIComponent(file.id)}`;
    download.setAttribute('aria-label', `Download ${file.name}`);
    download.textContent = 'Download';

    row.append(marker, details, download);
    fileList.append(row);
  }
}

async function refreshFiles() {
  if (document.hidden || !token) return;
  try {
    const response = await fetch(`${api}/files`, { cache: 'no-store' });
    if (!response.ok) throw new Error('The room is no longer available.');
    const data = await response.json();
    renderFiles(data.files);
    totalSize.textContent = formatBytes(data.totalBytes);
  } catch {
    fileList.setAttribute('aria-busy', 'false');
    fileList.replaceChildren();
    const error = document.createElement('p');
    error.className = 'empty-state error-state';
    error.textContent = 'The room could not be reached. Check that Handoff is still open on the host.';
    fileList.append(error);
    fileCount.textContent = 'Room unavailable';
  }
}

async function loadSession() {
  if (!token) {
    networkNote.textContent = 'This room link is incomplete. Reopen Handoff on the host.';
    showStatus('The room link is invalid.', 'error');
    return;
  }

  try {
    const response = await fetch(`${api}/session`, { cache: 'no-store' });
    if (!response.ok) throw new Error('Room unavailable');
    const session = await response.json();
    roomLink.value = session.shareUrl || 'No local network address';
    copyButton.disabled = !session.hasLanAddress;
    if (session.hasLanAddress) {
      qrCode.src = `${api}/qr.svg`;
      qrCode.addEventListener('load', () => {
        qrLoading.hidden = true;
        qrCode.hidden = false;
      }, { once: true });
      qrCode.addEventListener('error', () => {
        qrCode.hidden = true;
        qrLoading.textContent = 'Room code unavailable';
      }, { once: true });
      networkNote.textContent = 'Use this link on the same Wi-Fi.';
    } else {
      qrLoading.textContent = 'Connect host to Wi-Fi';
      networkNote.textContent = 'No local network address was found. Connect the host to Wi-Fi and restart Handoff.';
    }
    await refreshFiles();
  } catch {
    networkNote.textContent = 'The room could not be reached. Check the host device.';
    showStatus('The room could not be reached.', 'error');
  }
}

async function sendFiles(files) {
  const selected = [...files];
  if (!selected.length || uploadRunning) return;

  uploadRunning = true;
  chooseButton.disabled = true;
  dropZone.setAttribute('aria-disabled', 'true');
  showStatus(`Preparing ${selected.length} ${selected.length === 1 ? 'file' : 'files'}…`);
  let completed = 0;

  for (const file of selected) {
    showStatus(`Sending ${file.name} (${completed + 1} of ${selected.length})…`);
    try {
      const response = await fetch(`${api}/files?name=${encodeURIComponent(file.name)}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/octet-stream' },
        body: file
      });
      const result = await response.json();
      if (!response.ok) throw new Error(result.error || 'The file could not be sent.');
      completed += 1;
    } catch (error) {
      showStatus(error.message || 'The transfer stopped. Check your connection and try again.', 'error');
      break;
    }
  }

  if (completed === selected.length) {
    showStatus(`${completed} ${completed === 1 ? 'file is' : 'files are'} ready in the room.`, 'success');
  } else if (completed > 0) {
    showStatus(`${completed} of ${selected.length} files sent. ${transferStatus.textContent}`, 'error');
  }
  filePicker.value = '';
  chooseButton.disabled = false;
  dropZone.removeAttribute('aria-disabled');
  uploadRunning = false;
  await refreshFiles();
}

chooseButton.addEventListener('click', () => filePicker.click());
filePicker.addEventListener('change', () => sendFiles(filePicker.files));

dropZone.addEventListener('click', (event) => {
  if (event.target.closest('button')) return;
  filePicker.click();
});
dropZone.addEventListener('dragover', (event) => {
  event.preventDefault();
  dropZone.classList.add('is-dragging');
});
dropZone.addEventListener('dragleave', (event) => {
  if (!dropZone.contains(event.relatedTarget)) dropZone.classList.remove('is-dragging');
});
dropZone.addEventListener('drop', (event) => {
  event.preventDefault();
  dropZone.classList.remove('is-dragging');
  sendFiles(event.dataTransfer.files);
});

copyButton.addEventListener('click', async () => {
  if (!roomLink.value || roomLink.value === 'Loading link…' || copyButton.disabled) return;
  try {
    await navigator.clipboard.writeText(roomLink.value);
    copyButton.textContent = 'Copied';
    window.setTimeout(() => { copyButton.textContent = 'Copy'; }, 1600);
  } catch {
    roomLink.focus();
    roomLink.select();
    showStatus('The link is selected. Copy it from your browser.', '');
  }
});

document.addEventListener('visibilitychange', () => {
  if (!document.hidden) refreshFiles();
});

loadSession();
window.setInterval(refreshFiles, 2500);
