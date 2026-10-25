<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/users/notebooks.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `872fe7a321dce50fd9d1ceb94feae4cfcd0ee6616ed20b2cc84f41fbcedafd2c`.

<book:entry path="docs/users/notebooks.md">
# Read datasets and show Python results

For Python/Polars users writing analysis in Hickory's Markdown editor.
Embed a Parquet file or Delta table to inspect its schema and rows, then run
Python to display a transformed table or chart below the cell.

Install the readers into your project's Python environment:

```sh
python -m pip install 'polars>=1.30,<2' 'deltalake>=1,<2' 'pyarrow>=19,<24'
```

For local cell execution, launch Hickory from the activated Python environment
so `python3` (`python` on Windows) can import these packages. Dataset previews
also discover a project `.venv` beside `pyproject.toml`.

Add a dataset reference to your document. Paths resolve from the document's
folder, within the open workspace:

```xml
<hick:dataset src="data/sales.parquet" />
<hick:dataset src="data/sales-delta" format="delta" />
```

Use the dataset's rail icon to switch between the reference and its preview.
The preview shows column names and types, preserves nulls and nested values,
and offers 100-row pages. Filter and sort operate on the visible page.
Refresh reads the dataset again. These controls never modify the dataset.

Add a Python fence with XML-style attributes:

````markdown
<hick:volume name="sales-data" input="data" />

```python exec="true" id="regional-sales" mount="sales-data:data"
import polars as pl
sales = pl.read_parquet("data/sales.parquet")
display(sales.group_by("region").agg(pl.col("revenue").sum()))
```
````

Run it with the cell's Run action. `print(...)` produces terminal text;
`display(...)` produces a read-only table. A successful run saves an output
fence beside the cell, including its input fingerprint and output hash.
The document keeps that snapshot when reopened. Plain Python fences stay examples.
`exec="false"` keeps a fence from running. Optional `container="py"`,
`image="python:3.12"`, `mount="data:/data"`, `show="output"`, `freeze="true"`
and `timeout="30"` use the existing cell execution and recording behavior.
Each Python cell starts a fresh interpreter; share data through files.
Python file paths resolve inside the execution environment, so mount your
dataset when running in a container. Install dependencies in that container too.

`display` accepts Polars DataFrames, LazyFrames and Series, pandas DataFrames,
Arrow tables, matplotlib figures, and image bytes with an explicit MIME type:

```python
# Examples inside an executable cell:
display(result, limit=200)
display(figure)
display('<svg xmlns="http://www.w3.org/2000/svg">...</svg>')
display(png_bytes, mime="image/png")
```

A cell can mix text, tables and images. Rich output travels through the same
transcript and recording system as text, so replay requires no rerun. Woven
Markdown includes tables with their types and embedded image data. Tables
show 100 rows by default; `display(..., limit=1000)` raises the cap. Dataset
previews and displayed tables show at most 256 columns and 8 MiB per result.
Dataset previews read schema and a page without counting the entire dataset.

For S3, use `<hick:dataset src="s3://bucket/sales" format="delta" />` and click
**Load from S3**. Configure credentials in the host Python environment, using
the reader's normal AWS credential chain. Keep credentials out of Markdown.
Preview requests time out after 30 seconds and show a recoverable error when
a reader, dataset or credential is unavailable. S3 requires network access;
local datasets work offline.

External dataset previews are live references. Python outputs are snapshots
of their last run; rerun the cell to refresh them. Viewing a dataset does not
execute a Python cell. A Python cell runs with the same permissions as other
executable cells and may write files when its code asks to do so.
</book:entry>

</book:doc>
