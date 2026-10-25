<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/languages/zig.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `3937ed1c4f374a88265fa4c025e592d0757d9b4b6576561b75d446fd65528dcc`.

<book:entry path="crates/hick-plugins/src/languages/zig.rs">
use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "zig",
    languages: &["zig"],
    interfaces: &[
        Interface::Project {
            markers: &["build.zig"],
        },
        Interface::Syntax {
            file_names: &[],
            extensions: &[("zig", "zig", crate::routing::Highlight::PlainText)],
        },
        Interface::Lsp {
            candidates: lsp::C_ZIG,
            project_dirs: &[],
            installer: None,
        },
    ],
};
</book:entry>

</book:doc>
