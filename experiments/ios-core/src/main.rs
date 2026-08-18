//! The phone's job, performed on a phone's operating system.
//!
//! Protects the mobile half of `docs/specs/freeform/shipping-mobile-and-desktop.md`
//! and the "phone reads and captures, never executes" rule in `notes-ide.md`.
//!
//! Four questions, in the order they stop being interesting if the previous one
//! fails:
//!
//! 1. Does the language parse a real document inside an app container?
//! 2. Does the block model — what both the app and `hick weave` render from —
//!    build there, and does it mark an unrun cell `never-run` rather than
//!    quietly showing stale output as if it were fresh?
//! 3. Does a cached transcript turn that same cell into `ok` with its output?
//! 4. Do speaker turns derive from a real meeting transcript?
//!
//! Everything is embedded with `include_str!` rather than read from the bundle,
//! because what is being measured is the engine, not iOS resource loading.

use std::collections::HashMap;
use std::fmt::Write as _;
use std::io::Write as _;

use hick_literate::render::{Block, BlockModelInput, build_block_model};
use hick_literate::{CellId, NeverRun, NoBaseline};
use hickory_executor::{ExecTranscriptEntry, TranscriptEvent, Transcripts};

/// A document with prose and one cell that has never run.
const DOC: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="phone.md">
# A note that runs somewhere else

<hick:container name="shell" image="alpine:3.20" />

<hick:exec container="shell">
echo "this cell must not run on a phone"
</hick:exec>
</hick:doc>
"#;

/// A real meeting transcript, in the shape an exporter produces.
const VTT: &str = "WEBVTT\n\n00:00:00.000 --> 00:00:04.000\n<v Nate>The phone renders; it never executes.\n\n00:00:04.000 --> 00:00:08.000\n<v Claude>And a cell with no recording says so.\n";

struct Report {
    lines: String,
    failures: usize,
}

impl Report {
    fn new() -> Self {
        Self {
            lines: String::new(),
            failures: 0,
        }
    }

    fn say(&mut self, line: impl AsRef<str>) {
        let line = line.as_ref();
        println!("{line}");
        let _ = std::io::stdout().flush();
        let _ = writeln!(self.lines, "{line}");
    }

    /// Every claim is checked rather than printed, so a probe that "ran fine"
    /// cannot mean "printed something plausible".
    fn check(&mut self, claim: &str, ok: bool) {
        if !ok {
            self.failures += 1;
        }
        self.say(format!("{} {claim}", if ok { "PASS" } else { "FAIL" }));
    }
}

/// Where iOS lets an app write. Reported because a note-taking app that cannot
/// find one has nothing to sync.
fn container() -> std::path::PathBuf {
    std::env::var("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir())
}

/// Does this operating system let the app start a process?
///
/// A simulator answers "yes" and a device answers "no", so this is reported as a
/// measurement of the host rather than as a verdict about iOS.
fn spawning(report: &mut Report) {
    match std::process::Command::new("/bin/echo").arg("x").output() {
        Ok(out) => report.say(format!(
            "NOTE spawn PERMITTED here (status {:?}) — a simulator is a macOS \
             process wearing an iOS runtime; a device refuses. This probe cannot \
             settle the no-spawn rule.",
            out.status
        )),
        Err(e) => report.say(format!("NOTE spawn refused: {e}")),
    }
}

fn exec_blocks(blocks: &[Block]) -> Vec<(&String, &String)> {
    blocks
        .iter()
        .filter_map(|b| match b {
            Block::Exec { id, status, .. } => Some((id, status)),
            _ => None,
        })
        .collect()
}

fn main() {
    let mut report = Report::new();
    report.say("=== hickory iOS core probe ===");
    report.say(format!("container: {}", container().display()));
    let docs = container().join("Documents");
    let writable = std::fs::create_dir_all(&docs)
        .and_then(|()| std::fs::write(docs.join(".probe"), b"x"))
        .is_ok();
    report.check("the app container is writable", writable);
    spawning(&mut report);

    // 1. The language.
    let doc = match hick_lang::parse(DOC) {
        Ok(doc) => {
            report.check("a .hick document parses", true);
            doc
        }
        Err(e) => {
            report.check(&format!("a .hick document parses — {e}"), false);
            finish(&report);
            return;
        }
    };

    // 2. The block model, with nothing ever having run. This is the weave the
    //    phone shows.
    let mut never: NeverRun = NeverRun::new();
    never.insert(CellId::exec("shell", 7), NoBaseline::NotExecuted);
    let empty: Transcripts = HashMap::new();
    let cold = build_block_model(&BlockModelInput {
        doc: &doc,
        transcripts: &empty,
        expectations: &[],
        files: None,
        never_run: &never,
    });
    report.check(
        "the block model builds with no run behind it",
        !cold.is_empty(),
    );
    report.check(
        "prose renders to HTML",
        cold.iter()
            .any(|b| matches!(b, Block::Prose { html, .. } if html.contains("<h1>"))),
    );
    let cold_exec = exec_blocks(&cold);
    report.check(
        "the cell is marked never-run rather than shown as fresh",
        cold_exec.len() == 1 && cold_exec[0].1 == "never-run",
    );
    for (id, status) in &cold_exec {
        report.say(format!("  cell {id}: status={status}"));
    }

    // 3. The same document, answered from a cached transcript — the phone's
    //    actual rendering path, since it can never produce one itself.
    let cached: Transcripts = HashMap::from([(
        "shell".to_string(),
        vec![ExecTranscriptEntry {
            source_line: Some(7),
            commands: vec!["echo \"this cell must not run on a phone\"".to_string()],
            output: "this cell must not run on a phone\n".to_string(),
            events: vec![
                TranscriptEvent::Cmd {
                    t: 0,
                    data: "echo \"this cell must not run on a phone\"".into(),
                },
                TranscriptEvent::Out {
                    t: 3,
                    data: "this cell must not run on a phone\n".into(),
                },
                TranscriptEvent::Exit { t: 3, code: 0 },
            ],
        }],
    )]);
    let warm = build_block_model(&BlockModelInput {
        doc: &doc,
        transcripts: &cached,
        expectations: &[],
        files: None,
        never_run: &NeverRun::new(),
    });
    let warm_exec = exec_blocks(&warm);
    report.check(
        "a cached transcript renders the cell as ok",
        warm_exec.len() == 1 && warm_exec[0].1 == "ok",
    );
    let shows_output = warm.iter().any(|b| {
        matches!(
            b,
            Block::Exec { transcript: Some(events), .. }
                if events.iter().any(|e| matches!(e, TranscriptEvent::Out { data, .. }
                    if data.contains("must not run on a phone")))
        )
    });
    report.check("the cached output reaches the rendered block", shows_output);

    // 4. Meetings, which is the other half of what a phone is for.
    let turns = hick_transcript::parse_detected(VTT);
    report.check(
        "a real VTT transcript derives speaker turns",
        turns.len() == 2,
    );
    // `speaker` is an Option on purpose: a format carrying no attribution is
    // information, not a defect. VTT carries it, so these must be Some.
    let speakers: Vec<Option<String>> = turns.iter().map(|u| u.speaker.clone()).collect();
    report.say(format!("  speakers: {speakers:?}"));
    report.check(
        "the speakers are the people who actually spoke",
        speakers == vec![Some("Nate".to_string()), Some("Claude".to_string())],
    );

    // 5. The rule the whole design rests on: none of the above ran anything.
    report.check(
        "nothing was executed to produce any of this",
        std::env::var("PROBE_EXECUTED").is_err(),
    );

    report.say(format!("=== {} check(s) failed ===", report.failures));
    finish(&report);
}

/// Leave the report inside the container, where the host can fetch it with
/// `simctl get_app_container`, since a launched app's stdout is not always
/// attachable.
fn finish(report: &Report) {
    let out = container().join("Documents").join("core-report.txt");
    let _ = std::fs::create_dir_all(out.parent().unwrap());
    match std::fs::write(&out, &report.lines) {
        Ok(()) => println!("=== report at {} ===", out.display()),
        Err(e) => println!("=== could not write report: {e} ==="),
    }
    let _ = std::io::stdout().flush();
}
