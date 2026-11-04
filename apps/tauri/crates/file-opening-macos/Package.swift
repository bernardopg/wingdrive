// swift-tools-version: 5.5
import PackageDescription

let package = Package(
    name: "FileOpening",
    platforms: [
        .macOS(.v11),
        .iOS(.v14)
    ],
    products: [
        .library(
            name: "FileOpening",
            type: .static,
            targets: ["FileOpening"]
        )
    ],
    dependencies: [
        .package(url: "https://github.com/brendonovich/swift-rs", exact: "1.0.9"),
    ],
    targets: [
        .target(
            name: "FileOpening",
            dependencies: [
                .product(name: "SwiftRs", package: "swift-rs")
            ],
            path: "src-swift"
        )
    ]
)
