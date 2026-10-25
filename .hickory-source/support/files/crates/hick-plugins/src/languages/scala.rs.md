<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/languages/scala.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `8bb5e32e2dafc52bb418f2c8d122fd9dec0cc0a676dc037cb6a5e5a8b428ba6a`.

<book:entry path="crates/hick-plugins/src/languages/scala.rs">
use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "scala",
    languages: &["scala"],
    interfaces: &[
        Interface::Project {
            markers: &["build.sbt"],
        },
        Interface::Syntax {
            file_names: &[],
            extensions: &[
                ("scala", "scala", crate::routing::Highlight::Editor),
                ("sc", "scala", crate::routing::Highlight::Editor),
            ],
        },
        Interface::Lsp {
            candidates: lsp::C_SCALA,
            project_dirs: &[],
            installer: None,
        },
    ],
};
</book:entry>

</book:doc>
