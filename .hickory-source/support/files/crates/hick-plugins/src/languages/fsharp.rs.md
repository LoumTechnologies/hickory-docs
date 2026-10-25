<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/languages/fsharp.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `df3593f2e94d5d9092f8fdece998a12451754e49c6479e41d249df3dfb9c17ff`.

<book:entry path="crates/hick-plugins/src/languages/fsharp.rs">
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "fsharp",
    languages: &["fsharp"],
    interfaces: &[Interface::Syntax {
        file_names: &[],
        extensions: &[
            ("fs", "fsharp", crate::routing::Highlight::Editor),
            ("fsx", "fsharp", crate::routing::Highlight::Editor),
        ],
    }],
};
</book:entry>

</book:doc>
