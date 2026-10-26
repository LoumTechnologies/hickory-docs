<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/languages/c.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-9bf9251963f2"></a>
## `crates/hick-plugins/src/languages/c.rs`

<a id="f-9bf9251963f2-PLUGIN"></a>
### `PLUGIN`

<storycode:copy id="f-9bf9251963f2-PLUGIN">
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
</storycode:copy>

### Assemble `crates/hick-plugins/src/languages/c.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/languages/c.rs"><storycode:paste select="#f-9bf9251963f2-PLUGIN"/></storycode:file>

</storycode:doc>
