# Quickstart

Getting CrypRQ running locally in listener/dialer mode.

## Prerequisites
- Rust toolchain 1.83.0 (`rustup toolchain install 1.83.0`)
- Git
- Optional: Docker for container smoke tests

## Build
```bash
git clone https://github.com/codethor0/cryprq.git
cd cryprq
rustup override set 1.83.0
cargo build --release -p cryprq
```

## Start a Listener
```bash
./target/release/cryprq \
  --listen /ip4/0.0.0.0/udp/9999/quic-v1 \
  --allow-peer 12D3KooExamplePeerID \
  --metrics-addr 127.0.0.1:9464
## Logs include:
## Local peer id: 12D3Koo...
## Listening on /ip4/0.0.0.0/udp/9999/quic-v1
```

## Dial the Listener
```bash
./target/release/cryprq --peer /ip4/127.0.0.1/udp/9999/quic-v1
## Expect: Connected to <peer-id>
```

## Verify
- Listener prints ping events after handshake.
- `RUST_LOG=debug` shows rotation events every 300 seconds.
- `curl http://127.0.0.1:9464/metrics` returns rotation counters and handshake stats.
- `curl http://127.0.0.1:9464/healthz` returns `ok` once the swarm is ready.

---

**Checklist**
- [ ] Built CLI with Rust 1.83.0.
- [ ] Listener produced a peer ID and listened on desired port.
- [ ] Dialer connected and logged success.

