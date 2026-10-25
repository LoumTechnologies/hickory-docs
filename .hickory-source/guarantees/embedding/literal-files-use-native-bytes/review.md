<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Literal files use native bytes: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/debug/browser/compile.ts](../../debugging/browser-debugging-runs-edited-source/files/apps/web/src/debug/browser/compile.ts.md)
- [crates/hick-lang-wasm/src/lib.rs](../../../shared/files/crates/hick-lang-wasm/src/lib.rs.md)
- [crates/hick-lang/src/literal_files.rs](files/crates/hick-lang/src/literal_files.rs.md)

## Tests and fixtures

- [apps/web/src/debug/browser/runtime.test.ts](../../../shared/files/apps/web/src/debug/browser/runtime.test.ts.md)
- [apps/web/src/elements/patch/view.test.tsx](../../../shared/files/apps/web/src/elements/patch/view.test.tsx.md)
- [apps/web/src/embed/DocumentEmbed.test.tsx](../../../shared/files/apps/web/src/embed/DocumentEmbed.test.tsx.md)
- [crates/hick-lang/tests/project.rs](../../../shared/files/crates/hick-lang/tests/project.rs.md)
- [crates/hick-literate/tests/browser_literal_files.rs](files/crates/hick-literate/tests/browser_literal_files.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
