<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-dap/src/discovery.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-edc49b1ccd22"></a>
## `crates/hick-dap/src/discovery.rs`

Discovery and ecosystem recipes belong to the language plugin package.

Related contracts: [Launch the artifact built from the selected compiled project](../../../../guarantees/debugging/a-compiled-language-launches-what-a-build-produced/contract.md), [A Debugger That Lives In A Language Server Is Asked For](../../../../guarantees/debugging/a-debugger-that-lives-in-a-language-server-is-asked-for/contract.md), [Use the adapter’s transport and the current measured language support](../../../../guarantees/debugging/an-adapter-that-listens-is-connected-to/contract.md), [An Adapter That Runs On A Second Connection Is Followed](../../../../guarantees/debugging/an-adapter-that-runs-on-a-second-connection-is-followed/contract.md), [Install a supported debugger archive into its declared prefix](../../../../guarantees/debugging/an-adapter-that-ships-as-an-archive-installs-like-any-other/contract.md).

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="crates/hick-dap/src/discovery.rs">
//! Discovery and ecosystem recipes belong to the language plugin package.
pub use hick_plugins::dap_discovery::*;
</storycode:entry>

</storycode:doc>
