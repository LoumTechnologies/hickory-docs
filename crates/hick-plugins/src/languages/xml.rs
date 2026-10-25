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
