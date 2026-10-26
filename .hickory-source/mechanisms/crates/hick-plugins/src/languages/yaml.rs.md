<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/languages/yaml.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-3475d85d0eac"></a>
## `crates/hick-plugins/src/languages/yaml.rs`

<a id="f-3475d85d0eac-PLUGIN"></a>
### `PLUGIN`

<storycode:copy id="f-3475d85d0eac-PLUGIN">
use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "yaml",
    languages: &["yaml"],
    interfaces: &[
        Interface::Syntax {
            file_names: &[],
            extensions: &[
                ("yaml", "yaml", crate::routing::Highlight::Editor),
                ("yml", "yaml", crate::routing::Highlight::Editor),
            ],
        },
        Interface::Lsp {
            candidates: lsp::C_YAML,
            project_dirs: &[],
            installer: None,
        },
    ],
};
</storycode:copy>

### Assemble `crates/hick-plugins/src/languages/yaml.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/languages/yaml.rs"><storycode:paste select="#f-3475d85d0eac-PLUGIN"/></storycode:file>

</storycode:doc>
