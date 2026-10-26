<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/languages/php.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-087f7a38d451"></a>
## `crates/hick-plugins/src/languages/php.rs`

<a id="f-087f7a38d451-PLUGIN"></a>
### `PLUGIN`

<storycode:copy id="f-087f7a38d451-PLUGIN">
use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "php",
    languages: &["php"],
    interfaces: &[
        Interface::Project {
            markers: &["composer.json"],
        },
        Interface::Syntax {
            file_names: &[],
            extensions: &[("php", "php", crate::routing::Highlight::Editor)],
        },
        Interface::Lsp {
            candidates: lsp::C_PHP,
            project_dirs: &[],
            installer: None,
        },
    ],
};
</storycode:copy>

### Assemble `crates/hick-plugins/src/languages/php.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/languages/php.rs"><storycode:paste select="#f-087f7a38d451-PLUGIN"/></storycode:file>

</storycode:doc>
