# Troubleshooting Guide

Common issues and solutions for CrypRQ VPN.

## Connection Issues

### Cannot Connect to Peer

**Symptoms:** Connection timeout or refused

**Solutions:**
1. Verify listener is running: `docker logs cryprq-vpn`
2. Check firewall settings: Ensure UDP port 9999 is open
3. Verify peer address format: `/ip4/IP/udp/PORT/quic-v1`
4. Check network connectivity: `ping <peer-ip>`

### Connection Drops

**Symptoms:** Connection established but drops immediately

**Solutions:**
1. Check logs for errors: `docker logs cryprq-vpn`
2. Verify key rotation is working: Look for rotation events
3. Check system resources: Memory, CPU usage
4. Review network stability: Packet loss, latency

## VPN Mode Issues

### TUN Interface Not Created

**Symptoms:** VPN mode enabled but no TUN interface

**Solutions:**
1. Check permissions: Root/sudo may be required
2. Verify TUN support: `lsmod | grep tun` (Linux)
3. Check Docker capabilities: `NET_ADMIN`, `SYS_MODULE`
4. Review logs: Look for TUN creation errors

### Packets Not Forwarding

**Symptoms:** TUN interface exists but no traffic

**Solutions:**
1. Verify connection established: Check logs
2. Check routing tables: `ip route show`
3. Verify packet forwarding enabled: `sysctl net.ipv4.ip_forward`
4. Review encryption events: Check debug console

## Docker Issues

### Container Won't Start

**Symptoms:** Container exits immediately

**Solutions:**
1. Check logs: `docker logs cryprq-vpn`
2. Verify Docker is running: `docker ps`
3. Check port conflicts: `lsof -i :9999`
4. Review docker-compose.yml: Verify configuration

### Web UI Not Accessible

**Symptoms:** Cannot access http://localhost:8787

**Solutions:**
1. Verify web server running: Check process
2. Check port conflicts: `lsof -i :8787`
3. Review server logs: Check for errors
4. Verify Docker mode: `USE_DOCKER=true`

## Performance Issues

### Slow Connection Establishment

**Symptoms:** Takes >1 second to connect

**Solutions:**
1. Check network latency: `ping <peer>`
2. Review system resources: CPU, memory
3. Verify QUIC support: Check libp2p version
4. Review logs: Look for delays

### High CPU Usage

**Symptoms:** CPU usage >50%

**Solutions:**
1. Reduce log verbosity: Set `RUST_LOG=info`
2. Check packet rate: High traffic may be normal
3. Review key rotation: Frequent rotations increase CPU
4. Monitor system: Use `top` or `htop`

## Getting Help

- Check [Connection Troubleshooting](../docs/CONNECTION_TROUBLESHOOTING.md)
- Review [Docker VPN Setup](../docs/DOCKER_VPN_SETUP.md)
- Open [GitHub Issue](https://github.com/codethor0/cryprq/issues)
- Email: codethor@gmail.com

