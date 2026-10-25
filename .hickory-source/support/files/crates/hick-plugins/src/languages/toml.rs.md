<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/languages/toml.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `b3a4b6715c3829191804dfd50181339240d6837c94caf3b5640abac9e70a3e5d`.

<book:entry path="crates/hick-plugins/src/languages/toml.rs">
use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "toml",
    languages: &["toml"],
    interfaces: &[
        Interface::Syntax {
            file_names: &[],
            extensions: &[("toml", "toml", crate::routing::Highlight::Editor)],
        },
        Interface::Lsp {
            candidates: lsp::C_TOML,
            project_dirs: &[],
            installer: None,
        },
    ],
};
</book:entry>

</book:doc>
