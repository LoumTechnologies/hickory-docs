use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "dockerfile",
    languages: &["dockerfile"],
    interfaces: &[
        Interface::Syntax {
            extensions: &[(
                "dockerfile",
                "dockerfile",
                crate::routing::Highlight::PlainText,
            )],
            file_names: &[
                (
                    "Dockerfile",
                    "dockerfile",
                    crate::routing::Highlight::PlainText,
                ),
                (
                    "Containerfile",
                    "dockerfile",
                    crate::routing::Highlight::PlainText,
                ),
            ],
        },
        Interface::Lsp {
            candidates: lsp::C_DOCKERFILE,
            project_dirs: &[],
            installer: None,
        },
    ],
};
