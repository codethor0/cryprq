# Operations

Guidance for monitoring, upgrading, and maintaining CrypRQ deployments.

## Observability
- Logs: stdout/stderr (structured). Use `RUST_LOG=debug` for handshake details.
- Key markers:
  - Listener: `Listening on ...`, `Ping event`.
  - Dialer: `Connected to ...`.
- Integrate logs with your aggregation platform via systemd or container runtime.

## Health Checks
- No native health endpoint yet.
- Recommend external check that attempts QUIC handshake (e.g., `./scripts/docker_vpn_test.sh`) on a schedule.

## Common Failure Modes
| Symptom | Likely Cause | Resolution |
|---------|--------------|------------|
| `Select(Failed)` transport errors | QUIC retry / port blocked | Ensure UDP 9999 reachable; verify firewall rules. |
| Dialer never connects | Incorrect multiaddr or missing `/p2p/<id>` | Use listener’s logged address and peer ID. |
| Excessive logs | `RUST_LOG` set to `debug` in production | Reduce to `info` after troubleshooting. |

## Upgrades
1. Pull latest `main`.
2. Run lint/test/audit suite.
3. Rebuild binaries or Docker image.
4. Deploy via rolling restart (systemd reload / container update).
5. Monitor logs for handshake success.

## Rollback
- Retain prior binary or image.
- Repoint systemd service or container tag to previous version.
- Re-run smoke test to confirm handshake succeeds.

---

**Checklist**
- [ ] Logs monitored for handshake success and rotation events.
- [ ] Upgrade procedure includes lint/test/audit suite.
- [ ] Rollback plan documented and tested.

