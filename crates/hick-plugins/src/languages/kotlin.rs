use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "kotlin",
    languages: &["kotlin"],
    interfaces: &[
        Interface::Project {
            markers: &["build.gradle.kts"],
        },
        Interface::Syntax {
            file_names: &[],
            extensions: &[
                ("kt", "kotlin", crate::routing::Highlight::Editor),
                ("kts", "kotlin", crate::routing::Highlight::Editor),
            ],
        },
        Interface::Lsp {
            candidates: lsp::C_KOTLIN,
            project_dirs: &[],
            installer: None,
        },
    ],
};
