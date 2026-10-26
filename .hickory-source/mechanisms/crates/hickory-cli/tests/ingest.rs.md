<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/tests/ingest.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-a492c9d4e3cc"></a>
## `crates/hickory-cli/tests/ingest.rs`

Related contracts: [Ingest Keeps The Original Bytes, Calls No Model, And Never Deletes Your File](../../../../guarantees/authoring/ingest-keeps-the-original-bytes/contract.md), [One Verb Brings Bytes Into A Document](../../../../guarantees/authoring/one-verb-brings-bytes-into-a-document/contract.md), [Typed Text Is Prose, Not A Transcript](../../../../guarantees/authoring/typed-text-is-prose-not-a-transcript/contract.md).

[Read the test `hick` beside its requirement](../../../../guarantees/authoring/ingest-keeps-the-original-bytes/contract.md#f-a492c9d4e3cc-hick).
<a id="f-a492c9d4e3cc-VTT"></a>
### `VTT`

<storycode:copy id="f-a492c9d4e3cc-VTT">
const VTT: &str = "WEBVTT\n\n00:14:03.000 --> 00:14:11.000\n<v Sam>We can't ship until the index lands.\n\n00:14:19.000 --> 00:14:24.000\n<v Nate>Agreed. What's the write volume?\n";

</storycode:copy>
<a id="f-a492c9d4e3cc-folder_with"></a>
### `folder_with`

A notes folder with `file` sitting in its inbox.

<storycode:copy id="f-a492c9d4e3cc-folder_with">
/// A notes folder with `file` sitting in its inbox.
fn folder_with(name: &str, contents: &[u8]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let inbox = dir.path().join("inbox");
    std::fs::create_dir_all(&inbox).unwrap();
    std::fs::write(inbox.join(name), contents).unwrap();
    dir
}

</storycode:copy>
<a id="f-a492c9d4e3cc-drain"></a>
### `drain`

<storycode:copy id="f-a492c9d4e3cc-drain">
fn drain(root: &Path) -> Vec<Outcome> {
    ingest_inbox(root, &InboxConfig::default()).unwrap()
}

</storycode:copy>
<a id="f-a492c9d4e3cc-only_note"></a>
### `only_note`

<storycode:copy id="f-a492c9d4e3cc-only_note">
fn only_note(root: &Path) -> String {
    let note = std::fs::read_dir(root)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|e| e == "md"))
        .expect("a note was written");
    std::fs::read_to_string(note).unwrap()
}

</storycode:copy>
<a id="f-a492c9d4e3cc-a_transcript_becomes_a_note_holding_the_original_bytes"></a>
### `a_transcript_becomes_a_note_holding_the_original_bytes`

<storycode:copy id="f-a492c9d4e3cc-a_transcript_becomes_a_note_holding_the_original_bytes">
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

</storycode:copy>
<a id="f-a492c9d4e3cc-attendees_are_derived_from_who_actually_spoke"></a>
### `attendees_are_derived_from_who_actually_spoke`

<storycode:copy id="f-a492c9d4e3cc-attendees_are_derived_from_who_actually_spoke">
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

</storycode:copy>
<a id="f-a492c9d4e3cc-ingest_never_calls_a_model"></a>
### `ingest_never_calls_a_model`

<storycode:copy id="f-a492c9d4e3cc-ingest_never_calls_a_model">
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
        .find(|p| p.extension().is_some_and(|e| e == "md"))
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

</storycode:copy>
<a id="f-a492c9d4e3cc-the_same_transcript_twice_produces_one_note"></a>
### `the_same_transcript_twice_produces_one_note`

<storycode:copy id="f-a492c9d4e3cc-the_same_transcript_twice_produces_one_note">
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
        .filter(|e| e.path().extension().is_some_and(|x| x == "md"))
        .count();
    assert_eq!(notes, 1, "a second note was written");
}

</storycode:copy>
<a id="f-a492c9d4e3cc-reimporting_a_legacy_note_preserves_it_and_returns_a_discoverable_markdown_copy"></a>
### `reimporting_a_legacy_note_preserves_it_and_returns_a_discoverable_markdown_copy`

<storycode:copy id="f-a492c9d4e3cc-reimporting_a_legacy_note_preserves_it_and_returns_a_discoverable_markdown_copy">
#[test]
fn reimporting_a_legacy_note_preserves_it_and_returns_a_discoverable_markdown_copy() {
    let dir = folder_with("sync.vtt", VTT.as_bytes());
    let first = drain(dir.path());
    let Outcome::Ingested { note, .. } = &first[0] else {
        panic!("{first:?}")
    };
    let legacy = dir.path().join("legacy.hick");
    std::fs::rename(note, &legacy).unwrap();
    let annotated = format!(
        "{}\nHuman correction stays here.\n",
        std::fs::read_to_string(&legacy).unwrap()
    );
    std::fs::write(&legacy, &annotated).unwrap();
    let collision = dir.path().join("legacy.md");
    std::fs::write(&collision, "Unrelated user note").unwrap();
    let duplicate = dir.path().join("inbox/again.vtt");
    std::fs::write(&duplicate, VTT).unwrap();
    let outcome = ingest_one(&duplicate, dir.path(), &InboxConfig::default()).unwrap();
    let Outcome::AlreadyIngested { note: current, .. } = outcome else {
        panic!("{outcome:?}")
    };
    assert_eq!(current.extension().unwrap(), "md");
    assert_ne!(current, collision);
    assert_eq!(std::fs::read_to_string(&current).unwrap(), annotated);
    assert_eq!(std::fs::read_to_string(&legacy).unwrap(), annotated);
    assert_eq!(
        std::fs::read_to_string(&collision).unwrap(),
        "Unrelated user note"
    );
    assert_eq!(std::fs::read_to_string(&duplicate).unwrap(), VTT);
    assert!(
        hickory_cli::expand_docs(dir.path())
            .unwrap()
            .contains(&current)
    );
    let outcome = ingest_one(&duplicate, dir.path(), &InboxConfig::default()).unwrap();
    let Outcome::AlreadyIngested { note, .. } = outcome else {
        panic!("{outcome:?}")
    };
    assert_eq!(note, current);
}

</storycode:copy>
<a id="f-a492c9d4e3cc-the_source_file_is_moved_never_deleted"></a>
### `the_source_file_is_moved_never_deleted`

<storycode:copy id="f-a492c9d4e3cc-the_source_file_is_moved_never_deleted">
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

</storycode:copy>
<a id="f-a492c9d4e3cc-an_unreadable_file_is_skipped_with_a_reason_and_left_alone"></a>
### `an_unreadable_file_is_skipped_with_a_reason_and_left_alone`

<storycode:copy id="f-a492c9d4e3cc-an_unreadable_file_is_skipped_with_a_reason_and_left_alone">
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

</storycode:copy>
<a id="f-a492c9d4e3cc-one_bad_file_does_not_stop_the_good_ones"></a>
### `one_bad_file_does_not_stop_the_good_ones`

<storycode:copy id="f-a492c9d4e3cc-one_bad_file_does_not_stop_the_good_ones">
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

</storycode:copy>
<a id="f-a492c9d4e3cc-an_unrecognised_format_still_keeps_the_material"></a>
### `an_unrecognised_format_still_keeps_the_material`

<storycode:copy id="f-a492c9d4e3cc-an_unrecognised_format_still_keeps_the_material">
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

</storycode:copy>
<a id="f-a492c9d4e3cc-a_transcript_containing_its_own_close_tag_is_refused_with_a_reason"></a>
### `a_transcript_containing_its_own_close_tag_is_refused_with_a_reason`

<storycode:copy id="f-a492c9d4e3cc-a_transcript_containing_its_own_close_tag_is_refused_with_a_reason">
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

</storycode:copy>
<a id="f-a492c9d4e3cc-ingesting_the_same_bytes_twice_produces_identical_notes"></a>
### `ingesting_the_same_bytes_twice_produces_identical_notes`

<storycode:copy id="f-a492c9d4e3cc-ingesting_the_same_bytes_twice_produces_identical_notes">
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

</storycode:copy>
<a id="f-a492c9d4e3cc-a_part_file_is_passed_over_in_silence"></a>
### `a_part_file_is_passed_over_in_silence`

<storycode:copy id="f-a492c9d4e3cc-a_part_file_is_passed_over_in_silence">
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

</storycode:copy>
<a id="f-a492c9d4e3cc-every_partial_naming_convention_is_recognised"></a>
### `every_partial_naming_convention_is_recognised`

<storycode:copy id="f-a492c9d4e3cc-every_partial_naming_convention_is_recognised">
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

</storycode:copy>
<a id="f-a492c9d4e3cc-a_google_drive_shortcut_says_how_to_export_it"></a>
### `a_google_drive_shortcut_says_how_to_export_it`

<storycode:copy id="f-a492c9d4e3cc-a_google_drive_shortcut_says_how_to_export_it">
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

</storycode:copy>
<a id="f-a492c9d4e3cc-a_google_meet_caption_export_becomes_a_note"></a>
### `a_google_meet_caption_export_becomes_a_note`

<storycode:copy id="f-a492c9d4e3cc-a_google_meet_caption_export_becomes_a_note">
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

</storycode:copy>
<a id="f-a492c9d4e3cc-a_missing_inbox_is_not_an_error"></a>
### `a_missing_inbox_is_not_an_error`

<storycode:copy id="f-a492c9d4e3cc-a_missing_inbox_is_not_an_error">
#[test]
fn a_missing_inbox_is_not_an_error() {
    // Most notes folders will never have one, and `hick up` calls this every
    // cycle.
    let dir = tempfile::tempdir().unwrap();
    assert!(drain(dir.path()).is_empty());
}

</storycode:copy>
<a id="f-a492c9d4e3cc-one_named_file_can_be_ingested_directly"></a>
### `one_named_file_can_be_ingested_directly`

<storycode:copy id="f-a492c9d4e3cc-one_named_file_can_be_ingested_directly">
#[test]
fn one_named_file_can_be_ingested_directly() {
    let dir = folder_with("sync.vtt", VTT.as_bytes());
    let source = dir.path().join("inbox").join("sync.vtt");
    let outcome = ingest_one(&source, dir.path(), &InboxConfig::default()).unwrap();
    assert!(matches!(outcome, Outcome::Ingested { .. }), "{outcome:?}");
}

</storycode:copy>
<a id="f-a492c9d4e3cc-the_inbox_defaults_to_working_with_nothing_set"></a>
### `the_inbox_defaults_to_working_with_nothing_set`

<storycode:copy id="f-a492c9d4e3cc-the_inbox_defaults_to_working_with_nothing_set">
// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

#[test]
fn the_inbox_defaults_to_working_with_nothing_set() {
    let config = InboxConfig::from_lookup(|_| None).unwrap();
    assert_eq!(config.inbox(Path::new("/notes")), Path::new("/notes/inbox"));
}

</storycode:copy>
<a id="f-a492c9d4e3cc-a_bad_inbox_setting_fails_with_a_message_that_names_the_variable"></a>
### `a_bad_inbox_setting_fails_with_a_message_that_names_the_variable`

<storycode:copy id="f-a492c9d4e3cc-a_bad_inbox_setting_fails_with_a_message_that_names_the_variable">
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

</storycode:copy>
[Read the test `an_inbox_starting_at_a_root_is_refused_on_every_platform` beside its requirement](../../../../guarantees/authoring/ingest-keeps-the-original-bytes/contract.md#f-a492c9d4e3cc-an_inbox_starting_at_a_root_is_refused_on_every_platform).
<a id="f-a492c9d4e3cc-the_windows_spellings_of_outside_the_folder_are_refused"></a>
### `the_windows_spellings_of_outside_the_folder_are_refused`

The spellings only Windows has. Asserted where they mean something rather
than on a platform that reads `C:\tmp` as an ordinary directory name.

<storycode:copy id="f-a492c9d4e3cc-the_windows_spellings_of_outside_the_folder_are_refused">
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

</storycode:copy>
<a id="f-a492c9d4e3cc-a_name_inside_the_folder_is_still_accepted"></a>
### `a_name_inside_the_folder_is_still_accepted`

<storycode:copy id="f-a492c9d4e3cc-a_name_inside_the_folder_is_still_accepted">
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

</storycode:copy>
<a id="f-a492c9d4e3cc-the_ingest_command_reports_every_outcome"></a>
### `the_ingest_command_reports_every_outcome`

<storycode:copy id="f-a492c9d4e3cc-the_ingest_command_reports_every_outcome">
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

</storycode:copy>
<a id="f-a492c9d4e3cc-a_download_in_progress_is_reported_as_such_not_as_an_empty_inbox"></a>
### `a_download_in_progress_is_reported_as_such_not_as_an_empty_inbox`

<storycode:copy id="f-a492c9d4e3cc-a_download_in_progress_is_reported_as_such_not_as_an_empty_inbox">
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

</storycode:copy>
<a id="f-a492c9d4e3cc-an_empty_inbox_says_what_to_put_in_it"></a>
### `an_empty_inbox_says_what_to_put_in_it`

<storycode:copy id="f-a492c9d4e3cc-an_empty_inbox_says_what_to_put_in_it">
#[test]
fn an_empty_inbox_says_what_to_put_in_it() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("inbox")).unwrap();
    let out = hick().arg("ingest").arg(dir.path()).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("nothing to ingest"), "stdout: {stdout}");
    assert!(stdout.contains(".vtt"), "no example given: {stdout}");
}

</storycode:copy>
<a id="f-a492c9d4e3cc-scratchpad_text_becomes_prose_not_a_transcript"></a>
### `scratchpad_text_becomes_prose_not_a_transcript`

<storycode:copy id="f-a492c9d4e3cc-scratchpad_text_becomes_prose_not_a_transcript">
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

</storycode:copy>
<a id="f-a492c9d4e3cc-a_leading_heading_becomes_the_title_and_is_not_repeated"></a>
### `a_leading_heading_becomes_the_title_and_is_not_repeated`

<storycode:copy id="f-a492c9d4e3cc-a_leading_heading_becomes_the_title_and_is_not_repeated">
#[test]
fn a_leading_heading_becomes_the_title_and_is_not_repeated() {
    let note = note_from_scratchpad("# Standup\n\nAll green.", Some("2026-08-18")).unwrap();
    assert_eq!(note.matches("# Standup").count(), 1, "in:\n{note}");
    assert!(note.contains("All green."), "in:\n{note}");
}

</storycode:copy>
<a id="f-a492c9d4e3cc-a_first_line_without_a_heading_still_titles_the_note"></a>
### `a_first_line_without_a_heading_still_titles_the_note`

<storycode:copy id="f-a492c9d4e3cc-a_first_line_without_a_heading_still_titles_the_note">
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

</storycode:copy>
<a id="f-a492c9d4e3cc-an_empty_scratchpad_is_refused_with_a_next_step"></a>
### `an_empty_scratchpad_is_refused_with_a_next_step`

<storycode:copy id="f-a492c9d4e3cc-an_empty_scratchpad_is_refused_with_a_next_step">
#[test]
fn an_empty_scratchpad_is_refused_with_a_next_step() {
    let err = note_from_scratchpad("   \n ", None)
        .unwrap_err()
        .to_string();
    assert!(err.contains("nothing in the scratchpad"), "{err}");
    assert!(err.contains("Next step"), "{err}");
}

</storycode:copy>
<a id="f-a492c9d4e3cc-scratchpad_prose_containing_hick_markup_is_refused"></a>
### `scratchpad_prose_containing_hick_markup_is_refused`

<storycode:copy id="f-a492c9d4e3cc-scratchpad_prose_containing_hick_markup_is_refused">
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

</storycode:copy>
<a id="f-a492c9d4e3cc-saving_the_scratchpad_writes_a_note_that_parses_and_weaves"></a>
### `saving_the_scratchpad_writes_a_note_that_parses_and_weaves`

<storycode:copy id="f-a492c9d4e3cc-saving_the_scratchpad_writes_a_note_that_parses_and_weaves">
#[test]
fn saving_the_scratchpad_writes_a_note_that_parses_and_weaves() {
    let dir = tempfile::tempdir().unwrap();
    let path = save_scratchpad("# Standup\n\nAll green.", dir.path()).unwrap();
    assert!(path.exists());
    assert_eq!(path.extension().unwrap(), "md");
    let before = std::fs::read_to_string(&path).unwrap();

    let out = hick().arg("weave").arg(&path).output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let source = std::fs::read_to_string(&path).unwrap();
    assert_eq!(
        source, before,
        "Weaving a prose note must not alter its owned Markdown"
    );
    assert!(source.contains("All green."), "in:\n{source}");
}

</storycode:copy>
<a id="f-a492c9d4e3cc-two_scratchpad_notes_with_the_same_title_do_not_collide"></a>
### `two_scratchpad_notes_with_the_same_title_do_not_collide`

<storycode:copy id="f-a492c9d4e3cc-two_scratchpad_notes_with_the_same_title_do_not_collide">
#[test]
fn two_scratchpad_notes_with_the_same_title_do_not_collide() {
    let dir = tempfile::tempdir().unwrap();
    let first = save_scratchpad("# Standup\n\nOne.", dir.path()).unwrap();
    let second = save_scratchpad("# Standup\n\nTwo.", dir.path()).unwrap();
    assert_ne!(first, second);
    assert!(std::fs::read_to_string(&first).unwrap().contains("One."));
    assert!(std::fs::read_to_string(&second).unwrap().contains("Two."));
}

</storycode:copy>
[Read the test `a_macos_download_records_where_it_came_from` beside its requirement](../../../../guarantees/authoring/ingest-keeps-the-original-bytes/contract.md#f-a492c9d4e3cc-a_macos_download_records_where_it_came_from).
<a id="f-a492c9d4e3cc-agent_assisted_scratchpad_keeps_prose_with_explicit_authorship"></a>
### `agent_assisted_scratchpad_keeps_prose_with_explicit_authorship`

<storycode:copy id="f-a492c9d4e3cc-agent_assisted_scratchpad_keeps_prose_with_explicit_authorship">
#[test]
fn agent_assisted_scratchpad_keeps_prose_with_explicit_authorship() {
    let note = hickory_cli::ingest::note_from_scratchpad_with_attribution(
        "# Draft\n\nAgent-assisted words.",
        Some("2026-10-20"),
        true,
    )
    .unwrap();
    assert!(note.contains("source: scratchpad\nauthorship: agent-assisted\n"));
    assert!(note.contains("# Draft\n\nAgent-assisted words."));
    assert!(!note.contains("hick:transcript"));
    let human = note_from_scratchpad("# Draft\n\nHuman words.", None).unwrap();
    assert!(!human.contains("authorship: agent-assisted"));
    let dir = tempfile::tempdir().unwrap();
    let path = hickory_cli::ingest::save_scratchpad_with_attribution(
        "# Draft\n\nAgent-assisted words.",
        dir.path(),
        true,
    )
    .unwrap();
    let saved = std::fs::read_to_string(&path).unwrap();
    assert!(saved.contains("authorship: agent-assisted"));
    let woven = hick().arg("weave").arg(&path).output().unwrap();
    assert!(
        woven.status.success(),
        "{}",
        String::from_utf8_lossy(&woven.stderr)
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), saved);
    assert!(
        hickory_cli::ingest::note_from_scratchpad_with_attribution("<hick:run>", None, true)
            .is_err()
    );
}

</storycode:copy>
<a id="f-a492c9d4e3cc-concurrent_scratchpad_saves_never_overwrite_another_note"></a>
### `concurrent_scratchpad_saves_never_overwrite_another_note`

<storycode:copy id="f-a492c9d4e3cc-concurrent_scratchpad_saves_never_overwrite_another_note">
#[test]
fn concurrent_scratchpad_saves_never_overwrite_another_note() {
    let dir = tempfile::tempdir().unwrap();
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(8));
    let threads: Vec<_> = (0..8)
        .map(|index| {
            let root = dir.path().to_path_buf();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                let text = format!("# Same title\n\nUnique payload {index}.");
                let path =
                    hickory_cli::ingest::save_scratchpad_with_attribution(&text, &root, true)
                        .unwrap();
                assert!(
                    std::fs::read_to_string(&path)
                        .unwrap()
                        .contains(&format!("Unique payload {index}."))
                );
                path
            })
        })
        .collect();
    let paths: std::collections::HashSet<_> = threads
        .into_iter()
        .map(|thread| thread.join().unwrap())
        .collect();
    assert_eq!(paths.len(), 8);
}
</storycode:copy>

### Assemble `crates/hickory-cli/tests/ingest.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/ingest.rs"><storycode:paste select="#f-a492c9d4e3cc-hick"/><storycode:paste select="#f-a492c9d4e3cc-VTT"/><storycode:paste select="#f-a492c9d4e3cc-folder_with"/><storycode:paste select="#f-a492c9d4e3cc-drain"/><storycode:paste select="#f-a492c9d4e3cc-only_note"/><storycode:paste select="#f-a492c9d4e3cc-a_transcript_becomes_a_note_holding_the_original_bytes"/><storycode:paste select="#f-a492c9d4e3cc-attendees_are_derived_from_who_actually_spoke"/><storycode:paste select="#f-a492c9d4e3cc-ingest_never_calls_a_model"/><storycode:paste select="#f-a492c9d4e3cc-the_same_transcript_twice_produces_one_note"/><storycode:paste select="#f-a492c9d4e3cc-reimporting_a_legacy_note_preserves_it_and_returns_a_discoverable_markdown_copy"/><storycode:paste select="#f-a492c9d4e3cc-the_source_file_is_moved_never_deleted"/><storycode:paste select="#f-a492c9d4e3cc-an_unreadable_file_is_skipped_with_a_reason_and_left_alone"/><storycode:paste select="#f-a492c9d4e3cc-one_bad_file_does_not_stop_the_good_ones"/><storycode:paste select="#f-a492c9d4e3cc-an_unrecognised_format_still_keeps_the_material"/><storycode:paste select="#f-a492c9d4e3cc-a_transcript_containing_its_own_close_tag_is_refused_with_a_reason"/><storycode:paste select="#f-a492c9d4e3cc-ingesting_the_same_bytes_twice_produces_identical_notes"/><storycode:paste select="#f-a492c9d4e3cc-a_part_file_is_passed_over_in_silence"/><storycode:paste select="#f-a492c9d4e3cc-every_partial_naming_convention_is_recognised"/><storycode:paste select="#f-a492c9d4e3cc-a_google_drive_shortcut_says_how_to_export_it"/><storycode:paste select="#f-a492c9d4e3cc-a_google_meet_caption_export_becomes_a_note"/><storycode:paste select="#f-a492c9d4e3cc-a_missing_inbox_is_not_an_error"/><storycode:paste select="#f-a492c9d4e3cc-one_named_file_can_be_ingested_directly"/><storycode:paste select="#f-a492c9d4e3cc-the_inbox_defaults_to_working_with_nothing_set"/><storycode:paste select="#f-a492c9d4e3cc-a_bad_inbox_setting_fails_with_a_message_that_names_the_variable"/><storycode:paste select="#f-a492c9d4e3cc-an_inbox_starting_at_a_root_is_refused_on_every_platform"/><storycode:paste select="#f-a492c9d4e3cc-the_windows_spellings_of_outside_the_folder_are_refused"/><storycode:paste select="#f-a492c9d4e3cc-a_name_inside_the_folder_is_still_accepted"/><storycode:paste select="#f-a492c9d4e3cc-the_ingest_command_reports_every_outcome"/><storycode:paste select="#f-a492c9d4e3cc-a_download_in_progress_is_reported_as_such_not_as_an_empty_inbox"/><storycode:paste select="#f-a492c9d4e3cc-an_empty_inbox_says_what_to_put_in_it"/><storycode:paste select="#f-a492c9d4e3cc-scratchpad_text_becomes_prose_not_a_transcript"/><storycode:paste select="#f-a492c9d4e3cc-a_leading_heading_becomes_the_title_and_is_not_repeated"/><storycode:paste select="#f-a492c9d4e3cc-a_first_line_without_a_heading_still_titles_the_note"/><storycode:paste select="#f-a492c9d4e3cc-an_empty_scratchpad_is_refused_with_a_next_step"/><storycode:paste select="#f-a492c9d4e3cc-scratchpad_prose_containing_hick_markup_is_refused"/><storycode:paste select="#f-a492c9d4e3cc-saving_the_scratchpad_writes_a_note_that_parses_and_weaves"/><storycode:paste select="#f-a492c9d4e3cc-two_scratchpad_notes_with_the_same_title_do_not_collide"/><storycode:paste select="#f-a492c9d4e3cc-a_macos_download_records_where_it_came_from"/><storycode:paste select="#f-a492c9d4e3cc-agent_assisted_scratchpad_keeps_prose_with_explicit_authorship"/><storycode:paste select="#f-a492c9d4e3cc-concurrent_scratchpad_saves_never_overwrite_another_note"/></storycode:file>

</storycode:doc>
