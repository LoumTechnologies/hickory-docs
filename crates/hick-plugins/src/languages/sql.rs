use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "sql",
    languages: &["sql"],
    interfaces: &[
        Interface::Syntax {
            file_names: &[],
            extensions: &[("sql", "sql", crate::routing::Highlight::Editor)],
        },
        Interface::Lsp {
            candidates: lsp::C_SQL,
            project_dirs: &[],
            installer: None,
        },
    ],
};
