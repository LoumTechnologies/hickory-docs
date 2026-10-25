<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/languages/kotlin.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `e3fdbfb92878a46be48d4a0ca411b999751f53aad72a4a56cd2f8749f856bb90`.

<book:entry path="crates/hick-plugins/src/languages/kotlin.rs">
use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "kotlin",
    languages: &["kotlin"],
    interfaces: &[
        Interface::Project {
            markers: &["build.gradle.kts"],
        },
        Interface::Syntax {
            file_names: &[],
            extensions: &[
                ("kt", "kotlin", crate::routing::Highlight::Editor),
                ("kts", "kotlin", crate::routing::Highlight::Editor),
            ],
        },
        Interface::Lsp {
            candidates: lsp::C_KOTLIN,
            project_dirs: &[],
            installer: None,
        },
    ],
};
</book:entry>

</book:doc>
