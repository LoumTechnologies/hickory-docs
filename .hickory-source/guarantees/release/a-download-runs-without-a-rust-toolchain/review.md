<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# A Download Runs Without A Rust Toolchain: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

## Implementation

- [crates/hickory-cli/src/init.rs](../../../shared/files/crates/hickory-cli/src/init.rs.md)
- [crates/hickory-cli/src/main.rs](../../../shared/files/crates/hickory-cli/src/main.rs.md)
- [crates/hickory-executor-canopy/src/executor.rs](../../../shared/files/crates/hickory-executor-canopy/src/executor.rs.md)
- [scripts/dist.sh](files/scripts/dist.sh.md)
- [scripts/install.sh](../../../shared/files/scripts/install.sh.md)
- [vmtest/scripts/download.ps1](files/vmtest/scripts/download.ps1.md)
- [vmtest/scripts/download.sh](files/vmtest/scripts/download.sh.md)

## Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
