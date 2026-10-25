use super::{dap, lsp};
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "go",
    languages: &["go"],
    interfaces: &[
        Interface::Project {
            markers: &["go.mod", "go.work"],
        },
        Interface::TypedClient {
            languages: &["go"],
            target: crate::typed_client::emit::Target::Go,
        },
        Interface::TestRunner { plan: test_command },
        Interface::Syntax {
            file_names: &[],
            extensions: &[("go", "go", crate::routing::Highlight::Editor)],
        },
        Interface::Lsp {
            candidates: lsp::C_GO,
            project_dirs: &[],
            installer: None,
        },
        Interface::Dap {
            candidates: dap::C_GO,
            project_dirs: &[],
            installer: None,
        },
        Interface::CodeModel {
            bin: "hick-model-go",
        },
    ],
};

fn test_command(
    root: &std::path::Path,
    file: &std::path::Path,
    name: &str,
) -> Result<crate::testing::TestCommand, String> {
    use crate::testing::{TestCommand, nearest};
    let rel = file
        .strip_prefix(root)
        .unwrap_or(file)
        .to_string_lossy()
        .replace('\\', "/");
    let name = name.trim();
    let title = if name.is_empty() {
        format!("test: {rel}")
    } else {
        format!("test: {name}")
    };
    let s = |text: &str| text.to_string();

    nearest(root, file, "go.mod")
        .ok_or_else(|| format!("{rel} is not inside a module: no go.mod above it"))?;
    let dir = file.parent().unwrap_or(root).to_path_buf();
    let mut argv = vec![s("go"), s("test"), s(".")];
    if !name.is_empty() {
        argv.push(s("-run"));
        argv.push(format!("^{name}$"));
    }
    Ok(TestCommand {
        title,
        cwd: dir,
        argv,
    })
}
