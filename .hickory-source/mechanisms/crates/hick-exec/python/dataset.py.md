<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-exec/python/dataset.py

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-b3d02d2ac613"></a>
## `crates/hick-exec/python/dataset.py`

<a id="f-b3d02d2ac613-if_statement"></a>
### `if_statement`

Appended to notebook.py by the dataset preview endpoint.

<storycode:copy id="f-b3d02d2ac613-if_statement">
# Appended to notebook.py by the dataset preview endpoint.
if __name__ == "__main__":
    import sys
    try:
        import polars as pl
        request = _h_json.loads(sys.stdin.read())
        src = request["src"]
        if request["format"] == "delta":
            # Arrow resolves Delta log paths correctly even when the workspace
            # contains spaces (including macOS's Application Support). Keep the
            # scan lazy and honor the transaction log, including deletions.
            frame = pl.scan_delta(src, use_pyarrow=True)
        else:
            frame = pl.scan_parquet(src, glob=False)
        result = _h_table(frame, request["limit"], request["offset"])
        encoded = _h_json.dumps(result, allow_nan=False)
        if len(encoded) > 8 * 1024 * 1024:
            raise ValueError("Preview exceeds 8 MiB; request fewer rows")
        print(encoded)
    except ImportError:
        print("Install notebook readers in your Python environment: python -m pip install -r crates/hick-exec/python/requirements.txt", file=sys.stderr)
        sys.exit(1)
</storycode:copy>

### Assemble `crates/hick-exec/python/dataset.py`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-exec/python/dataset.py"><storycode:paste select="#f-b3d02d2ac613-if_statement"/></storycode:file>

</storycode:doc>
