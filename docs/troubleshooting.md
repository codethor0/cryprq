# Troubleshooting

## Listener Logs `Incoming connection error`
- Initial QUIC retries are expected. Persistent errors indicate firewall or NAT issues.
- Confirm UDP 9999 reachable and multiaddr matches network interface.

## Dialer Hangs
- Ensure multiaddr includes correct IP and optional `/p2p/<peer-id>` segment.
- Verify listener is running and logging `Listening on ...`.

## Dialer Reports `HandshakeTimedOut`
- UDP 9999 might be blocked—open the port bi-directionally between nodes.
- Confirm the multiaddr references the listener’s reachable IP (avoid `127.0.0.1` across hosts).
- Review any `tc netem` loss/latency settings; excessive impairment triggers timeouts.
- Ensure the listener is still running and logging new `Listening on ...` entries.
- See [Negative Tests](#negative-tests) for deliberate failure scenarios and recovery tips.

## Listener Drops Connections Immediately
- Logs show `event=inbound_backoff` or `event=inbound_rate_limit`.
- A peer exceeded the handshake concurrency cap (`CRYPRQ_MAX_INBOUND`) or triggered exponential backoff.
- Wait for the cooldown to expire or adjust the relevant environment variables for trusted peers.

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

## Negative Tests
- Use the chaos smoke workflow (`.github/workflows/qa-smoke.yml`) or `scripts/docker_vpn_test.sh` with loss/delay flags to reproduce expected `HandshakeTimedOut` behaviour before remediation.

---

**Checklist**
- [ ] Verified network reachability.
- [ ] Confirmed correct multiaddr and peer ID usage.
- [ ] Reran lint/test/audit suite after fixes.

