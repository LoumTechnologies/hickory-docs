use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "r",
    languages: &["r"],
    interfaces: &[
        Interface::Syntax {
            file_names: &[],
            extensions: &[("r", "r", crate::routing::Highlight::Editor)],
        },
        Interface::Lsp {
            candidates: lsp::C_R,
            project_dirs: &[],
            installer: None,
        },
    ],
};
