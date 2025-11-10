import Foundation

public protocol CrypRQHandleProtocol {
    func connect(_ params: CrypRQPeerParams) throws
    func close()
}

public final class CrypRQHandle: CrypRQHandleProtocol {
    public init(config _: CrypRQConfig) throws {
        throw CrypRQError.unsupported
    }

    public func connect(_ params: CrypRQPeerParams) throws {
        throw CrypRQError.unsupported
    }

    public func close() {}
}

