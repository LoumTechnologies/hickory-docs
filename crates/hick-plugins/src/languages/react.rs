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
