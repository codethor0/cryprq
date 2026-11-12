"""The macOS VPN preflight: parsing of the system tools and conflict detection."""

from pathlib import Path
import ipaddress
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import vpn_preflight_macos as preflight  # noqa: E402

IFCONFIG = """\
lo0: flags=8049<UP,LOOPBACK,RUNNING,MULTICAST> mtu 16384
\tinet 127.0.0.1 netmask 0xff000000
\tinet6 ::1 prefixlen 128
gif0: flags=8010<POINTOPOINT,MULTICAST> mtu 1280
en0: flags=8863<UP,BROADCAST,SMART,RUNNING,SIMPLEX,MULTICAST> mtu 1500
\tether aa:bb:cc:dd:ee:ff
\tinet 192.168.1.161 netmask 0xffffff00 broadcast 192.168.1.255
utun0: flags=8051<UP,POINTOPOINT,RUNNING,MULTICAST> mtu 1280
\tinet 10.8.0.2 --> 10.8.0.1 netmask 0xff000000
utun1: flags=8051<UP,POINTOPOINT,RUNNING,MULTICAST> mtu 1500
\tinet6 fe80::1%utun1 prefixlen 64 scopeid 0x10
"""

ROUTES4 = """\
Routing tables

Internet:
Destination        Gateway            Flags               Netif Expire
default            link#29            UGdScg              utun0
default            192.168.1.1       UGdScIg               en0
10.8.0.1          link#29            UGHdS               utun0
127                127.0.0.1          UCS                   lo0
169.254            link#14            UCS                   en0      !
192.168.1         link#14            UCS                   en0      !
192.168.1.1/32    link#14            UCS                   en0      !
172.20/14          192.168.1.1       UGSc                  en0
224.0.0/4          link#14            UmCS                  en0      !
255.255.255.255/32 link#14            UCS                   en0      !
"""

ROUTES6 = """\
Routing tables

Internet6:
Destination                             Gateway                                 Flags               Netif Expire
default                                 fe80::%utun1                            UGcIg               utun1
::1                                     ::1                                     UHL                   lo0
"""

DNS = """\
DNS configuration

resolver #1
  nameserver[0] : 192.168.1.1
  nameserver[1] : 9.9.9.9

resolver #2
  domain   : local
  options  : mdns
"""

SERVICES = """\
Available network connection services in the current set (*=enabled):
* (Connected)      11111111-1111-1111-1111-111111111111 VPN (example) "Office"   [VPN:example]
* (Disconnected)   22222222-2222-2222-2222-222222222222 VPN (example) "Spare"    [VPN:example]
"""

OUTPUTS = {
    "ifconfig": IFCONFIG,
    "routes4": ROUTES4,
    "routes6": ROUTES6,
    "dns": DNS,
    "vpn_services": SERVICES,
}

# A host with one LAN interface and nothing else.
QUIET = {
    "ifconfig": "\n".join(IFCONFIG.splitlines()[:7]) + "\n",
    "routes4": "\n".join(
        line for line in ROUTES4.splitlines() if "utun" not in line and "172.20" not in line) + "\n",
    "routes6": "Routing tables\n\nInternet6:\nDestination Gateway Flags Netif Expire\n",
    "dns": DNS,
    "vpn_services": "Available network connection services in the current set (*=enabled):\n",
}


def analyse(address, netmask="255.255.255.0", name="cryprq0", outputs=None):
    return preflight.analyse(preflight.validate_tunnel(address, netmask), name, outputs or OUTPUTS)


class ParsingTests(unittest.TestCase):
    def test_interfaces_addresses_and_far_ends_are_read(self):
        interfaces = preflight.parse_ifconfig(IFCONFIG)
        self.assertEqual(sorted(interfaces), ["en0", "gif0", "lo0", "utun0", "utun1"])
        self.assertIn("POINTOPOINT", interfaces["utun0"]["flags"])
        self.assertEqual(
            interfaces["en0"]["inet"],
            [(ipaddress.IPv4Address("192.168.1.161"), ipaddress.IPv4Network("192.168.1.0/24"), None)],
        )
        address, network, peer = interfaces["utun0"]["inet"][0]
        self.assertEqual(str(address), "10.8.0.2")
        self.assertEqual(str(network), "10.0.0.0/8")
        self.assertEqual(str(peer), "10.8.0.1")
        self.assertEqual(interfaces["utun1"]["inet"], [])

    def test_abbreviated_route_destinations_are_expanded(self):
        cases = {
            "127": "127.0.0.0/8",
            "169.254": "169.254.0.0/16",
            "192.168.1": "192.168.1.0/24",
            "192.168.1.1": "192.168.1.1/32",
            "192.168.1.1/32": "192.168.1.1/32",
            "172.20/14": "172.20.0.0/14",
            "224.0.0/4": "224.0.0.0/4",
        }
        for text, expected in cases.items():
            self.assertEqual(str(preflight.parse_destination(text)), expected, text)
        self.assertIsNone(preflight.parse_destination("default"))
        for text in ["", "link#14", "300.1", "1.2.3.4.5", "fe80::1"]:
            with self.assertRaises(ValueError, msg=text):
                preflight.parse_destination(text)

    def test_routes_default_interfaces_services_and_resolvers_are_read(self):
        routes = preflight.parse_routes(ROUTES4)
        self.assertEqual(len(routes), 10)
        self.assertEqual(routes[0], (None, "link#29", "UGdScg", "utun0"))
        self.assertEqual(preflight.default_interfaces(ROUTES4), ["utun0", "en0"])
        self.assertEqual(preflight.default_interfaces(ROUTES6), ["utun1"])
        self.assertEqual(preflight.connected_vpn_services(SERVICES), ["Office"])
        self.assertEqual(preflight.nameservers(DNS), ["192.168.1.1", "9.9.9.9"])


class TunnelAddressTests(unittest.TestCase):
    def test_usable_addresses_are_accepted(self):
        self.assertEqual(str(preflight.validate_tunnel("10.0.0.1", "255.255.255.0")), "10.0.0.1/24")
        self.assertEqual(str(preflight.validate_tunnel("100.64.0.1", "255.0.0.0")), "100.64.0.1/8")
        self.assertEqual(str(preflight.validate_tunnel("10.77.0.2", "255.255.255.252")), "10.77.0.2/30")

    def test_addresses_cryprq_refuses_are_refused_here_too(self):
        for address, netmask in [
            ("10.0.0.1", "0.0.0.0"),
            ("10.0.0.1", "254.0.0.0"),
            ("10.0.0.1", "255.0.255.0"),
            ("10.0.0.1", "255.255.255.254"),
            ("10.0.0.1", "255.255.255.255"),
            ("0.0.0.0", "255.255.255.0"),
            ("127.0.0.1", "255.255.255.0"),
            ("169.254.1.1", "255.255.0.0"),
            ("224.0.0.1", "255.255.255.0"),
            ("255.255.255.255", "255.255.255.0"),
            ("10.0.0.0", "255.255.255.0"),
            ("10.0.0.255", "255.255.255.0"),
            ("fd00::1", "255.255.255.0"),
            ("10.0.0.1", "24"),
            ("", ""),
        ]:
            with self.assertRaises(ValueError, msg=f"{address}/{netmask}"):
                preflight.validate_tunnel(address, netmask)


class ConflictTests(unittest.TestCase):
    def test_a_free_subnet_has_no_conflict_but_the_active_vpn_is_reported(self):
        failures, warnings, notes = analyse("172.31.250.1")
        self.assertEqual(failures, [])
        self.assertTrue(any("default route goes through utun0" in w for w in warnings), warnings)
        self.assertTrue(any("Office" in w for w in warnings), warnings)
        self.assertFalse(any("Spare" in w for w in warnings), warnings)
        self.assertTrue(any("IPv6" in n and "utun1" in n for n in notes), notes)
        self.assertTrue(any("9.9.9.9" in n for n in notes), notes)

    def test_a_subnet_inside_another_interface_is_a_conflict(self):
        # utun0 holds 10.0.0.0/8, which contains the default 10.0.0.1/24.
        failures, _, _ = analyse("10.0.0.1")
        self.assertTrue(any("overlaps 10.0.0.0/8 on utun0" in f for f in failures), failures)

    def test_an_address_that_is_already_assigned_is_a_conflict(self):
        failures, _, _ = analyse("192.168.1.161")
        self.assertTrue(any("already assigned to en0" in f for f in failures), failures)

    def test_the_far_end_of_a_tunnel_inside_the_subnet_is_a_conflict(self):
        failures, _, _ = analyse("10.8.0.2", "255.255.255.0")
        self.assertTrue(any("10.8.0.1, the far end of utun0" in f for f in failures), failures)

    def test_a_route_without_an_interface_address_is_a_conflict(self):
        # 172.20.0.0/14 is routed through en0 but assigned to no interface.
        failures, _, _ = analyse("172.21.0.1")
        self.assertEqual(len(failures), 1, failures)
        self.assertIn("a route for 172.20.0.0/14 via 192.168.1.1 on en0", failures[0])

    def test_default_multicast_and_loopback_routes_are_not_conflicts(self):
        failures, warnings, _ = analyse("172.31.250.1", outputs=QUIET)
        self.assertEqual((failures, warnings), ([], []))

    def test_an_existing_utun_name_is_a_conflict_and_other_names_are_left_to_the_system(self):
        failures, _, _ = analyse("172.31.250.1", name="utun1")
        self.assertTrue(any("utun1 already exists" in f for f in failures), failures)
        failures, _, notes = analyse("172.31.250.1", name="utun9")
        self.assertEqual(failures, [])
        self.assertFalse(any("picks a free one" in n for n in notes), notes)
        failures, _, notes = analyse("172.31.250.1", name="cryprq0")
        self.assertEqual(failures, [])
        self.assertTrue(any("picks a free one" in n for n in notes), notes)


class SuggestionTests(unittest.TestCase):
    def test_the_first_free_candidate_is_suggested_with_two_host_addresses(self):
        listener, dialer = preflight.suggest("cryprq0", OUTPUTS)
        self.assertEqual((str(listener), str(dialer)), ("172.31.250.1/30", "172.31.250.2/30"))
        for address in (listener, dialer):
            self.assertEqual(preflight.analyse(address, "cryprq0", OUTPUTS)[0], [])

    def test_candidates_in_use_are_skipped(self):
        busy = dict(OUTPUTS)
        busy["ifconfig"] = IFCONFIG + "utun7: flags=8051<UP,POINTOPOINT,RUNNING,MULTICAST> mtu 1380\n" \
            "\tinet 172.31.250.1 --> 172.31.250.2 netmask 0xfffffffc\n"
        busy["routes4"] = ROUTES4 + "172.30/16          192.168.1.1       UGSc                  en0\n"
        listener, _ = preflight.suggest("cryprq0", busy)
        self.assertEqual(str(listener), "192.168.250.1/30")

    def test_nothing_is_suggested_when_every_candidate_is_taken(self):
        taken = dict(OUTPUTS)
        taken["routes4"] = ROUTES4 + "".join(
            f"{network}  192.168.1.1  UGSc  en0\n" for network in preflight.CANDIDATES)
        self.assertIsNone(preflight.suggest("cryprq0", taken))

    def test_every_candidate_is_an_address_cryprq_accepts(self):
        for candidate in preflight.CANDIDATES:
            network = ipaddress.IPv4Network(candidate)
            self.assertTrue(network.is_private or network.subnet_of(ipaddress.IPv4Network("100.64.0.0/10")))
            for host in network.hosts():
                preflight.validate_tunnel(str(host), str(network.netmask))


class ReadFailureTests(unittest.TestCase):
    def runner(self, **overrides):
        by_command = {tuple(command): OUTPUTS[key] for key, command in preflight.COMMANDS.items()}
        for key, value in overrides.items():
            by_command[tuple(preflight.COMMANDS[key])] = value
        return lambda command: by_command[tuple(command)]

    def test_a_host_that_cannot_be_read_is_not_reported_as_free(self):
        for key in ("ifconfig", "routes4"):
            for value in (None, "", "   \n", "something else entirely\n"):
                with self.assertRaises(RuntimeError, msg=f"{key}={value!r}"):
                    preflight.collect(self.runner(**{key: value}))

    def test_optional_tools_that_fail_are_reported_not_ignored(self):
        outputs = preflight.collect(self.runner(routes6=None, dns=None, vpn_services=None))
        failures, warnings, _ = preflight.analyse(
            preflight.validate_tunnel("172.31.250.1", "255.255.255.0"), "cryprq0", outputs)
        self.assertEqual(failures, [])
        for what in ("IPv6 routing table", "DNS resolvers", "VPN services"):
            self.assertTrue(any(what in w and "could not be read" in w for w in warnings), warnings)

    def test_a_failing_command_yields_no_output(self):
        self.assertIsNone(preflight.run(["/nonexistent/cryprq-preflight-tool"]))
        self.assertIsNone(preflight.run([sys.executable, "-c", "print('partial'); raise SystemExit(3)"]))
        self.assertEqual(preflight.run([sys.executable, "-c", "print('ok')"]), "ok\n")

    def test_interface_names_are_checked(self):
        for name in ("cryprq0", "utun12", "tun-a_b.1"):
            self.assertEqual(preflight.validate_name(name), name)
        for name in ("", "a" * 16, "utun0; rm -rf /", "tun 0", "tun/0", "ütun"):
            with self.assertRaises(ValueError, msg=name):
                preflight.validate_name(name)


class CommandTests(unittest.TestCase):
    def test_every_command_only_reads(self):
        for command in preflight.COMMANDS.values():
            self.assertIn(command[0], {"ifconfig", "netstat", "scutil"})
            self.assertNotIn("sudo", command)
        self.assertEqual(preflight.COMMANDS["ifconfig"], ["ifconfig", "-a"])
        for key in ("routes4", "routes6"):
            self.assertEqual(preflight.COMMANDS[key][:2], ["netstat", "-rn"])
        self.assertEqual(preflight.COMMANDS["dns"], ["scutil", "--dns"])
        self.assertEqual(preflight.COMMANDS["vpn_services"], ["scutil", "--nc", "list"])

    def test_an_address_cryprq_refuses_ends_with_a_conflict_status(self):
        if preflight.platform.system() != "Darwin":
            self.skipTest("macOS only")
        self.assertEqual(preflight.main(["--tun-address", "127.0.0.1"]), 1)
        self.assertEqual(preflight.main(["--tun-name", "not a name"]), 2)


if __name__ == "__main__":
    unittest.main()
