//! A recording is keyed by what the cell READ, not only by what it says.
//!
//! Guarantee: `docs/guarantees/verification/a-recording-is-keyed-by-the-cells-inputs.md`
//!
//! The failure this protects against is silent and expensive: a document
//! assembles a file, a cell runs that file, the file changes, the command
//! does not — and the document reports numbers produced by code it no longer
//! contains. Nothing about that looks wrong from the outside.

use std::path::Path;
use std::process::Command;

fn hick() -> Command {
    Command::new(env!("CARGO_BIN_EXE_hick"))
}

/// A document whose exec reads a file the document itself assembles. The
/// command text is identical no matter what `numbers` is, so the command
/// alone cannot distinguish two versions of this document.
fn doc_with(numbers: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="report.md">
# Report

<hick:container name="py" image="python:3.12" />

<hick:file path="count.py">
NUMBERS = {numbers}
print("total:", sum(NUMBERS))
</hick:file>

<hick:volume name="src" input="." />

<hick:exec container="py" mount="src:proj">
python3 proj/count.py
</hick:exec>
</hick:doc>
"#
    )
}

/// Run `hick run --cache`, returning how many cells were answered from a
/// recording.
fn run_cached(dir: &Path) -> usize {
    let out = hick()
        .arg("run")
        .arg(dir.join("report.hick"))
        .arg("--cache")
        // Run FROM the project, the way a person does. Not cosmetic: the
        // local executor names its scratch root after the process's working
        // directory (`LocalExecutor::scratch_root`, which exists so a cell
        // that prints its own cwd reproduces). Without this every test in
        // this file inherits the harness's cwd, so the two below derive the
        // SAME root, contend for its one lock, and whichever loses silently
        // falls back to a random directory — a different input digest, a
        // cache miss, and an assertion that fails on `hits: 0`. Worse, a
        // process that exits releases the lock while its files are still
        // there, so the other test's `count.py` gets read: this file's
        // sharpest symptom was `total: 10` in the document that says
        // `[1, 2, 3]`. Serially it always passed; in parallel it always
        // failed.
        .current_dir(dir)
        .env("RUST_LOG", "info")
        .output()
        .expect("run hick");
    String::from_utf8_lossy(&out.stderr)
        .lines()
        .filter(|l| l.contains("Cache hit for exec"))
        .count()
}

fn write_doc(dir: &Path, numbers: &str) {
    std::fs::write(dir.join("report.hick"), doc_with(numbers)).expect("write doc");
}

fn transcript_total(dir: &Path) -> String {
    let woven = std::fs::read_to_string(dir.join("report.md")).expect("read weave");
    woven
        .lines()
        .find(|l| l.starts_with("total:"))
        .unwrap_or("<no total in weave>")
        .to_string()
}

/// The whole point: identical command, different input, no stale answer.
#[test]
fn a_changed_input_file_re_executes_the_cell_that_reads_it() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_doc(dir.path(), "[1, 2, 3]");

    // Establish the recording, then settle: the assembled file is written to
    // disk by the run that produces it, so the first run after a change sees
    // the previous version of it on disk.
    run_cached(dir.path());
    run_cached(dir.path());
    assert_eq!(
        run_cached(dir.path()),
        1,
        "an unchanged document should answer its cell from the recording"
    );
    assert_eq!(transcript_total(dir.path()), "total: 6");

    // Change ONLY the data the file carries. The command is byte-identical.
    write_doc(dir.path(), "[10, 20, 30]");
    assert_eq!(
        run_cached(dir.path()),
        0,
        "a changed input must not be answered from the old recording"
    );

    run_cached(dir.path());
    assert_eq!(
        transcript_total(dir.path()),
        "total: 60",
        "the document must report what its current code produces"
    );
}

/// Invalidation must not be permanent: once the new inputs are settled, the
/// cell is served from its recording again. A key that never stabilises is
/// not a cache.
#[test]
fn an_unchanged_document_settles_back_onto_its_recording() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_doc(dir.path(), "[5, 5]");

    run_cached(dir.path());
    run_cached(dir.path());

    for attempt in 0..3 {
        assert_eq!(
            run_cached(dir.path()),
            1,
            "repeat run {attempt} should have hit the recording"
        );
    }

    // And the recording set stops growing. It does not settle at exactly one:
    // the very first run happens before the document's assembled file exists
    // on disk, so it reads a different input set than every run after it and
    // records under a key nothing looks up again. That is one wasted entry per
    // cold start, not a leak — what would be a leak is growth per run, which
    // is what this asserts against.
    let settled = walk_files(&dir.path().join(".hick-cache"));
    for _ in 0..3 {
        run_cached(dir.path());
    }
    assert_eq!(
        walk_files(&dir.path().join(".hick-cache")),
        settled,
        "a stable document must not record a new entry on every run"
    );
}

fn walk_files(dir: &Path) -> usize {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    entries
        .filter_map(Result::ok)
        .map(|e| {
            let path = e.path();
            if path.is_dir() { walk_files(&path) } else { 1 }
        })
        .sum()
}
