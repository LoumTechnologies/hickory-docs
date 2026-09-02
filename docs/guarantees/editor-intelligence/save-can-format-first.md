# Save Can Format First, With The File's Own Formatter

Given a file or document with a language server behind it, when the person
presses Shift+Alt+F, then the buffer is formatted by whatever formatter that
language server offers — rustfmt through rust-analyzer, the Python server's
own, prettier through the TypeScript server — and the edits are applied as
one user edit; and given the per-user setting **Format on save** is on, when
the person chooses Save or Save All, then the focused buffer is formatted
first and the formatted text is what is saved. A `hick:file` block in a
document is formatted in the block's own indentation. The setting is off by
default.

Off by default because this product's documents are prose as much as code,
and a formatter rewriting a document nobody asked it to is a surprise. On
because a person leaving an IDE expects Save to mean "and tidy it".

Three properties hold it up:

1. **The edits come back in document coordinates and indentation.**
   `hick-lsp` forwards `textDocument/formatting` to each block's child and
   `translate_edits` maps ranges through the position map and puts the
   block's indentation back after every inserted newline — bare only for a
   trailing newline at column 0, where the next line's indentation is
   outside the range. A block with an edit that cannot be mapped is left
   alone, never half-formatted.
2. **Applied back to front, as a user edit.** `formatDocument` sorts the
   edits by descending offset so each lands where the server meant it, and
   dispatches with `userEvent: "input.format"` so a pane that saves only what
   a person did saves this.
3. **Save reaches the formatter through a facet.** `formatter` is provided by
   `lspFeatures`; `formatView` reads it from any editor; the workspace's Save
   and Save All call it on the focused editor when `formatOnSave()` is on,
   then save everything. The setting persists in `ui.json` as
   `format_on_save` beside the window title.

## Boundary

Save All formats the focused buffer only: formatting every open buffer on
one keystroke is a change to files nobody is looking at. A file with no
formatter says so in its toolbar and is saved as it is.

---

Last LLM verification:
- Date: 2026-09-02
- Reviewer: Claude (Fable 5.1)
- Result: verified
- Evidence: `crates/hick-lsp/src/backend.rs` (`formatting`, `translate_edits`,
  `reindent`; `document_formatting_provider` advertised);
  `crates/hickory-cli/src/serve/lsp_bridge.rs` declares `formatting` to the
  server; `apps/web/src/lsp/client.ts` `formatting`;
  `apps/web/src/lsp/cmLspFeatures.ts` (`formatter`, `formatView`,
  `formatDocument`, the Shift+Alt+F binding);
  `apps/web/src/lib/formatOnSave.ts`; `apps/web/src/views/WorkspaceView.tsx`
  `formatFirst` on `save` and `save-all`; `crates/hickory-cli/src/serve/mod.rs`
  `UiStore::format_on_save` and `api.rs` `put_settings_ui`;
  `apps/web/src/views/SettingsView.tsx` the toggle.
- Test coverage: `crates/hick-lsp/src/backend.rs::tests`
  (`a_formatting_edit_gets_the_blocks_indentation_back`,
  `a_plain_files_formatting_edit_is_untouched`,
  `an_edit_that_cannot_be_mapped_withholds_the_whole_answer`);
  `crates/hickory-cli/tests/lsp_languages.rs`
  (`a_plain_rust_file_formats_with_rustfmt_through_the_language_server`, real
  rust-analyzer and rustfmt); `apps/web/src/lsp/cmLspFeatures.test.ts`
  ("formatting"); `apps/web/src/views/SettingsView.test.tsx` (the toggle).
- Caveats: the Save path's call into `formatFirst` is exercised by review,
  not by a test; a browser test that presses Ctrl+S with the setting on is
  still to write.
