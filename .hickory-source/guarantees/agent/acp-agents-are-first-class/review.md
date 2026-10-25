<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# ACP Agents Work In The Agent Pane And Leave A Hickory Record: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/AcpControls.tsx](../python-execution-has-two-boundaries/files/apps/web/src/components/AcpControls.tsx.md)
- [apps/web/src/components/AcpRecord.tsx](../../../support/files/apps/web/src/components/AcpRecord.tsx.md)
- [apps/web/src/components/AgentPicker.tsx](../../verification/predictions-precede-managed-observation/files/apps/web/src/components/AgentPicker.tsx.md)
- [apps/web/src/components/ChatDock.tsx](../../../shared/files/apps/web/src/components/ChatDock.tsx.md)
- [apps/web/src/views/AgentSettings.tsx](../a-desktop-app-bundles-its-python-agent/files/apps/web/src/views/AgentSettings.tsx.md)
- [crates/hickory-cli/src/engine/client.rs](../../../shared/files/crates/hickory-cli/src/engine/client.rs.md)
- [crates/hickory-cli/src/engine/watch.rs](../../../support/files/crates/hickory-cli/src/engine/watch.rs.md)
- [crates/hickory-cli/src/serve/acp/config.rs](files/crates/hickory-cli/src/serve/acp/config.rs.md)
- [crates/hickory-cli/src/serve/acp/record.rs](../../../shared/files/crates/hickory-cli/src/serve/acp/record.rs.md)
- [crates/hickory-cli/src/serve/acp/workspace.rs](files/crates/hickory-cli/src/serve/acp/workspace.rs.md)
- [crates/hickory-cli/src/serve/agent.rs](../../../shared/files/crates/hickory-cli/src/serve/agent.rs.md)
- [crates/hickory-cli/src/up/state.rs](../../../shared/files/crates/hickory-cli/src/up/state.rs.md)

## Tests and fixtures

- [apps/web/src/components/AcpControls.test.tsx](../../../shared/files/apps/web/src/components/AcpControls.test.tsx.md)
- [apps/web/src/components/ChatDock.test.tsx](../../../shared/files/apps/web/src/components/ChatDock.test.tsx.md)
- [apps/web/src/components/FeatureSettings.test.tsx](../../../shared/files/apps/web/src/components/FeatureSettings.test.tsx.md)
- [crates/hickory-cli/tests/engine_lifecycle.rs](../../../shared/files/crates/hickory-cli/tests/engine_lifecycle.rs.md)
- [crates/hickory-cli/tests/serve_acp.rs](../../../shared/files/crates/hickory-cli/tests/serve_acp.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</book:doc>
