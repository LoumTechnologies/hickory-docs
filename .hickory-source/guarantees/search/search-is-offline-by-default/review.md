<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Search Works Offline; Only An Explicit Install Touches The Network: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [crates/hick-search/src/lib.rs](files/crates/hick-search/src/lib.rs.md)
- [crates/hickory-cli/src/main.rs](../../../shared/files/crates/hickory-cli/src/main.rs.md)
- [crates/hickory-cli/src/mcp.rs](../../../shared/files/crates/hickory-cli/src/mcp.rs.md)
- [crates/hickory-cli/src/search_install.rs](files/crates/hickory-cli/src/search_install.rs.md)
- [crates/hickory-cli/src/serve/api.rs](../../../shared/files/crates/hickory-cli/src/serve/api.rs.md)

## Tests and fixtures

- [apps/web/src/lib/workspaceSearch.test.ts](files/apps/web/src/lib/workspaceSearch.test.ts.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
