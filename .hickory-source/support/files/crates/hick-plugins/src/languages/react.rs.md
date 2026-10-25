<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/languages/react.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `29cd87f9d69061b13d089e4dec42e8593a33dae9a4058a6fd40bd30932359426`.

Related requirements:

- [Use the adapter’s transport and the current measured language support](../../../../../../guarantees/debugging/an-adapter-that-listens-is-connected-to/contract.md)

<book:entry path="crates/hick-plugins/src/languages/react.rs">
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
</book:entry>

</book:doc>
