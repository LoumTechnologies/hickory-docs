# A Scaffolder's Output Becomes Bytes The Document Owns, Or The Ingest Is Refused

Given an `<hick:exec>` cell whose command is wrapped in a `<hick:copy id=…>`
child and which mounts a volume declared with `output=`, when `hick ingest
--from '#id' <doc>.hick` runs, then the cell's output volume is written into
the document as ordinary `<hick:file>` blocks inside an `<hick:ingested>`
element recording the run — and the document is verified to weave those exact
bytes before the ingest is accepted.

The element shape is the load-bearing decision, and it is fixed:

```
<hick:exec container="sdk" image="…">
<hick:copy id="scaffold">
dotnet new webapi -o .
</hick:copy>
<hick:ingested from="#scaffold" sha256="9f2c…" at="2026-08-23" files="38" skipped="2">
<hick:file path="Program.cs">…</hick:file>
…
</hick:ingested>
</hick:exec>
```

Nesting is `exec > ingested > file`, **never** `exec > file`, because
`file > exec` already means the opposite ("run this, paste the output here").
An intervening element that names the relation is what stops the same two tags
in either order meaning inverse things. `hick:copy` around the command is what
stops forty file bodies from sitting as siblings of ambient command text.

Corollaries that are part of the guarantee:

- **The base survives a clone.** Weaving the document — no execution at all —
  reproduces the scaffold. This is the property a `from=` pointing at a
  captured exec could not have: transcripts live in the gitignored
  `.hick-cache/`, so a clone held the reference and not the referent.
- **Ingest never deletes the user's bytes and never calls a model.** Nothing
  outside the document is touched; parsing a volume into elements is offline,
  first-party and deterministic.
- **Identity is one fingerprint per run, N files under it.** `sha256=` is
  over the bytes the document now holds, path by path in sorted order, each
  length-prefixed. A scaffold is a single event that happens to write forty
  things, and forty unrelated hashes would lose that.
- **The project's own `.gitignore` is the filter.** `bin/`, `obj/`,
  `node_modules/` are skipped, counted in `skipped=`, and named on stdout. A
  document that owns the source must not carry build output, and this needs no
  new configuration because it is what the user already means.
- **Non-UTF-8 output is refused by name, not mangled.** A `hick:file` body is
  raw bytes under the no-escaping invariant, so there is no encoding to hide a
  binary in. The refusal names the files, says why, and points at the
  gitignore as the fix. Nothing is written. A side-car for binaries is a later
  question and is deliberately not designed.
- **Text the parser would read as structure is refused before anything is
  written**, and the whole ingest is additionally proven byte-exact by a weave
  afterwards, with the document restored on any failure — the same discipline
  `hick adopt` uses.
- **A volume the document has ingested is no longer flushed as a pipeline
  output.** Ownership transferred: the document holds those bytes and your
  edits to them, and flushing a fresh run over the top would silently
  overwrite your four lines on every `hick run`.
- **A second ingest into the same cell is refused**, naming the recorded base
  and its date, because re-ingesting is a three-way merge against that base
  and that is deliberately a later step.

What this does NOT claim: nothing here merges, records a correspondence, or
detects a volatile region. Re-ingest as a three-way merge, volatile regions,
and the correspondence record are steps 3–5 of
`docs/specs/freeform/owning-what-a-scaffolder-wrote.md` and are not built.

---

Last LLM verification:
- Date: 2026-08-23
- Reviewer: Claude (Opus 5)
- Result: verified by tests.
- Evidence:
  - `crates/hickory-cli/src/ingest_exec.rs` — `ingest_from_exec` (locate the
    cell, refuse a second ingest, resolve the output volume, run, filter,
    refuse, write, verify, restore), `gitignored` (`git check-ignore --stdin
    --no-index`), `run_fingerprint`, `ingested_block`, `element_content_end`.
  - `crates/hickory-cli/src/main.rs` — `IngestArgs::from`,
    `cmd_ingest_from_exec` and its report.
  - `crates/hick-exec/src/dag.rs` — `command_text`, `extract_exec_info`'s
    `stdin_children`, and `scan_copy_paste` all exclude `ingested`, so a
    document's scaffold is never shipped to the shell or read as a pipeline
    edge.
  - `crates/hick-literate/src/lib.rs` — `ingested_file_blocks` and
    `add_file_output_for` (the `exec > ingested > file` outputs),
    `ingested_volume_names` and the volume flush's skip, and the binary-safe
    flush via `volume_state::read_tar_files`.
  - `crates/hick-literate/src/weave.rs` — `weave_ingested_block`, and the
    `exec` fall-through that reaches it.
  - `crates/hick-literate/src/render.rs` — the block model the app draws:
    `command_text` excludes `ingested` (so a cell's command line is the
    command, not a scaffolder's whole output) and `walk` descends into an
    exec's `ingested` child (so the files the document now owns are visible
    in the editor).
  - Tests: `crates/hickory-cli/tests/ingest_scaffold.rs` — seven cases covering
    the element shape and nesting, the gitignore filter and its naming, the
    weave-from-a-clone property, the ingested origin, the second-ingest
    refusal, the non-UTF-8 refusal, an unknown id, and the app's block model.
- Caveat requiring LLM review: `skipped=` records a COUNT and not the reasons.
  The names are printed by the command and are not durable in the document. A
  reader of the `.hick` six months later sees "2 skipped" and must consult
  `.gitignore` to know which two. This is a deliberate consequence of pinning
  the element shape; if it proves insufficient it wants a new attribute rather
  than an overloaded one.
