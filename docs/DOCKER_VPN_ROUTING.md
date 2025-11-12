# Docker VPN Routing Implementation

## Current Status

The Docker container is running and listening, but VPN mode (system-wide routing) is not yet routing traffic through the container.

## What's Needed

For VPN mode to work, we need:

1. **Container-side:**
   - Run `cryprq` with `--vpn` flag inside container
   - Create TUN interface inside container
   - Set up routing and NAT inside container
   - Forward packets between TUN and encrypted tunnel

2. **Mac-side:**
   - Configure Mac routing to send traffic to container
   - Set up route to container IP
   - Configure DNS (optional)

## Implementation Plan

### Option 1: Container handles VPN (Recommended)
- Container runs `cryprq --listen --vpn`
- Container creates TUN interface
- Mac routes traffic to container IP
- Container handles encryption and forwards to Internet

### Option 2: Mac connects via VPN
- Mac runs `cryprq --peer <container> --vpn`
- Mac creates TUN interface
- Mac routes traffic through TUN
- TUN packets encrypted and sent to container

## Current Limitation

The container is currently running without `--vpn` flag, so:
- No TUN interface is created
- No routing is configured
- Only P2P tunnel encryption works (peer-to-peer)

## Next Steps

1. Update `docker-compose.vpn.yml` to run with `--vpn` flag
2. Ensure container has TUN device access (`/dev/net/tun`)
3. Configure routing inside container
4. Set up Mac routing to container
5. Test traffic routing

