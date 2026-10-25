<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/languages/shellscript.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `f3b69c66cef079f8896335a4aafeec6f1d5a059c50488fc4ca0fbae8813cd052`.

<book:entry path="crates/hick-plugins/src/languages/shellscript.rs">
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
</book:entry>

</book:doc>
