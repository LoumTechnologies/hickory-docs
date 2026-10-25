<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# New Project Offers The Scaffolder's Own Options, Not A List This Product Maintains: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [apps/web/src/components/NewProjectDialog.tsx](../../../shared/files/apps/web/src/components/NewProjectDialog.tsx.md)
- [crates/hick-plugins/src/scaffold.rs](../../../support/files/crates/hick-plugins/src/scaffold.rs.md)
- [crates/hickory-cli/src/scaffold.rs](../../../shared/files/crates/hickory-cli/src/scaffold.rs.md)
- [crates/hickory-cli/src/serve/scaffold.rs](../../../shared/files/crates/hickory-cli/src/serve/scaffold.rs.md)

## Tests and fixtures

- [apps/web/src/components/NewProjectDialog.test.tsx](../../../shared/files/apps/web/src/components/NewProjectDialog.test.tsx.md)
- [crates/hickory-cli/tests/clap_help_live.rs](files/crates/hickory-cli/tests/clap_help_live.rs.md)
- [crates/hickory-cli/tests/scaffold_templates.rs](../../../shared/files/crates/hickory-cli/tests/scaffold_templates.rs.md)
- [crates/hickory-cli/tests/serve_scaffold.rs](../../../shared/files/crates/hickory-cli/tests/serve_scaffold.rs.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
