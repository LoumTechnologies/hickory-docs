use super::lsp;
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "zig",
    languages: &["zig"],
    interfaces: &[
        Interface::Project {
            markers: &["build.zig"],
        },
        Interface::Syntax {
            file_names: &[],
            extensions: &[("zig", "zig", crate::routing::Highlight::PlainText)],
        },
        Interface::Lsp {
            candidates: lsp::C_ZIG,
            project_dirs: &[],
            installer: None,
        },
    ],
};
