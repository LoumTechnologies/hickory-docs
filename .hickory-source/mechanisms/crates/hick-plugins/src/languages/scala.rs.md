<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/languages/scala.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-e6d40010b114"></a>
## `crates/hick-plugins/src/languages/scala.rs`

<a id="f-e6d40010b114-PLUGIN"></a>
### `PLUGIN`

<storycode:copy id="f-e6d40010b114-PLUGIN">
use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "scala",
    languages: &["scala"],
    interfaces: &[
        Interface::Project {
            markers: &["build.sbt"],
        },
        Interface::Syntax {
            file_names: &[],
            extensions: &[
                ("scala", "scala", crate::routing::Highlight::Editor),
                ("sc", "scala", crate::routing::Highlight::Editor),
            ],
        },
        Interface::Lsp {
            candidates: lsp::C_SCALA,
            project_dirs: &[],
            installer: None,
        },
    ],
};
</storycode:copy>

### Assemble `crates/hick-plugins/src/languages/scala.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/languages/scala.rs"><storycode:paste select="#f-e6d40010b114-PLUGIN"/></storycode:file>

</storycode:doc>
