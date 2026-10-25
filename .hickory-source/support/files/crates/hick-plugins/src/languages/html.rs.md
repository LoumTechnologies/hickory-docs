<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/languages/html.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `42a4ab290e233ee44c8945864eb9665f613652443b8c48cb17ea257d179e633d`.

<book:entry path="crates/hick-plugins/src/languages/html.rs">
use super::{lsp, lsp_install};
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "html",
    languages: &["html"],
    interfaces: &[
        Interface::Syntax {
            file_names: &[],
            extensions: &[
                ("html", "html", crate::routing::Highlight::Editor),
                ("htm", "html", crate::routing::Highlight::Editor),
            ],
        },
        Interface::Lsp {
            candidates: lsp::C_HTML,
            project_dirs: &[],
            installer: Some(&lsp_install::JSON),
        },
    ],
};
</book:entry>

</book:doc>
