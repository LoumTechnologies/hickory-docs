use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "lua",
    languages: &["lua"],
    interfaces: &[
        Interface::Syntax {
            file_names: &[],
            extensions: &[("lua", "lua", crate::routing::Highlight::Editor)],
        },
        Interface::Lsp {
            candidates: lsp::C_LUA,
            project_dirs: &[],
            installer: None,
        },
    ],
};
