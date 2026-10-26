<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/languages/fsharp.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-830b8d6cc605"></a>
## `crates/hick-plugins/src/languages/fsharp.rs`

<a id="f-830b8d6cc605-PLUGIN"></a>
### `PLUGIN`

<storycode:copy id="f-830b8d6cc605-PLUGIN">
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "fsharp",
    languages: &["fsharp"],
    interfaces: &[Interface::Syntax {
        file_names: &[],
        extensions: &[
            ("fs", "fsharp", crate::routing::Highlight::Editor),
            ("fsx", "fsharp", crate::routing::Highlight::Editor),
        ],
    }],
};
</storycode:copy>

### Assemble `crates/hick-plugins/src/languages/fsharp.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/languages/fsharp.rs"><storycode:paste select="#f-830b8d6cc605-PLUGIN"/></storycode:file>

</storycode:doc>
