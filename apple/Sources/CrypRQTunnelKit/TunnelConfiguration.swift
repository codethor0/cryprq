import Foundation
import NetworkExtension

public struct TunnelConfiguration: Equatable {
    public struct Route: Equatable {
        public var address: String
        public var prefixLength: Int

        public init(address: String, prefixLength: Int) {
            self.address = address
            self.prefixLength = prefixLength
        }
    }

    public var mtu: Int
    public var address: String
    public var routes: [Route]
    public var dns: [String]
    public var allowPeers: [String]
    public var multiaddr: String
    public var mode: CrypRQPeerParams.Mode
    public var logLevel: String?

    public init(
        mtu: Int = 1500,
        address: String,
        routes: [Route],
        dns: [String] = [],
        allowPeers: [String] = [],
        multiaddr: String,
        mode: CrypRQPeerParams.Mode = .dial,
        logLevel: String? = nil
    ) {
        self.mtu = mtu
        self.address = address
        self.routes = routes
        self.dns = dns
        self.allowPeers = allowPeers
        self.multiaddr = multiaddr
        self.mode = mode
        self.logLevel = logLevel
    }

    public func networkSettings() -> NEPacketTunnelNetworkSettings {
        let settings = NEPacketTunnelNetworkSettings(tunnelRemoteAddress: "CrypRQ")
        settings.mtu = NSNumber(value: mtu)
        settings.ipv4Settings = NEIPv4Settings(addresses: [address], subnetMasks: ["255.255.255.0"])
        settings.ipv4Settings?.includedRoutes = routes.map {
            NEIPv4Route(destinationAddress: $0.address, subnetMask: subnetMask(fromPrefix: $0.prefixLength))
        }
        if !dns.isEmpty {
            settings.dnsSettings = NEDNSSettings(servers: dns)
        }
        return settings
    }

    public func ffiConfig() -> CrypRQConfig {
        CrypRQConfig(logLevel: logLevel, allowPeers: allowPeers)
    }

    public func peerParams() -> CrypRQPeerParams {
        CrypRQPeerParams(mode: mode, multiaddr: multiaddr)
    }

    private func subnetMask(fromPrefix prefix: Int) -> String {
        var mask: UInt32 = prefix == 0 ? 0 : ~UInt32(0) << (32 - UInt32(prefix))
        var octets: [String] = []
        for _ in 0..<4 {
            octets.insert(String(mask & 0xFF), at: 0)
            mask >>= 8
        }
        return octets.joined(separator: ".")
    }
}

