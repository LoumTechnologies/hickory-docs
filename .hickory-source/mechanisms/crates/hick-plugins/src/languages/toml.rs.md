<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/languages/toml.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-89ccee758d04"></a>
## `crates/hick-plugins/src/languages/toml.rs`

<a id="f-89ccee758d04-PLUGIN"></a>
### `PLUGIN`

<storycode:copy id="f-89ccee758d04-PLUGIN">
use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "toml",
    languages: &["toml"],
    interfaces: &[
        Interface::Syntax {
            file_names: &[],
            extensions: &[("toml", "toml", crate::routing::Highlight::Editor)],
        },
        Interface::Lsp {
            candidates: lsp::C_TOML,
            project_dirs: &[],
            installer: None,
        },
    ],
};
</storycode:copy>

### Assemble `crates/hick-plugins/src/languages/toml.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/languages/toml.rs"><storycode:paste select="#f-89ccee758d04-PLUGIN"/></storycode:file>

</storycode:doc>
