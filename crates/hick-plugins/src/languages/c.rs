use super::{dap, lsp};
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "c",
    languages: &["c"],
    interfaces: &[
        Interface::Build {
            recipe: crate::build::Build::Compile {
                compilers: &["cc", "gcc", "clang"],
                flags: &["-g", "-O0"],
            },
        },
        Interface::Syntax {
            file_names: &[],
            extensions: &[("c", "c", crate::routing::Highlight::Editor)],
        },
        Interface::Lsp {
            candidates: lsp::C_C,
            project_dirs: &[],
            installer: None,
        },
        Interface::Dap {
            candidates: dap::C_NATIVE,
            project_dirs: &[],
            installer: None,
        },
    ],
};
