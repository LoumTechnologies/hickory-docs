<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/languages/zig.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-8cec92dcd91f"></a>
## `crates/hick-plugins/src/languages/zig.rs`

<a id="f-8cec92dcd91f-PLUGIN"></a>
### `PLUGIN`

<storycode:copy id="f-8cec92dcd91f-PLUGIN">
use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "zig",
    languages: &["zig"],
    interfaces: &[
        Interface::Project {
            markers: &["build.zig"],
        },
        Interface::Syntax {
            file_names: &[],
            extensions: &[("zig", "zig", crate::routing::Highlight::PlainText)],
        },
        Interface::Lsp {
            candidates: lsp::C_ZIG,
            project_dirs: &[],
            installer: None,
        },
    ],
};
</storycode:copy>

### Assemble `crates/hick-plugins/src/languages/zig.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/languages/zig.rs"><storycode:paste select="#f-8cec92dcd91f-PLUGIN"/></storycode:file>

</storycode:doc>
