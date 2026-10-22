# Handoff

Send files between devices on the same Wi-Fi network. No account or cloud storage required.

## Getting started

Requires Node.js 22 or newer.

```sh
npm install
npm start
```

Open <http://localhost:4173> on the computer running Handoff. Scan the QR code with another device on the same Wi-Fi, then open the link.

## Send a file

Choose **Choose files** on either device and select one or more files. They appear under **Shared files** for everyone in the room. Choose **Download** on the receiving device; the browser saves it in its usual downloads folder.

To send in the other direction, choose files on the other device and download them from the first.

## If the QR code does not work

- Connect both devices to the same Wi-Fi. Guest networks may block devices from reaching one another.
- Keep the terminal running while transferring files.
- If Handoff was restarted, refresh the page on the computer and scan the new QR code. The previous room link expires when the server stops.
- On the phone, scan the QR code instead of typing `localhost`; that address only works on the computer itself.

## Close a room

Press `Ctrl+C` in the terminal running Handoff. The room closes and temporary files are removed. Start a new room and use its new QR code next time.

## Limits and privacy

Files are stored temporarily on the computer running Handoff and are transferred over the local network. Each file can be up to 512 MB, with a 2 GB total limit per room.
