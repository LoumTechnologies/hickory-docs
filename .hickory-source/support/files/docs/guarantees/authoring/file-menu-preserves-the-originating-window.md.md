<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/authoring/file-menu-preserves-the-originating-window.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `54a2b81e77fc7a0af765b6d1b387d0699b2d84b7543aefc848500b189c0380ef`.

<book:entry path="docs/guarantees/authoring/file-menu-preserves-the-originating-window.md">
# The File Menu Preserves The Originating Window

Given a Hickory Docs window with no folder open, File → Open Folder or a
recent folder attaches the selected folder to that same window. The native
window and editor origin stay in place; existing tabs, unsaved unnamed notes,
recovery drafts, and explicitly opened files stay available. Files appears
beside the editors. This also applies to a window launched on a single file.

When the window already has a folder, opening another folder creates another
window and keeps the original session open. A cancelled picker or failed folder
startup leaves the original session untouched.

Given that same window, when the person chooses File → Open File or a recent
file, then the selected text file opens as a tab in that window, including when
it is outside the window's folder or no folder is open. The current folder and
other tabs remain in place. Markdown files use document rooms; other text files
use the existing editor and hash-checked save path. File → New Project retains
its explicit choice between this window and a new window.

An explicit `POST /api/open-file` validates a selected file as UTF-8 text below
the editor's size cap. It registers Markdown in the document index or grants
plain-file editing of that exact canonical path. Ordinary file reads and writes
still refuse absolute paths that were not explicitly opened, traversal, and
symlink escapes. Opening files does not broaden the workspace file tree.

---

Last LLM verification:
- Date: 2026-10-08
- Reviewer: Codex
- Result: verified by automated checks and implementation review.
- First-folder validation: `just check-desktop` passed formatting and Clippy;
  `just test-first-folder` passed 10 live desktop HTTP tests and 5 frontend
  tests; `just test-startup-web` passed 57 tests with the real editors. These
  include preserving existing file reads/writes and keeping unselected absolute
  files inaccessible after attaching the first folder. The updated app built,
  passed signature verification, and was installed locally.
- Original validation: `just test-web` passed all 2,017 tests; the real HTTP
  `open_selected_file` test passed for folderless and folder sessions;
  `just check-desktop` passed; the native app built, passed signature
  verification, and was installed locally. Native menu/picker clicks were
  not exercised by automation.
- Evidence: desktop `lib.rs::open_file` forwards to its own page;
  `open_folder_window` attaches the first folder through `Session::attach_folder`
  and starts another process for subsequent folders without exiting this one;
  `folderPane.ts::openSelectedFile` and `serve/plain_file.rs` register and open
  the chosen text file in the current session.
- Tests: `just test-first-folder` exercises a live folderless editor attaching
  its first folder on the same HTTP origin, preserving layout, drafts, old
  Markdown access and plain-file reads/writes; it also checks failed attachment
  and the second-folder decision. The frontend test preserves editor tab
  identities and unsaved text when Files appears. `just test-file-menu` includes real HTTP opening and saving outside
  files, Markdown document access, unchanged folder visibility, and rejection
  of an absolute plain-file path before explicit opening. Frontend tests verify
  the selected document/file routing and error notices.
- Boundary: native menu/picker behavior is reviewed and typechecked; these
  automated checks do not drive the operating system's file picker.
</book:entry>

</book:doc>
