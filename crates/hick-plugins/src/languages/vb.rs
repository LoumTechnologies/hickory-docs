use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "vb",
    languages: &["vb"],
    interfaces: &[Interface::Syntax {
        file_names: &[],
        extensions: &[("vb", "vb", crate::routing::Highlight::Editor)],
    }],
};
