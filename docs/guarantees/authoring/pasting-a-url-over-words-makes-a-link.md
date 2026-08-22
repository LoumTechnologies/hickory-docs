# Pasting A URL Over Words Makes A Link

Given a selection of prose in a document and a URL on the clipboard, when the
paste happens then the selected words become the label of a markdown link
pointing at the pasted address — `[the spec](https://example.com/spec)` — and
the caret lands after it. With nothing selected, or with something on the
clipboard that is not a URL, the paste is the ordinary paste it has always
been.

## Why this is the right default

It is what every editor people already use does, so doing anything else here
is the surprising choice. And the alternative interpretation — replace three
words with an address — is one nobody wants: the words were selected because
they are the thing the link is about.

## The rules

1. **A URL is a scheme and something after it, on one line, with no
   whitespace.** `https:`, `http:`, `ftp:`, `mailto:`, `tel:`, `file:`, plus a
   bare `www.host.tld`, which is the other thing browsers and mail clients put
   on the clipboard. "See https://example.com for more" is prose being pasted,
   and swallowing it into a link would be the wrong guess.
2. **A schemeless `www.` address gets `https://`.** A markdown destination
   with no scheme is a relative path, so leaving it bare would produce a link
   to a file that does not exist.
3. **Spaces and parentheses in the destination are percent-encoded**, because
   either of them ends a markdown destination early.
4. **Selecting a whole link and pasting a URL RE-POINTS it.** `[the
   spec](old.md)` with a new address pasted over it becomes `[the
   spec](new)` — not a link nested inside a label, which is what wrapping
   would produce and what nobody means.
5. **A multi-line selection is not a label.** The paste is ordinary.
6. **Prose only.** A URL pasted over part of a shell command inside a cell is
   a URL: the verbatim ranges of the document are excluded, the same ones the
   maths renderer and the prose measure exclude, and for the same reason.

## Boundary

Nothing is escaped inside the LABEL. Selecting text that contains `]` and
pasting a URL over it produces a link markdown will read as ending early —
which is visible immediately in the buffer, and is the same thing that happens
if you type it. Escaping it would mean editing text the reader selected, which
is a larger liberty than this act should take.

---

Last LLM verification:
- Date: 2026-08-21
- Reviewer: Claude (Opus 5)
- Result: verified (implemented and reviewed in the same change)
- Evidence: `apps/web/src/lib/mdLinks.ts` — `isUrl` (rule 1), `normalizeUrl`
  (rules 2 and 3), `linkOverSelection` (rules 4 and 5).
  `apps/web/src/editor/mdPaste.ts` — the `paste` handler: images first, then
  the selection check, then `isProse` (rule 6), then the dispatch with the
  caret after the whole link.
  `apps/web/src/editor/DocumentEditor.tsx` — `isProse` supplied from
  `verbatimRanges(structureOf(state).blocks)`.
- Test coverage: `apps/web/src/lib/mdLinks.test.ts` — `isUrl` in both
  directions, the schemeless case, the encoding, the re-point, and the three
  cases that stay out of the way.
  `apps/web/src/editor/mdPaste.test.ts` — the same acts through a real
  `EditorView` and a real paste event, including that an ordinary paste and a
  caret-only paste still produce what CodeMirror's own handler produces, and
  that a range the caller calls non-prose is left alone. Note that
  `defaultPrevented` is NOT the signal for "this stayed out of it" —
  CodeMirror prevents the default on every paste — so the assertions are on
  the buffer.
- Caveat requiring review: rule 6 depends on the caller's `isProse`, which
  checks the range's START only. A selection that begins in prose and ends
  inside a cell would be wrapped; that selection is not a thing anyone makes
  on purpose, and narrowing it further would cost a second scan per paste.
