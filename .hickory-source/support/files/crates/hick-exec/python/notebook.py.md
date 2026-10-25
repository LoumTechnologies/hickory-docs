<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-exec/python/notebook.py

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `a381cc0ce2374daa99270d8cc830ba6b3543acbeb3d0b90d2f99a9a11946b6b2`.

<book:entry path="crates/hick-exec/python/notebook.py">
"""Rich notebook outputs. Embedded in each Python cell; no runtime import required."""
import base64 as _h_base64
import datetime as _h_datetime
import decimal as _h_decimal
import io as _h_io
import json as _h_json

_HICKORY_PREFIX = "\x1eHICKORY_DISPLAY "


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
</book:entry>

</book:doc>
