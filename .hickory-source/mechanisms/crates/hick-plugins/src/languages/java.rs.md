<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/languages/java.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-ce848aea8da3"></a>
## `crates/hick-plugins/src/languages/java.rs`

<a id="f-ce848aea8da3-PLUGIN"></a>
### `PLUGIN`

<storycode:copy id="f-ce848aea8da3-PLUGIN">
use super::{dap, dap_install, lsp, lsp_install};
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "java",
    languages: &["java"],
    interfaces: &[
        Interface::Project {
            markers: &["pom.xml", "build.gradle", "build.gradle.kts"],
        },
        Interface::Syntax {
            file_names: &[],
            extensions: &[("java", "java", crate::routing::Highlight::Editor)],
        },
        Interface::Lsp {
            candidates: lsp::C_JAVA,
            project_dirs: &[],
            installer: Some(&lsp_install::JAVA),
        },
        Interface::Dap {
            candidates: dap::C_JAVA,
            project_dirs: &[],
            installer: Some(&dap_install::JAVA),
        },
    ],
};
</storycode:copy>

### Assemble `crates/hick-plugins/src/languages/java.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/languages/java.rs"><storycode:paste select="#f-ce848aea8da3-PLUGIN"/></storycode:file>

</storycode:doc>
