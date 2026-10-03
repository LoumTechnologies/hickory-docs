# Markdown Paragraphs Unwrap By Default

Given an editable document in the Document view, when it opens, receives source
from the live room, or receives pasted or dropped text, then ordinary Markdown
paragraphs have their soft line breaks replaced with spaces. The editor wraps
the resulting paragraph visually at the ruler's measure.

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

- Date: 2026-10-03
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
