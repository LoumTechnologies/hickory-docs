use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "shellscript",
    languages: &["shellscript"],
    interfaces: &[
        Interface::SessionWrites {
            aliases: &["sh", "bash", "shell", "posix-sh", "shellscript"],
            extract: crate::session_writes::extract_shell_writes,
        },
        Interface::Syntax {
            file_names: &[],
            extensions: &[
                ("sh", "shellscript", crate::routing::Highlight::Editor),
                ("bash", "shellscript", crate::routing::Highlight::Editor),
                ("zsh", "shellscript", crate::routing::Highlight::Editor),
            ],
        },
        Interface::Lsp {
            candidates: lsp::C_SHELLSCRIPT,
            project_dirs: &[],
            installer: None,
        },
    ],
};
