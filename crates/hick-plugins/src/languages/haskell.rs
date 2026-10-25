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
