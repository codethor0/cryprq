# Portal validation

Run against the exact feature revision being reviewed. Record the commit,
commands, exit codes, file sizes, original and downloaded SHA-256 values, and
hosted workflow URLs. A checklist is not a passing result.

## Real browser transfer

```bash
docker compose -p cryprq-portal-test -f docker-compose.portal.yml up --build -d
cd web
npm ci
npx playwright install chromium
npx playwright test
```

The browser suite opens two independent portals, copies public pairing codes,
explicitly pins peers, uploads actual files, waits for receiving rows, downloads
those files, and compares bytes, sizes, and SHA-256 in both directions. It must
not replace the CLI, intercept successful API responses, or infer delivery from
sender exit. The smoke spec checks the current portal interface.

```bash
cd ..
docker compose -p cryprq-portal-test -f docker-compose.portal.yml down -v
```

The adapter tests run the release CLI without Docker or a browser:

```bash
cargo build --release -p cryprq --locked
cd web && npm run test:adapter
```

## Required checks

| Area | Acceptance |
| --- | --- |
| CLI | Existing authenticated file-transfer script; distinct identities, mutual pins, wrong-identity and untrusted-peer rejection |
| Adapter | Real CLI integration; bounded binary uploads; unsafe-name, symlink, origin, and unauthorized-download rejection |
| Pairing | Concurrent and repeated pairing requests give one pin per identity and no duplicate trust-store entry |
| Recovery | Abort an upload and interrupt a receiver session, verify temporary cleanup, then successfully retry |
| Browser | Visible receiving row and verified downloaded bytes in both directions |
| Rust | Tests, formatting, Clippy, Cargo audit and deny |
| Web | Build, lint, adapter tests, Playwright, npm audits |
| Existing transport | Authenticated VPN, isolated Docker TUN, and legacy Docker transport gates remain required |

Host VPN changes are outside this browser task. Run VPN checks only within
isolated test processes or containers. Existing CI gates are preserved; inspect
the feature PR's actual workflow results before claiming they pass.

## Evidence limits

The protocol has no delivery acknowledgement or retransmission. Small successful
transfers do not prove delivery on lossy networks or arbitrary large-file
reliability. The portal serializes outgoing work and restarts the one-session
receiver. Its ready state is not an encryption claim. Receiver verification and
download verification are separate from sender completion.

The portal is local software with no remote account authentication. Passing
these tests does not establish production hosting suitability or an independent
security review.
