import Foundation

public enum CrypRQError: Error, Equatable {
    case unsupported
    case initializationFailed(code: UInt32)
    case connectFailed(code: UInt32)
    case invalidConfiguration(String)
}

