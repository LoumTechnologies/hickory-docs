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
<ui:include file="editor-mounts.md" />
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
<ui:include file="scripts/test-native-debug.py.md" />
<ui:include file="apps/web/src/simple/Terminal.tsx.md" />
<ui:include file="apps/apple-simple/Sources/TerminalPane.swift.md" />
<ui:include file="apps/android-simple/app/src/main/java/com/hickorydocs/simple/TerminalPane.kt.md" />
<ui:include file="scripts/simple-android-gradle.rs.md" />
<ui:include file="crates/hickory-cli/examples/simple_dev.rs.md" />
<ui:include file="scripts/simple-window-layout.swift.md" />
<ui:include file="simple-lsp.md" />
<ui:include file="simple-showcase.md" />
<ui:include file="scripts/test-simple-mobile-reload.py.md" />

<ui:include file="intelligence.md" />

<ui:include file="workspace-languages.md" />

<ui:include file="problems.md" />

<ui:include file="workspace-search-browser.md" />

<ui:include file="workspace-search-apple.md" />

<ui:include file="workspace-search-android.md" />

<ui:include file="change-plans-browser.md" />

<ui:include file="change-plans-apple.md" />

<ui:include file="change-plans-android.md" />

<ui:include file="change-plan-cases.md" />

<ui:include file="workspace-changes-browser.md" />

<ui:include file="workspace-changes-apple.md" />

<ui:include file="workspace-changes-android.md" />

<ui:include file="diagnostic-revisions.md" />

<ui:include file="language-connection-apple.md" />

<ui:include file="language-connection-android.md" />

<ui:include file="language-wire-fixture.md" />

<ui:include file="workspace-analysis-browser.md" />

<ui:include file="workspace-analysis-apple.md" />

<ui:include file="workspace-analysis-android.md" />

<ui:include file="workspace-analysis-browser-tests.md" />

<ui:include file="workspace-analysis-apple-tests.md" />

<ui:include file="workspace-analysis-android-tests.md" />

<ui:include file="workspace-analysis-browser-view.md" />

<ui:include file="workspace-analysis-apple-view.md" />

<ui:include file="workspace-analysis-android-view.md" />

<ui:include file="workspace-commands.md" />

<ui:include file="workspace-commands-browser.md" />

<ui:include file="workspace-commands-apple.md" />

<ui:include file="workspace-commands-android.md" />

<ui:include file="workspace-commands-browser-tests.md" />

<ui:include file="workspace-commands-apple-tests.md" />

<ui:include file="workspace-commands-android-tests.md" />

<ui:include file="search-everywhere-browser.md" />

<ui:include file="search-everywhere-apple.md" />

<ui:include file="search-everywhere-android.md" />

<ui:include file="search-everywhere-browser-tests.md" />

<ui:include file="search-everywhere-apple-tests.md" />

<ui:include file="search-everywhere-android-tests.md" />

<ui:include file="workspace-command-adapters-apple.md" />

<ui:include file="workspace-command-adapters-android.md" />

<ui:include file="search-everywhere-apple-view.md" />

<ui:include file="search-everywhere-android-view.md" />

<ui:include file="search-everywhere-browser-view.md" />

<ui:include file="search-everywhere-apple-view-tests.md" />

<ui:include file="workspace-command-button-apple.md" />

<ui:include file="test-sources-browser.md" />

<ui:include file="test-sources-apple.md" />

<ui:include file="test-sources-android.md" />

<ui:include file="test-source-view-browser.md" />

<ui:include file="test-source-view-apple.md" />

<ui:include file="test-source-view-android.md" />

<ui:include file="test-source-browser-witness.md" />

<ui:include file="test-source-index-browser.md" />

<ui:include file="test-source-index-apple.md" />

<ui:include file="test-source-index-android.md" />

<ui:include file="source-test-gutters-browser.md" />

<ui:include file="source-test-gutters-apple.md" />

<ui:include file="source-test-gutters-android.md" />

<ui:include file="coverage-view-browser.md" />

<ui:include file="coverage-view-apple.md" />

<ui:include file="coverage-view-android.md" />

<ui:include file="coverage-browser-witness.md" />

<ui:include file="source-coverage-browser.md" />

<ui:include file="action-chooser-browser-witness.md" />

<ui:include file="git-review-browser.md" />

<ui:include file="git-review-apple.md" />

<ui:include file="git-review-android.md" />

<ui:include file="git-history-browser.md" />

<ui:include file="git-history-apple.md" />

<ui:include file="git-history-android.md" />

<ui:include file="git-attribution-browser.md" />

<ui:include file="git-attribution-apple.md" />

<ui:include file="git-attribution-android.md" />

<ui:include file="construction-reading-browser.md" />

<ui:include file="construction-reading-apple.md" />

<ui:include file="construction-reading-android.md" />

<ui:include file="source-cleanup-acceptance.md" />

<ui:include file="git-conflict-browser.md" />

<ui:include file="git-conflict-apple.md" />

<ui:include file="git-conflict-android.md" />

<ui:include file="workspace-change-events-browser.md" />

<ui:include file="source-recovery.md" />

<ui:include file="resource-editors.md" />

<ui:include file="debugger-source-guards-browser.md" />

<ui:include file="debugger-values-browser.md" />

<ui:include file="test-debug-android.md" />

<ui:include file="source-templates-browser.md" />

<ui:include file="source-templates-android.md" />

<ui:include file="source-templates-apple.md" />

<ui:include file="scripts/test-native-templates.py.md" />

<ui:include file="source-history-apple.md" />
<ui:include file="source-history-browser.md" />

<ui:include file="git-conflict-resolution-android.md" />

<ui:include file="git-conflict-resolution-browser.md" />

<ui:include file="source-history-android.md" />

<ui:include file="behavior-readings-android.md" />

<ui:include file="debug-adapter-sources-browser.md" />

<ui:include file="debug-adapter-sources-android.md" />

<ui:include file="source-history-browser-witness.md" />

<ui:include file="debug-adapter-sources-apple.md" />

<ui:include file="git-resolution-browser-witness.md" />

<ui:include file="source-create-android.md" />

<ui:include file="behavior-readings-apple.md" />

<ui:include file="source-identity-apple.md" />

<ui:include file="agent-proposals-android.md" />

<ui:include file="agent-proposals-browser.md" />

<ui:include file="agent-proposal-browser-witness.md" />

<ui:include file="debug-attach-apple.md" />

<ui:include file="agent-proposals-apple.md" />

<ui:include file="debug-attach-android.md" />

</ui:doc>
