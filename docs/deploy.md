# Deploy

CrypRQ supports bare-metal, Docker, and Nix-based deployments. Choose the approach that aligns with your infrastructure and security posture.

## Port Matrix
| Purpose | Protocol | Default |
|---------|----------|---------|
| QUIC handshake | UDP | 9999 |
| TCP fallback | TCP | 9999 |

Allow inbound traffic on these ports between peers.

## Bare Metal (Linux/macOS)
1. Install Rust 1.83.0 and build `cryprq`.
2. Create a service account (optional but recommended).
3. Place binary in `/usr/local/bin`.

Systemd unit:
```ini
[Unit]
Description=CrypRQ Listener
After=network-online.target
Wants=network-online.target

[Service]
User=cryprq
Group=cryprq
Environment=RUST_LOG=info
ExecStart=/usr/local/bin/cryprq --listen /ip4/0.0.0.0/udp/9999/quic-v1
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

## Docker
```bash
docker build -t cryprq-node .
docker run -d --name cryprq-listener \
  -p 9999:9999/udp \
  -e RUST_LOG=info \
  cryprq-node --listen /ip4/0.0.0.0/udp/9999/quic-v1
```

Docker Compose:
```yaml
services:
  listener:
    image: cryprq-node:latest
    build: .
    command: ["--listen", "/ip4/0.0.0.0/udp/9999/quic-v1"]
    environment:
      RUST_LOG: info
    ports:
      - "9999:9999/udp"
    restart: unless-stopped
```

## Nix
```bash
nix build
./result/bin/cryprq --listen /ip4/0.0.0.0/udp/9999/quic-v1
```

## Cloud Tips
- Restrict inbound peers via security groups/firewalls.
- Use encrypted disks for log storage (future data-plane).
- Maintain monotonic clocks; rotation relies on accurate timers.

---

**Checklist**
- [ ] Firewall allows UDP/TCP 9999 between peers.
- [ ] Deployment method chosen (bare metal, Docker, or Nix).
- [ ] Service supervised (systemd, container orchestrator, etc.).

