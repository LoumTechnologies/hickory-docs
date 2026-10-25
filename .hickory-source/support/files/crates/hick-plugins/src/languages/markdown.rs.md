<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/languages/markdown.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `01955b5656e1eaa8168a237fe52c32b79ff3c080d08323974158ac6ade896bfe`.

<book:entry path="crates/hick-plugins/src/languages/markdown.rs">
use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "markdown",
    languages: &["markdown"],
    interfaces: &[
        Interface::Syntax {
            file_names: &[],
            extensions: &[("md", "markdown", crate::routing::Highlight::Editor)],
        },
        Interface::Lsp {
            candidates: lsp::C_MARKDOWN,
            project_dirs: &[],
            installer: None,
        },
    ],
};
</book:entry>

</book:doc>
