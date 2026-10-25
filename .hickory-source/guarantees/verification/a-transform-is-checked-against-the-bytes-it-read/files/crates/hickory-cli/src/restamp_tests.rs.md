<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/restamp_tests.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A Transform Is Checked Against The Bytes It Read, Wherever They Live](../../../../contract.md)

Captured SHA-256: `348f9b31012b64e0652d53d3649f65fad2d48703e3644aecf4d5e6b4d51988c0`.

<book:entry path="crates/hickory-cli/src/restamp_tests.rs">
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
</book:entry>

</book:doc>
