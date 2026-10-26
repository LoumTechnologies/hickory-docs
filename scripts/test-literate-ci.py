import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

spec = importlib.util.spec_from_file_location("literate_ci", Path(__file__).with_name("literate-ci.py"))
ci = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ci)


class LiterateCITests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        (self.root / "owner.md").write_text("Explained behavior")
        subprocess.run(["git", "init", "-q", str(self.root)], check=True)
        subprocess.run(["git", "-C", str(self.root), "-c", "user.name=Test", "-c", "user.email=test@example.com", "commit", "--allow-empty", "-qm", "fixture"], check=True)

    def check(self, identity="first", needs=None, code="pass"):
        return dict(id=identity, owner="owner.md", cwd=".", argv=["{python}", "-c", code],
                    needs=needs or [], assertion="exit-zero", timeout_seconds=5)

    def plan(self, checks, required=None):
        (self.root / "definitions.json").write_text(json.dumps(dict(version=1, checks=checks)))
        (self.root / "policy.json").write_text(json.dumps(dict(version=1, definitions=["definitions.json"], required=required or [checks[-1]["id"]])))
        return ci.load_plan(self.root, "policy.json")

    def test_dependency_order_and_explicit_selection(self):
        plan, hashes = self.plan([self.check("unused"), self.check("second", ["first"]), self.check()], ["second"])
        self.assertEqual([c["id"] for c in plan], ["first", "second"])
        self.assertEqual(set(hashes), {"owner.md", "policy.json", "definitions.json"})
        self.assertFalse((self.root / "target").exists())

    def test_invalid_catalog_is_refused_even_when_unselected(self):
        for checks in ([self.check(), self.check()],
                       [self.check(needs=["missing"])],
                       [self.check(needs=["second"]), self.check("second", ["first"])],
                       [dict(self.check(), owner="../outside.md")],
                       [dict(self.check(), timeout_seconds=True)],
                       [dict(self.check(), timeout_seconds=float("inf"))],
                       [dict(self.check(), assertion="anything")]):
            with self.subTest(checks=checks), self.assertRaises(ValueError):
                self.plan(checks)
        with self.assertRaises(ValueError):
            self.plan([self.check(), self.check("unused", ["missing"])], ["first"])

    def test_failure_blocks_dependents_but_keeps_independent_results(self):
        plan, hashes = self.plan([self.check(code="print('failure'); raise SystemExit(3)"),
            self.check("consumer", ["first"], "open('marker', 'w').write('wrong')"),
            self.check("independent", code="print('evidence')")], ["consumer", "independent"])
        report = ci.execute(self.root, plan, hashes, self.root / "results")
        self.assertEqual([c["status"] for c in report["checks"]], ["failed", "blocked", "passed"])
        self.assertEqual(report["status"], "failed")
        self.assertIsNone(report["tested_revision"])
        self.assertFalse((self.root / "marker").exists())
        self.assertEqual((self.root / "results/independent.stdout.log").read_text(), "evidence\n")
        self.assertEqual(json.loads((self.root / "results/report.json").read_text()), report)

    def test_timeout_and_unavailable_commands_fail(self):
        checks = [dict(self.check(code="import time; time.sleep(30)"), timeout_seconds=0.05),
                  dict(self.check("missing"), argv=["/unavailable/hickory-command"])]
        plan, hashes = self.plan(checks, ["first", "missing"])
        report = ci.execute(self.root, plan, hashes, self.root / "results")
        self.assertEqual([c["status"] for c in report["checks"]], ["timed_out", "unavailable"])
        self.assertEqual(report["status"], "failed")

    def test_clean_receipt_identifies_commit_and_refuses_result_reuse(self):
        plan, hashes = self.plan([self.check()])
        (self.root / ".gitignore").write_text("results/\n")
        subprocess.run(["git", "-C", str(self.root), "add", "."], check=True)
        subprocess.run(["git", "-C", str(self.root), "-c", "user.name=Test", "-c", "user.email=test@example.com", "commit", "-qm", "inputs"], check=True)
        report = ci.execute(self.root, plan, hashes, self.root / "results")
        self.assertEqual(report["status"], "passed")
        self.assertEqual(report["tested_revision"], report["base_revision"])
        with self.assertRaises(FileExistsError):
            ci.execute(self.root, plan, hashes, self.root / "results")

    def test_a_command_that_changes_inputs_cannot_certify_the_commit(self):
        plan, hashes = self.plan([self.check(code="open('owner.md', 'w').write('changed')")])
        (self.root / ".gitignore").write_text("results/\n")
        subprocess.run(["git", "-C", str(self.root), "add", "."], check=True)
        subprocess.run(["git", "-C", str(self.root), "-c", "user.name=Test", "-c", "user.email=test@example.com", "commit", "-qm", "inputs"], check=True)
        report = ci.execute(self.root, plan, hashes, self.root / "results")
        self.assertEqual(report["status"], "passed")
        self.assertIsNone(report["tested_revision"])
        self.assertTrue(report["after"]["dirty"])


if __name__ == "__main__":
    unittest.main()
