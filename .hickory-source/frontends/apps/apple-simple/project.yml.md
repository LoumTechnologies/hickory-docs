<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Build the same SwiftUI app for Mac, iPhone and iPad

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

Generate the Xcode project into an ignored build directory, using paths relative to this specification. Simulator builds need no signing identity. A server address is entered by the user, never compiled into the application.

<ui:copy id="simple-9203f6a7de-0">
name: HickorySimple
options:
  bundleIdPrefix: com.hickorydocs
settings:
  base:
    SWIFT_VERSION: "5.0"
    GENERATE_INFOPLIST_FILE: YES
    CODE_SIGNING_ALLOWED: NO
    INFOPLIST_KEY_NSLocalNetworkUsageDescription: Connect to your Hickory workspace server.
    INFOPLIST_KEY_NSAppTransportSecurity_NSAllowsLocalNetworking: YES
targets:
  HickorySimpleMac:
    type: application
    platform: macOS
    deploymentTarget: "14.0"
    sources:
      - path: Sources
        excludes: [Resources]
      - path: Sources/Resources
        type: folder
        buildPhase: resources
    settings:
      base:
        PRODUCT_BUNDLE_IDENTIFIER: com.hickorydocs.simple.mac
  HickorySimpleMobile:
    type: application
    platform: iOS
    deploymentTarget: "17.0"
    sources:
      - path: Sources
        excludes: [Resources]
      - path: Sources/Resources
        type: folder
        buildPhase: resources
    settings:
      base:
        PRODUCT_BUNDLE_IDENTIFIER: com.hickorydocs.simple.mobile
        TARGETED_DEVICE_FAMILY: "1,2"
        INFOPLIST_KEY_UILaunchScreen_Generation: YES
        INFOPLIST_KEY_UIApplicationSceneManifest_Generation: YES
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/apple-simple/project.yml"><ui:paste select="#simple-9203f6a7de-0"/></ui:file>

</ui:doc>
