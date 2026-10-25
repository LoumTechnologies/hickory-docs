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
