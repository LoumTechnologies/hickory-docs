//! `<hick:needs>`, through the real binary.
//!
//! Protects docs/guarantees/execution/a-document-says-what-it-needs.md
//!
//! Driven as a user drives it — `hick run` on a document — because the claim
//! is about what happens to a person whose machine is missing something, and
//! that includes the exit status, the message, and crucially the fact that
//! nothing ran first.

use std::path::PathBuf;
use std::process::Command;

/// The `hick` this workspace just built.
fn hick() -> Option<PathBuf> {
    let mut path = std::env::current_exe().ok()?;
    path.pop();
    path.pop();
    let candidate = path.join(if cfg!(windows) { "hick.exe" } else { "hick" });
    candidate.is_file().then_some(candidate)
}

fn document(needs: &str, body: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\" weave=\"out.md\">\n\
         # Needs\n\n\
         <hick:container name=\"lab\" image=\"alpine\">\n\
         {needs}\
         </hick:container>\n\n\
         <hick:exec container=\"lab\">\n\
         {body}\n\
         </hick:exec>\n\
         </hick:doc>\n"
    )
}

struct Run {
    ok: bool,
    output: String,
    woven: Option<String>,
    /// Whether the cell's own side effect happened.
    side_effect: bool,
}

fn run(document_text: &str) -> Option<Run> {
    let hick = hick()?;
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("d.hick"), document_text).unwrap();
    let out = Command::new(hick)
        .arg("run")
        .arg("d.hick")
        .current_dir(dir.path())
        .output()
        .expect("hick runs");
    // A transcript contains the COMMAND as well as its output, so a cell
    // that echoed a marker would put that marker in the document whether or
    // not it ever ran — the assertion would be reading its own fixture back.
    // The marker is therefore assembled by `tr`: lowercase in the command,
    // uppercase only if something actually executed.
    let woven = std::fs::read_to_string(dir.path().join("out.md")).ok();
    let side_effect = woven.as_deref().is_some_and(|text| text.contains("ITRAN"));
    Some(Run {
        ok: out.status.success(),
        output: format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
        woven,
        side_effect,
    })
}

#[test]
fn a_missing_program_stops_the_document_before_anything_runs() {
    let Some(result) = run(&document(
        "  <hick:needs bin=\"definitely-not-a-real-program\" for=\"the analysis\" />\n",
        "echo itran | tr a-z A-Z",
    )) else {
        eprintln!("SKIPPED: hick has not been built into this target dir");
        return;
    };

    assert!(!result.ok, "a document with a missing program succeeded");
    assert!(
        result.output.contains("definitely-not-a-real-program"),
        "the message does not name what is missing: {}",
        result.output
    );
    // The reason the author wrote, so a reader can decide whether they care.
    assert!(result.output.contains("the analysis"), "{}", result.output);
    // And the line, so they can find the declaration.
    assert!(result.output.contains("line 6"), "{}", result.output);

    // And nothing ran. Half a pipeline's side effects followed by
    // `not found` is the worst ordering available: the document has already
    // changed things and still cannot finish.
    //
    // (`hick run` writes the woven file even when a run fails — that is how
    // it has always behaved, for ordinary cell failures too — so the check
    // is that the cell produced no OUTPUT, not that no file was written.)
    assert!(
        !result.side_effect,
        "a cell ran despite the document missing a program it declared:\n{}",
        result.woven.unwrap_or_default()
    );
    assert!(
        result
            .woven
            .as_deref()
            .is_some_and(|text| text.contains("[never run]")),
        "the woven document does not record the cell as never run:\n{}",
        result.woven.unwrap_or_default()
    );
}

#[test]
fn every_missing_program_is_named_at_once() {
    // Three round trips to learn about three missing programs is how a person
    // decides a tool hates them.
    let Some(result) = run(&document(
        "  <hick:needs bin=\"not-real-one\" />\n  <hick:needs bin=\"not-real-two\" />\n",
        "echo hi",
    )) else {
        return;
    };
    assert!(result.output.contains("not-real-one"), "{}", result.output);
    assert!(result.output.contains("not-real-two"), "{}", result.output);
    assert!(result.output.contains("2 programs"), "{}", result.output);
}

#[test]
fn a_program_that_is_present_lets_the_document_run() {
    // `sh` is the one program that must exist for any of this to work at all,
    // so it is the only safe thing to assert is installed on a test machine.
    let Some(result) = run(&document(
        "  <hick:needs bin=\"sh\" for=\"the cells\" />\n",
        "echo hello",
    )) else {
        return;
    };
    assert!(result.ok, "a satisfied document failed: {}", result.output);
    let woven = result.woven.expect("the document wove");
    assert!(woven.contains("hello"), "{woven}");
}

#[test]
fn the_check_never_appears_in_the_document() {
    // The probe is our bookkeeping. A woven document listing `command -v sh`
    // beside the author's own commands would be a page about our
    // implementation in the middle of somebody else's work.
    let Some(result) = run(&document("  <hick:needs bin=\"sh\" />\n", "echo hello")) else {
        return;
    };
    let woven = result.woven.expect("the document wove");
    assert!(
        !woven.contains("command -v"),
        "the probe leaked into the document:\n{woven}"
    );
    assert!(
        !woven.contains("needs"),
        "the declaration was woven as content:\n{woven}"
    );
}

#[test]
fn a_document_that_declares_nothing_still_runs() {
    // `needs` is optional, and adding it must not have made every existing
    // document without one suspect.
    let Some(result) = run(&document("", "echo hello")) else {
        return;
    };
    assert!(result.ok, "{}", result.output);
}
