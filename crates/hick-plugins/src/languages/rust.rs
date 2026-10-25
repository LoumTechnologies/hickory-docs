use super::{dap, dap_install, lsp};
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "rust",
    languages: &["rust"],
    interfaces: &[
        Interface::Structure {
            languages: &["rust"],
            language: |_| "rust",
            grammar: |_| Some(tree_sitter_rust::LANGUAGE.into()),
        },
        Interface::Project {
            markers: &["Cargo.toml"],
        },
        Interface::Build {
            recipe: crate::build::Build::Command {
                project: "Cargo.toml",
                argv: &["cargo", "build"],
                artifact: "debug/*",
                needs: "a `hick:file path=\"app/Cargo.toml\"` block with a `[package]`, beside a \
                    `src/main.rs`",
            },
        },
        Interface::TestRunner { plan: test_command },
        Interface::Syntax {
            file_names: &[],
            extensions: &[("rs", "rust", crate::routing::Highlight::Editor)],
        },
        Interface::Lsp {
            candidates: lsp::C_RUST,
            project_dirs: &[],
            installer: None,
        },
        Interface::Dap {
            candidates: dap::C_NATIVE,
            project_dirs: &[],
            installer: Some(&dap_install::RUST),
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

    let dir = nearest(root, file, "Cargo.toml")
        .ok_or_else(|| format!("{rel} is not inside a cargo package: no Cargo.toml above it"))?;
    let mut argv = vec![s("cargo"), s("test")];
    if !name.is_empty() {
        argv.push(name.to_string());
    }
    Ok(TestCommand {
        title,
        cwd: dir,
        argv,
    })
}
