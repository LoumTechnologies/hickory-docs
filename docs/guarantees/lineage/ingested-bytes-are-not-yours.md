# Ingested Bytes Are Editable And Never Report As Yours

Given generated output whose bytes came from an `<hick:ingested>` block, when
a reader asks for its lineage, then every such byte reports
`kind: "ingested"` carrying the document span AND the run fingerprint — never
`literal`, which would say a person here typed a scaffolder's forty files, and
never `synthetic`, which would make the bytes uneditable.

Both halves matter, and they are the reason this is its own origin rather than
a reuse of an existing one:

1. **Present, so editable.** An ingested byte is a byte-precise span in the
   `.hick` document, so an edit made in the generated file maps back through
   the path that already exists (`Origin::source()`), and lands in the
   document. Exec-origin bytes are synthetic and refused with 422 — correctly,
   because there is nowhere in a document to put an edit to a command's
   output. That refusal must not reach the four lines you changed in
   `Program.cs`.
2. **Not yours, so labelled.** `SourceOrigin::Ingested` carries the `run`
   fingerprint from `<hick:ingested sha256=…>`, which is what keeps blame
   honest: it is the difference between "you wrote this" and "this arrived on
   2026-08-23 from `dotnet new`".

Corollaries that are part of the guarantee:

- **A length mismatch degrades to `synthetic`, exactly as a literal does.**
  The claim "editable" is false the moment the emitted bytes are no longer
  byte-identical to the span, and half a true origin is worse than none.
- **Narrowing keeps the run.** A transform that slices an ingested span
  produces a narrower ingested span with the same fingerprint; a slice of a
  scaffolder's file is still that scaffolder's bytes.
- **The variant is additive.** `SourceOrigin` is a serde-tagged enum, so
  provenance serialized before it existed still deserializes.
- **The app draws it apart.** `cm-prov-ingested` is neither the editable
  highlight nor the synthetic grey: three states, because there are three.
- **An agent is told the same thing.** `origin_kind` reports `"ingested"`, so
  a model reading lineage does not tell itself somebody here typed the
  scaffold.

Rendered shape (`hick lineage`):

```
       0..512      ingested     app.hick bytes 1204..1716 · run 9f2c0f1a2b3c
```

---

Last LLM verification:
- Date: 2026-08-23
- Reviewer: Claude (Opus 5)
- Result: verified by tests.
- Evidence:
  - `crates/hick-flow/src/node.rs` — `SourceOrigin::Ingested { file, span,
    run }`, additive on the `#[serde(tag = "type")]` enum.
  - `crates/hick-flow/src/transform.rs` — `narrowed` keeps the fingerprint.
  - `crates/hickory-lineage/src/lib.rs` — `Origin::Ingested`,
    `from_provenance_map`'s byte-precision guard, and `Origin::source()` /
    `location()` including it. Tests
    `ingested_bytes_are_editable_and_never_report_as_literal` and
    `an_edit_to_ingested_bytes_maps_back_to_the_document`.
  - `crates/hick-literate/src/lib.rs` — `process_file_children`'s `ingested`
    parameter chooses the origin; `crates/hick-literate/src/weave.rs` —
    `process_file_children_to_weave` does the same for the woven markdown.
  - `crates/hickory-cli/src/main.rs` — the `ingested` row in `cmd_lineage`.
  - `crates/hickory-agent/src/tools/mod.rs` — `origin_kind`.
  - `apps/web/src/api/types.ts` — the `ingested` member of
    `ProvenanceOrigin`; `apps/web/src/components/OutputEditorPane.tsx` and
    `apps/web/src/styles.css` — `cm-prov-ingested`.
  - Test: `crates/hickory-cli/tests/ingest_scaffold.rs::ingested_bytes_report_their_run_rather_than_reading_as_yours`
    drives the real binary end to end.
- Caveat requiring LLM review: the app draws an ingested ribbon in the lineage
  palette, since this is a lineage origin and not a fourth provenance family.
  Whether that reads clearly enough beside a literal ribbon has not been
  checked against a real document with forty ingested files.
