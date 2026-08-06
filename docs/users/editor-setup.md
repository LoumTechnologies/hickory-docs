# Editor setup: diagnostics inside your `.hick` documents

*For engineers who edit `.hick` documents in Zed, VS Code, or Neovim and want
real language diagnostics — rust-analyzer errors, pyright type errors — inside
`hick:file` blocks, not just syntax-highlighted text.*

A `.hick` document embeds real source files:

```
<h:file path="src/stats.py">
def mean(xs):
    return sum(xs) / len(xs)
</h:file>
```

(Real documents use the `hick:` prefix; docs about hick use `h:` so examples
stay literal.)

Without an LSP, that Python is inert text — a typo surfaces only when
`hickory run` fails. With `hick-lsp`, your editor shows pyright's diagnostics
on those lines as you type, at the correct positions in the `.hick` file.

## 1. Install `hick-lsp`

```sh
git clone https://github.com/LoumTechnologies/hickory-docs
cargo install --path hickory-docs/crates/hick-lsp
```

Verify: `hick-lsp` must be on your `PATH` (it speaks LSP over stdio; running
it by hand just waits for input — that's normal).

`hick-lsp` does not analyze languages itself. It spawns the real language
servers, so install the ones for the languages your documents embed:

| Language | Binary `hick-lsp` spawns | Install |
|---|---|---|
| Rust | `rust-analyzer` | `rustup component add rust-analyzer` |
| Python | `pyright-langserver` | `npm i -g pyright` |
| TypeScript/JS | `typescript-language-server` | `npm i -g typescript-language-server typescript` |
| Go | `gopls` | `go install golang.org/x/tools/gopls@latest` |

Missing servers are non-fatal: you just get no diagnostics for that language.
`hickory init` warns about missing ones.

## 2. Wire it into your editor

### Zed

The extension lives in this repo at `editors/zed-hick`:

1. Zed → `zed: install dev extension` (command palette) → select the
   `editors/zed-hick` directory.
2. Open any `.hick` file. The extension finds `hick-lsp` on your `PATH` and
   starts it.

### VS Code

There is no dedicated VS Code extension yet; any generic LSP client works.
With the [Generic LSP Client](https://marketplace.visualstudio.com/items?itemName=llllvvuu.glspc)
style of extension, configure:

```json
{
  "glspc.languageId": "hick",
  "glspc.serverCommand": "hick-lsp",
  "files.associations": { "*.hick": "hick" }
}
```

Any client that can say "run `hick-lsp` over stdio for files matching
`*.hick`" is equivalent.

### Neovim

With `nvim-lspconfig` (Neovim 0.10+):

```lua
vim.filetype.add({ extension = { hick = "hick" } })

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

When you open a `.hick` file, `hick-lsp`:

1. parses the document and reports hick syntax errors itself;
2. extracts each `h:file` block (resolving `h:copy`/`h:paste` references)
   into an in-memory virtual file — the file the block *would* write;
3. detects each virtual file's language from its `path=` extension and spawns
   the matching child language server (one per language, shared across
   blocks);
4. opens the virtual files in those children, translating every position
   between `.hick` coordinates and virtual-file coordinates in both
   directions;
5. merges the children's diagnostics back onto your `.hick` buffer, so a
   pyright error on line 2 of the embedded file appears on the corresponding
   line of the document.

## Don't assume

- **`hick-lsp` replaces your other language servers only inside `.hick`
  files.** Your normal `.py`/`.rs` files are untouched.
- **Diagnostics come from real servers, not a reimplementation** — if
  rust-analyzer needs a `Cargo.toml` to be useful, the same applies to
  embedded Rust.
- **`h:exec` shell blocks get no diagnostics** — only `h:file` blocks with a
  recognized extension do. Drift in exec blocks is caught by `hickory check`,
  not the editor.
