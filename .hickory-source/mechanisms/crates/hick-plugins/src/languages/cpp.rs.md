<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/languages/cpp.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-8814eebf8c0c"></a>
## `crates/hick-plugins/src/languages/cpp.rs`

<a id="f-8814eebf8c0c-PLUGIN"></a>
### `PLUGIN`

<storycode:copy id="f-8814eebf8c0c-PLUGIN">
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
</storycode:copy>

### Assemble `crates/hick-plugins/src/languages/cpp.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/languages/cpp.rs"><storycode:paste select="#f-8814eebf8c0c-PLUGIN"/></storycode:file>

</storycode:doc>
