<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Build a small codebase for exploring the simple UI

For a maintainer or tester who wants a concrete workspace covering the current simple frontend slice. The order calculator remains runnable without third-party packages; optional services enrich the same files independently. Prepare a copy before editing so UI experiments never mutate this repository's constructed source.

## Run, inspect and debug a real order calculation

Integer cents avoid floating-point rounding. Keep the calculation in a small helper so step-into, stack-frame selection and watches have useful values. Repeated identifiers and nested blocks also provide semantic, folding and multi-selection targets.

<ui:copy id="simple-showcase-0">
"""A dependency-free order calculator for the simple UI playground."""
from __future__ import annotations

import argparse
from dataclasses import dataclass
import time


@dataclass(frozen=True)
class Product:
    """A catalog item with an integer price in cents."""

    name: str
    price_cents: int


@dataclass(frozen=True)
class LineItem:
    product: Product
    quantity: int


CATALOG = {
    "notebook": Product("Notebook", 800),
    "pen": Product("Pen", 250),
    "guide": Product("Field guide", 2400),
}


def line_total(item: LineItem, discount_percent: int = 0) -> int:
    """Return discounted cents, rounding half a cent upward."""
    if item.quantity < 1:
        raise ValueError("quantity must be positive")
    if item.product.price_cents < 0:
        raise ValueError("price cannot be negative")
    if not 0 <= discount_percent <= 100:
        raise ValueError("discount must be between 0 and 100")
    subtotal = item.product.price_cents * item.quantity
    discounted = (subtotal * (100 - discount_percent) + 50) // 100
    return discounted


def order_total(items: list[LineItem], discount_percent: int = 0) -> int:
    """Sum line totals; put a breakpoint on the line_total call."""
    total = 0
    for item in items:
        amount = line_total(item, discount_percent)
        total += amount
    return total


def money(cents: int) -> str:
    return f"${cents // 100}.{cents % 100:02d}"


def demo() -> int:
    items = [LineItem(CATALOG["notebook"], 2), LineItem(CATALOG["pen"], 3)]
    total = order_total(items, discount_percent=20)
    print(f"Order: {money(total)}")
    print(f"Items: {sum(item.quantity for item in items)}")
    return total


# Language probes (undo each probe after trying it):
# Completion: type CATALOG["notebook"]. on a new line, then Ctrl-Space.
# Hover: inspect Product, LineItem or line_total.
# Signature: type line_total( in a new expression.
# Diagnostic: uncomment the next line; quantity expects an integer.
# invalid = LineItem(CATALOG["pen"], "three")
# Definition in browser: Ctrl/Cmd-click the demo call below.


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--watch", action="store_true", help="repeat until Interrupt")
    arguments = parser.parse_args()
    try:
        if arguments.watch:
            while True:
                demo()
                time.sleep(1)
        else:
            demo()
    except KeyboardInterrupt:
        print("Stopped.")
</ui:copy>
<ui:file path="examples/simple-ui/orders.py"><ui:paste select="#simple-showcase-0"/></ui:file>

## Check the arithmetic and failure boundaries

These focused standard-library tests establish the demo result and rounding policy. Invalid quantities and discounts give debugger exception controls a concrete failure to inspect without leaving the starting workspace broken.

<ui:copy id="simple-showcase-1">
"""Run with: python3 -m unittest -v"""
import unittest

from orders import CATALOG, LineItem, Product, line_total, money, order_total


class OrderTests(unittest.TestCase):
    def test_demo_total(self):
        items = [LineItem(CATALOG["notebook"], 2), LineItem(CATALOG["pen"], 3)]
        self.assertEqual(order_total(items, 20), 1880)
        self.assertEqual(money(1880), "$18.80")

    def test_half_cent_rounds_up(self):
        self.assertEqual(line_total(LineItem(Product("Sample", 101), 1), 50), 51)

    def test_empty_order(self):
        self.assertEqual(order_total([]), 0)

    def test_invalid_inputs(self):
        for quantity, discount in [(0, 0), (-1, 0), (1, -1), (1, 101)]:
            with self.subTest(quantity=quantity, discount=discount):
                with self.assertRaises(ValueError):
                    line_total(LineItem(CATALOG["pen"], quantity), discount)


if __name__ == "__main__":
    unittest.main()
</ui:copy>
<ui:file path="examples/simple-ui/test_orders.py"><ui:paste select="#simple-showcase-1"/></ui:file>

## Give search and multi-selection stable targets

Repeated labels, aligned columns, indentation and Unicode make edits easy to see. Nothing imports this file, so regex replacement and rectangular editing can be explored without breaking the runnable program.

<ui:copy id="simple-showcase-2">
Search playground — café 😀

name=notebook count=2
name=pen count=3
name=guide count=1

TOKEN north TOKEN
TOKEN south TOKEN
TOKEN west  TOKEN

sku       quantity  state
notebook  2         pending
pen       3         pending
guide     1         pending

order
    customer
        name: Ada
        city: Montréal
    delivery
        method: standard
        instructions: Leave the package beside the blue door.
    items
        notebook
            quantity: 2
        pen
            quantity: 3

This deliberately long logical line wraps in a narrow pane. Resize or split the editor and check that the left gutter gives this whole paragraph one number, while the right action stays attached to its first visual row. Undo should restore your edits after moving the tab into another pane.

Checkpoint 001: scroll the minimap, search for this number, and return to the first line.
Checkpoint 002: scroll the minimap, search for this number, and return to the first line.
Checkpoint 003: scroll the minimap, search for this number, and return to the first line.
Checkpoint 004: scroll the minimap, search for this number, and return to the first line.
Checkpoint 005: scroll the minimap, search for this number, and return to the first line.
Checkpoint 006: scroll the minimap, search for this number, and return to the first line.
Checkpoint 007: scroll the minimap, search for this number, and return to the first line.
Checkpoint 008: scroll the minimap, search for this number, and return to the first line.
Checkpoint 009: scroll the minimap, search for this number, and return to the first line.
Checkpoint 010: scroll the minimap, search for this number, and return to the first line.
Checkpoint 011: scroll the minimap, search for this number, and return to the first line.
Checkpoint 012: scroll the minimap, search for this number, and return to the first line.
Checkpoint 013: scroll the minimap, search for this number, and return to the first line.
Checkpoint 014: scroll the minimap, search for this number, and return to the first line.
Checkpoint 015: scroll the minimap, search for this number, and return to the first line.
Checkpoint 016: scroll the minimap, search for this number, and return to the first line.
Checkpoint 017: scroll the minimap, search for this number, and return to the first line.
Checkpoint 018: scroll the minimap, search for this number, and return to the first line.
Checkpoint 019: scroll the minimap, search for this number, and return to the first line.
Checkpoint 020: scroll the minimap, search for this number, and return to the first line.
Checkpoint 021: scroll the minimap, search for this number, and return to the first line.
Checkpoint 022: scroll the minimap, search for this number, and return to the first line.
Checkpoint 023: scroll the minimap, search for this number, and return to the first line.
Checkpoint 024: scroll the minimap, search for this number, and return to the first line.
Checkpoint 025: scroll the minimap, search for this number, and return to the first line.
Checkpoint 026: scroll the minimap, search for this number, and return to the first line.
Checkpoint 027: scroll the minimap, search for this number, and return to the first line.
Checkpoint 028: scroll the minimap, search for this number, and return to the first line.
Checkpoint 029: scroll the minimap, search for this number, and return to the first line.
Checkpoint 030: scroll the minimap, search for this number, and return to the first line.
Checkpoint 031: scroll the minimap, search for this number, and return to the first line.
Checkpoint 032: scroll the minimap, search for this number, and return to the first line.
Checkpoint 033: scroll the minimap, search for this number, and return to the first line.
Checkpoint 034: scroll the minimap, search for this number, and return to the first line.
Checkpoint 035: scroll the minimap, search for this number, and return to the first line.
Checkpoint 036: scroll the minimap, search for this number, and return to the first line.
Checkpoint 037: scroll the minimap, search for this number, and return to the first line.
Checkpoint 038: scroll the minimap, search for this number, and return to the first line.
Checkpoint 039: scroll the minimap, search for this number, and return to the first line.
Checkpoint 040: scroll the minimap, search for this number, and return to the first line.
Checkpoint 041: scroll the minimap, search for this number, and return to the first line.
Checkpoint 042: scroll the minimap, search for this number, and return to the first line.
Checkpoint 043: scroll the minimap, search for this number, and return to the first line.
Checkpoint 044: scroll the minimap, search for this number, and return to the first line.
Checkpoint 045: scroll the minimap, search for this number, and return to the first line.
Checkpoint 046: scroll the minimap, search for this number, and return to the first line.
Checkpoint 047: scroll the minimap, search for this number, and return to the first line.
Checkpoint 048: scroll the minimap, search for this number, and return to the first line.
Checkpoint 049: scroll the minimap, search for this number, and return to the first line.
Checkpoint 050: scroll the minimap, search for this number, and return to the first line.
Checkpoint 051: scroll the minimap, search for this number, and return to the first line.
Checkpoint 052: scroll the minimap, search for this number, and return to the first line.
Checkpoint 053: scroll the minimap, search for this number, and return to the first line.
Checkpoint 054: scroll the minimap, search for this number, and return to the first line.
Checkpoint 055: scroll the minimap, search for this number, and return to the first line.
Checkpoint 056: scroll the minimap, search for this number, and return to the first line.
Checkpoint 057: scroll the minimap, search for this number, and return to the first line.
Checkpoint 058: scroll the minimap, search for this number, and return to the first line.
Checkpoint 059: scroll the minimap, search for this number, and return to the first line.
Checkpoint 060: scroll the minimap, search for this number, and return to the first line.
Checkpoint 061: scroll the minimap, search for this number, and return to the first line.
Checkpoint 062: scroll the minimap, search for this number, and return to the first line.
Checkpoint 063: scroll the minimap, search for this number, and return to the first line.
Checkpoint 064: scroll the minimap, search for this number, and return to the first line.
Checkpoint 065: scroll the minimap, search for this number, and return to the first line.
Checkpoint 066: scroll the minimap, search for this number, and return to the first line.
Checkpoint 067: scroll the minimap, search for this number, and return to the first line.
Checkpoint 068: scroll the minimap, search for this number, and return to the first line.
Checkpoint 069: scroll the minimap, search for this number, and return to the first line.
Checkpoint 070: scroll the minimap, search for this number, and return to the first line.
Checkpoint 071: scroll the minimap, search for this number, and return to the first line.
Checkpoint 072: scroll the minimap, search for this number, and return to the first line.
Checkpoint 073: scroll the minimap, search for this number, and return to the first line.
Checkpoint 074: scroll the minimap, search for this number, and return to the first line.
Checkpoint 075: scroll the minimap, search for this number, and return to the first line.
Checkpoint 076: scroll the minimap, search for this number, and return to the first line.
Checkpoint 077: scroll the minimap, search for this number, and return to the first line.
Checkpoint 078: scroll the minimap, search for this number, and return to the first line.
Checkpoint 079: scroll the minimap, search for this number, and return to the first line.
Checkpoint 080: scroll the minimap, search for this number, and return to the first line.
</ui:copy>
<ui:file path="examples/simple-ui/scratch.txt"><ui:paste select="#simple-showcase-2"/></ui:file>

## Keep a plain data file beside inline tables

A standalone CSV remains ordinary text in this slice. The workbook supplies the inline grid; this file instead exercises Unicode, quoted commas and long wrapped rows in the plain editor.

<ui:copy id="simple-showcase-3">
name,price_cents,description
Notebook,800,"Grid paper, pocket size"
Pen,250,Smooth blue ink
Field guide,2400,Café walks and city wildlife 😀
</ui:copy>
<ui:file path="examples/simple-ui/data/catalog.csv"><ui:paste select="#simple-showcase-3"/></ui:file>

## Construct an executable workbook with inline content

Preserve the nested Hickory document literally. Its generated file belongs to the copied demo workspace at runtime, not to this repository construction. The cached receipt is created by an actual explicit run; no execution output is invented here.

<ui:entry path="examples/simple-ui/workbook.md">
<h:doc xmlns:h="http://www.hickorydocs.com/1.0">
# The corner shop order book

Open this document beside `orders.py`. **Bold decisions**, *emphasized notes*,
`literal code` and headings remain editable. Try typing and undoing beside a
rendered table; use **Edit source** in the browser to reveal its backing text.
Native table and equation controls are in each source line's right-hand ⋯ menu.

## A Markdown table you can edit

| Product | Quantity | Unit price |
| --- | ---: | ---: |
| Notebook | 2 | 8.00 |
| Pen | 3 | 2.50 |
| Café guide 😀 | 1 | 24.00 |

The order program uses its own typed catalog; editing these demonstration tables
does not silently rewrite Python business logic.

## A CSV-backed table

<h:table path="generated/stock.csv" header="true">
Product,Stock,Note
Notebook,12,"Grid paper, pocket size"
Pen,30,Blue ink
Field guide,7,Café walks 😀
</h:table>

## An editable equation

An inline formula: $p(1-d)$.

<h:math>
T = \sum_{i=1}^{n} q_i p_i (1-d)
</h:math>

Edit the equation body, apply it and undo. Native platforms keep source visible
and open a local typesetter in the line's floating controls.

## Literate code and a read-only generated output

The code below constructs `generated/coupon.py`. Edit this source file block,
not the generated Files entry. This also gives the debugger a literate Python
launch target.

<h:file path="generated/coupon.py" language="python">
def coupon_total(cents: int, discount_percent: int) -> int:
    discounted = (cents * (100 - discount_percent) + 50) // 100
    return discounted


if __name__ == "__main__":
    total = coupon_total(2350, 20)
    print(f"Coupon total: {total} cents")
</h:file>

## A real recorded execution

The preparation command executes this cell once and stores a source-matched
receipt locally. Browser cards and native line controls display the saved output.
Viewing it does not execute the command again. The cell runs Python from the
copied workspace using the local host executor; the image declaration describes
the environment and does not provide isolation in that mode.

<h:container name="python" image="python:3.12" />
<h:volume name="workspace" input="." />
<h:exec id="order-check" container="python" mount="workspace:project">
python3 project/orders.py
<h:expect match="exact">Order: $18.80
Items: 5
</h:expect>
</h:exec>

## A browser diagram

The browser renders this architecture inline. Native editors retain its readable
source; diagram rendering is not implemented there.

<h:diagram renderer="graph">
{"topology":{"nodes":[{"id":"catalog"},{"id":"order"},{"id":"receipt"}],"edges":[{"from":"catalog","to":"order"},{"from":"order","to":"receipt"}]},"layout":{"catalog":{"x":20,"y":20,"w":130,"h":50},"order":{"x":200,"y":20,"w":130,"h":50},"receipt":{"x":380,"y":20,"w":130,"h":50}}}
</h:diagram>

## Literal content stays literal

```python
# **This is code, not bold prose.**
print("<script>literal text</script>")
```

Search for `Notebook`, fold an indented Python block, split a pane and move this
tab. Keep an unsaved edit while moving it, then undo in the destination pane.
</h:doc>
</ui:entry>

## Exclude local capability and execution artifacts

The prepared workspace can acquire managed servers, transcript receipts and generated files. They are reproducible local artifacts rather than checked-in fixtures.

<ui:copy id="simple-showcase-5">
.hick-cache/
__pycache__/
generated/
</ui:copy>
<ui:file path="examples/simple-ui/.gitignore"><ui:paste select="#simple-showcase-5"/></ui:file>

## Prepare an isolated editable working copy

Copy into a new directory and refuse an existing destination, preserving every previous experiment. Recording is opt-in and runs only the supplied workbook with the local executor. Resolve the CLI before copying so a missing prerequisite does not leave a misleading half-prepared workspace.

<ui:copy id="simple-showcase-6">
"""Prepare the simple UI demo without overwriting an existing workspace."""
import argparse
import os
from pathlib import Path
import shutil
import subprocess


def main():
    source = Path(__file__).resolve().parent
    root = source.parents[1]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dest", type=Path, default=root / ".dev/simple-ui/project")
    parser.add_argument("--record", action="store_true", help="run the supplied cell on the host and cache its real output")
    args = parser.parse_args()
    executable = root / "target/debug/hick"
    if not executable.is_file():
        parser.error("Build the CLI first: cargo build --locked -p hickory-cli --bin hick")
    destination = args.dest.resolve()
    if destination.exists():
        parser.error(f"Preserving existing workspace: {destination}. Choose a new --dest.")
    if source == destination or source in destination.parents:
        parser.error("Choose a destination outside the maintained fixture folder.")
    shutil.copytree(source, destination, ignore=shutil.ignore_patterns("__pycache__", ".hick-cache", "generated"))
    environment = {**os.environ, "HICKORY_EXECUTOR": "local"}
    command = [str(executable), "run" if args.record else "weave", "workbook.md"]
    if args.record:
        command.append("--cache")
    subprocess.run(command, cwd=destination, env=environment, check=True)
    print(f"Demo ready: {destination}")
    print(f'HICKORY_PROJECT_DIR="{destination}" just simple-dev browser')


if __name__ == "__main__":
    main()
</ui:copy>
<ui:file path="examples/simple-ui/prepare.py"><ui:paste select="#simple-showcase-6"/></ui:file>

## Walk every available feature without implying platform parity

Name the file, action and observable result for each capability. Distinguish configuration-dependent services, browser-only rich widgets and native source-attached editors. A fixture makes these paths reproducible; it does not establish UI acceptance.

<ui:entry path="examples/simple-ui/README.md">
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
</ui:entry>

## Follow the receipt from execution to display

A cache hit has retained stdout but no timed events. The sample exposed that
missing display path: the [shared block projection](../mechanisms/crates/hick-literate/src/render.rs.md)
now exports `recorded_output` separately, and each simple frontend reads it only
through its existing source-hash checks. The Rust witness constructs an actual
cached entry without events; browser and Swift witnesses check display and
optional exit-event decoding. This does not create a live execution history or
establish device UI acceptance.

</ui:doc>
