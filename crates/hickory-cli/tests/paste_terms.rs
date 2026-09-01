//! What a `<hick:paste>` collects, on terms it can be held to: compound class
//! selectors mean "all of these", `distinct` collapses repeats, and an unmet
//! `min=`/`max=` fails the run instead of quietly weaving an empty file.
//! Guarantee: docs/guarantees/authoring/a-paste-collects-on-terms-it-can-be-held-to.md

fn hick() -> std::process::Command {
    std::process::Command::new(env!("CARGO_BIN_EXE_hick"))
}

/// Write one document into a fresh directory and return (dir, doc path).
fn doc(source: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("n.hick");
    std::fs::write(&path, source).unwrap();
    (dir, path)
}

#[test]
fn a_compound_class_selector_requires_every_class() {
    let (dir, path) = doc(
        r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="n.md">
<hick:copy class="usings dto">using System;</hick:copy>
<hick:copy class="usings test">using Xunit;</hick:copy>
<hick:copy class="body dto">record Pallet();</hick:copy>

<hick:file path="out.txt">[<hick:paste select=".usings.dto" />]</hick:file>
</hick:doc>
"##,
    );
    let ok = hick()
        .args(["weave", path.to_str().unwrap()])
        .status()
        .unwrap()
        .success();
    assert!(ok, "weave failed");
    let out = std::fs::read_to_string(dir.path().join("out.txt")).unwrap();
    // `.usings` alone would also collect `using Xunit;`, and `.dto` alone
    // would also collect the record. Both classes together match one block.
    assert_eq!(out, "[using System;]");
}

#[test]
fn a_comma_is_still_union_so_the_two_forms_do_not_collide() {
    let (dir, path) = doc(
        r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="n.md">
<hick:copy class="a">1</hick:copy>
<hick:copy class="b">2</hick:copy>

<hick:file path="out.txt">[<hick:paste select=".a,.b" separator="," />]</hick:file>
</hick:doc>
"##,
    );
    assert!(
        hick()
            .args(["weave", path.to_str().unwrap()])
            .status()
            .unwrap()
            .success()
    );
    assert_eq!(
        std::fs::read_to_string(dir.path().join("out.txt")).unwrap(),
        "[1,2]"
    );
}

#[test]
fn distinct_keeps_the_first_of_repeated_text() {
    let (dir, path) = doc(
        r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="n.md">
<hick:copy class="ignore">bin/</hick:copy>
<hick:copy class="ignore">obj/</hick:copy>
<hick:copy class="ignore">bin/</hick:copy>

<hick:file path="out.txt"><hick:paste select=".ignore" distinct separator="," /></hick:file>
</hick:doc>
"##,
    );
    assert!(
        hick()
            .args(["weave", path.to_str().unwrap()])
            .status()
            .unwrap()
            .success()
    );
    assert_eq!(
        std::fs::read_to_string(dir.path().join("out.txt")).unwrap(),
        "bin/,obj/"
    );
}

#[test]
fn an_unmet_min_fails_the_run_and_writes_nothing() {
    let (dir, path) = doc(
        r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="n.md">
<hick:file path="out.txt"><hick:paste select=".nobody" min="1" /></hick:file>
</hick:doc>
"##,
    );
    // The regression this exists for: this used to log a warning, write an
    // EMPTY out.txt, and exit 0 from both `run` and `test`.
    for verb in ["run", "test"] {
        let out = hick()
            .args([verb, path.to_str().unwrap()])
            .output()
            .unwrap();
        assert!(!out.status.success(), "`hick {verb}` passed an unmet min=");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains("min=1"),
            "`hick {verb}` did not say which gate failed:\n{stderr}"
        );
    }
    // Known and deliberate: `run_doc_cached` STAGES the woven files before
    // executing, so a cell can run a file its own document assembles on the
    // first run. That staging weave cannot fairly judge a gate — no cell has
    // run yet — so it writes `out.txt` empty and the executing pass is what
    // refuses. The run fails loudly, which is the guarantee; the staged file
    // is left behind, which is a separate wart recorded in the guarantee.
    let staged = dir.path().join("out.txt");
    assert!(
        staged.exists() && std::fs::read_to_string(&staged).unwrap().is_empty(),
        "staging behaviour changed — if the staged file is now cleaned up, \
         update the guarantee's caveat and assert its absence instead"
    );
}

#[test]
fn min_counts_what_distinct_will_actually_emit() {
    let (_dir, path) = doc(
        r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="n.md">
<hick:copy class="p">same</hick:copy>
<hick:copy class="p">same</hick:copy>

<hick:file path="out.txt"><hick:paste select=".p" distinct min="2" /></hick:file>
</hick:doc>
"##,
    );
    // Two blocks match, but they collapse to one line — so a gate asking for
    // two was never satisfied, and counting matches rather than output would
    // have said it was.
    let out = hick()
        .args(["run", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!out.status.success(), "min= counted matches, not output");
}

#[test]
fn a_gate_is_checked_against_a_run_not_against_a_weave() {
    let (_dir, path) = doc(
        r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="n.md">
<hick:file path="out.txt"><hick:paste select=".nobody" min="1" /></hick:file>
</hick:doc>
"##,
    );
    // `weave` executes nothing on purpose, and a fragment a cell writes does
    // not exist until that cell has run. Failing here would report every such
    // document as broken.
    assert!(
        hick()
            .args(["weave", path.to_str().unwrap()])
            .status()
            .unwrap()
            .success(),
        "weave enforced a gate it cannot fairly judge"
    );
}
