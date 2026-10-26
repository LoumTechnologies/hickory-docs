<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/languages/xml.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-8fd9920ca94f"></a>
## `crates/hick-plugins/src/languages/xml.rs`

<a id="f-8fd9920ca94f-PLUGIN"></a>
### `PLUGIN`

<storycode:copy id="f-8fd9920ca94f-PLUGIN">
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "xml",
    languages: &["xml"],
    interfaces: &[Interface::Syntax {
        file_names: &[],
        extensions: &[
            ("xml", "xml", crate::routing::Highlight::Editor),
            ("csproj", "xml", crate::routing::Highlight::Editor),
            ("props", "xml", crate::routing::Highlight::Editor),
            ("targets", "xml", crate::routing::Highlight::Editor),
            ("xaml", "xml", crate::routing::Highlight::Editor),
            ("xsd", "xml", crate::routing::Highlight::Editor),
        ],
    }],
};
</storycode:copy>

### Assemble `crates/hick-plugins/src/languages/xml.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/languages/xml.rs"><storycode:paste select="#f-8fd9920ca94f-PLUGIN"/></storycode:file>

</storycode:doc>
