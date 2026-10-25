<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/languages/swift.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `d99e671381705f82786eaa40e74678f6b1746305960a14d3164111554bf40e16`.

<book:entry path="crates/hick-plugins/src/languages/swift.rs">
use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "swift",
    languages: &["swift"],
    interfaces: &[
        Interface::Project {
            markers: &["Package.swift"],
        },
        Interface::Syntax {
            file_names: &[],
            extensions: &[("swift", "swift", crate::routing::Highlight::Editor)],
        },
        Interface::Lsp {
            candidates: lsp::C_SWIFT,
            project_dirs: &[],
            installer: None,
        },
    ],
};
</book:entry>

</book:doc>
