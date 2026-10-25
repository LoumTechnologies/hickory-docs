use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "toml",
    languages: &["toml"],
    interfaces: &[
        Interface::Syntax {
            file_names: &[],
            extensions: &[("toml", "toml", crate::routing::Highlight::Editor)],
        },
        Interface::Lsp {
            candidates: lsp::C_TOML,
            project_dirs: &[],
            installer: None,
        },
    ],
};
