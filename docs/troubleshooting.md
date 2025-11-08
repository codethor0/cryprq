# Troubleshooting

## Listener Logs `Incoming connection error`
- Initial QUIC retries are expected. Persistent errors indicate firewall or NAT issues.
- Confirm UDP 9999 reachable and multiaddr matches network interface.

## Dialer Hangs
- Ensure multiaddr includes correct IP and optional `/p2p/<peer-id>` segment.
- Verify listener is running and logging `Listening on ...`.

## Docker Build Fails (`if-watch` missing)
- Update to latest `main`; Dockerfile copies `third_party/if-watch`.
- Ensure repository cloned with subdirectories intact.

## `cargo audit` or `cargo deny` Fails
- Update dependencies flagged by advisories.
- Check `deny.toml` for allowed licenses and adjust if necessary.

## High CPU Usage
- Production builds should use `cargo build --release`.
- Reduce log verbosity (`RUST_LOG=info`).

## Key Rotation Stops
- Look for panics or `RUST_LOG=debug` output.
- Restart service to resume rotation.

## Docker Smoke Test Fails
- Run `./scripts/docker_vpn_test.sh` locally; inspect listener/dialer logs.
- Ensure Docker network allows container-to-container UDP.

---

**Checklist**
- [ ] Verified network reachability.
- [ ] Confirmed correct multiaddr and peer ID usage.
- [ ] Reran lint/test/audit suite after fixes.

