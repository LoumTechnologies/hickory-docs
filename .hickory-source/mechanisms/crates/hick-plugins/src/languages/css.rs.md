<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/languages/css.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-f94eabd49773"></a>
## `crates/hick-plugins/src/languages/css.rs`

<a id="f-f94eabd49773-PLUGIN"></a>
### `PLUGIN`

<storycode:copy id="f-f94eabd49773-PLUGIN">
use super::{lsp, lsp_install};
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "css",
    languages: &["css", "scss", "less"],
    interfaces: &[
        Interface::Syntax {
            file_names: &[],
            extensions: &[
                ("css", "css", crate::routing::Highlight::Editor),
                ("scss", "scss", crate::routing::Highlight::PlainText),
                ("less", "less", crate::routing::Highlight::PlainText),
            ],
        },
        Interface::Lsp {
            candidates: lsp::C_CSS,
            project_dirs: &[],
            installer: Some(&lsp_install::JSON),
        },
    ],
};
</storycode:copy>

### Assemble `crates/hick-plugins/src/languages/css.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/languages/css.rs"><storycode:paste select="#f-f94eabd49773-PLUGIN"/></storycode:file>

</storycode:doc>
