# Align Web Backend with CrypRQ v1.0.1 Record Layer

**Summary**

Refactor the CrypRQ web backend so that *all* network traffic (VPN packets, file transfer, generic data) flows through the v1.0.1-compliant record layer and `Tunnel` APIs, matching the already-validated CLI behavior.

**Context**

* v1.0.1 spec is the source of truth:
  * 20-byte record header
  * Epoch = `u8` (mod 256)
  * TLS 1.3–style nonce (static IV XOR seq)
  * HKDF key schedule with labeled derivation
* The CLI file transfer path is already aligned and validated end-to-end using the record layer.
* The web backend must no longer depend on libp2p request-response framing or legacy headers.

**Tasks**

* [ ] Create/update `docs/WEB_STACK_MAP.md`:
  * [ ] Map web backend entrypoints (e.g., `/connect`, `/events`, `/api/send-file`).
  * [ ] Document which modules handle sessions/tunnels.
  * [ ] Note any remaining libp2p / legacy protocol usage.
* [ ] Identify all backend paths that send/receive network data for:
  * [ ] VPN/TUN packets.
  * [ ] File transfer.
  * [ ] Generic DATA/control messages (if applicable).
* [ ] Refactor backend to use the v1.0.1 stack consistently:
  * [ ] All outbound data goes through `Tunnel` + record layer APIs:
    - `Tunnel::send_vpn_packet()`
    - `Tunnel::send_record()` / file-transfer helpers.
  * [ ] All inbound data is processed via `Tunnel::recv_and_handle_record()` / `Tunnel::handle_incoming_record()`.
  * [ ] Remove or bypass any libp2p request-response framing still in use for web.
* [ ] Add/update Rust docs on key structs/functions to make it clear:
  * [ ] Web backend is now a "thin adapter" over `Tunnel` + record layer.
* [ ] Update `docs/PROTOCOL_ALIGNMENT_REPORT.md`:
  * [ ] Add a "Web backend alignment" section describing:
    - Pre-refactor vs post-refactor state.
    - Confirmation that web traffic uses:
    - 20-byte header
    - `u8` epoch
    - TLS 1.3–style nonce
    - Epoch-scoped HKDF keys.

**Acceptance Criteria**

* [ ] No web backend code constructs its own non-spec header or nonce.
* [ ] All web-originated network traffic flows through the same record layer as the CLI.
* [ ] `PROTOCOL_ALIGNMENT_REPORT.md` explicitly states that the web backend is aligned with v1.0.1.

**Related**

* See `docs/MASTER_WEB_RELEASE_PROMPT.md` Section 2 for detailed guidance.
* CLI reference implementation: `cli/src/main.rs` (validated and working).

