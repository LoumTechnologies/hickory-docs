<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/languages/lua.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `382f996531da11d93cbd9b34d0a0c2937850015a90b2b772cb5718b0ef1b5337`.

<book:entry path="crates/hick-plugins/src/languages/lua.rs">
use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "lua",
    languages: &["lua"],
    interfaces: &[
        Interface::Syntax {
            file_names: &[],
            extensions: &[("lua", "lua", crate::routing::Highlight::Editor)],
        },
        Interface::Lsp {
            candidates: lsp::C_LUA,
            project_dirs: &[],
            installer: None,
        },
    ],
};
</book:entry>

</book:doc>
