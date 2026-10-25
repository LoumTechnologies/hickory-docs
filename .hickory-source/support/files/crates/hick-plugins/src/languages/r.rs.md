<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/languages/r.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `c38eaa6c2930c51b471ed1a9b3649ab4a2b60b6bd792589c482e5dd086dda155`.

<book:entry path="crates/hick-plugins/src/languages/r.rs">
use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "r",
    languages: &["r"],
    interfaces: &[
        Interface::Syntax {
            file_names: &[],
            extensions: &[("r", "r", crate::routing::Highlight::Editor)],
        },
        Interface::Lsp {
            candidates: lsp::C_R,
            project_dirs: &[],
            installer: None,
        },
    ],
};
</book:entry>

</book:doc>
