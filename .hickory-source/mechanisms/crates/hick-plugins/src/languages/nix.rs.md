<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/languages/nix.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-06243ad93e60"></a>
## `crates/hick-plugins/src/languages/nix.rs`

<a id="f-06243ad93e60-PLUGIN"></a>
### `PLUGIN`

<storycode:copy id="f-06243ad93e60-PLUGIN">
use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "nix",
    languages: &["nix"],
    interfaces: &[
        Interface::Project {
            markers: &["flake.nix"],
        },
        Interface::Syntax {
            file_names: &[],
            extensions: &[("nix", "nix", crate::routing::Highlight::PlainText)],
        },
        Interface::Lsp {
            candidates: lsp::C_NIX,
            project_dirs: &[],
            installer: None,
        },
    ],
};
</storycode:copy>

### Assemble `crates/hick-plugins/src/languages/nix.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/languages/nix.rs"><storycode:paste select="#f-06243ad93e60-PLUGIN"/></storycode:file>

</storycode:doc>
