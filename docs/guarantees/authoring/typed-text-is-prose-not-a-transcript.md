# Typed Text Is Prose, Not A Transcript

Given text typed into the app's scratchpad, when it is saved, then it becomes a
note whose body is **prose** — no `<hick:transcript>`, no derived speaker turns,
no summary transforms — recording only that it came from the scratchpad; and
when the text is empty, or contains hick markup, then it is refused with a
reason and the text is left where the person can act on it.

The reason is the provenance rule in
`docs/specs/freeform/provenance-and-standing.md`, applied at the point material
enters. A `hick:transcript` means *bytes another tool produced*, which is
exactly right for a downloaded export and exactly wrong for a sentence somebody
just typed. On the attributable axis, scratchpad text is the strongest material
there is: a named human wrote it, in this app, and git will say so. Wrapping it
as machine-conveyed material would throw that away and imply a machine step
between the person and the words that never happened.

Three properties hold it up:

1. **Nothing is summarized, because there is nothing to summarize.** An ingested
   transcript gets two empty, stale `hick:transform` passages because a machine
   step stands between the speakers and any account of what they said. Here
   there is no such step — the person already wrote what they meant.
2. **Saving is explicit, never automatic.** A scratchpad that committed every
   keystroke to a note would turn a place to think into a place that keeps a
   record, and people stop thinking in those.
3. **A refusal never costs the words.** The failures reachable here are an empty
   box and prose containing `<hick:`, and both leave the text in place with a
   message that says what to do — there is no escaping in this language by
   design, so markup in prose is a refusal rather than a transformation.

## Boundary

**The clock is read here, and nowhere else in ingest.** A scratchpad note is
*created* now, so "now" is a fact about it. File ingest derives its date from
the source file instead, because re-reading the same file must not depend on the
day.

**One scratchpad at a time.** A second would split a train of thought across two
places, and the point of it is that there is one place to put a thought before
it has a name. It is distinct from the untitled buffer, which is a *document*
that has no name yet; scratchpad text may never become one.

---

Last LLM verification:
- Date: 2026-08-18
- Reviewer: Claude (Opus 5)
- Result: verified (implemented and reviewed in the same change)
- Evidence: `crates/hickory-cli/src/ingest.rs` — `note_from_scratchpad` (prose
  body, `source: scratchpad`, the heading-becomes-title rule, both refusals),
  `today` (the one clock read, documented as such), `save_scratchpad` (unique
  path per note). Route: `post_scratchpad` in
  `crates/hickory-cli/src/serve/plain_file.rs`, registered as
  `POST /api/scratchpad`. Client: `api.saveScratchpad` in
  `apps/web/src/api/client.ts`. UI: `apps/web/src/components/ScratchpadPane.tsx`,
  opened by `openScratchpadTab` (`apps/web/src/views/workspaceState.ts`) via the
  `#/scratchpad` route.
- Test coverage: `crates/hickory-cli/tests/ingest.rs` — prose not transcript,
  the heading not being repeated, a first line still titling the note, the empty
  refusal, the hick-markup refusal, a saved note parsing and weaving through the
  real binary, and two same-titled notes not colliding.
  `apps/web/src/components/ScratchpadPane.test.tsx` (4 tests) — the button
  disabled with nothing typed, the save reporting where the note landed, the box
  clearing afterwards, and a refusal being shown *without* losing the text.
  `apps/web/src/views/workspaceState.test.ts` covers one-scratchpad-at-a-time
  and coexistence with the untitled buffer; `apps/web/src/router.test.ts` covers
  the route.
- Caveats — what LLM review could NOT establish:
  - **No end-to-end test drives the HTTP route.** The handler is a thin
    composition of a tested function and a tested client, but nothing exercises
    the two together.
  - **Nothing has been run in the real app.** The pane typechecks and its tests
    pass under jsdom; it has not been opened in a running desktop window, so
    its placement and sizing in a real layout are unverified.
  - **There is no mobile app to put this in**, which is where a scratchpad is
    most useful. See `notes-ide.md` — `apps/mobile` does not exist yet.
