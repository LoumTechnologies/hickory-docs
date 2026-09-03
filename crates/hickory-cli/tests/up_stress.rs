//! `hick up` under load: rapid edits, interleaved edits, and edits that arrive
//! while the loop is busy.
//!
//! Guarantees stressed here:
//! - `docs/guarantees/authoring/an-output-edit-lands-in-its-document.md`
//! - `docs/guarantees/authoring/one-loop-owns-a-directory.md`
//!
//! ## How these avoid being flaky
//!
//! Nothing here asserts that something happened *within* a duration. Every
//! assertion is about the state the system settles into once the storm stops,
//! polled up to a generous deadline. A timing-sensitive test of a debounced
//! filesystem loop would fail on a loaded CI runner for reasons that say
//! nothing about the code, and `.instructions/pre-commit-ci-parity.md` treats a
//! flaky test as a defect — so the properties are chosen to be true regardless
//! of how the scheduler interleaves things:
//!
//! * **Convergence** — after the last write, the document and its outputs agree.
//! * **The last write wins** — the final value is the one written last, not an
//!   earlier one that a race let through.
//! * **Never corrupt** — the document parses at every moment, even mid-storm.
//!
//! "Eventually correct" is the honest contract for a loop driven by filesystem
//! events, and it is also the one a user cares about.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

mod common;
use common::sleeps_then_echoes;

fn hick() -> Command {
    Command::new(env!("CARGO_BIN_EXE_hick"))
}

/// A document whose generated file is entirely editable, so an output edit has
/// somewhere to land.
const DOC: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="notes.md">
# Notes

Prose that belongs to the document.

<hick:file path="notes.py">
MARKER = "start"


def describe():
    return MARKER
</hick:file>
</hick:doc>
"#;

/// A document with a cell slow enough that an edit can land mid-run.
///
/// The cell's slowness is the entire fixture: the test below types into the
/// output while the run is in flight, and a cell that finishes instantly means
/// there is no run to type during — every assertion then holds for a document
/// that was never busy. `sleep` is not a cmd builtin, so on Windows this cell
/// used to fail immediately and the race the test exists for was never
/// created. `sleeps_then_echoes` spells the same two seconds in whichever
/// shell the cell is actually handed to.
fn slow_doc() -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="slow.md">
# Slow

<hick:container name="sh" image="alpine:3.20" />

<hick:file path="slow.py">
MARKER = "start"
</hick:file>

<hick:exec container="sh">
{}
</hick:exec>
</hick:doc>
"#,
        sleeps_then_echoes(2, "settled")
    )
}

struct Loop {
    child: Child,
    dir: tempfile::TempDir,
}

impl Loop {
    fn start(doc_name: &str, doc: &str, run: bool) -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join(doc_name), doc).expect("write doc");
        let mut cmd = hick();
        cmd.arg("up").arg(dir.path());
        if run {
            cmd.arg("--run");
        }
        // The loop's own narration is captured, not discarded: when one of
        // these tests fails it is almost always because the loop refused
        // something, and the refusal message says which.
        let log = dir.path().join("up.log");
        let sink = std::fs::File::create(&log).expect("create log");
        let child = cmd
            .stdout(Stdio::null())
            .stderr(Stdio::from(sink))
            .spawn()
            .expect("spawn hick up");
        Self { child, dir }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.dir.path().join(name)
    }

    /// What the loop said, for a failure message worth reading.
    fn log(&self) -> String {
        std::fs::read_to_string(self.path("up.log")).unwrap_or_default()
    }

    /// Wait for `pred`, and panic with the loop's own output when it never
    /// holds.
    fn settle_or_explain(&self, path: &Path, what: &str, pred: impl Fn(&str) -> bool) -> String {
        match settle(path, pred) {
            Some(content) => content,
            None => panic!(
                "{what}\n--- {} now:\n{}\n--- the loop said:\n{}",
                path.display(),
                std::fs::read_to_string(path).unwrap_or_default(),
                self.log()
            ),
        }
    }
}

impl Drop for Loop {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Wait until `pred` holds, up to a deadline generous enough for a loaded
/// machine. Returns the content that satisfied it.
fn settle(path: &Path, pred: impl Fn(&str) -> bool) -> Option<String> {
    let deadline = Instant::now() + Duration::from_secs(30);
    while Instant::now() < deadline {
        if let Ok(content) = std::fs::read_to_string(path)
            && pred(&content)
        {
            return Some(content);
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    None
}

/// Save the way a well-behaved editor does: write a sibling, rename over.
fn save(path: &Path, content: &str) {
    let tmp = path.with_extension("stress-tmp");
    std::fs::write(&tmp, content).expect("write temp");
    std::fs::rename(&tmp, path).expect("rename");
}

/// Replace the marker in whatever is on disk right now.
fn set_marker(path: &Path, marker: &str) {
    let current = std::fs::read_to_string(path).expect("read");
    let replaced = current
        .lines()
        .map(|l| {
            if l.trim_start().starts_with("MARKER = ") {
                format!("MARKER = \"{marker}\"")
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    save(path, &format!("{replaced}\n"));
}

/// A document that does not parse is corruption, whatever else is true.
fn assert_parses(doc: &Path) {
    let source = std::fs::read_to_string(doc).expect("read doc");
    assert!(
        hick_lang::parse(&source).is_ok(),
        "the document stopped parsing:\n{source}"
    );
}

// ---------------------------------------------------------------------------
// Rapid edits to one output
// ---------------------------------------------------------------------------

/// Twenty saves in a row, as fast as the filesystem takes them. The document
/// must end up holding the LAST one — not an earlier value that a coalesced
/// batch let through, and not a mixture.
#[test]
fn rapid_output_edits_converge_on_the_last_one() {
    let up = Loop::start("notes.hick", DOC, false);
    let output = up.path("notes.py");
    let doc = up.path("notes.hick");
    settle(&output, |c| c.contains("start")).expect("initial weave");

    for i in 0..20 {
        set_marker(&output, &format!("edit{i}"));
        std::thread::sleep(Duration::from_millis(25));
    }

    let settled = settle(&doc, |c| c.contains("edit19")).expect("the last edit must land");
    assert!(settled.contains("edit19"), "{settled}");
    // Exactly one marker line survives: a race that appended rather than
    // replaced would leave several.
    assert_eq!(
        settled.matches("MARKER = ").count(),
        1,
        "the document should hold one marker:\n{settled}"
    );
    assert_parses(&doc);
}

/// The same, with no pause at all between saves — every event lands inside one
/// debounce window, so the loop sees a single batch covering many writes.
#[test]
fn a_burst_with_no_pause_still_lands() {
    let up = Loop::start("notes.hick", DOC, false);
    let output = up.path("notes.py");
    let doc = up.path("notes.hick");
    settle(&output, |c| c.contains("start")).expect("initial weave");

    for i in 0..30 {
        set_marker(&output, &format!("burst{i}"));
    }

    let settled = settle(&doc, |c| c.contains("burst29")).expect("the last write must win");
    assert_eq!(settled.matches("MARKER = ").count(), 1, "{settled}");
    assert_parses(&doc);
}

// ---------------------------------------------------------------------------
// Rapid edits to the document
// ---------------------------------------------------------------------------

/// Editing the source quickly must leave the generated file agreeing with it.
#[test]
fn rapid_document_edits_converge_into_the_output() {
    let up = Loop::start("notes.hick", DOC, false);
    let doc = up.path("notes.hick");
    let output = up.path("notes.py");
    settle(&output, |c| c.contains("start")).expect("initial weave");

    for i in 0..20 {
        set_marker(&doc, &format!("src{i}"));
        std::thread::sleep(Duration::from_millis(25));
    }

    settle(&output, |c| c.contains("src19")).expect("the output must catch up");
    assert_parses(&doc);
}

// ---------------------------------------------------------------------------
// Both ends at once
// ---------------------------------------------------------------------------

/// Edits arriving from both directions at once must not corrupt the document.
///
/// This does not assert which side wins — with two writers and no ordering
/// between them, "the document still parses" is the strongest honest claim.
/// The two sides either agree, or an output edit that was computed against
/// a document that had already moved is HELD: the file keeps what was typed
/// into it, the loop says so, and nothing is undone. A test that demanded a
/// particular winner would be asserting a scheduling accident; one that
/// demanded agreement after a hold would be asking for the destruction the
/// hold exists to prevent
/// (docs/guarantees/authoring/an-output-that-cannot-be-carried-back-is-held.md).
#[test]
fn interleaved_document_and_output_edits_never_corrupt() {
    let up = Loop::start("notes.hick", DOC, false);
    let doc = up.path("notes.hick");
    let output = up.path("notes.py");
    settle(&output, |c| c.contains("start")).expect("initial weave");

    for i in 0..12 {
        if i % 2 == 0 {
            set_marker(&doc, &format!("d{i}"));
        } else {
            set_marker(&output, &format!("o{i}"));
        }
        std::thread::sleep(Duration::from_millis(40));
        assert_parses(&doc);
    }

    // Let it quiesce. Then either the two sides agree, or the output is
    // held with an edit typed into it and the loop said so.
    std::thread::sleep(Duration::from_secs(2));
    let doc_text = std::fs::read_to_string(&doc).expect("read doc");
    let out_text = std::fs::read_to_string(&output).expect("read output");
    let doc_marker = marker_of(&doc_text).expect("document has a marker");
    let out_marker = marker_of(&out_text).expect("output has a marker");
    if doc_marker != out_marker {
        let log = up.log();
        assert!(
            log.contains("diverged (held)"),
            "document and output disagree and the loop never said it was holding the \
             file:\ndoc={doc_marker}\nout={out_marker}\n{log}"
        );
        assert!(
            marker_value(&out_text).is_some_and(|v| v.starts_with('o')),
            "a held file must hold what was typed into it, not a weave: {out_marker}"
        );
    }
    assert_parses(&doc);
}

/// The marker's value alone: `MARKER = "x"` → `x`.
fn marker_value(text: &str) -> Option<String> {
    marker_of(text).map(|line| {
        line.trim_start_matches("MARKER = ")
            .trim_matches('"')
            .to_string()
    })
}

fn marker_of(text: &str) -> Option<String> {
    text.lines()
        .find(|l| l.trim_start().starts_with("MARKER = "))
        .map(|l| l.trim().to_string())
}

// ---------------------------------------------------------------------------
// Editing while the loop is busy
// ---------------------------------------------------------------------------

/// An edit saved while a run is in flight must not be thrown away.
///
/// This is the case a debounce cannot help with: under `--run` the loop sits
/// inside a multi-second execution when the save arrives, and the weave that
/// finishes afterwards was computed from bytes that predate the edit. Writing
/// those bytes out would delete the user's typing *and* make the pending event
/// look like an echo of the loop's own write — so it would vanish with no
/// error, which is the worst way to lose work.
#[test]
fn an_edit_during_a_run_is_not_lost() {
    let up = Loop::start("slow.hick", &slow_doc(), true);
    let output = up.path("slow.py");
    let doc = up.path("slow.hick");
    settle(&output, |c| c.contains("start")).expect("initial weave");

    // The first edit starts a re-run, because carrying it into the document
    // makes the document dirty. Nothing edits the document directly here: an
    // edit made against spans the user has since changed themselves is
    // genuinely unsafe to apply, and this test is about the case that is safe.
    set_marker(&output, "first");
    up.settle_or_explain(&doc, "the first edit never reached the document", |c| {
        c.contains("first")
    });

    // The cell now sleeps for two seconds. Type into the output while it does.
    std::thread::sleep(Duration::from_millis(400));
    set_marker(&output, "typed-during-run");

    let settled = settle(&doc, |c| c.contains("typed-during-run"))
        .expect("an edit typed during a run must survive it");
    assert!(settled.contains("typed-during-run"), "{settled}");
    assert_eq!(settled.matches("MARKER = ").count(), 1, "{settled}");
    assert_parses(&doc);

    // And the two sides agree once everything settles.
    settle(&output, |c| c.contains("typed-during-run")).expect("the output catches up");
}

/// An output edit made while the *document* is also changing is refused — the
/// spans it was computed against no longer describe the document — and the
/// file is HELD with the edit in it, never put back.
///
/// It used to be restored to agree with the document, on the argument that
/// a silent fork is worse than a refusal. The fork is now loud instead of
/// silent — the loop names the file and why, the tree and the pane show it —
/// and what was typed stays, because the loop has nothing truer to put in
/// its place. Guarantee:
/// docs/guarantees/authoring/an-output-that-cannot-be-carried-back-is-held.md
#[test]
fn a_refused_edit_holds_the_file_with_the_edit_in_it() {
    let up = Loop::start("notes.hick", DOC, false);
    let doc = up.path("notes.hick");
    let output = up.path("notes.py");
    settle(&output, |c| c.contains("start")).expect("initial weave");

    // Both sides inside one debounce window, so the output edit is computed
    // against a document that has already moved.
    set_marker(&doc, "from-document");
    set_marker(&output, "from-output");

    std::thread::sleep(Duration::from_secs(3));

    let doc_text = std::fs::read_to_string(&doc).expect("read doc");
    let out_text = std::fs::read_to_string(&output).expect("read output");
    let log = up.log();
    match (
        marker_value(&doc_text).as_deref(),
        marker_value(&out_text).as_deref(),
    ) {
        // The output edit won the race and was carried back: both agree.
        (Some("from-output"), Some("from-output")) => {}
        // The document edit won and the output edit was refused: the file
        // keeps what was typed, the document keeps its own, and the loop
        // said it was holding the file.
        (Some("from-document"), Some("from-output")) => {
            assert!(
                log.contains("diverged (held)"),
                "a refused edit was not reported as held:\n{log}"
            );
        }
        // The document edit landed and its weave reached the file before the
        // output edit was even seen: agreement, nothing refused.
        (Some("from-document"), Some("from-document")) => {}
        other => panic!("an outcome that undid somebody's typing: {other:?}\n{log}"),
    }
    assert_parses(&doc);
}

// ---------------------------------------------------------------------------
// Editors that do not save atomically
// ---------------------------------------------------------------------------

/// Some editors truncate the file and write into it, so a reader can observe
/// an empty or half-written file. None of those intermediate states may reach
/// the document.
#[test]
fn a_non_atomic_save_does_not_push_a_partial_file_into_the_document() {
    use std::io::Write as _;

    let up = Loop::start("notes.hick", DOC, false);
    let output = up.path("notes.py");
    let doc = up.path("notes.hick");
    settle(&output, |c| c.contains("start")).expect("initial weave");

    let target = std::fs::read_to_string(&output)
        .expect("read")
        .replace("\"start\"", "\"slowly-written\"");

    // Truncate, then dribble the content in.
    //
    // Two numbers make this deterministic, and BOTH were wrong before:
    //
    // * the pause must exceed the debounce, or the loop never sees a quiet
    //   directory holding a half-written file. It paused 30ms against a 120ms
    //   weave debounce, so the batch fired only when a machine was slow enough
    //   to stall a chunk past 120ms — the test passed by arithmetic, and failed
    //   the first time it ran on a loaded CI runner;
    // * the chunk must be smaller than the file, or there is no partial state
    //   at all. `notes.py` is about 60 bytes, so 64-byte chunks wrote it in one
    //   go and the test proved nothing while passing in a second.
    //
    // 16 bytes and 200ms give several genuinely partial states, the first of
    // which already carries the marker: `MARKER = "slowly-written"` sits at the
    // top of the file and `def describe():` below it.
    let writer = {
        let output = output.clone();
        std::thread::spawn(move || {
            let mut file = std::fs::File::create(&output).expect("truncate");
            for chunk in target.as_bytes().chunks(16) {
                file.write_all(chunk).expect("write chunk");
                file.flush().expect("flush");
                std::thread::sleep(Duration::from_millis(200));
            }
        })
    };

    // Watch the document WHILE it is written, because the violation is
    // transient: once the last chunk lands the loop maps the whole file back
    // and the document heals itself. Asserting only on the final state catches
    // this the way CI did — occasionally, on a slow machine, at random.
    //
    // A document carrying the new marker without the rest of the file is a
    // half-written save that reached it, which is exactly what the guarantee
    // forbids.
    // `def describe():` is in the document before the save starts and in the
    // file being written, so it must be there at EVERY instant in between. Its
    // disappearance means a truncated read was mapped back and ate it.
    //
    // Checked continuously rather than at the end, because the violation is
    // transient — once the last chunk lands the loop maps the whole file back
    // and the document heals. Asserting only on the final state catches this
    // the way CI did: occasionally, on a slow machine, at random.
    let mut partial_seen: Option<String> = None;
    while !writer.is_finished() {
        if let Ok(content) = std::fs::read_to_string(&doc)
            && !content.contains("def describe():")
        {
            partial_seen = Some(content);
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    writer.join().expect("the writer finished");

    assert!(
        partial_seen.is_none(),
        "a half-written save reached the document:\n{}",
        partial_seen.unwrap_or_default()
    );

    let settled =
        settle(&doc, |c| c.contains("slowly-written")).expect("the finished content must land");
    assert!(settled.contains("slowly-written"), "{settled}");
    // The prose around the block must survive: a partial read mapped back
    // would have eaten it.
    assert!(
        settled.contains("Prose that belongs to the document."),
        "{settled}"
    );
    assert!(settled.contains("def describe():"), "{settled}");
    assert_parses(&doc);
}

// ---------------------------------------------------------------------------
// Many documents at once
// ---------------------------------------------------------------------------

/// Ten documents, every output edited in the same instant. Each edit belongs
/// to exactly one document and must land there.
#[test]
fn edits_across_many_documents_all_land() {
    let dir = tempfile::tempdir().expect("tempdir");
    for i in 0..10 {
        let doc = DOC
            .replace("weave=\"notes.md\"", &format!("weave=\"notes{i}.md\""))
            .replace("path=\"notes.py\"", &format!("path=\"notes{i}.py\""));
        std::fs::write(dir.path().join(format!("notes{i}.hick")), doc).expect("write doc");
    }
    let child = hick()
        .arg("up")
        .arg(dir.path())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn");
    let _guard = Kill(child);

    for i in 0..10 {
        settle(&dir.path().join(format!("notes{i}.py")), |c| {
            c.contains("start")
        })
        .unwrap_or_else(|| panic!("initial weave of document {i}"));
    }

    for i in 0..10 {
        set_marker(&dir.path().join(format!("notes{i}.py")), &format!("doc{i}"));
    }

    for i in 0..10 {
        let doc = dir.path().join(format!("notes{i}.hick"));
        let settled = settle(&doc, |c| c.contains(&format!("doc{i}")))
            .unwrap_or_else(|| panic!("edit for document {i} never landed"));
        // Each document got ITS marker, not a neighbour's.
        for other in 0..10 {
            if other != i {
                assert!(
                    !settled.contains(&format!("\"doc{other}\"")),
                    "document {i} received document {other}'s edit:\n{settled}"
                );
            }
        }
        assert_parses(&doc);
    }
}

struct Kill(Child);
impl Drop for Kill {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
