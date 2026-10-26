<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/languages/sql.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-39a98f91bc1e"></a>
## `crates/hick-plugins/src/languages/sql.rs`

<a id="f-39a98f91bc1e-PLUGIN"></a>
### `PLUGIN`

<storycode:copy id="f-39a98f91bc1e-PLUGIN">
use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "sql",
    languages: &["sql"],
    interfaces: &[
        Interface::Syntax {
            file_names: &[],
            extensions: &[("sql", "sql", crate::routing::Highlight::Editor)],
        },
        Interface::Lsp {
            candidates: lsp::C_SQL,
            project_dirs: &[],
            installer: None,
        },
    ],
};
</storycode:copy>

### Assemble `crates/hick-plugins/src/languages/sql.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/languages/sql.rs"><storycode:paste select="#f-39a98f91bc1e-PLUGIN"/></storycode:file>

</storycode:doc>
