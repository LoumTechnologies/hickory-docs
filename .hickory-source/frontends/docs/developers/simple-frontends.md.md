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

All three provide a file tree, UTF-8 text documents, source editing for literate documents, guarded saves, a workspace agent conversation, a terminal, tab closing, recursive right/below splits, resizing, tab dragging and a **Move tab to next group** alternative. To split by dragging, drop a tab near the left, right, top or bottom edge of pane content. The shaded half shows where the new pane will appear. The outer twenty percent triggers splitting; left/right own corners. The new pane gets the existing tab and focus. An empty source group closes automatically, and its sibling fills the freed space. Closing a group’s last tab does the same. If all tabs close, one reusable empty root remains. The explicit Split command can create a temporary empty destination. A center drop adds the dragged tab to the existing group and activates it without splitting that area.

To reorder, drag a tab onto the tab it should precede. Drop onto unused strip space to append. The same gesture inserts tabs at a chosen position in another pane. The insertion mark overlays the target edge without moving content.

Use the File and View/Panes menus; no permanent feature toolbar is added. Generated outputs are read-only until source-mapped editing is connected. Save refuses conflicts and retains local edits. Dirty tabs refuse to close. Workspace lifetime retains document buffers and unsent agent messages when tabs move or hide.

The terminal uses a real backend PTY and its emulated plain-text screen. It accepts line input, Interrupt and EOF; moving or hiding it retains its process and unsent input. Full ANSI color rendering, cursor-key interaction with full-screen terminal programs and terminal scrollback navigation are outside this matched first slice.

The agent uses the configured Python backend. Configure providers with the existing frontend; this first slice does not implement agent installation, provider settings, permission-review surfaces, or token streaming. It refreshes durable conversation history and provides explicit Send and Stop. No provider is required to edit files.

The browser attaches the existing workspace LSP bridge after opening text: semantic colors, completion (Ctrl-Space), hover, signature help and inlay hints are available when a language server supports them. Same-document definition navigation is connected; cross-file navigation, rename and code actions remain outside this slice. Diagnostics are not displayed yet because publications have no source revision to guard against stale results.

Apple now attaches the existing backend LSP channel after text opens. Semantic colors use server-returned token ranges; the footer reports the span count or why colors are unavailable. Click **Language completions** (⌁) with the caret in code, or use the native Complete command (Option-Escape on macOS). Suggestions come from the language server and replace only the current source/selection snapshot through native undo. Snippet completions and additional/multi-file edits are excluded from this slice. Hover, diagnostics and the broader browser language action set are not connected in the Apple editor yet.

For Python in a fresh comparison workspace, install the real server once:

```sh
target/debug/hick lsp install python --root .dev/simple/project
```

Use your selected workspace path when `HICKORY_PROJECT_DIR` differs. The installer requests managed Python 3.12; system Python 3.9 cannot install current basedpyright. The server lives under that workspace's ignored `.hick-cache/servers/`. Missing servers leave text readable and report an unavailable/empty-result state, with Retry in the completion popover.

Android still exposes a **Language preview** popover using local keyword coloring and document-word suggestions, not a native LSP transport. Its keyword preview is bounded to 200,000 UTF-16 units. This remains UI feasibility evidence rather than semantic correctness.

All targets have a fixed-width minimap beside the editor. Click/tap to navigate; browser and Apple also support dragging. Browser keyboard users can focus the minimap and use Home/End; native minimaps expose accessibility adjustment. Browser and Android indicate the visible region. Apple currently shows document density without a viewport marker.

Rich literate rendering, backend capability menus, provenance, debugging, file operations, layout persistence and crash recovery are not included yet. This is an iteration baseline, not full parity with the old frontend. Native spellcheck, input methods and platform keyboard behavior may differ. No speed or memory superiority has been established by this scaffold.

## Empty pane cleanup, 2026-11-07

Closing a group’s last tab or moving it elsewhere removes that group and its split branch; surviving siblings keep their identities and proportions. Collapse runs after the complete operation, preserving same-group reorders. Retired native nodes are removed from the index, and focus falls back to a surviving group after close. Pending file opens choose a live destination if their original pane disappeared.

The focused checks pass 33 shared layout scenarios in TypeScript, Swift and Kotlin, including nested collapse, sole-tab docking, center merging and closing/reopening the last root. All 65 focused browser witnesses pass, including center merge reducing the pane count while preserving the editor, unsaved text, selection and undo. The native Mac walkthrough observed sole-tab edge docking without an empty source, closing the last document expanding Files, reopening after collapse, and a center merge to one group with visible text and undo restoring an unsaved edit to its saved baseline. No document was saved. This is manual Mac acceptance; iOS and Android gestures remain device acceptance work.

The earlier snapshots below describe behavior before automatic empty-pane cleanup.

## Native Apple verification after the reported UI failure, 2026-11-07

A native Mac walkthrough observed real right, left, top and bottom tab drops over the embedded editor. The Mac tracker now uses window-local drag/up events, not a container-of-buttons SwiftUI drag or a system pasteboard session. A follow-up walkthrough moved a right-hand tab into the center of the left pane: both documents became tabs in that existing group, the source group stayed empty, and no split was added. Dragging the joined tab before its neighbor also reordered the strip. The same walkthrough observed basedpyright returning 29 semantic spans in `debugging.md`, completed `quan` to `quantity`, and undid/redid the completion back to the saved buffer baseline without saving a document file. These are manual computer-use observations, not unattended UI tests.

The native-channel XCTest witness uses `HICKORY_NATIVE_LSP_ORIGIN` to connect to a real backend with the Python server installed. It checks semantic tokens and completion after changing plain Python source, and tokens/completion through Markdown projection. The focused real-clangd backend witness enters through document processing to ensure resync does not close a surviving plain file. Other platform gestures still need device acceptance.

The final center-drop checks passed 61 focused browser witnesses, 13 Swift witnesses (including both real Python language-channel cases), and 29 shared layout scenarios in Kotlin. Apple source type-checked for the iOS simulator, complete Android source compiled with Kotlin/Compose, and TypeScript checking and the browser bundle passed. Earlier verification snapshots below record their scope at that stage.

## Edge docking verification, 2026-11-07

All 27 shared layout scenarios and fifteen edge-coordinate cases passed in TypeScript, Swift and Kotlin. They check all four directions, resulting split axes and tab order, nested splits, empty groups and invalid payload/target/ID refusals. All 58 focused browser witnesses passed, including edge drops in all four directions while retaining the same editor, unsaved text, selection and undo. TypeScript checking and the browser bundle passed.

Ten Swift witnesses passed, Apple source type-checked for the iOS simulator, and complete Android source compiled with the installed Kotlin/Compose compiler. The real-backend browser driver now has all four edge-drop gestures, but this host still aborts Chromium during macOS bootstrap registration before the UI runs. Native provider delivery, gestures, hover geometry and layout remain device acceptance work; neither model parity nor synthetic DOM events prove them.

## Tab reordering verification, 2026-11-07

All eleven shared layout scenarios passed in TypeScript, Swift and Kotlin. They cover forward/backward ordering, append, cross-pane insertion, self-drops, stale anchors and foreign tab identities. All 23 focused browser witnesses passed, including workspace drag/drop events that verify insertion rather than parent-strip append, unchanged editor identity, retained unsaved text and working undo. TypeScript checking and the browser bundle passed.

The ten Swift witnesses passed, the shared Apple source type-checked for iOS simulator, and the complete Android source compiled with the installed Kotlin/Compose compiler. The real browser acceptance driver now drags tabs in both directions, but Chromium still aborts during macOS bootstrap registration before the UI runs. Native drag gestures and insertion-marker layout remain device acceptance work; compiled source and synthetic DOM events do not prove gesture equivalence. APK packaging remains subject to the Maven dependency limitation below.

## Language UI verification, 2026-11-07

TypeScript checking, the browser bundle and fifteen focused frontend witnesses passed, including delayed JSON-RPC semantic coloring and completion attachment after typing without losing selection or undo. Ten Swift witnesses passed, including UTF-16 keyword ranges and the large-file cutoff; the shared Apple source also type-checked for an iOS simulator. These are not native device UI acceptance receipts.

The real-backend browser suite includes minimap navigation, but this host still aborts Chromium at macOS bootstrap registration before the UI runs. Android APK compilation remains blocked before source compilation by Maven DNS failure for Kotlin build tools 2.2.10. The complete Android source compiled separately with the installed Kotlin/Compose compiler and cached AndroidX dependencies. This does not produce an APK. Native gestures, input methods, completion undo and visual layout still require device acceptance.

## Earlier baseline verification

Browser TypeScript checking, eleven focused browser witnesses, the production bundle, seven Swift witnesses, simulator-target Swift type checking and four shared Kotlin pane scenarios passed. A native-browser walkthrough confirmed a real plain save and undo after moving a split tab. Real PTY Unicode input, output and resize also passed.

The launch runner’s three target-selection and process-cleanup tests passed. The browser launch recipe served its HTML entry and real workspace API; Ctrl-C closed both ports. Native recipe attempts reached macOS app/emulator startup, but this execution host reported GUI bootstrap/notification service failures; device UI behavior remains unverified. Strict Clippy for the launcher was blocked by existing warnings in the CLI library.

The complete automated browser acceptance suite could not launch Chromium because this host refused macOS bootstrap registration. `xcodebuild` also aborted in the host before compilation. The complete Android source compiled with Studio’s bundled Kotlin and Compose compiler against the installed platform 37.0 and cached AndroidX dependencies. Gradle 9.3.1 is downloaded and the scoped commands find it in Studio’s cache. APK assembly remains unverified because shell DNS failures blocked remaining Maven dependencies, including Kotlin build tools 2.2.10. Run the platform builds after those tools are ready; model tests are not substitutes for them.

## Try a real language server

Use [the TypeScript playground](../../examples/simple-lsp/README.md) for local server setup and concrete completion, hover, semantic-color and diagnostic probes in the browser.

## Focused checks

- `cargo test --locked -p hickory-cli --example simple_dev`: launch-target selection and real subprocess cleanup.
- `just simple-test browser`: TypeScript checking, focused layout/save tests and the small production build.
- `just simple-test apple`: Swift model and save-protocol witnesses.
- `just simple-test android`: Gradle unit tests, followed by debug APK compilation.
- `just simple-build ios`: unsigned simulator build of the shared Apple source.
- `python3 scripts/test-simple-browser.py`: real browser flows against an isolated real backend. Requires the built `engine_client` example, Node dependencies and an installed Playwright Chromium.

The JSON pane scenarios in `apps/simple-contract/layout-cases.json` are executed by all three languages. They establish command-model parity, not device gesture equivalence. Native IME, accessibility, drag/drop and editor-state preservation still need platform UI acceptance. The Android SDK/Gradle build and iOS simulator build must pass before claiming those targets run successfully.

Build output stays in `target/`, `apps/web/dist-simple/` and ignored Android build directories. No stable release or installed old frontend is replaced. Broad regression checks remain a maintainer action; use `just checks-status` to inspect asynchronous post-commit checks.

All simple frontends use the existing DAP transport for plain files and Hickory file blocks. Choose the right-side Debug icon, then use the controls beside the paused line. Breakpoints, inline values, stack selection, watches and inline expression evaluation share the main debugger implementation. Save edits before launching. The session survives switching or moving tabs. On macOS, iOS and Android, choose a line’s right-side action button to open the floating grid. Choose Debug, then a file block and launch profile when requested. One active native session belongs to the workspace and survives tab switching. A paused line shows live values beside the rail; evaluate expressions and manage watches inside its popover. Locals belong to the stopped top frame; selecting a caller clears those locals while watches evaluate in the selected frame. Unsupported targets and missing adapters report setup details without blocking editing. Breakpoint dots distinguish pending, bound and refused stops.

Run the focused live witness with `python3 scripts/test-simple-browser.py --debug-only`; it needs a working Chromium host and Python debugger. On this host Chromium aborts during macOS Mach-port registration, and installing debugpy failed at PyPI DNS resolution. Live adapter and native UI acceptance remain unverified.

Focused native checks:

- `swift test --package-path apps/apple-simple --scratch-path target/apple-simple-spm --filter NativeDebugTests`
- `target/simple-android-gradle :app:testDebugUnitTest --tests com.hickorydocs.simple.NativeDebugTest` after the existing Gradle wrapper helper is built and dependencies are available.

Set `HICKORY_NATIVE_DEBUG_ORIGIN` to a reachable engine serving a `debug.py` fixture containing `value = 1`, `value += 1`, and `print(value)` on separate lines, with the Python adapter installed, to run the opt-in native transport witnesses. They use each actual native client through profile resolution, the binary debug channel, breakpoint pause, step, watch evaluation and stop, and assert the source buffer remains unchanged. Without that variable the live cases explicitly skip; controller fixtures still run.

Six browser witnesses, three Swift controller witnesses and one Kotlin controller witness passed for this extension. macOS built; the shared Apple source type-checked for iOS simulator; Android source and focused tests compiled with the cached Kotlin/Compose compiler. The Gradle compile attempt was blocked while resolving OkHttp from Maven by DNS failure; the cached compiler check is not an APK build. These checks do not prove native pointer geometry, IME, accessibility or device behavior.
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="docs/developers/simple-frontends.md"><ui:paste select="#simple-72e004e1b0-0"/></ui:file>

</ui:doc>
