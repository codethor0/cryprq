#!/usr/bin/env python3
"""Strict interpretation of infrastructure evidence for the isolated TUN test."""

import argparse
import ipaddress
import json
from pathlib import Path
import re
import struct
import sys


def bridge_link_local(mac):
    if not re.fullmatch(r"(?:[0-9a-fA-F]{2}:){5}[0-9a-fA-F]{2}", mac):
        raise ValueError("invalid bridge MAC address")
    octets = bytearray.fromhex(mac.replace(":", ""))
    if octets[0] & 1:
        raise ValueError("bridge MAC must be unicast")
    octets[0] ^= 2
    iid = bytes(octets[:3]) + b"\xff\xfe" + bytes(octets[3:])
    return ipaddress.IPv6Address(b"\xfe\x80" + b"\0" * 6 + iid)


def normalize_host(text, kind, mac):
    """Only omit docker0's kernel EUI-64 address and its connected prefix."""
    address = bridge_link_local(mac) if mac else None
    result = []
    for line in text.splitlines():
        fields = line.split()
        expected = False
        if address is not None and kind == "addresses" and len(fields) == 3:
            if fields[:2] == ["docker0", "inet6"]:
                expected = fields[2] == f"{address}/64"
        if address is not None and kind == "routes6":
            expected = fields == [
                "fe80::/64", "dev", "docker0", "proto", "kernel",
                "metric", "256", "pref", "medium",
            ]
        if not expected:
            result.append(line)
    return "".join(line + "\n" for line in result)


def is_router_solicitation(frame):
    """Accept only the observed EUI-64/SLLA solicitation, not IPv6 data."""
    if len(frame) != 70 or frame[12:14] != b"\x86\xdd":
        return False
    header, message = frame[14:54], frame[54:]
    if header[0] >> 4 != 6 or header[4:8] != b"\0\x10\x3a\xff":
        return False  # 16-byte ICMPv6, no extension headers, hop limit 255
    if frame[:6] != b"\x33\x33\0\0\0\x02":
        return False
    if header[24:40] != ipaddress.IPv6Address("ff02::2").packed:
        return False
    mac = frame[6:12]
    if mac[0] & 1 or header[8:24] != bridge_link_local(mac.hex(":")).packed:
        return False
    if message[:2] != b"\x85\0" or message[4:10] != b"\0\0\0\0\x01\x01":
        return False  # RS type/code, reserved field, one SLLA option
    if message[10:] != mac:
        return False
    pseudo = header[8:40] + struct.pack("!I", len(message)) + b"\0\0\0\x3a"
    data = pseudo + message
    total = sum(struct.unpack("!" + "H" * (len(data) // 2), data))
    while total >> 16:
        total = (total & 0xffff) + (total >> 16)
    return total == 0xffff


def count_router_solicitations(path):
    """Reject missing, truncated or unsupported captures, never count them clean."""
    with Path(path).open("rb") as capture:
        header = capture.read(24)
        if len(header) != 24:
            raise ValueError("missing pcap header")
        formats = {b"\xd4\xc3\xb2\xa1": "<", b"\xa1\xb2\xc3\xd4": ">",
                   b"\x4d\x3c\xb2\xa1": "<", b"\xa1\xb2\x3c\x4d": ">"}
        endian = formats.get(header[:4])
        if endian is None:
            raise ValueError("unsupported capture format")
        major, minor, _, _, snaplen, link = struct.unpack(endian + "HHIIII", header[4:])
        if (major, minor, link) != (2, 4, 1) or not 0 < snaplen <= 16 * 1024 * 1024:
            raise ValueError("unsupported pcap header")
        count = 0
        while True:
            record = capture.read(16)
            if not record:
                return count
            if len(record) != 16:
                raise ValueError("truncated pcap record")
            _, _, included, original = struct.unpack(endian + "IIII", record)
            if not 0 < included <= snaplen or included != original:
                raise ValueError("truncated or invalid captured packet")
            frame = capture.read(included)
            if len(frame) != included:
                raise ValueError("truncated packet data")
            count += is_router_solicitation(frame)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    nd = commands.add_parser("count-router-solicitations")
    nd.add_argument("capture")
    norm = commands.add_parser("normalize-host")
    norm.add_argument("kind", choices=["routes6", "addresses"])
    norm.add_argument("--bridge-mac", default="")
    commands.add_parser("bridge-mac")
    commands.add_parser("link-inventory")
    args = parser.parse_args()
    try:
        if args.command == "count-router-solicitations":
            print(count_router_solicitations(args.capture))
        elif args.command == "normalize-host":
            sys.stdout.write(normalize_host(sys.stdin.read(), args.kind, args.bridge_mac))
        elif args.command == "link-inventory":
            links = json.load(sys.stdin)
            for link in links:
                device = Path("/sys/class/net") / link["ifname"] / "device"
                for name, path in [("device", device), ("driver", device / "driver")]:
                    link[name] = str(path.resolve()) if path.exists() else None
            json.dump(links, sys.stdout, indent=2)
            print()
        else:
            links = json.load(sys.stdin)
            if len(links) != 1 or links[0].get("ifname") != "docker0":
                raise ValueError("not the default Docker bridge")
            link = links[0]
            if link.get("linkinfo", {}).get("info_kind") != "bridge":
                raise ValueError("docker0 is not a bridge")
            mac = link["address"]
            bridge_link_local(mac)
            print(mac)
    except (OSError, ValueError, KeyError, TypeError) as error:
        parser.exit(1, f"Invalid TUN test evidence: {error}\n")


if __name__ == "__main__":
    main()
