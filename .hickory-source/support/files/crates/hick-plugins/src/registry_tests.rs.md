<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-plugins/src/registry_tests.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `fcb3e49eeaa5e121c7b1e0573406fba6772e18976043a253d60263547bd0176d`.

<book:entry path="crates/hick-plugins/src/registry_tests.rs">
use super::*;
#[test]
fn csharp_is_one_plugin_with_only_implemented_interfaces() {
    let csharp: Vec<_> = registry()
        .plugins()
        .filter(|p| p.languages.contains(&"csharp"))
        .collect();
    assert_eq!(csharp.len(), 1);
    assert_eq!(csharp[0].id, "csharp");
    for kind in [
        InterfaceKind::Lsp,
        InterfaceKind::Dap,
        InterfaceKind::CodeModel,
        InterfaceKind::Runtime,
    ] {
        assert!(registry().interface("csharp", kind).is_some());
    }
    for kind in [InterfaceKind::Formula, InterfaceKind::Indexer] {
        assert!(registry().interface("csharp", kind).is_none());
    }
    assert!(
        registry()
            .interface("unknown", InterfaceKind::Lsp)
            .is_none()
    );
}
#[test]
fn registration_rejects_ambiguous_ownership() {
    static DUPLICATE: &[&Plugin] = &[&csharp::PLUGIN, &csharp::PLUGIN];
    assert!(Registry::new(DUPLICATE).is_err());
    static OTHER: Plugin = Plugin {
        id: "other",
        languages: &["csharp"],
        interfaces: &[Interface::CodeModel { bin: "other-model" }],
    };
    static CONFLICT: &[&Plugin] = &[&csharp::PLUGIN, &OTHER];
    assert!(Registry::new(CONFLICT).is_err());
}
#[test]
fn runtime_state_cannot_escape_the_host_selected_directory() {
    for path in ["../host", "/host", "C:\\host", "nested/../../host", ""] {
        let environment = Box::leak(Box::new([Environment {
            name: "TEST_STATE",
            value: EnvValue::StatePath(path),
        }]));
        let interfaces = Box::leak(Box::new([Interface::Runtime { environment }]));
        let plugin = Box::leak(Box::new(Plugin {
            id: "test",
            languages: &[],
            interfaces,
        }));
        let plugins = Box::leak(Box::new([&*plugin]));
        assert!(Registry::new(plugins).is_err(), "{path}");
    }
}
#[test]
fn environment_names_and_plugin_names_cannot_be_shell_or_path_syntax() {
    static BAD_ENV: Plugin = Plugin {
        id: "test",
        languages: &[],
        interfaces: &[Interface::Runtime {
            environment: &[Environment {
                name: "BAD; touch escaped",
                value: EnvValue::Literal("1"),
            }],
        }],
    };
    static BAD_ID: Plugin = Plugin {
        id: "../host",
        languages: &[],
        interfaces: &[],
    };
    static BAD_ENVS: &[&Plugin] = &[&BAD_ENV];
    static BAD_IDS: &[&Plugin] = &[&BAD_ID];
    assert!(Registry::new(BAD_ENVS).is_err());
    assert!(Registry::new(BAD_IDS).is_err());
}
#[test]
fn runtime_defaults_are_namespaced_and_do_not_touch_the_filesystem() {
    let root = tempfile::tempdir().unwrap();
    let state = root.path().join("not-created");
    let values = registry().runtime_environment(&state);
    assert!(!state.exists());
    let cli = values
        .iter()
        .find(|(key, _)| *key == "DOTNET_CLI_HOME")
        .unwrap();
    assert_eq!(Path::new(&cli.1), state.join("csharp/dotnet"));
    assert!(
        values
            .iter()
            .any(|(key, value)| *key == "DOTNET_CLI_TELEMETRY_OPTOUT" && value == "1")
    );
}
</book:entry>

</book:doc>
