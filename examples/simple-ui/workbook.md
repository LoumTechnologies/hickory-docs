<h:doc xmlns:h="http://www.hickorydocs.com/1.0">
# The corner shop order book

Open this document beside `orders.py`. **Bold decisions**, *emphasized notes*,
`literal code` and headings remain editable. Try typing and undoing beside a
rendered table; use **Edit source** in the browser to reveal its backing text.
Markdown tables appear as editable grids on every simple UI. Native equation and CSV cell controls remain in each source line's right-hand ⋯ menu.

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
