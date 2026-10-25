<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/languages/cpp.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `bf630253cabf194d9ebee1a1595c2d6c339ccb1cb59e1563238409dc3ec484d4`.

<book:entry path="crates/hick-plugins/src/languages/cpp.rs">
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
</book:entry>

</book:doc>
