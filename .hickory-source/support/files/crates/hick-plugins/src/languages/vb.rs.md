<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/languages/vb.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `d15e366d739385d535bce44d65fa40ed24d7833b214e0b2440efe8aecb54553c`.

<book:entry path="crates/hick-plugins/src/languages/vb.rs">
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "vb",
    languages: &["vb"],
    interfaces: &[Interface::Syntax {
        file_names: &[],
        extensions: &[("vb", "vb", crate::routing::Highlight::Editor)],
    }],
};
</book:entry>

</book:doc>
