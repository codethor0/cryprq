# File transfer test plan

File transfer runs over the authenticated handshake: both devices have an
Ed25519 identity, each pins the other's public key, and the receiver verifies
the announced size and SHA-256 before it publishes a file. The protocol has no
delivery acknowledgement and no retransmission, so every test ends by comparing
the received file with the original.

## CLI

```bash
cargo build --release -p cryprq --locked
bash scripts/e2e-authenticated-file-transfer.sh ./target/release/cryprq
```

The script uses two identities on loopback. It covers a successful transfer
with a matching SHA-256, an untrusted sender, a wrong pinned identity, a
receiver with an empty trust store, and the refusal of test mode. The manual
commands are in the [README](../README.md).

## Adapter

```bash
cd web
npm run test:adapter
```

Two adapters with separate state run the release binary. The tests cover
transfers in both directions with downloads compared byte for byte, concurrent
and repeated pairing, invalid and conflicting pins, unsafe filenames, oversized
and chunked uploads, interrupted uploads and transfers with cleanup and retry,
an unpinned sender, a wrong identity, unauthorized requests and tampered inbox
files.

## Browser

Two portals in containers, driven by Playwright. See
[Portal validation](WEB_VALIDATION_RUN.md).

## What to record

- The commit under test and the commands run.
- File size and SHA-256 of the original, of the receiver's listed file and of
  the downloaded file.
- For each rejection test, that no file was published on the receiver.

## Limits

- One receiver process serves one session; the portal restarts its receiver
  between sessions.
- Transfers were tested on loopback and on a Docker bridge network. Lossy
  networks and large files are not covered.
- A sender that exits successfully has not proven delivery.
