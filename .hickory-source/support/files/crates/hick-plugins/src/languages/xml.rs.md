<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/languages/xml.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `20ce209c4816667d65091570a391e629b2793a1b19a34f4e36b55971a7061db1`.

<book:entry path="crates/hick-plugins/src/languages/xml.rs">
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "xml",
    languages: &["xml"],
    interfaces: &[Interface::Syntax {
        file_names: &[],
        extensions: &[
            ("xml", "xml", crate::routing::Highlight::Editor),
            ("csproj", "xml", crate::routing::Highlight::Editor),
            ("props", "xml", crate::routing::Highlight::Editor),
            ("targets", "xml", crate::routing::Highlight::Editor),
            ("xaml", "xml", crate::routing::Highlight::Editor),
            ("xsd", "xml", crate::routing::Highlight::Editor),
        ],
    }],
};
</book:entry>

</book:doc>
