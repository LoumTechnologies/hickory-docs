<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Build the native comparison without changing the desktop host

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

A standalone Swift package builds the macOS app and focused model tests. The XcodeGen specification below uses the same source files for iPhone and iPad. Keep build products outside the declared literate outputs.

<ui:copy id="simple-fe680c5df3-0">
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
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/apple-simple/Package.swift"><ui:paste select="#simple-fe680c5df3-0"/></ui:file>

</ui:doc>
