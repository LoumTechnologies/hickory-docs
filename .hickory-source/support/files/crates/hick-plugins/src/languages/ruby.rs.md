<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/languages/ruby.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `c7ff258a2a077e2625b9361e71b3240404d294981eddc013c8e2bb13d62c488e`.

Related requirements:

- [Use the adapter’s transport and the current measured language support](../../../../../../guarantees/debugging/an-adapter-that-listens-is-connected-to/contract.md)

<book:entry path="crates/hick-plugins/src/languages/ruby.rs">
use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "ruby",
    languages: &["ruby"],
    interfaces: &[
        Interface::Project {
            markers: &["Gemfile"],
        },
        Interface::Syntax {
            file_names: &[],
            extensions: &[("rb", "ruby", crate::routing::Highlight::Editor)],
        },
        Interface::Lsp {
            candidates: lsp::C_RUBY,
            project_dirs: &[],
            installer: None,
        },
    ],
};
</book:entry>

</book:doc>
