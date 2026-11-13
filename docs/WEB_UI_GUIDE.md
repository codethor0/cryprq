# Browser portal

The portal sends files through the authenticated Rust CLI. Each running portal
owns one identity, an explicit peer trust store, a receiver, and private storage.
The browser has no private identity key and implements no transfer cryptography.

## Pair devices

1. Open two independently running portals. Two tabs on the same portal share one device.
2. Copy the pairing code from device A to device B.
3. Compare the public key with device A over a trusted channel. An address alone does not establish identity.
4. Name the peer and pin it explicitly.
5. Repeat in the other direction. Both devices must trust each other.

Pairing codes contain a version, an Ed25519 public key, and an IPv4 UDP address.
They contain no private key. Do not accept a pairing code from an unverified source.
Removing a peer removes its pin and restarts the receiver.

Pairing requests are applied one at a time. Pinning an identity that is already
pinned at the same address changes nothing and reports the existing pin. The
same identity at another address is refused until the old pin is removed.
Pinning and removal are refused while a transfer is active.

## Send and receive

Select a pinned destination, choose or drop a file, then select **Send file**.
The portal uploads the file as bounded binary data to its local adapter. The
adapter invokes `send-file` with explicit identity, trust, address, and peer-key
arguments. Authentication is reported only after the CLI confirms the handshake.

**Sent. Check the receiving device** means transmission ended. The protocol has
no application-level delivery acknowledgement or retransmission. It does not
mean the receiver published the file.

The receiving CLI checks the announced size and SHA-256 before atomic publication.
The adapter independently opens and hashes the published regular file before
listing it. Download from **Receive**, then compare the download with the original.
The browser also checks downloaded bytes against the listed size and SHA-256.
A failed or interrupted transfer must be retried. Each published file has a unique
storage ID. Repeated filenames create separate inbox entries and preserve earlier files.

The receiver restarts between sessions and after a bounded stalled session.
Wait for **Ready to receive** before retrying. **Ready** means listening; it does
not mean a peer has completed authentication.

## Diagnostics

Open **Diagnostics** for receiver state, public addresses, and transfer hashes.
The default screen contains pairing, send, transfer status, and received files.
There are no browser VPN controls or host network changes.

## Local API

`GET /api/session` returns an in-memory portal token. Other API requests require
`X-Portal-Token`. Keep this token out of URLs and logs. The adapter checks Host,
Origin, and cross-site request metadata, and sends no permissive CORS headers.

| Request | Purpose |
| --- | --- |
| `GET /api/state` | Public identity, pairing code, peers, receiver state, transfers, verified inbox, limits |
| `POST /api/peers` | Explicitly pin `{name, pairing: {version: 1, key, address}}`. `201` for a new pin, `200` if that pin already exists, `409` for a conflicting address or an active transfer |
| `DELETE /api/peers/:id` | Remove a peer pin |
| `POST /api/send?peer=:id` | Binary upload with `Content-Type: application/octet-stream` and URI-encoded `X-Filename` |
| `GET /api/files/:id` | Download an indexed verified file using the token header |

Uploads are streamed into private temporary storage. Downloads use opaque IDs,
never caller-supplied filesystem paths. The adapter rejects unsafe names,
symlinks, oversized uploads, unknown peers, and missing tokens.
