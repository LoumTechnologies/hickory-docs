# Many Documents Contribute To One File, And None Of Them Names The Others

Given `<hick:paste select=".ignore" />` in the document that owns a generated
file, when any other `.hick` document **in the same folder** declares
`<hick:copy class="ignore">`, then that fragment is collected into the file —
and the owning document names no contributor, while the contributor names no
collector. They meet on the class.

`<hick:private />` at the top of a document withholds its fragments from the
folder. There is no `public`: contributing is the default, because the case
this exists for is many documents feeding one shared file, and making each of
them announce itself puts the coupling straight back.

Four rules hold it up:

1. **A folder, not a repository.** Discovery is the document's own directory
   and is not recursive. A folder is the unit a person can hold in their head;
   walking a whole tree would let a fragment in an unrelated corner change
   this document's output.
2. **A contribution is the contributor's OWN fragments.** Ambient edges do not
   follow the contributor's own `hick:upstream`. Following them would make
   every document upstream of every other, and the first explicit chain in the
   folder would then be a cycle — which is exactly what happened the first
   time this was built as a synthesised `hick:upstream` edge run back through
   the resolver.
3. **Local wins, ambient adds.** Contributions are appended, so `#id`
   resolution — first match — keeps the document's own declaration, while
   `.class` collects both. A document can be added to; it cannot be overridden.
4. **A sibling already reached is not reached twice.** A file spliced by
   `hick:include` or `hick:upstream` is skipped, found through the document's
   `span_files` — the resolver's `seen` set is a cycle-detection *stack* and
   holds nothing by the time it returns.

Because a contribution is spliced exactly as a declared upstream is, it gets
what a declared one gets: `hick lineage` names the **contributing file** for
those bytes, and an edit to them in the generated file routes to the document
that wrote them.

## Boundary

Ordering is document order, contributors sorted by filename, so two machines
weave the same file. A contributor that cannot be read or does not parse is
skipped rather than failing this run: that is its own error, reported when it
is run.

---

Last LLM verification:
- Date: 2026-08-31
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence:
  - `attach_ambient_contributors` and `declares_private` in
    `crates/hick-literate/src/lib.rs`, called from `prepare_pipeline` AFTER
    `resolve_includes` so the spliced nodes are never re-resolved.
  - `hick_lang::attach_contribution` in `crates/hick-lang/src/lib.rs` — parses
    the contributor, takes `fragments_of` its own nodes only, stamps
    `span_file_id` so spans index the contributor, and appends one
    `hick:upstream` node holding them.
  - Verified by hand in a three-document folder: `owner.hick` naming nobody
    wove `bin/` and `node_modules/` from `bot-a.hick` and `bot-b.hick`;
    `hick lineage` reported `literal bot-a.hick bytes 26..30` and
    `literal bot-b.hick bytes 26..39`; adding `<hick:private />` to a third
    document removed its line.
- Test coverage: `crates/hickory-cli/tests/ambient_contribution.rs`.
- Caveat requiring LLM review: id collision between a contributor and the
  collector is resolved by "local wins" rather than reported. That is quiet in
  a way this codebase usually refuses, and it is quiet on purpose — erroring
  would break any folder whose documents already share an id — but nothing
  tells a person it happened.
