use super::{dap, lsp};
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "cpp",
    languages: &["cpp"],
    interfaces: &[
        Interface::Build {
            recipe: crate::build::Build::Compile {
                compilers: &["c++", "g++", "clang++"],
                flags: &["-g", "-O0"],
            },
        },
        Interface::Syntax {
            file_names: &[],
            extensions: &[
                ("cpp", "cpp", crate::routing::Highlight::Editor),
                ("cc", "cpp", crate::routing::Highlight::Editor),
                ("cxx", "cpp", crate::routing::Highlight::Editor),
                ("h", "cpp", crate::routing::Highlight::Editor),
                ("hpp", "cpp", crate::routing::Highlight::Editor),
            ],
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
