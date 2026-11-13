# Local portal setup

Use [Docker](DOCKER_WEB_GUIDE.md) for two isolated devices. To develop locally:

```bash
cargo build --release -p cryprq --locked
cd web
npm ci
npm run build
cd ..
```

Run each command below in its own terminal from the repository root. Use two
new private state directories. Do not point both processes at the same directory.

```bash
PORT=18787 CRYPRQ_STATE_DIR="$PWD/.portal-a" CRYPRQ_LISTEN_ADDR=/ip4/127.0.0.1/udp/20441 CRYPRQ_BIN="$PWD/target/release/cryprq" node web/server/server.mjs
```

```bash
PORT=18788 CRYPRQ_STATE_DIR="$PWD/.portal-b" CRYPRQ_LISTEN_ADDR=/ip4/127.0.0.1/udp/20442 CRYPRQ_BIN="$PWD/target/release/cryprq" node web/server/server.mjs
```

Open `http://127.0.0.1:18787` and `http://127.0.0.1:18788`, then follow
[Pair devices](WEB_UI_GUIDE.md#pair-devices). The adapter defaults to loopback.
Each process generates its identity only if none exists. Existing identity and
trust failures stop startup instead of resetting state.

| Variable | Purpose |
| --- | --- |
| `PORT` | Local HTTP port |
| `HOST` | HTTP bind address; default `127.0.0.1` |
| `CRYPRQ_BIN` | Path to the authenticated CLI |
| `CRYPRQ_STATE_DIR` | Exclusive private device state |
| `CRYPRQ_LISTEN_ADDR` | Receiver IPv4 UDP multiaddress |
| `CRYPRQ_ADVERTISE_ADDR` | Reachable peer address; defaults to listen address |
| `CRYPRQ_MAX_FILE_BYTES` | Upload and published-inbox size limit; default 64 MiB |
| `CRYPRQ_RECEIVER_IDLE_MS` | Stalled authenticated-session timeout; default 15,000 ms |

For frontend development, build and restart the adapter after changes. The
production build and API share one origin. No server dependency installation is
needed when the adapter uses only Node's standard library.

See [portal validation](WEB_VALIDATION_RUN.md) for checks and
[security boundaries](SECURITY_NOTES.md#browser-portal-boundary) for limitations.
