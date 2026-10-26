<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/languages/terraform.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-f4c6a4b23f32"></a>
## `crates/hick-plugins/src/languages/terraform.rs`

<a id="f-f4c6a4b23f32-PLUGIN"></a>
### `PLUGIN`

<storycode:copy id="f-f4c6a4b23f32-PLUGIN">
use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "terraform",
    languages: &["terraform"],
    interfaces: &[
        Interface::Syntax {
            extensions: &[
                ("tf", "terraform", crate::routing::Highlight::PlainText),
                ("tfvars", "terraform", crate::routing::Highlight::PlainText),
            ],
            file_names: &[],
        },
        Interface::Project { markers: &["*.tf"] },
        Interface::Lsp {
            candidates: lsp::C_TERRAFORM,
            project_dirs: &[],
            installer: None,
        },
    ],
};
</storycode:copy>

### Assemble `crates/hick-plugins/src/languages/terraform.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/languages/terraform.rs"><storycode:paste select="#f-f4c6a4b23f32-PLUGIN"/></storycode:file>

</storycode:doc>
