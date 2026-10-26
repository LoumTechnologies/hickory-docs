<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Use the adapter’s transport and the current measured language support

For: maintainers and authors selecting the current editor or debugger behavior.

Start an adapter that listens on TCP rather than reading stdin. Connection transport and language support must come from the current catalogue, not a dated installation note.

## What must happen

A server-style adapter starts with its declared port placeholder substituted and uses DAP over the selected TCP connection; pipe-style adapters retain their declared transport. Discovery carries required launch metadata. Current JavaScript/TypeScript discovery and live-suite declarations include multi-session support; the old claim that they cannot be driven is historical. React component files are not standalone Node launch programs, and Ruby remains unavailable in the current discovery guidance. Finding an executable or installing an archive alone does not establish a live debugging witness.

## Limits of the evidence

The source inspection does not run a real adapter. Catalogue/live-suite agreement is distinct from a current machine having that adapter installed; no installer or network operation was executed.



## Implementation

<a id="output-0a64a5aacbd1"></a>
## `crates/hick-dap/tests/debug_coverage.rs`

Related contracts: [Use the adapter’s transport and the current measured language support](contract.md).

<a id="f-0a64a5aacbd1-borrow_this_repos_adapters"></a>
### `borrow_this_repos_adapters`

The guard that keeps the live debug suites from testing nothing.
Every `live_session_*.rs` file skips loudly when its adapter is missing,
which is right: a machine without delve cannot debug Go, and failing there
would say something about the runner rather than about the product.
The failure mode that creates is real and was live in this repository's CI
until 2026-09-04. No debug adapter was installed on the runner, `.hick-cache`
is gitignored, and so **all eight live debug tests skipped and the job went
green** — the product's largest claim, covered by nothing, reported as
passing. Measured, not supposed: hiding `.hick-cache` locally reproduces it
exactly.
So this asserts what no individual suite can: that *something* was
debuggable here. It is the same guard `lsp_languages.rs` already makes for
language servers, for the same reason and in the same words.
# Why it is asked for rather than always on
Nobody has codelldb, debugpy or netcoredbg by default. A guard that always
failed would break `cargo test` on a clean clone for every contributor —
demanding five installs before a single test can run — which is a worse
product than the gap it closes.
So it reports on every machine and **fails only where debugging is meant
to be covered**: CI's `debuggers` job sets
`HICKORY_REQUIRE_DEBUG_ADAPTERS=1` after installing them. That job also
asserts each adapter exists in a step of its own, so the env var going
missing does not quietly disarm anything — two independent checks, and
neither is the only one.

<storycode:copy id="f-0a64a5aacbd1-borrow_this_repos_adapters">
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
//!
//! # Why it is asked for rather than always on
//!
//! Nobody has codelldb, debugpy or netcoredbg by default. A guard that always
//! failed would break `cargo test` on a clean clone for every contributor —
//! demanding five installs before a single test can run — which is a worse
//! product than the gap it closes.
//!
//! So it reports on every machine and **fails only where debugging is meant
//! to be covered**: CI's `debuggers` job sets
//! `HICKORY_REQUIRE_DEBUG_ADAPTERS=1` after installing them. That job also
//! asserts each adapter exists in a step of its own, so the env var going
//! missing does not quietly disarm anything — two independent checks, and
//! neither is the only one.

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

</storycode:copy>
<a id="f-0a64a5aacbd1-borrow_this_repos_adapters-2"></a>
### `borrow_this_repos_adapters`

<storycode:copy id="f-0a64a5aacbd1-borrow_this_repos_adapters-2">
#[cfg(not(unix))]
fn borrow_this_repos_adapters(_into: &Path) {}

</storycode:copy>
<a id="f-0a64a5aacbd1-LIVE"></a>
### `LIVE`

Languages with a live debug suite. Kept beside them deliberately: a new
`live_session_*.rs` with no row here is a suite nothing counts.

<storycode:copy id="f-0a64a5aacbd1-LIVE">
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

</storycode:copy>
<a id="f-0a64a5aacbd1-at_least_one_language_can_actually_be_debugged_here"></a>
### `at_least_one_language_can_actually_be_debugged_here`

<storycode:copy id="f-0a64a5aacbd1-at_least_one_language_can_actually_be_debugged_here">
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
    if std::env::var_os("HICKORY_REQUIRE_DEBUG_ADAPTERS").is_none() {
        eprintln!(
            "not required on this machine — set HICKORY_REQUIRE_DEBUG_ADAPTERS=1 to make \
             an empty list a failure, as CI's `debuggers` job does. Missing:\n{}",
            missing.join("\n")
        );
        return;
    }
    assert!(
        !covered.is_empty(),
        "no debug adapter is installed, so every live debug suite skipped and tested \
         nothing.\nInstall at least one:\n{}",
        missing.join("\n")
    );
}

</storycode:copy>
<a id="f-0a64a5aacbd1-the_languages_hick_debugs_and_the_ones_it_exercises_are_the_same_set"></a>
### `the_languages_hick_debugs_and_the_ones_it_exercises_are_the_same_set`

The two lists must be the SAME list, in both directions.
One direction stops a suite testing a language the product does not offer.
The other is the one that has actually bitten, twice in a day: `cpp`
claimed a debugger with no suite anywhere — its own compilers and its own
build-table row, never once driven — and `typescriptreact` and
`javascriptreact` claimed one that could not work at all, because node
cannot execute either file.
Asking the question in both directions is what makes "is every language
covered?" answerable by running the tests rather than by reading them.

<storycode:copy id="f-0a64a5aacbd1-the_languages_hick_debugs_and_the_ones_it_exercises_are_the_same_set">
/// The two lists must be the SAME list, in both directions.
///
/// One direction stops a suite testing a language the product does not offer.
/// The other is the one that has actually bitten, twice in a day: `cpp`
/// claimed a debugger with no suite anywhere — its own compilers and its own
/// build-table row, never once driven — and `typescriptreact` and
/// `javascriptreact` claimed one that could not work at all, because node
/// cannot execute either file.
///
/// Asking the question in both directions is what makes "is every language
/// covered?" answerable by running the tests rather than by reading them.
#[test]
fn the_languages_hick_debugs_and_the_ones_it_exercises_are_the_same_set() {
    let known = hick_dap::known_languages();
    // Listed once each. A duplicate row is how a list of nine reads as
    // covering nine while covering eight, which happened while this very
    // table was being written.
    let mut seen: Vec<&str> = LIVE.iter().map(|(l, _)| *l).collect();
    let before = seen.len();
    seen.sort_unstable();
    seen.dedup();
    assert_eq!(before, seen.len(), "a language is listed twice: {seen:?}");
    for (language, _) in LIVE {
        assert!(
            known.contains(language),
            "`{language}` has a live debug suite and is not advertised as debuggable"
        );
    }
    for language in &known {
        assert!(
            LIVE.iter().any(|(live, _)| live == language),
            "`{language}` is advertised as debuggable and no live suite drives it. Either \
             add `live_session_{language}.rs`, or stop claiming it — those are the only two \
             honest options."
        );
    }
}
</storycode:copy>

### Assemble `crates/hick-dap/tests/debug_coverage.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-dap/tests/debug_coverage.rs"><storycode:paste select="#f-0a64a5aacbd1-borrow_this_repos_adapters"/><storycode:paste select="#f-0a64a5aacbd1-borrow_this_repos_adapters-2"/><storycode:paste select="#f-0a64a5aacbd1-LIVE"/><storycode:paste select="#f-0a64a5aacbd1-at_least_one_language_can_actually_be_debugged_here"/><storycode:paste select="#f-0a64a5aacbd1-the_languages_hick_debugs_and_the_ones_it_exercises_are_the_same_set"/></storycode:file>

## Implementation review

### Implementation

- [crates/hick-dap/src/adapter.rs](../../../mechanisms/crates/hick-dap/src/adapter.rs.md#output-2b5d1652c2d2)
- [crates/hick-dap/src/build.rs](../../../mechanisms/crates/hick-dap/src/build.rs.md#output-0fdc81f45247)
- [crates/hick-dap/src/discovery.rs](../../../mechanisms/crates/hick-dap/src/discovery.rs.md#output-edc49b1ccd22)
- [crates/hick-dap/src/session.rs](../../../mechanisms/crates/hick-dap/src/session.rs.md#output-3c41c22e0551)
- [crates/hick-plugins/src/build.rs](../a-plain-file-has-the-same-debugger/contract.md#output-a283ca144066)
- [crates/hick-plugins/src/dap_discovery.rs](../../../mechanisms/crates/hick-plugins/src/dap_discovery.rs.md#output-72962e7f25e2)
- [crates/hick-plugins/src/languages/typescript.rs](../../../mechanisms/crates/hick-plugins/src/languages/typescript.rs.md#output-1d2ae9a2e366)

### Tests and fixtures

- [crates/hick-dap/tests/debug_coverage.rs](contract.md#output-0a64a5aacbd1)
- [crates/hick-dap/tests/live_session_c.rs](../a-compiled-language-launches-what-a-build-produced/contract.md#output-c4d5afae8600)
- [crates/hick-dap/tests/live_session_cpp.rs](../a-compiled-language-launches-what-a-build-produced/contract.md#output-e8bb12be9677)
- [crates/hick-dap/tests/live_session_go.rs](../../../mechanisms/crates/hick-dap/tests/live_session_go.rs.md#output-eb0b9ddf7573)

- [crates/hick-plugins/src/languages/dap.rs](../../../mechanisms/crates/hick-plugins/src/languages/dap.rs.md#output-3fdb9991751a)

- [crates/hick-plugins/src/languages/react.rs](../../../mechanisms/crates/hick-plugins/src/languages/react.rs.md#output-6a691c076e46)

- [crates/hick-plugins/src/languages/ruby.rs](../../../mechanisms/crates/hick-plugins/src/languages/ruby.rs.md#output-17056a3abebc)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
