<h:doc xmlns:h="http://www.hickorydocs.com/1.0">
# The corner shop order book

Open this document beside `orders.py`. **Bold decisions**, *emphasized notes*,
`literal code` and headings remain editable. Try typing and undoing beside a
rendered table; use **Edit source** in the browser to reveal its backing text.
Markdown and CSV tables appear as editable grids on every simple UI. Use the arrow keys to move between cells. **Edit source text** reveals just that table; **Edit tables as grids** restores its grid. Equation controls use the function icon beside their source line.

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
debug `generated/coupon.py` from the patch destination header. Added code has
source ownership; context and deleted rows do not offer new-code breakpoints.
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
that cell separate lets this workbook reconstruct its complete file-and-patch
project without pretending an unexecuted command has retained filesystem outputs.
Viewing a recorded result never reruns its command.

## An inline diagram

Every Simple UI renders this architecture inline. Use Edit source text to change
the graph, then restore its preview. Native previews also support Mermaid.

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

<h:project />
</h:doc>
