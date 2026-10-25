# Literate Surface UI Audit

This records the 2026-10-24 request to inventory every user-facing way Hickory
shows literate content and align them with the established compact,
editor-first direction. It complements, rather than replaces,
`authoring/the-editor-keeps-its-space.md` and
`specs/freeform/shell-layouts.md`.

## Direction applied

- The authored document is primary. Persistent chrome is limited to identity,
  view state, and current work; occasional actions use a local or overflow
  menu.
- Controls appear with the thing they affect: cell controls on the rail,
  document controls in the pane header, comparison and lens controls inside
  their own view.
- Compact is the default for editing. Larger spacing is appropriate only for a
  deliberately reading-oriented, read-only surface.
- The visual system is restrained and tool-grade: system text, mono source,
  cool dark ground, quiet borders, and semantic accent colour rather than
  decorative colour.

## Inventory

| Surface | Role | Consistency decision |
| --- | --- | --- |
| Desktop document and Untitled editor | Primary authored Markdown/Hick document | Compact `DocumentEditor`; header/menu and rail placement are the baseline. |
| Generated-file tab | Woven output that can map edits back to its source | Shares the desktop pane/header and editor chrome; output-specific controls remain local. |
| Literate view / representation | Personal arrangement over files or an owning document | Reuses `DocumentEditor`, the pane header, and local comparison/tool strips. Its source and write destination remain explicit. |
| Change review and commit reading | Captured/historical comparison | Intentionally read-oriented and more explicit about evidence; actions stay inside the review rather than global chrome. |
| Worktree merged lens | Synthetic comparison across worktrees | Intentionally denser monospace comparison, visibly marked as a lens; region controls remain beside their regions. |
| Lineage pipeline | Project-level stage overview | An overview rather than an editor; compact stage columns and scoped warnings are appropriate. Opening content returns to the owning document/file. |
| Agent conversation and session lens | A session document presented as a conversation | Reuses document rendering and provenance; conversation-specific controls stay in that pane. |
| Browser/iframe `DocumentEmbed` and demo editor | Host-owned embedded authored or read-only document | Uses the same parser/editor conventions. Its self-contained palette now matches the desktop cool dark token family instead of the old green/brown theme. |
| Published snapshot viewer | Historical public reading | Keeps a wider prose measure and evidence disclosures intentionally: it is read-only publication, not an authoring workspace. It imports the shared product theme. |

## Findings and changes

The desktop surfaces already share `DocumentEditor`, pane-header slots,
density preferences, and the global tokens. The only direct visual contradiction
found was `embed/embed.css`: its independent palette was green/brown while the
desktop default is cool dark. The embed must remain self-contained because a
host need not provide Hickory CSS, so it now carries private tokens that mirror
the desktop palette rather than inheriting unscoped variables.

The audit does not require a uniform geometry across every surface. Comparison,
lineage, conversation, and publication views answer different questions; their
exception is valid only while they retain their local identity and do not add
permanent workspace chrome.

Follow-up visual review removed the fixed 240px embedded-editor minimum and
the demo host's grid stretch, so short embeds now take their content height.
The snapshot viewer now keeps its verification outcome visible while putting
long provenance, requirement, and environment receipts behind one compact
"Snapshot details" disclosure. A bad publication URL now explains that it
returned non-snapshot data and gives a recovery path instead of exposing a raw
JSON parser exception.

---

Last LLM implementation review:

- Date: 2026-10-24
- Reviewer: Codex
- Result: partially verified. The inventory was traced through the listed
  surface components and their style ownership. Desktop visual consistency was
  inspected previously; the browser embed palette and density, plus snapshot
  detail and error presentation, were normalized in this change.
- Evidence: `apps/web/src/views/workspaceTabs.tsx`,
  `RepresentationPane.tsx`, `DocumentReading.tsx`, `CommitReading.tsx`,
  `MergedView.tsx`, `LineageView.tsx`, `ConversationEditor.tsx`,
  `SessionLens.tsx`, `embed/DocumentEmbed.tsx`,
  `embed/PublishedSnapshot.tsx`, `embed/SnapshotPage.tsx`, `styles.css`,
  `embed/embed.css`, and `embed/snapshot.css`.
- Executed checks: `cd apps/web && npm run typecheck && npm test --
  src/embed/DocumentEmbed.test.tsx src/embed/PublishedSnapshot.test.ts
  src/embed/PublishedSnapshot.controls.test.tsx src/views/RepresentationPane.test.tsx
  src/views/DocumentReading.controls.test.tsx src/views/CommitReading.controls.test.tsx
  src/views/HistoryLens.test.tsx src/views/SessionLens.test.tsx
  src/views/ConversationEditor.test.tsx src/views/workspaceTabs.test.tsx
  --maxWorkers=2` passed: TypeScript plus 65 tests in 10 files.
- Follow-up executed checks: `cd apps/web && npm run typecheck && npm test --
  src/embed/SnapshotPage.test.tsx src/embed/PublishedSnapshot.controls.test.tsx
  src/embed/PublishedSnapshot.test.ts src/embed/DocumentEmbed.test.tsx
  --maxWorkers=2` passed: TypeScript plus 22 tests in 4 files. The live mock
  was also visually checked for compact browser embed cards and the friendly
  non-JSON publication recovery state.
- Human judgment: the desired direction was supplied by the project owner; no
  human has reviewed this exact implementation pass yet.
- Coverage gap: this records implemented first-party surfaces. Arbitrary host
  CSS and every responsive/native display configuration remain outside this
  audit.
