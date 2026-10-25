<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Browser-local prediction practice uses the shared gate: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/BrowserPredictionPractice.tsx](files/apps/web/src/components/BrowserPredictionPractice.tsx.md)
- [apps/web/src/components/WorkflowTour.tsx](files/apps/web/src/components/WorkflowTour.tsx.md)
- [apps/web/src/debug/browser/runtime.ts](../../../shared/files/apps/web/src/debug/browser/runtime.ts.md)
- [apps/web/src/debug/browser/transport.ts](../../../shared/files/apps/web/src/debug/browser/transport.ts.md)
- [apps/web/src/editor/generated/hick-lang/hick_lang.d.ts](files/apps/web/src/editor/generated/hick-lang/hick_lang.d.ts.md)
- [apps/web/src/editor/generated/hick-lang/hick_lang.js](files/apps/web/src/editor/generated/hick-lang/hick_lang.js.md)
- [apps/web/src/editor/generated/hick-lang/hick_lang_bg.wasm](../../../shared/files/apps/web/src/editor/generated/hick-lang/hick_lang_bg.wasm.md)
- [apps/web/src/editor/generated/hick-lang/hick_lang_bg.wasm.d.ts](files/apps/web/src/editor/generated/hick-lang/hick_lang_bg.wasm.d.ts.md)
- [apps/web/src/editor/hickLang.ts](../../../shared/files/apps/web/src/editor/hickLang.ts.md)
- [apps/web/src/embed/StaticUiSurface.tsx](files/apps/web/src/embed/StaticUiSurface.tsx.md)
- [apps/web/src/embed/uiHostSession.ts](../../../shared/files/apps/web/src/embed/uiHostSession.ts.md)
- [apps/web/src/lib/uiActions.ts](../../../shared/files/apps/web/src/lib/uiActions.ts.md)
- [apps/web/src/lib/uiReading.ts](../../../shared/files/apps/web/src/lib/uiReading.ts.md)
- [apps/web/src/tour/browserPrediction.ts](files/apps/web/src/tour/browserPrediction.ts.md)
- [apps/web/src/views/LandingView.tsx](../../../shared/files/apps/web/src/views/LandingView.tsx.md)
- [crates/hick-lang-wasm/src/lib.rs](../../../shared/files/crates/hick-lang-wasm/src/lib.rs.md)
- [crates/hick-lang-wasm/src/prediction.rs](../../../shared/files/crates/hick-lang-wasm/src/prediction.rs.md)
- [crates/hick-observe/src/lib.rs](../../../shared/files/crates/hick-observe/src/lib.rs.md)
- [crates/hick-observe/src/prediction.rs](../../../shared/files/crates/hick-observe/src/prediction.rs.md)

## Tests and fixtures

- [apps/web/e2e/staticUiParity.spec.ts](../../../shared/files/apps/web/e2e/staticUiParity.spec.ts.md)
- [apps/web/src/components/BrowserPredictionPractice.test.tsx](../../../shared/files/apps/web/src/components/BrowserPredictionPractice.test.tsx.md)
- [apps/web/src/lib/uiActions.test.ts](../../../shared/files/apps/web/src/lib/uiActions.test.ts.md)
- [apps/web/src/tour/browserPrediction.test.ts](../../../shared/files/apps/web/src/tour/browserPrediction.test.ts.md)
- [crates/hick-observe/src/prediction_tests.rs](../../../shared/files/crates/hick-observe/src/prediction_tests.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
