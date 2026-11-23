# Simple UI playground: the corner shop

An executable corner-shop workbook with Python examples and a live TypeScript
architecture diagram. Preparing the demo uses Python 3.10+ and Node.js with
npm. Language servers, debug adapters and an agent provider are optional.

Open [tables.md](tables.md) for named-table formulas and a read-only Python output.

## Start with an editable copy

From the Hickory repository root:

```sh
just simple-dev browser
```

The launcher opens `workbook.md` in `.dev/simple-ui/code-review`. First launch builds
the CLI, prepares a copy, generates its files, records the supplied Python
cells on your host and creates a local Git baseline. Later launches reuse the
folder and preserve your edits. Set `HICKORY_PROJECT_DIR=/absolute/path` to use
another existing project, including an earlier demo at `.dev/simple-ui/project`.

For another fresh copy, run `python3 examples/simple-ui/prepare.py --record --dest /path/to/a/new/folder` after building the CLI. The optional `--record`
executes the sample cell and caches actual output. Omit it to construct the workbook and weave the runtime example without
executing; the output card will then be unrecorded. Preparation refuses to
overwrite an existing workspace.

Add `HICKORY_OPEN_FILE=workbook.md` when launching an explicitly selected copy.
Use `apple`, `ios`, `android`, `linux`, `windows` or `all` to select the existing
platform launcher. Native launchers require their platform SDK/device setup.
Phone clients attach to the engine on the development machine; Python runs on
that machine, not on the phone. See
[the frontend guide](../../docs/developers/simple-frontends.md) for prerequisites.

## Guided feature tour

Open [architecture.md](architecture.md) for a live view of the TypeScript
modules in `shop/`. View source reveals the code-model query and graph builder.
Outside the checkout, install the TypeScript code-model provider on the engine’s
PATH before opening the diagram.

| Feature | File/action | What to look for |
| --- | --- | --- |
| Files and immediate text | Open `orders.py`, then `workbook.md` | Text is readable before optional services finish. |
| Line numbers and wrapping | Narrow `scratch.txt` | Wrapped continuations retain one logical line number. |
| Folding | Left − beside a workbook heading or `for item in items:` in `orders.py` | Fold/unfold the section or indented body without changing saved source. |
| Minimap | Click near the bottom of `scratch.txt` | Jump to the numbered checkpoints, then return to the top. |
| Find and replace | Cmd/Ctrl-F in `scratch.txt`; regex `name=(\w+)` and replacement `sku=$1` | Preview/navigate matches, replace all, undo once. |
| Multiple selections | Find `TOKEN`; native Select all matches, browser select-next occurrence | Replace all selected words, then undo. |
| Rectangular editing | Select the three aligned `pending` values in `scratch.txt` | Native Edit → column selection; browser Alt-drag rectangular selection. |
| Additional cursors | Native Edit → add cursor above/below | Apple and Android physical keyboards broadcast typing; Android touch uses Apply to selections. IME composition returns to a single cursor. |
| Source formatting | `workbook.md` headings, bold, italics and code | Native fonts style source while retaining delimiters; browser has richer inline rendering. |
| Inline tables | Change Notebook quantity in either workbook table | Both tables replace their source lines with spreadsheet grids. Arrow keys move between cells. Edit a cell, press Return/Done, then undo. Edit source text reveals that table; Edit tables as grids restores it. Opening `data/catalog.csv` uses the same editor. |
| Inline equations | Open the workbook equation | Browser renders in flow; native function icon opens typesetting and an editable equation body. Apply, then undo. |
| Diagram | Workbook's catalog → order → receipt graph | Inline graph on every Simple UI; Edit source text and restore its preview. |
| Fenced code | Workbook’s plain Python fence | Local syntax colors without inventing a workspace file or waiting for LSP. |
| Patch code | Workbook’s `patch` fence | Inspect before/after code, use language tools and debug owned additions; deleted rows stay review text. |
| Generated files | Open `generated/coupon.py` after preparation | Generated output is readable and read-only. Edit its owning workbook file block instead. |
| Recorded output | README's `order-check` cell | `Order: $18.80` and `Items: 5`; native uses the cell line's output icon. Viewing does not rerun code. |
| Save and undo/redo | Make an edit in the copied `scratch.txt` | Save it; undo/redo remains available. Generated files refuse editing. |
| Pane split/resize | Split from the menu, drag the divider | Keep code and workbook side by side. |
| Tab move/reorder/dock | Drag a dirty tab to another strip or pane edge | Text, selection and undo follow it; dirty tabs refuse to close. |
| Terminal | Open Terminal and run commands below | Real host process, retained draft/process when the tab moves. |
| Agent | Open Agent and send the workbook prompt | Review and apply a patch to the draft, then save only if you want to keep it. |
| Git review | Save a one-line edit to `test_orders.py`, then open Repository changes | Read the diff against the starting commit and try stage/unstage. Repository history shows the baseline. |
| Source history and recovery | Save an edit to `scratch.txt` or keep an unsaved draft, then open the corresponding view | Local source history shows a saved revision. Source recovery lets you preview retained drafts before restoring them. |
| Tests and coverage | Open `test_orders.py` and choose Tests and coverage or a source test gutter | Run the local test and follow the result to its source. Coverage depends on an installed collector. |
| Workspace search and outline | Search Everywhere for `line_total`; open File outline in `orders.py` | See where the function is called and jump back after inspecting it. |
| Templates and refactoring | Open Edit → Insert reusable template or Refactor and quick fixes in `orders.py` | Preview an available change before applying it. Python provider support varies by action. |
| Construction and analysis | Open Construction reading on the workbook and Workspace analysis on the project | Inspect the authored coupon block and patch, then follow the generated file and any available evidence. |

The next sections exercise semantic services and debugger controls. The actions
a provider offers depend on its capabilities and the selected source. Native
math requires local KaTeX assets, which the launcher prepares from the installed
web dependencies.

## Real language services

The default Python playground does not install language servers automatically.
Its native code colors and IntelliSense need basedpyright in this workspace;
servers installed in a different demo do not carry over. Use the same
`HICKORY_PROJECT_DIR` in each launcher when comparing platforms.

Install Python's real server for the prepared workspace:

```sh
target/debug/hick lsp install python --root "$PWD/.dev/simple-ui/code-review"
```

A launcher can attach to the engine already running for an installed desktop
app. If that older engine does not support the current editor correctly, keep
its open sessions intact and launch a separate development engine:

```sh
HICKORY_STATE_DIR="$PWD/.dev/simple-ui/engine" just simple-dev apple
```

Use that same state directory and project directory for a browser comparison.
Restart/Retry language services, then open `orders.py`:

1. Inspect semantic colors on `Product`, `LineItem`, parameters and variables.
2. Type `CATALOG["notebook"].` on a temporary new line. Request Ctrl-Space and
   choose `price_cents`; undo the probe afterward.
3. Hover `line_total` in browser. Native Edit → Language tools shows hover,
   signatures and inlay information at the selected source position.
4. Type `line_total(` for parameter help, then undo.
5. Uncomment `invalid = LineItem(CATALOG["pen"], "three")`. Inspect the type
   diagnostic and undo. Native unversioned diagnostics are messages without
   clickable source positions. Actual hints depend on server capabilities.
6. Browser Ctrl/Cmd-click the final `demo()` call for definition. In the browser Edit menu or native Language tools, use Go to definition, Find usages, Go to type definition and Find implementations. Choose a returned authored file to open its tab at the symbol. Queries do not save text or resize the editor; unsupported provider targets remain read-only information. Retry an empty result if the server is still indexing.

## Debugger

Install the real adapter if the launch control requests it:

```sh
target/debug/hick dap install python --root "$PWD/.dev/simple-ui/code-review"
```

Save `orders.py`. In its right-hand line controls, put a breakpoint on
`amount = line_total(item, discount_percent)`, then launch Python. When paused:

- Step into `line_total`, step over the arithmetic and step out to the caller.
- Inspect locals; add watch `item.quantity` and evaluate `discount_percent`.
- Choose the caller in the stack, then return to the top frame.
- Try breakpoint condition `item.quantity > 2`, continue, and stop.
- Repeat using the workbook's Python file block for literate source mapping.

Jump-here, conditional breakpoints and other optional actions depend on adapter
capabilities. Save edits before launch; stale source cannot authorize debugging.

## Terminal and agent

In Terminal (its working directory is the prepared workspace):

```sh
python3 orders.py
python3 -m unittest -v
python3 orders.py --watch
```

Expected program output is `Order: $18.80` and `Items: 5`. Interrupt stops the
watch loop. Move the terminal tab while it runs, or keep an unsent command in its
draft. Full-screen terminal applications and rich ANSI rendering are outside the
matched terminal slice.

On the workspace host, install the locked Python worker from the Hickory
checkout and sign in to the subscription you plan to use:

```sh
uv sync --project apps/agent --locked --no-editable
codex login
```

Relaunch `just simple-dev` after installing the worker. In **Agent**, choose
**ChatGPT subscription** for Codex. If you use Claude, install Claude Code, run
`claude auth login` on the same host and choose **Claude Code subscription**.
The picker also offers Grok. Google One and Gemini accounts have no provider
option here. Use the workbook prompt to try a guarded patch review.

## Refresh a recorded result

After intentionally changing saved source, rerun from the repository root:

```sh
cd .dev/simple-ui/code-review
HICKORY_EXECUTOR=local ../../../target/debug/hick run README.md --cache
```

Refresh recorded output in the UI. Dirty or mismatched source hides positional
receipts until they match again. The copied project is disposable; permanent
fixture changes belong to `.hickory-source/frontends/simple-showcase.md`.

## Recorded execution

The preparation command records this cell on the local host when `--record` is
selected. Its output is real cached stdout; viewing the cell does not rerun it.
This standalone execution example is separate from the workbook’s pure project
construction. Its declared container describes the environment; the local
executor runs Python on the development host.

<hick:container name="python" image="python:3.12" />
<hick:volume name="workspace" input="." />
<hick:exec id="order-check" container="python" mount="workspace:project">
python3 project/orders.py
<hick:expect match="exact">Order: $18.80
Items: 5
</hick:expect>
</hick:exec>
