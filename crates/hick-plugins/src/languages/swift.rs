use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "swift",
    languages: &["swift"],
    interfaces: &[
        Interface::Project {
            markers: &["Package.swift"],
        },
        Interface::Syntax {
            file_names: &[],
            extensions: &[("swift", "swift", crate::routing::Highlight::Editor)],
        },
        Interface::Lsp {
            candidates: lsp::C_SWIFT,
            project_dirs: &[],
            installer: None,
        },
    ],
};
