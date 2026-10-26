<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-exec/python/test_notebook.py

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-5f80873534a0"></a>
## `crates/hick-exec/python/test_notebook.py`

<a id="f-5f80873534a0-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-5f80873534a0-expression_statement">
"""Run with: python -m unittest discover -s crates/hick-exec/python -p 'test_*.py'."""
</storycode:copy>
<a id="f-5f80873534a0-contextlib"></a>
### `contextlib`

<storycode:copy id="f-5f80873534a0-contextlib">
import contextlib
</storycode:copy>
<a id="f-5f80873534a0-io"></a>
### `io`

<storycode:copy id="f-5f80873534a0-io">
import io
</storycode:copy>
<a id="f-5f80873534a0-json"></a>
### `json`

<storycode:copy id="f-5f80873534a0-json">
import json
</storycode:copy>
<a id="f-5f80873534a0-pathlib"></a>
### `pathlib`

<storycode:copy id="f-5f80873534a0-pathlib">
import pathlib
</storycode:copy>
<a id="f-5f80873534a0-tempfile"></a>
### `tempfile`

<storycode:copy id="f-5f80873534a0-tempfile">
import tempfile
</storycode:copy>
<a id="f-5f80873534a0-unittest"></a>
### `unittest`

<storycode:copy id="f-5f80873534a0-unittest">
import unittest
</storycode:copy>
<a id="f-5f80873534a0-subprocess"></a>
### `subprocess`

<storycode:copy id="f-5f80873534a0-subprocess">
import subprocess
</storycode:copy>
<a id="f-5f80873534a0-sys"></a>
### `sys`

<storycode:copy id="f-5f80873534a0-sys">
import sys

</storycode:copy>
<a id="f-5f80873534a0-polars-as-pl"></a>
### `polars as pl`

<storycode:copy id="f-5f80873534a0-polars-as-pl">
import polars as pl
</storycode:copy>
<a id="f-5f80873534a0-DeltaTable"></a>
### `DeltaTable`

<storycode:copy id="f-5f80873534a0-DeltaTable">
from deltalake import DeltaTable
</storycode:copy>
<a id="f-5f80873534a0-display"></a>
### `display`

<storycode:copy id="f-5f80873534a0-display">
from notebook import display, _h_table, _HICKORY_PREFIX


</storycode:copy>
<a id="f-5f80873534a0-NotebookTest"></a>
### `NotebookTest`

<storycode:copy id="f-5f80873534a0-NotebookTest">
class NotebookTest(unittest.TestCase):
    def test_dataset_entrypoint_reads_delta_log_in_a_path_with_spaces(self):
        with tempfile.TemporaryDirectory(prefix="hickory dataset ") as directory:
            root = pathlib.Path(directory)
            pl.DataFrame({"id": [1, 2, 3]}).write_delta(root / "delta table")
            DeltaTable(str(root / "delta table")).delete("id = 1")
            scripts = pathlib.Path(__file__).parent
            program = (scripts / "notebook.py").read_text() + "\n" + (scripts / "dataset.py").read_text()
            result = subprocess.run([sys.executable, "-c", program], input=json.dumps({
                "src": str(root / "delta table"), "format": "delta", "limit": 1, "offset": 1,
            }), text=True, capture_output=True, timeout=30)
            self.assertEqual(result.returncode, 0, result.stderr)
            preview = json.loads(result.stdout)
            self.assertEqual(preview["rows"], [[3]])
            # Lazy preview deliberately avoids a full count scan.
            self.assertIsNone(preview["total"])
            self.assertFalse(preview["has_more"])

    def test_display_keeps_types_nulls_nested_values_and_limits(self):
        frame = pl.DataFrame({"number": [3, None, 10], "list": [[1], [], [2, 3]]})
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            display(frame, limit=2)
        value = json.loads(buf.getvalue().removeprefix(_HICKORY_PREFIX))
        self.assertEqual(value["rows"], [[3, [1]], [None, []]])
        self.assertEqual(value["schema"][0], {"name": "number", "dtype": "Int64"})
        self.assertTrue(value["has_more"])
        self.assertEqual(value["total"], 3)

    def test_parquet_and_delta_have_equal_paged_views_and_delta_uses_log(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            frame = pl.DataFrame({"n": range(205), "text": ["<b>|hello"] * 205})
            frame.write_parquet(root / "data.parquet")
            frame.write_delta(root / "delta")
            DeltaTable(str(root / "delta")).delete("n < 5")
            parquet = _h_table(pl.scan_parquet(root / "data.parquet"), 100, 100)
            delta = _h_table(pl.scan_delta(str(root / "delta")), 100, 0)
            self.assertEqual(parquet["rows"][0][0], 100)
            self.assertTrue(parquet["has_more"])
            self.assertEqual(delta["rows"][0][0], 5)
            self.assertEqual(delta["schema"], parquet["schema"])
            self.assertEqual(_h_table(pl.scan_parquet(root / "data.parquet"), 100, 200)["rows"], [list(r) for r in frame.tail(5).rows()])

    def test_preview_preserves_large_integers_and_caps_wide_schemas(self):
        frame = pl.DataFrame({"id": [9223372036854775807]})
        self.assertEqual(_h_table(frame)["rows"], [["9223372036854775807"]])
        wide = pl.DataFrame({f"c{i}": [i] for i in range(260)})
        result = _h_table(wide.lazy())
        self.assertEqual(len(result["schema"]), 256)
        self.assertEqual(len(result["rows"][0]), 256)
        self.assertTrue(result["columns_truncated"])

    def test_svg_and_text_generate_separate_safe_records(self):
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            display('<svg xmlns="http://www.w3.org/2000/svg"></svg>')
            display("hello")
        records = [json.loads(line.removeprefix(_HICKORY_PREFIX)) for line in buf.getvalue().split("\n") if line]
        self.assertEqual(records[0]["mime"], "image/svg+xml")
        self.assertEqual(records[1], {"kind": "text", "text": "hello"})


</storycode:copy>
<a id="f-5f80873534a0-if_statement"></a>
### `if_statement`

<storycode:copy id="f-5f80873534a0-if_statement">
if __name__ == "__main__":
    unittest.main()
</storycode:copy>

### Assemble `crates/hick-exec/python/test_notebook.py`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-exec/python/test_notebook.py"><storycode:paste select="#f-5f80873534a0-expression_statement"/><storycode:paste select="#f-5f80873534a0-contextlib"/><storycode:paste select="#f-5f80873534a0-io"/><storycode:paste select="#f-5f80873534a0-json"/><storycode:paste select="#f-5f80873534a0-pathlib"/><storycode:paste select="#f-5f80873534a0-tempfile"/><storycode:paste select="#f-5f80873534a0-unittest"/><storycode:paste select="#f-5f80873534a0-subprocess"/><storycode:paste select="#f-5f80873534a0-sys"/><storycode:paste select="#f-5f80873534a0-polars-as-pl"/><storycode:paste select="#f-5f80873534a0-DeltaTable"/><storycode:paste select="#f-5f80873534a0-display"/><storycode:paste select="#f-5f80873534a0-NotebookTest"/><storycode:paste select="#f-5f80873534a0-if_statement"/></storycode:file>

</storycode:doc>
