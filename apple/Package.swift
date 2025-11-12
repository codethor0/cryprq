// swift-tools-version: 5.9

import PackageDescription

let package = Package(
    name: "CrypRQTunnelKit",
    platforms: [
        .iOS(.v15),
        .macOS(.v13)
    ],
    products: [
        .library(
            name: "CrypRQTunnelKit",
            targets: ["CrypRQTunnelKit"]
        ),
    ],
    dependencies: [],
    targets: [
        .target(
            name: "CrypRQTunnelKit",
            path: "Sources/CrypRQTunnelKit",
            resources: []
        ),
        .testTarget(
            name: "CrypRQTunnelKitTests",
            dependencies: ["CrypRQTunnelKit"],
            path: "Tests/CrypRQTunnelKitTests"
        ),
    ]
)

