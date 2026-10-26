<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/languages/go.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-9b16480d1289"></a>
## `crates/hick-plugins/src/languages/go.rs`

<a id="f-9b16480d1289-PLUGIN"></a>
### `PLUGIN`

<storycode:copy id="f-9b16480d1289-PLUGIN">
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

</storycode:copy>
<a id="f-9b16480d1289-test_command"></a>
### `test_command`

<storycode:copy id="f-9b16480d1289-test_command">
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
</storycode:copy>

### Assemble `crates/hick-plugins/src/languages/go.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/languages/go.rs"><storycode:paste select="#f-9b16480d1289-PLUGIN"/><storycode:paste select="#f-9b16480d1289-test_command"/></storycode:file>

</storycode:doc>
