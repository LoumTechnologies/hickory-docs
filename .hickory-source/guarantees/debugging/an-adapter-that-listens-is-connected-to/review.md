<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Use the adapter’s transport and the current measured language support: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [crates/hick-dap/src/adapter.rs](../../../shared/files/crates/hick-dap/src/adapter.rs.md)
- [crates/hick-dap/src/build.rs](../../../shared/files/crates/hick-dap/src/build.rs.md)
- [crates/hick-dap/src/discovery.rs](../../../shared/files/crates/hick-dap/src/discovery.rs.md)
- [crates/hick-dap/src/session.rs](../../../shared/files/crates/hick-dap/src/session.rs.md)
- [crates/hick-plugins/src/build.rs](../a-plain-file-has-the-same-debugger/files/crates/hick-plugins/src/build.rs.md)
- [crates/hick-plugins/src/dap_discovery.rs](../../../support/files/crates/hick-plugins/src/dap_discovery.rs.md)
- [crates/hick-plugins/src/languages/typescript.rs](../../../support/files/crates/hick-plugins/src/languages/typescript.rs.md)

## Tests and fixtures

- [crates/hick-dap/tests/debug_coverage.rs](files/crates/hick-dap/tests/debug_coverage.rs.md)
- [crates/hick-dap/tests/live_session_c.rs](../a-compiled-language-launches-what-a-build-produced/files/crates/hick-dap/tests/live_session_c.rs.md)
- [crates/hick-dap/tests/live_session_cpp.rs](../a-compiled-language-launches-what-a-build-produced/files/crates/hick-dap/tests/live_session_cpp.rs.md)
- [crates/hick-dap/tests/live_session_go.rs](../../../shared/files/crates/hick-dap/tests/live_session_go.rs.md)

- [crates/hick-plugins/src/languages/dap.rs](../../../support/files/crates/hick-plugins/src/languages/dap.rs.md)

- [crates/hick-plugins/src/languages/react.rs](../../../support/files/crates/hick-plugins/src/languages/react.rs.md)

- [crates/hick-plugins/src/languages/ruby.rs](../../../support/files/crates/hick-plugins/src/languages/ruby.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
