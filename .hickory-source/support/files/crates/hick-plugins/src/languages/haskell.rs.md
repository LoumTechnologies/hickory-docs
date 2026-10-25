<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/languages/haskell.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `729d6f500daab31723ddcd9dc7dbca0ff07ca96d9810f109e521a65e7e183617`.

<book:entry path="crates/hick-plugins/src/languages/haskell.rs">
use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "haskell",
    languages: &["haskell"],
    interfaces: &[
        Interface::Syntax {
            extensions: &[
                ("hs", "haskell", crate::routing::Highlight::PlainText),
                ("lhs", "haskell", crate::routing::Highlight::PlainText),
            ],
            file_names: &[],
        },
        Interface::Project {
            markers: &["stack.yaml", "cabal.project"],
        },
        Interface::Lsp {
            candidates: lsp::C_HASKELL,
            project_dirs: &[],
            installer: None,
        },
    ],
};
</book:entry>

</book:doc>
