//! A document that generates C#, taken from source to launchable program.
//!
//! The unit tests in `build.rs` drive `build` directly. This drives the path
//! the app drives: weave the document into a scratch directory, pick the
//! entry point, and turn it into the thing a debugger launches. The step
//! being checked is the one `docs/specs/freeform/launching-what-a-document-builds.md`
//! is about — **`hick-dap` assumed the generated file IS the program**, which
//! is false for every compiled language.
//!
//! It stops short of starting an adapter, and that boundary is honest rather
//! than convenient: netcoredbg is not installed on the machine this was
//! written on, and pretending otherwise is what the spec's fourth refusal is
//! about.

use std::path::Path;

const DOC: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="o.md">

# A console app

<hick:file path="app/app.csproj">
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>net10.0</TargetFramework>
  </PropertyGroup>
</Project>
</hick:file>

<hick:file path="app/Program.cs">
class Program
{
    static void Main()
    {
        System.Console.WriteLine("hello");
    }
}
</hick:file>

</hick:doc>
"#;

fn have_dotnet() -> bool {
    std::process::Command::new("dotnet")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[tokio::test]
async fn the_program_a_csharp_document_launches_is_the_assembly_not_the_source() {
    if !have_dotnet() {
        eprintln!("skipped: dotnet is not installed on this machine");
        return;
    }
    let scratch = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();

    let files = hick_dap::weave_into(DOC, scratch.path()).expect("the document weaves");
    // The `.csproj` is generated too, and it is not debuggable — the entry
    // point has to walk past it to the `.cs`.
    assert!(files.iter().any(|f| f.ends_with("app/app.csproj")));
    let entry = hick_dap::entry_point(&files).expect("Program.cs is debuggable");
    assert!(entry.ends_with("app/Program.cs"), "{entry:?}");
    assert_eq!(hick_dap::language_of(&entry), Some("csharp"));

    let mut lines: Vec<String> = Vec::new();
    let program = hick_dap::build(&entry, scratch.path(), cache.path(), &mut |line| {
        if let hick_dap::BuildOutput::Out(text) | hick_dap::BuildOutput::Err(text) = line {
            lines.push(text);
        }
    })
    .await
    .unwrap_or_else(|e| panic!("the build failed:\n{e:#}\n{}", lines.join("\n")));

    // The assumption this whole change exists to remove: what is launched is
    // NOT the generated file.
    assert_ne!(program, entry);
    assert!(program.ends_with("app.dll"), "{program:?}");
    assert!(program.exists(), "{program:?} was not written");
    // And it came from the project's own directory, not the scratch root —
    // `dotnet build` at the root of a tree whose project is in `app/` fails
    // with MSB1003 and nothing anybody can act on.
    assert!(
        program.starts_with(scratch.path().join("app")),
        "{program:?}"
    );
}

#[tokio::test]
async fn a_csharp_document_with_no_project_file_refuses_and_writes_nothing() {
    // No SDK needed: the refusal happens before anything runs.
    let scratch = tempfile::tempdir().unwrap();
    let doc = DOC.replace(
        &DOC[DOC.find("<hick:file path=\"app/app.csproj\">").unwrap()
            ..DOC.find("</hick:file>").unwrap() + "</hick:file>\n".len()],
        "",
    );
    let files = hick_dap::weave_into(&doc, scratch.path()).expect("weaves");
    let entry = hick_dap::entry_point(&files).expect("Program.cs is still debuggable");

    let error = hick_dap::build(&entry, scratch.path(), scratch.path(), &mut |_| {})
        .await
        .expect_err("a C# document with no project file cannot be built");
    let text = format!("{error:#}");
    assert!(text.contains("which is compiled"), "{text}");
    assert!(text.contains("hick ingest"), "{text}");
    // Never invent a project file.
    assert!(!scratch.path().join("app/app.csproj").exists());
    assert!(!list(scratch.path()).iter().any(|p| p.ends_with(".csproj")));
}

fn list(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                out.push(path.display().to_string());
            }
        }
    }
    out
}
