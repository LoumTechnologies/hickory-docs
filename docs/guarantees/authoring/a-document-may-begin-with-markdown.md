# A Document May Begin With Markdown

Given a `.hick` file whose prologue does not open a `<PREFIX:doc>` element,
when it is parsed, then the whole file is the body of an implicit root: there
is no wrapper to write at the top and none to close at the bottom, the
namespace prefix is `hick` without any `xmlns:` declaration, and every byte of
prose — before, between, and after any tags — survives byte-for-byte. When the
file *does* open that element in its prologue, it is parsed exactly as it
always was.

This exists because the first line of a file is the whole first impression of
a format, and a note-taking IDE (`docs/specs/freeform/notes-ide.md`) cannot ask
someone to open a meeting with `<?xml version="1.0"?>`. The wrapper stops being
mandatory ceremony and becomes what it should always have been: the explicit
form, for documents that need something only it can express.

Four properties hold it up:

1. **The prologue decides, not the file.** Wrapped-or-bare is answered after an
   optional XML declaration, comments, and frontmatter — never by searching the
   source. Searching was safe only while prose could not precede the root; now
   that it can, a note that mentions `<hick:doc` would otherwise be parsed
   from the middle, silently discarding everything before it.
2. **Only the root is optional.** A bare body ends at EOF, but an unclosed tag
   inside it is still `UnclosedTag`, and a closing tag with nothing to close is
   still `UnexpectedClose` — there is no root that could have opened it. The
   ambiguity bare documents introduce is *only* about whether a root exists.
3. **YAML frontmatter configures a bare document, and only a bare one.** A
   wrapped document configures itself with attributes on its root; two
   mechanisms for one thing in one file is how a format rots. `weave`,
   `prefix`, and `volatile` are reserved and typed; **every other key is
   metadata that is preserved and never interpreted**, because the format does
   not get an opinion about what `attendees` means.
4. **Frontmatter bytes stay in the node stream.** `HickDocument::frontmatter`
   is a *view* over text that is also present in `nodes`, never a replacement
   for it. That is what makes the block weave through verbatim, keeps its spans
   addressable by lineage, and leaves the reverse-edit path with nothing to
   reconstruct.

## Boundary

**Rebinding the prefix still requires the explicit root.** Documentation about
hick uses `h:` so that `hick:` examples stay literal text (`AGENTS.md`), and
those documents keep their `<h:doc xmlns:h="…">` wrapper — the file that needs
to talk about the syntax is exactly the file that can afford four lines of it.
A bare document may name a prefix in frontmatter, but nothing else can.

**A leading `---` is ambiguous and is resolved toward content.** It is
frontmatter only when the first line is exactly `---`, a closing fence exists,
and what lies between reads as a mapping. A note that opens with a horizontal
rule keeps its rule. This is the same ambiguity every tool in this space has,
resolved the same way, and it is named rather than solved.

**The recognised frontmatter subset is smaller than YAML** — top-level
`key: value` entries, comments, blank lines, and indented or `- ` block content
under a key. Anything unrecognised is content, which is the safe direction to
be wrong in: a note loses a metadata block it never had, rather than losing its
first paragraph.

---

Last LLM verification:
- Date: 2026-08-18
- Reviewer: Claude (Opus 5)
- Result: verified (implemented and reviewed in the same change)
- Evidence: `crates/hick-lang/src/lib.rs` — `opens_root` (prologue-only
  detection, including the `<hick:document` vs `<hick:doc` boundary check),
  `Parser::parse_bare_document`, `parse_children` taking `Option<&str>` so a
  bare body terminates at EOF while a nested one still requires its close tag,
  `split_frontmatter` / `frontmatter_entry` / `unquote_scalar` (the recognised
  subset), `resolve_prefix` and `declared_prefix` (xmlns wins, frontmatter
  second, `hick` last), and `Frontmatter` with `RESERVED_FRONTMATTER_KEYS`.
- Test coverage: `crates/hick-lang/src/lib.rs::tests` —
  `a_rootless_file_is_a_bare_document`, `a_bare_document_recognises_tags`,
  `an_empty_file_is_an_empty_bare_document`,
  `a_stray_closing_tag_in_a_bare_document_is_an_error`,
  `an_unclosed_tag_in_a_bare_document_still_errors`,
  `the_root_is_detected_from_the_prologue_not_the_file`,
  `a_wrapped_document_is_still_wrapped`, `frontmatter_sets_reserved_keys`,
  `frontmatter_bytes_stay_in_the_node_stream`,
  `frontmatter_metadata_is_preserved_and_never_interpreted`,
  `frontmatter_quotes_are_stripped`, `frontmatter_can_rebind_the_prefix`,
  `a_horizontal_rule_is_not_frontmatter`, `an_unterminated_fence_is_content`,
  `a_non_mapping_fence_is_content`,
  `a_colon_without_a_space_is_not_a_mapping_entry`. The previous
  `error_on_missing_root` test was replaced: `ParseError::MissingRoot` is no
  longer reachable from a rootless `.hick` file, and survives only on the
  `parse_session` path.
- Run end to end on this machine: a bare `notes/standup.hick` with frontmatter
  (`date`, `attendees`, a `tags` block list), a heading, `hick:copy`, and
  `hick:paste` wove `notes/standup.md` with the frontmatter block reproduced
  byte-for-byte and the paste resolved; `hick test` then reported `ok`. The
  same content in wrapped form produced identical output, which is the property
  that matters — bare and wrapped are two spellings of one document.
- Caveats — what LLM review could NOT establish:
  - **No editor or app surface has been touched.** This is the parser only.
    Whether the app's editor, the LSP, or the WYSIWYG layer render a bare
    document sensibly — in particular whether the frontmatter block is
    presented as metadata rather than as prose — is unverified and untested.
  - **The reverse-edit path over frontmatter is unproven.** `hick up` carries
    an edit made in `standup.md` back into `standup.hick`; an edit made *inside
    the woven frontmatter block* is a case that has never existed before, and
    nothing here establishes what it does. Named as an open edge in
    `bare-documents.md`.
  - **A bare document that opens a nested `hick:doc` is undefined.** Rule 1
    makes it safe to *mention*, and the prologue rule keeps it from being read
    as the root, but no decision has been made about what such a tag means.

## A byte-order mark is an encoding signature, not the start of the document

Measured on Windows, 2026-08-20, by running the shipped binary on a real
machine rather than in CI.

A `.hick` file saved by Notepad, by PowerShell's `Set-Content -Encoding UTF8`,
or by any of the Windows editors that write a UTF-8 BOM began with `U+FEFF`.
`str::trim_start` does **not** remove that — it is a format character, not
`White_Space` — so `opens_root` tested the BOM against `<?xml` and `<hick:doc`,
failed both, and the file was parsed as a **bare** document. Its wrapper went
unrecognised, every cell in it was skipped, and the weave came out as a single
line of leftover XML declaration.

The failure mode is what makes this worth stating: `hick run` printed
`1 file(s) written` and **exited 0**. A Windows author got no error, no
diagnostic, and nothing to search for — only a document that quietly did
nothing. A file in someone's git repository outlives every version of this
tool, so the parser tolerates the signature rather than rejecting the file.

Only a **leading** BOM is removed. One anywhere else is ordinary text and stays
byte-for-byte, because the no-escaping invariant is about content and only the
leading mark is not content.

Verified by `a_byte_order_mark_does_not_turn_a_wrapped_document_into_a_bare_one`
and `a_byte_order_mark_that_is_not_leading_is_left_alone` in
`crates/hick-lang/src/lib.rs`.
