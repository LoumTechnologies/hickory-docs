use super::{lsp, lsp_install};
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "html",
    languages: &["html"],
    interfaces: &[
        Interface::Syntax {
            file_names: &[],
            extensions: &[
                ("html", "html", crate::routing::Highlight::Editor),
                ("htm", "html", crate::routing::Highlight::Editor),
            ],
        },
        Interface::Lsp {
            candidates: lsp::C_HTML,
            project_dirs: &[],
            installer: Some(&lsp_install::JSON),
        },
    ],
};
