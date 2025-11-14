# 🐳 CrypRQ Docker VPN - Quick Start

## Prerequisites

1. **Docker Desktop** must be running
   - Download: https://www.docker.com/products/docker-desktop
   - Start Docker Desktop application
   - Verify: `docker ps` should work

2. **CrypRQ binary** built (for Mac-to-container connection)
   ```bash
   cargo build --release -p cryprq
   ```

## 🚀 Quick Start (3 Commands)

### Option 1: Start Everything at Once

```bash
./scripts/start-full-docker-vpn.sh
```

This will:
- ✅ Check Docker is running
- ✅ Start Docker container
- ✅ Start web server with Docker mode
- ✅ Show connection info

### Option 2: Start Individually

```bash
# 1. Start Docker container
./scripts/docker-vpn-start.sh

# 2. Start web server (in another terminal)
./scripts/start-web-docker.sh

# 3. Open browser
open http://localhost:8787
```

## ✅ Check Status

```bash
./scripts/check-docker-vpn-status.sh
```

## 🔗 Connect

### Via Web UI

1. Open browser: `http://localhost:8787`
2. Select mode: **listener** (container is listener)
3. Click **Connect**
4. Container will show as listening
5. In another tab, select mode: **dialer**
6. Click **Connect** to connect Mac to container

### Via CLI

```bash
# Get container IP
CONTAINER_IP=$(docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' cryprq-vpn)

# Connect Mac to container
./target/release/cryprq --peer /ip4/$CONTAINER_IP/udp/9999/quic-v1
```

## 📊 View Logs

```bash
# Container logs
docker-compose -f docker-compose.vpn.yml logs -f

# Or directly
docker logs -f cryprq-vpn
```

## 🛑 Stop Everything

```bash
# Stop container
./scripts/docker-vpn-stop.sh

# Stop web server
# Press Ctrl+C in the terminal running the web server
```

## 🐛 Troubleshooting

### Docker not running
```bash
# Start Docker Desktop, then:
docker ps
```

### Container won't start
```bash
# Check logs
docker-compose -f docker-compose.vpn.yml logs

# Rebuild
docker-compose -f docker-compose.vpn.yml build --no-cache
docker-compose -f docker-compose.vpn.yml up -d
```

### Port 8787 already in use
```bash
# Find what's using it
lsof -ti:8787

# Kill it
kill $(lsof -ti:8787)

# Or use different port
export BRIDGE_PORT=8788
./scripts/start-web-docker.sh
```

### Can't connect to container
```bash
# Get container IP
docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' cryprq-vpn

# Test connectivity
ping <CONTAINER_IP>

# Check container is listening
docker exec cryprq-vpn netstat -uln | grep 9999
```

## 📚 More Info

- **Detailed Setup**: `docs/DOCKER_VPN_SETUP.md`
- **Quick Start**: `docs/DOCKER_VPN_QUICKSTART.md`
- **Implementation**: `docs/DOCKER_VPN_SUMMARY.md`

## 🎯 What's Happening

1. **Container** runs CrypRQ listener on port 9999
2. **Mac** connects to container via web UI or CLI
3. **Container** handles all encryption (ML-KEM + X25519)
4. **Container** handles key rotation (every 5 minutes)
5. **Traffic** flows: Mac → Container → Encrypted Tunnel → Internet

## ✨ Benefits

✅ No macOS Network Extension required  
✅ Isolated container environment  
✅ Easy start/stop with scripts  
✅ Centralized logging  
✅ Works on any platform with Docker  

---

**Ready to go?** Start Docker Desktop and run `./scripts/start-full-docker-vpn.sh`!

