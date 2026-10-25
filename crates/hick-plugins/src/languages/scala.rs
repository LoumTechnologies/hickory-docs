use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "scala",
    languages: &["scala"],
    interfaces: &[
        Interface::Project {
            markers: &["build.sbt"],
        },
        Interface::Syntax {
            file_names: &[],
            extensions: &[
                ("scala", "scala", crate::routing::Highlight::Editor),
                ("sc", "scala", crate::routing::Highlight::Editor),
            ],
        },
        Interface::Lsp {
            candidates: lsp::C_SCALA,
            project_dirs: &[],
            installer: None,
        },
    ],
};
