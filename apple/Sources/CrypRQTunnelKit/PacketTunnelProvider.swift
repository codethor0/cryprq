// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

import Foundation
import NetworkExtension

/// Network Extension Packet Tunnel Provider for macOS/iOS
///
/// This implements NEPacketTunnelProvider to enable system-wide VPN routing
/// through the CrypRQ encrypted tunnel.
@available(macOS 13.0, iOS 15.0, *)
public class CrypRQPacketTunnelProvider: NEPacketTunnelProvider {
    private var packetPump: PacketPump?
    private var tunnelController: CrypRQTunnelController?
    
    public override init() {
        super.init()
    }
    
    public override func startTunnel(options: [String : NSObject]?, completionHandler: @escaping (Error?) -> Void) {
        // Extract configuration from tunnel settings
        guard let tunnelSettings = protocolConfiguration as? NETunnelProviderProtocol else {
            completionHandler(NSError(domain: "CrypRQ", code: 1, userInfo: [NSLocalizedDescriptionKey: "Invalid tunnel configuration"]))
            return
        }
        
        // Parse configuration
        guard let configData = tunnelSettings.providerConfiguration?["config"] as? Data,
              let config = try? JSONDecoder().decode(TunnelConfiguration.self, from: configData) else {
            completionHandler(NSError(domain: "CrypRQ", code: 2, userInfo: [NSLocalizedDescriptionKey: "Failed to parse tunnel configuration"]))
            return
        }
        
        // Create tunnel controller
        let controller = CrypRQTunnelController(configuration: config)
        self.tunnelController = controller
        
        // Start packet forwarding
        let pump = CrypRQPacketPump(tunnelController: controller)
        self.packetPump = pump
        
        // Configure tunnel settings using TunnelConfiguration helper
        let settings = config.networkSettings()
        
        // Set network settings
        setTunnelNetworkSettings(settings) { [weak self] error in
            if let error = error {
                completionHandler(error)
                return
            }
            
            // Start packet pump
            self?.packetPump?.start(flow: self!.packetFlow)
            
            // Start tunnel controller
            controller.start()
            
            completionHandler(nil)
        }
    }
    
    public override func stopTunnel(with reason: NEProviderStopReason, completionHandler: @escaping () -> Void) {
        packetPump?.stop()
        tunnelController?.stop()
        packetPump = nil
        tunnelController = nil
        completionHandler()
    }
    
    public override func handleAppMessage(_ messageData: Data, completionHandler: ((Data?) -> Void)?) {
        // Handle messages from the main app
        completionHandler?(nil)
    }
    
    public override func sleep(completionHandler: @escaping () -> Void) {
        // Handle sleep - pause packet forwarding
        packetPump?.stop()
        completionHandler()
    }
    
    public override func wake() {
        // Handle wake - resume packet forwarding
        if let flow = packetFlow {
            packetPump?.start(flow: flow)
        }
    }
}

