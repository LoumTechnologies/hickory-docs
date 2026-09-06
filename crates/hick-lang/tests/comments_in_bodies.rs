//! Comments are a PROSE construct: inside the body of an element that holds
//! bytes, `<!--` is four characters of somebody's source file.
//!
//! Guarantee: docs/guarantees/language/prefix-only-parsing-no-escaping.md
//!
//! Out here rather than in `lib.rs`'s `tests` module because that file is
//! past the file-length ratchet's line and may only shrink
//! (`scripts/check-file-length.sh`); a test directory is excluded from the
//! count for exactly this reason.

/// Protects docs/guarantees/language/prefix-only-parsing-no-escaping.md.
#[test]
fn a_comment_marker_inside_a_file_body_is_bytes_not_a_comment() {
    let src = r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:file path="index.html"><body>
<!-- the nav lives here -->
</body>
</hick:file>
</hick:doc>"#;
    let doc = hick_lang::parse(src).unwrap();
    let files = doc.find_tags("file");
    assert_eq!(files.len(), 1);
    assert_eq!(
        files[0].text_content(),
        "<body>\n<!-- the nav lives here -->\n</body>\n",
        "a file's own bytes survive verbatim, comment marker included"
    );
}

/// Protects docs/guarantees/language/prefix-only-parsing-no-escaping.md.
#[test]
fn a_comment_marker_inside_a_copy_body_is_bytes_not_a_comment() {
    let src = r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:copy id="frag" language="rust">let s = "<!-- x -->";
</hick:copy>
</hick:doc>"#;
    let doc = hick_lang::parse(src).unwrap();
    let copies = doc.find_tags("copy");
    assert_eq!(copies.len(), 1);
    assert_eq!(copies[0].text_content(), "let s = \"<!-- x -->\";\n");
}

/// An UNCLOSED `<!--` inside a body is not an error either: there is no
/// comment there to be unclosed. Before this, a Rust file containing the
/// four characters `<!--` in a string made the whole document fail to
/// parse.
#[test]
fn an_unclosed_comment_marker_inside_a_body_is_not_an_error() {
    let src = r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:file path="a.rs">const OPEN: &str = "<!--";
</hick:file>
</hick:doc>"#;
    let doc = hick_lang::parse(src).unwrap();
    assert_eq!(
        doc.find_tags("file")[0].text_content(),
        "const OPEN: &str = \"<!--\";\n"
    );
}

/// Rawness is inherited: a prose-level wrapper nested inside a body does
/// not turn commenting back on halfway down a file.
#[test]
fn a_gate_inside_a_file_body_does_not_re_enable_comments() {
    let src = r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:file path="a.html"><hick:when feature="nav"><!-- nav --></hick:when>
</hick:file>
</hick:doc>"#;
    let doc = hick_lang::parse(src).unwrap();
    let file = &doc.find_tags("file")[0];
    let when = file
        .child_tags()
        .find(|t| t.name == "when")
        .expect("the gate is a child of the file");
    assert_eq!(when.text_content(), "<!-- nav -->");
}

/// And a tag whose children are prose still takes comments.
#[test]
fn a_comment_inside_a_prose_wrapper_is_still_a_comment() {
    let src = r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:claim by="sam">kept <!-- dropped --> also kept</hick:claim>
</hick:doc>"#;
    let doc = hick_lang::parse(src).unwrap();
    let claim = &doc.find_tags("claim")[0];
    assert_eq!(claim.text_content(), "kept  also kept");
}
