import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location("export_ci", Path(__file__).with_name("export-ci-site.py"))
export = importlib.util.module_from_spec(spec)
spec.loader.exec_module(export)


class ExportTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.run = Path(temporary.name)
        self.source = "# Captured, not the current checkout\n"
        self.report = dict(version=1, finished_at="2026-10-26T00:00:00Z", base_revision="a"*40, tested_revision=None,
            documents={"owner.md": dict(source=self.source, source_hash=export.digest(self.source))},
            definition_hashes={"owner.md": export.digest(self.source)},
            checks=[dict(id="selected", owner="owner.md", status="failed", argv=["/host/python3", "-c", "exit(1)"], needs=[], assertion="exit-zero", exit_code=1, stdout="out.log"),
                    dict(id="private", owner="private.md", status="passed")])
        (self.run / "out.log").write_text("failure evidence")
        self.selection = dict(version=1, checks=["selected"], include_logs=True)

    def bundle(self):
        (self.run / "report.json").write_text(json.dumps(self.report))
        return export.bundle(self.run, self.selection)

    def test_export_keeps_captured_failure_and_excludes_unselected_content(self):
        value = self.bundle()
        self.assertEqual(len(value["checks"]), 1)
        self.assertEqual(value["checks"][0]["source"], self.source)
        self.assertEqual(value["checks"][0]["status"], "failed")
        self.assertEqual(value["checks"][0]["logs"]["stdout"], "failure evidence")
        self.assertEqual(value["checks"][0]["argv"][0], "python3")
        self.assertIsNone(value["tested_revision"])
        self.assertNotIn("private.md", json.dumps(value))
        self.selection["include_logs"] = False
        self.assertEqual(self.bundle()["checks"][0]["logs"], {})

    def test_changed_or_missing_capture_is_refused(self):
        self.report["documents"]["owner.md"]["source"] = "changed"
        with self.assertRaises(ValueError): self.bundle()
        self.report["documents"] = {}
        with self.assertRaises(ValueError): self.bundle()

    def test_escape_and_duplicate_selection_are_refused(self):
        self.report["checks"][0]["stdout"] = "../outside.log"
        with self.assertRaises(ValueError): self.bundle()
        self.selection["checks"] *= 2
        with self.assertRaises(ValueError): self.bundle()


if __name__ == "__main__":
    unittest.main()
