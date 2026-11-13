# CrypRQ browser portal

A local web interface for file transfer between explicitly trusted devices. The
React front end talks to a Node.js adapter (`server/server.mjs`), which runs the
authenticated `cryprq` CLI. The browser holds no private key and implements no
transfer cryptography.

## Build and run

```bash
cargo build --release -p cryprq --locked   # from the repository root
cd web
npm ci
npm run build
PORT=18787 CRYPRQ_STATE_DIR="$PWD/../.portal-a" \
  CRYPRQ_BIN="$PWD/../target/release/cryprq" node server/server.mjs
```

Open `http://127.0.0.1:18787`. A second device needs its own port, state
directory and `CRYPRQ_LISTEN_ADDR`; see
[local setup](../docs/WEB_STACK_QUICK_START.md) or run two devices with
[Docker](../docs/DOCKER_WEB_GUIDE.md).

## Tests

```bash
npm run lint
npm run test:adapter    # adapter against the real CLI (needs the release binary)
npm run test:e2e        # Playwright, against two running portals
```

`test:e2e` reads `CRYPRQ_WEB_BASE_URL` and `CRYPRQ_WEB_PEER_URL` and defaults to
ports 18787 and 18788. See [portal validation](../docs/WEB_VALIDATION_RUN.md).

## Layout

| Path | Content |
| --- | --- |
| `src/App.tsx` | The interface: pairing, send, received files, diagnostics |
| `server/server.mjs` | Adapter: HTTP API, CLI processes, private state, inbox |
| `server/portal.test.mjs` | Adapter tests |
| `tests/` | Playwright tests |

Usage and limits are in the [Web UI guide](../docs/WEB_UI_GUIDE.md) and the
[Security Notes](../docs/SECURITY_NOTES.md#browser-portal-boundary).
