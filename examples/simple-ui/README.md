# Simple UI playground: the corner shop

A runnable Python project and executable workbook for the browser, macOS, iOS
and Android simple UIs. Python 3.10+ is sufficient for the program and tests.
Language servers, debug adapters and an agent provider are optional services.

## Start with an editable copy

From the Hickory repository root:

```sh
just simple-dev browser
```

The default is `.dev/simple-ui/project`. First launch builds the CLI, prepares a
copy, generates its files and records the supplied Python cell on your host using
the local executor. Later launches reuse the folder and preserve your edits.
Set `HICKORY_PROJECT_DIR=/absolute/path` to use another existing project.

For another fresh copy, run `python3 examples/simple-ui/prepare.py --record --dest /path/to/a/new/folder` after building the CLI. The optional `--record`
executes the sample cell and caches actual output. Omit it to weave without
executing; the output card will then be unrecorded. Preparation refuses to
overwrite an existing workspace.

Use `apple`, `ios`, `android` or `all` instead of `browser` to select the existing
platform launcher. Native launchers require their platform SDK/device setup.
Phone clients attach to the engine on the development machine; Python runs on
that machine, not on the phone. See
[the frontend guide](../../docs/developers/simple-frontends.md) for prerequisites.

## Guided feature tour

| Feature | File/action | What to look for |
| --- | --- | --- |
| Files and immediate text | Open `orders.py`, then `workbook.md` | Text is readable before optional services finish. |
| Line numbers and wrapping | Narrow `scratch.txt` | Wrapped continuations retain one logical line number. |
| Folding | Right ⋯ beside `for item in items:` in `orders.py` | Fold/unfold the indented body without changing saved source. |
| Minimap | Click near the bottom of `scratch.txt` | Jump to the numbered checkpoints, then return to the top. |
| Find and replace | Cmd/Ctrl-F in `scratch.txt`; regex `name=(\w+)` and replacement `sku=$1` | Preview/navigate matches, replace all, undo once. |
| Multiple selections | Find `TOKEN`; native Select all matches, browser select-next occurrence | Replace all selected words, then undo. |
| Rectangular editing | Select the three aligned `pending` values in `scratch.txt` | Native Edit → column selection; browser Alt-drag rectangular selection. |
| Additional cursors | Native Edit → add cursor above/below | Apple and Android physical keyboards broadcast typing; Android touch uses Apply to selections. IME composition returns to a single cursor. |
| Source formatting | `workbook.md` headings, bold, italics and code | Native fonts style source while retaining delimiters; browser has richer inline rendering. |
| Inline tables | Change Notebook quantity in either workbook table | Browser edits its grid; native opens cells in the line's ⋯ controls. Apply and undo preserve the surrounding text. |
| Inline equations | Open the workbook equation | Browser renders in flow; native ⋯ opens typesetting and an editable equation body. Apply, then undo. |
| Diagram | Workbook's catalog → order → receipt graph | Inline graph in browser; readable source on native. |
| Generated files | Open `generated/coupon.py` after preparation | Generated output is readable and read-only. Edit its owning workbook file block instead. |
| Recorded output | Workbook's `order-check` cell | `Order: $18.80` and `Items: 5`; native uses the cell line's ⋯ controls. Viewing does not rerun code. |
| Save and undo/redo | Make an edit in the copied `scratch.txt` | Save it; undo/redo remains available. Generated files refuse editing. |
| Pane split/resize | Split from the menu, drag the divider | Keep code and workbook side by side. |
| Tab move/reorder/dock | Drag a dirty tab to another strip or pane edge | Text, selection and undo follow it; dirty tabs refuse to close. |
| Terminal | Open Terminal and run commands below | Real host process, retained draft/process when the tab moves. |
| Agent | Open Agent; enter the prompt below | Send explicitly after configuring a provider; move the tab with an unsent draft. |

The next sections exercise semantic services and debugger controls. Unsupported
features are not simulated: full native WYSIWYG, completion snippets,
cross-file rename/code actions, persistent pane layout, and crash recovery are
outside the current simple UI slice. Native math requires prepared local KaTeX
assets (the launcher prepares them from installed web dependencies).

## Real language services

Install Python's real server for the prepared workspace:

```sh
target/debug/hick lsp install python --root "$PWD/.dev/simple-ui/project"
```

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
6. Browser Ctrl/Cmd-click the final `demo()` call for same-file definition.

## Debugger

Install the real adapter if the launch control requests it:

```sh
target/debug/hick dap install python --root "$PWD/.dev/simple-ui/project"
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

After configuring an agent provider through the existing settings UI, use this
prompt in the simple UI's Agent pane:

> Read orders.py and test_orders.py. Explain why the sample total is $18.80.
> Suggest a boundary test without changing files.

The sample supplies no provider credentials and sends nothing automatically.

## Refresh a recorded result

After intentionally changing saved source, rerun from the repository root:

```sh
cd .dev/simple-ui/project
HICKORY_EXECUTOR=local ../../../target/debug/hick run workbook.md --cache
```

Refresh recorded output in the UI. Dirty or mismatched source hides positional
receipts until they match again. The copied project is disposable; permanent
fixture changes belong to `.hickory-source/frontends/simple-showcase.md`.
