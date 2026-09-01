# A Syntax Error Names The Invariant It Hit

Given a document whose prose quotes hick's own tag syntax — a sentence or
code span containing literal text shaped like `<prefix:name attr="value">` —
when that text is malformed enough to fail parsing, then the error explains
that hick has no escaping and points at the `h:`-prefix convention for
writing about hick's own syntax, rather than naming only the unexpected
token.

## Why

The no-escaping invariant (`docs/specs/freeform/bare-documents.md`) means
there is no such thing as "just talking about" a tag: any `<prefix:name`
found anywhere — inside a sentence, inside a fenced code block, inside a
backtick span — is parsed as markup. A well-formed example
(`` `<hick:paste select="foo" />` ``) parses fine and silently becomes a real
tag, which is a different, quieter problem `bare-documents.md` already
accepts as the cost of no escaping. A *malformed* example — the natural shape
of documentation that shows syntax errors, or prose that trails off before
closing a quote — hits a real parse failure, and until now that failure said
only what token it choked on: `empty attribute name`, `attribute value must
be quoted`, `unterminated attribute value`. None of those three sentences
gives a reader who did not know they were writing markup any reason to
suspect that's what happened.

This was found writing a tutorial that had to explain hick's own `<hick:paste>`
syntax as part of its content — a document *about* hick hits this far more
than an ordinary one, which is exactly the case `bare-documents.md`'s `h:`
convention exists for, and exactly the case a first-time author would not
know to reach for without being told.

## What changed, and what didn't

`ParseError::Syntax { line, message }` keeps its shape — the fix is text
added to `message` at the three tag/attribute-parsing call sites most likely
reached from prose, not a new error variant or a detection mechanism (there
isn't one to build: the parser has no way to know prose was intended, by
design). The added sentence is constant across all three sites
(`NO_ESCAPING_HINT` in `crates/hick-lang/src/lib.rs`), so it reads the same
regardless of which specific token tripped the parser.

---

Last LLM verification:
- Date: 2026-09-01
- Reviewer: Claude (Sonnet 5)
- Result: verified
- Evidence: `crates/hick-lang/src/lib.rs` — `NO_ESCAPING_HINT` (near `BOM`),
  appended to the messages in `read_attr_name`'s empty-name check and
  `read_attr_value`'s unquoted-value and unterminated-value checks. The
  outer `attr_name.is_empty()` check in the tag-parsing loop also carries the
  hint for symmetry, though tracing the control flow shows `read_attr_name`
  never actually returns `Ok("")` — that branch is unreachable today, and is
  left as-is rather than removed, since deleting dead code was out of scope
  for this fix and a future change to `read_attr_name` could make it live
  again.
- Test coverage:
  `tests::a_malformed_tag_in_prose_names_the_no_escaping_invariant` in
  `crates/hick-lang/src/lib.rs` — three cases (`<hick:x =bad>`,
  `<hick:x y=unquoted>`, an unterminated quoted value), each asserting the
  error's `Display` contains both the invariant explanation and a pointer to
  `bare-documents.md`.
- Caveat requiring LLM review: this covers the three sites most plausibly
  reached from prose describing hick's own syntax. `ParseError::Syntax` has
  other call sites (unclosed comments, EOF mid-tag, malformed frontmatter)
  that were not touched — they are reachable from genuinely malformed
  documents more often than from prose-about-hick, so adding the same hint
  there was judged more likely to mislead than help; revisit if evidence
  says otherwise.
