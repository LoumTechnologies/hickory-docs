<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/languages/php.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `8bbcea1bd76fcc8dd211905a09024ef8ea28032d1055bf5d68592b520ec43925`.

<book:entry path="crates/hick-plugins/src/languages/php.rs">
use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "php",
    languages: &["php"],
    interfaces: &[
        Interface::Project {
            markers: &["composer.json"],
        },
        Interface::Syntax {
            file_names: &[],
            extensions: &[("php", "php", crate::routing::Highlight::Editor)],
        },
        Interface::Lsp {
            candidates: lsp::C_PHP,
            project_dirs: &[],
            installer: None,
        },
    ],
};
</book:entry>

</book:doc>
