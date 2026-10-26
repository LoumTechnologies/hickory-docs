<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/typed_client/mod.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-1fd538a78d94"></a>
## `crates/hickory-cli/src/typed_client/mod.rs`

Plugin-owned typed client protocol and emitters.

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="crates/hickory-cli/src/typed_client/mod.rs">
//! Plugin-owned typed client protocol and emitters.
pub use hick_plugins::typed_client::*;
</storycode:entry>

</storycode:doc>
