<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/languages/dockerfile.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `04955e376f71dc5e4378d0994b8f49a6686dc63b03e6ec6955b5fb340e6d1b7e`.

<book:entry path="crates/hick-plugins/src/languages/dockerfile.rs">
use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "dockerfile",
    languages: &["dockerfile"],
    interfaces: &[
        Interface::Syntax {
            extensions: &[(
                "dockerfile",
                "dockerfile",
                crate::routing::Highlight::PlainText,
            )],
            file_names: &[
                (
                    "Dockerfile",
                    "dockerfile",
                    crate::routing::Highlight::PlainText,
                ),
                (
                    "Containerfile",
                    "dockerfile",
                    crate::routing::Highlight::PlainText,
                ),
            ],
        },
        Interface::Lsp {
            candidates: lsp::C_DOCKERFILE,
            project_dirs: &[],
            installer: None,
        },
    ],
};
</book:entry>

</book:doc>
