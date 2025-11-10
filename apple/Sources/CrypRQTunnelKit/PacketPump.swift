import Foundation
import NetworkExtension

public protocol PacketPump {
    func start(flow: NEPacketTunnelFlow)
    func stop()
}

public final class NoopPacketPump: PacketPump {
    public init() {}

    public func start(flow _: NEPacketTunnelFlow) {}
    public func stop() {}
}

