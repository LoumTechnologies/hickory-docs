//! Integration tests: host project directory mounted at `/workspace`.
//!
//! These exercises require **guest virtio networking disabled** for the WASM kernel (see
//! [`common::create_executor_with_guest_network`]): with `--net=socket`, current container2wasm
//! images omit the virtio-9p mount at `/workspace`, so the sandbox would only see stub paths (the
//! “only `.hick` visible” symptom when artifacts were injected without a backing host mount).
//!
//! Covers:
//!
//! - **Visibility** — Files and dirs that exist under the temp project dir appear in `ls` / `cat`
//! - **Change propagation** — A host overwrite is visible inside the sandbox on the next read
//!   (shared mount coherence; true inotify in the WASM guest is not asserted here.)
//! - **Writes** — Guest redirects and mkdir create real files/dirs under the temp project dir
//!
//! Production `hick-agent run` passes `WasmSandboxConfig { guest_network_enabled: true }` so
//! `apk` / `pip` work; resolving mount+network coexistence belongs in the container/kernel layer.
//!
//! Run with:
//! ```text
//! cargo test -p hick-agent --features integration-test --test sandbox_workspace_mount -- --test-threads=1
//! ```

#![cfg(feature = "integration-test")]

mod common;

use std::sync::Arc;

use common::create_executor_with_guest_network;
use hick_agent_core::{CommandExecutor, CommandOutput};
use tokio::sync::Mutex;

/// Must match `ReactLoop` sandbox container id (`react_loop.rs`).
const SANDBOX_CONTAINER: &str = "agent-sandbox";

async fn exec_sh(
    executor: &Arc<Mutex<dyn CommandExecutor>>,
    image: &str,
    shell_cmd: &str,
) -> CommandOutput {
    executor
        .lock()
        .await
        .execute(SANDBOX_CONTAINER, shell_cmd, image, None)
        .await
        .unwrap_or_else(|e| panic!("sandbox execute failed: {e}\ncommand: {shell_cmd}"))
}

#[tokio::test(flavor = "multi_thread")]
async fn sandbox_sees_pre_existing_host_files_and_directories() {
    let workdir = tempfile::tempdir().unwrap();
    let root = workdir.path();
    std::fs::write(root.join("host_seed.txt"), "seed").unwrap();
    std::fs::create_dir_all(root.join("pre_existing")).unwrap();
    std::fs::write(root.join("pre_existing/inner.txt"), "inner-body").unwrap();

    let (executor, image) = create_executor_with_guest_network(root, false);

    let out = exec_sh(&executor, &image, "ls -1 /workspace | sort").await;
    assert_eq!(
        out.exit_code,
        Some(0),
        "unexpected exit, output: {:?}",
        out.output
    );
    let listing = &out.output;
    assert!(
        listing.contains("host_seed.txt"),
        "expected host file in `ls`; got:\n{listing}"
    );
    assert!(
        listing.contains("pre_existing"),
        "expected host directory in `ls`; got:\n{listing}"
    );

    let out = exec_sh(&executor, &image, "cat /workspace/pre_existing/inner.txt").await;
    assert!(
        out.output.contains("inner-body"),
        "expected nested file contents; exit={:?}; output={:?}",
        out.exit_code,
        out.output
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn sandbox_observes_host_file_updates_on_next_read() {
    let workdir = tempfile::tempdir().unwrap();
    let root = workdir.path();
    let path = root.join("mutual.txt");
    std::fs::write(&path, "before-host-write").unwrap();

    let (executor, image) = create_executor_with_guest_network(root, false);

    let out = exec_sh(&executor, &image, "cat /workspace/mutual.txt").await;
    assert!(out.output.contains("before-host-write"));

    std::fs::write(&path, "after-host-write").unwrap();

    let out = exec_sh(&executor, &image, "cat /workspace/mutual.txt").await;
    assert!(
        out.output.contains("after-host-write"),
        "sandbox should observe host updates via shared mount; got {:?}",
        out.output
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn sandbox_writes_are_visible_on_host_including_new_paths() {
    let workdir = tempfile::tempdir().unwrap();
    let root = workdir.path();
    std::fs::write(root.join("rw_roundtrip.txt"), "initial").unwrap();

    let (executor, image) = create_executor_with_guest_network(root, false);

    let out = exec_sh(
        &executor,
        &image,
        "printf '%s' sandbox-overwrite > /workspace/rw_roundtrip.txt",
    )
    .await;
    assert_eq!(out.exit_code, Some(0), "{}", out.output);
    assert_eq!(
        std::fs::read_to_string(root.join("rw_roundtrip.txt")).unwrap(),
        "sandbox-overwrite"
    );

    let out = exec_sh(
        &executor,
        &image,
        "mkdir -p /workspace/from_sandbox/nested \
            && echo new-file-line > /workspace/from_sandbox/nested/leaf.txt",
    )
    .await;
    assert_eq!(out.exit_code, Some(0), "{}", out.output);

    let leaf = root.join("from_sandbox/nested/leaf.txt");
    assert!(leaf.is_file(), "expected new file at {leaf:?}");
    assert!(
        std::fs::read_to_string(&leaf)
            .unwrap()
            .contains("new-file-line"),
        "unexpected leaf contents"
    );
}
