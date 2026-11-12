# CI gates for the authenticated VPN

This describes what CI runs for the VPN link and under which conditions.

Both gates are jobs in `.github/workflows/e2e-tests.yml`:
`Authenticated VPN E2E` and `VPN on TUN devices (containers)`. The job
definitions below are what the workflow contains. Check the workflow runs for
their results; the figures in this document are from developer machines.

The legacy Docker harness in the same workflow covers the old transport only;
what replaced its three retired protocol placeholders is in
[E2E_COVERAGE_MAP.md](E2E_COVERAGE_MAP.md).

## Gate 1: VPN end-to-end

```sh
cargo build --release -p cryprq --locked
bash scripts/e2e-authenticated-vpn.sh ./target/release/cryprq
bash scripts/e2e-authenticated-file-transfer.sh ./target/release/cryprq
```

- The exit status is the result: 0 only if every check passed (69 and 18).
- Needs bash, python3 (standard library only) and a loopback interface. No
  root, no TUN device, no network access, no containers. About 60 s and 10 s.
- `CRYPRQ_TEST_MODE` must be unset: the scripts check that the insecure test
  mode is refused.
- On failure the scripts keep their logs and print the directory. Upload it.
  The logs hold public keys and session ids, no secrets.

The job (shown in the workflow's own form in `e2e-tests.yml`):

```yaml
  vpn-e2e:
    name: Authenticated VPN E2E
    runs-on: ubuntu-latest
    timeout-minutes: 20
    steps:   # TMPDIR is set per step: the runner context does not exist at job level
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Build
        run: cargo build --release -p cryprq --locked
      - name: VPN link, two processes
        run: bash scripts/e2e-authenticated-vpn.sh ./target/release/cryprq
        env:
          CRYPRQ_E2E_KEEP_LOGS: "1"
      - name: File transfer, two processes
        run: bash scripts/e2e-authenticated-file-transfer.sh ./target/release/cryprq
      - name: Upload logs
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: vpn-e2e-logs
          path: ${{ runner.temp }}/cryprq-*e2e.*
```

What makes it a gate and not a decoration:

- No `continue-on-error`, no `|| true`, no `if:` on the test steps, no path
  filter that lets a change to `p2p/`, `node/`, `crypto/`, `core/` or `cli/`
  skip the job. If the script or the binary is missing the step fails, which
  is the intended result.
- The job is listed in the branch's required status checks. A job that runs
  but is not required does not block a merge.
- `cargo test --workspace --locked` keeps running the `p2p` crate. Its 36
  `secure_vpn` tests bind loopback UDP sockets and take about 17 s; they
  include the renewal and lost-CONFIRM cases.

The scripts create their log directories as `cryprq-vpn-e2e.*` and
`cryprq-e2e.*` under `TMPDIR`, which the job sets to the runner's temp
directory.

## Gate 2: TUN devices in containers

```sh
bash scripts/e2e-tun-docker.sh
```

What it needs from a runner:

- Linux with Docker and `/dev/net/tun` in the host kernel. GitHub's hosted
  `ubuntu-*` runners have both. The script checks for the device itself and
  fails if it is missing.
- Permission to start containers with `--cap-add NET_ADMIN --device
  /dev/net/tun`. Nothing else: no `--privileged`, no host network, no
  published ports, no mounts, no Docker socket inside a container.
- Network access during the image build only (base images, crates, Debian
  packages). The containers under test are on an `--internal` network.
- About 4 minutes with a cold build on an 8-core machine; allow 30.
- `git` and a checkout with history at `HEAD`: the image is built from
  `git archive HEAD`, so the job tests the commit, not the workspace.

What it must not be given:

- A self-hosted runner that is somebody's workstation or sits on a
  production network. The test is harmless there by construction, but a CI
  job that creates network devices belongs on a disposable machine.
- `sudo`. The script never asks for it.

What it has been run on: macOS with Docker Desktop (the containers run in
Docker's Linux VM), 71 checks, three runs at two commits. It has **not** been
run on a Linux host. There the script records the host's state with `ip`
instead of the macOS tools; that branch of the script is untested. On a Linux
host Docker's own bridge and veth devices appear while the test runs and are
gone after cleanup, so the before/after comparison should still hold, but
that is an expectation until the first run on a runner confirms it. If that
first run fails in the before/after comparison only, fix the comparison; do
not remove the job.

The job:

```yaml
  vpn-tun-containers:
    name: VPN on TUN devices (containers)
    runs-on: ubuntu-latest
    timeout-minutes: 30
    steps:
      - uses: actions/checkout@v4
      - name: Device test
        run: bash scripts/e2e-tun-docker.sh
        env:
          CRYPRQ_TUN_EVIDENCE: ${{ runner.temp }}/tun-evidence
      - name: Upload evidence
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: tun-evidence
          path: ${{ runner.temp }}/tun-evidence
```

The evidence directory holds logs, packet captures and the before/after
state. It never holds identity keys: they are created inside the containers
and destroyed with them, and the script fails if it finds one.

## Browser smoke test on another port

The workflow's `Web E2E Smoke` job starts the Docker web stack on port 8787.
Locally the same test runs without Docker and on any free port, which avoids
a port that something else holds:

```sh
(cd web && npm ci && npm run build && cd server && npm ci)
cargo build --release -p cryprq --locked
BRIDGE_PORT=18787 CRYPRQ_BIN="$PWD/target/release/cryprq" node web/server/server.mjs &
(cd web && CRYPRQ_WEB_BASE_URL=http://127.0.0.1:18787 npx playwright test tests/smoke.spec.ts)
```

The server port (`BRIDGE_PORT`) and the test's target (`CRYPRQ_WEB_BASE_URL`)
are both existing settings; no code change is involved. The test checks that
the page loads and the connect control is present. It says nothing about the
authenticated protocol.

## Required status checks

For a failing job to block a merge, the repository's branch protection for
`main` has to require it. The check names, as GitHub reports them:

- `Quick Check (fmt + clippy)`
- `Build and Test`
- `Security Checks`
- `Governance Checks`
- `Web E2E Smoke`
- `dependency-governance`
- `Legacy transport E2E (Docker)`
- `Authenticated VPN E2E`
- `VPN on TUN devices (containers)`

`Nightly Checks` is skipped on pull requests and must not be required.
`Security Checks` fails while the advisories in `docs/SECURITY_NOTES.md` are
open; requiring it blocks merges until they are decided, which is the
intended effect.

The `runner` context is only defined inside steps. A workflow that uses it in
a workflow-level or job-level key is rejected as a whole and creates no job;
`scripts/tests/test_e2e_gates.py` checks every workflow file for that.

## What neither gate covers

macOS utun devices, routes, DNS and leak resistance. `cryprq --vpn` installs
no routes and no DNS settings. No CI job should be named or described in a
way that suggests otherwise.
