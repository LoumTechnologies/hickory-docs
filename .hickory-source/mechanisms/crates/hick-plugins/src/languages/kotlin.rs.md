<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/languages/kotlin.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-46f8f768fa2f"></a>
## `crates/hick-plugins/src/languages/kotlin.rs`

<a id="f-46f8f768fa2f-PLUGIN"></a>
### `PLUGIN`

<storycode:copy id="f-46f8f768fa2f-PLUGIN">
use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "kotlin",
    languages: &["kotlin"],
    interfaces: &[
        Interface::Project {
            markers: &["build.gradle.kts"],
        },
        Interface::Syntax {
            file_names: &[],
            extensions: &[
                ("kt", "kotlin", crate::routing::Highlight::Editor),
                ("kts", "kotlin", crate::routing::Highlight::Editor),
            ],
        },
        Interface::Lsp {
            candidates: lsp::C_KOTLIN,
            project_dirs: &[],
            installer: None,
        },
    ],
};
</storycode:copy>

### Assemble `crates/hick-plugins/src/languages/kotlin.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/languages/kotlin.rs"><storycode:paste select="#f-46f8f768fa2f-PLUGIN"/></storycode:file>

</storycode:doc>
