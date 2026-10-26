<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/languages/dart.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-8808bc8642f5"></a>
## `crates/hick-plugins/src/languages/dart.rs`

<a id="f-8808bc8642f5-PLUGIN"></a>
### `PLUGIN`

<storycode:copy id="f-8808bc8642f5-PLUGIN">
use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "dart",
    languages: &["dart"],
    interfaces: &[
        Interface::Project {
            markers: &["pubspec.yaml"],
        },
        Interface::Syntax {
            file_names: &[],
            extensions: &[("dart", "dart", crate::routing::Highlight::Editor)],
        },
        Interface::Lsp {
            candidates: lsp::C_DART,
            project_dirs: &[],
            installer: None,
        },
    ],
};
</storycode:copy>

### Assemble `crates/hick-plugins/src/languages/dart.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/languages/dart.rs"><storycode:paste select="#f-8808bc8642f5-PLUGIN"/></storycode:file>

</storycode:doc>
