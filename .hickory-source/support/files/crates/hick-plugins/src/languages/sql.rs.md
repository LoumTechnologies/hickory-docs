<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/languages/sql.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `ea67687f3b547666ca70be5a506e8f647a911fd763a48e0a6178b24d86a3abeb`.

<book:entry path="crates/hick-plugins/src/languages/sql.rs">
use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "sql",
    languages: &["sql"],
    interfaces: &[
        Interface::Syntax {
            file_names: &[],
            extensions: &[("sql", "sql", crate::routing::Highlight::Editor)],
        },
        Interface::Lsp {
            candidates: lsp::C_SQL,
            project_dirs: &[],
            installer: None,
        },
    ],
};
</book:entry>

</book:doc>
