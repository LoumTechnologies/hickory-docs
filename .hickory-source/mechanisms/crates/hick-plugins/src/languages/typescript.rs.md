<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/languages/typescript.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-1d2ae9a2e366"></a>
## `crates/hick-plugins/src/languages/typescript.rs`

Related contracts: [Use the adapter’s transport and the current measured language support](../../../../../guarantees/debugging/an-adapter-that-listens-is-connected-to/contract.md).

<a id="f-1d2ae9a2e366-PLUGIN"></a>
### `PLUGIN`

<storycode:copy id="f-1d2ae9a2e366-PLUGIN">
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
        Interface::TestRunner {
            debug: None,
            plan: test_command,
            detect: Some(crate::testing::candidates::javascript),
            scope: crate::testing::Scope::FileFilter,
        },
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
</storycode:copy>
<a id="f-1d2ae9a2e366-test_command"></a>
### `test_command`

A source declaration supplies a literal name filter, not a discovered concrete case. Preserve those bytes, including whitespace, while selecting the exact native file. Jest positional paths and Vitest substring paths cannot establish this file scope. Reuse the native specification adapter used by retained failed-only collection; it checks the original physical package and saved source before loading bodies, uses private caches, and refuses an unknown project/harness. No tool is installed implicitly.

<storycode:copy id="f-1d2ae9a2e366-test_command">
pub(super) fn test_command(root:&std::path::Path,file:&std::path::Path,name:&str)->Result<crate::testing::TestCommand,String>{
 use crate::testing::{TestCommand,nearest};use sha2::{Digest,Sha256};
 let root=root.canonicalize().map_err(|error|error.to_string())?;let file=file.canonicalize().map_err(|error|error.to_string())?;
 if !file.starts_with(&root){return Err("Test file is outside the original workspace".into())}
 let rel=file.strip_prefix(&root).unwrap().to_string_lossy().replace('\\',"/");
 let dir=nearest(&root,&file,"package.json").ok_or_else(||format!("{rel} is not inside a package: no package.json above it"))?;
 let manifest:serde_json::Value=serde_json::from_slice(&std::fs::read(dir.join("package.json")).map_err(|error|error.to_string())?).map_err(|error|error.to_string())?;
 let providers=["vitest","jest"].into_iter().filter(|name|manifest["dependencies"].get(*name).is_some()||manifest["devDependencies"].get(*name).is_some()).collect::<Vec<_>>();if providers.len()!=1{return Err(format!("{} must declare one unambiguous Vitest or Jest package runner; no process was started",dir.join("package.json").display()))}let provider=providers[0];
 let bytes=std::fs::read(&file).map_err(|error|error.to_string())?;let hash=format!("{:x}",Sha256::digest(&bytes));
 #[cfg(unix)]let(dev,ino)={use std::os::unix::fs::MetadataExt;let metadata=std::fs::metadata(&dir).map_err(|error|error.to_string())?;(metadata.dev().to_string(),metadata.ino().to_string())};
 #[cfg(not(unix))]let(dev,ino):(String,String)={return Err("Native JavaScript source Run physical project binding is unavailable on this host".into())};
 let spec=serde_json::json!({"mode":"source","provider":provider,"project":dir,"project_dev":dev,"project_ino":ino,"file":file,"source_hash":hash,"pattern":if name.is_empty(){None}else{Some(name)}});
 Ok(TestCommand{title:if name.is_empty(){format!("test: {rel}")}else{format!("test name filter: {name} · {rel}")},cwd:dir,argv:vec!["node".into(),"--input-type=module".into(),"--eval".into(),crate::testing::JAVASCRIPT_TEST_DRIVER.into(),spec.to_string()]})
}
</storycode:copy>

### Assemble `crates/hick-plugins/src/languages/typescript.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

### Read literal relative module references

The language service can use saved sibling modules for a woven source. Recognize
actual import and re-export syntax so comments, strings and computed expressions
cannot request a filesystem read. The workspace decides which literal paths it
can project.

<storycode:copy id="typescript-relative-imports">
pub fn relative_ecmascript_imports(language: &str, source: &str) -> Vec<String> {
    let grammar = match language {
        "typescript" => tree_sitter_typescript::LANGUAGE_TYPESCRIPT,
        "typescriptreact" => tree_sitter_typescript::LANGUAGE_TSX,
        "javascript" => tree_sitter_javascript::LANGUAGE,
        _ => return Vec::new(),
    };
    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&grammar.into()).is_err() {
        return Vec::new();
    }
    let Some(tree) = parser.parse(source, None) else { return Vec::new() };
    let root = tree.root_node();
    let mut cursor = root.walk();
    root.named_children(&mut cursor)
        .filter(|node| matches!(node.kind(), "import_statement" | "export_statement") && !node.has_error())
        .filter_map(|node| {
            let text = node.child_by_field_name("source")?.utf8_text(source.as_bytes()).ok()?;
            let first = *text.as_bytes().first()?;
            if text.len() < 2 || !matches!(first, b'\'' | b'"') || text.as_bytes().last() != Some(&first) {
                return None;
            }
            let specifier = &text[1..text.len() - 1];
            ((specifier.starts_with("./") || specifier.starts_with("../")) && !specifier.contains('\\'))
                .then(|| specifier.to_owned())
        })
        .take(128)
        .collect()
}
</storycode:copy>

<storycode:file path="crates/hick-plugins/src/languages/typescript.rs"><storycode:paste select="#typescript-relative-imports"/><storycode:paste select="#f-1d2ae9a2e366-PLUGIN"/><storycode:paste select="#f-1d2ae9a2e366-test_command"/></storycode:file>

</storycode:doc>
