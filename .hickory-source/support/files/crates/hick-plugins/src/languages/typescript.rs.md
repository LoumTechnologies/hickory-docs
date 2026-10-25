<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/languages/typescript.rs

For: maintainers changing this file and reviewers following the related requirements.

Each file has one constructor. These links establish navigation, not verified behavioral coverage.

- [Use the adapter’s transport and the current measured language support](../../../../../../guarantees/debugging/an-adapter-that-listens-is-connected-to/contract.md)

<book:entry path="crates/hick-plugins/src/languages/typescript.rs">
use super::{dap, dap_install, index_install, lsp, lsp_install};
use crate::*;
pub static PLUGIN: Plugin = Plugin {
    id: "typescript",
    languages: &["typescript", "javascript"],
    interfaces: &[
        Interface::Structure {
            languages: &["typescript", "javascript"],
            language: |language| {
                if language == "javascript" {
                    "javascript"
                } else {
                    "typescript"
                }
            },
            grammar: |language| {
                Some(if language == "javascript" {
                    tree_sitter_javascript::LANGUAGE.into()
                } else {
                    tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
                })
            },
        },
        Interface::ProjectEnvironment {
            provider: || Box::new(hick_project_env::providers::node::Node),
        },
        Interface::Project {
            markers: &["package.json", "tsconfig.json"],
        },
        Interface::TypedClient {
            languages: &["typescript"],
            target: crate::typed_client::emit::Target::TypeScript,
        },
        Interface::TestRunner { plan: test_command },
        Interface::Formula(FormulaBackend {
            language: "javascript",
            aliases: &["javascript", "js", "node", "mjs", "typescript", "ts"],
            file_name: "javascript_backend.mjs",
            interpreters: &["node"],
            source: include_str!("javascript_backend.mjs"),
        }),
        Interface::Syntax {
            file_names: &[],
            extensions: &[
                ("js", "javascript", crate::routing::Highlight::Editor),
                ("mjs", "javascript", crate::routing::Highlight::Editor),
                ("cjs", "javascript", crate::routing::Highlight::Editor),
                ("ts", "typescript", crate::routing::Highlight::Editor),
            ],
        },
        Interface::Lsp {
            candidates: lsp::C_TYPESCRIPT,
            project_dirs: &[],
            installer: Some(&lsp_install::TYPESCRIPT),
        },
        Interface::CodeModel {
            bin: "hick-model-typescript",
        },
        Interface::Indexer {
            bin: "scip-typescript",
            args: &["index"],
            output_flag: "--output",
            installer: Some(&index_install::TYPESCRIPT),
        },
        Interface::Dap {
            project_dirs: &[],
            candidates: dap::C_NODE,
            installer: Some(&dap_install::TYPESCRIPT),
        },
    ],
};

pub(super) fn test_command(
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

    let dir = nearest(root, file, "package.json")
        .ok_or_else(|| format!("{rel} is not inside a package: no package.json above it"))?;
    let rel_to_dir = file
        .strip_prefix(&dir)
        .unwrap_or(file)
        .to_string_lossy()
        .replace('\\', "/");
    // vitest and jest share the `-t <name>` spelling; which one is
    // the package's business, read from its own manifest.
    let manifest = std::fs::read_to_string(dir.join("package.json")).unwrap_or_default();
    let runner = if manifest.contains("\"vitest\"") {
        vec![s("npx"), s("vitest"), s("run"), rel_to_dir]
    } else if manifest.contains("\"jest\"") {
        vec![s("npx"), s("jest"), rel_to_dir]
    } else {
        return Err(format!(
            "{} names neither vitest nor jest, so hick does not know how to run {rel}",
            dir.join("package.json").display()
        ));
    };
    let mut argv = runner;
    if !name.is_empty() {
        argv.push(s("-t"));
        argv.push(name.to_string());
    }
    Ok(TestCommand {
        title,
        cwd: dir,
        argv,
    })
}
</book:entry>

</book:doc>
