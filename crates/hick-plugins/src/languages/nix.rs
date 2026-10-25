use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "nix",
    languages: &["nix"],
    interfaces: &[
        Interface::Project {
            markers: &["flake.nix"],
        },
        Interface::Syntax {
            file_names: &[],
            extensions: &[("nix", "nix", crate::routing::Highlight::PlainText)],
        },
        Interface::Lsp {
            candidates: lsp::C_NIX,
            project_dirs: &[],
            installer: None,
        },
    ],
};
