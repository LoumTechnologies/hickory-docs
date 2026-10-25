<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Private edit proposals open review before writing: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/agent/src/hickory_agent_worker/core.py](files/apps/agent/src/hickory_agent_worker/core.py.md)
- [apps/agent/src/hickory_agent_worker/instructions.py](../../../shared/files/apps/agent/src/hickory_agent_worker/instructions.py.md)
- [apps/web/src/components/PrivatePredictionCoach.tsx](../../../shared/files/apps/web/src/components/PrivatePredictionCoach.tsx.md)
- [crates/hickory-agent/src/llm_openai.rs](../../../shared/files/crates/hickory-agent/src/llm_openai.rs.md)
- [crates/hickory-agent/src/provider.rs](../../../shared/files/crates/hickory-agent/src/provider.rs.md)
- [crates/hickory-cli/src/engine/mcp.rs](../../../shared/files/crates/hickory-cli/src/engine/mcp.rs.md)
- [crates/hickory-cli/src/serve/private_predictions.rs](../../../shared/files/crates/hickory-cli/src/serve/private_predictions.rs.md)
- [scripts/test-private-prediction-ui.sh](../../../shared/files/scripts/test-private-prediction-ui.sh.md)

## Tests and fixtures

- [apps/agent/tests/test_worker.py](../../../shared/files/apps/agent/tests/test_worker.py.md)
- [apps/web/e2e/privatePredictionCoach.spec.ts](../../../shared/files/apps/web/e2e/privatePredictionCoach.spec.ts.md)
- [apps/web/src/components/PrivatePredictionCoach.test.tsx](../../../shared/files/apps/web/src/components/PrivatePredictionCoach.test.tsx.md)
- [crates/hickory-cli/src/serve/private_prediction_tests.rs](../../../shared/files/crates/hickory-cli/src/serve/private_prediction_tests.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
