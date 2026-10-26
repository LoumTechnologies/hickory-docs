<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-dap/src/build.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-0fdc81f45247"></a>
## `crates/hick-dap/src/build.rs`

Plugin-owned debug build implementation.

Related contracts: [Launch the artifact built from the selected compiled project](../../../../guarantees/debugging/a-compiled-language-launches-what-a-build-produced/contract.md), [A Plain File Has The Same Debugger A Document Has](../../../../guarantees/debugging/a-plain-file-has-the-same-debugger/contract.md), [Use the adapter’s transport and the current measured language support](../../../../guarantees/debugging/an-adapter-that-listens-is-connected-to/contract.md).

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="crates/hick-dap/src/build.rs">
//! Plugin-owned debug build implementation.
pub use hick_plugins::build::*;
</storycode:entry>

</storycode:doc>
