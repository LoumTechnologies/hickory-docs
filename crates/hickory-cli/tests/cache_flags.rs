//! `hick run --cache` / `--freeze`, driven through the shipped binary.
//!
//! These protect
//! `docs/guarantees/verification/recordings-are-written-only-when-asked-for.md`:
//! a recording exists only because a run was asked to write one — by a flag,
//! or by the cell's own `freeze="true"` — `hick test` can never ask, and
//! the whole freeze lifecycle is reachable from the CLI.

use std::path::{Path, PathBuf};
use std::process::Command;

fn hick() -> Command {
    Command::new(env!("CARGO_BIN_EXE_hick"))
}

/// The one exec command, shared between the frozen and unfrozen spellings of
/// the document so both compute the same cache key.
const COMMAND: &str = "\nprintf 'one\\ntwo\\n'\n";

fn doc_source(freeze: Option<bool>) -> String {
    let attr = match freeze {
        Some(v) => format!(r#" freeze="{v}""#),
        None => String::new(),
    };
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="cached.md">
# Cached

<hick:container name="c" image="alpine:3.20" />

<hick:exec container="c"{attr}>{COMMAND}</hick:exec>
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
fn run_without_the_flag_records_nothing() {
    // The default has to stay "ask the world, remember nothing": a run that
    // recorded by accident would hand `check` a baseline nobody asked for.
    let dir = tempfile::tempdir().unwrap();
    let doc = write_doc(dir.path(), None);
    assert!(hick().arg("run").arg(&doc).status().unwrap().success());
    assert!(
        recordings(dir.path()).is_empty(),
        "plain `hick run` must not write a recording"
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
fn a_frozen_cell_is_the_only_thing_a_flagless_run_records() {
    // The default stays "ask the world, remember nothing" for every cell that
    // did not ask to be remembered. Only the frozen cell is recorded.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("mixed.hick");
    std::fs::write(
        &path,
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="mixed.md">
# Mixed

<hick:container name="c" image="alpine:3.20" />

<hick:exec container="c" freeze="true">{COMMAND}</hick:exec>
<hick:exec container="c">
printf 'live'
</hick:exec>
</hick:doc>
"#
        ),
    )
    .unwrap();

    assert!(hick().arg("run").arg(&path).status().unwrap().success());
    assert_eq!(
        recordings(dir.path()).len(),
        1,
        "only the cell that declared freeze=\"true\" asked to be recorded"
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
