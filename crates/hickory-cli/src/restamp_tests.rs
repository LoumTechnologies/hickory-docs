use super::restamp;

/// Guarantee: docs/guarantees/verification/a-transform-is-checked-against-the-bytes-it-read.md
#[test]
fn restamp_rewrites_an_existing_fingerprint_and_inserts_the_model() {
    let src = "<hick:transform select=\"#a\" from=\"old\" instruct=\"x\">\nbody\n</hick:transform>";
    let body = src.find('>').unwrap() + 1;
    let out = restamp(
        src,
        0,
        body,
        &[
            ("from", "new1"),
            ("provider", "anthropic"),
            ("model", "claude-sonnet-5"),
        ],
    );
    assert!(out.starts_with(
        "<hick:transform provider=\"anthropic\" model=\"claude-sonnet-5\" select=\"#a\" from=\"new1\" instruct=\"x\">\nbody"
    ), "{out}");
}

/// The passage's own `#id` mentions, filtered to what it was shown,
/// become `cites=` — declared, and only ever what the input labelled.
#[test]
fn cited_ids_keeps_only_ids_the_input_labelled() {
    let input = "[#m1] Sentence.\n\n[#transcript-u3] Sam: The SLO.\n\n[#p95] 212 ms.";
    let passage = "BACKED by [#transcript-u3] and #p95; also #p95 again and #nope and #m1.";
    assert_eq!(super::cited_ids(passage, input), "#transcript-u3,#p95,#m1");
    assert_eq!(super::cited_ids("nothing here", input), "");
}

/// A document may bind any prefix to the namespace; refresh must find the
/// tag it is rewriting by its span, not by the spelling `<hick:transform`.
#[test]
fn restamp_works_under_a_rebound_prefix() {
    let src = "intro\n<slack:transform select=\"#a\" instruct=\"x\">\nbody\n</slack:transform>";
    let open = src.find("<slack:").unwrap();
    let body = src.find('>').unwrap() + 1;
    let out = restamp(src, open, body, &[("from", "abcd1234")]);
    assert!(
        out.contains("<slack:transform from=\"abcd1234\" select=\"#a\""),
        "{out}"
    );
}
