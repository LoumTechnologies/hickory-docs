<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/languages/ruby.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-17056a3abebc"></a>
## `crates/hick-plugins/src/languages/ruby.rs`

Related contracts: [Use the adapter’s transport and the current measured language support](../../../../../guarantees/debugging/an-adapter-that-listens-is-connected-to/contract.md).

<a id="f-17056a3abebc-PLUGIN"></a>
### `PLUGIN`

<storycode:copy id="f-17056a3abebc-PLUGIN">
use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "ruby",
    languages: &["ruby"],
    interfaces: &[
        Interface::Project {
            markers: &["Gemfile"],
        },
        Interface::Syntax {
            file_names: &[],
            extensions: &[("rb", "ruby", crate::routing::Highlight::Editor)],
        },
        Interface::Lsp {
            candidates: lsp::C_RUBY,
            project_dirs: &[],
            installer: None,
        },
    ],
};
</storycode:copy>

### Assemble `crates/hick-plugins/src/languages/ruby.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/languages/ruby.rs"><storycode:paste select="#f-17056a3abebc-PLUGIN"/></storycode:file>

</storycode:doc>
