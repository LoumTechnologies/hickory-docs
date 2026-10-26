<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/languages/groovy.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-b67184022d08"></a>
## `crates/hick-plugins/src/languages/groovy.rs`

<a id="f-b67184022d08-PLUGIN"></a>
### `PLUGIN`

<storycode:copy id="f-b67184022d08-PLUGIN">
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "groovy",
    languages: &["groovy"],
    interfaces: &[Interface::Syntax {
        file_names: &[],
        extensions: &[
            ("groovy", "groovy", crate::routing::Highlight::Editor),
            ("gradle", "groovy", crate::routing::Highlight::Editor),
        ],
    }],
};
</storycode:copy>

### Assemble `crates/hick-plugins/src/languages/groovy.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/languages/groovy.rs"><storycode:paste select="#f-b67184022d08-PLUGIN"/></storycode:file>

</storycode:doc>
