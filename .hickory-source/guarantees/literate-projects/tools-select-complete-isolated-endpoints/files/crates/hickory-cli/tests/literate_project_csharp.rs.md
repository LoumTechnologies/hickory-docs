<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/tests/literate_project_csharp.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Native project tools select complete endpoints](../../../../contract.md)

Captured SHA-256: `81720ce6f4c5cfcd281fc2a4616d645fc66c10fe23c21b4c6d3b4e0dc6954479`.

<book:entry path="crates/hickory-cli/tests/literate_project_csharp.rs">
//! Explicit native acceptance, separated from pure equality tests.
use hickory_cli::{ExecutorChoice, RunMode};
use std::{path::Path, process::Command, time::Duration};

// Guarantee: docs/guarantees/literate-projects/tools-select-complete-isolated-endpoints.md
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires .NET, NuGet access and an installed pinned netcoredbg"]
async fn efcore_final_project_applies_migration_and_debugs_inserted_code() {
    let source = include_str!("../../hick-lang/tests/fixtures/efcore-project/project.md");
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(root.join("project.md"), source).unwrap();
    std::fs::write(
        root.join("persistence.md"),
        include_str!("../../hick-lang/tests/fixtures/efcore-project/persistence.md"),
    )
    .unwrap();
    let run = hickory_cli::run_doc(
        &root.join("project.md"),
        &[],
        RunMode::Weave,
        ExecutorChoice::Local,
    )
    .await
    .unwrap();
    assert_eq!(
        run.result
            .evaluated_project
            .as_ref()
            .unwrap()
            .new
            .entries()
            .len(),
        6
    );
    // Pure equality must complete before any tool materialization/build.
    assert!(!root.join("app").exists());
    hickory_cli::write_outputs(&run, None).unwrap();
    assert!(hickory_cli::check_failures(&run, None).unwrap().is_empty());
    let native = Command::new("dotnet")
        .args(["run", "--project", "app/app.csproj"])
        .current_dir(root)
        .env("DOTNET_CLI_TELEMETRY_OPTOUT", "1")
        .output()
        .unwrap();
    assert!(
        native.status.success(),
        "native EFCore run failed:\n{}\n{}",
        String::from_utf8_lossy(&native.stdout),
        String::from_utf8_lossy(&native.stderr)
    );
    let out = String::from_utf8(native.stdout).unwrap();
    assert!(out.lines().any(|l| l == "verified"), "{out}");
    assert!(out.lines().any(|l| l == "202610120001_Initial"), "{out}");
    assert!(hickory_cli::check_failures(&run, None).unwrap().is_empty());
    // Borrow only the installed adapter. No generated source/config contributes
    // to pure construction or verification.
    let cache = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.hick-cache/adapters")
        .canonicalize()
        .unwrap();
    std::fs::create_dir_all(root.join(".hick-cache")).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(&cache, root.join(".hick-cache/adapters")).unwrap();
    #[cfg(not(unix))]
    copy_adapter_directory(&cache, &root.join(".hick-cache/adapters")).unwrap();
    let inserted = source.find("+db.SaveChanges();").unwrap() + 1;
    let line = source[..inserted].bytes().filter(|b| *b == b'\n').count() as u32;
    let registry = hickory_cli::debug_sessions::Registry::new();
    let mut messages = vec![];
    let (id, live, statuses) = registry
        .start(
            &root.join("project.md"),
            &[hick_dap::Breakpoint {
                line,
                condition: None,
                hit_condition: None,
                log_message: None,
            }],
            Some("app/Program.cs"),
            &mut |message| messages.push(format!("{message:?}")),
        )
        .await
        .unwrap_or_else(|e| panic!("native launch failed: {e:#}\n{}", messages.join("\n")));
    assert_eq!(statuses.len(), 1);
    assert!(!statuses[0].state.is_refused(), "{statuses:?}");
    let mut events = live.session.events();
    let stopped = match live
        .session
        .wait_for_stop(Duration::from_secs(60))
        .await
        .unwrap()
    {
        Some(stopped) => stopped,
        None => {
            let mut observed = vec![];
            while let Ok(event) = events.try_recv() {
                if event.event != "module" {
                    observed.push(format!("{event:?}"));
                }
            }
            panic!(
                "inserted source breakpoint did not stop; statuses={:?}; exit={:?}; events={:?}; build={:?}",
                live.session.breakpoint_statuses(),
                live.session.exit(),
                observed,
                messages
            );
        }
    };
    let frames = live.session.stack(stopped.thread_id).await.unwrap();
    let top = frames.first().unwrap();
    assert_eq!(top.line, Some(line));
    assert_eq!(
        live.session
            .evaluate("note.Text", Some(top.id), "watch")
            .await
            .unwrap()
            .value,
        "\"verified\""
    );
    let staged =
        std::fs::read_to_string(live.scratch_path().unwrap().join("app/Program.cs")).unwrap();
    assert!(
        staged.contains("db.Database.GetAppliedMigrations()"),
        "later complete code must be present"
    );
    registry.stop(&id).await.unwrap();
    assert!(hickory_cli::check_failures(&run, None).unwrap().is_empty());
}

#[cfg(not(unix))]
fn copy_adapter_directory(source: &Path, destination: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(destination)?;
    for item in std::fs::read_dir(source)? {
        let item = item?;
        if item.file_type()?.is_symlink() {
            return Err(std::io::Error::other(
                "adapter fixture does not copy symlinks",
            ));
        }
        let target = destination.join(item.file_name());
        if item.file_type()?.is_dir() {
            copy_adapter_directory(&item.path(), &target)?;
        } else {
            std::fs::copy(item.path(), target)?;
        }
    }
    Ok(())
}
</book:entry>

</book:doc>
