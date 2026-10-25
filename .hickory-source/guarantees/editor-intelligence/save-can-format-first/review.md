<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Save Can Format First, With The File's Own Formatter: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/lib/formatOnSave.ts](files/apps/web/src/lib/formatOnSave.ts.md)
- [apps/web/src/lsp/client.ts](files/apps/web/src/lsp/client.ts.md)
- [apps/web/src/lsp/cmLspFeatures.ts](files/apps/web/src/lsp/cmLspFeatures.ts.md)
- [apps/web/src/views/SettingsView.tsx](../../../shared/files/apps/web/src/views/SettingsView.tsx.md)
- [apps/web/src/views/WorkspaceView.tsx](../../../shared/files/apps/web/src/views/WorkspaceView.tsx.md)
- [crates/hick-lsp/src/backend.rs](../../../shared/files/crates/hick-lsp/src/backend.rs.md)
- [crates/hickory-cli/src/serve/api.rs](../../../shared/files/crates/hickory-cli/src/serve/api.rs.md)
- [crates/hickory-cli/src/serve/lsp_bridge.rs](../../../shared/files/crates/hickory-cli/src/serve/lsp_bridge.rs.md)
- [crates/hickory-cli/src/serve/mod.rs](../../../shared/files/crates/hickory-cli/src/serve/mod.rs.md)

## Tests and fixtures

- [apps/web/src/lsp/cmLspFeatures.test.ts](files/apps/web/src/lsp/cmLspFeatures.test.ts.md)
- [apps/web/src/views/SettingsView.test.tsx](../../../shared/files/apps/web/src/views/SettingsView.test.tsx.md)
- [crates/hickory-cli/tests/lsp_languages.rs](../../../shared/files/crates/hickory-cli/tests/lsp_languages.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
