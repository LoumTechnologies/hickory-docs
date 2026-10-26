<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/tests/cache_inputs.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-518e60ffbf9d"></a>
## `crates/hickory-cli/tests/cache_inputs.rs`

Related contracts: [A Cell's Own Path Belongs To Its Project, Not To Your Shell](../../../../guarantees/execution/a-cells-own-path-belongs-to-its-project/contract.md), [Reuse a recording only for the inputs that produced it](../../../../guarantees/verification/a-recording-is-keyed-by-the-cells-inputs/contract.md).

[Read the test `hick` beside its requirement](../../../../guarantees/verification/a-recording-is-keyed-by-the-cells-inputs/contract.md#f-518e60ffbf9d-hick).
<a id="f-518e60ffbf9d-doc_with"></a>
### `doc_with`

A document whose exec reads a file the document itself assembles. The
command text is identical no matter what `numbers` is, so the command
alone cannot distinguish two versions of this document.

<storycode:copy id="f-518e60ffbf9d-doc_with">
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

</storycode:copy>
<a id="f-518e60ffbf9d-run_cached"></a>
### `run_cached`

Run `hick run --cache`, returning how many cells were answered from a
recording.

<storycode:copy id="f-518e60ffbf9d-run_cached">
/// Run `hick run --cache`, returning how many cells were answered from a
/// recording.
fn run_cached(dir: &Path) -> usize {
    let out = hick()
        .arg("run")
        .arg(dir.join("report.md"))
        .arg("--cache")
        // Deliberately NOT `.current_dir(dir)`. These two tests run in
        // parallel, in separate tempdirs, from the harness's own cwd — which
        // is exactly the shape that used to break: the local executor named
        // its scratch root after the PROCESS's working directory, so both
        // derived one name, contended for its single lock, and whichever lost
        // fell back to a random directory. A process that exits releases the
        // lock while its files are still there, so one test read the other's
        // `count.py` and this file's sharpest symptom was `total: 10` in the
        // document that says `[1, 2, 3]`. Serially it always passed; in
        // parallel it failed 6 times in 6. The root is keyed on the
        // document's directory now, so leaving the cwd alone is what proves
        // it.
        .env("RUST_LOG", "info")
        .output()
        .expect("run hick");
    String::from_utf8_lossy(&out.stderr)
        .lines()
        .filter(|l| l.contains("Cache hit for exec"))
        .count()
}

</storycode:copy>
<a id="f-518e60ffbf9d-write_doc"></a>
### `write_doc`

<storycode:copy id="f-518e60ffbf9d-write_doc">
fn write_doc(dir: &Path, numbers: &str) {
    std::fs::write(dir.join("report.md"), doc_with(numbers)).expect("write doc");
}

</storycode:copy>
<a id="f-518e60ffbf9d-transcript_total"></a>
### `transcript_total`

<storycode:copy id="f-518e60ffbf9d-transcript_total">
fn transcript_total(dir: &Path) -> String {
    let woven = std::fs::read_to_string(dir.join("report.md")).expect("read weave");
    woven
        .lines()
        .find(|l| l.starts_with("total:"))
        .unwrap_or("<no total in weave>")
        .to_string()
}

</storycode:copy>
<a id="f-518e60ffbf9d-a_changed_input_file_re_executes_the_cell_that_reads_it"></a>
### `a_changed_input_file_re_executes_the_cell_that_reads_it`

The whole point: identical command, different input, no stale answer.

<storycode:copy id="f-518e60ffbf9d-a_changed_input_file_re_executes_the_cell_that_reads_it">
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

</storycode:copy>
<a id="f-518e60ffbf9d-an_unchanged_document_settles_back_onto_its_recording"></a>
### `an_unchanged_document_settles_back_onto_its_recording`

Invalidation must not be permanent: once the new inputs are settled, the
cell is served from its recording again. A key that never stabilises is
not a cache.

<storycode:copy id="f-518e60ffbf9d-an_unchanged_document_settles_back_onto_its_recording">
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

</storycode:copy>
<a id="f-518e60ffbf9d-walk_files"></a>
### `walk_files`

<storycode:copy id="f-518e60ffbf9d-walk_files">
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
</storycode:copy>

### Assemble `crates/hickory-cli/tests/cache_inputs.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/cache_inputs.rs"><storycode:paste select="#f-518e60ffbf9d-hick"/><storycode:paste select="#f-518e60ffbf9d-doc_with"/><storycode:paste select="#f-518e60ffbf9d-run_cached"/><storycode:paste select="#f-518e60ffbf9d-write_doc"/><storycode:paste select="#f-518e60ffbf9d-transcript_total"/><storycode:paste select="#f-518e60ffbf9d-a_changed_input_file_re_executes_the_cell_that_reads_it"/><storycode:paste select="#f-518e60ffbf9d-an_unchanged_document_settles_back_onto_its_recording"/><storycode:paste select="#f-518e60ffbf9d-walk_files"/></storycode:file>

</storycode:doc>
