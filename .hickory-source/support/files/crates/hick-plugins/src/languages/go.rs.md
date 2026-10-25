<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/languages/go.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `d7a7d55e0cf0e4a83c650e768c5fbe1eb36779ee998509f8fe1ebe76437a9840`.

<book:entry path="crates/hick-plugins/src/languages/go.rs">
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
</book:entry>

</book:doc>
