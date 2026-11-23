<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Build the same SwiftUI app for Mac, iPhone and iPad

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

Generate the Xcode project into an ignored build directory, using paths relative to this specification. Simulator builds need no signing identity. A server address is entered by the user, never compiled into the application.

The build generates its ignored Xcode project beside this spec and the Sources directory. The installed XcodeGen emits folder references relative to the spec root even when a project is relocated; keeping the project beside its sources makes both Swift and Resources resolve consistently. Derived build products still go under target/, and no user configuration is needed. Copy the source Resources folder beneath HickoryAssets: a top-level Resources folder makes Foundation misclassify the flat iOS app bundle and lose its bundle identifier during installation.

The build-only `HickorySimpleMobile` scheme is explicit so XcodeGen still
provides the scheme used by `just simple-build ios` alongside the test schemes.

The opt-in hosted `HickorySimpleMobileParity` test scheme mounts the production UIKit editor and uses an owned real-engine fixture. Its separate test folder is excluded from application source discovery and the macOS SPM suite. Building the application alone does not run the witness or start a simulator. See the [mounted UIKit acceptance](../../../mechanisms/ios-mounted-parity-acceptance.md) for resource coordination and the exact focused invocation.

`HickorySimpleMobileRecordedRun` drives the installed app through XCUITest so the recorded Run test uses the same SwiftUI controls and WebKit accessibility tree as a person using the app.

The dev build embeds a stamp as a bundle resource. Capture it before signing so
a later launch cannot relabel an already installed simulator app. The same
build directory can produce a distribution app, so remove the dev resource
when the build opts out.

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
targetTemplates:
  DevBuildStamp:
    postBuildScripts:
      - name: Stamp development build
        basedOnDependencyAnalysis: false
        script: |
          set -eu
          file="$TARGET_BUILD_DIR/$UNLOCALIZED_RESOURCES_FOLDER_PATH/HickoryDevBuild.txt"
          if [ "${HICKORY_DEV_BUILD:-}" != 1 ] || [ -n "${HICKORY_VERSION:-}" ] || [ "$CONFIGURATION" != Debug ]; then
            rm -f "$file"
            exit 0
          fi
          cd "$SRCROOT/../.."
          stamp=unknown
          if test -z "$(git rev-parse --show-prefix)" &&
             head=$(git rev-parse --verify HEAD) &&
             changes=$(git --no-optional-locks status --porcelain=v1 --untracked-files=normal) &&
             test "$head" = "$(git rev-parse --verify HEAD)"; then
            stamp="$head"
            if [ -n "$changes" ]; then stamp="$stamp+"; fi
          fi
          mkdir -p "$(dirname "$file")"
          printf '%s' "$stamp" > "$file"
targets:
  HickorySimpleMac:
    templates: [DevBuildStamp]
    type: application
    platform: macOS
    deploymentTarget: "14.0"
    sources:
      - path: Sources
        excludes: [Resources]
      - path: Sources/Resources
        type: folder
        buildPhase:
          copyFiles:
            destination: resources
            subpath: HickoryAssets
    settings:
      base:
        PRODUCT_BUNDLE_IDENTIFIER: com.hickorydocs.simple.mac
  HickorySimpleMobile:
    templates: [DevBuildStamp]
    type: application
    platform: iOS
    deploymentTarget: "17.0"
    sources:
      - path: Sources
        excludes: [Resources]
      - path: Sources/Resources
        type: folder
        buildPhase:
          copyFiles:
            destination: resources
            subpath: HickoryAssets
    settings:
      base:
        PRODUCT_BUNDLE_IDENTIFIER: com.hickorydocs.simple.mobile
        TARGETED_DEVICE_FAMILY: "1,2"
        INFOPLIST_KEY_UILaunchScreen_Generation: YES
        INFOPLIST_KEY_UIApplicationSceneManifest_Generation: YES
  HickorySimpleMobileParityTests:
    type: bundle.unit-test
    platform: iOS
    deploymentTarget: "17.0"
    sources:
      - path: IOSAcceptance
    dependencies:
      - target: HickorySimpleMobile
    settings:
      base:
        PRODUCT_BUNDLE_IDENTIFIER: com.hickorydocs.simple.mobile.paritytests
        TEST_HOST: "$(BUILT_PRODUCTS_DIR)/HickorySimpleMobile.app/HickorySimpleMobile"
        BUNDLE_LOADER: "$(TEST_HOST)"
  HickorySimpleMobileRecordedRunTests:
    type: bundle.ui-testing
    platform: iOS
    deploymentTarget: "17.0"
    sources:
      - path: IOSUIAcceptance
    dependencies:
      - target: HickorySimpleMobile
    settings:
      base:
        PRODUCT_BUNDLE_IDENTIFIER: com.hickorydocs.simple.mobile.recordedruntests
        TEST_TARGET_NAME: HickorySimpleMobile
schemes:
  HickorySimpleMobile:
    build:
      targets:
        HickorySimpleMobile: all
  HickorySimpleMobileParity:
    build:
      targets:
        HickorySimpleMobile: all
        HickorySimpleMobileParityTests: [test]
    test:
      targets:
        - HickorySimpleMobileParityTests
  HickorySimpleMobileRecordedRun:
    build:
      targets:
        HickorySimpleMobile: all
        HickorySimpleMobileRecordedRunTests: [test]
    test:
      targets:
        - HickorySimpleMobileRecordedRunTests
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/apple-simple/project.yml"><ui:paste select="#simple-9203f6a7de-0"/></ui:file>

</ui:doc>
