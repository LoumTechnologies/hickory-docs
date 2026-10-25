use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "dart",
    languages: &["dart"],
    interfaces: &[
        Interface::Project {
            markers: &["pubspec.yaml"],
        },
        Interface::Syntax {
            file_names: &[],
            extensions: &[("dart", "dart", crate::routing::Highlight::Editor)],
        },
        Interface::Lsp {
            candidates: lsp::C_DART,
            project_dirs: &[],
            installer: None,
        },
    ],
};
