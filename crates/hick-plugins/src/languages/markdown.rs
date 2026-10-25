use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "markdown",
    languages: &["markdown"],
    interfaces: &[
        Interface::Syntax {
            file_names: &[],
            extensions: &[("md", "markdown", crate::routing::Highlight::Editor)],
        },
        Interface::Lsp {
            candidates: lsp::C_MARKDOWN,
            project_dirs: &[],
            installer: None,
        },
    ],
};
