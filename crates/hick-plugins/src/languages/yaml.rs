use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "yaml",
    languages: &["yaml"],
    interfaces: &[
        Interface::Syntax {
            file_names: &[],
            extensions: &[
                ("yaml", "yaml", crate::routing::Highlight::Editor),
                ("yml", "yaml", crate::routing::Highlight::Editor),
            ],
        },
        Interface::Lsp {
            candidates: lsp::C_YAML,
            project_dirs: &[],
            installer: None,
        },
    ],
};
