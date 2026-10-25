<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Rebuild the program from its Markdown source: implementation and witnesses

For: reviewers inspecting the code and assertions behind this requirement.

Resolve the implementation first, then inspect what its tests actually assert. A reference, re-export or nearby annotation is not semantic coverage.

Discovery reads the declared closure. The evaluator orders construction acts, builds predecessor states, and combines their outputs. The native writer receives the completed tree only after evaluation. This separation lets a reviewer check equality before any file is written.

## Implementation

- [crates/hick-lang-wasm/src/lib.rs](../../../shared/files/crates/hick-lang-wasm/src/lib.rs.md)
- [crates/hick-lang/src/lib.rs](../../../shared/files/crates/hick-lang/src/lib.rs.md)
- [crates/hick-lang/src/project/discovery.rs](files/crates/hick-lang/src/project/discovery.rs.md)
- [crates/hick-lang/src/project/evaluate.rs](../../../shared/files/crates/hick-lang/src/project/evaluate.rs.md)
- [crates/hick-lang/src/project/recorded.rs](files/crates/hick-lang/src/project/recorded.rs.md)
- [crates/hick-lang/src/recording.rs](../../../support/files/crates/hick-lang/src/recording.rs.md)
- [crates/hick-literate/src/project.rs](files/crates/hick-literate/src/project.rs.md)
- [crates/hick-project-env/src/tree.rs](../../../shared/files/crates/hick-project-env/src/tree.rs.md)
- [crates/hickory-cli/src/project_documents.rs](../../../shared/files/crates/hickory-cli/src/project_documents.rs.md)

## Tests and fixtures

- [crates/hick-lang/tests/project.rs](../../../shared/files/crates/hick-lang/tests/project.rs.md)
- [crates/hick-literate/tests/project.rs](../../../shared/files/crates/hick-literate/tests/project.rs.md)
- [crates/hickory-cli/tests/project_revision.rs](../../../shared/files/crates/hickory-cli/tests/project_revision.rs.md)

## Focused witnesses

- `closure_constructs_atomic_endpoints_and_can_be_reorganized` in `crates/hick-lang/tests/project.rs`.
- `refuses_unordered_transforms_hidden_inputs_and_processes` in `crates/hick-lang/tests/project.rs`.

## Evidence status

- Implementation review: partially verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</book:doc>
