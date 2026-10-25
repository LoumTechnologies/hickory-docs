use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "php",
    languages: &["php"],
    interfaces: &[
        Interface::Project {
            markers: &["composer.json"],
        },
        Interface::Syntax {
            file_names: &[],
            extensions: &[("php", "php", crate::routing::Highlight::Editor)],
        },
        Interface::Lsp {
            candidates: lsp::C_PHP,
            project_dirs: &[],
            installer: None,
        },
    ],
};
