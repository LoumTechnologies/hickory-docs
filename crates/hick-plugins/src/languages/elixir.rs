use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "elixir",
    languages: &["elixir"],
    interfaces: &[
        Interface::Syntax {
            extensions: &[
                ("ex", "elixir", crate::routing::Highlight::PlainText),
                ("exs", "elixir", crate::routing::Highlight::PlainText),
            ],
            file_names: &[],
        },
        Interface::Project {
            markers: &["mix.exs"],
        },
        Interface::Lsp {
            candidates: lsp::C_ELIXIR,
            project_dirs: &[],
            installer: None,
        },
    ],
};
