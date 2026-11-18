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
        Interface::TestRunner {
            debug: Some(test_debug_command),
            plan: test_command,
            detect: Some(crate::testing::candidates::rust),
            scope: crate::testing::Scope::Crate,
        },
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
fn test_debug_command(root:&std::path::Path,file:&std::path::Path,name:&str,manifest:Option<&std::path::Path>)->Result<crate::testing::TestDebugCommand,String>{
 let command=if let Some(manifest)=manifest{let relative=manifest.strip_prefix(root).map_err(|_|"Constructed Rust manifest escaped the runtime root")?;if manifest.file_name().and_then(|n|n.to_str())!=Some("Cargo.toml")||relative.components().any(|c|!matches!(c,std::path::Component::Normal(_))){return Err("Invalid constructed Rust manifest descriptor".into())};crate::testing::TestCommand{title:format!("test: {name}"),cwd:manifest.parent().unwrap().to_owned(),argv:Vec::new()}}else{test_command(root,file,name)?};if name.is_empty(){return Err("Rust test Debug requires a saved selected test identity".into())}
 Ok(crate::testing::TestDebugCommand{title:format!("Debug {}",command.title),cwd:command.cwd,program:file.to_owned(),launch:serde_json::json!({}),framework:"libtest",preparation:Some(crate::testing::TestDebugPreparation::RustCargo{filter:name.to_owned()})})
}

#[cfg(test)]mod selected_test_debug_plan_tests{use super::*;
 #[test]fn rust_debug_plan_is_typed_and_has_no_build_effects(){let dir=tempfile::tempdir().unwrap();std::fs::write(dir.path().join("Cargo.toml"),"[package]\nname=\"selected\"\nversion=\"0.1.0\"\n").unwrap();let source=dir.path().join("lib.rs");std::fs::write(&source,"#[test]fn chosen(){}").unwrap();let plan=test_debug_command(dir.path(),&source,"chosen",None).unwrap();assert_eq!(plan.framework,"libtest");assert_eq!(plan.program,source);assert!(matches!(plan.preparation,Some(crate::testing::TestDebugPreparation::RustCargo{filter})if filter=="chosen"));assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(),2);assert!(test_debug_command(dir.path(),&source,"",None).is_err());}
}
