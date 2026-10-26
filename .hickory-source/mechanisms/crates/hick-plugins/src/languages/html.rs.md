<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/languages/html.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-6a3af4e2c34d"></a>
## `crates/hick-plugins/src/languages/html.rs`

<a id="f-6a3af4e2c34d-PLUGIN"></a>
### `PLUGIN`

<storycode:copy id="f-6a3af4e2c34d-PLUGIN">
use super::{lsp, lsp_install};
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "html",
    languages: &["html"],
    interfaces: &[
        Interface::Syntax {
            file_names: &[],
            extensions: &[
                ("html", "html", crate::routing::Highlight::Editor),
                ("htm", "html", crate::routing::Highlight::Editor),
            ],
        },
        Interface::Lsp {
            candidates: lsp::C_HTML,
            project_dirs: &[],
            installer: Some(&lsp_install::JSON),
        },
    ],
};
</storycode:copy>

### Assemble `crates/hick-plugins/src/languages/html.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/languages/html.rs"><storycode:paste select="#f-6a3af4e2c34d-PLUGIN"/></storycode:file>

</storycode:doc>
