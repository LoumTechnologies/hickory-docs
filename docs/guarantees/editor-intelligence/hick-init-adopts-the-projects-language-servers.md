# `hick init` Adopts The Project's Own Language Servers

Given a repository whose editor configuration already names a language server
for a language — `"python.languageServer": "Pylsp"` in `.vscode/settings.json`,
a `language_servers` list in `.zed/settings.json`, a `[language-server.…]`
entry bound to a `[[language]]` in `.helix/languages.toml` — when `hick init`
runs in that repository, then it records that server's command in
`.hick-lsp.json`, and `hick-lsp` spawns **that** server for `hick:file` blocks
of that language instead of its own built-in default.

The reason is drift, which is the thing this product exists to remove. A
`hick:file` block holds the same code as the file it generates. If the block
is checked by `pyright` while the generated `.py` is checked by `pylsp`, one
buffer disagrees with the other about code that is byte-for-byte identical,
and the document becomes the surface a person stops trusting.

Three rules bound it:

- **Never overwrite.** An entry already present in `.hick-lsp.json` is left
  exactly as it is, whatever the editor config now says. The file is written
  once by `hick init` and belongs to whoever edits it afterwards, so re-running
  init can never undo a hand-tuned command. A repository with no editor
  configuration at all gets no file written.
- **First source wins per language**, in the order VS Code, Zed, Helix. Two
  editors configured for the same language is a repository that has already
  made a choice twice; picking one deterministically is better than merging
  them into something neither editor does.
- **Adoption is a copy, not an endorsement.** `hick init` does not check that
  the command exists, and `hick-lsp` does not fail when it cannot spawn it —
  see [lsp-channel-degrades-never-errors](lsp-channel-degrades-never-errors.md).

The second half of the same step is registration: `hick init` writes the
project-local configuration that points an editor at `hick-lsp` for `*.hick`
**only where such a file can actually do the job** — `.helix/languages.toml`
(complete on its own) and `.vscode/settings.json` (which still needs a generic
LSP-client extension, and says so). Zed can only gain a language server through
an extension and Neovim has no project-local LSP registration at all, so for
those two `hick init` prints the step instead of writing a file that would do
nothing. Printing beats a plausible-looking file that never starts anything.

---

Last LLM verification:
- Date: 2026-08-13
- Reviewer: Claude (Opus 5)
- Result: verified for the discovery, precedence, and idempotence rules;
  editor-side registration verified as file contents, not as observed editor
  behaviour
- Evidence:
  - `crates/hickory-cli/src/editor_lsp.rs` — `discover_servers` calls
    `from_vscode`, `from_zed`, `from_helix` in that order and keeps the first
    hit per language; `write_server_overrides` skips a language already
    present in `.hick-lsp.json` and returns `false` (writing nothing) when
    there is neither an existing file nor anything to adopt.
  - `crates/hick-lsp/src/server_config.rs` — `lsp_command` in
    `crates/hick-lsp/src/child_lsp.rs` consults `command_for` before its
    built-in table, so an override also enables a language the built-in table
    does not know. Load is process-global and first-call-wins, from
    `did_open` in `backend.rs`: one `hick-lsp` process serves one workspace.
    A missing or malformed file leaves the defaults in place.
  - Unit tests in `editor_lsp.rs`:
    `adopts_the_python_server_vscode_already_chose`,
    `adopts_from_zed_language_server_lists`,
    `adopts_from_helix_language_config`,
    `writing_overrides_never_clobbers_a_hand_edited_entry`,
    `no_editor_config_writes_no_override_file`,
    `helix_block_is_idempotent_and_preserves_user_content`,
    `vscode_settings_are_added_without_disturbing_existing_keys`,
    `jsonc_comments_and_trailing_commas_are_tolerated`. Tests in
    `server_config.rs` cover parsing and nearest-config-wins.
  - Run end to end on this machine: a fresh `git init` repository with
    `.vscode/settings.json` containing a comment, a trailing comma, and
    `"python.languageServer": "Pylsp"` produced
    `.hick-lsp.json` with `python → pylsp`, reported the source file it came
    from, and left the existing VS Code keys untouched while adding the
    `files.associations` and `glspc.*` entries.
- Caveats — what LLM review could NOT establish:
  - **No editor has been driven end to end.** That Helix starts `hick-lsp`
    from the written `.helix/languages.toml`, that the glspc extension reads
    the VS Code keys, and that the Zed dev extension finds `hick-lsp` on PATH
    are all read from those tools' documented behaviour, not observed here.
  - The Helix reader is a line scanner, not a TOML parser: it handles the
    flat `command`/`args`/`language-servers` shapes those files are written
    in, and will miss an inline-table or multi-line-array spelling. Missing
    one degrades to the built-in default, which is the pre-existing
    behaviour.
  - Pylance cannot be spawned outside VS Code, so a repository asking for it
    adopts `pyright-langserver` — the server Pylance is built on, but not the
    same analyzer. A user who cares will see the command in `.hick-lsp.json`
    and can change it.
- Test coverage: the unit tests listed above. Nothing yet exercises an actual
  editor; that is the gap this section names.
