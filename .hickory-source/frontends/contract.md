<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Keep the comparison frontends equivalent within their declared slice

For maintainers extending the new frontends without silently changing the comparison.

Given a reachable existing Hickory engine, each frontend exposes the same four pane kinds: files, text document, workspace agent and terminal. Plain and literate source open as text. Generated outputs open read-only in this slice. File and pane commands remain in menus or tab strips; editor content has no permanent feature toolbar.

Opening text does not wait for rendering, provenance, debugging or language services. Pane splitting, resizing and moving retain document buffer identities and unsent agent messages. Both drag and menu movement are available. Dirty document tabs refuse to close.

A plain save includes its original content hash. A literate save includes its original source as both room and disk expectation. Failed saves retain edits. A successful reply marks only the submitted snapshot saved; typing during that request remains dirty. Save and agent mutations are never automatically replayed by these frontends after transport failure.

These are the initial frontend requirements, not full parity with the old frontend. Rich literate views, minimaps, capability menus, provenance, debugging, additional agent backends, settings, durable layout and crash recovery are outside this slice. A framework build or shared model fixture cannot establish native input-method, accessibility, gesture or latency equivalence.

---

Last verification: 2026-11-07, Codex. Partially verified. Eleven TypeScript witnesses, seven Swift witnesses and four shared Kotlin pane scenarios passed. The complete Apple source type-checked for an iOS simulator target. A native-browser walkthrough observed a plain-file save and undo after splitting and moving its tab. A real PTY check observed Unicode input, screen output and resize, then closed its test session.

The Python browser acceptance suite could start its isolated real engine but Chromium launch aborted while registering a macOS bootstrap service; the complete UI suite is not a passing receipt. Xcode project builds also aborted in this host before compiling, although Swift package compilation and simulator-target type checking succeeded. The complete Android source compiled with Android Studio’s bundled Kotlin and Compose compiler. APK assembly remains unverified: Gradle is available from Studio’s cache, but shell DNS failures prevented downloading remaining Maven artifacts, including Kotlin build tools 2.2.10. The build is now aligned with the installed SDK platform 37.0 and Android Studio JBR 25 through AGP 9.1.1 and Gradle 9.3.1. Native input methods, accessibility and gestures remain UI acceptance work. No speed or memory advantage has been measured.

Implementation and focused tests are constructed in the [frontend index](index.md). The [developer guide](../../docs/developers/simple-frontends.md) records commands and verification limits.

</ui:doc>
