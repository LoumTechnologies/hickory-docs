<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/languages/json.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-1e72e6d77a13"></a>
## `crates/hick-plugins/src/languages/json.rs`

<a id="f-1e72e6d77a13-PLUGIN"></a>
### `PLUGIN`

<storycode:copy id="f-1e72e6d77a13-PLUGIN">
use super::{lsp, lsp_install};
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "json",
    languages: &["json"],
    interfaces: &[
        Interface::Syntax {
            file_names: &[],
            extensions: &[("json", "json", crate::routing::Highlight::Editor)],
        },
        Interface::Lsp {
            candidates: lsp::C_JSON,
            project_dirs: &[],
            installer: Some(&lsp_install::JSON),
        },
    ],
};
</storycode:copy>

### Assemble `crates/hick-plugins/src/languages/json.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/languages/json.rs"><storycode:paste select="#f-1e72e6d77a13-PLUGIN"/></storycode:file>

</storycode:doc>
