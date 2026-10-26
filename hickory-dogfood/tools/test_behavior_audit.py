import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest


AUDIT = Path(__file__).with_name("behavior.py")
NAMESPACE = "http://www.hickorydocs.com/1.0"


class BehaviorAuditTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="hickory-audit-witness-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.base = self.root / "hickory-dogfood"
        (self.base / "tools").mkdir(parents=True)
        shutil.copy2(AUDIT, self.base / "tools/behavior.py")
        self.source = ".hickory-source/behavior.md"
        (self.root / ".hickory-source").mkdir()
        self.prose = ('<s:doc xmlns:s="' + NAMESPACE + '">\n'
            '<s:copy id="decision">accepted\n</s:copy>\n'
            '<s:file path="one.py"><s:paste select="#decision"/></s:file>\n'
            '<s:entry path="two.py">literal\n</s:entry>\n</s:doc>\n')
        self.document = self.root / self.source
        self.document.write_text(self.prose)
        (self.root / "project.md").write_text("Source index\n")
        ownership = {"constructors": {name: {"source": self.source}
            for name in ("one.py", "two.py")}, "contracts": {},
            "evidence_records": [], "freeform_specs": [], "unmapped_implementation": []}
        (self.base / "behavior-map.json").write_text(json.dumps(ownership))
        (self.base / "baseline.json").write_text(json.dumps({"entries": [
            {"path": "one.py"}, {"path": "two.py"}]}))
        (self.base / "narrative-map.json").write_text(json.dumps({
            "fragments": [{"id": "decision", "page": self.source}],
            "explanation_gaps": {}}))

    def audit(self):
        return subprocess.run([sys.executable, str(self.base / "tools/behavior.py"), "audit"],
            cwd=self.root, capture_output=True, text=True)

    def test_several_outputs_can_share_one_narrative(self):
        result = self.audit()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_duplicate_output_is_refused(self):
        self.document.write_text(self.prose.replace("</s:doc>",
            '<s:entry path="one.py">duplicate</s:entry></s:doc>'))
        result = self.audit()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Duplicate or misplaced construction", result.stdout + result.stderr)

    def test_duplicate_fragment_is_refused(self):
        self.document.write_text(self.prose.replace("</s:doc>",
            '<s:copy id="decision">another</s:copy></s:doc>'))
        result = self.audit()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("more than one definition", result.stdout + result.stderr)

    def test_missing_selected_fragment_is_refused(self):
        self.document.write_text(self.prose.replace('select="#decision"', 'select="#absent"'))
        result = self.audit()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Unresolved selected fragment", result.stdout + result.stderr)
