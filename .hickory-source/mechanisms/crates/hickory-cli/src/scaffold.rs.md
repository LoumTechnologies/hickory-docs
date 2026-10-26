<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/scaffold.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-a3d176caef4d"></a>
## `crates/hickory-cli/src/scaffold.rs`

Implementation owned by the language plugin package.

Related contracts: [A New Project Is A Recipe Commit, Made As One Act](../../../../guarantees/authoring/a-new-project-is-a-recipe-commit/contract.md), [New Project Offers The Scaffolder's Own Options, Not A List This Product Maintains](../../../../guarantees/authoring/a-new-project-reads-the-scaffolder-s-own-options/contract.md).

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="crates/hickory-cli/src/scaffold.rs">
//! Implementation owned by the language plugin package.
pub use hick_plugins::scaffold::*;
</storycode:entry>

</storycode:doc>
