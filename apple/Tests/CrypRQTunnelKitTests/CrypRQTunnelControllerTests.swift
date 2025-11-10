import XCTest
@testable import CrypRQTunnelKit

final class CrypRQTunnelControllerTests: XCTestCase {
    func testControllerTransitionsToConnectedWithStubHandle() async {
        let config = TunnelConfiguration(
            address: "10.0.0.2",
            routes: [.init(address: "0.0.0.0", prefixLength: 0)],
            allowPeers: ["12D3KooFakePeer"],
            multiaddr: "/ip4/127.0.0.1/udp/9999/quic-v1",
            mode: .listen
        )

        let expectation = expectation(description: "state updated to connected")

        let controller = CrypRQTunnelController(configuration: config) { _ in
            return StubHandle(onConnect: {
                expectation.fulfill()
            })
        }

        await MainActor.run {
            controller.start()
        }

        await fulfillment(of: [expectation], timeout: 1.0)

        await MainActor.run {
            XCTAssertEqual(controller.state, .connected)
            controller.stop()
            XCTAssertEqual(controller.state, .idle)
        }
    }
}

private final class StubHandle: CrypRQHandleProtocol {
    let onConnect: () -> Void

    init(onConnect: @escaping () -> Void) {
        self.onConnect = onConnect
    }

    func connect(_ params: CrypRQPeerParams) throws {
        XCTAssertEqual(params.mode, .listen)
        onConnect()
    }

    func close() {}
}

