<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/languages/vb.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-6420a98a0a58"></a>
## `crates/hick-plugins/src/languages/vb.rs`

<a id="f-6420a98a0a58-PLUGIN"></a>
### `PLUGIN`

<storycode:copy id="f-6420a98a0a58-PLUGIN">
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "vb",
    languages: &["vb"],
    interfaces: &[Interface::Syntax {
        file_names: &[],
        extensions: &[("vb", "vb", crate::routing::Highlight::Editor)],
    }],
};
</storycode:copy>

### Assemble `crates/hick-plugins/src/languages/vb.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/languages/vb.rs"><storycode:paste select="#f-6420a98a0a58-PLUGIN"/></storycode:file>

</storycode:doc>
