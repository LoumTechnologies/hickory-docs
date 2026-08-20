//! Ingest: a meeting becomes a note.
//!
//! Protects docs/guarantees/authoring/ingest-keeps-the-original-bytes.md
//!
//! The three properties worth testing are the three promises the spec makes:
//! the user's file is never destroyed, no model is ever called, and dropping
//! the same transcript twice does not produce two notes.

use std::path::Path;
use std::process::Command;

use hickory_cli::ingest::{
    InboxConfig, Outcome, ingest_inbox, ingest_one, note_from_scratchpad, save_scratchpad,
};

fn hick() -> Command {
    Command::new(env!("CARGO_BIN_EXE_hick"))
}

const VTT: &str = "WEBVTT\n\n00:14:03.000 --> 00:14:11.000\n<v Sam>We can't ship until the index lands.\n\n00:14:19.000 --> 00:14:24.000\n<v Nate>Agreed. What's the write volume?\n";

/// A notes folder with `file` sitting in its inbox.
fn folder_with(name: &str, contents: &[u8]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let inbox = dir.path().join("inbox");
    std::fs::create_dir_all(&inbox).unwrap();
    std::fs::write(inbox.join(name), contents).unwrap();
    dir
}

fn drain(root: &Path) -> Vec<Outcome> {
    ingest_inbox(root, &InboxConfig::default()).unwrap()
}

fn only_note(root: &Path) -> String {
    let note = std::fs::read_dir(root)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|e| e == "hick"))
        .expect("a note was written");
    std::fs::read_to_string(note).unwrap()
}

#[test]
fn a_transcript_becomes_a_note_holding_the_original_bytes() {
    let dir = folder_with("sync.vtt", VTT.as_bytes());
    let outcomes = drain(dir.path());
    assert!(
        matches!(outcomes[0], Outcome::Ingested { .. }),
        "{outcomes:?}"
    );

    let note = only_note(dir.path());
    // Byte-for-byte: the raw block is the source of truth, and an exporter's
    // bytes are not ours to normalise.
    assert!(note.contains(VTT), "original bytes missing from:\n{note}");
    assert!(note.contains("<hick:transcript"), "no transcript block");
    assert!(note.contains("format=\"vtt\""), "format not recorded");
}

#[test]
fn attendees_are_derived_from_who_actually_spoke() {
    // Derived, not asserted by whoever ran the command.
    let dir = folder_with("sync.vtt", VTT.as_bytes());
    drain(dir.path());
    assert!(
        only_note(dir.path()).contains("attendees: [Sam, Nate]"),
        "attendees not derived"
    );
}

#[test]
fn ingest_never_calls_a_model() {
    // The summaries are written EMPTY and STALE, so a note that has never been
    // summarized is visibly unsummarized rather than silently blank — and a
    // meeting recorded on a plane still becomes a note on the plane.
    let dir = folder_with("sync.vtt", VTT.as_bytes());
    drain(dir.path());
    let note = only_note(dir.path());
    assert_eq!(note.matches("from=\"\"").count(), 2, "in:\n{note}");

    let doc = std::fs::read_dir(dir.path())
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|e| e == "hick"))
        .unwrap();
    let out = hick().arg("test").arg(&doc).output().unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success(), "a fresh note must read as stale");
    assert!(stderr.contains("STALE"), "stderr: {stderr}");
    assert!(
        stderr.contains("hick refresh"),
        "the fix must name the command: {stderr}"
    );
}

#[test]
fn the_same_transcript_twice_produces_one_note() {
    // The reflex when something looks like it failed is to try again.
    let dir = folder_with("sync.vtt", VTT.as_bytes());
    drain(dir.path());
    std::fs::write(dir.path().join("inbox").join("again.vtt"), VTT).unwrap();

    let outcomes = drain(dir.path());
    assert!(
        matches!(outcomes[0], Outcome::AlreadyIngested { .. }),
        "{outcomes:?}"
    );
    let notes = std::fs::read_dir(dir.path())
        .unwrap()
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "hick"))
        .count();
    assert_eq!(notes, 1, "a second note was written");
}

#[test]
fn the_source_file_is_moved_never_deleted() {
    // The bytes came from outside and we do not own them. A move is reversible
    // by anyone looking at the directory; a delete is a support ticket from
    // someone who dropped in their only copy.
    let dir = folder_with("sync.vtt", VTT.as_bytes());
    let outcomes = drain(dir.path());
    let Outcome::Ingested { moved_to, .. } = &outcomes[0] else {
        panic!("{outcomes:?}");
    };
    assert!(moved_to.exists(), "the source is gone");
    assert_eq!(std::fs::read_to_string(moved_to).unwrap(), VTT);
    assert!(!dir.path().join("inbox").join("sync.vtt").exists());
}

#[test]
fn an_unreadable_file_is_skipped_with_a_reason_and_left_alone() {
    // A file we could not read is a file we must not move: the user still has
    // to be able to find it.
    let dir = folder_with("meeting.m4a", &[0x00, 0xff, 0xfe, 0x00]);
    let outcomes = drain(dir.path());
    let Outcome::Skipped { reason, .. } = &outcomes[0] else {
        panic!("{outcomes:?}");
    };
    assert!(reason.contains("not text"), "reason: {reason}");
    assert!(
        reason.contains("transcribe"),
        "the reason must say what to do: {reason}"
    );
    assert!(dir.path().join("inbox").join("meeting.m4a").exists());
}

#[test]
fn one_bad_file_does_not_stop_the_good_ones() {
    let dir = folder_with("sync.vtt", VTT.as_bytes());
    std::fs::write(
        dir.path().join("inbox").join("broken.m4a"),
        [0x00, 0xff, 0xfe],
    )
    .unwrap();
    let outcomes = drain(dir.path());
    assert_eq!(outcomes.len(), 2);
    assert!(
        outcomes
            .iter()
            .any(|o| matches!(o, Outcome::Ingested { .. })),
        "{outcomes:?}"
    );
}

#[test]
fn an_unrecognised_format_still_keeps_the_material() {
    // Refusing would leave the file in the inbox forever; adopting keeps the
    // material and loses only the structure.
    let prose = "Just some notes I typed.\nNo speakers anywhere.\n";
    let dir = folder_with("thoughts.txt", prose.as_bytes());
    let outcomes = drain(dir.path());
    assert!(
        matches!(outcomes[0], Outcome::Ingested { .. }),
        "{outcomes:?}"
    );

    let note = only_note(dir.path());
    assert!(note.contains(prose), "material lost from:\n{note}");
    assert!(
        note.contains("source-format: unrecognised"),
        "the gap must be recorded, not hidden:\n{note}"
    );
}

#[test]
fn a_transcript_containing_its_own_close_tag_is_refused_with_a_reason() {
    // There is no escaping in this language by design, so such a file cannot
    // be stored verbatim — and saying so is the only honest option.
    let dir = folder_with("evil.txt", b"Sam: look at </hick:transcript> here\n");
    let outcomes = drain(dir.path());
    let Outcome::Skipped { reason, .. } = &outcomes[0] else {
        panic!("{outcomes:?}");
    };
    assert!(reason.contains("no escaping"), "reason: {reason}");
    assert!(dir.path().join("inbox").join("evil.txt").exists());
}

#[test]
fn ingesting_the_same_bytes_twice_produces_identical_notes() {
    // Determinism: there is no clock in ingest, so the note does not depend on
    // when it was made.
    let a = folder_with("sync.vtt", VTT.as_bytes());
    let b = folder_with("sync.vtt", VTT.as_bytes());
    drain(a.path());
    drain(b.path());
    assert_eq!(only_note(a.path()), only_note(b.path()));
}

#[test]
fn a_part_file_is_passed_over_in_silence() {
    // A half-written download is a truncated transcript. It gets its real name
    // when the transfer finishes, so there is nothing to report.
    let dir = folder_with("sync.vtt.crdownload", VTT.as_bytes());
    assert!(drain(dir.path()).is_empty());
    assert!(
        dir.path()
            .join("inbox")
            .join("sync.vtt.crdownload")
            .exists(),
        "an in-flight download must be left alone"
    );
}

#[test]
fn every_partial_naming_convention_is_recognised() {
    for name in [
        "a.vtt.crdownload",
        "a.vtt.part",
        "a.vtt.partial",
        "a.vtt.download",
        "a.vtt.opdownload",
        "a.vtt.filepart",
        "a.vtt.tmp",
        ".a.vtt",
        "~$a.vtt",
        "a.vtt~",
    ] {
        let dir = folder_with(name, VTT.as_bytes());
        assert!(drain(dir.path()).is_empty(), "{name} was not passed over");
    }
}

#[test]
fn a_google_drive_shortcut_says_how_to_export_it() {
    // A `.gdoc` holds a URL; the words are still in the cloud. Ingesting it
    // would produce a note containing a JSON stub, which is worse than useless
    // because it looks like it worked.
    let dir = folder_with(
        "Meet Recording.gdoc",
        br#"{"url": "https://docs.google.com/..."}"#,
    );
    let outcomes = drain(dir.path());
    let Outcome::Skipped { reason, .. } = &outcomes[0] else {
        panic!("{outcomes:?}");
    };
    assert!(reason.contains("Drive shortcut"), "reason: {reason}");
    assert!(
        reason.contains("Download"),
        "no export path given: {reason}"
    );
    assert!(
        dir.path()
            .join("inbox")
            .join("Meet Recording.gdoc")
            .exists(),
        "a stub we could not read must be left where the user can find it"
    );
}

#[test]
fn a_google_meet_caption_export_becomes_a_note() {
    // Meet exports captions as SubViewer, which neither the WebVTT nor the SRT
    // reader recognises.
    let sbv = "0:00:03.000,0:00:07.000\nAda: The migration finished overnight.\n\n0:00:08.000,0:00:12.000\nSam: Good, I'll close the ticket.\n";
    let dir = folder_with("meet-captions.sbv", sbv.as_bytes());
    let outcomes = drain(dir.path());
    assert!(
        matches!(outcomes[0], Outcome::Ingested { .. }),
        "{outcomes:?}"
    );
    let note = only_note(dir.path());
    assert!(note.contains("source-format: sbv"), "in:\n{note}");
    assert!(note.contains("attendees: [Ada, Sam]"), "in:\n{note}");
}

#[test]
fn a_missing_inbox_is_not_an_error() {
    // Most notes folders will never have one, and `hick up` calls this every
    // cycle.
    let dir = tempfile::tempdir().unwrap();
    assert!(drain(dir.path()).is_empty());
}

#[test]
fn one_named_file_can_be_ingested_directly() {
    let dir = folder_with("sync.vtt", VTT.as_bytes());
    let source = dir.path().join("inbox").join("sync.vtt");
    let outcome = ingest_one(&source, dir.path(), &InboxConfig::default()).unwrap();
    assert!(matches!(outcome, Outcome::Ingested { .. }), "{outcome:?}");
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

#[test]
fn the_inbox_defaults_to_working_with_nothing_set() {
    let config = InboxConfig::from_lookup(|_| None).unwrap();
    assert_eq!(config.inbox(Path::new("/notes")), Path::new("/notes/inbox"));
}

#[test]
fn a_bad_inbox_setting_fails_with_a_message_that_names_the_variable() {
    // A bad value here fails on somebody else's laptop, where nobody can debug
    // it for them.
    for bad in ["", "   ", "/tmp/elsewhere", "../outside"] {
        let err = InboxConfig::from_lookup(|_| Some(bad.to_string()))
            .expect_err("{bad} should be refused")
            .to_string();
        assert!(err.contains("HICKORY_INBOX"), "{bad}: {err}");
        assert!(err.contains("Next step"), "{bad}: {err}");
    }
}

/// Protects `docs/guarantees/authoring/ingest-keeps-the-original-bytes.md`
#[test]
fn an_inbox_starting_at_a_root_is_refused_on_every_platform() {
    // `is_absolute()` is the obvious guard and it was the wrong one. On
    // Windows it is FALSE for `/tmp/elsewhere` — rooted, but naming no drive,
    // which Windows calls relative — while `join` still resolves it to
    // `C:\tmp\elsewhere`. Ingest MOVES files after reading them, so a guard
    // that fails open relocates somebody's recording somewhere they never
    // named. The value above was already in the list below when this was
    // found: the test was right and had simply never run on Windows.
    for bad in ["/tmp/elsewhere", "/etc/hickory", "/"] {
        let err = InboxConfig::from_lookup(|_| Some(bad.to_string()))
            .expect_err("a rooted inbox must be refused")
            .to_string();
        assert!(err.contains("HICKORY_INBOX"), "{bad}: {err}");
        assert!(err.contains("Next step"), "{bad}: {err}");
    }
}

/// The spellings only Windows has. Asserted where they mean something rather
/// than on a platform that reads `C:\tmp` as an ordinary directory name.
#[cfg(windows)]
#[test]
fn the_windows_spellings_of_outside_the_folder_are_refused() {
    // `C:tmp` is the sharp one: drive-relative, so it has no root at all and
    // `has_root()` would let it through, but it still resolves against another
    // drive's current directory rather than inside the notes folder.
    for bad in [r"C:\tmp", r"\tmp", r"C:tmp", r"\\server\share"] {
        let err = InboxConfig::from_lookup(|_| Some(bad.to_string()))
            .expect_err("a rooted or drive-qualified inbox must be refused")
            .to_string();
        assert!(err.contains("HICKORY_INBOX"), "{bad}: {err}");
    }
}

#[test]
fn a_name_inside_the_folder_is_still_accepted() {
    // The guard must not be so broad that the feature stops working.
    for good in ["inbox", "meetings/inbox", "a/b/c"] {
        let config = InboxConfig::from_lookup(|_| Some(good.to_string()))
            .unwrap_or_else(|e| panic!("{good} should be accepted: {e}"));
        assert!(
            config
                .inbox(Path::new("/notes"))
                .starts_with(Path::new("/notes")),
            "{good} escaped the notes folder"
        );
    }
}

#[test]
fn the_ingest_command_reports_every_outcome() {
    let dir = folder_with("sync.vtt", VTT.as_bytes());
    let out = hick().arg("ingest").arg(dir.path()).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("ingested"), "stdout: {stdout}");
    assert!(
        stdout.contains("hick refresh"),
        "the next step must be named: {stdout}"
    );
}

#[test]
fn a_download_in_progress_is_reported_as_such_not_as_an_empty_inbox() {
    // "nothing to ingest" while a download is visibly in progress reads as a
    // broken tool.
    let dir = folder_with("sync.vtt.crdownload", VTT.as_bytes());
    let out = hick().arg("ingest").arg(dir.path()).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("still downloading"), "stdout: {stdout}");
    assert!(stdout.contains("Next step"), "stdout: {stdout}");
}

#[test]
fn an_empty_inbox_says_what_to_put_in_it() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("inbox")).unwrap();
    let out = hick().arg("ingest").arg(dir.path()).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("nothing to ingest"), "stdout: {stdout}");
    assert!(stdout.contains(".vtt"), "no example given: {stdout}");
}

// ---------------------------------------------------------------------------
// Scratchpad
// ---------------------------------------------------------------------------

#[test]
fn scratchpad_text_becomes_prose_not_a_transcript() {
    // Typed text is the strongest material on the attributable axis: a named
    // human wrote it, here, and git will say so. Wrapping it as bytes another
    // tool produced would throw that away.
    let note = note_from_scratchpad("Remember to close the ticket.", Some("2026-08-18")).unwrap();
    assert!(!note.contains("hick:transcript"), "in:\n{note}");
    assert!(note.contains("source: scratchpad"), "in:\n{note}");
    assert!(
        note.contains("Remember to close the ticket."),
        "in:\n{note}"
    );
    // Nothing was summarized, so there is nothing to be stale.
    assert!(!note.contains("hick:transform"), "in:\n{note}");
}

#[test]
fn a_leading_heading_becomes_the_title_and_is_not_repeated() {
    let note = note_from_scratchpad("# Standup\n\nAll green.", Some("2026-08-18")).unwrap();
    assert_eq!(note.matches("# Standup").count(), 1, "in:\n{note}");
    assert!(note.contains("All green."), "in:\n{note}");
}

#[test]
fn a_first_line_without_a_heading_still_titles_the_note() {
    let note = note_from_scratchpad("Close the ticket\nand tell Sam", Some("2026-08-18")).unwrap();
    assert!(note.contains("# Close the ticket"), "in:\n{note}");
    // The line is a title AND still part of what was written.
    assert!(
        note.contains("Close the ticket\nand tell Sam"),
        "in:\n{note}"
    );
}

#[test]
fn an_empty_scratchpad_is_refused_with_a_next_step() {
    let err = note_from_scratchpad("   \n ", None)
        .unwrap_err()
        .to_string();
    assert!(err.contains("nothing in the scratchpad"), "{err}");
    assert!(err.contains("Next step"), "{err}");
}

#[test]
fn scratchpad_prose_containing_hick_markup_is_refused() {
    // Prose becomes document content, and there is no escaping in this
    // language by design.
    let err = note_from_scratchpad("I used <hick:copy id=\"x\"> today", None)
        .unwrap_err()
        .to_string();
    assert!(err.contains("no escaping"), "{err}");
    assert!(err.contains("Next step"), "{err}");
}

#[test]
fn saving_the_scratchpad_writes_a_note_that_parses_and_weaves() {
    let dir = tempfile::tempdir().unwrap();
    let path = save_scratchpad("# Standup\n\nAll green.", dir.path()).unwrap();
    assert!(path.exists());

    let out = hick().arg("weave").arg(&path).output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let woven = std::fs::read_to_string(path.with_extension("md")).unwrap();
    assert!(woven.contains("All green."), "in:\n{woven}");
}

#[test]
fn two_scratchpad_notes_with_the_same_title_do_not_collide() {
    let dir = tempfile::tempdir().unwrap();
    let first = save_scratchpad("# Standup\n\nOne.", dir.path()).unwrap();
    let second = save_scratchpad("# Standup\n\nTwo.", dir.path()).unwrap();
    assert_ne!(first, second);
    assert!(std::fs::read_to_string(&first).unwrap().contains("One."));
    assert!(std::fs::read_to_string(&second).unwrap().contains("Two."));
}

/// The macOS download-origin path, end to end, against the real extended
/// attribute rather than the parser alone.
///
/// Protects docs/guarantees/authoring/ingest-keeps-the-original-bytes.md
///
/// macOS only, because `com.apple.metadata:*` is not a namespace Linux will let
/// anything write — which is exactly why this went unverified until someone ran
/// it on a Mac.
#[cfg(target_os = "macos")]
#[test]
fn a_macos_download_records_where_it_came_from() {
    // Captured with `xattr -px` from a file Chrome downloaded through a link, so
    // the value holds the file URL and then the referring page.
    const WHERE_FROMS: &[u8] =
        include_bytes!("fixtures/wherefroms/safari-linked-with-referrer.bplist");

    let dir = folder_with("linked.vtt", VTT.as_bytes());
    let source = dir.path().join("inbox").join("linked.vtt");
    xattr::set(&source, "com.apple.metadata:kMDItemWhereFroms", WHERE_FROMS)
        .expect("setting kMDItemWhereFroms on a temp file");

    drain(dir.path());
    let note = only_note(dir.path());

    assert!(
        note.contains("source-url: http://127.0.0.1:8791/safari-linked.vtt\n"),
        "the download URL is not recorded exactly:\n{note}"
    );
    // The referring page is not where the bytes came from.
    assert!(
        !note.contains("safari-page.html"),
        "the referrer was recorded instead of the download:\n{note}"
    );
    // The security property, checked on the note itself and not on the parser:
    // a signed export link carries its credential in the query string.
    assert!(
        !note.contains("SAFARI_LINKED_SECRET"),
        "the query string reached the note:\n{note}"
    );
}
