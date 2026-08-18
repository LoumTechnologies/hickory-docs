# Ingest Keeps The Original Bytes, Calls No Model, And Never Deletes Your File

Given a file in the notes folder's inbox, when it is ingested, then a `.hick`
note is written holding the file's bytes verbatim, its attendees derived from
who actually spoke, and its summary passages **empty and stale**; and the source
file is **moved** into `inbox/ingested/`, never deleted. When the same bytes are
ingested again, then no second note is written.

Three promises, and each exists because of a specific way this could go wrong:

1. **Nothing here calls a model.** Parsing is offline, first-party, and
   deterministic, so a meeting recorded on a plane becomes a note on the plane,
   and a machine that has never had internet can still ingest. The summaries are
   written with an empty `from=""` fingerprint, so `hick test` reports them stale
   immediately and names `hick refresh` as the fix — a note that has never been
   summarized is *visibly* unsummarized rather than silently blank.
2. **The user's file is never destroyed.** The bytes came from outside and we do
   not own them. A move is reversible by anyone looking at the directory; a
   delete is a support ticket from someone who dropped in their only copy. A
   file that could **not** be read is not moved either — the user still has to
   be able to find it.
3. **A file that is still arriving is never read.** A download or a large copy
   lands in pieces, and reading one mid-flight produces a truncated transcript
   that would then be fingerprinted, ingested, and moved out of the inbox — a
   silently incomplete note, which is the worst outcome available. In-flight
   names (`.crdownload`, `.part`, `.partial`, `.download`, `.opdownload`,
   `.filepart`, `.tmp`, dotfiles, `~$…`, `…~`) are passed over **in silence**,
   because reporting a file that is working correctly is noise; anything else
   must stop growing before it is read, and is reported as `Waiting` if it has
   not.
4. **The same transcript twice is one note.** Identity is the SHA-256 of the
   source bytes, recorded in the note's frontmatter, because the reflex when
   something looks like it failed is to try again.

## What the note records about where it came from

`source:` is the file's name, and `source-url:` is where the operating system
says it was downloaded from — `kMDItemWhereFroms` on macOS, the NTFS
`Zone.Identifier` stream on Windows, `user.xdg.origin.url` on the Linux browsers
that set it. Nobody types either, which is what makes them evidence rather than
assertions (`provenance-and-standing.md`).

**The URL's query string is dropped, and that is not a rounding error.** An
export link from Drive, S3, or any signing service carries its credential in the
query; this value goes into a file in a git repository that may be pushed to a
remote other people can read, and `config-and-environments` says never write a
secret into a file we create. Scheme, host, and path survive, which is the part
that answers "where did this come from".

**A Google Drive shortcut is refused, with the export path spelled out.** A
`.gdoc` holds a URL, not a document — ingesting one would produce a note
containing a JSON stub, which is worse than useless because it looks like it
worked. Since a Google Meet transcript and a Gemini notes document are both
Google Docs, this is the failure a Workspace user hits first, so the refusal
names the fix: File > Download > Markdown, then put that file in the inbox.

## Boundary

**Ingest is deterministic: there is no clock in it.** The note's `date:` comes
from the source file's own modification time, so ingesting the same file twice
produces byte-identical notes. "When did I get round to it" is not a fact about
the meeting.

**One bad file never stops the good ones.** Every outcome is reported,
including the skips, with a reason and a next step: a file that silently stayed
in the inbox with no explanation is how someone concludes the feature is broken.

**A file containing `</hick:transcript>` is refused, not mangled.** There is no
escaping in this language by design, so such a file cannot be stored verbatim,
and saying so is the only honest option.

**An unrecognised format is adopted, not refused.** The note records
`source-format: unrecognised` and derives no turns — the material survives and
only the structure is missing, and the gap is written down rather than hidden.

**The inbox is configuration with a working default.** `HICKORY_INBOX` is
validated at construction and refuses an absolute path or one containing `..`,
because ingest *moves* files and an escaping path would move somebody's file
somewhere they did not ask for.

---

Last LLM verification:
- Date: 2026-08-18
- Reviewer: Claude (Opus 5)
- Result: verified (implemented and reviewed in the same change)
- Evidence: `crates/hickory-cli/src/ingest.rs` — `InboxConfig` (typed,
  validated, defaults to working with nothing set), `note_for` (frontmatter,
  derived attendees, verbatim raw block, two empty transforms, the close-tag
  refusal), `ingest_one` (fingerprint, dedupe against existing notes, write,
  then move with a copy-then-remove fallback across filesystems),
  `ingest_inbox` (stable order, one failure never stopping the rest),
  `source_date` / `civil_date` (the file's own mtime, no clock). CLI:
  `cmd_ingest` in `crates/hickory-cli/src/main.rs`. Loop:
  `drain_inbox` in `crates/hickory-cli/src/up/mod.rs`, called before the first
  weave so a waiting transcript is covered by it, and again on any batch
  touching the inbox so a new note is woven in the same cycle.
- Test coverage: `crates/hickory-cli/tests/ingest.rs` (27 tests) — verbatim
  bytes, derived attendees, empty-and-stale summaries driven through the real
  `hick test` binary, dedupe, move-not-delete, an unreadable file skipped with a
  reason *and left in place*, one bad file not stopping a good one, an
  unrecognised format keeping its material, the close-tag refusal, determinism
  across two folders, a missing inbox being fine, single-file ingest, the
  config default, every invalid config value naming the variable and a next
  step, and the command's own reporting.
- Caveats — what LLM review could NOT establish:
  - **The `hick up` integration is not covered by a test.** `drain_inbox` is
    called from the loop in two places and both were exercised by hand, not by
    an automated test; the loop's own tests do not drop a file in an inbox.
  - **No real exporter output has been ingested.** Every fixture is
    hand-written. The formats this exists to read are Granola, Otter, Fathom,
    and Zoom exports, and none has been tried.
  - **Download origin is unverified on this machine.** The Windows and macOS
    readers were written from the documented formats and compiled, but no test
    exercises them — the extended attributes only exist on a file a browser
    actually downloaded, and CI runs on files a test wrote. The macOS reader
    additionally scans a URL out of a binary plist rather than parsing one,
    which is a heuristic that fails closed.
  - **Settling is a heuristic.** A transfer that stalls longer than the settle
    window and then resumes could be read mid-flight; the next pass corrects
    it, but the first note would be briefly wrong.
  - **Note names can collide in ways that surprise.** Two meetings with the
    same title on the same day become `…-2.hick`, which is correct but is not
    a naming scheme anyone chose.
  - **Nothing decides whether a notes repository wants the pre-commit drift
    gate.** A commit blocked because a meeting summary is stale is defensible
    for code and questionable for notes; named as an open edge in
    `notes-ide.md`.
