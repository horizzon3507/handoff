# Handoff

Send files directly between devices on the same Wi-Fi network. No account or cloud storage required.

## Requirements

- Rust 1.85 or newer
- A computer and receiving devices on the same local network

## Run from source

```sh
cargo run --release
```

Open the **This device** address printed in the terminal. Scan its QR code from another device on the same Wi-Fi and open the room link. Leave the terminal running while transferring files.

To choose a different port, set `HANDOFF_PORT` (default: `4173`). If automatic network address discovery does not work, set `HANDOFF_HOST` to the computer's local IPv4 address.

## Send and receive files

Select one or more files in the room and choose **Share selected files**. They appear in the shared file list for every device using that room link. Choose **Download** on the receiving device; the browser saves each file in its usual downloads folder.

Files can also be sent in the other direction from the receiving device.

## Limits and privacy

Each file can be up to 512 MB. A room can hold up to 200 files and 2 GB total. Files are written to a private temporary directory on the host computer and removed when Handoff closes. The room link acts as its access key, so share it only with trusted devices.

## Troubleshooting

- Connect both devices to the same Wi-Fi. Guest networks may block devices from reaching one another.
- On a phone, scan the QR code or open the printed **Same Wi-Fi** link. `localhost` only works on the computer running Handoff.
- If the room link stops working after a restart, open the new QR code. Each run creates a new temporary room and link.

## Close a room

Press `Ctrl-C` in the terminal. The server stops and removes the temporary files.
