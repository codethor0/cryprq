# CrypRQ File Transfer Guide

## Receiver hardening checkpoint — November 10, 2026

This branch hardens the receiver independently of the authenticated transport.
The historical CLI examples below require verification on the final combined
protocol branch; receiver unit tests alone do not prove authenticated delivery.

Incoming transfers use random, exclusively created private temporary files
(mode 0600 on Unix). Only plain filenames are accepted. The receiver checks the
announced size and SHA-256, syncs the data, and publishes using a same-directory
hard link that fails if the destination already exists, including a symlink.
Corrupt or oversized data is never published as a completed file.

Limits are 1 GiB per file, 64 active transfers, and 2 GiB of total announced
active reservations, enforced together under the receiver state lock.
Only nonempty byte progress refreshes the 120-second inactivity deadline.
A single worker per manager, started on first admission, checks inactivity
every second even when no further receiver operation occurs. Admission fails
before creating a temporary file if that worker cannot start. Manager shutdown
wakes and joins the worker rather than waiting for the next polling interval.

If the filesystem refuses temporary-file deletion, the receiver closes the
file, quarantines its state, retains its slot and full reservation, rejects
further writes, and retries cleanup automatically. Persistent failure at final
shutdown logs the exact temporary path for operator cleanup. Filesystem calls
that themselves hang have no hard cancellation bound. Successful verified
files are outside the active reservation quota: completed-file storage and
per-peer quotas remain separate deployment requirements.

Validation covers 26 receiver tests, including idle cleanup without traffic,
independent progress, concurrent admission, worker startup failure, shutdown,
and failed-unlink quarantine and retry. The broader node library has 66 tests.

## Overview

CrypRQ supports encrypted file transfer over the post-quantum encrypted tunnel. Files are transferred in chunks with SHA-256 hash verification to ensure integrity.

## Protocol

The file transfer protocol uses three packet types:

1. **Metadata Packet (type 0)**: Contains filename, file size, and SHA-256 hash
2. **Data Packet (type 1)**: Contains chunk ID and chunk data (64KB chunks)
3. **End Packet (type 2)**: Signals end of transfer

All packets are encrypted using the same ML-KEM + X25519 encryption as the VPN tunnel.

## Usage

### Receiving Files

Start a listener that will receive files:

```bash
RUST_LOG=info ./target/release/cryprq receive-file \
  --listen /ip4/0.0.0.0/udp/9999/quic-v1 \
  --output-dir ./received
```

The listener will:
- Accept connections from peers
- Receive file metadata, chunks, and end packet
- Write the file to the output directory
- Verify the SHA-256 hash matches
- Log success or failure

### Sending Files

Send a file to a peer:

```bash
RUST_LOG=info ./target/release/cryprq send-file \
  --peer /ip4/127.0.0.1/udp/9999/quic-v1 \
  --file sample.txt
```

The sender will:
- Connect to the peer
- Calculate file hash
- Send metadata packet
- Send file in 64KB chunks
- Send end packet
- Log completion

## Example Workflow

### Step 1: Start Receiver

```bash
# Terminal 1
cd cryprq
RUST_LOG=info ./target/release/cryprq receive-file \
  --listen /ip4/0.0.0.0/udp/9999/quic-v1 \
  --output-dir ./received_files
```

Expected output:
```
INFO  cryprq::p2p event=listener_starting peer_id=... listen_addr=/ip4/0.0.0.0/udp/9999/quic-v1
Local peer id: 12D3KooW...
INFO  cryprq::p2p event=listener_ready peer_id=... listen_addr=/ip4/0.0.0.0/udp/9999/quic-v1
Listening on /ip4/0.0.0.0/udp/9999/quic-v1
```

### Step 2: Create Test File

```bash
# Terminal 2
cd cryprq
head -c 1M </dev/urandom > sample.bin
shasum -a 256 sample.bin
```

### Step 3: Send File

```bash
# Terminal 2 (continued)
RUST_LOG=info ./target/release/cryprq send-file \
  --peer /ip4/127.0.0.1/udp/9999/quic-v1 \
  --file sample.bin
```

Expected output:
```
INFO  cryprq::p2p event=dialer_starting peer_id=... target_addr=/ip4/127.0.0.1/udp/9999/quic-v1
Local peer id: 12D3KooW...
INFO  cryprq::p2p event=handshake_complete peer_id=...
INFO  cryprq::p2p event=connection_established peer_id=...
INFO  cryprq::p2p Sent file metadata to ...: sample.bin (1048576 bytes)
INFO  cryprq::p2p Sent chunk 0 to ... (65536 bytes)
...
INFO  cryprq::p2p Sent end packet to ...
INFO  cryprq File transfer completed successfully
```

### Step 4: Verify Received File

```bash
# Terminal 1 (receiver should show)
INFO  cryprq Receiving file: sample.bin (1048576 bytes) from peer ...
INFO  cryprq File received successfully: sample.bin (1048576 bytes) from peer ...

# Verify hash matches
cd received_files
shasum -a 256 sample.bin
# Should match the hash from Terminal 2
```

## Log Events

### Sender Logs

- `event=dialer_starting` - Starting connection
- `event=handshake_complete` - ML-KEM + X25519 handshake done
- `event=connection_established` - Encrypted tunnel ready
- `Sent file metadata to...` - Metadata sent
- `Sent chunk N to...` - Chunk sent
- `Sent end packet to...` - Transfer complete
- `File transfer completed successfully` - Success

### Receiver Logs

- `event=listener_starting` - Listener starting
- `event=listener_ready` - Ready for connections
- `Receiving file:...` - File transfer started
- `Received chunk N from peer...` - Chunk received
- `File received successfully:...` - Transfer complete, hash verified

## Error Handling

### Hash Mismatch

If the received file's hash doesn't match the metadata:
- The file is deleted
- An error is returned to the sender
- Log shows: `File hash mismatch`

### Missing Chunks

If chunks are missing when the end packet arrives:
- Error returned: `Missing chunk N`
- File is not written

### Connection Errors

If connection fails:
- Sender: `Failed to connect:...`
- Receiver: Connection errors logged

## Security

- All file data is encrypted using ML-KEM + X25519 hybrid encryption
- SHA-256 hash verification ensures file integrity
- Chunks are sent over the encrypted tunnel (same as VPN packets)
- No file data is exposed in plaintext

## Limitations

- Maximum chunk size: 64KB
- Maximum file size: Limited by available memory (chunks stored in memory during transfer)
- Single file per connection (for now)
- Protocol selection: Uses same request-response protocol as packet forwarding (detected by packet type)

## Troubleshooting

### File not received

- Check that receiver is listening on correct address
- Verify network connectivity
- Check firewall rules
- Look for connection errors in logs

### Hash mismatch

- Check for network errors during transfer
- Verify both peers are using compatible versions
- Check logs for missing chunks

### Connection fails

- Verify addresses are correct (multiaddr format)
- Check that listener is running
- Verify UDP port is not blocked by firewall

