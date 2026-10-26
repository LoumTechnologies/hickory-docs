<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/languages/haskell.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-979dd855e0c7"></a>
## `crates/hick-plugins/src/languages/haskell.rs`

<a id="f-979dd855e0c7-PLUGIN"></a>
### `PLUGIN`

<storycode:copy id="f-979dd855e0c7-PLUGIN">
use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "haskell",
    languages: &["haskell"],
    interfaces: &[
        Interface::Syntax {
            extensions: &[
                ("hs", "haskell", crate::routing::Highlight::PlainText),
                ("lhs", "haskell", crate::routing::Highlight::PlainText),
            ],
            file_names: &[],
        },
        Interface::Project {
            markers: &["stack.yaml", "cabal.project"],
        },
        Interface::Lsp {
            candidates: lsp::C_HASKELL,
            project_dirs: &[],
            installer: None,
        },
    ],
};
</storycode:copy>

### Assemble `crates/hick-plugins/src/languages/haskell.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/languages/haskell.rs"><storycode:paste select="#f-979dd855e0c7-PLUGIN"/></storycode:file>

</storycode:doc>
