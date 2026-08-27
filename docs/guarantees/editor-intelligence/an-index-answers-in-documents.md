# An Index Answers In Documents

Given a project with a SCIP index, when a person asks where a name is used,
then every answer is a place they can actually edit:

- **A reference in a hand-written file is shown as itself.**
- **A reference in a GENERATED file is shown as the line of the DOCUMENT that
  wrote it**, mapped through the same lineage the reverse edit and the
  debugger use. That is the whole reason this product wants an index: its code
  lives in documents, and the files appear only when something weaves them.
- **A reference that cannot be mapped back is not shown, and is counted.**
  Synthetic bytes, or an agent's without a document span, have nowhere to send
  anybody — so nobody is sent. But the number left out is reported, because
  showing fewer results than exist with no sign anything was omitted is its
  own kind of lie.
- **The answer says how old it is.** An index is a cache; it is stale the
  moment you type. Every report opens with when it was built, and names how
  many of the files it covers have changed since. Staleness is not a second
  mechanism — it is the same question a recording answers, *did an input
  change*, asked with the same tool.
- **The index is never required.** Nothing here is on the path of any
  navigation feature that already worked, and the language server's live
  answer wins wherever the two disagree.

Producing an index is `hick index install` and `hick index build`: the same
sandbox, prefix and discovery-prefers-yours rule every other tool install
gets, writing into `.hick-cache/` which `hick init` already ignores.

---

Last LLM verification:
- Date: 2026-08-27
- Reviewer: Claude (Opus 5)
- Result: verified end to end against a real indexer
- Evidence:
  - `crates/hickory-cli/src/index_install.rs` — catalogue, per-language
    `Recipe`, discovery, and the honest `how_to_get`.
  - `crates/hickory-cli/src/index_read.rs` — reading the index, the
    `explain` step that maps occurrences through lineage or drops them, and
    `BuiltFrom` for staleness.
  - `hick index find <name> <language>` in `main.rs`.
  - **The licence question is settled**: `AGENTS.md`'s heading was amended
    from **MIT only** to **No copyleft** on 2026-08-27, because the rule
    beneath it was always the policy. `scip` is Apache-2.0 and its only
    transitive dependencies, `protobuf` and `protobuf-support`, are MIT — no
    copyleft anywhere in the chain.
- Test coverage:
  - `index_read::tests` — a hand-written hit is shown as itself, a generated
    hit is shown at its document, a generated hit with no lineage is dropped
    **and counted**, synthetic bytes have nowhere to send anybody, line
    arithmetic survives both ends, and an edited file is what makes an index
    stale.
  - `crates/hickory-cli/tests/index_reads_back_into_documents.rs` — the claim
    a unit test cannot make, with a real `scip-typescript` over a real weave:
    a definition living only in the generated `billing.ts` comes back as
    `billing.hick` line 5, the hand-written file's own reference survives
    beside it, and nothing goes unexplained. It borrows this repository's own
    installed indexer, so it runs on the machine most likely to be running
    it rather than skipping there.
- Verified by running, in a scratch project: `hick index find invoiceRef
  typescript` reported the definition at `billing.hick:6` (1-based) with
  `(via billing.ts)`, the two references in the hand-written `use.ts`, and —
  after editing `use.ts` — `1 of 2 indexed file(s) have changed since`.
  **The bug that found:** `generated_outputs` maps an output path to the
  document's **id**, not its path. Reading it as a path fails silently — the
  weave simply fails and every generated hit is dropped as unexplainable,
  which looks exactly like lineage having nothing to say. The first run
  reported "3 occurrences in generated files are not shown" and looked
  plausible.
- Caveat requiring review:
  - **Only TypeScript has been driven.** `scip-python` is in the catalogue on
    the same machinery and has never been installed or run here, and its
    argument shape was read from documentation rather than observed.
  - **`find` matches a symbol by substring**, because a SCIP symbol is a long
    structured string and a person types a name. That means `invoiceRef` also
    matches `invoiceRefFor`, and there is no way to ask for an exact symbol.
  - **`find` weaves every document that produces a hit**, synchronously, to
    get its lineage. On a project with many generated hits across many
    documents that is a lot of weaving, and it has only been run against two
    files.
  - **Nothing in the app uses this.** It is a CLI command; no editor
    surface, no right-click, no references panel.
  - **Definitions and references are told apart by one bit** of SCIP's role
    bitset. The other roles (import, read, write) are read and discarded, so
    a reference that is really an import is reported as an ordinary
    reference.
  - **Multi-line occurrences use their start line only.** SCIP ranges may
    span lines; only the first is mapped, which is right for going to a place
    and wrong if anything ever wants the extent.
