# An Index Is Installed And Spawned, And Read By Nothing

Given a project in a language with a SCIP indexer, when a person runs
`hick index install <language>` and then `hick index build <language>`, then
an indexer is fetched under the same confinement every other tool install
gets, found the same way, spawned, and its index written to `.hick-cache/` —
and **nothing in the product reads it**, which is said every time rather than
implied.

- **SCIP is in addition to LSP, never in place of it.** A language server
  knows the file you are looking at including the parts you have not saved;
  an index knows the whole project as it was when it was built. `hick index
  list` says so before it says anything else.
- **The same install machinery**, so the same refusals: confined to
  `.hick-cache/indexers`, never automatic, never without a sandbox, and a copy
  you installed yourself always wins.
- **Never index into the repository.** `.hick-cache/` is where a build
  artifact of this machine belongs, and `hick init` already ignores it.
- **Never name a command that does not exist.** Every language the report
  mentions either names a real `hick index install` target or names the way
  that ecosystem actually installs its indexer.

**The half that is not built is the reading half.** Until 2026-08-27 what
blocked it was a contradiction — reading an index means linking the `scip`
crate, which is Apache-2.0: permissive, so it passed `AGENTS.md`'s *rule*
about copyleft and contradicted its **MIT only** *heading*. **That is
resolved: the heading was amended**, because the rule beneath it was always
the policy. Apache-2.0 is allowed and the crate may be linked. Nothing blocks
the reading half now except that nobody has written it. The indexers
themselves never needed the decision — they are **spawned**, exactly as
language servers and debug adapters are, and nothing links them.

---

Last LLM verification:
- Date: 2026-08-27
- Reviewer: Claude (Opus 5)
- Result: verified by running the whole loop
- Evidence: `crates/hickory-cli/src/index_install.rs` — the catalogue, the
  per-language `Recipe` for invoking one, `discover` (project first, then
  `PATH`), and `how_to_get`. The CLI is `hick index list|install|build` in
  `main.rs`.
- Verified by running (2026-08-27), in a scratch TypeScript project:
  `hick index install typescript` fetched `@sourcegraph/scip-typescript`
  through the sandbox into `.hick-cache/indexers`, and `hick index build
  typescript` discovered that copy, spawned it, and wrote a real 954-byte
  `.hick-cache/index/typescript.scip`.
- Test coverage: `index_install::tests` — every installable language has a
  recipe to run it (installing an indexer nothing can invoke is a download
  that does nothing), and **every command the report names is a command that
  exists**. The second one earned its keep immediately: the first draft told
  a JavaScript project to run `hick index install javascript`, which fails
  with "no installer for 'javascript'" because the catalogue is keyed by
  ecosystem. A weaker check that only asserted *some* install command was
  named would have passed.
- **The licence decision was taken by Nate on 2026-08-27: amend the heading.**
  `AGENTS.md`'s **MIT only** became **No copyleft**, which is what the rule
  beneath it always said — the shorthand had drifted from the thing it was
  shorthand for. Apache-2.0 is permissive and allowed, so the `scip` crate may
  be linked and the reading half is unblocked.
- Caveat requiring review:
  - **Nothing consumes an index, so no navigation feature is better than it
    was.** This is the whole of the reading half and it is untouched.
  - **The mapping the spec insists on does not exist yet.** "A reference that
    cannot be mapped back through lineage to a document span is not shown" is
    a rule about a feature that has not been built; nothing here shows a
    reference at all, so nothing yet obeys or violates it.
  - **Staleness is unaddressed.** The spec is clear the signal should be the
    same input digest recordings already use. `hick index build` writes an
    index with no record of what it was built from, so nothing can say
    "indexed at 14:02" or know that it is stale.
  - **Only `scip-typescript` has been run.** `scip-python` is in the catalogue
    on the same npm machinery and has never been installed or spawned here.
  - **The recipes are a guess about argument shape for everything but
    TypeScript.** `scip-python`'s `index .` / `--output` was read from its
    documentation, not observed.
