<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/languages/react.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-6a691c076e46"></a>
## `crates/hick-plugins/src/languages/react.rs`

Related contracts: [Use the adapter’s transport and the current measured language support](../../../../../guarantees/debugging/an-adapter-that-listens-is-connected-to/contract.md).

<a id="f-6a691c076e46-PLUGIN"></a>
### `PLUGIN`

<storycode:copy id="f-6a691c076e46-PLUGIN">
use super::{index_install, lsp, lsp_install};
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "react",
    languages: &["typescriptreact", "javascriptreact"],
    interfaces: &[
        Interface::Structure {
            languages: &["typescriptreact", "javascriptreact"],
            language: |language| {
                if language == "javascriptreact" {
                    "javascript"
                } else {
                    "tsx"
                }
            },
            grammar: |language| {
                Some(if language == "javascriptreact" {
                    tree_sitter_javascript::LANGUAGE.into()
                } else {
                    tree_sitter_typescript::LANGUAGE_TSX.into()
                })
            },
        },
        Interface::TestRunner {
            plan: super::typescript::test_command,
        },
        Interface::Syntax {
            file_names: &[],
            extensions: &[
                ("jsx", "javascriptreact", crate::routing::Highlight::Editor),
                ("tsx", "typescriptreact", crate::routing::Highlight::Editor),
            ],
        },
        Interface::Lsp {
            candidates: lsp::C_TYPESCRIPT,
            project_dirs: &[],
            installer: Some(&lsp_install::TYPESCRIPT),
        },
        Interface::CodeModel {
            bin: "hick-model-typescript",
        },
        Interface::Indexer {
            bin: "scip-typescript",
            args: &["index"],
            output_flag: "--output",
            installer: Some(&index_install::TYPESCRIPT),
        },
    ],
};
</storycode:copy>

### Assemble `crates/hick-plugins/src/languages/react.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/languages/react.rs"><storycode:paste select="#f-6a691c076e46-PLUGIN"/></storycode:file>

</storycode:doc>
