use super::*;

/// Real `com.apple.metadata:kMDItemWhereFroms` values, captured with
/// `xattr -px` from files downloaded by Safari 26.5 and Chrome 151 on macOS
/// 15.7.7. Byte-for-byte what the operating system wrote, so these protect
/// the parser against the real format rather than against an idea of it.
const CHROME_NO_QUERY: &[u8] =
    include_bytes!("../../tests/fixtures/wherefroms/chrome-linked-no-query.bplist");
const SAFARI_ONE_ENTRY: &[u8] =
    include_bytes!("../../tests/fixtures/wherefroms/safari-direct-one-entry.bplist");
const SAFARI_WITH_REFERRER: &[u8] =
    include_bytes!("../../tests/fixtures/wherefroms/safari-linked-with-referrer.bplist");

/// The regression this parser exists for.
///
/// Scanning the plist as text returned `…/plain-standup.vtt_`: `0x5F` is the
/// marker introducing the *next* string and is also a legal URL character.
/// A download with a query string hid it, because the trailing marker landed
/// after the `?` and was redacted away.
#[test]
fn a_download_url_does_not_pick_up_the_next_plist_marker() {
    let url = first_url(CHROME_NO_QUERY).expect("a URL in a real WhereFroms value");
    assert_eq!(url, "http://127.0.0.1:8792/plain-standup.vtt");
    assert!(!url.ends_with('_'), "trailing plist marker survived: {url}");
}

/// The download URL, not the page that linked it. Both browsers put the file
/// first and the referrer second; taking the wrong one would record where the
/// user was browsing instead of where the bytes came from.
#[test]
fn the_download_url_wins_over_the_referring_page() {
    assert_eq!(
        first_url(SAFARI_WITH_REFERRER).unwrap(),
        "http://127.0.0.1:8791/safari-linked.vtt?token=SAFARI_LINKED_SECRET"
    );
    // Both are in there; the referrer is simply not the one chosen.
    let all = bplist_strings(SAFARI_WITH_REFERRER);
    assert_eq!(all.len(), 2, "expected URL and referrer, got {all:?}");
    assert!(all[1].ends_with("safari-page.html"), "{all:?}");
}

#[test]
fn a_single_entry_value_reads() {
    assert_eq!(
        first_url(SAFARI_ONE_ENTRY).unwrap(),
        "http://127.0.0.1:8791/standup-meeting.vtt?token=SAFARI_SECRET_MUST_BE_REDACTED"
    );
}

/// The security property: what reaches the note carries no query string.
/// A signed export link keeps its credential there.
#[test]
fn an_imported_transcript_passes_attribute_validation() {
    let source = super::note_for(
        "WEBVTT\n\n00:00:01.000 --> 00:00:02.000\nHello meeting.\n",
        std::path::Path::new("standup.vtt"),
        None,
        None,
    )
    .unwrap();
    let doc = hick_lang::parse(&source).unwrap();
    assert!(
        hick_blocks::attribute_errors(&source, &doc).is_empty(),
        "{:?}",
        hick_blocks::attribute_errors(&source, &doc)
    );
}

#[test]
fn the_query_string_never_reaches_the_note() {
    for value in [SAFARI_ONE_ENTRY, SAFARI_WITH_REFERRER] {
        let recorded = redact_url(&first_url(value).unwrap());
        assert!(!recorded.contains('?'), "query survived: {recorded}");
        assert!(
            !recorded.contains("SECRET"),
            "credential survived: {recorded}"
        );
    }
}

/// Linux's `user.xdg.origin.url` is a plain string, not a plist, and still
/// has to work.
#[test]
fn a_plain_url_string_still_reads() {
    assert_eq!(
        first_url(b"https://example.com/transcript.vtt").unwrap(),
        "https://example.com/transcript.vtt"
    );
}

/// Fails closed. A truncated or unreadable plist means "nothing was
/// recorded", which is the honest answer, rather than a guess.
#[test]
fn a_damaged_plist_records_nothing() {
    assert_eq!(first_url(b"bplist00"), None);
    assert_eq!(first_url(&CHROME_NO_QUERY[..40]), None);
    assert_eq!(first_url(b"not a url at all"), None);
}

/// A plist is free to hold a URL as UTF-16, which a text scan cannot see at
/// all. Browsers percent-encode instead, so this is the branch no real
/// download exercised — built by hand, and labelled as such.
#[test]
fn a_utf16_string_reads() {
    let url = "https://example.com/café.vtt";
    let units: Vec<u8> = url.encode_utf16().flat_map(|u| u.to_be_bytes()).collect();
    let count = url.encode_utf16().count();
    let mut plist = b"bplist00".to_vec();
    let string_at = plist.len();
    plist.push(0x6f); // UTF-16 string, length follows as an integer object
    plist.push(0x11); // 2-byte integer
    plist.extend_from_slice(&(count as u16).to_be_bytes());
    plist.extend_from_slice(&units);
    let table_at = plist.len();
    plist.push(string_at as u8);
    let mut trailer = [0u8; 32];
    trailer[6] = 1; // offset width
    trailer[7] = 1; // object ref width
    trailer[15] = 1; // one object
    trailer[31] = table_at as u8;
    plist.extend_from_slice(&trailer);

    assert_eq!(first_url(&plist).as_deref(), Some(url));
}
