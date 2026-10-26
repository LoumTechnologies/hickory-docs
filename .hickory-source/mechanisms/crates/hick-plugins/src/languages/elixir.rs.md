<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/languages/elixir.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-7e0e48e4baec"></a>
## `crates/hick-plugins/src/languages/elixir.rs`

<a id="f-7e0e48e4baec-PLUGIN"></a>
### `PLUGIN`

<storycode:copy id="f-7e0e48e4baec-PLUGIN">
use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "elixir",
    languages: &["elixir"],
    interfaces: &[
        Interface::Syntax {
            extensions: &[
                ("ex", "elixir", crate::routing::Highlight::PlainText),
                ("exs", "elixir", crate::routing::Highlight::PlainText),
            ],
            file_names: &[],
        },
        Interface::Project {
            markers: &["mix.exs"],
        },
        Interface::Lsp {
            candidates: lsp::C_ELIXIR,
            project_dirs: &[],
            installer: None,
        },
    ],
};
</storycode:copy>

### Assemble `crates/hick-plugins/src/languages/elixir.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/languages/elixir.rs"><storycode:paste select="#f-7e0e48e4baec-PLUGIN"/></storycode:file>

</storycode:doc>
