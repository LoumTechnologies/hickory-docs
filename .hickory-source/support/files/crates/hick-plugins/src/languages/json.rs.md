<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/languages/json.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `5689e1170c8889749d2c76d6d025ab17e8889a13ca8788137bc1c0d97d048cc6`.

<book:entry path="crates/hick-plugins/src/languages/json.rs">
use super::{lsp, lsp_install};
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "json",
    languages: &["json"],
    interfaces: &[
        Interface::Syntax {
            file_names: &[],
            extensions: &[("json", "json", crate::routing::Highlight::Editor)],
        },
        Interface::Lsp {
            candidates: lsp::C_JSON,
            project_dirs: &[],
            installer: Some(&lsp_install::JSON),
        },
    ],
};
</book:entry>

</book:doc>
