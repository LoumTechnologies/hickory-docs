<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/execution/doc-saves-commit-to-project-git.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `f62c38270fffa2cdd2abcf63c8f8607da618831bc776a03636fd10e0b7871897`.

<book:entry path="docs/guarantees/execution/doc-saves-commit-to-project-git.md">
# Every Doc Save Is a Commit in the Project's Git Repository

Given a project, when a document is created, saved via `PUT /api/docs/:id`,
or persisted from the collaborative CRDT session, then the server writes the
file into the project's plain git repository under `GIT_DATA_DIR` and
commits it (one commit per save; unchanged content is a no-op) — git is the
durable truth for `.md` sources, and run checkouts are seeded from it.

---

Last LLM verification:
- Date: 2026-08-05
- Reviewer: Claude (Fable 5)
- Result: verified
- Evidence: `apps/server/src/gitstore.rs` (`save_file`: write → `git add` →
  commit only when the index changed; path validation rejects traversal and
  `.git` components) is called from doc create, `PUT /api/docs/:id`
  (`apps/server/src/routes/docs.rs`), and the WS debounced persist
  (`apps/server/src/ws.rs::persist_now`); runs seed their temp dir via
  `seed_checkout`. Plain git CLI was chosen over the vendored
  content-addressed `GitVersionStore` so operators get an inspectable
  `git log` per project.
- Test coverage: `apps/server/tests/integration.rs` —
  `auth_projects_docs_and_git` asserts commit count equals the number of
  saves and the working-tree file matches the last saved source.
</book:entry>

</book:doc>
