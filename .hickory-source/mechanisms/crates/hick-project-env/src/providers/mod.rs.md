<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-project-env/src/providers/mod.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-f3efd8c0f516"></a>
## `crates/hick-project-env/src/providers/mod.rs`

Discover workspace environments and publish evaluated project entries through the host filesystem boundary.

<a id="f-f3efd8c0f516-node"></a>
### `node`

<storycode:copy id="f-f3efd8c0f516-node">
pub mod node;
</storycode:copy>
<a id="f-f3efd8c0f516-uv"></a>
### `uv`

<storycode:copy id="f-f3efd8c0f516-uv">
pub mod uv;
</storycode:copy>

### Assemble `crates/hick-project-env/src/providers/mod.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-project-env/src/providers/mod.rs"><storycode:paste select="#f-f3efd8c0f516-node"/><storycode:paste select="#f-f3efd8c0f516-uv"/></storycode:file>

</storycode:doc>
