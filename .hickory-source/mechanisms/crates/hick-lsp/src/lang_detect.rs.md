<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-lsp/src/lang_detect.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-0053bd574cca"></a>
## `crates/hick-lsp/src/lang_detect.rs`

Language routing is owned by the language plugins.

Related contracts: [Launch the artifact built from the selected compiled project](../../../../guarantees/debugging/a-compiled-language-launches-what-a-build-produced/contract.md), [A Typed Path Opens That File](../../../../guarantees/editor-intelligence/a-typed-path-opens-that-file/contract.md), [A Routed Language Is Drawn, Or Says It Is Not](../../../../guarantees/languages/a-routed-language-is-drawn/contract.md).

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="crates/hick-lsp/src/lang_detect.rs">
//! Language routing is owned by the language plugins.
pub use hick_plugins::routing::*;
</storycode:entry>

</storycode:doc>
