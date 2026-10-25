<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Browser debugging runs edited source: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/debug/browser/compile.ts](files/apps/web/src/debug/browser/compile.ts.md)
- [apps/web/src/debug/browser/inspect.ts](files/apps/web/src/debug/browser/inspect.ts.md)
- [apps/web/src/debug/browser/runtime.ts](../../../shared/files/apps/web/src/debug/browser/runtime.ts.md)
- [apps/web/src/debug/browser/transport.ts](../../../shared/files/apps/web/src/debug/browser/transport.ts.md)
- [apps/web/src/debug/browser/worker.ts](files/apps/web/src/debug/browser/worker.ts.md)
- [apps/web/src/embed/DebuggableDocument.tsx](../../../support/files/apps/web/src/embed/DebuggableDocument.tsx.md)
- [apps/web/src/landing/demos/BrowserDebugDemo.tsx](../../../support/files/apps/web/src/landing/demos/BrowserDebugDemo.tsx.md)

## Tests and fixtures

- [apps/web/e2e/browserEmbedding.spec.ts](../../../shared/files/apps/web/e2e/browserEmbedding.spec.ts.md)
- [apps/web/src/debug/browser/runtime.test.ts](../../../shared/files/apps/web/src/debug/browser/runtime.test.ts.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
