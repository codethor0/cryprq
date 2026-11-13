# Docker portal guide

The two-device stack uses `Dockerfile.web`, the existing Rust CLI, React, Vite,
and Node.js. It does not require privileged mode, TUN devices, host networking,
or changes to the host's VPN, interfaces, routes, DNS, or firewall.

```bash
docker compose -p cryprq-portal -f docker-compose.portal.yml up --build -d
```

Open device A at `http://127.0.0.1:18787` and device B at
`http://127.0.0.1:18788`. Exchange and verify both pairing codes, pin both peers,
then send a synthetic file in each direction. Verify each received download.
If the default ports are occupied, set `CRYPRQ_PORTAL_A_PORT` and
`CRYPRQ_PORTAL_B_PORT` to two free loopback ports when starting the stack.

Each service has its own identity, trust store, receiver directory, and state
volume. Compose assigns resource names within the selected project. HTTP binds
inside the container with an explicit `HOST` override, while published web ports
bind only to host loopback. The UDP transfer path remains on the project Docker
network and has no host port publication. Pairing advertises each container's current IPv4 address.

```bash
docker compose -p cryprq-portal -f docker-compose.portal.yml ps
docker compose -p cryprq-portal -f docker-compose.portal.yml logs --tail=100
docker compose -p cryprq-portal -f docker-compose.portal.yml down
```

`down` preserves device state volumes. Recreate the containers and exchange
updated pairing codes if their network addresses change. Remove only this
project's volumes with `down -v` when deliberately discarding its test identities
and files. Never reuse another session's project name or remove unrelated Docker
resources.

`docker-compose.web.yml` starts one device. It publishes the web port (8787,
or `CRYPRQ_PORTAL_PORT`) and the UDP transfer port (9999, or
`CRYPRQ_PORTAL_UDP_PORT`) on host loopback, and advertises
`CRYPRQ_ADVERTISE_ADDR` in its pairing code. Use it when the other device runs
elsewhere on the same host; use the two-device stack for acceptance testing.
Check that the ports are free before starting it.

If the portal is unavailable, inspect its scoped logs and port ownership. If a
file does not arrive, check mutual pins, addresses, receiver readiness, file policy and transfer size. A successful sender exit is insufficient evidence.
The [validation guide](WEB_VALIDATION_RUN.md) describes browser-driven checks.
