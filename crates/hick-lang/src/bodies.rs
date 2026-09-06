//! Which elements hold BYTES, and which hold bytes so completely that the
//! parser does not look inside them at all.
//!
//! Two closed lists, next to each other because they are two strengths of
//! the same idea and a reader comparing them should not have to search.
//! Beside `lib.rs` rather than in it because that file is past the
//! file-length ratchet's line and may only shrink
//! (`scripts/check-file-length.sh`).

/// Is `name` an element whose body is BYTES rather than prose?
///
/// Inside one of these, `<!--` is four characters of somebody's file and
/// nothing else. That is what
/// `docs/guarantees/language/prefix-only-parsing-no-escaping.md` promises —
/// only a prefixed tag is structure — and the comment scanner was the one
/// place the parser broke it: a `<hick:file path="index.html">` could not
/// hold an HTML comment, and a `<hick:copy language="rust">` could not hold
/// a Rust string, doc comment, or test fixture containing `<!--` without the
/// bytes silently disappearing between the document and the file it tangles.
/// There is no escape to reach for, by design, so the parser has to be the
/// one that stops looking.
///
/// The list is every element whose children are content:
///
/// - `copy`, `cut` — a fragment, in whatever language it is.
/// - `file` — a file, in whatever language it is.
/// - `exec` — a command, and shells have their own comment syntax.
/// - `expect` — bytes a command is claimed to have printed.
/// - `diagram`, `table`, `math` — a picture's source, CSV, and LaTeX.
///
/// Deliberately NOT here: `doc`, `when`, `claim`, `section`, `said`,
/// `transcript`'s prose siblings — the wrappers whose children are prose,
/// where a comment is what an author means by `<!--`. Commenting an entire
/// `<hick:file>` block out from OUTSIDE it keeps working, because the
/// comment starts in prose.
///
/// [`is_raw_content_tag`] is the stronger version of this idea — those
/// elements are not parsed for nested tags at all — and every one of them is
/// already immune for that reason, so none is repeated here.
pub(crate) fn holds_bytes(name: &str) -> bool {
    matches!(
        name,
        "copy" | "cut" | "file" | "exec" | "expect" | "diagram" | "table" | "math"
    )
}

/// Is `name` a verbatim-capture element? See the note on `parse_children`.
pub(crate) fn is_raw_content_tag(name: &str) -> bool {
    // `transcript` joins the session vocabulary here for the same reason: its
    // content is bytes some other tool produced, and a meeting where somebody
    // said "the hick:copy tag" is not a parse error. See
    // `docs/specs/freeform/ingest.md` — the raw block is the source of truth,
    // and speaker turns are derived from it rather than stored beside it.
    // `context` is what a harness put in front of the model besides the
    // conversation — a compaction summary, a hook's output, an attached file.
    // Bytes another tool produced, like a tool result; see `hick ingest --from claude-code`.
    matches!(
        name,
        "input" | "tool-result" | "transcript" | "reasoning" | "context"
    )
}
