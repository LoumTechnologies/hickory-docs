// swift-tools-version: 6.0
import PackageDescription
let package = Package(
    name: "HickorySimple", platforms: [.macOS(.v14), .iOS(.v17)],
    products: [.executable(name: "HickorySimple", targets: ["HickorySimple"])],
    targets: [
        .executableTarget(name: "HickorySimple", path: "Sources", resources: [.copy("Resources")]),
        .testTarget(name: "HickorySimpleTests", dependencies: ["HickorySimple"], path: "Tests")
    ], swiftLanguageModes: [.v5]
)
