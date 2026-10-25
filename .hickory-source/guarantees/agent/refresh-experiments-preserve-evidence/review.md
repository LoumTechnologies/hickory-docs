<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Local refresh experiments preserve historical request evidence: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/agent/src/hickory_agent_worker/context.py](../python-requests-have-validated-context/files/apps/agent/src/hickory_agent_worker/context.py.md)
- [apps/agent/src/hickory_agent_worker/refresh.py](../../../support/files/apps/agent/src/hickory_agent_worker/refresh.py.md)
- [apps/agent/src/hickory_agent_worker/refresh_experiment.py](../../../support/files/apps/agent/src/hickory_agent_worker/refresh_experiment.py.md)

## Tests and fixtures

- [apps/agent/tests/test_context_runtime.py](../python-requests-have-validated-context/files/apps/agent/tests/test_context_runtime.py.md)
- [apps/agent/tests/test_refresh_experiment.py](files/apps/agent/tests/test_refresh_experiment.py.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</book:doc>
