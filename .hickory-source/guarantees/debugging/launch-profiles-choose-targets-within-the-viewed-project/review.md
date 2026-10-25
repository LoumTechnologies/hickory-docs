<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Launch profiles choose targets within the viewed project: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/debug/breakpointPreferences.ts](../../../support/files/apps/web/src/debug/breakpointPreferences.ts.md)
- [apps/web/src/debug/launchProfiles.tsx](../../../shared/files/apps/web/src/debug/launchProfiles.tsx.md)
- [apps/web/src/debug/plainDebugHosts.tsx](files/apps/web/src/debug/plainDebugHosts.tsx.md)
- [apps/web/src/debug/projectDebugSurface.tsx](files/apps/web/src/debug/projectDebugSurface.tsx.md)
- [apps/web/src/editor/generated/hick-lang/hick_lang_bg.wasm](../../../shared/files/apps/web/src/editor/generated/hick-lang/hick_lang_bg.wasm.md)
- [crates/hick-lang/src/project/evaluate.rs](../../../shared/files/crates/hick-lang/src/project/evaluate.rs.md)
- [crates/hickory-cli/src/debug_profiles.rs](files/crates/hickory-cli/src/debug_profiles.rs.md)
- [crates/hickory-cli/src/launch_profiles.rs](files/crates/hickory-cli/src/launch_profiles.rs.md)
- [crates/hickory-cli/src/serve/launch_profiles.rs](files/crates/hickory-cli/src/serve/launch_profiles.rs.md)

## Tests and fixtures

- [apps/web/e2e/literate-editor.spec.ts](../../../shared/files/apps/web/e2e/literate-editor.spec.ts.md)
- [apps/web/src/debug/launchProfiles.test.tsx](../../../shared/files/apps/web/src/debug/launchProfiles.test.tsx.md)
- [apps/web/src/debug/plainDebugHosts.test.tsx](files/apps/web/src/debug/plainDebugHosts.test.tsx.md)
- [apps/web/src/debug/projectDebugSurface.test.tsx](../../../shared/files/apps/web/src/debug/projectDebugSurface.test.tsx.md)
- [crates/hick-lang/tests/project_debug.rs](../../../shared/files/crates/hick-lang/tests/project_debug.rs.md)
- [crates/hickory-cli/tests/launch_profiles.rs](files/crates/hickory-cli/tests/launch_profiles.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
