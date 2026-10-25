<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/languages/c.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `e649340319b5e055ec1fc2eafd22b1e6be16e7d632d4db0bfbe00f6e152e8453`.

<book:entry path="crates/hick-plugins/src/languages/c.rs">
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
</book:entry>

</book:doc>
