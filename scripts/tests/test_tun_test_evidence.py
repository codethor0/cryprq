"""Adversarial controls for TUN capture and host-state interpretation."""

from pathlib import Path
import struct
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import tun_test_evidence as evidence


# Ethernet / IPv6 / router solicitation with a valid checksum and SLLA option.
RS = bytes.fromhex(
    "33330000000202000000000186dd6000000000103aff"
    "fe80000000000000000000fffe000001ff020000000000000000000000000002"
    "85007b2c000000000101020000000001"
)


def with_checksum(frame):
    packet = bytearray(frame)
    packet[56:58] = b"\0\0"
    pseudo = packet[22:54] + struct.pack("!I", len(packet) - 54) + b"\0\0\0\x3a"
    data = pseudo + packet[54:]
    total = sum(struct.unpack("!" + "H" * (len(data) // 2), data))
    while total >> 16:
        total = (total & 65535) + (total >> 16)
    packet[56:58] = struct.pack("!H", ~total & 65535)
    return bytes(packet)


def pcap(frames, endian="<", magic=b"\xd4\xc3\xb2\xa1"):
    result = magic + struct.pack(endian + "HHIIII", 2, 4, 0, 0, 262144, 1)
    for frame in frames:
        result += struct.pack(endian + "IIII", 0, 0, len(frame), len(frame)) + frame
    return result


class RouterSolicitationTests(unittest.TestCase):
    def test_valid_solicitation_is_recognized(self):
        self.assertTrue(evidence.is_router_solicitation(RS))

    def test_echo_and_other_neighbor_discovery_are_not_exempt(self):
        for kind in [128, 129, 134, 135, 136, 137, 200]:
            with self.subTest(kind=kind):
                packet = bytearray(RS)
                packet[54] = kind
                self.assertFalse(evidence.is_router_solicitation(with_checksum(packet)))

    def test_incorrect_hop_limit_code_reserved_and_options_are_not_exempt(self):
        for offset, value in [(21, 64), (55, 1), (58, 1), (62, 5), (63, 2), (64, 4)]:
            with self.subTest(offset=offset):
                packet = bytearray(RS)
                packet[offset] = value
                self.assertFalse(evidence.is_router_solicitation(with_checksum(packet)))

    def test_other_ipv6_headers_and_addresses_are_not_exempt(self):
        for offset, value in [(12, 8), (20, 17), (20, 44), (22, 0x20), (53, 1), (6, 1)]:
            with self.subTest(offset=offset, value=value):
                packet = bytearray(RS)
                packet[offset] = value
                self.assertFalse(evidence.is_router_solicitation(with_checksum(packet)))

    def test_extra_payload_and_bad_checksum_are_not_exempt(self):
        self.assertFalse(evidence.is_router_solicitation(RS + b"plaintext payload"))
        packet = bytearray(RS)
        packet[56] ^= 1
        self.assertFalse(evidence.is_router_solicitation(packet))

    def test_pcap_byte_orders_and_nanosecond_headers(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "wire.pcap"
            for endian, magic in [("<", b"\xd4\xc3\xb2\xa1"), (">", b"\xa1\xb2\xc3\xd4"),
                                  ("<", b"\x4d\x3c\xb2\xa1"), (">", b"\xa1\xb2\x3c\x4d")]:
                with self.subTest(magic=magic):
                    echo = bytearray(RS)
                    echo[54] = 128
                    path.write_bytes(pcap([RS, with_checksum(echo), RS], endian, magic))
                    self.assertEqual(evidence.count_router_solicitations(path), 2)

    def test_truncated_or_unsupported_capture_fails_closed(self):
        good = pcap([RS])
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "wire.pcap"
            invalid = [b"", good[:23], good[:30], good[:-1], b"bad!" + good[4:]]
            unsupported = bytearray(good)
            unsupported[20:24] = struct.pack("<I", 113)  # Linux cooked, not Ethernet
            invalid.append(unsupported)
            sliced_packet = bytearray(good)
            sliced_packet[36:40] = struct.pack("<I", len(RS) + 1)
            invalid.append(sliced_packet)
            for content in invalid:
                with self.subTest(content=bytes(content[:24])):
                    path.write_bytes(content)
                    with self.assertRaises(ValueError):
                        evidence.count_router_solicitations(path)

    def test_capture_after_the_peer_stops_counts_only_the_solicitation(self):
        # The shape seen on the dialer's wire after the listener stops:
        # IPv4 link traffic, ARP and one kernel router solicitation. A second
        # IPv6 packet that is not a solicitation must not be counted.
        ipv4 = bytes.fromhex("0200000000020200000000010800") + b"\x45" + b"\0" * 47
        arp = bytes.fromhex("ffffffffffff0200000000010806") + b"\0" * 28
        echo = bytearray(RS)
        echo[54] = 128
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "afterstop-wire.pcap"
            path.write_bytes(pcap([ipv4, arp, RS, ipv4, with_checksum(echo)]))
            self.assertEqual(evidence.count_router_solicitations(path), 1)

    def test_both_wire_captures_use_the_strict_count(self):
        script = (Path(__file__).resolve().parents[1] / "e2e-tun-docker.sh").read_text()
        for capture in ["wire.pcap", "afterstop-wire.pcap"]:
            with self.subTest(capture=capture):
                self.assertIn(
                    f'count-router-solicitations "$E/{capture}")', script)
        # No blanket IPv6 exclusion in any capture filter.
        for forbidden in ["not ip6", "not icmp6", "and ip)", "ip and "]:
            self.assertNotIn(forbidden, script)

    def test_missing_capture_is_a_command_failure(self):
        with tempfile.TemporaryDirectory() as directory:
            process = subprocess.run(
                [sys.executable, evidence.__file__, "count-router-solicitations",
                 str(Path(directory) / "absent.pcap")], capture_output=True, text=True)
            self.assertNotEqual(process.returncode, 0)
            self.assertEqual(process.stdout, "")


class DockerBridgeTests(unittest.TestCase):
    mac = "02:00:00:00:00:01"

    def test_only_matching_bridge_address_is_normalized(self):
        kept = ("docker0 inet 172.17.0.1/16\n"
                "docker0 inet6 2001:db8::1/64\n"
                "docker0 inet6 fe80::1234/64\n"
                "eth0 inet6 fe80::ff:fe00:1/64\n"
                "utun0 inet6 fe80::ff:fe00:1/64\n")
        raw = kept + "docker0 inet6 fe80::ff:fe00:1/64\n"
        self.assertEqual(evidence.normalize_host(raw, "addresses", self.mac), kept)
        self.assertEqual(evidence.normalize_host(raw, "addresses", ""), raw)

    def test_changed_mac_or_prefix_remains_visible(self):
        raw = "docker0 inet6 fe80::ff:fe00:2/64\ndocker0 inet6 fe80::ff:fe00:1/128\n"
        self.assertEqual(evidence.normalize_host(raw, "addresses", self.mac), raw)

    def test_only_exact_connected_docker_link_local_route_is_normalized(self):
        expected = "fe80::/64 dev docker0 proto kernel metric 256 pref medium\n"
        kept = ("default via fe80::1 dev eth0 proto ra metric 1024 pref medium\n"
                "fe80::/64 dev eth0 proto kernel metric 256 pref medium\n"
                "2001:db8::/64 dev docker0 proto kernel metric 256 pref medium\n"
                "fe80::/64 dev docker0 proto static metric 256 pref medium\n"
                "fe80::/64 dev docker0 proto kernel metric 1024 pref medium\n")
        self.assertEqual(evidence.normalize_host(expected + kept, "routes6", self.mac), kept)
        self.assertEqual(evidence.normalize_host(expected, "routes6", ""), expected)

    def test_hardware_interface_names_are_never_filtered(self):
        raw = "docker0\nenP20410s1\neth0\nlo\n"
        self.assertEqual(evidence.normalize_host(raw, "addresses", self.mac), raw)

    def test_invalid_or_multicast_mac_is_rejected(self):
        for mac in ["", "bad", "01:00:00:00:00:01", "02:00:00:00:00:xyz"]:
            with self.subTest(mac=mac), self.assertRaises(ValueError):
                evidence.bridge_link_local(mac)


if __name__ == "__main__":
    unittest.main()
