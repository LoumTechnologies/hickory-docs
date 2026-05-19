# hick-lsp

LSP (Language Server Protocol) server for `.hick` files.

Parses hick documents, generates virtual files from `<hick:file>` blocks
(resolving `<hick:copy>` / `<hick:paste>`), detects the target language of each
virtual file, and provides full IDE support by proxying to child language servers.

## Architecture

- `lang_detect` — identifies the language of each virtual file
- `child_lsp` — spawns and manages per-language LSP child processes
- `position_map` — translates LSP positions between `.hick` source coordinates
  and virtual file coordinates
- `Dispatcher` — routes incoming LSP requests to the correct child server by
  language ID, spawning lazily, and funnels child notifications (e.g.,
  `publishDiagnostics`) back through a shared channel

This allows editors to see diagnostics, completions, and go-to-definition for
the generated code as if the `.hick` file were the generated file directly.
