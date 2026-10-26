<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/languages/r.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-9a84575d99e6"></a>
## `crates/hick-plugins/src/languages/r.rs`

<a id="f-9a84575d99e6-PLUGIN"></a>
### `PLUGIN`

<storycode:copy id="f-9a84575d99e6-PLUGIN">
use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "r",
    languages: &["r"],
    interfaces: &[
        Interface::Syntax {
            file_names: &[],
            extensions: &[("r", "r", crate::routing::Highlight::Editor)],
        },
        Interface::Lsp {
            candidates: lsp::C_R,
            project_dirs: &[],
            installer: None,
        },
    ],
};
</storycode:copy>

### Assemble `crates/hick-plugins/src/languages/r.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/languages/r.rs"><storycode:paste select="#f-9a84575d99e6-PLUGIN"/></storycode:file>

</storycode:doc>
