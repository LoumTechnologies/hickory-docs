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
