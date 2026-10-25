<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Launch the artifact built from the selected compiled project: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/debug/DebugStrip.tsx](../../../shared/files/apps/web/src/debug/DebugStrip.tsx.md)
- [apps/web/src/debug/languages.ts](files/apps/web/src/debug/languages.ts.md)
- [crates/hick-dap/src/build.rs](../../../shared/files/crates/hick-dap/src/build.rs.md)
- [crates/hick-dap/src/capture.rs](../../../shared/files/crates/hick-dap/src/capture.rs.md)
- [crates/hick-dap/src/discovery.rs](../../../shared/files/crates/hick-dap/src/discovery.rs.md)
- [crates/hick-lsp/src/lang_detect.rs](../../../shared/files/crates/hick-lsp/src/lang_detect.rs.md)
- [crates/hick-plugins/src/build.rs](../a-plain-file-has-the-same-debugger/files/crates/hick-plugins/src/build.rs.md)
- [crates/hick-plugins/src/dap_discovery.rs](../../../support/files/crates/hick-plugins/src/dap_discovery.rs.md)
- [crates/hick-plugins/src/routing.rs](../../../support/files/crates/hick-plugins/src/routing.rs.md)
- [crates/hickory-cli/src/dap_install.rs](../../../shared/files/crates/hickory-cli/src/dap_install.rs.md)
- [crates/hickory-cli/src/debug_sessions.rs](../../../shared/files/crates/hickory-cli/src/debug_sessions.rs.md)
- [crates/hickory-cli/src/mcp.rs](../../../shared/files/crates/hickory-cli/src/mcp.rs.md)
- [crates/hickory-cli/src/serve/debug_bridge.rs](../../../shared/files/crates/hickory-cli/src/serve/debug_bridge.rs.md)

## Tests and fixtures

- [apps/web/src/debug/DebugStrip.test.tsx](../../../shared/files/apps/web/src/debug/DebugStrip.test.tsx.md)
- [apps/web/src/debug/cmDebug.test.ts](../../../shared/files/apps/web/src/debug/cmDebug.test.ts.md)
- [crates/hick-dap/tests/csharp_document.rs](files/crates/hick-dap/tests/csharp_document.rs.md)
- [crates/hick-dap/tests/live_session_c.rs](files/crates/hick-dap/tests/live_session_c.rs.md)
- [crates/hick-dap/tests/live_session_cpp.rs](files/crates/hick-dap/tests/live_session_cpp.rs.md)
- [crates/hick-dap/tests/live_session_csharp.rs](../../../shared/files/crates/hick-dap/tests/live_session_csharp.rs.md)
- [crates/hick-dap/tests/live_session_go.rs](../../../shared/files/crates/hick-dap/tests/live_session_go.rs.md)
- [crates/hick-dap/tests/live_session_rust.rs](files/crates/hick-dap/tests/live_session_rust.rs.md)
- [crates/hick-lang/tests/fixtures/efcore-project/app/Program.cs](../../../support/files/crates/hick-lang/tests/fixtures/efcore-project/app/Program.cs.md)
- [crates/hick-term/tests/full_screen_apps.rs](../../../shared/files/crates/hick-term/tests/full_screen_apps.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
