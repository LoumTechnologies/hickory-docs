<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/languages/swift.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-ccb95d8bd665"></a>
## `crates/hick-plugins/src/languages/swift.rs`

<a id="f-ccb95d8bd665-PLUGIN"></a>
### `PLUGIN`

<storycode:copy id="f-ccb95d8bd665-PLUGIN">
use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "swift",
    languages: &["swift"],
    interfaces: &[
        Interface::Project {
            markers: &["Package.swift"],
        },
        Interface::Syntax {
            file_names: &[],
            extensions: &[("swift", "swift", crate::routing::Highlight::Editor)],
        },
        Interface::Lsp {
            candidates: lsp::C_SWIFT,
            project_dirs: &[],
            installer: None,
        },
    ],
};
</storycode:copy>

### Assemble `crates/hick-plugins/src/languages/swift.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/languages/swift.rs"><storycode:paste select="#f-ccb95d8bd665-PLUGIN"/></storycode:file>

</storycode:doc>
