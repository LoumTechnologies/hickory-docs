<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Claude Code Transcript Imports As A Session: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/SessionTurns.tsx](../../../shared/files/apps/web/src/components/SessionTurns.tsx.md)
- [apps/web/src/editor/folding.ts](../../../shared/files/apps/web/src/editor/folding.ts.md)
- [apps/web/src/editor/wysiwyg.ts](../../../shared/files/apps/web/src/editor/wysiwyg.ts.md)
- [crates/hick-lang/src/lib.rs](../../../shared/files/crates/hick-lang/src/lib.rs.md)
- [crates/hickory-agent/src/session_view.rs](../../../shared/files/crates/hickory-agent/src/session_view.rs.md)
- [crates/hickory-cli/src/claude_code.rs](files/crates/hickory-cli/src/claude_code.rs.md)
- [crates/hickory-cli/src/claude_code/tests.rs](files/crates/hickory-cli/src/claude_code/tests.rs.md)
- [crates/hickory-cli/src/main.rs](../../../shared/files/crates/hickory-cli/src/main.rs.md)

## Tests and fixtures

- [crates/hickory-cli/tests/fixtures/claude-code-small.jsonl](files/crates/hickory-cli/tests/fixtures/claude-code-small.jsonl.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
