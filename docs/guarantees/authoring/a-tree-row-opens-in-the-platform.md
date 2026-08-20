# A Tree Row Hands Its File To The Rest Of The Machine

Given a folder open in the app, when a row in the file tree is right-clicked —
a file, a directory, a binary the app cannot display, or the folder itself —
then a menu offers the path as text (absolute, relative, and the bare name),
the path shown in this desktop's own file manager under that manager's real
name, and, for a file, the path opened in whatever program this machine
already opens that kind of file with; and when the named path is not inside
the open folder, then nothing is started and the refusal says why.

A notes folder is an ordinary directory on an ordinary computer. The app is
one way in, not the only one: the terminal wants a path pasted into it, a
photo wants the system viewer, and a `.psd` wants the program that made it. A
tree that can only open what it can render makes the user go find the folder
by hand, which they can do — so the menu is not new power, only the removal of
a detour.

Three properties hold it up:

1. **The handover is the server's, not the shell's.** `hick up` and the
   desktop app are one engine over one machine, and it is *that* machine's
   file manager the user means. Putting it in the Tauri shell would give the
   feature two implementations, one of them untested.
2. **Only paths inside the open folder are handed over.** Traversal, absolute
   paths, and symlinks that leave the root are refused before any process is
   started. "Open anything you name in your default program" is a much larger
   surface than "open something in this folder", and only the second is the
   feature.
3. **What cannot be honest is not offered.** "Copy absolute path" is absent
   when the server did not say where the folder is, rather than present and
   quietly copying a relative one; a directory offers no default-program item;
   and a failure to start the platform's program is reported in the pane
   rather than looking like a menu item that does nothing.

## Boundary

Starting the program is where the promise ends. These routes spawn and do not
wait: a file manager outlives the click that opened it, so only a failure to
*start* is detectable, and that is the only failure reported. What the chosen
program then does with the file — including nothing, because the user has no
handler for that type — is between them and their desktop.

On Linux, *selecting* an entry needs the freedesktop `FileManager1` interface,
which is not always installed. When it is missing the containing folder is
opened instead: the folder is what the user asked to see, and a degraded
answer beats an error.

The path is copied, not the file. There is no "copy file" or "copy URL" item,
because there is no second machine in this product for a URL to mean anything
to.

---

Last LLM verification:
- Date: 2026-08-20
- Implementation: `crates/hickory-cli/src/serve/reveal.rs` — `reveal` and
  `open_external` (routed as `POST /api/reveal` and `POST /api/open-external`
  in `crates/hickory-cli/src/serve/mod.rs`); `resolve_in_root` canonicalizes
  under the served root and refuses traversal (400), a missing entry (404),
  and a symlink that escapes (403) before anything is spawned; `spawn_reveal`
  is `open -R` / `explorer /select,` / `FileManager1.ShowItems` with an
  `xdg-open` fallback on the parent directory; `spawn_open` is `open` /
  `rundll32 url.dll,FileProtocolHandler` / `xdg-open`; `file_manager_name`
  supplies the menu's wording. `crates/hickory-cli/src/serve/api.rs::files`
  adds `root_path`, `separator`, and `file_manager` to `GET /api/files`, which
  is where the absolute path in the menu comes from.
- Frontend: `apps/web/src/shell/treeMenu.ts` computes the items (and rewrites
  separators for Windows in `absolutePath`); `apps/web/src/shell/TreeContextMenu.tsx`
  renders them at the pointer and closes on Escape, an outside click, a
  scroll, or a resize; `apps/web/src/shell/FolderTreePane.tsx` puts
  `onContextMenu` on every row kind — directory, file, inert binary, and the
  root header — and shows a failed call in `.folder-tree__notice`.
- Test coverage: `apps/web/src/shell/treeMenu.test.ts` covers the item sets
  (file, directory, root), the Windows join, and the missing-`root_path` case
  from property 3; `apps/web/src/shell/FolderTreePane.test.tsx` —
  "the right-click menu" drives real right-clicks through the pane and asserts
  the clipboard writes, the two POSTs, the menu on an inert row, the error
  notice, and the closing behavior. `crates/hickory-cli/tests/serve_reveal.rs`
  drives the refusals over a real socket; the unit tests in `reveal.rs` cover
  the accepting half of `resolve_in_root`, symlink escape included.
- Caveat requiring review: **no test starts a real file manager.** A passing
  suite proves the refusals, the wiring, and the request shapes — not that
  `explorer /select,` selects the file on Windows or that `FileManager1` is
  reachable on a given Linux desktop. Those three command lines are verified
  by reading, and are the part to re-check by hand on each platform before a
  release advertises it.
