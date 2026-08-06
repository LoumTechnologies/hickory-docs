//! CI-greppable enforcement of
//! docs/guarantees/execution/canopy-api-isolated-to-one-crate.md:
//! only `crates/hickory-executor-canopy` may know Cloud Canopy's API.
//!
//! Two rules, checked over the whole workspace:
//! 1. No Rust source outside this crate mentions a canopy API token
//!    (proto package, RPC names, metadata key, pty sentinels, `CANOPY_*`
//!    env names). The bare word "canopy" stays legal — executor selection
//!    (`HICKORY_EXECUTOR=canopy`) and docs may name the backend without
//!    knowing its API.
//! 2. Only allowlisted crates (the executor-selection points) may depend on
//!    or import `hickory-executor-canopy` itself.

use std::path::{Path, PathBuf};

/// API-level knowledge of canopy. Any of these outside the adapter crate is
/// a guarantee violation.
const FORBIDDEN_API_TOKENS: &[&str] = &[
    "canopy.v1",
    "CanopyAgent",
    "SpawnSandbox",
    "AttachSandbox",
    "DestroySandbox",
    "spawn_sandbox",
    "attach_sandbox",
    "destroy_sandbox",
    "x-canopy-capability",
    "__CANOPY_S__",
    "__CANOPY_E__",
    "canopy-guest",
    "CANOPY_AGENT",
    "CANOPY_TOKEN",
    "CANOPY_NODE",
    "CANOPY_IMAGE_MAP",
    "CANOPY_VCPUS",
    "CANOPY_MEM_MIB",
    "CANOPY_LIFETIME_SECS",
    "CANOPY_EGRESS_HOSTS",
];

/// Crates allowed to depend on the adapter: the executor-selection points
/// that construct it behind the `Executor` trait.
const ADAPTER_DEPENDENTS_ALLOWED: &[&str] = &["hickory-cli", "hickory-server"];

const ADAPTER_CRATE: &str = "hickory-executor-canopy";

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/<name> has a workspace root two levels up")
        .to_path_buf()
}

fn rust_files_under(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if name == "target" || name == "node_modules" || name.starts_with('.') {
                continue; // hidden dirs include .git and .claude/worktrees
            }
            rust_files_under(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn no_rust_source_outside_the_adapter_crate_knows_canopys_api() {
    let root = workspace_root();
    let adapter = root.join("crates").join(ADAPTER_CRATE);
    let mut files = Vec::new();
    // Scan every Rust file in the repo (crates, apps, examples, build
    // scripts), not just crates/, so a new location cannot dodge the rule.
    rust_files_under(&root, &mut files);
    assert!(
        files.iter().any(|f| f.starts_with(&adapter)),
        "sanity: the scan must see the adapter crate itself"
    );

    let mut violations = Vec::new();
    for file in files {
        if file.starts_with(&adapter) {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&file) else {
            continue;
        };
        for token in FORBIDDEN_API_TOKENS {
            if content.contains(token) {
                violations.push(format!("{}: contains {token:?}", file.display()));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "Cloud Canopy API knowledge leaked outside crates/{ADAPTER_CRATE} \
         (docs/guarantees/execution/canopy-api-isolated-to-one-crate.md):\n{}",
        violations.join("\n")
    );
}

#[test]
fn only_allowlisted_crates_depend_on_the_adapter() {
    let root = workspace_root();
    let crates_dir = root.join("crates");
    let mut violations = Vec::new();
    for entry in std::fs::read_dir(&crates_dir).expect("crates/ exists").flatten() {
        let crate_dir = entry.path();
        let name = crate_dir.file_name().unwrap_or_default().to_string_lossy().into_owned();
        if name == ADAPTER_CRATE {
            continue;
        }
        let manifest = crate_dir.join("Cargo.toml");
        if let Ok(content) = std::fs::read_to_string(&manifest)
            && content.contains(ADAPTER_CRATE)
            && !ADAPTER_DEPENDENTS_ALLOWED.contains(&name.as_str())
        {
            violations.push(format!(
                "{}: depends on {ADAPTER_CRATE} but is not an allowed executor-selection point",
                manifest.display()
            ));
        }
        // The import side of the same rule.
        let mut files = Vec::new();
        rust_files_under(&crate_dir.join("src"), &mut files);
        rust_files_under(&crate_dir.join("tests"), &mut files);
        if !ADAPTER_DEPENDENTS_ALLOWED.contains(&name.as_str()) {
            for file in files {
                if std::fs::read_to_string(&file)
                    .is_ok_and(|c| c.contains("hickory_executor_canopy"))
                {
                    violations.push(format!(
                        "{}: imports hickory_executor_canopy",
                        file.display()
                    ));
                }
            }
        }
    }
    assert!(
        violations.is_empty(),
        "unexpected dependents of the canopy adapter \
         (docs/guarantees/execution/canopy-api-isolated-to-one-crate.md):\n{}",
        violations.join("\n")
    );
}
