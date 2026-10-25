<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Observations read beside their captured source: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/playwright.embed.config.ts](../../../support/files/apps/web/playwright.embed.config.ts.md)
- [apps/web/src/debug/ObservationReading.tsx](files/apps/web/src/debug/ObservationReading.tsx.md)
- [apps/web/src/debug/browser/runtime.ts](../../../shared/files/apps/web/src/debug/browser/runtime.ts.md)
- [apps/web/src/debug/browser/transport.ts](../../../shared/files/apps/web/src/debug/browser/transport.ts.md)
- [apps/web/src/debug/observations.ts](files/apps/web/src/debug/observations.ts.md)
- [apps/web/src/views/WorkspaceChat.tsx](../../agent/unsent-responses-survive-restart/files/apps/web/src/views/WorkspaceChat.tsx.md)
- [crates/hick-dap/src/mapping.rs](../../../shared/files/crates/hick-dap/src/mapping.rs.md)
- [crates/hick-observe/src/reading.rs](files/crates/hick-observe/src/reading.rs.md)
- [crates/hickory-cli/src/debug_reading.rs](files/crates/hickory-cli/src/debug_reading.rs.md)
- [crates/hickory-cli/src/python_agent.rs](../../../shared/files/crates/hickory-cli/src/python_agent.rs.md)
- [crates/hickory-cli/src/serve/debug_bridge.rs](../../../shared/files/crates/hickory-cli/src/serve/debug_bridge.rs.md)

## Tests and fixtures

- [apps/web/e2e/browserEmbedding.spec.ts](../../../shared/files/apps/web/e2e/browserEmbedding.spec.ts.md)
- [apps/web/src/debug/ObservationReading.test.tsx](../../../shared/files/apps/web/src/debug/ObservationReading.test.tsx.md)
- [apps/web/src/debug/browser/runtime.test.ts](../../../shared/files/apps/web/src/debug/browser/runtime.test.ts.md)
- [apps/web/src/debug/fixtures/reading.json](files/apps/web/src/debug/fixtures/reading.json.md)
- [apps/web/src/debug/inlineEval.test.tsx](files/apps/web/src/debug/inlineEval.test.tsx.md)
- [apps/web/src/debug/providerObservations.test.ts](files/apps/web/src/debug/providerObservations.test.ts.md)
- [crates/hickory-cli/src/serve/debug_bridge/tests/tests.rs](files/crates/hickory-cli/src/serve/debug_bridge/tests/tests.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
