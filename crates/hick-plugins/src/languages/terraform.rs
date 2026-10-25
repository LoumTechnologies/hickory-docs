use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "terraform",
    languages: &["terraform"],
    interfaces: &[
        Interface::Syntax {
            extensions: &[
                ("tf", "terraform", crate::routing::Highlight::PlainText),
                ("tfvars", "terraform", crate::routing::Highlight::PlainText),
            ],
            file_names: &[],
        },
        Interface::Project { markers: &["*.tf"] },
        Interface::Lsp {
            candidates: lsp::C_TERRAFORM,
            project_dirs: &[],
            installer: None,
        },
    ],
};
