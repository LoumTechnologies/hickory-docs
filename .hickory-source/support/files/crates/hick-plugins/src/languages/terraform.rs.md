<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/languages/terraform.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `45f726f926c1cfa8d3d722ea9f48caaf17f9942ab497db6929ccbd2d3599e3b0`.

<book:entry path="crates/hick-plugins/src/languages/terraform.rs">
use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "terraform",
    languages: &["terraform"],
    interfaces: &[
        Interface::Syntax {
            extensions: &[
                ("tf", "terraform", crate::routing::Highlight::PlainText),
                ("tfvars", "terraform", crate::routing::Highlight::PlainText),
            ],
            file_names: &[],
        },
        Interface::Project { markers: &["*.tf"] },
        Interface::Lsp {
            candidates: lsp::C_TERRAFORM,
            project_dirs: &[],
            installer: None,
        },
    ],
};
</book:entry>

</book:doc>
