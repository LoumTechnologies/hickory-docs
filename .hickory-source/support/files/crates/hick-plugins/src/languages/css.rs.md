<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/languages/css.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `88d0fe3145c921f9168cf0c7389426e71eb7d482baa97685a597462c5ec8441e`.

<book:entry path="crates/hick-plugins/src/languages/css.rs">
use super::{lsp, lsp_install};
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "css",
    languages: &["css", "scss", "less"],
    interfaces: &[
        Interface::Syntax {
            file_names: &[],
            extensions: &[
                ("css", "css", crate::routing::Highlight::Editor),
                ("scss", "scss", crate::routing::Highlight::PlainText),
                ("less", "less", crate::routing::Highlight::PlainText),
            ],
        },
        Interface::Lsp {
            candidates: lsp::C_CSS,
            project_dirs: &[],
            installer: Some(&lsp_install::JSON),
        },
    ],
};
</book:entry>

</book:doc>
