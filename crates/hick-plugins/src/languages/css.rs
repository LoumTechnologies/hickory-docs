use super::{lsp, lsp_install};
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "css",
    languages: &["css", "scss", "less"],
    interfaces: &[
        Interface::Syntax {
            file_names: &[],
            extensions: &[
                ("css", "css", crate::routing::Highlight::Editor),
                ("scss", "scss", crate::routing::Highlight::PlainText),
                ("less", "less", crate::routing::Highlight::PlainText),
            ],
        },
        Interface::Lsp {
            candidates: lsp::C_CSS,
            project_dirs: &[],
            installer: Some(&lsp_install::JSON),
        },
    ],
};
