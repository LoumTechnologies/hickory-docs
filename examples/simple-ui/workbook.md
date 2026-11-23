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

## Architecture from the TypeScript code model

Open [architecture.md](architecture.md) to follow the repository’s diagram
script from its GraphQL query to the clickable nodes.

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
