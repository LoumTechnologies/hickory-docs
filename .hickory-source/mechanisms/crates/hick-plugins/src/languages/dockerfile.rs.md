<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/languages/dockerfile.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-5b98d7f30553"></a>
## `crates/hick-plugins/src/languages/dockerfile.rs`

<a id="f-5b98d7f30553-PLUGIN"></a>
### `PLUGIN`

<storycode:copy id="f-5b98d7f30553-PLUGIN">
use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "dockerfile",
    languages: &["dockerfile"],
    interfaces: &[
        Interface::Syntax {
            extensions: &[(
                "dockerfile",
                "dockerfile",
                crate::routing::Highlight::PlainText,
            )],
            file_names: &[
                (
                    "Dockerfile",
                    "dockerfile",
                    crate::routing::Highlight::PlainText,
                ),
                (
                    "Containerfile",
                    "dockerfile",
                    crate::routing::Highlight::PlainText,
                ),
            ],
        },
        Interface::Lsp {
            candidates: lsp::C_DOCKERFILE,
            project_dirs: &[],
            installer: None,
        },
    ],
};
</storycode:copy>

### Assemble `crates/hick-plugins/src/languages/dockerfile.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/languages/dockerfile.rs"><storycode:paste select="#f-5b98d7f30553-PLUGIN"/></storycode:file>

</storycode:doc>
