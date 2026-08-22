//! An ingested note is named for the meeting, not for the download: a date in
//! the file name is the meeting's date, the title is the rest, and the
//! transcript's id is the note's own name so two meetings upstream of one
//! document never both answer to `#transcript`.
//! Guarantee: docs/guarantees/authoring/ingest-keeps-the-original-bytes.md

use hickory_cli::ingest::{date_in_name, note_for};
use std::path::Path;

const VTT: &str = "WEBVTT\n\n00:00:01.000 --> 00:00:02.000\n<v Sam>Ship it.\n";

#[test]
fn the_date_in_the_name_wins_and_the_title_is_the_rest() {
    let path = Path::new("inbox/2026-08-20-checkout-latency-sync.vtt");
    assert_eq!(date_in_name(path).as_deref(), Some("2026-08-20"));
    let note = note_for(VTT, path, Some("2026-08-22"), None).unwrap();
    assert!(note.contains("date: 2026-08-20\n"), "{note}");
    assert!(note.contains("# Checkout Latency Sync\n"), "{note}");
    assert!(
        note.contains("<hick:transcript id=\"2026-08-20-checkout-latency-sync\""),
        "{note}"
    );
    assert!(
        note.contains("select=\"#2026-08-20-checkout-latency-sync\""),
        "the summary transforms select the same id: {note}"
    );
}

#[test]
fn a_name_without_a_date_keeps_the_given_date_and_a_plain_title() {
    let path = Path::new("inbox/sync-with-sam.vtt");
    assert_eq!(date_in_name(path), None);
    let note = note_for(VTT, path, Some("2026-08-22"), None).unwrap();
    assert!(note.contains("date: 2026-08-22\n"), "{note}");
    assert!(note.contains("# Sync With Sam\n"), "{note}");
    assert!(
        note.contains("<hick:transcript id=\"2026-08-22-sync-with-sam\""),
        "{note}"
    );
}
