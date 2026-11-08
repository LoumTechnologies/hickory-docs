# Try language intelligence in the simple browser UI

Requires Node.js 20 or later, npm, and the repository's Rust/web development dependencies.
From the repository root:

```sh
npm --prefix examples/simple-lsp ci
cargo build --locked -p hick-lsp
HICKORY_PROJECT_DIR="$PWD/examples/simple-lsp" just simple-dev browser
```

Open `src/main.ts` in Files. Allow a moment for the language server to start; text opens immediately. This folder installs TypeScript 5 and its language server locally so Hickory discovers the server without global installs or editor configuration.

- Semantic colors: compare `Product`, `discountedPrice`, `featured` and the function parameters. The server colors identifiers; this example does not promise TextMate-style coloring of every punctuation mark.
- Completion: type `featured.` on a new line. Choose `name`, `price` or `category`. Ctrl-Space also requests completion. After `featured.name.` expect string methods such as `toUpperCase`.
- Hover: pause over `discountedPrice` to see its signature and documentation.
- Signature help: type `discountedPrice(` and inspect the parameter help.
- Diagnostics: uncomment the `invalid` declaration to introduce a string where a number is required, then undo.
- Definition: Ctrl/Cmd-click the final `describe` call to jump within this file. Cross-file navigation is not connected in the simple browser frontend yet.

`src/catalog.ts` supplies imported types and documentation; open it to inspect the other semantic categories. Native frontends have separate capability implementations; this walkthrough targets the browser.

To run the program or check its clean starting state:

```sh
npm --prefix examples/simple-lsp run check
npm --prefix examples/simple-lsp start
```

Expected output: `Literate programming: $24.00`, then `book: Literate programming`.
Stop the UI with Ctrl-C. If intelligence is absent, check that `node_modules/.bin/typescript-language-server` exists in this folder and rebuild `hick-lsp` before restarting the UI. Keep experiments unsaved or undo them: the checked-in example files are generated from `.hickory-source/frontends/simple-lsp.md`; permanent changes belong there.
