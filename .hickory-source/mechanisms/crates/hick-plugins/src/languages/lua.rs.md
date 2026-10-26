<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/languages/lua.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-562bb8942ce4"></a>
## `crates/hick-plugins/src/languages/lua.rs`

<a id="f-562bb8942ce4-PLUGIN"></a>
### `PLUGIN`

<storycode:copy id="f-562bb8942ce4-PLUGIN">
use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "lua",
    languages: &["lua"],
    interfaces: &[
        Interface::Syntax {
            file_names: &[],
            extensions: &[("lua", "lua", crate::routing::Highlight::Editor)],
        },
        Interface::Lsp {
            candidates: lsp::C_LUA,
            project_dirs: &[],
            installer: None,
        },
    ],
};
</storycode:copy>

### Assemble `crates/hick-plugins/src/languages/lua.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/languages/lua.rs"><storycode:paste select="#f-562bb8942ce4-PLUGIN"/></storycode:file>

</storycode:doc>
