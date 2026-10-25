<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/languages/dart.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `3db99d2ba793c7e860d2cd5d32fb791d41e5a9c4bb77a246e843321176424e9b`.

<book:entry path="crates/hick-plugins/src/languages/dart.rs">
use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "dart",
    languages: &["dart"],
    interfaces: &[
        Interface::Project {
            markers: &["pubspec.yaml"],
        },
        Interface::Syntax {
            file_names: &[],
            extensions: &[("dart", "dart", crate::routing::Highlight::Editor)],
        },
        Interface::Lsp {
            candidates: lsp::C_DART,
            project_dirs: &[],
            installer: None,
        },
    ],
};
</book:entry>

</book:doc>
