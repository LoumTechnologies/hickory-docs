<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Run and compare the independent frontends

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

Start with an already running Hickory backend. The browser, SwiftUI and Compose clients use the same workspace API. They do not replace the existing desktop or browser frontend.

<ui:copy id="simple-72e004e1b0-0">
# Compare the simple frontends

For maintainers iterating on the same small editor on Apple, Android and the web.

From the repository root, choose one:

```sh
just simple-dev browser
just simple-dev apple
just simple-dev ios
just simple-dev android
just simple-dev all
```

`all` opens all four frontends together on macOS, using the same workspace. A macOS Accessibility helper tries to arrange the browser, Mac app and simulator/emulator windows in a 2×2 grid on the first display. Set `HICKORY_COMPARISON_SCREEN=1` for the second display, and so on. The terminal running `just` needs Accessibility authorization in System Settings > Privacy & Security > Accessibility. Without it, the helper reports that layout was skipped and the frontends remain open. No permissions are changed automatically. It arranges each unique matching window once; close duplicate comparison windows when selection is ambiguous. Emulator minimum sizes and aspect ratios can prevent an exact fit. Native SDKs, simulator runtime and an Android device/AVD must already be installed. If one platform exits or fails, the runner stops the other sessions. Stop and rerun the whole comparison after saving edits.

The browser hot-reloads its React source. Native source changes require a rebuild and relaunch; there is no common automatic, state-preserving hot reload in this comparison setup.

Each command starts a local workspace backend and opens the chosen frontend with its address already configured. Keep the command running while comparing; Ctrl-C stops the frontend session and disconnects the backend. Browser windows and previously running simulator windows remain open, but their development server/session stops. The original frontend is untouched.

The default workspace is `.dev/simple/project`, seeded idempotently and shared across these commands. Saved edits survive relaunch. Set `HICKORY_PROJECT_DIR=/absolute/path` to compare using another folder. Concurrent frontends can attach to the same workspace engine; each gets its own callback port. This is a host-backed comparison setup, not an embedded mobile Rust engine or a cloud requirement.

Android requires Android Studio’s initial project sync, an installed SDK, and a connected authorized device or an AVD created in Device Manager. The runner builds, installs and opens the debug app, and forwards its backend port over ADB. One connected device is selected automatically; set `ANDROID_SERIAL` when several are attached. With none attached, the runner boots the sole AVD, or the one named by `HICKORY_ANDROID_AVD`. It removes its forwarding and stops the test app on exit; an emulator it started is stopped too. It leaves existing emulators and the shared ADB server running.

iOS launches a simulator, not a physical phone. It prefers a booted iPhone/iPad and otherwise boots an available one. Set `HICKORY_IOS_DEVICE` to a simulator name or UDID to choose an iPad or another target. If multiple simulators are already booted, selection is required. The runner builds, installs and opens the app, and terminates its session on Ctrl-C. Only a simulator started by the runner is shut down.

Native launches use the backend on the host: iOS simulator loopback and Android ADB reverse. A physical iPhone requires a separately reachable backend and manual connection; this recipe does not expose the loopback server on the network. A new launch of the same mobile app may replace a previous comparison session, so save edits before relaunching.

The macOS recipe builds a locally signed `Hickory Simple.app` under `target/simple-apple-launch/`, then opens it through macOS Launch Services with normal Dock/menu activation. It opens a workspace window without waiting for a backend request. The app watches the development launcher’s lifetime, so Ctrl-C still stops this test app.

The Apple build requires full Xcode. `simple-dev` honors `DEVELOPER_DIR` and otherwise finds Xcode under `/Applications` if command-line tools are selected. For standalone build/test recipes, set `DEVELOPER_DIR` to your installed Xcode's `Contents/Developer` directory when needed. `xcodegen` is needed for the iOS project. Run `just simple-build ios`, then open `target/apple-simple-xcode/HickorySimple.xcodeproj` for device or simulator work. Device signing is not configured by this comparison scaffold.

## Shared first slice

All three provide a file tree, UTF-8 text documents, source editing for literate documents, guarded saves, a workspace agent conversation, a terminal, tab closing, recursive right/below splits, resizing, tab dragging and a **Move tab to next group** alternative. Use the File and View/Panes menus; no permanent feature toolbar is added. Generated outputs are read-only until source-mapped editing is connected. Save refuses conflicts and retains local edits. Dirty tabs refuse to close. Workspace lifetime retains document buffers and unsent agent messages when tabs move or hide.

The terminal uses a real backend PTY and its emulated plain-text screen. It accepts line input, Interrupt and EOF; moving or hiding it retains its process and unsent input. Full ANSI color rendering, cursor-key interaction with full-screen terminal programs and terminal scrollback navigation are outside this matched first slice.

The agent uses the configured Python backend. Configure providers with the existing frontend; this first slice does not implement agent installation, provider settings, permission-review surfaces, or token streaming. It refreshes durable conversation history and provides explicit Send and Stop. No provider is required to edit files.

Rich literate rendering, backend capability menus, minimaps, provenance, debugging, file operations, layout persistence and crash recovery are not included yet. This is an iteration baseline, not full parity with the old frontend. Native spellcheck, input methods and platform keyboard behavior may differ. No speed or memory superiority has been established by this scaffold.

## Verification in this session

Browser TypeScript checking, eleven focused browser witnesses, the production bundle, seven Swift witnesses, simulator-target Swift type checking and four shared Kotlin pane scenarios passed. A native-browser walkthrough confirmed a real plain save and undo after moving a split tab. Real PTY Unicode input, output and resize also passed.

The launch runner’s three target-selection and process-cleanup tests passed. The browser launch recipe served its HTML entry and real workspace API; Ctrl-C closed both ports. Native recipe attempts reached macOS app/emulator startup, but this execution host reported GUI bootstrap/notification service failures; device UI behavior remains unverified. Strict Clippy for the launcher was blocked by existing warnings in the CLI library.

The complete automated browser acceptance suite could not launch Chromium because this host refused macOS bootstrap registration. `xcodebuild` also aborted in the host before compilation. The complete Android source compiled with Studio’s bundled Kotlin and Compose compiler against the installed platform 37.0 and cached AndroidX dependencies. Gradle 9.3.1 is downloaded and the scoped commands find it in Studio’s cache. APK assembly remains unverified because shell DNS failures blocked remaining Maven dependencies, including Kotlin build tools 2.2.10. Run the platform builds after those tools are ready; model tests are not substitutes for them.

## Focused checks

- `cargo test --locked -p hickory-cli --example simple_dev`: launch-target selection and real subprocess cleanup.
- `just simple-test browser`: TypeScript checking, focused layout/save tests and the small production build.
- `just simple-test apple`: Swift model and save-protocol witnesses.
- `just simple-test android`: Gradle unit tests, followed by debug APK compilation.
- `just simple-build ios`: unsigned simulator build of the shared Apple source.
- `python3 scripts/test-simple-browser.py`: real browser flows against an isolated real backend. Requires the built `engine_client` example, Node dependencies and an installed Playwright Chromium.

The JSON pane scenarios in `apps/simple-contract/layout-cases.json` are executed by all three languages. They establish command-model parity, not device gesture equivalence. Native IME, accessibility, drag/drop and editor-state preservation still need platform UI acceptance. The Android SDK/Gradle build and iOS simulator build must pass before claiming those targets run successfully.

Build output stays in `target/`, `apps/web/dist-simple/` and ignored Android build directories. No stable release or installed old frontend is replaced. Broad regression checks remain a maintainer action; use `just checks-status` to inspect asynchronous post-commit checks.
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="docs/developers/simple-frontends.md"><ui:paste select="#simple-72e004e1b0-0"/></ui:file>

</ui:doc>
