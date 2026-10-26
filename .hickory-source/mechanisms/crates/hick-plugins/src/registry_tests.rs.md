<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-plugins/src/registry_tests.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-9643c52d56c0"></a>
## `crates/hick-plugins/src/registry_tests.rs`

<a id="f-9643c52d56c0-csharp_is_one_plugin_with_only_implemented_interfaces"></a>
### `csharp_is_one_plugin_with_only_implemented_interfaces`

<storycode:copy id="f-9643c52d56c0-csharp_is_one_plugin_with_only_implemented_interfaces">
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
</storycode:copy>
<a id="f-9643c52d56c0-registration_rejects_ambiguous_ownership"></a>
### `registration_rejects_ambiguous_ownership`

<storycode:copy id="f-9643c52d56c0-registration_rejects_ambiguous_ownership">
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
</storycode:copy>
<a id="f-9643c52d56c0-runtime_state_cannot_escape_the_host_selected_directory"></a>
### `runtime_state_cannot_escape_the_host_selected_directory`

<storycode:copy id="f-9643c52d56c0-runtime_state_cannot_escape_the_host_selected_directory">
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
</storycode:copy>
<a id="f-9643c52d56c0-environment_names_and_plugin_names_cannot_be_shell_or_path_syntax"></a>
### `environment_names_and_plugin_names_cannot_be_shell_or_path_syntax`

<storycode:copy id="f-9643c52d56c0-environment_names_and_plugin_names_cannot_be_shell_or_path_syntax">
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
</storycode:copy>
<a id="f-9643c52d56c0-runtime_defaults_are_namespaced_and_do_not_touch_the_filesystem"></a>
### `runtime_defaults_are_namespaced_and_do_not_touch_the_filesystem`

<storycode:copy id="f-9643c52d56c0-runtime_defaults_are_namespaced_and_do_not_touch_the_filesystem">
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
</storycode:copy>

### Assemble `crates/hick-plugins/src/registry_tests.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-plugins/src/registry_tests.rs"><storycode:paste select="#f-9643c52d56c0-csharp_is_one_plugin_with_only_implemented_interfaces"/><storycode:paste select="#f-9643c52d56c0-registration_rejects_ambiguous_ownership"/><storycode:paste select="#f-9643c52d56c0-runtime_state_cannot_escape_the_host_selected_directory"/><storycode:paste select="#f-9643c52d56c0-environment_names_and_plugin_names_cannot_be_shell_or_path_syntax"/><storycode:paste select="#f-9643c52d56c0-runtime_defaults_are_namespaced_and_do_not_touch_the_filesystem"/></storycode:file>

</storycode:doc>
