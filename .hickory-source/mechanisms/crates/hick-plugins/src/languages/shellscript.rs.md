<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/languages/shellscript.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-700135e00e99"></a>
## `crates/hick-plugins/src/languages/shellscript.rs`

<a id="f-700135e00e99-PLUGIN"></a>
### `PLUGIN`

<storycode:copy id="f-700135e00e99-PLUGIN">
use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "shellscript",
    languages: &["shellscript"],
    interfaces: &[
        Interface::SessionWrites {
            aliases: &["sh", "bash", "shell", "posix-sh", "shellscript"],
            extract: crate::session_writes::extract_shell_writes,
        },
        Interface::Syntax {
            file_names: &[],
            extensions: &[
                ("sh", "shellscript", crate::routing::Highlight::Editor),
                ("bash", "shellscript", crate::routing::Highlight::Editor),
                ("zsh", "shellscript", crate::routing::Highlight::Editor),
            ],
        },
        Interface::Lsp {
            candidates: lsp::C_SHELLSCRIPT,
            project_dirs: &[],
            installer: None,
        },
    ],
};
</storycode:copy>

### Assemble `crates/hick-plugins/src/languages/shellscript.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/languages/shellscript.rs"><storycode:paste select="#f-700135e00e99-PLUGIN"/></storycode:file>

</storycode:doc>
