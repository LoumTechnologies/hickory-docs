<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/languages/java.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `dde56c5fa27f1f484e49ab01a2ef9f2381e5cde23dbc6dca475ab15b384c3fb0`.

<book:entry path="crates/hick-plugins/src/languages/java.rs">
use super::{dap, dap_install, lsp, lsp_install};
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "java",
    languages: &["java"],
    interfaces: &[
        Interface::Project {
            markers: &["pom.xml", "build.gradle", "build.gradle.kts"],
        },
        Interface::Syntax {
            file_names: &[],
            extensions: &[("java", "java", crate::routing::Highlight::Editor)],
        },
        Interface::Lsp {
            candidates: lsp::C_JAVA,
            project_dirs: &[],
            installer: Some(&lsp_install::JAVA),
        },
        Interface::Dap {
            candidates: dap::C_JAVA,
            project_dirs: &[],
            installer: Some(&dap_install::JAVA),
        },
    ],
};
</book:entry>

</book:doc>
