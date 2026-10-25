<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# An Empty Response Never Kills a Session: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [crates/hickory-agent/src/llm_anthropic.rs](../../../support/files/crates/hickory-agent/src/llm_anthropic.rs.md)
- [crates/hickory-agent/src/llm_openai.rs](../../../shared/files/crates/hickory-agent/src/llm_openai.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: some-source-citations-do-not-resolve; no-existing-code-or-test-reference-resolved.

</book:doc>
