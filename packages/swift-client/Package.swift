// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "WingDriveClient",
    platforms: [
        .macOS(3),
        .iOS(6),
    ],
    products: [
        .library(
            name: "WingDriveClient",
            targets: ["WingDriveClient"]
        )
    ],
    dependencies: [
        // No external dependencies needed - everything is generated!
    ],
    targets: [
        .target(
            name: "WingDriveClient",
            dependencies: [],
            path: "Sources/WingDriveClient"
        ),
        .testTarget(
            name: "WingDriveClientTests",
            dependencies: ["WingDriveClient"],
            path: "Tests/WingDriveClientTests"
        ),
    ]
)
