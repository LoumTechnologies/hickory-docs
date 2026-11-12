<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Compare three small editor frontends

<ui:include file="contract.md" />
<ui:include file="inline-editing.md" />
<ui:include file="smart-enter.md" />

The Apple SwiftUI app, Android Jetpack Compose app and React browser entry share the existing engine, not the old workspace shell. Start with the same commands and safety boundaries on both. Files, documents, the workspace agent and terminal are the only pane kinds. Literate documents initially show editable source text; generated outputs remain readable but read-only. Inline DAP debugging now shares the existing engine protocol across the browser, macOS, iOS and Android. Inline presentation now reuses browser rich widgets and native source-attached controls; backend capability menus remain subsequent parity work.

Each client reads text before requesting optional capabilities. A save uses the revision loaded by that buffer, never a revision fetched just before writing. Failed saves retain edits. Moving a tab retains its buffer identity. Agent sends are explicit and are not retried automatically. The initial conversation uses the configured Python backend and refreshes completed turns; token streaming and other agent backends are outside this slice.

<ui:include file="apps/web/simple.html.md" />
<ui:include file="apps/web/vite.simple.config.ts.md" />
<ui:include file="apps/web/src/simple/layout.ts.md" />
<ui:include file="apps/web/src/simple/buffers.ts.md" />
<ui:include file="apps/web/src/simple/Document.tsx.md" />
<ui:include file="apps/web/src/simple/Agent.tsx.md" />
<ui:include file="apps/web/src/simple/App.tsx.md" />
<ui:include file="apps/web/src/simple/main.tsx.md" />
<ui:include file="apps/web/src/simple/simple.css.md" />
<ui:include file="apps/apple-simple/Package.swift.md" />
<ui:include file="apps/apple-simple/project.yml.md" />
<ui:include file="apps/apple-simple/Sources/Layout.swift.md" />
<ui:include file="apps/apple-simple/Sources/Engine.swift.md" />
<ui:include file="apps/apple-simple/Sources/Workspace.swift.md" />
<ui:include file="apps/apple-simple/Sources/NativeEditor.swift.md" />
<ui:include file="apps/apple-simple/Sources/AgentPane.swift.md" />
<ui:include file="apps/apple-simple/Sources/WorkspaceView.swift.md" />
<ui:include file="apps/apple-simple/Sources/HickorySimpleApp.swift.md" />
<ui:include file="apps/simple-contract/layout-cases.json.md" />
<ui:include file="apps/web/src/simple/parity.test.ts.md" />
<ui:include file="apps/apple-simple/Tests/ParityTests.swift.md" />
<ui:include file="apps/apple-simple/Tests/SaveTests.swift.md" />

<ui:include file="apps/android-simple/settings.gradle.kts.md" />
<ui:include file="apps/android-simple/build.gradle.kts.md" />
<ui:include file="apps/android-simple/gradle.properties.md" />
<ui:include file="apps/android-simple/app/build.gradle.kts.md" />
<ui:include file="apps/android-simple/app/src/main/AndroidManifest.xml.md" />
<ui:include file="apps/android-simple/app/src/main/java/com/hickorydocs/simple/PaneLayout.kt.md" />
<ui:include file="apps/android-simple/app/src/main/java/com/hickorydocs/simple/Engine.kt.md" />
<ui:include file="apps/android-simple/app/src/main/java/com/hickorydocs/simple/Workspace.kt.md" />
<ui:include file="apps/android-simple/app/src/main/java/com/hickorydocs/simple/MainActivity.kt.md" />
<ui:include file="apps/android-simple/app/src/main/java/com/hickorydocs/simple/Pane.kt.md" />
<ui:include file="apps/android-simple/app/src/main/java/com/hickorydocs/simple/AgentPane.kt.md" />
<ui:include file="apps/android-simple/app/src/test/java/com/hickorydocs/simple/ParityTest.kt.md" />
<ui:include file="apps/android-simple/gradle/wrapper/gradle-wrapper.properties.md" />
<ui:include file="scripts/test-simple-kotlin.py.md" />
<ui:include file="docs/developers/simple-frontends.md.md" />
<ui:include file=".github/workflows/simple-frontends.yml.md" />
<ui:include file="scripts/simple-browser-driver.mjs.md" />
<ui:include file="scripts/test-simple-browser.py.md" />
<ui:include file="apps/web/src/simple/Terminal.tsx.md" />
<ui:include file="apps/apple-simple/Sources/TerminalPane.swift.md" />
<ui:include file="apps/android-simple/app/src/main/java/com/hickorydocs/simple/TerminalPane.kt.md" />
<ui:include file="scripts/simple-android-gradle.rs.md" />
<ui:include file="crates/hickory-cli/examples/simple_dev.rs.md" />
<ui:include file="scripts/simple-window-layout.swift.md" />
<ui:include file="simple-lsp.md" />
<ui:include file="simple-showcase.md" />
</ui:doc>
