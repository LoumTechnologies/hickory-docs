<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A `hick:file` Is A Generated File However Deeply It Is Nested: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [crates/hick-dap/src/program.rs](../../../shared/files/crates/hick-dap/src/program.rs.md)
- [crates/hick-dap/src/session.rs](../../../shared/files/crates/hick-dap/src/session.rs.md)
- [crates/hick-literate/src/lib.rs](../../../shared/files/crates/hick-literate/src/lib.rs.md)
- [crates/hick-literate/src/pipeline.rs](files/crates/hick-literate/src/pipeline.rs.md)
- [crates/hick-lsp/src/document.rs](../../../shared/files/crates/hick-lsp/src/document.rs.md)
- [crates/hick-lsp/src/position_map.rs](../../../shared/files/crates/hick-lsp/src/position_map.rs.md)
- [crates/hick-lsp/src/virtual_file.rs](files/crates/hick-lsp/src/virtual_file.rs.md)

## Tests and fixtures

- [crates/hick-dap/tests/live_capture.rs](../../../shared/files/crates/hick-dap/tests/live_capture.rs.md)
- [crates/hick-dap/tests/live_session.rs](../../../shared/files/crates/hick-dap/tests/live_session.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
