<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-executor-sandbox/tests/plugin_runtime.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-47b72571d6ac"></a>
## `crates/hickory-executor-sandbox/tests/plugin_runtime.rs`

<a id="f-47b72571d6ac-plugin_runtime_state_is_writable_and_survives_the_next_cell"></a>
### `plugin_runtime_state_is_writable_and_survives_the_next_cell`

Focused coverage of the C# plugin's confined runtime configuration.

<storycode:copy id="f-47b72571d6ac-plugin_runtime_state_is_writable_and_survives_the_next_cell">
//! Focused coverage of the C# plugin's confined runtime configuration.
use hickory_executor::Executor;
use hickory_executor_sandbox::{Sandbox, SandboxedExecutor};

#[tokio::test]
async fn plugin_runtime_state_is_writable_and_survives_the_next_cell() {
    if Sandbox::detect() == Sandbox::None {
        eprintln!("SKIPPED: no working sandbox");
        return;
    }
    let executor = SandboxedExecutor::new().unwrap();
    executor.ensure_started("main", "host").await.unwrap();
    #[cfg(unix)]
    let write = r#"test "$DOTNET_CLI_TELEMETRY_OPTOUT" = 1 && test "$DOTNET_GENERATE_ASPNET_CERTIFICATE" = false && mkdir -p "$DOTNET_CLI_HOME" "$NUGET_PACKAGES" && printf remembered > "$DOTNET_CLI_HOME/receipt""#;
    #[cfg(windows)]
    let write = r#"mkdir "%DOTNET_CLI_HOME%" && echo remembered> "%DOTNET_CLI_HOME%\receipt""#;
    executor.execute("main", write).await.unwrap();
    #[cfg(unix)]
    let read = r#"cat "$DOTNET_CLI_HOME/receipt""#;
    #[cfg(windows)]
    let read = r#"type "%DOTNET_CLI_HOME%\receipt""#;
    let output = executor.execute("main", read).await.unwrap();
    assert!(output.contains("remembered"), "{output}");

    executor.ensure_started("other", "host").await.unwrap();
    #[cfg(unix)]
    let isolated = r#"test ! -e "$DOTNET_CLI_HOME/receipt""#;
    #[cfg(windows)]
    let isolated = r#"if exist "%DOTNET_CLI_HOME%\receipt" (exit /b 1) else (exit /b 0)"#;
    executor.execute("other", isolated).await.unwrap();
}

</storycode:copy>
<a id="f-47b72571d6ac-dotnet_new_needs_no_environment_boilerplate_in_the_cell"></a>
### `dotnet_new_needs_no_environment_boilerplate_in_the_cell`

<storycode:copy id="f-47b72571d6ac-dotnet_new_needs_no_environment_boilerplate_in_the_cell">
#[tokio::test]
async fn dotnet_new_needs_no_environment_boilerplate_in_the_cell() {
    if Sandbox::detect() == Sandbox::None {
        eprintln!("SKIPPED: no working sandbox");
        return;
    }
    if !std::process::Command::new("dotnet")
        .env("DOTNET_CLI_TELEMETRY_OPTOUT", "1")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
    {
        eprintln!("SKIPPED: no working .NET SDK");
        return;
    }
    let executor = SandboxedExecutor::new().unwrap();
    executor.ensure_started("main", "host").await.unwrap();
    // Offline and without a package restore: tests first-run setup and
    // scaffolding, not the network or whatever NuGet happens to serve today.
    executor
        .execute("main", "dotnet new console -n blah --no-restore")
        .await
        .unwrap();
    #[cfg(unix)]
    let check = "test -f blah/blah.csproj";
    #[cfg(windows)]
    let check = r#"if exist blah\blah.csproj (exit /b 0) else (exit /b 1)"#;
    executor.execute("main", check).await.unwrap();
}
</storycode:copy>

### Assemble `crates/hickory-executor-sandbox/tests/plugin_runtime.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-executor-sandbox/tests/plugin_runtime.rs"><storycode:paste select="#f-47b72571d6ac-plugin_runtime_state_is_writable_and_survives_the_next_cell"/><storycode:paste select="#f-47b72571d6ac-dotnet_new_needs_no_environment_boilerplate_in_the_cell"/></storycode:file>

</storycode:doc>
