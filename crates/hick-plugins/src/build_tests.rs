use super::*;

#[test]
fn an_interpreted_language_has_nothing_to_build() {
    assert!(!is_compiled("python"));
    assert!(!is_compiled("go"));
    assert!(!is_compiled("typescript"));
    // Go is the interesting one: it is compiled, and it is still `None`
    // here, because delve compiles it on the way past.
    assert!(is_compiled("csharp"));
}

#[tokio::test]
async fn an_interpreted_program_is_the_file_it_was_handed() {
    let scratch = tempfile::tempdir().unwrap();
    let source = scratch.path().join("app.py");
    std::fs::write(&source, "print(1)\n").unwrap();
    let built = build(&source, scratch.path(), scratch.path(), &mut |_| {})
        .await
        .unwrap();
    assert_eq!(built, source);
}

#[tokio::test]
async fn a_csharp_document_with_no_project_file_is_refused_by_name() {
    let scratch = tempfile::tempdir().unwrap();
    let source = scratch.path().join("Program.cs");
    std::fs::write(&source, "class P { static void Main() {} }\n").unwrap();
    let error = build(&source, scratch.path(), scratch.path(), &mut |_| {})
        .await
        .unwrap_err();
    let text = format!("{error:#}");
    // The refusal has to say all three things: that C# is compiled, that
    // the missing thing is a project file, and what to do about it.
    assert!(text.contains("which is compiled"), "{text}");
    assert!(text.contains("no project file"), "{text}");
    assert!(text.contains(".csproj"), "{text}");
    // And it must NOT have written one.
    assert!(
        walk(scratch.path())
            .iter()
            .all(|p| p.extension().map(|e| e != "csproj").unwrap_or(true)),
        "a project file was invented"
    );
}

#[test]
fn the_wildcard_matches_within_one_component_only() {
    assert!(wildcard("*.csproj", "app.csproj"));
    assert!(!wildcard("*.csproj", "app.csproj.user"));
    assert!(wildcard("*", "anything"));
    assert!(wildcard("net*", "net10.0"));
    assert!(!wildcard("net*", "standard2.0"));
    assert!(wildcard("bin", "bin"));
    assert!(!wildcard("bin", "obj"));
}

#[test]
fn the_glob_walks_components_and_does_not_wander() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::create_dir_all(root.join("bin/Debug/net10.0")).unwrap();
    std::fs::create_dir_all(root.join("obj/Debug/net10.0")).unwrap();
    std::fs::write(root.join("bin/Debug/net10.0/app.dll"), "").unwrap();
    std::fs::write(root.join("obj/Debug/net10.0/app.dll"), "").unwrap();
    let found = glob(root, "bin/Debug/*/*.dll");
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].ends_with("bin/Debug/net10.0/app.dll"));
}

#[test]
fn the_artifact_named_after_the_project_wins_over_its_dependencies() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let project = root.join("app.csproj");
    std::fs::write(&project, "").unwrap();
    std::fs::create_dir_all(root.join("bin/Debug/net10.0")).unwrap();
    for name in ["app.dll", "Newtonsoft.Json.dll", "Serilog.dll"] {
        std::fs::write(root.join("bin/Debug/net10.0").join(name), "").unwrap();
    }
    let found = locate_artifact(root, "bin/Debug/*/*.dll", &project).unwrap();
    assert!(found.ends_with("app.dll"), "{found:?}");
}

#[test]
fn several_candidates_and_no_stem_match_is_a_refusal_that_lists_them() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let project = root.join("app.csproj");
    std::fs::write(&project, "").unwrap();
    std::fs::create_dir_all(root.join("bin/Debug/net10.0")).unwrap();
    for name in ["Renamed.dll", "Serilog.dll"] {
        std::fs::write(root.join("bin/Debug/net10.0").join(name), "").unwrap();
    }
    let error = locate_artifact(root, "bin/Debug/*/*.dll", &project).unwrap_err();
    let text = format!("{error}");
    assert!(
        text.contains("Renamed.dll") && text.contains("Serilog.dll"),
        "{text}"
    );
}

#[test]
fn the_project_file_is_found_at_whatever_depth_the_document_chose() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("app")).unwrap();
    std::fs::write(dir.path().join("app/app.csproj"), "").unwrap();
    let found = find_one(dir.path(), "*.csproj").unwrap();
    assert!(found.ends_with("app/app.csproj"), "{found:?}");
}
