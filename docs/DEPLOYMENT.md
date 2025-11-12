# Deployment Guide

## Overview

This guide covers deploying CrypRQ to production environments, including Docker, bare metal, and cloud deployments.

## Prerequisites

- Production-ready binary or Docker image
- Network access (UDP port 9999 by default)
- System dependencies (if bare metal)
- Firewall configuration

## Deployment Methods

### 1. Docker Deployment

#### Build Production Image

```bash
docker build -t cryprq:latest -f Dockerfile .
```

#### Run Container

```bash
## Listener node
docker run -d \
  --name cryprq-listener \
  -p 9999:9999/udp \
  --restart unless-stopped \
  cryprq:latest \
  --listen /ip4/0.0.0.0/udp/9999/quic-v1

## Check logs
docker logs -f cryprq-listener
```

#### Docker Compose

```yaml
## docker-compose.prod.yml
version: '3.8'

services:
  cryprq-listener:
    image: cryprq:latest
    container_name: cryprq-listener
    ports:
      - "9999:9999/udp"
    command: ["--listen", "/ip4/0.0.0.0/udp/9999/quic-v1"]
    restart: unless-stopped
    environment:
      - RUST_LOG=info
```

```bash
docker compose -f docker-compose.prod.yml up -d
```

### 2. Bare Metal Deployment

#### Linux (systemd)

1. **Copy binary**:
```bash
sudo cp target/release/cryprq /usr/local/bin/
sudo chmod +x /usr/local/bin/cryprq
```

2. **Create systemd service**:
```ini
## /etc/systemd/system/cryprq.service
[Unit]
Description=CrypRQ VPN Listener
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=cryprq
Group=cryprq
Environment=RUST_LOG=info
ExecStart=/usr/local/bin/cryprq --listen /ip4/0.0.0.0/udp/9999/quic-v1
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```

3. **Enable and start**:
```bash
sudo systemctl daemon-reload
sudo systemctl enable cryprq
sudo systemctl start cryprq
sudo systemctl status cryprq
```

#### macOS (launchd)

1. **Create plist**:
```xml
<!-- ~/Library/LaunchAgents/com.cryprq.listener.plist -->
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.cryprq.listener</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/cryprq</string>
        <string>--listen</string>
        <string>/ip4/0.0.0.0/udp/9999/quic-v1</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/var/log/cryprq.log</string>
    <key>StandardErrorPath</key>
    <string>/var/log/cryprq.error.log</string>
</dict>
</plist>
```

2. **Load service**:
```bash
launchctl load ~/Library/LaunchAgents/com.cryprq.listener.plist
launchctl start com.cryprq.listener
```

### 3. Cloud Deployment

#### AWS EC2

1. **Launch EC2 instance** (Ubuntu/Debian)
2. **Install dependencies**:
```bash
sudo apt-get update
sudo apt-get install -y docker.io
sudo systemctl start docker
```

3. **Deploy with Docker**:
```bash
docker pull cryprq:latest
docker run -d --name cryprq -p 9999:9999/udp --restart unless-stopped cryprq:latest
```

4. **Configure Security Group**:
   - Allow UDP port 9999 from trusted IPs

#### Google Cloud Platform

1. **Create VM instance**
2. **Deploy container**:
```bash
gcloud compute instances create-with-container cryprq-node \
  --container-image cryprq:latest \
  --container-arg="--listen" \
  --container-arg="/ip4/0.0.0.0/udp/9999/quic-v1" \
  --tags=cryprq
```

3. **Configure firewall**:
```bash
gcloud compute firewall-rules create allow-cryprq \
  --allow udp:9999 \
  --source-ranges <TRUSTED_IPS> \
  --target-tags cryprq
```

#### Azure

1. **Create Container Instance**:
```bash
az container create \
  --resource-group myResourceGroup \
  --name cryprq-listener \
  --image cryprq:latest \
  --ports 9999 \
  --protocol UDP \
  --command-line "--listen /ip4/0.0.0.0/udp/9999/quic-v1"
```

## Configuration

### Environment Variables

- `RUST_LOG`: Log level (default: `info`)
  - Options: `error`, `warn`, `info`, `debug`, `trace`

### Command-Line Arguments

```bash
## Listener mode
cryprq --listen /ip4/0.0.0.0/udp/9999/quic-v1

## Dialer mode
cryprq --peer /ip4/<PEER_IP>/udp/9999/quic-v1

## Help
cryprq --help
```

## Monitoring

### Logs

```bash
## Docker
docker logs -f cryprq-listener

## Systemd
journalctl -u cryprq -f

## macOS launchd
tail -f /var/log/cryprq.log
```

### Health Checks

```bash
## Check if process is running
ps aux | grep cryprq

## Check port
netstat -uln | grep 9999

## Test connectivity
nc -u <PEER_IP> 9999
```

### Metrics

CrypRQ exposes Prometheus metrics (if configured):
- Connection count
- Packet statistics
- Key rotation events
- Error rates

## Security Considerations

### Firewall Configuration

```bash
## Allow UDP 9999 from trusted IPs only
sudo ufw allow from <TRUSTED_IP> to any port 9999 proto udp
```

### Network Isolation

- Use private networks when possible
- Restrict access to VPN endpoints
- Use VPN-to-VPN connections for additional security

### Key Management

- Keys rotate automatically every 5 minutes
- Old keys are securely zeroized
- No long-term keys stored

## Troubleshooting

### Connection Issues

1. **Check firewall**: Ensure UDP port 9999 is open
2. **Verify network**: Test connectivity with `nc` or `ping`
3. **Check logs**: Review application logs for errors

### Performance Issues

1. **Monitor resources**: Check CPU/memory usage
2. **Network latency**: Verify network conditions
3. **Load balancing**: Consider multiple instances

### Common Problems

- **"Address already in use"**: Port is already bound
- **"Permission denied"**: Check file permissions
- **"Connection refused"**: Firewall blocking access

## Rollback Procedure

### Docker

```bash
## Stop current version
docker stop cryprq-listener

## Start previous version
docker run -d --name cryprq-listener cryprq:previous-version
```

### Systemd

```bash
## Stop service
sudo systemctl stop cryprq

## Revert binary
sudo cp cryprq.previous /usr/local/bin/cryprq

## Start service
sudo systemctl start cryprq
```

## Updates

### Docker

```bash
## Pull new image
docker pull cryprq:latest

## Restart container
docker restart cryprq-listener
```

### Bare Metal

```bash
## Stop service
sudo systemctl stop cryprq

## Update binary
sudo cp target/release/cryprq /usr/local/bin/

## Start service
sudo systemctl start cryprq
```

## References

- [Docker Setup Guide](DOCKER.md)
- [Production Readiness Checklist](PRODUCTION_READINESS.md)
- [Security Policy](../SECURITY.md)

