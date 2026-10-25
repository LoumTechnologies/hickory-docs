<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A desktop app bundles its own Python agent: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/desktop/src-tauri/src/server.rs](../../../shared/files/apps/desktop/src-tauri/src/server.rs.md)
- [apps/web/src/views/AgentSettings.tsx](files/apps/web/src/views/AgentSettings.tsx.md)
- [crates/hickory-cli/src/engine/daemon.rs](../../../shared/files/crates/hickory-cli/src/engine/daemon.rs.md)
- [crates/hickory-cli/src/engine/mod.rs](../../../shared/files/crates/hickory-cli/src/engine/mod.rs.md)
- [crates/hickory-cli/src/serve/mod.rs](../../../shared/files/crates/hickory-cli/src/serve/mod.rs.md)
- [crates/hickory-cli/src/serve/python_agent.rs](../../../shared/files/crates/hickory-cli/src/serve/python_agent.rs.md)
- [scripts/build-python-agent-runtime.sh](../../../shared/files/scripts/build-python-agent-runtime.sh.md)
- [scripts/dist-desktop.sh](../../../shared/files/scripts/dist-desktop.sh.md)

## Tests and fixtures

- [apps/web/src/views/AgentSettings.test.tsx](../../../shared/files/apps/web/src/views/AgentSettings.test.tsx.md)
- [crates/hickory-cli/tests/serve_python_agent.rs](../../../shared/files/crates/hickory-cli/tests/serve_python_agent.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
