# Configuration

CrypRQ uses CLI flags and environment variables for runtime configuration.

## CLI Flags
| Flag | Description |
|------|-------------|
| `--listen <multiaddr>` | Start listener mode on the given multiaddr. |
| `--peer <multiaddr>` | Dial a remote peer (optionally include `/p2p/<peer-id>`). |

Exactly one flag must be provided.

## Environment Variables
| Variable | Default | Purpose |
|----------|---------|---------|
| `RUST_LOG` | `info` | Log verbosity (`error`, `warn`, `info`, `debug`, `trace`). |
| `CRYPRQ_ROTATE_SECS` | `300` | Key rotation interval in seconds. |
| `CRYPRQ_MAX_INBOUND` | `64` | Maximum concurrent inbound handshake attempts. |
| `CRYPRQ_BACKOFF_BASE_MS` | `500` | Initial backoff delay for repeated inbound failures (milliseconds). |
| `CRYPRQ_BACKOFF_MAX_MS` | `30000` | Upper bound for inbound backoff delay (milliseconds). |

## Key Rotation
- Background task rotates ML-KEM and X25519 secrets every `CRYPRQ_ROTATE_SECS`.
- Previous key material is overwritten in memory.
- Logs emit rotation events at `info` level.

## Peer Workflow
1. Listener prints `Local peer id`.
2. Share multiaddr (and peer ID if required) with remote nodes.
3. Dialer uses `--peer` to establish QUIC session.
4. Successful handshake shows `Connected to <peer-id>`.

## Logging
- Default `RUST_LOG=info`; set to `debug` for handshake traces.
- Docker containers inherit the environment and log to `docker logs`.

---

**Checklist**
- [ ] `--listen` or `--peer` provided (not both).
- [ ] `CRYPRQ_ROTATE_SECS` adjusted if default cadence is unsuitable.
- [ ] Logging configured per deployment requirements.

