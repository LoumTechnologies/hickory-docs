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
