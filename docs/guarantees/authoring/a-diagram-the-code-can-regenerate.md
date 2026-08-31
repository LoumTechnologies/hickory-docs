# A Diagram The Code Can Regenerate

Given a folder of source code, when `hick diagram <path>` runs — at a
terminal, in CI, or inside a `hick:exec` cell, which is the intended home —
then a scene topology is deduced deterministically —
no model, no API key, no toolchain: tree-sitter compiled into the binary,
walking gitignore-aware — as nodes per file or per top-level directory and
edges from name-resolved references, **never** carrying a `layout`. Node
order is **path order, not walk order**: the same folder yields the same
topology on every machine, which is what makes an `<hick:expect>` around it a
pin rather than a record of one filesystem. And when
`hick diagram <path> --refresh <doc> --fragment <id>` runs, exactly that
`<hick:copy>` fragment's content is rewritten with the fresh topology, so a
derived diagram pasting it updates while the layout in the diagram's own body
is untouched by construction.

The reason: the third way a diagram gets authored (after a person and an
agent) must be a script anyone can run in CI, which means AI-free and
setup-free. Positions are excluded because they are the person's half of the
scene — a generator that writes them destroys an arrangement on every run —
and the refresh writes into a fragment, not into the diagram, because the
fragment is the recorded seam between generator and picture: the cell that
produced it can pin it, and the diagram that pastes it cannot disagree with
it. The output is honestly labelled structural: resolution is by name
(`hick-structure`'s documented limitation), a place to start looking rather
than a compiler's call graph — and SCIP in a user-built container inside a
`hick:exec` cell is the precise variant, emitting the same shape so the same
derived diagram consumes either.

---

Last LLM verification:
- Date: 2026-08-31
- Reviewer: Claude (Opus 5)
- Result: verified, after fixing the half of "deterministically" that was not
- What changed: the node order was the **walker's**, and
  `ignore::WalkBuilder` yields entries in whatever order the directory hands
  back — so the topology differed between machines and filesystems. The
  guarantee's own worked example pinned `lib` before `app`, which is what the
  author's machine produced and not what anything else did, so
  `examples/architecture-that-draws-itself.hick` failed its own
  `<hick:expect>` with a diff that was correct about the shape and wrong only
  about the order. A generator a document is invited to freeze cannot be one
  entry ahead of readdir. `generate_topology` now sorts by path before
  resolving links (`crates/hickory-cli/src/diagram.rs`), and the example's pin
  and woven `.md` were regenerated to the stable order.
- Caveat: this had never been caught because the example reached CI only
  after a long red streak, and until now no green run had executed it. See
  also `docs/guarantees/execution/a-cells-own-path-belongs-to-its-project.md`
  for the other half of why it could not run there — `hick diagram` is invoked
  from inside a confined cell, which cannot see a binary in `target/debug`.

Previous verification:
- Date: 2026-08-25
- Reviewer: Claude (Fable 5)
- Result: verified
- Evidence:
  - Generator: `crates/hickory-cli/src/diagram.rs` — `generate_topology`
    (ignore::WalkBuilder, 512 KB cap, `hick_structure::analyze`/`resolve`,
    dir/file grouping, self-edges dropped, reference counts as edge labels),
    `render` (scene JSON or the same `to_mermaid` downgrade the weave uses),
    `refresh_fragment` (rewrites only the named fragment's bytes).
  - CLI: `cmd_diagram` in `crates/hickory-cli/src/main.rs` — `--format`,
    `--group`, `--refresh`, `--fragment`; misuse and an empty folder get
    sentences with next steps, per the user-facing-errors rules.
  - There is deliberately NO app verb and no API route for this (a
    "Generate from code" toolbar button and `GET /api/diagram` existed
    briefly and were removed 2026-08-25): the way a generated diagram
    enters a document is a `hick:exec` cell running the generator with an
    `<hick:expect>` pinning its output, a fragment stating it, and a
    derived diagram pasting the fragment — the exec block IS the source,
    and a second, unpinned path would be a way to draw that nothing checks.
  - Worked example: `examples/architecture-that-draws-itself.hick` — the
    code, the expect-pinned scan, the fragment, the derived graph diagram,
    and the SCIP-in-a-user-container variant as literal text (the document
    uses the `h:` prefix so the `hick:` snippet stays prose). Ran end to end
    on this machine: `hick run` then `hick test` → verified, exit 0.
- Caveats — what LLM review could NOT establish:
  - The SCIP variant is shown, not executed — no `scip-tools:local` image
    exists here to run it, which is the point of saying the image is the
    user's to build. Its jq query is illustrative and untested against a
    real SCIP index.
- Test coverage: `crates/hickory-cli/tests/diagram_cli.rs` (4 e2e tests:
  topology with no layout, mermaid downgrade parity, refresh rewriting the
  fragment while the layout survives and the refreshed document weaving, the
  empty-folder message); `crates/hickory-cli/src/diagram.rs::tests` (4 unit
  tests including a real two-directory walk).
