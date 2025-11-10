# Operations

Guidance for monitoring, upgrading, and maintaining CrypRQ deployments.

## Observability
- Logs: stdout/stderr (structured). Use `RUST_LOG=debug` for handshake details.
- Prometheus: start the CLI with `--metrics-addr 127.0.0.1:9464` (or `CRYPRQ_METRICS_ADDR`). Metrics include `rotations_total`, `handshakes_*`, and `current_peers`.
- Health: `/healthz` returns `200 OK` once the swarm is initialised.
- Integrate logs and metrics with your aggregation platform via systemd or container runtime.

## Health Checks
- `curl http://<host>:9464/healthz` for liveness.
- `./scripts/docker_vpn_test.sh` or `./scripts/sweep.sh --scenario baseline-docker-handshake` for end-to-end QUIC verification.

## Common Failure Modes
| Symptom | Likely Cause | Resolution |
|---------|--------------|------------|
| `Select(Failed)` transport errors | QUIC retry / port blocked | Ensure UDP 9999 reachable; verify firewall rules. |
| Dialer denied with `event=peer_denied` | Peer not allowlisted | Add the peer ID via `--allow-peer` or `CRYPRQ_ALLOW_PEERS`. |
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

## Release Security Artifacts
- `./finish_qa_and_package.sh` collects QA logs, extracts multi-arch binaries, and writes checksums.
- The script now also generates:
  - `release-*/security/sbom-<version>.spdx.json` (Syft, SPDX).
  - `release-*/security/grype-<version>.txt` (vulnerability scan; defaults to `--fail-on critical`).
- Adjust `SYFT_IMAGE`, `GRYPE_IMAGE`, or `GRYPE_FAIL_LEVEL` environment variables to customise tooling.

---

**Checklist**
- [ ] Logs monitored for handshake success and rotation events.
- [ ] Upgrade procedure includes lint/test/audit suite.
- [ ] Rollback plan documented and tested.

