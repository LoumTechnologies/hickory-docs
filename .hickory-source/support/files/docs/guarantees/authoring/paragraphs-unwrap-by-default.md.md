<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/authoring/paragraphs-unwrap-by-default.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `f3de5fc10908d1d0bdc9c80c7db26d8e10cb12af5c9d4f3178092aea5ca84439`.

<book:entry path="docs/guarantees/authoring/paragraphs-unwrap-by-default.md">
# Markdown Paragraphs Unwrap By Default

Given an editable document in the Document view, when it opens, receives source
from the live room, or receives pasted or dropped text, then ordinary Markdown
paragraphs have their soft line breaks replaced with spaces. The editor wraps
the resulting paragraph visually at the ruler's measure.

A seeded Untitled introduction keeps its initial bytes through opening and
initial sync, so opening the app does not mark it edited. Paragraph reflow
starts after the person's first edit, including pasted text.

Settings → Editing → **Unwrap Markdown paragraphs** is on by default and can
be turned off. The choice persists per browser profile, like word navigation,
and is checked at each reflow. Turning it off does not restore past edits.

Blank paragraph boundaries, explicit Markdown hard breaks (two spaces or a
backslash), lists, quotes, tables, code, HTML blocks, YAML frontmatter, multi-line
inline code and links, display math, Hickory tags, and verbatim element bodies
are preserved. Prose inside a Hickory document container can unwrap. Read-only
documents are never reflowed. Typing Enter is left alone; reflow is an ordinary
source edit with mapped selections and a separate Undo step. Undo does not
immediately reapply the reflow.

---

Last LLM verification:

- Date: 2026-10-05
- Reviewer: Codex
- Result: verified
- Evidence: `apps/web/src/editor/unwrapParagraphs.ts` uses the Markdown parser
  for paragraph boundaries and `parseHickDoc` for tags and verbatim bodies.
  `DocumentEditor.tsx` installs the extension beside history and the Yjs binding;
  `lib/unwrapParagraphs.ts` persists the choice, exposed by `SettingsView.tsx`.
- Test coverage: `editor/unwrapParagraphs.test.ts` exercises preservation,
  mount/sync/paste, read-only, selection mapping, Undo and Enter;
  `editor/DocumentEditor.test.tsx` verifies the installed extension reflows
  synced source, reports the edit, supports Undo, and respects the opt-out;
  `views/SettingsView.test.tsx` exercises the default and persisted opt-out.
- Validation: `just test-agent-web` passed typechecking and all 1,897 tests
  in 182 files; `just check-file-length` and `git diff --check` passed.
- Scope: generated output panes and CLI ingestion are not reformatted. Unmarked
  newlines in an ordinary Markdown paragraph are soft breaks; intentional verse
  should use explicit hard breaks or disable the setting.
</book:entry>

</book:doc>
