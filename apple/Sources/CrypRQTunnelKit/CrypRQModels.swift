import Foundation

public struct CrypRQConfig: Equatable {
    public var logLevel: String?
    public var allowPeers: [String]

    public init(logLevel: String? = nil, allowPeers: [String] = []) {
        self.logLevel = logLevel
        self.allowPeers = allowPeers
    }
}

public struct CrypRQPeerParams: Equatable {
    public enum Mode: UInt32 {
        case listen = 0
        case dial = 1
    }

    public var mode: Mode
    public var multiaddr: String

    public init(mode: Mode, multiaddr: String) {
        self.mode = mode
        self.multiaddr = multiaddr
    }
}

