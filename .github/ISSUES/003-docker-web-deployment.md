# Web-Only Docker Deployment & Documentation

**Summary**

Finalize the web-only deployment path using Docker and docker-compose, and document a "one-command" flow to bring up the CrypRQ web stack in test mode.

**Context**

* `docker-compose.web.yml` (and related Dockerfiles) are the primary entrypoint for web-only deployment.
* We already have working binaries and test-mode keys; this is about making the deployment reproducible and documented.

**Tasks**

* [ ] Audit `docker-compose.web.yml` and web-related Dockerfiles:
  * [ ] Confirm backend container:
    - Exposes/uses correct UDP ports.
    - Has env vars/config for test-mode keys and listen address.
  * [ ] Confirm frontend container:
    - Can reach backend on the internal Docker network.
    - Serves the built web assets (React/TS).
* [ ] Ensure standard startup for test mode:
  * [ ] Document or configure backend command, e.g.:
    - `cryprq web-server --mode test --listen 0.0.0.0:PORT ...`
  * [ ] Confirm `docker compose -f docker-compose.web.yml up --build` is sufficient to start the stack.
* [ ] Create or update `docs/DOCKER_WEB_GUIDE.md`:
  * [ ] Prerequisites (Docker, docker-compose).
  * [ ] Exact command(s) to start the web-only stack.
  * [ ] Default frontend URL (e.g. `http://localhost:3000`).
  * [ ] Expected behavior: open UI, run a test file transfer.
* [ ] Update `README.md`:
  * [ ] Add a "Web-Only Mode (Test Environment)" section.
  * [ ] Link to `DOCKER_WEB_GUIDE.md`.

**Acceptance Criteria**

* [ ] A fresh user can:
  * [ ] Run `docker compose -f docker-compose.web.yml up --build`.
  * [ ] Open the web UI.
  * [ ] Perform an end-to-end file transfer using the test stack.
* [ ] `README.md` and `DOCKER_WEB_GUIDE.md` are accurate and up to date.

**Related**

* See `docs/MASTER_WEB_RELEASE_PROMPT.md` Section 4 for detailed guidance.

