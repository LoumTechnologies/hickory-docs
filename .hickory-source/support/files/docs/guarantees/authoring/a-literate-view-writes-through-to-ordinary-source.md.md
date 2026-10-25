<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/authoring/a-literate-view-writes-through-to-ordinary-source.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `aff7641fc1d8b3502f6183304fdd1f30eb19ff5c6aac00cd4c7a08590c9ecb7a`.

<book:entry path="docs/guarantees/authoring/a-literate-view-writes-through-to-ordinary-source.md">
# A literate view writes through to ordinary source

Given ordinary UTF-8 files in a repository, opening a source-backed literate view
or organizing its exact code slices changes no repository files. Every selected
file reconstructs byte-for-byte. A view's source does not become a document in
the repository index.

When the person or ACP saves a code edit, the selected files receive the compiled
bytes only if the view revision and all its recorded backing bytes still match.
Prose and reading order remain local. Invalid assembly, an undeclared output,
execution tags, path escape, and stale backing refuse publication. External edits
refresh through exact provenance or require a new reading; they never overwrite
the repository during refresh. Code changes mark explanatory prose stale.

Keeping the reading writes a `.md` document and reconstruction metadata outside
the repository. A restart can reopen it. View ACP sessions also stay in personal
storage. Persistent document-backed views update the original document and its
live room; the existing weave loop publishes outputs.

---

Last LLM verification:

- Date: 2026-10-03
- Reviewer: Codex
- Result: partially verified
- Evidence: `serve/representation.rs`, `representation_tools.rs`,
  `representation_store.rs`, `serve/acp/mcp.rs`, and `views/RepresentationPane.tsx`.
- Test coverage: `tests/literate_views.rs` exercises exact reordered assembly,
  Unicode, whitespace, missing final newline, empty/deleted files, write-through,
  conflicts, refresh, persistence and refusals. `tests/serve_acp.rs` drives a real
  ACP protocol peer through read/organize/edit with an external session record.
  `tests/engine_lifecycle.rs` checks the actual engine write gate. Browser flow:
  `e2e/literate-editor.spec.ts`.
- Limits: tests do not verify AI prose accuracy or crash-atomic multi-file writes.
</book:entry>

</book:doc>
