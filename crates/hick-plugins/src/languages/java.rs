use super::{dap, dap_install, lsp, lsp_install};
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "java",
    languages: &["java"],
    interfaces: &[
        Interface::Project {
            markers: &["pom.xml", "build.gradle", "build.gradle.kts"],
        },
        Interface::Syntax {
            file_names: &[],
            extensions: &[("java", "java", crate::routing::Highlight::Editor)],
        },
        Interface::Lsp {
            candidates: lsp::C_JAVA,
            project_dirs: &[],
            installer: Some(&lsp_install::JAVA),
        },
        Interface::Dap {
            candidates: dap::C_JAVA,
            project_dirs: &[],
            installer: Some(&dap_install::JAVA),
        },
    ],
};
