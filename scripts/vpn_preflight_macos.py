#!/usr/bin/env python3
# © 2025 Thor Thor
# SPDX-License-Identifier: MIT
"""Read-only check of a macOS host before `cryprq --vpn` is run on a TUN device.

It answers one question: would the address the tunnel is about to get collide
with something this host already uses? It reads the interface list, the
routing tables, the DNS resolvers and the configured VPN services, and reports
what it finds.

It changes nothing. It runs no command with elevated privileges, creates no
interface and touches no route, DNS or firewall setting. The commands it runs
are `ifconfig -a`, `netstat -rn -f inet`, `netstat -rn -f inet6`,
`scutil --dns` and `scutil --nc list`.

Exit status: 0 no conflict found, 1 at least one conflict or the host could
not be read, 2 wrong usage or not macOS.

Usage: scripts/vpn_preflight_macos.py [--tun-address 10.0.0.1]
           [--netmask 255.255.255.0] [--tun-name cryprq0] [--suggest]

With --suggest, or whenever a conflict is found, it also names a /30 subnet
that is free on this host right now. That is a candidate, not a clearance:
run the check again with that address immediately before using it.
"""

import argparse
import ipaddress
import platform
import re
import subprocess
import sys

COMMANDS = {
    "ifconfig": ["ifconfig", "-a"],
    "routes4": ["netstat", "-rn", "-f", "inet"],
    "routes6": ["netstat", "-rn", "-f", "inet6"],
    "dns": ["scutil", "--dns"],
    "vpn_services": ["scutil", "--nc", "list"],
}


# Subnets tried by --suggest, in this order. All are private or shared address
# space; each is checked against this host like any other address.
CANDIDATES = [
    "172.31.250.0/30",
    "172.30.250.0/30",
    "192.168.250.0/30",
    "192.168.251.0/30",
    "10.250.250.0/30",
    "100.127.250.0/30",
]

# The two tools without which no conflict can be ruled out.
REQUIRED = ("ifconfig", "routes4")


def validate_name(name):
    """An interface name the system could be asked for."""
    if not re.fullmatch(r"[A-Za-z0-9_.-]{1,15}", name):
        raise ValueError(f"{name!r} is not an interface name (1 to 15 letters, digits, '_', '.', '-')")
    return name


def validate_tunnel(address, netmask):
    """The tunnel's interface, by the rules of `TunConfig::validate`."""
    try:
        ip = ipaddress.IPv4Address(address)
    except ValueError:
        raise ValueError(f"{address!r} is not an IPv4 address")
    try:
        mask = ipaddress.IPv4Address(netmask)
        network = ipaddress.IPv4Network(f"{ip}/{mask}", strict=False)
    except ValueError:
        raise ValueError(f"{netmask!r} is not a contiguous IPv4 netmask")
    if ip.is_unspecified or ip.is_loopback or ip.is_link_local or ip.is_multicast \
            or ip == ipaddress.IPv4Address("255.255.255.255"):
        raise ValueError(f"{ip} is reserved and cannot be assigned to a tunnel")
    if not 8 <= network.prefixlen <= 30:
        raise ValueError(f"/{network.prefixlen} is outside the supported range /8 to /30")
    if ip in (network.network_address, network.broadcast_address):
        raise ValueError(f"{ip} is the network or broadcast address of {network}")
    return ipaddress.IPv4Interface(f"{ip}/{network.prefixlen}")


def parse_ifconfig(text):
    """`{name: {"flags": set, "inet": [(address, network, peer)]}}`."""
    interfaces, current = {}, None
    for line in text.splitlines():
        head = re.match(r"^([A-Za-z0-9_.-]+): flags=[0-9a-f]+<([^>]*)>", line)
        if head:
            current = interfaces.setdefault(
                head.group(1), {"flags": set(filter(None, head.group(2).split(","))), "inet": []})
            continue
        inet = re.match(
            r"^\s+inet (\d+\.\d+\.\d+\.\d+)(?: --> (\d+\.\d+\.\d+\.\d+))? netmask (0x[0-9a-f]{8})", line)
        if inet and current is not None:
            address = ipaddress.IPv4Address(inet.group(1))
            prefix = bin(int(inet.group(3), 16)).count("1")
            network = ipaddress.IPv4Network(f"{address}/{prefix}", strict=False)
            peer = ipaddress.IPv4Address(inet.group(2)) if inet.group(2) else None
            current["inet"].append((address, network, peer))
    return interfaces


def parse_destination(text):
    """A destination of `netstat -rn -f inet`; `None` for the default route.

    netstat drops trailing zero octets: `127` is 127.0.0.0/8 and `192.168.60`
    is 192.168.60.0/24. Four octets without a prefix are a host route.
    """
    if text == "default":
        return None
    address, _, prefix = text.partition("/")
    octets = address.split(".")
    if not 1 <= len(octets) <= 4 or not all(o.isdigit() and int(o) <= 255 for o in octets):
        raise ValueError(f"unrecognised route destination {text!r}")
    if not prefix:
        prefix = "32" if len(octets) == 4 else str(8 * len(octets))
    padded = ".".join(octets + ["0"] * (4 - len(octets)))
    return ipaddress.IPv4Network(f"{padded}/{prefix}", strict=False)


def parse_routes(text):
    """`[(network or None for default, gateway, flags, interface)]` for IPv4."""
    routes, in_table = [], False
    for line in text.splitlines():
        fields = line.split()
        if fields[:2] == ["Destination", "Gateway"]:
            in_table = True
            continue
        if not in_table or len(fields) < 4:
            continue
        try:
            destination = parse_destination(fields[0])
        except ValueError:
            continue
        routes.append((destination, fields[1], fields[2], fields[3]))
    return routes


def default_interfaces(text):
    """Interfaces that carry a default route in a `netstat -rn` table."""
    found = []
    for line in text.splitlines():
        fields = line.split()
        if len(fields) >= 4 and fields[0] == "default" and fields[3] not in found:
            found.append(fields[3])
    return found


def connected_vpn_services(text):
    """Names of the VPN services `scutil --nc list` reports as connected."""
    names = []
    for line in text.splitlines():
        state = re.search(r"\((Connected|Connecting)\)", line)
        name = re.search(r'"([^"]*)"', line)
        if state and name:
            names.append(name.group(1))
    return names


def nameservers(text):
    found = []
    for match in re.finditer(r"nameserver\[\d+\]\s*:\s*(\S+)", text):
        if match.group(1) not in found:
            found.append(match.group(1))
    return found


def is_utun_name(name):
    return re.fullmatch(r"utun\d{1,4}", name) is not None


def analyse(tunnel, name, outputs):
    """`(failures, warnings, notes)` for the tunnel interface on this host."""
    failures, warnings, notes = [], [], []
    interfaces = parse_ifconfig(outputs["ifconfig"])
    routes = parse_routes(outputs["routes4"])
    subnet = tunnel.network

    # Addresses that are already in use.
    for ifname, info in sorted(interfaces.items()):
        for address, network, peer in info["inet"]:
            if address == tunnel.ip:
                failures.append(f"{tunnel.ip} is already assigned to {ifname}")
            elif ifname != "lo0" and network.overlaps(subnet):
                failures.append(
                    f"the tunnel subnet {subnet} overlaps {network} on {ifname} (address {address})")
            if peer is not None and peer in subnet:
                failures.append(f"{peer}, the far end of {ifname}, lies inside the tunnel subnet {subnet}")

    # Routes that already cover part of the tunnel subnet. The default route
    # covers everything by definition and is reported separately.
    seen = set()
    for destination, gateway, _flags, ifname in routes:
        if destination is None or ifname == "lo0":
            continue
        if destination.is_multicast or destination.prefixlen == 32 and destination.network_address in (
                ipaddress.IPv4Address("255.255.255.255"),):
            continue
        if destination.overlaps(subnet) and (destination, ifname) not in seen:
            seen.add((destination, ifname))
            failures.append(
                f"a route for {destination} via {gateway} on {ifname} overlaps the tunnel subnet {subnet}")

    # The interface name.
    if is_utun_name(name):
        if name in interfaces:
            failures.append(f"{name} already exists; cryprq could not create it")
    else:
        notes.append(f"macOS names TUN devices utun<N>; for --tun-name {name} the system picks a free one")

    # Other VPNs. A second tunnel on a host with a live VPN is where routes
    # and DNS interact in ways this check cannot predict.
    utuns = sorted(n for n in interfaces if n.startswith("utun"))
    addressed = [n for n in utuns if interfaces[n]["inet"]]
    v4_default = default_interfaces(outputs["routes4"])
    v6_default = default_interfaces(outputs["routes6"] or "")
    if any(n.startswith("utun") for n in v4_default):
        warnings.append(
            "the IPv4 default route goes through "
            + ", ".join(n for n in v4_default if n.startswith("utun"))
            + ": a VPN is active on this host")
    for key, what in (("routes6", "the IPv6 routing table"), ("dns", "the DNS resolvers"),
                      ("vpn_services", "the VPN services")):
        if outputs[key] is None:
            warnings.append(f"{what} could not be read; nothing is known about them")
    services = connected_vpn_services(outputs["vpn_services"] or "")
    if services:
        warnings.append("connected VPN services: " + ", ".join(services))
    if addressed:
        notes.append("utun interfaces with an IPv4 address: " + ", ".join(addressed))
    notes.append(f"{len(utuns)} utun interfaces exist; IPv4 default route via "
                 + (", ".join(v4_default) or "none"))

    # What the tunnel does not cover.
    if v6_default:
        notes.append(
            "IPv6 default route via " + ", ".join(v6_default)
            + "; cryprq assigns no IPv6 address, so IPv6 traffic does not use the tunnel")
    servers = nameservers(outputs["dns"] or "")
    notes.append(
        "DNS resolvers: " + (", ".join(servers) or "none found")
        + "; cryprq does not change DNS, so lookups do not use the tunnel")
    notes.append(
        f"cryprq installs only the connected route for {subnet}; every other destination keeps its current route")
    return failures, warnings, notes


def suggest(name, outputs):
    """The first candidate subnet without a conflict, as `(listener, dialer)`."""
    for candidate in CANDIDATES:
        network = ipaddress.IPv4Network(candidate)
        first, second = list(network.hosts())[:2]
        clear = True
        for host in (first, second):
            failures, _, _ = analyse(ipaddress.IPv4Interface(f"{host}/{network.prefixlen}"), name, outputs)
            # The name is judged separately; only address and route conflicts count here.
            if any("already exists" not in failure for failure in failures):
                clear = False
        if clear:
            return ipaddress.IPv4Interface(f"{first}/{network.prefixlen}"), \
                ipaddress.IPv4Interface(f"{second}/{network.prefixlen}")
    return None


def run(command):
    """Output of a read-only command, or `None` if it failed."""
    try:
        done = subprocess.run(command, capture_output=True, text=True, timeout=20, check=False)
    except (OSError, subprocess.TimeoutExpired):
        return None
    return done.stdout if done.returncode == 0 else None


def collect(run=run):
    outputs = {key: run(command) for key, command in COMMANDS.items()}
    for key in REQUIRED:
        if not outputs[key] or not outputs[key].strip():
            raise RuntimeError(f"`{' '.join(COMMANDS[key])}` gave no output; the host could not be read")
    if not parse_ifconfig(outputs["ifconfig"]) or not parse_routes(outputs["routes4"]):
        raise RuntimeError("the interface list or the routing table was not understood")
    return outputs


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--tun-address", default="10.0.0.1")
    parser.add_argument("--netmask", default="255.255.255.0")
    parser.add_argument("--tun-name", default="cryprq0")
    parser.add_argument("--suggest", action="store_true",
                        help="also name a /30 subnet that is free on this host right now")
    args = parser.parse_args(argv)

    if platform.system() != "Darwin":
        print("ERROR: this check reads macOS tools; it does not apply to this system", file=sys.stderr)
        return 2
    try:
        validate_name(args.tun_name)
    except ValueError as error:
        print(f"ERROR: {error}", file=sys.stderr)
        return 2
    try:
        tunnel = validate_tunnel(args.tun_address, args.netmask)
    except ValueError as error:
        print(f"FAIL: cryprq would refuse this address: {error}")
        return 1
    try:
        outputs = collect()
    except RuntimeError as error:
        # Not being able to look is not the same as finding nothing.
        print(f"FAIL: {error}")
        return 1

    failures, warnings, notes = analyse(tunnel, args.tun_name, outputs)
    print(f"Tunnel interface under test: {tunnel} (name {args.tun_name}). Nothing on this host was changed.")
    for line in failures:
        print(f"FAIL: {line}")
    for line in warnings:
        print(f"WARN: {line}")
    for line in notes:
        print(f"NOTE: {line}")
    if failures or args.suggest:
        pair = suggest(args.tun_name, outputs)
        if pair:
            print(f"NOTE: free right now: {pair[0].network} ({pair[0].ip} for one peer, {pair[1].ip} for the other,"
                  f" netmask {pair[0].netmask}). A candidate only: check it again immediately before use.")
        else:
            print("NOTE: none of the candidate subnets is free on this host.")
    if failures:
        print(f"RESULT: CONFLICT ({len(failures)}). Do not start the tunnel with this address.")
        return 1
    if warnings:
        print("RESULT: no address or route conflict, but see the warnings before starting the tunnel.")
    else:
        print("RESULT: no address or route conflict found.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
