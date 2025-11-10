import Foundation
import Network

public final class CrypRQTunnelController: ObservableObject {
    public enum State: Equatable {
        case idle
        case connecting
        case connected
        case failed(String)
    }

    public typealias HandleFactory = (CrypRQConfig) throws -> CrypRQHandleProtocol

    @Published public private(set) var state: State = .idle
    private var handle: CrypRQHandleProtocol?
    private let configuration: TunnelConfiguration
    private let handleFactory: HandleFactory

    public init(configuration: TunnelConfiguration, handleFactory: @escaping HandleFactory = { try CrypRQHandle(config: $0) }) {
        self.configuration = configuration
        self.handleFactory = handleFactory
    }

    @MainActor
    public func start() {
        guard case .idle = state else { return }
        state = .connecting

        Task.detached { [weak self] in
            guard let self else { return }
            do {
                let handle = try self.handleFactory(self.configuration.ffiConfig())
                try handle.connect(self.configuration.peerParams())
                await MainActor.run {
                    self.handle = handle
                    self.state = .connected
                }
            } catch let error as CrypRQError {
                await MainActor.run {
                    self.state = .failed("\(error)")
                }
            } catch {
                await MainActor.run {
                    self.state = .failed(error.localizedDescription)
                }
            }
        }
    }

    @MainActor
    public func stop() {
        handle?.close()
        handle = nil
        state = .idle
    }
}

