<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Ingest Keeps The Original Bytes, Calls No Model, And Never Deletes Your File

For: Authors editing executable documents and the files those documents produce.

Given a file in the notes folder's inbox, when it is ingested, then a `.md`
note is written holding the file's bytes verbatim, its attendees derived from
who actually spoke, and its summary passages **empty and stale**; and the source
file is **moved** into `inbox/ingested/`, never deleted. When the same bytes are
ingested again, then no second note is written.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
- [Requirements 2](requirements/02.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/ingest-keeps-the-original-bytes.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />
- [Requirement detail](requirements/02.md)
<storycode:include file="requirements/02.md" />

## Implementation

## Test the behavior: `hick`

This is the maintained test fragment from `crates/hickory-cli/tests/ingest.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-a492c9d4e3cc-hick"></a>
### `hick`

Ingest: a meeting becomes a note.
Protects docs/guarantees/authoring/ingest-keeps-the-original-bytes.md
The three properties worth testing are the three promises the spec makes:
the user's file is never destroyed, no model is ever called, and dropping
the same transcript twice does not produce two notes.

<storycode:copy id="f-a492c9d4e3cc-hick">
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

</storycode:copy>

## Test the behavior: `an_inbox_starting_at_a_root_is_refused_on_every_platform`

This is the maintained test fragment from `crates/hickory-cli/tests/ingest.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-a492c9d4e3cc-an_inbox_starting_at_a_root_is_refused_on_every_platform"></a>
### `an_inbox_starting_at_a_root_is_refused_on_every_platform`

Protects `docs/guarantees/authoring/ingest-keeps-the-original-bytes.md`

<storycode:copy id="f-a492c9d4e3cc-an_inbox_starting_at_a_root_is_refused_on_every_platform">
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

</storycode:copy>

## Test the behavior: `a_macos_download_records_where_it_came_from`

This is the maintained test fragment from `crates/hickory-cli/tests/ingest.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-a492c9d4e3cc-a_macos_download_records_where_it_came_from"></a>
### `a_macos_download_records_where_it_came_from`

The macOS download-origin path, end to end, against the real extended
attribute rather than the parser alone.
Protects docs/guarantees/authoring/ingest-keeps-the-original-bytes.md
macOS only, because `com.apple.metadata:*` is not a namespace Linux will let
anything write — which is exactly why this went unverified until someone ran
it on a Mac.

<storycode:copy id="f-a492c9d4e3cc-a_macos_download_records_where_it_came_from">
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

</storycode:copy>

<a id="output-44b2a022e6da"></a>
## `crates/hickory-cli/tests/ingest_naming.rs`

Related contracts: [Ingest Does Not Require The Rest Of The Document To Already Pass](../ingest-does-not-require-the-rest-of-the-document-to-already-pass/contract.md), [Ingest Keeps The Original Bytes, Calls No Model, And Never Deletes Your File](contract.md).

<a id="f-44b2a022e6da-VTT"></a>
### `VTT`

An ingested note is named for the meeting, not for the download: a date in
the file name is the meeting's date, the title is the rest, and the
transcript's id is the note's own name so two meetings upstream of one
document never both answer to `#transcript`.

<storycode:copy id="f-44b2a022e6da-VTT">
//! An ingested note is named for the meeting, not for the download: a date in
//! the file name is the meeting's date, the title is the rest, and the
//! transcript's id is the note's own name so two meetings upstream of one
//! document never both answer to `#transcript`.
//! Guarantee: docs/guarantees/authoring/ingest-keeps-the-original-bytes.md

use hickory_cli::ingest::{date_in_name, note_for};
use std::path::Path;

const VTT: &str = "WEBVTT\n\n00:00:01.000 --> 00:00:02.000\n<v Sam>Ship it.\n";

</storycode:copy>
<a id="f-44b2a022e6da-the_date_in_the_name_wins_and_the_title_is_the_rest"></a>
### `the_date_in_the_name_wins_and_the_title_is_the_rest`

<storycode:copy id="f-44b2a022e6da-the_date_in_the_name_wins_and_the_title_is_the_rest">
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

</storycode:copy>
<a id="f-44b2a022e6da-a_name_without_a_date_keeps_the_given_date_and_a_plain_title"></a>
### `a_name_without_a_date_keeps_the_given_date_and_a_plain_title`

<storycode:copy id="f-44b2a022e6da-a_name_without_a_date_keeps_the_given_date_and_a_plain_title">
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
</storycode:copy>

### Assemble `crates/hickory-cli/tests/ingest_naming.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/ingest_naming.rs"><storycode:paste select="#f-44b2a022e6da-VTT"/><storycode:paste select="#f-44b2a022e6da-the_date_in_the_name_wins_and_the_title_is_the_rest"/><storycode:paste select="#f-44b2a022e6da-a_name_without_a_date_keeps_the_given_date_and_a_plain_title"/></storycode:file>

## Implementation review

### Implementation

- [crates/hickory-cli/src/adopt.rs](../../../mechanisms/crates/hickory-cli/src/adopt.rs.md#output-b820c141d77c)
- [crates/hickory-cli/src/ingest.rs](../../../mechanisms/crates/hickory-cli/src/ingest.rs.md#output-9e9d98a631d5)
- [crates/hickory-cli/src/ingest_exec.rs](../../../mechanisms/crates/hickory-cli/src/ingest_exec.rs.md#output-4e4b92369bcd)
- [crates/hickory-cli/src/main.rs](../../../mechanisms/crates/hickory-cli/src/main.rs.md#output-5607f2721a2d)
- [crates/hickory-cli/src/up/mod.rs](../../../mechanisms/crates/hickory-cli/src/up/mod.rs.md#output-73148a3ab9db)

### Tests and fixtures

- [crates/hickory-cli/tests/ingest.rs](../../../mechanisms/crates/hickory-cli/tests/ingest.rs.md#output-a492c9d4e3cc)
- [crates/hickory-cli/tests/ingest_naming.rs](contract.md#output-44b2a022e6da)
- [crates/hickory-cli/tests/ingest_scaffold.rs](../../../mechanisms/crates/hickory-cli/tests/ingest_scaffold.rs.md#output-6682c6282b3c)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
