use hick_lang::{ProjectEntry, ProjectTree};
use hickory_cli::launch_profiles::{self, Profile, Provider};

fn tree(files: &[(&str, &str)]) -> ProjectTree {
    let mut tree = ProjectTree::default();
    for (path, text) in files {
        tree.insert(path, ProjectEntry::file(text.as_bytes()))
            .unwrap();
    }
    tree
}
// Guarantee: docs/guarantees/debugging/launch-profiles-choose-targets-within-the-viewed-project.md
#[test]
fn library_discovery_selects_the_calling_application_without_running_project_code() {
    let tree = tree(&[
        ("app.py", "from library import greet\nprint(greet())\n"),
        ("library.py", "def greet():\n    return 'Hello'\n"),
    ]);
    let profiles = launch_profiles::discover(&tree).unwrap();
    assert_eq!(profiles.len(), 1);
    assert_eq!(
        launch_profiles::select(&tree, None)
            .unwrap()
            .definition
            .program
            .as_deref(),
        Some("app.py")
    );
}

#[test]
fn java_source_discovery_offers_the_application_and_preserves_run_arguments() {
    let tree = tree(&[
        ("src/Main.java", "class Main { public static\nvoid main (String[] args) {} }"),
        ("src/Library.java", "class Library { static int value() { return 41; } }"),
    ]);
    let mut profiles = launch_profiles::discover(&tree).unwrap();
    assert_eq!(profiles.len(), 1);
    let mut profile = profiles.pop().unwrap();
    assert_eq!(profile.definition.program.as_deref(), Some("src/Main.java"));
    assert!(profile.can_debug);
    profile.definition.args = vec!["two words".into(), "".into()];
    let root = std::path::Path::new("owned workspace");
    let source = root.join("src/Main.java");
    assert_eq!(profile.run_argv(root).unwrap(), ["java", source.to_str().unwrap(), "two words", ""]);
}
// Guarantee: docs/guarantees/debugging/launch-profiles-choose-targets-within-the-viewed-project.md
#[test]
fn manual_profiles_share_discovery_override_by_id_and_keep_argument_boundaries() {
    let source = r#"{"version":1,"profiles":[{"id":"app:app.py","name":"Greeting Ada","program":"app.py","args":["Ada Lovelace","$(touch no)","'quoted'"],"env":{"GREETING":"Hello"},"default":true}]}"#;
    let tree = tree(&[
        ("app.py", "print('hello')"),
        (launch_profiles::CONFIG, source),
    ]);
    let profile = launch_profiles::select(&tree, None).unwrap();
    assert_eq!(launch_profiles::discover(&tree).unwrap().len(), 1);
    assert_eq!(profile.origin, launch_profiles::CONFIG);
    assert_eq!(
        profile.run_argv(std::path::Path::new("/project")).unwrap(),
        vec![
            "python3",
            "/project/app.py",
            "Ada Lovelace",
            "$(touch no)",
            "'quoted'"
        ]
    );
    assert_eq!(profile.definition.env["GREETING"], "Hello");
}

#[test]
fn endpoint_attach_is_explicit_debug_only_and_cannot_launch_or_build() {
    let source = r#"{"version":1,"profiles":[{"id":"external","name":"External Python","attach":{"port":5678,"arguments":{"justMyCode":false}}}]}"#;
    let inventory = tree(&[(launch_profiles::CONFIG, source)]);
    let profile = launch_profiles::select(&inventory, Some("external")).unwrap();
    assert!(profile.can_debug);
    assert_eq!(profile.definition.attach.as_ref().unwrap().port, 5678);
    assert!(profile.run_argv(std::path::Path::new("/unused")).unwrap_err().contains("Attach"));
    for extra in [serde_json::json!({"program":"app.py"}), serde_json::json!({"command":["echo","wrong"]}),
        serde_json::json!({"build":["echo","wrong"]}), serde_json::json!({"args":["wrong"]}),
        serde_json::json!({"env":{"A":"wrong"}}), serde_json::json!({"artifact":"app.py"}),
        serde_json::json!({"attach":{"port":0}}), serde_json::json!({"attach":{"port":5678,"host":"foreign"}}),
        serde_json::json!({"attach":{"port":5678,"arguments":[]}})] {
        let mut config: serde_json::Value = serde_json::from_str(source).unwrap();
        for (key,value) in extra.as_object().unwrap() { config["profiles"][0][key] = value.clone(); }
        let serialized = serde_json::to_string(&config).unwrap();
        let inventory = tree(&[(launch_profiles::CONFIG, &serialized), ("app.py", "print(1)")]);
        assert!(launch_profiles::discover(&inventory).is_err(), "accepted {extra}");
    }
}
// Guarantee: docs/guarantees/debugging/launch-profiles-choose-targets-within-the-viewed-project.md
#[test]
fn ambiguous_targets_require_a_choice_and_unknown_configuration_is_not_silently_ignored() {
    let mut tree = tree(&[("a.py", "print(1)"), ("b.py", "print(2)")]);
    assert!(
        launch_profiles::select(&tree, None)
            .unwrap_err()
            .contains("Choose a launch profile")
    );
    assert_eq!(
        launch_profiles::select(&tree, Some("app:b.py"))
            .unwrap()
            .definition
            .program
            .as_deref(),
        Some("b.py")
    );
    tree.insert(
        launch_profiles::CONFIG,
        ProjectEntry::file(
            br#"{"version":1,"profiles":[{"id":"x","name":"bad","program":"../outside.py"}]}"#
                .as_slice(),
        ),
    )
    .unwrap();
    assert!(
        launch_profiles::discover(&tree)
            .unwrap_err()
            .contains(launch_profiles::CONFIG)
    );
    tree.insert(
        launch_profiles::CONFIG,
        ProjectEntry::file(br#"{"version":1,"profiles":[],"unknown":true}"#.as_slice()),
    )
    .unwrap();
    assert!(
        launch_profiles::discover(&tree)
            .unwrap_err()
            .contains("unknown")
    );
}
// Guarantee: docs/guarantees/debugging/launch-profiles-choose-targets-within-the-viewed-project.md
#[test]
fn manifest_scripts_and_test_definitions_become_run_only_targets() {
    let tree = tree(&[
        (
            "web/package.json",
            r#"{"scripts":{"dev":"vite","test":"vitest"}}"#,
        ),
        ("test_greeting.py", "def test_greet():\n    assert True\n"),
    ]);
    let profiles = launch_profiles::discover(&tree).unwrap();
    assert_eq!(profiles.len(), 3);
    assert!(profiles.iter().all(|p| !p.can_debug));
    assert_eq!(
        profiles
            .iter()
            .find(|p| p.definition.id == "npm:web/package.json:dev")
            .unwrap()
            .definition
            .cwd,
        "web"
    );
    assert!(profiles.iter().any(
        |p| p.definition.command == ["python3", "-m", "pytest", "test_greeting.py::test_greet"]
    ));
}
struct ExampleProvider;
impl Provider for ExampleProvider {
    fn discover(&self, _: &dyn launch_profiles::SourceInventory) -> Result<Vec<Profile>, String> {
        Ok(vec![launch_profiles::command_profile(
            "custom".into(),
            "Provider target".into(),
            vec!["tool".into(), "test".into()],
            ".".into(),
            "example provider",
        )])
    }
}
// Guarantee: docs/guarantees/debugging/launch-profiles-choose-targets-within-the-viewed-project.md
#[test]
fn provider_extension_uses_the_same_validated_manual_merge() {
    let discovered =
        launch_profiles::discover_with(&ProjectTree::default(), &[&ExampleProvider]).unwrap();
    assert_eq!(discovered[0].definition.command, ["tool", "test"]);
    let configured = tree(&[(
        ".hick-launch.json",
        r#"{"version":1,"profiles":[{"id":"custom","name":"Manual target","command":["other"]}]}"#,
    )]);
    let merged = launch_profiles::discover_with(&configured, &[&ExampleProvider]).unwrap();
    assert_eq!(merged.len(), 1);
    assert_eq!(merged[0].definition.name, "Manual target");
    assert_eq!(merged[0].definition.command, ["other"]);
}
// Guarantee: docs/guarantees/debugging/launch-profiles-choose-targets-within-the-viewed-project.md
#[test]
fn launch_selection_is_frozen_to_the_listed_project() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("app.py"), "print(1)\n").unwrap();
    let registry = hickory_cli::debug_sessions::Registry::new();
    let context = registry
        .launch_context(temp.path(), "hick:///app.py", "new", None)
        .unwrap();
    let selection = hickory_cli::debug_sessions::ProfileSelection {
        id: "app:app.py".into(),
        fingerprint: context.fingerprint(),
    };
    assert_eq!(
        context.selected(&selection).unwrap().definition.name,
        "app.py"
    );
    std::fs::write(temp.path().join("app.py"), "print(2)\n").unwrap();
    let changed = registry
        .launch_context(temp.path(), "hick:///app.py", "new", None)
        .unwrap();
    assert!(
        changed
            .selected(&selection)
            .unwrap_err()
            .to_string()
            .contains("project changed")
    );
}
