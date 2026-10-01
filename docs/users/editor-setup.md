# Editor setup: diagnostics inside your `.md` documents

*For engineers who edit `.md` documents in Zed, VS Code, Helix, or Neovim
and want
real language diagnostics — rust-analyzer errors, pyright type errors — inside
`hick:file` blocks, not just syntax-highlighted text.*

A `.md` document embeds real source files:

```
<h:file path="src/stats.py">
def mean(xs):
    return sum(xs) / len(xs)
</h:file>
```

(Real documents use the `hick:` prefix; docs about hick use `h:` so examples
stay literal.)

Without an LSP, that Python is inert text — a typo surfaces only when
`hick run` fails. With `hick-lsp`, your editor shows pyright's diagnostics
on those lines as you type, at the correct positions in the `.md` file.

## 1. Get `hick-lsp`

It ships in the same release archive as `hick`, so if you installed with the
one-liner you already have it:

```sh
hick-lsp </dev/null && echo "installed"
```

(It speaks LSP over stdio; run by hand with a terminal attached it just waits
for input, which is why the check above closes stdin.) From a checkout
instead:

```sh
cargo install --path crates/hick-lsp
```

`hick-lsp` does not analyze languages itself. It spawns the real language
servers, so install the ones for the languages your documents embed:

| Language | Binary `hick-lsp` spawns | Install |
|---|---|---|
| Rust | `rust-analyzer` | `rustup component add rust-analyzer` |
| Python | `pyright-langserver` | `npm i -g pyright` |
| TypeScript/JS | `typescript-language-server` | `npm i -g typescript-language-server typescript` |
| Go | `gopls` | `go install golang.org/x/tools/gopls@latest` |

Missing servers are non-fatal: you just get no diagnostics for that language.
`hick init` warns about missing ones.

## 2. Let `hick init` do the wiring

Run it in the repository you work in:

```sh
hick init
```

Besides the pre-commit gate and the MCP registration, it does two things for
your editor:

- **It adopts the language servers this repository already uses.** If
  `.vscode/settings.json`, `.zed/settings.json`, or `.helix/languages.toml`
  names a server for a language — `pylsp` rather than the default `pyright`,
  say, or an explicit `rust-analyzer` path — it writes that into
  `.hick-lsp.json`, and `hick-lsp` spawns *that* server for the matching
  `hick:file` blocks. Without this, a document's Python would be checked by a
  different server than the `.py` file it generates, and the two would
  disagree.
- **It registers `hick-lsp` for `*.md` where a project file can do that.**
  Helix is complete from `.helix/languages.toml` alone. VS Code gets its
  settings written but still needs a generic LSP client extension. Zed needs
  the extension below, and Neovim has no project-local LSP registration at
  all — for those two `hick init` prints the step rather than writing a file
  that would do nothing.

Re-running is safe: entries already in `.hick-lsp.json` are never overwritten,
so a command you tuned by hand stays tuned.

`.hick-lsp.json` is a plain file you can write yourself:

```json
{
  "servers": {
    "python": { "command": ["pylsp"] },
    "rust":   { "command": ["rust-analyzer"] }
  }
}
```

Keys are LSP language ids (`python`, `rust`, `typescript`, `go`, …). Anything
not listed uses `hick-lsp`'s built-in default for that language.

## 3. Wire it into your editor by hand

Only needed for what `hick init` could not write, or if you skipped it.

### Zed

The extension lives in this repo at `editors/zed-hick`:

1. Zed → `zed: install dev extension` (command palette) → select the
   `editors/zed-hick` directory.
2. Open any `.md` file. The extension finds `hick-lsp` on your `PATH` and
   starts it.

### VS Code

There is no dedicated VS Code extension yet; any generic LSP client works.
With the [Generic LSP Client](https://marketplace.visualstudio.com/items?itemName=llllvvuu.glspc)
style of extension, configure:

```json
{
  "glspc.languageId": "hick",
  "glspc.serverCommand": "hick-lsp",
  "files.associations": { "*.md": "hick" }
}
```

Any client that can say "run `hick-lsp` over stdio for files matching
`*.md`" is equivalent.

### Neovim

With `nvim-lspconfig` (Neovim 0.10+):

```lua
vim.filetype.add({ extension = { md = "hick" } })

vim.api.nvim_create_autocmd("FileType", {
  pattern = "hick",
  callback = function()
    vim.lsp.start({
      name = "hick-lsp",
      cmd = { "hick-lsp" },
      root_dir = vim.fs.root(0, ".git"),
    })
  end,
})
```

## What just happened

When you open a `.md` file, `hick-lsp`:

1. parses the document and reports hick syntax errors itself;
2. extracts each `h:file` block (resolving `h:copy`/`h:paste` references)
   into an in-memory virtual file — the file the block *would* write;
3. detects each virtual file's language from its `path=` extension and spawns
   the matching child language server (one per language, shared across
   blocks) — the command from `.hick-lsp.json` if the project named one, and
   the built-in default otherwise;
4. opens the virtual files in those children, translating every position
   between `.md` coordinates and virtual-file coordinates in both
   directions;
5. merges the children's diagnostics back onto your `.md` buffer, so a
   pyright error on line 2 of the embedded file appears on the corresponding
   line of the document.

## Don't assume

- **`hick-lsp` replaces your other language servers only inside `.md`
  files.** Your normal `.py`/`.rs` files are untouched.
- **Diagnostics come from real servers, not a reimplementation** — if
  rust-analyzer needs a `Cargo.toml` to be useful, the same applies to
  embedded Rust.
- **A server named in `.hick-lsp.json` is spawned as written** — `hick init`
  copies what your editor config says, but it does not check that the command
  exists or that the server is configured the way that editor would have
  configured it. A server that fails to start is logged and skipped, and the
  document keeps its hick-level diagnostics.
- **`h:exec` shell blocks get no diagnostics** — only `h:file` blocks with a
  recognized extension do. Drift in exec blocks is caught by `hick test`,
  not the editor.
