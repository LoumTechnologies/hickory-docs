use super::{lsp, lsp_install};
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "json",
    languages: &["json"],
    interfaces: &[
        Interface::Syntax {
            file_names: &[],
            extensions: &[("json", "json", crate::routing::Highlight::Editor)],
        },
        Interface::Lsp {
            candidates: lsp::C_JSON,
            project_dirs: &[],
            installer: Some(&lsp_install::JSON),
        },
    ],
};
