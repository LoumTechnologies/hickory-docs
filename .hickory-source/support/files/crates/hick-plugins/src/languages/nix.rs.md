<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/languages/nix.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `b0b5c99aac9a127ebb7a84ede40831543be783e895e7f72466198e43ff947988`.

<book:entry path="crates/hick-plugins/src/languages/nix.rs">
use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "nix",
    languages: &["nix"],
    interfaces: &[
        Interface::Project {
            markers: &["flake.nix"],
        },
        Interface::Syntax {
            file_names: &[],
            extensions: &[("nix", "nix", crate::routing::Highlight::PlainText)],
        },
        Interface::Lsp {
            candidates: lsp::C_NIX,
            project_dirs: &[],
            installer: None,
        },
    ],
};
</book:entry>

</book:doc>
