<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/languages/yaml.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `f3ec4d0bd4224a673e42df47a320da8248a68303183b99b805d58d37257dbd2d`.

<book:entry path="crates/hick-plugins/src/languages/yaml.rs">
use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "yaml",
    languages: &["yaml"],
    interfaces: &[
        Interface::Syntax {
            file_names: &[],
            extensions: &[
                ("yaml", "yaml", crate::routing::Highlight::Editor),
                ("yml", "yaml", crate::routing::Highlight::Editor),
            ],
        },
        Interface::Lsp {
            candidates: lsp::C_YAML,
            project_dirs: &[],
            installer: None,
        },
    ],
};
</book:entry>

</book:doc>
