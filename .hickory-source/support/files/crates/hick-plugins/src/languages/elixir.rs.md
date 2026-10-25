<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/languages/elixir.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `a54faea46c0a24312e64465848d247d8cbd97ab731c2f8fd35fd4f12189c421c`.

<book:entry path="crates/hick-plugins/src/languages/elixir.rs">
use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "elixir",
    languages: &["elixir"],
    interfaces: &[
        Interface::Syntax {
            extensions: &[
                ("ex", "elixir", crate::routing::Highlight::PlainText),
                ("exs", "elixir", crate::routing::Highlight::PlainText),
            ],
            file_names: &[],
        },
        Interface::Project {
            markers: &["mix.exs"],
        },
        Interface::Lsp {
            candidates: lsp::C_ELIXIR,
            project_dirs: &[],
            installer: None,
        },
    ],
};
</book:entry>

</book:doc>
