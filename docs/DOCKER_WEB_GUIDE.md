# CrypRQ v1.0.1 — Docker Web Deployment Guide

**File:** docs/DOCKER_WEB_GUIDE.md  
**Scope:** Web-only CrypRQ v1.0.1 stack (frontend + backend, test mode)  
**Audience:** Developers / operators running the web stack locally or in a lab.

> ⚠️ **Important:** This guide describes the **test-mode** web stack.  
> It is **not** a production deployment guide. See `SECURITY_NOTES.md` and `WEB_ONLY_RELEASE_NOTES_v1.0.1.md` for details.

---

## 1. Versioning and Deployment Strategy

**Tag Strategy:**
- `v1.0.1-web-preview` tag remains fixed at the original preview release point
- For preview/demo deployments, rebuild and deploy from `main` branch (latest code)
- Next release will be a new tag (e.g., `v1.1.0-web-preview` or `v1.0.2-web-preview`) when handshake+identity lands

**Rebuilding from Main:**
```bash
# Ensure you're on latest main
git checkout main
git pull origin main

# Rebuild Docker images
docker compose -f docker-compose.web.yml build

# (Re)start the web stack
docker compose -f docker-compose.web.yml up
```

**For Registry Deployment:**
```bash
# Build and tag as preview (without changing git tags)
docker build -t your-registry/cryprq-web:preview -f Dockerfile.web .
docker push your-registry/cryprq-web:preview

# Or tag as latest
docker build -t your-registry/cryprq-web:latest -f Dockerfile.web .
docker push your-registry/cryprq-web:latest
```

---

## 2. Overview

The web-only CrypRQ stack runs via Docker Compose and includes:

- A **backend** service:
  - Rust binary using the v1.0.1 record layer (20-byte header, epoch, nonce, HKDF).
  - UDP-based CrypRQ tunnel for file transfer.
- A **frontend** service:
  - React + TypeScript web UI.
  - Talks to the backend over HTTP/WebSocket/SSE (depending on implementation).

The entrypoint is `docker-compose.web.yml`.

---

## 3. Prerequisites

- **Docker** installed and running.
- **Docker Compose** plugin (usually included with recent Docker Desktop / Docker Engine).
- CrypRQ repository checked out locally.
- Optional but recommended: `cargo` if you want to build the binary outside Docker.

---

## 4. File Layout

Key files involved in the web deployment:

- `docker-compose.web.yml`  
  Defines the frontend and backend services for the web stack.
- `Dockerfile` / `Dockerfile.*`  
  Backend and/or frontend Docker build configuration (names may vary; check actual repo).
- `docs/WEB_ONLY_RELEASE_NOTES_v1.0.1.md`  
  Security posture and scope for this web-only release.
- `docs/WEB_VALIDATION_RUN.md`  
  Validation tracker for the web stack.

If service names differ from this guide (e.g., `web-backend` / `web-frontend`), update commands accordingly.

---

## 4. Basic Usage

### 4.1 Start the Web Stack

From the repo root:

```bash
docker compose -f docker-compose.web.yml up --build
```

This will:
- Build the backend image (if not already built).
- Build the frontend image.
- Start both containers with the configuration defined in `docker-compose.web.yml`.

You should see logs from both services in the same terminal.

To run in the background:

```bash
docker compose -f docker-compose.web.yml up --build -d
```

### 4.2 Stop the Web Stack

```bash
docker compose -f docker-compose.web.yml down
```

This stops and removes the containers (but not images or volumes).

---

## 6. Configuration

### 5.1 Ports

Typical (example) mapping — update this to match the actual compose file:

- **Frontend HTTP port:**
  - Host: `http://localhost:3000`
  - Container: `3000` (React dev server or static web server)
- **Backend HTTP/API port:**
  - Host: `http://localhost:8080`
  - Container: `8080`
- **Backend UDP port for CrypRQ tunnel:**
  - Host: `20440` (example used in CLI tests)
  - Container: `20440/udp`

Check `docker-compose.web.yml` and adjust the values here if they differ.

### 5.2 Environment Variables

Common environment variables (names may vary; adjust to your actual config):

- `CRYPRQ_WEB_BACKEND_PORT` — HTTP port the backend listens on inside the container.
- `CRYPRQ_UDP_BIND_ADDR` — UDP bind address, e.g. `0.0.0.0:20440`.
- `CRYPRQ_TEST_MODE` — When set (e.g. `1`), enables:
  - Static test keys.
  - No handshake / peer auth.
  - Test-mode key-direction hack.

If `.env` is used, it will typically be automatically loaded by Docker Compose. Check `docker-compose.web.yml` for `env_file:` lines.

---

## 7. Common Workflows

### 6.1 Fresh Rebuild

When you've changed backend or frontend code and want a clean rebuild:

```bash
docker compose -f docker-compose.web.yml build --no-cache
docker compose -f docker-compose.web.yml up
```

or in one line:

```bash
docker compose -f docker-compose.web.yml up --build
```

### 6.2 View Logs

All logs:

```bash
docker compose -f docker-compose.web.yml logs -f
```

Individual service logs (replace with actual service names):

```bash
docker compose -f docker-compose.web.yml logs -f web-backend
docker compose -f docker-compose.web.yml logs -f web-frontend
```

### 6.3 Shell into the Backend Container

Useful for debugging, checking files, or running `tcpdump` inside the container:

```bash
docker compose -f docker-compose.web.yml exec web-backend /bin/sh
```

(or `/bin/bash` if available.)

---

## 7. End-to-End Web Test (Short Version)

End-to-end detailed validation lives in `WEB_VALIDATION_RUN.md`.  
Here is the short "smoke test" variant.

1. Start the stack:
   ```bash
   docker compose -f docker-compose.web.yml up --build
   ```
2. Open the frontend in your browser:
   - `http://localhost:<frontend_port>`
   - (e.g. `http://localhost:3000`)
3. Use the UI to send a small test file (e.g., `test-web-minimal.bin`).
4. Confirm:
   - The UI shows the transfer starting and completing.
   - Backend logs show `FILE_META` / `FILE_CHUNK` activity.
   - The received file exists at the configured output location and matches the original via SHA-256.

For full steps and fields to capture, see `WEB_VALIDATION_RUN.md` (WEB-1).

---

## 8. Troubleshooting

### 8.1 Frontend Doesn't Load

- Check `docker-compose.web.yml` port mapping for the frontend.
- Run:
  ```bash
  docker compose -f docker-compose.web.yml ps
  docker compose -f docker-compose.web.yml logs -f web-frontend
  ```
- Confirm the frontend container is up and not crashing.

### 8.2 Backend 500 Errors / API Fails

- Check backend logs:
  ```bash
  docker compose -f docker-compose.web.yml logs -f web-backend
  ```
- Verify backend container is binding to the expected HTTP port and UDP address.
- Make sure any required environment variables are set (see section 5.2).

### 8.3 No File Appearing on Receiver Side

- Confirm UDP port mapping in the compose file:
  - Host UDP port matches what the backend expects.
- Check backend logs for:
  - Record decryption errors.
  - File transfer errors (e.g. invalid metadata).
- Make sure you're running in test mode and that the test-mode keys are consistent with docs.

### 8.4 Port Conflicts

If ports like `3000`, `8080`, or `20440` are already in use:

- Change the `ports:` mapping in `docker-compose.web.yml`.
- Restart:
  ```bash
  docker compose -f docker-compose.web.yml down
  docker compose -f docker-compose.web.yml up --build
  ```

---

## 9. Non-Production Disclaimer

This Docker setup is:

- ✅ For local testing, lab demos, and protocol exploration.
- ❌ Not hardened for production, multi-tenant, or hostile network environments.

Before any production deployment, you must:

- Implement the full handshake and peer authentication (see `MASTER_HANDSHAKE_AND_IDENTITY_PROMPT.md`).
- Remove static test keys and test-mode key-direction hacks.
- Revisit security controls (timeout, rate limiting, input validation, logging hygiene).

See:

- `SECURITY_NOTES.md`
- `WEB_ONLY_RELEASE_NOTES_v1.0.1.md`
- `PROTOCOL_SPEC_v1.0.1` (or equivalent)

