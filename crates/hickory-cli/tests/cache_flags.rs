//! `hick run --cache` / `--freeze`, driven through the shipped binary.
//!
//! These protect
//! `docs/guarantees/verification/recordings-are-written-only-when-asked-for.md`:
//! a recording exists only because a run was asked to write one — by a flag,
//! or by the cell's own `freeze="true"` — `hick test` can never ask, and
//! the whole freeze lifecycle is reachable from the CLI.

use std::path::{Path, PathBuf};
use std::process::Command;

mod common;
use common::echo_lines;

/// The one exec command, shared between the frozen and unfrozen spellings of
/// the document so both compute the same cache key.
///
/// A function rather than a `const` because the text is per-shell: cells run
/// through `cmd.exe /C` on Windows, where `printf` does not exist. Nothing
/// here pins the output with `<hick:expect>`, so the CRLF cmd adds is
/// invisible to every assertion below — each looks for the word `one` inside
/// the recording or the woven page.
fn command() -> String {
    format!("\n{}\n", echo_lines(&["one", "two"]))
}

fn hick() -> Command {
    Command::new(env!("CARGO_BIN_EXE_hick"))
}

fn doc_source(freeze: Option<bool>) -> String {
    let attr = match freeze {
        Some(v) => format!(r#" freeze="{v}""#),
        None => String::new(),
    };
    let command = command();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="cached.md">
# Cached

<hick:container name="c" image="alpine:3.20" />

<hick:exec container="c"{attr}>{command}</hick:exec>
</hick:doc>
"#
    )
}

fn write_doc(dir: &Path, freeze: Option<bool>) -> PathBuf {
    let path = dir.join("cached.hick");
    std::fs::write(&path, doc_source(freeze)).unwrap();
    path
}

/// Every recording file under the project's transcript cache.
fn recordings(project_dir: &Path) -> Vec<PathBuf> {
    let root = project_dir.join(".hick-cache").join("transcripts");
    let mut found = Vec::new();
    let Ok(containers) = std::fs::read_dir(&root) else {
        return found;
    };
    for container in containers.flatten() {
        let Ok(entries) = std::fs::read_dir(container.path()) else {
            continue;
        };
        for entry in entries.flatten() {
            if entry.path().extension().is_some_and(|e| e == "json") {
                found.push(entry.path());
            }
        }
    }
    found.sort();
    found
}

#[test]
fn a_flagless_run_records_what_it_ran() {
    // This asserted the opposite until the consequence turned up in use: a
    // flagless run recorded nothing, so the NEXT weave had nothing to replay
    // and wrote `[never run]` over the artifact the run had just produced.
    // The marker reads as "this has never executed" and meant "I have no
    // recording", and with `run` recording nothing the two came apart on
    // every keystroke in the app.
    //
    // The fear the old test was written against — that a run would hand
    // `check` a baseline nobody asked for — is answered by a different
    // method. Recording and CONSULTING are separate: see the test below,
    // which shows a flagless check executing for real with a recording
    // sitting right there.
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), None);
    assert!(hick().arg("run").arg(&doc).status().unwrap().success());
    assert_eq!(
        recordings(dir.path()).len(),
        1,
        "a cell that really ran should be remembered, so a later weave can replay it"
    );
}

#[test]
fn a_flagless_check_is_never_answered_from_a_recording() {
    // The invariant the recording change must not break. `hick run` leaves a
    // recording; `hick test` without `--cache` must still EXECUTE, or a
    // verifier would be checking a document against its own memory.
    //
    // Proven with a cell whose output cannot repeat: if the check replayed
    // the recording it would pass, and it must not.
    //
    // The command differs by platform because a cell is `sh -c` on Unix and
    // `cmd.exe /C` on Windows, and nothing nondeterministic is spelled the
    // same in both. This used to be the Unix one unconditionally, so the
    // `hick run` below failed on Windows with `exit 1` and the test panicked
    // on the assertion beneath it rather than on anything it was testing —
    // the guarantee is not Unix-only and neither should its coverage be.
    // `%TIME%` is centisecond-resolution and the two invocations are seconds
    // apart, so it changes as reliably as urandom does.
    #[cfg(unix)]
    const NONDETERMINISTIC: &str = "head -c 8 /dev/urandom | od -An -tx1 | tr -d ' \n'";
    #[cfg(windows)]
    const NONDETERMINISTIC: &str = "echo %TIME%";

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("live.hick");
    std::fs::write(
        &path,
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="live.md">
# Live

<hick:container name="c" image="alpine:3.20" />

<hick:file path="v.txt"><hick:exec container="c">
<hick:copy id="c1">{NONDETERMINISTIC}</hick:copy>
</hick:exec></hick:file>
</hick:doc>
"#
        ),
    )
    .unwrap();

    assert!(hick().arg("run").arg(&path).status().unwrap().success());
    assert_eq!(recordings(dir.path()).len(), 1, "the run recorded");

    let out = hick().arg("test").arg(&path).output().unwrap();
    assert!(
        !out.status.success(),
        "test must re-execute and see the value change, not replay the recording: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn run_cache_writes_a_recording() {
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), None);
    let out = hick().arg("run").arg("--cache").arg(&doc).output().unwrap();
    assert!(
        out.status.success(),
        "run --cache should succeed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let found = recordings(dir.path());
    assert_eq!(
        found.len(),
        1,
        "one exec should leave exactly one recording, got {found:?}"
    );
    let recorded = std::fs::read_to_string(&found[0]).unwrap();
    assert!(
        recorded.contains("one"),
        "the recording should hold the cell's output: {recorded}"
    );
}

#[test]
fn a_frozen_cell_gets_its_baseline_entirely_through_the_cli() {
    // Issue #6, then #10: the cell is written frozen from the start, and one
    // plain `hick run` — no flag, no edit to the document — gives it the
    // baseline `hick test` then verifies. Exit 0, not 2.
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), Some(true));
    assert!(hick().arg("run").arg(&doc).status().unwrap().success());
    assert_eq!(
        recordings(dir.path()).len(),
        1,
        "the first run of a frozen cell must leave exactly one recording"
    );

    let out = hick().arg("test").arg(&doc).output().unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "the frozen cell has a baseline now, so test is verified: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn a_flagless_run_records_every_cell_it_ran() {
    // Both cells, not just the frozen one. `freeze="true"` decides whether a
    // recording is the ANSWER next time, which is a different question from
    // whether a cell that ran is remembered at all — and the second has no
    // useful "no".
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("mixed.hick");
    let command = command();
    let live = echo_lines(&["live"]);
    std::fs::write(
        &path,
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="mixed.md">
# Mixed

<hick:container name="c" image="alpine:3.20" />

<hick:exec container="c" freeze="true">{command}</hick:exec>
<hick:exec container="c">
{live}
</hick:exec>
</hick:doc>
"#
        ),
    )
    .unwrap();

    assert!(hick().arg("run").arg(&path).status().unwrap().success());
    assert_eq!(
        recordings(dir.path()).len(),
        2,
        "both cells ran, so both are replayable by a later weave"
    );
}

#[test]
fn run_freeze_serves_the_recording_instead_of_executing() {
    // Proved by doctoring the recording: the woven output can only contain
    // this text if the run never asked the executor.
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), None);
    assert!(
        hick()
            .arg("run")
            .arg("--cache")
            .arg(&doc)
            .status()
            .unwrap()
            .success()
    );

    let recording = recordings(dir.path()).remove(0);
    let doctored = std::fs::read_to_string(&recording)
        .unwrap()
        .replace("one", "SERVED-FROM-RECORDING");
    std::fs::write(&recording, doctored).unwrap();

    let out = hick()
        .arg("run")
        .arg("--freeze")
        .arg(&doc)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "run --freeze should succeed when every cell is recorded: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let woven = std::fs::read_to_string(dir.path().join("cached.md")).unwrap();
    assert!(
        woven.contains("SERVED-FROM-RECORDING"),
        "--freeze must answer the cell from its recording, not re-execute it: {woven}"
    );
}

#[test]
fn run_freeze_records_a_cell_that_has_no_recording_yet() {
    // `--freeze` is the run-wide spelling of the same three-valued setting a
    // cell declares, so a miss means the same thing either way: no baseline
    // yet, and `run` is what establishes one. The strict read-only assertion
    // — "every cell is already recorded, execute nothing" — is `hick test
    // --freeze`, which is exercised below.
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), None);
    let out = hick()
        .arg("run")
        .arg("--freeze")
        .arg(&doc)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "an unrecorded cell is a cell with no baseline, not an error: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        recordings(dir.path()).len(),
        1,
        "the run that executed it must record it"
    );

    // Doctor the recording: a second --freeze run can only produce this text
    // by replaying, which is what "executes at most once" means.
    let recording = recordings(dir.path()).remove(0);
    let doctored = std::fs::read_to_string(&recording)
        .unwrap()
        .replace("one", "SERVED-FROM-RECORDING");
    std::fs::write(&recording, doctored).unwrap();
    assert!(
        hick()
            .arg("run")
            .arg("--freeze")
            .arg(&doc)
            .status()
            .unwrap()
            .success()
    );
    let woven = std::fs::read_to_string(dir.path().join("cached.md")).unwrap();
    assert!(
        woven.contains("SERVED-FROM-RECORDING"),
        "the second run must replay the recording the first one wrote: {woven}"
    );
}

#[test]
fn test_refuses_a_cache_flag() {
    // `check` must never be able to write the baseline it then compares
    // against — the circularity `unverifiable` exists to prevent.
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), None);
    let out = hick()
        .arg("test")
        .arg("--cache")
        .arg(&doc)
        .output()
        .unwrap();
    assert!(
        !out.status.success(),
        "`hick test --cache` must not be accepted"
    );
    assert!(
        recordings(dir.path()).is_empty(),
        "check must never write a recording"
    );
}

#[test]
fn test_freeze_reports_an_unrecorded_cell_as_unverifiable() {
    // Read-only freeze on the verifier: nothing to compare against is exit 2,
    // not a silent pass and not a recording written on the spot.
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), None);
    let out = hick()
        .arg("test")
        .arg("--freeze")
        .arg(&doc)
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(2),
        "no baseline is unverifiable: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        recordings(dir.path()).is_empty(),
        "check --freeze must not write a recording"
    );
}
