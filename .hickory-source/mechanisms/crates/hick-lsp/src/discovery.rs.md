<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-lsp/src/discovery.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-9369ac0436fb"></a>
## `crates/hick-lsp/src/discovery.rs`

Discovery and ecosystem recipes belong to the language plugin package.

Related contracts: [A Language Server Is Found Without Being Configured](../../../../guarantees/editor-intelligence/a-language-server-is-found-without-being-configured/contract.md), [Installing A Language Server Is Itself Sandboxed](../../../../guarantees/editor-intelligence/installing-a-language-server-is-itself-sandboxed/contract.md).

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="crates/hick-lsp/src/discovery.rs">
//! Discovery and ecosystem recipes belong to the language plugin package.
pub use hick_plugins::lsp_discovery::*;
</storycode:entry>

</storycode:doc>
