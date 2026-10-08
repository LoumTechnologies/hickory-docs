# The File Menu Preserves The Originating Window

Given an open Hickory Docs window, when a person chooses File → Open Folder or
a recent folder, then the selected folder opens in another window and the
original window stays open, with its tabs, unsaved text, and terminals.

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
- Validation: `just test-web` passed all 2,017 tests; the real HTTP
  `open_selected_file` test passed for folderless and folder sessions;
  `just check-desktop` passed; the native app built, passed signature
  verification, and was installed locally. Native menu/picker clicks were
  not exercised by automation.
- Evidence: desktop `lib.rs::open_file` forwards to its own page;
  `open_folder_window` starts another process without exiting this one;
  `folderPane.ts::openSelectedFile` and `serve/plain_file.rs` register and open
  the chosen text file in the current session.
- Tests: `just test-file-menu` includes real HTTP opening and saving outside
  files, Markdown document access, unchanged folder visibility, and rejection
  of an absolute plain-file path before explicit opening. Frontend tests verify
  the selected document/file routing and error notices.
- Boundary: native menu/picker behavior is reviewed and typechecked; these
  automated checks do not drive the operating system's file picker.
