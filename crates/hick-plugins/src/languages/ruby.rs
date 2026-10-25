use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "ruby",
    languages: &["ruby"],
    interfaces: &[
        Interface::Project {
            markers: &["Gemfile"],
        },
        Interface::Syntax {
            file_names: &[],
            extensions: &[("rb", "ruby", crate::routing::Highlight::Editor)],
        },
        Interface::Lsp {
            candidates: lsp::C_RUBY,
            project_dirs: &[],
            installer: None,
        },
    ],
};
