<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Try a real language server in the small browser editor

For maintainers manually checking semantic highlighting and IntelliSense against a real TypeScript server. These files form an independent workspace; no global server configuration is needed.

## package.json

Pin the wrapper and its TypeScript backend together: the server must find tsserver, which TypeScript 5 supplies. Keep the example private and provide both a type check and an executable result.

<ui:copy id="simple-lsp-0">
{
  "name": "hickory-simple-lsp",
  "private": true,
  "scripts": {
    "check": "tsc --noEmit",
    "start": "tsc && node dist/main.js"
  },
  "devDependencies": {
    "typescript": "5.9.3",
    "typescript-language-server": "5.0.0"
  }
}
</ui:copy>

<ui:file path="examples/simple-lsp/package.json"><ui:paste select="#simple-lsp-0"/></ui:file>

## .gitignore

Dependencies and compiled output are local artifacts, not maintained example source.

<ui:copy id="simple-lsp-1">
node_modules/
dist/
</ui:copy>

<ui:file path="examples/simple-lsp/.gitignore"><ui:paste select="#simple-lsp-1"/></ui:file>

## tsconfig.json

A strict Node project gives the server a concrete import graph and a reproducible type error. Emit to a separate directory so running the example never overwrites source.

<ui:copy id="simple-lsp-2">
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "NodeNext",
    "moduleResolution": "NodeNext",
    "strict": true,
    "outDir": "dist",
    "rootDir": "src"
  },
  "include": [
    "src/**/*.ts"
  ]
}
</ui:copy>

<ui:file path="examples/simple-lsp/tsconfig.json"><ui:paste select="#simple-lsp-2"/></ui:file>

## src/catalog.ts

Expose an interface, documented function and typed data in a second module. Imported hover and member completion must come from the server’s project analysis rather than a list of words in the open file.

<ui:copy id="simple-lsp-3">
/** A product whose fields are offered by member completion. */
export interface Product {
  readonly name: string;
  price: number;
  category: "book" | "tool";
}

/** Apply a fractional discount, such as 0.2 for twenty percent. */
export function discountedPrice(product: Product, discount = 0.1): number {
  return Math.round(product.price * (1 - discount) * 100) / 100;
}

export const catalog: Product[] = [
  { name: "Literate programming", price: 30, category: "book" },
  { name: "Notebook", price: 8, category: "tool" },
];
</ui:copy>

<ui:file path="examples/simple-lsp/src/catalog.ts"><ui:paste select="#simple-lsp-3"/></ui:file>

## src/main.ts

Keep the starting program valid and runnable. Comments identify reversible probes; the same-file call also exercises the simple frontend’s currently supported definition navigation.

<ui:copy id="simple-lsp-4">
import { catalog, discountedPrice, type Product } from "./catalog.js";

// Hover Product or discountedPrice for type information and documentation.
const featured: Product = catalog[0];

// Completion: on a new line type featured. or featured.name.
// Explicit completion: press Ctrl-Space after typing featured.
// Signature help: on a new line type discountedPrice(
const total = discountedPrice(featured, 0.2);
console.log(`${featured.name}: $${total.toFixed(2)}`);

// Diagnostics: uncomment the next line, then remove it to recover.
// const invalid: Product = { name: "Broken", price: "free", category: "book" };

// Same-file definition: Ctrl/Cmd-click describe below to jump to its declaration.
function describe(product: Product) {
  return `${product.category}: ${product.name}`;
}
console.log(describe(featured));
</ui:copy>

<ui:file path="examples/simple-lsp/src/main.ts"><ui:paste select="#simple-lsp-4"/></ui:file>

## README.md

Give the person testing the browser exact setup steps, gestures, expected outcomes and the current cross-file navigation limit.

<ui:copy id="simple-lsp-5">
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
</ui:copy>

<ui:file path="examples/simple-lsp/README.md"><ui:paste select="#simple-lsp-5"/></ui:file>

## Lock the server install

Retain the registry integrity hashes so `npm ci` installs the same server and compiler for each walkthrough. These two packages have no runtime dependencies.

<ui:copy id="simple-lsp-lock">
{
  "name": "hickory-simple-lsp",
  "lockfileVersion": 3,
  "requires": true,
  "packages": {
    "": {
      "name": "hickory-simple-lsp",
      "devDependencies": {
        "typescript": "5.9.3",
        "typescript-language-server": "5.0.0"
      }
    },
    "node_modules/typescript": {
      "version": "5.9.3",
      "resolved": "https://registry.npmjs.org/typescript/-/typescript-5.9.3.tgz",
      "integrity": "sha512-jl1vZzPDinLr9eUt3J/t7V6FgNEw9QjvBPdysz9KfQDD41fQrC2Y4vKQdiaUpFT4bXlb1RHhLpp8wtm6M5TgSw==",
      "dev": true,
      "license": "Apache-2.0",
      "bin": {
        "tsc": "bin/tsc",
        "tsserver": "bin/tsserver"
      },
      "engines": {
        "node": ">=14.17"
      }
    },
    "node_modules/typescript-language-server": {
      "version": "5.0.0",
      "resolved": "https://registry.npmjs.org/typescript-language-server/-/typescript-language-server-5.0.0.tgz",
      "integrity": "sha512-Glr+4Ss9g6AupBPj3tnk3mrlSH1ROS2Zmr5h0Vmy3DA+MVq+5qq4tv4+SogB8VsEZ13Z9xU01zm54MMyI7Gdzw==",
      "dev": true,
      "license": "Apache-2.0",
      "bin": {
        "typescript-language-server": "lib/cli.mjs"
      },
      "engines": {
        "node": ">=20"
      }
    }
  }
}
</ui:copy>
<ui:file path="examples/simple-lsp/package-lock.json"><ui:paste select="#simple-lsp-lock"/></ui:file>

</ui:doc>
