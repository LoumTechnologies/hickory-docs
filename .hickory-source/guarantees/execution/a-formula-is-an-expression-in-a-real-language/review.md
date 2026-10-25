<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Formula Is An Expression In A Real Language: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/FormulaDebugger.tsx](../../../shared/files/apps/web/src/components/FormulaDebugger.tsx.md)
- [apps/web/src/components/TablePanel.tsx](../../../shared/files/apps/web/src/components/TablePanel.tsx.md)
- [apps/web/src/lib/cellRef.ts](files/apps/web/src/lib/cellRef.ts.md)
- [apps/web/src/lib/formulaPoint.ts](files/apps/web/src/lib/formulaPoint.ts.md)
- [crates/hick-formula/src/backend.rs](files/crates/hick-formula/src/backend.rs.md)
- [crates/hick-formula/src/evaluate.rs](../../../shared/files/crates/hick-formula/src/evaluate.rs.md)
- [crates/hick-formula/src/graph.rs](files/crates/hick-formula/src/graph.rs.md)
- [crates/hick-formula/src/protocol.rs](files/crates/hick-formula/src/protocol.rs.md)
- [crates/hick-formula/src/session.rs](files/crates/hick-formula/src/session.rs.md)
- [crates/hickory-cli/src/main.rs](../../../shared/files/crates/hickory-cli/src/main.rs.md)
- [crates/hickory-cli/src/serve/formula.rs](../../../shared/files/crates/hickory-cli/src/serve/formula.rs.md)

## Tests and fixtures

- [apps/web/src/components/FormulaDebugger.test.tsx](../../../shared/files/apps/web/src/components/FormulaDebugger.test.tsx.md)
- [apps/web/src/components/TablePanel.test.tsx](../../../shared/files/apps/web/src/components/TablePanel.test.tsx.md)
- [apps/web/src/lib/formulaPoint.test.ts](files/apps/web/src/lib/formulaPoint.test.ts.md)
- [crates/hick-formula/tests/backends.rs](../../../shared/files/crates/hick-formula/tests/backends.rs.md)
- [crates/hickory-cli/tests/serve_formula.rs](../../../shared/files/crates/hickory-cli/tests/serve_formula.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
