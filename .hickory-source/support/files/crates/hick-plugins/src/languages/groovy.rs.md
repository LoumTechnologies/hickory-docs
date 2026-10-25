<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/languages/groovy.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `0e127d424be6dd1a93bde7492bcaf6bc550c388dc8d7d65c008f6a21a8296a07`.

<book:entry path="crates/hick-plugins/src/languages/groovy.rs">
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "groovy",
    languages: &["groovy"],
    interfaces: &[Interface::Syntax {
        file_names: &[],
        extensions: &[
            ("groovy", "groovy", crate::routing::Highlight::Editor),
            ("gradle", "groovy", crate::routing::Highlight::Editor),
        ],
    }],
};
</book:entry>

</book:doc>
