# Isolated TUN test for the authenticated VPN link

Status: **stage 1 has been run and passes** (Linux containers, `scripts/e2e-tun-docker.sh`,
71 checks). **Stage 2 has not been run.**

What stage 1 establishes: on Linux, `cryprq --vpn` creates a TUN device, and
packets go from one kernel through its device, the authenticated link and the
other device to the other kernel, in both directions, with nothing but the
link's datagrams on the wire between them.

What it does not establish: anything about macOS utun devices, and anything
about routes, DNS or leak resistance. `cryprq --vpn` installs no routes and no
DNS settings; a point-to-point link between two containers says nothing about
a host's traffic.

The first run of stage 1 found that `--vpn` refused to start on every Linux
TUN device: the address was assigned once when the device was created and
then a second time with `ip addr add`, which fails. That is fixed.

## What the test has to show

1. `cryprq --vpn` creates a TUN device and assigns the address it was given.
2. An IP packet written to the device by the kernel arrives, decrypted, at the
   device of the other endpoint, and the reply comes back.
3. On the wire between the endpoints there is only QUIC; the inner packets do
   not appear in clear.
4. Without a session nothing is forwarded, and when a peer stops, the other
   side stops forwarding.
5. After cleanup the environment is exactly what it was before.

## Stage 1: two Linux containers (no change to the host's network)

Implemented as `scripts/e2e-tun-docker.sh`. Both endpoints run in containers
inside Docker's Linux VM, each in its own network namespace with its own TUN
device. A third container holds an identity that the listener does not pin.

Safeguards the script enforces, and checks rather than assumes:

- A dedicated Docker network created with `--internal` (no route to the
  outside); the script verifies the flag, that the containers have no default
  route, cannot reach an outside address, and are in three different network
  namespaces. If the namespaces are not distinct it aborts.
- No host networking, no published ports, no Docker socket, no host
  directory mounts, no `--privileged`. Each container gets `NET_ADMIN` and
  `/dev/net/tun` and nothing else; the script reads this back with
  `docker inspect` for every container.
- It refuses to start if Docker is unavailable or an existing Docker network
  uses the address ranges of the test, and it checks that `/dev/net/tun`
  exists before creating anything else.
- Every container, the network and the image carry a name and a label that
  are unique to the run. Cleanup removes them by the ids recorded at
  creation, then verifies nothing with the run's label is left. Nothing is
  ever removed by a generic name. Base images are removed only if this run
  pulled them.
- The image is built from `git archive HEAD`: committed files only, so no
  untracked file, credential or build output of the workspace can enter the
  build context. The file list is checked against the commit and kept.
- Every `docker exec`, capture, wait and shutdown has a hard timeout.
- Logs and packet captures are kept under `~/cryprq-tun-evidence/<run>/`.
  Identity keys never leave the containers and are destroyed with them.
- The host's routes, DNS configuration, VPN services, interfaces and
  addresses, and Docker's containers, networks, volumes and images, are
  recorded before and compared after. Any difference fails the run.

Addresses: the link between the containers is `172.31.251.0/29`, the tunnel
is `172.31.250.0/30` (`.1` listener, `.2` dialer). Both exist only inside the
VM. The containers run as root, which is not root on the host; the image
contains no `sudo`.

What it checks:

1. A device without a session forwards nothing: a ping into the tunnel gets
   no reply and its payload does not appear on the wire.
2. Both ends establish one session; each namespace has exactly one route to
   the tunnel subnet and no default route.
3. Kernel to TUN to authenticated link to TUN to kernel, in both directions:
   10 of 10 echo replies each way, 1 MiB over TCP each way with equal
   SHA-256, and the MTU boundary (1420 passes unfragmented, 1421 does not).
4. The wire carries only the link's UDP datagrams and ARP: no ICMP, and the
   ping payload does not occur in the capture. The same search finds the
   payload in the capture of the TUN device, which proves the search works.
5. The unpinned peer is refused, gets no session and no reply, and the
   pinned link keeps its session.
6. When the listener stops (SIGTERM, exit status 0, device gone), the dialer
   drops the session, its packets get no reply and do not reach the wire.
7. The dialer reconnects to a restarted listener with a new session.
8. A dialer killed with SIGKILL leaves no device and no route; the listener
   drops the session.

It does not check routes, DNS or leak protection; see the top of this file.

Rollback: there is nothing on the host to roll back. If a comparison at the
end reports a difference, the run fails and the difference is reported to
the user; nothing on the host is repaired by the script.

## Stage 2: a utun device on a macOS host

**Not run.** Stage 1 says nothing about macOS: utun devices have been
exercised only through packet framing and address checks in unit tests. This
stage needs root on the host and changes its interface list, so it belongs on
a machine set aside for testing, never on one whose network connection
matters.

Preconditions:

- Stage 1 passes on the revision under test.
- `scripts/vpn_preflight_macos.py --tun-address A --netmask M` reports no
  conflict immediately before the start. The default `10.0.0.1/24` lies in a
  range that other VPNs commonly use.

Design: one endpoint on a real utun device, the other on `--packet-bridge`
with a small userspace responder that answers ICMP echo for the peer address.
Both addresses of a subnet cannot be assigned to two devices of one host and
still exchange packets through the tunnel, because the kernel delivers
traffic between its own addresses locally. The link itself runs over
`127.0.0.1`.

What it changes on the host, all of it removed when the process exits: one
new `utun<N>` interface, its address, and the connected route that the
address brings with it. It adds no default route, no DNS setting and no
firewall rule.

Evidence: snapshots of `ifconfig -a`, `netstat -rn` and `scutil --dns` before
and after, `route -n get <peer address>` naming the new utun while the
tunnel is up, an unchanged default route and DNS configuration throughout,
and `ping` answered through the tunnel.

Rollback: SIGTERM to the process and verify that the device, its address and
its route are gone; SIGKILL after 10 s if it has not exited (the kernel
destroys the device when its descriptor closes); compare the snapshots. Abort
if the default route or the DNS configuration changes at any point.
