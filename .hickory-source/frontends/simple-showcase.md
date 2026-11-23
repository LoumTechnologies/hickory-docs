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

A standalone CSV opens in the same spreadsheet grid as inline tables. This file exercises Unicode, quoted commas and long rows. Use **Edit source text** to inspect its original CSV and **Edit table as grid** to return. Choose a formula language for the grid session before entering formulas in plain CSV.

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

Open **Terminal** from **View** (the **Panes** menu on macOS) and run
`python3 -m unittest -v`. Then open `orders.py` and find `line_total` to see
what the tests exercise.

For a first review, open **Agent** from the same menu and send this prompt:

> Read orders.py and test_orders.py using project.md as the document context.
> Explain why the example totals $18.80.
> Put a unified diff for a new 100% discount test in a display-only `diff`
> fence inside the accepted response. Leave the workspace files unchanged.

Use the patch's right gutter to review it. Apply to the draft, inspect
`test_orders.py`, then save and rerun the tests. **Repository changes** shows the
saved diff against the sample's starting commit. The [feature tour](README.md)
also covers language tools, debugging, source history, coverage and recovery.

Keep this document beside `orders.py`. **Bold decisions**, *emphasized notes*,
`literal code` and headings remain editable. Try typing and undoing beside a
rendered table; use **Edit source** in the browser to reveal its backing text.
Markdown and CSV tables appear as editable grids. Click a row number or column
letter to select it, or drag across the cells to select a rectangle. Drag the
lower-right **+** handle out from the selection to extend its pattern. **Edit
source text** reveals one table's backing text. **Edit tables as grids** returns
to the grid.

## A Markdown table you can edit

| Product | Quantity | Unit price |
| --- | ---: | ---: |
| Notebook | 2 | 8.00 |
| Pen | 3 | 2.50 |
| Café guide 😀 | 1 | 24.00 |

The order program uses its own typed catalog; editing these demonstration tables
does not silently rewrite Python business logic.

## A CSV-backed table

<h:table header="true">
Product,Stock,Note
Notebook,12,"Grid paper, pocket size"
Pen,30,Blue ink
Field guide,7,Café walks 😀
</h:table>

## Calculate with LaTeX in a table

The cells in column B evaluate LaTeX arithmetic against column A, which gives
4 in B1 and 4 in B2. Use explicit operators between terms. Formulas begin with
`=` and accept fractions, roots, powers, and cell references.

<h:table id="discount_math" language="latex" header="false">
8,=\frac{A1}{2}
12,=\sqrt{A2+4}
</h:table>

## An editable equation

An inline formula: $p(1-d)$.

<h:math>
T = \sum_{i=1}^{n} q_i p_i (1-d)
</h:math>

In Simple, double-click or double-tap an equation to reveal its LaTeX.
Edit it in place, then double-activate its text to render it again. The function
icon also opens an equation editor with a live preview.

## Literate code and a read-only generated output

The code below constructs `generated/coupon.py`. Edit this source file block,
not the generated Files entry. This also gives the debugger a literate Python
launch target.

<h:file id="coupon-base" path="generated/coupon.py" language="python">
def coupon_total(cents: int, discount_percent: int) -> int:
    discounted = (cents * (100 - discount_percent) + 50) // 100
    return discounted


if __name__ == "__main__":
    total = coupon_total(2350, 20)
    print(f"Coupon total: {total} cents")
</h:file>

## Patch the coupon without copying its program

This patch constructs the receipt-label change over the Python base above.
Inspect its before/after code, use language tools on the added statement, or
debug `generated/coupon.py` from the patch destination header. In Simple, click an added or context line's number to toggle a pending
breakpoint in the destination program. A stop on context code still points to
the authored file block that owns those bytes. The fold control at the hunk header collapses its body.
Save the document before debugging so the adapter runs the displayed project.

```patch id="coupon-receipt" after="coupon-base"
--- a/generated/coupon.py
+++ b/generated/coupon.py
@@ -6,3 +6,3 @@
 if __name__ == "__main__":
     total = coupon_total(2350, 20)
-    print(f"Coupon total: {total} cents")
+    print(f"Coupon receipt: {total} cents")
```

## A rendered diff

This display-only diff shows the coupon receipt change. Red minus rows show the
old text; green plus rows show the replacement. Unlike the patch above, this
block does not change generated files.

```diff
--- a/generated/coupon.py
+++ b/generated/coupon.py
@@ -6,3 +6,3 @@
 if __name__ == "__main__":
     total = coupon_total(2350, 20)
-    print(f"Coupon total: {total} cents")
+    print(f"Coupon receipt: {total} cents")
```

## A real recorded execution

Open [the recorded execution example](README.md#recorded-execution) to inspect
`order-check` and its saved stdout. It is a standalone runtime document: keeping
that cell separate lets the workspace reconstruct its complete file-and-patch
project without pretending an unexecuted command has retained filesystem outputs.
Viewing a recorded result never reruns its command.

## An inline diagram

Every Simple UI renders this architecture inline. Use Edit source text to change
the graph, then restore its preview. Native previews also support Mermaid.

<h:diagram renderer="graph">
{"topology":{"nodes":[{"id":"catalog"},{"id":"order"},{"id":"receipt"}],"edges":[{"from":"catalog","to":"order"},{"from":"order","to":"receipt"}]},"layout":{"catalog":{"x":20,"y":20,"w":130,"h":50},"order":{"x":200,"y":20,"w":130,"h":50},"receipt":{"x":380,"y":20,"w":130,"h":50}}}
</h:diagram>

## Architecture from the code model

This diagram reads the saved Python files beside the workbook. The arrows
show name-resolved references. Select a file node to open it at its first
definition, or refresh after saving changes to the code.

<h:diagram renderer="architecture">
</h:diagram>

## Literal content stays literal

Use the play gutter to debug this Python fence or choose **Launch profiles**
to run it as a fresh program. Click a code line number to set a breakpoint.
Run and Debug put terminal output below the fence. Debug output links to
source when the adapter reports a matching location.

```python
# **This is code, not bold prose.**
import sys

print("<script>literal text</script>")
print("This line is stderr.", file=sys.stderr)
```

Search for `Notebook`, fold an indented Python block, split a pane and move this
tab. Keep an unsaved edit while moving it, then undo in the destination pane.

</h:doc>
</ui:entry>

The workspace's canonical project binds construction and review to the same
outputs. The workbook remains the first file the engineer reads and owns the
coupon source and patch. The project excludes local tooling and retained agent
sessions from captured work, keeping the review boundary small after installs.

<ui:entry path="examples/simple-ui/project.md">
<h:doc xmlns:h="http://www.hickorydocs.com/1.0">
# Corner shop construction

Open [workbook.md](workbook.md) for the tour and the authored coupon source.

<h:project sources="workbook.md" exclude=".hick-cache/ .hick-journal/ __pycache__/ sessions/" />
</h:doc>
</ui:entry>

## Exclude local capability and execution artifacts

Keep the cache, local journal, language server configuration and agent sessions
out of Git. Keep the generated coupon visible in Files so readers can inspect
its guarded output and compare it with the committed baseline.

<ui:copy id="simple-showcase-5">
.hick-cache/
.hick-journal/
.hick-lsp.json
__pycache__/
sessions/
</ui:copy>
<ui:file path="examples/simple-ui/.gitignore"><ui:paste select="#simple-showcase-5"/></ui:file>

## Prepare an isolated editable working copy

Copy into a new directory and refuse an existing destination, preserving every
previous experiment. Recording is opt-in and runs the supplied runtime documents
with the local executor. Commit the prepared sources and deterministic output to
a new, local-only Git repository, giving review and history a starting point
even when the demo lives under another checkout.

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
    parser.add_argument("--dest", type=Path, default=root / ".dev/simple-ui/code-review")
    parser.add_argument("--record", action="store_true", help="run the supplied cells on the host and cache their real output")
    args = parser.parse_args()
    executable = root / "target/debug" / ("hick.exe" if os.name == "nt" else "hick")
    if not executable.is_file():
        parser.error("Build the CLI first: cargo build --locked -p hickory-cli --bin hick")
    destination = args.dest.resolve()
    if destination.exists():
        parser.error(f"Preserving existing workspace: {destination}. Choose a new --dest.")
    if source == destination or source in destination.parents:
        parser.error("Choose a destination outside the maintained fixture folder.")
    shutil.copytree(source, destination, ignore=shutil.ignore_patterns(
        "__pycache__", ".git", ".hick-cache", ".hick-journal", ".hick-lsp.json", "generated", "sessions",
    ))
    environment = {**os.environ, "HICKORY_EXECUTOR": "local"}
    subprocess.run([str(executable), "project", "materialize", "project.md"], cwd=destination, env=environment, check=True)
    command = [str(executable), "run" if args.record else "weave", "README.md"]
    if args.record:
        command.append("--cache")
    subprocess.run(command, cwd=destination, env=environment, check=True)
    command[2] = "tables.md"
    subprocess.run(command, cwd=destination, env=environment, check=True)
    # The sample uses its own Git identity and configuration. Parent index
    # settings and the developer's commit templates must not enter the demo.
    git_environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
    git_environment.update(GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull)
    for arguments in [
        ["init", "--quiet", "--template="],
        ["add", "--", "."],
        ["-c", "user.name=Hickory Simple", "-c", "user.email=demo@example.invalid",
         "commit", "--quiet", "--message", "Prepare the corner shop workbook"],
    ]:
        subprocess.run(["git", *arguments], cwd=destination, env=git_environment, check=True)
    print(f"Demo ready: {destination}")
    print(f'HICKORY_PROJECT_DIR="{destination}" HICKORY_OPEN_FILE=workbook.md just simple-dev browser')


if __name__ == "__main__":
    main()
</ui:copy>
<ui:file path="examples/simple-ui/prepare.py"><ui:paste select="#simple-showcase-6"/></ui:file>

## Walk every available feature without implying platform parity

Name the file, action and observable result for each capability. Distinguish configuration-dependent services, browser-only rich widgets and native source-attached editors. A fixture makes these paths reproducible; it does not establish UI acceptance.

<ui:entry path="examples/simple-ui/README.md">
# Simple UI playground: the corner shop

A runnable Python project and executable workbook for the Simple frontends.
Python 3.10+ is sufficient for the program and tests.
Language servers, debug adapters and an agent provider are optional services.

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
</ui:entry>

## Follow the receipt from execution to display

A cache hit has retained stdout but no timed events. The sample exposed that
missing display path: the [shared block projection](../mechanisms/crates/hick-literate/src/render.rs.md)
now exports `recorded_output` separately, and each simple frontend reads it only
through its existing source-hash checks. The Rust witness constructs an actual
cached entry without events; browser and Swift witnesses check display and
optional exit-event decoding. This does not create a live execution history or
establish device UI acceptance.

## Follow named inputs into a read-only result

Keep the demo small enough to read on a phone: two editable tables, one explicit
Python fence and one reference to its published result. Preparation records
real output only when requested. Existing workspaces remain untouched.

<ui:entry path="examples/simple-ui/tables.md">
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
# Tables that calculate together

Open this file in any simple UI. Edit a price or quantity, select cells and use
the formula bar. `Prices!B2` reads another named table; `$B$2` locks an address
when filling. Drag the bottom edge of a grid to change its height.

<hick:container name="python" image="python:3.12" />

## Prices

<hick:table id="Prices" language="python">
Item,Price
Tea,3
Cake,5
</hick:table>

## Order

<hick:table id="Order" language="python">
Item,Quantity,Total
Tea,2,=B2 * Prices!B2
Cake,1,=B3 * Prices!B3
</hick:table>

## A table from Python

Run explicitly with `HICKORY_EXECUTOR=local hick run tables.md --cache` (use the CLI path in README
for a repository checkout), then choose Refresh recorded output. `tables="Order"`
declares its input. `table("Order")` reads computed values; add `raw=True` to
read formula source instead. The published `Summary` table is read-only.
Viewing formulas never runs this block. After editing inputs, save and rerun to
publish current results.

```python id="summarize" container="python" tables="Order" table="Summary"
order = table("Order")
display_table(["Metric", "Value"], [["Order total", sum(order["Total"])]])
```

## Read the published result

<hick:table id="Receipt" language="python">
Metric,Value
Total,=Summary!B2
</hick:table>
</hick:doc>
</ui:entry>

</ui:doc>
