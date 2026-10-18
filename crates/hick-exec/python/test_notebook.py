"""Run with: python -m unittest discover -s crates/hick-exec/python -p 'test_*.py'."""
import contextlib
import io
import json
import pathlib
import tempfile
import unittest

import polars as pl
from deltalake import DeltaTable
from notebook import display, _h_table, _HICKORY_PREFIX


class NotebookTest(unittest.TestCase):
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


if __name__ == "__main__":
    unittest.main()
