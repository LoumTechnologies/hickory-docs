//! The guard that keeps the live debug suites from testing nothing.
//!
//! Every `live_session_*.rs` file skips loudly when its adapter is missing,
//! which is right: a machine without delve cannot debug Go, and failing there
//! would say something about the runner rather than about the product.
//!
//! The failure mode that creates is real and was live in this repository's CI
//! until 2026-09-04. No debug adapter was installed on the runner, `.hick-cache`
//! is gitignored, and so **all eight live debug tests skipped and the job went
//! green** — the product's largest claim, covered by nothing, reported as
//! passing. Measured, not supposed: hiding `.hick-cache` locally reproduces it
//! exactly.
//!
//! So this asserts what no individual suite can: that *something* was
//! debuggable here. It is the same guard `lsp_languages.rs` already makes for
//! language servers, for the same reason and in the same words.

use std::path::Path;

/// Point the scratch project at whatever this repository has installed, so a
/// developer who ran `hick dap install` once at the top of the repo is
/// covered rather than reported as bare.
#[cfg(unix)]
fn borrow_this_repos_adapters(into: &Path) {
    let cache = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.hick-cache");
    if cache.is_dir() {
        let _ = std::os::unix::fs::symlink(cache, into.join(".hick-cache"));
    }
}

#[cfg(not(unix))]
fn borrow_this_repos_adapters(_into: &Path) {}

/// Languages with a live debug suite. Kept beside them deliberately: a new
/// `live_session_*.rs` with no row here is a suite nothing counts.
const LIVE: &[(&str, &str)] = &[
    ("python", "hick dap install python"),
    ("rust", "hick dap install rust"),
    ("c", "hick dap install rust (codelldb serves C and C++ too)"),
    (
        "cpp",
        "hick dap install rust (codelldb serves C and C++ too)",
    ),
    ("go", "go install github.com/go-delve/delve/cmd/dlv@latest"),
    ("javascript", "hick dap install typescript"),
    ("typescript", "hick dap install typescript"),
    ("csharp", "hick dap install csharp"),
    ("java", "hick lsp install java && hick dap install java"),
];

#[test]
fn at_least_one_language_can_actually_be_debugged_here() {
    let dir = tempfile::tempdir().expect("a temp project");
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    borrow_this_repos_adapters(dir.path());

    let mut covered = Vec::new();
    let mut missing = Vec::new();
    for (language, how) in LIVE {
        // Java's adapter is not spawned but asked for, so discovery answers
        // for it only when BOTH halves are installed — which is the honest
        // test of whether this machine could debug Java.
        if hick_dap::discover(language, dir.path()).is_some() {
            covered.push(*language);
        } else {
            missing.push(format!("  {language}: {how}"));
        }
    }
    eprintln!("debuggable on this machine: {covered:?}");
    assert!(
        !covered.is_empty(),
        "no debug adapter is installed on this machine, so every live debug suite skipped \
         and tested nothing.\nInstall at least one:\n{}",
        missing.join("\n")
    );
}

/// Every language with a live suite must be one `hick lang` calls debuggable.
///
/// The two lists drifting apart is how a suite ends up testing a language the
/// product does not offer, or the product offering one nothing exercises.
#[test]
fn every_live_suite_is_for_a_language_hick_claims() {
    let known = hick_dap::known_languages();
    for (language, _) in LIVE {
        assert!(
            known.contains(language),
            "`{language}` has a live debug suite and is not advertised as debuggable"
        );
    }
}
