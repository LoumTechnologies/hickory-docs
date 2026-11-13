<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-exec/python/notebook.py

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-bd7da3c75a90"></a>
## `crates/hick-exec/python/notebook.py`

<a id="f-bd7da3c75a90-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-bd7da3c75a90-expression_statement">
"""Rich notebook outputs. Embedded in each Python cell; no runtime import required."""
</storycode:copy>
<a id="f-bd7da3c75a90-base64-as-_h_base64"></a>
### `base64 as _h_base64`

<storycode:copy id="f-bd7da3c75a90-base64-as-_h_base64">
import base64 as _h_base64
</storycode:copy>
<a id="f-bd7da3c75a90-datetime-as-_h_datetime"></a>
### `datetime as _h_datetime`

<storycode:copy id="f-bd7da3c75a90-datetime-as-_h_datetime">
import datetime as _h_datetime
</storycode:copy>
<a id="f-bd7da3c75a90-decimal-as-_h_decimal"></a>
### `decimal as _h_decimal`

<storycode:copy id="f-bd7da3c75a90-decimal-as-_h_decimal">
import decimal as _h_decimal
</storycode:copy>
<a id="f-bd7da3c75a90-io-as-_h_io"></a>
### `io as _h_io`

<storycode:copy id="f-bd7da3c75a90-io-as-_h_io">
import io as _h_io
</storycode:copy>
<a id="f-bd7da3c75a90-json-as-_h_json"></a>
### `json as _h_json`

<storycode:copy id="f-bd7da3c75a90-json-as-_h_json">
import json as _h_json

</storycode:copy>
<a id="f-bd7da3c75a90-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-bd7da3c75a90-expression_statement-2">
_HICKORY_PREFIX = "\x1eHICKORY_DISPLAY "


</storycode:copy>
<a id="f-bd7da3c75a90-_h_value"></a>
### `_h_value`

<storycode:copy id="f-bd7da3c75a90-_h_value">
def _h_value(value):
    if isinstance(value, int) and not isinstance(value, bool) and abs(value) > 9007199254740991:
        return str(value)  # Preserve Int64/UInt64 exactly in JavaScript viewers.
    if value is None or isinstance(value, (str, bool, int)):
        return value
    if isinstance(value, float):
        import math
        return value if math.isfinite(value) else str(value)
    if isinstance(value, (_h_datetime.date, _h_datetime.time, _h_decimal.Decimal)):
        return str(value)
    if isinstance(value, bytes):
        return "0x" + value.hex()
    if isinstance(value, (list, tuple)):
        return [_h_value(v) for v in value]
    if isinstance(value, dict):
        return {str(k): _h_value(v) for k, v in value.items()}
    return str(value)


</storycode:copy>
<a id="f-bd7da3c75a90-_h_table"></a>
### `_h_table`

<storycode:copy id="f-bd7da3c75a90-_h_table">
def _h_table(frame, limit=100, offset=0):
    """Keep native nulls and nested values, with schema separate from row data."""
    import polars as pl
    if isinstance(frame, pl.LazyFrame):
        schema = frame.collect_schema()
        sample = frame.slice(offset, limit + 1).collect()
        total = None  # Avoid scanning the whole dataset for a preview count.
    else:
        schema = frame.schema
        sample = frame.slice(offset, limit + 1)
        total = frame.height
    names = list(schema.names())
    if len(names) > 256:
        names = names[:256]
        sample = sample.select(names)
    return {
        "kind": "table",
        "schema": [{"name": n, "dtype": str(schema[n])} for n in names],
        "rows": [[_h_value(v) for v in row] for row in sample.head(limit).rows()],
        "offset": offset, "has_more": sample.height > limit, "total": total,
        "columns_truncated": len(schema) > len(names),
    }


</storycode:copy>
<a id="f-bd7da3c75a90-display"></a>
### `display`

A Python cell can publish a small ordinary list of rows with display_table, without installing a dataframe library. Its declared table inputs are copies of typed values from the host graph. Raw reads are explicit, and undeclared reads fail with a next step rather than borrowing hidden kernel state. Every display stays a read-only stdout record; equals-prefixed strings remain literal output data.

<storycode:copy id="f-bd7da3c75a90-display">
def display(value, *, mime=None, limit=100):
    """Display a Polars/pandas/Arrow table, matplotlib figure, PNG, or SVG."""
    limit = max(1, min(int(limit), 1000))
    module = type(value).__module__.split(".")[0]
    if module == "pandas":
        import polars as pl
        value = pl.from_pandas(value)
        module = "polars"
    elif module == "pyarrow":
        import polars as pl
        value = pl.from_arrow(value)
        module = "polars"
    if module == "polars":
        if type(value).__name__ == "Series":
            value = value.to_frame()
        result = _h_table(value, limit)
    elif hasattr(value, "savefig"):
        buf = _h_io.BytesIO()
        value.savefig(buf, format="png", bbox_inches="tight")
        result = {"kind": "image", "mime": "image/png", "data": _h_base64.b64encode(buf.getvalue()).decode()}
    elif mime in ("image/png", "image/jpeg", "image/webp", "image/svg+xml") or hasattr(value, "_repr_svg_"):
        if hasattr(value, "_repr_svg_") and mime is None:
            value, mime = value._repr_svg_(), "image/svg+xml"
        raw = value.encode() if isinstance(value, str) else bytes(value)
        result = {"kind": "image", "mime": mime, "data": _h_base64.b64encode(raw).decode()}
    elif isinstance(value, str) and value.lstrip().startswith("<svg"):
        result = {"kind": "image", "mime": "image/svg+xml", "data": _h_base64.b64encode(value.encode()).decode()}
    else:
        result = {"kind": "text", "text": str(value)}
    encoded = _h_json.dumps(result, ensure_ascii=True, allow_nan=False)
    if len(encoded) > 8 * 1024 * 1024:
        raise ValueError("Display exceeds 8 MiB; reduce the table limit or image size")
    print(_HICKORY_PREFIX + encoded, flush=True)

# Named notebook inputs are immutable snapshots, not shared kernel variables.
_HICKORY_TABLES = {}
_HICKORY_RAW_TABLES = {}

class _HickoryTable:
    def __init__(self, value):
        # Copy so modifying a local Python result cannot mutate another read.
        value = _h_json.loads(_h_json.dumps(value))
        self.columns = value["columns"]
        self.rows = value["rows"]

    def column(self, name):
        if self.columns.count(name) != 1:
            raise KeyError(f"Column {name!r} must identify one column; inspect table.columns")
        at = self.columns.index(name)
        return [row[at] if at < len(row) else None for row in self.rows]

    def __getitem__(self, name):
        return self.column(name)


def table(name, *, raw=False):
    """Read a declared tables= input, including a range/column selector."""
    values = _HICKORY_RAW_TABLES if raw else _HICKORY_TABLES
    if name not in values:
        raise KeyError(f"Table input {name!r} is unavailable; declare it in this code block's tables= attribute and run its producers")
    return _HickoryTable(values[name])


def display_table(columns, rows, *, limit=100):
    """Publish an ordinary Python iterable as a read-only table; no packages needed."""
    import itertools
    limit = max(1, min(int(limit), 1000))
    columns = list(columns)
    if not columns or len(columns) > 256 or any(not isinstance(name, str) for name in columns):
        raise ValueError("Use between 1 and 256 text column names")
    sample = list(itertools.islice(iter(rows), limit + 1))
    if any(not isinstance(row, (list, tuple)) or len(row) != len(columns) for row in sample):
        raise ValueError("Each output row must have one value for every column")
    result = {"kind": "table", "schema": [{"name": name, "dtype": "value"} for name in columns],
              "rows": [[_h_value(value) for value in row] for row in sample[:limit]],
              "offset": 0, "has_more": len(sample) > limit, "total": len(sample) if len(sample) <= limit else None}
    encoded = _h_json.dumps(result, ensure_ascii=True, allow_nan=False)
    if len(encoded) > 8 * 1024 * 1024:
        raise ValueError("Display exceeds 8 MiB; reduce the table limit")
    print(_HICKORY_PREFIX + encoded, flush=True)
</storycode:copy>

### Assemble `crates/hick-exec/python/notebook.py`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-exec/python/notebook.py"><storycode:paste select="#f-bd7da3c75a90-expression_statement"/><storycode:paste select="#f-bd7da3c75a90-base64-as-_h_base64"/><storycode:paste select="#f-bd7da3c75a90-datetime-as-_h_datetime"/><storycode:paste select="#f-bd7da3c75a90-decimal-as-_h_decimal"/><storycode:paste select="#f-bd7da3c75a90-io-as-_h_io"/><storycode:paste select="#f-bd7da3c75a90-json-as-_h_json"/><storycode:paste select="#f-bd7da3c75a90-expression_statement-2"/><storycode:paste select="#f-bd7da3c75a90-_h_value"/><storycode:paste select="#f-bd7da3c75a90-_h_table"/><storycode:paste select="#f-bd7da3c75a90-display"/></storycode:file>

</storycode:doc>
