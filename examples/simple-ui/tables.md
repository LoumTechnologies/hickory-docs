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

<hick:exec id="summarize" container="python" language="python" tables="Order" table="Summary">
order = table("Order")
display_table(["Metric", "Value"], [["Order total", sum(order["Total"])]])
</hick:exec>

## Read the published result

<hick:table id="Receipt" language="python">
Metric,Value
Total,=Summary!B2
</hick:table>
</hick:doc>
