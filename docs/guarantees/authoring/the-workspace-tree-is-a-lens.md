# The Workspace Tree Is A Lens

Given files and other objects associated with folders, when the workspace tree
is shown, then each object is a node at its place; expanding and editing a node
asks its source through capabilities that node declared; the tree has no save
path or serialized editable form of its own; and a batch spanning independent
sources reports an outcome per node rather than claiming atomicity.

The first implemented non-file node is a terminal session. Window, work-item,
review, notification, and multi-edit nodes remain desired behavior and are
specified by `docs/specs/freeform/the-workspace-tree.md`.

---

Last LLM verification:

- Date: 2026-09-14
- Reviewer: Codex (GPT-5)
- Result: partially verified
- Evidence: filesystem rows and their semantic operations are in
  `apps/web/src/shell/FolderTreePane.tsx`, `dired.ts`, and `useDired.ts`;
  terminal sessions are projected by `relativeCwd` and `placeSessions` and
  rendered as visible `TerminalRows`. `apps/web/src/lib/workspaceTree.ts`
  defines stable source identity, node kinds, freshness, and semantic
  capabilities for later adapters. The provider-neutral server contract and
  every other non-file node are not implemented yet.
- Test coverage: `FolderTreePane.test.tsx` covers files, semantic dired verbs,
  terminal placement, activation, collapsed aggregation, and exclusion outside
  the root. No test can cover the unimplemented provider and window slices yet.
