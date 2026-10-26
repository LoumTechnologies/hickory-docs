<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Run explicitly selected checks from their owning explanations

For: maintainers composing focused CI checks and inspecting evidence for a revision.

A check belongs beside the behavior it exercises. The project policy selects stable
check IDs; the runner validates the entire declared catalog and dependency graph
before starting any process. Selected dependencies run first. Failure, timeout or
an unavailable command blocks dependent checks and makes the run fail. Independent
checks still run so one failure does not hide their results.

This first slice is a repository-local experiment, using ordinary Hickory
copy/file constructions to emit JSON. It adds no language tags and does not yet
execute `hick:exec` graphs. Commands use host tools, inherit the host environment,
and are trusted code with the runner's permissions. No sandbox, secret policy,
changed-file inference, caching, matrix, or distributed scheduling is claimed.
The manually dispatched GitHub workflow has read-only repository permissions and
no selected secrets. Existing merge and release checks keep their current policy.

From the repository root, inspect the plan, then explicitly execute it:

```sh
python3 scripts/literate-ci.py
python3 scripts/literate-ci.py --execute
python3 scripts/literate-ci.py --execute --site-output target/ci-site
```

The optional `--site-output` runs the static exporter after the checks, including
failed outcomes; a failing check still makes the command exit nonzero. Export
requires the existing web Node dependencies and a fresh destination.

Each execution creates a new directory under `target/literate-ci/`, with a JSON
report and per-check stdout/stderr logs. Results record commands, owner pages,
definition hashes, host identity, times and exit codes. A successful command means
its declared exit-code assertion passed; it does not certify the English contract.
`tested_revision` is populated only when Git is clean before and after execution
and HEAD is unchanged. Dirty runs retain the base HEAD and have no tested commit.
These observations do not freeze the working tree or detect transient edits during
a run. CI checkouts provide the intended immutable input boundary. Receipts are
local evidence, not signed attestations or a complete environment fingerprint.

## Evidence from the pilot

On 2026-10-26, local execution passed six runner safety tests and the single named
publication input-refusal witness selected below. The runner tests exercise graph
validation and selection, failure blocking, independent results, timeout and
missing-command outcomes, clean/dirty revision association, and receipt reuse
refusal. The local checkout was dirty, so the pilot receipt identifies no tested
commit. This does not claim a hosted execution or broad regression coverage.

## Validate every definition before executing the selected closure

Explicit catalog files avoid discovering executable code by globbing prose. Owner
paths and working directories must stay inside the repository. Stable IDs are
unique across the catalog; unknown dependencies and cycles are errors, including
in unselected checks. A plan never launches commands. `{python}` in argv selects
the interpreter running this tool without relying on a shell or a fixed path.

Resolve the full catalog and reject malformed plans before any side effect.

<book:copy id="ci-plan">
#!/usr/bin/env python3
"""Plan or execute explicitly selected, literately owned host checks."""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import re
import signal
import subprocess
import sys
import time
import uuid


def inside(root, relative):
    if not isinstance(relative, str) or not relative or Path(relative).is_absolute():
        raise ValueError("Expected a repository-relative path")
    path = (root / relative).resolve()
    if not path.is_relative_to(root.resolve()):
        raise ValueError("Path escapes repository: " + relative)
    return path


def load_plan(root, policy_path):
    policy = json.loads(inside(root, policy_path).read_text())
    if policy.get("version") != 1:
        raise ValueError("Unsupported CI policy version")
    if not isinstance(policy.get("definitions"), list) or not policy["definitions"]:
        raise ValueError("Policy needs explicit definition files")
    checks = {}
    hashes = {policy_path: sha(inside(root, policy_path).read_bytes())}
    for filename in policy["definitions"]:
        path = inside(root, filename)
        hashes[filename] = sha(path.read_bytes())
        definition = json.loads(path.read_text())
        if definition.get("version") != 1 or not isinstance(definition.get("checks"), list):
            raise ValueError("Unsupported check definition")
        for check in definition["checks"]:
            identity = check.get("id")
            if not isinstance(identity, str) or not re.fullmatch(r"[a-z][a-z0-9-]*", identity):
                raise ValueError("Invalid check ID")
            if identity in checks:
                raise ValueError("Duplicate check ID: " + identity)
            owner = inside(root, check["owner"])
            if not owner.is_file() or not inside(root, check["cwd"]).is_dir():
                raise ValueError("Owner or working directory is unavailable")
            hashes[check["owner"]] = sha(owner.read_bytes())
            argv = check["argv"]
            if not isinstance(argv, list) or not argv or any(not isinstance(a, str) or not a or "\0" in a for a in argv):
                raise ValueError("Check needs a nonempty argv")
            timeout = check["timeout_seconds"]
            if isinstance(timeout, bool) or not isinstance(timeout, (int, float)) or not math.isfinite(timeout) or timeout <= 0:
                raise ValueError("Check needs a finite positive timeout")
            if check.get("assertion") != "exit-zero":
                raise ValueError("Only explicit exit-zero assertions are supported")
            if not isinstance(check.get("needs"), list) or any(not isinstance(n, str) for n in check["needs"]):
                raise ValueError("Dependencies must be check IDs")
            checks[identity] = check
    visiting, visited, order = set(), set(), []

    def visit(identity, selected):
        if identity not in checks:
            raise ValueError("Unknown check: " + str(identity))
        if identity in visiting:
            raise ValueError("Dependency cycle: " + identity)
        if identity in visited:
            return
        visiting.add(identity)
        for dependency in checks[identity]["needs"]:
            visit(dependency, selected)
        visiting.remove(identity)
        visited.add(identity)
        if selected:
            order.append(checks[identity])

    for identity in checks:
        visit(identity, False)
    visited.clear()
    required = policy.get("required")
    if not isinstance(required, list) or not required or any(not isinstance(i, str) for i in required):
        raise ValueError("Policy needs required check IDs")
    for identity in required:
        visit(identity, True)
    return order, hashes


def sha(data):
    return hashlib.sha256(data).hexdigest()


</book:copy>

Use separate logs and bounded process time. On POSIX, stop the process group on timeout so child processes do not survive their check. Preserve failed and blocked outcomes in the report.

<book:copy id="ci-execution">
def timestamp():
    return datetime.now(timezone.utc).isoformat()


def git_state(root):
    def git(*args):
        return subprocess.check_output(["git", *args], cwd=root)
    return {"head": git("rev-parse", "HEAD").decode().strip(),
            "dirty": bool(git("status", "--porcelain", "--untracked-files=all"))}


def execute(root, plan, hashes, destination):
    destination.mkdir(parents=True, exist_ok=False)
    before = git_state(root)
    documents = {}
    for check in plan:
        source = inside(root, check["owner"]).read_bytes()
        source_hash = sha(source)
        if source_hash != hashes.get(check["owner"]):
            raise ValueError("Owning source changed after planning")
        documents[check["owner"]] = dict(source=source.decode("utf-8"), source_hash=source_hash)
    report = dict(version=1, started_at=timestamp(), base_revision=before["head"],
                  tested_revision=None, before=before, definition_hashes=hashes, documents=documents,
                  environment=dict(platform=platform.platform(), python=sys.version,
                                   executor="host", inherited_environment=True,
                                   runner_sha256=sha(Path(__file__).read_bytes())),
                  scope="Declared exit-zero checks only; no prose certification or sandbox.",
                  status="running", checks=[])
    report_path = destination / "report.json"

    def save():
        temporary = destination / "report.tmp"
        temporary.write_text(json.dumps(report, indent=2) + "\n")
        temporary.replace(report_path)

    save()
    outcomes = {}
    try:
        for check in plan:
            identity = check["id"]
            argv = [sys.executable if a == "{python}" else a for a in check["argv"]]
            result = dict(id=identity, owner=check["owner"], argv=argv, cwd=check["cwd"],
                          assertion=check["assertion"], needs=check["needs"],
                          timeout_seconds=check["timeout_seconds"], started_at=timestamp())
            started = time.monotonic()
            if any(outcomes[n] != "passed" for n in check["needs"]):
                result["status"] = "blocked"
            else:
                result.update(stdout=identity + ".stdout.log", stderr=identity + ".stderr.log")
                with (destination / result["stdout"]).open("wb") as out, (destination / result["stderr"]).open("wb") as err:
                    try:
                        process = subprocess.Popen(argv, cwd=inside(root, check["cwd"]),
                                                   stdout=out, stderr=err, start_new_session=os.name == "posix")
                        try:
                            code = process.wait(timeout=check["timeout_seconds"])
                            result.update(exit_code=code, status="passed" if code == 0 else "failed")
                        except subprocess.TimeoutExpired:
                            if os.name == "posix":
                                os.killpg(process.pid, signal.SIGKILL)
                            else:
                                process.kill()
                            process.wait()
                            result.update(status="timed_out", exit_code=process.returncode)
                        except BaseException:
                            if os.name == "posix":
                                os.killpg(process.pid, signal.SIGKILL)
                            else:
                                process.kill()
                            process.wait()
                            raise
                    except OSError as error:
                        result.update(status="unavailable", error=str(error))
            result.update(finished_at=timestamp(), elapsed_seconds=round(time.monotonic() - started, 3))
            outcomes[identity] = result["status"]
            report["checks"].append(result)
            print(identity + ": " + result["status"], flush=True)
            save()
        report["status"] = "passed" if all(v == "passed" for v in outcomes.values()) else "failed"
    finally:
        after = git_state(root)
        report.update(after=after, finished_at=timestamp())
        if report["status"] == "running":
            report["status"] = "interrupted"
        if not before["dirty"] and before == after:
            report["tested_revision"] = before["head"]
        save()
    return report


</book:copy>

The default action prints a reviewable plan. Execution requires an explicit flag and gets a fresh receipt directory, so an earlier passing result cannot be reused as the current run.

<book:copy id="ci-cli">
def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--policy", default="ci/policy.json")
    parser.add_argument("--execute", action="store_true")
    parser.add_argument("--site-output", type=Path, help="Export this run as a standalone static site")
    args = parser.parse_args()
    if args.site_output and not args.execute:
        raise ValueError("--site-output requires --execute")
    root = Path(__file__).resolve().parents[1]
    plan, hashes = load_plan(root, args.policy)
    if not args.execute:
        print(json.dumps({"checks": plan, "definition_hashes": hashes}, indent=2))
        return 0
    destination = root / "target/literate-ci" / uuid.uuid4().hex
    print("Results: " + str(destination), flush=True)
    report = execute(root, plan, hashes, destination)
    if args.site_output:
        subprocess.run([sys.executable, str(root / "scripts/export-ci-site.py"),
                        "--run", str(destination), "--output", str(args.site_output)], cwd=root, check=True)
    return 0 if report["status"] == "passed" else 1


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (ValueError, KeyError, TypeError, OSError, subprocess.SubprocessError) as error:
        print("CI refused: " + str(error), file=sys.stderr)
        sys.exit(1)
</book:copy>

Small temporary repositories test the planner and actual child processes. Unknown dependencies, duplicate IDs, cycles and escaping paths must be refused before execution. The plan must include dependencies even when only their consumer is required.

<book:copy id="ci-plan-tests">
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

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

</book:copy>

Run real success, failure and timeout commands. A blocked consumer must not create its marker, while independent checks still produce logs. Clean and dirty repository receipts must be distinguishable.

<book:copy id="ci-process-tests">
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

    def test_failed_checks_still_export_without_returning_success(self):
        with patch.object(ci.sys, "argv", ["literate-ci.py", "--execute", "--site-output", "target/example"]), patch.object(ci, "load_plan", return_value=([], {})), patch.object(ci, "execute", return_value={"status": "failed"}), patch.object(ci.subprocess, "run") as exported:
            self.assertEqual(ci.main(), 1)
            self.assertIn("--run", exported.call_args.args[0])
            self.assertIn("target/example", exported.call_args.args[0])

    def test_a_command_that_changes_inputs_cannot_certify_the_commit(self):
        plan, hashes = self.plan([self.check(code="open('owner.md', 'w').write('changed')")])
        (self.root / ".gitignore").write_text("results/\n")
        subprocess.run(["git", "-C", str(self.root), "add", "."], check=True)
        subprocess.run(["git", "-C", str(self.root), "-c", "user.name=Test", "-c", "user.email=test@example.com", "commit", "-qm", "inputs"], check=True)
        report = ci.execute(self.root, plan, hashes, self.root / "results")
        self.assertEqual(report["status"], "passed")
        self.assertIsNone(report["tested_revision"])
        self.assertTrue(report["after"]["dirty"])
        self.assertEqual(report["documents"]["owner.md"]["source"], "Explained behavior")


if __name__ == "__main__":
    unittest.main()
</book:copy>

The runner owns its own focused safety checks. A CI command needs an explicit assertion; this pilot uses the test runner’s exit code.

<book:copy id="ci-runner-check">
{
  "version": 1,
  "checks": [
    {
      "id": "ci-runner-safety",
      "owner": ".hickory-source/guarantees/verification/literate-ci/contract.md",
      "cwd": ".",
      "argv": [
        "{python}",
        "scripts/test-literate-ci.py"
      ],
      "needs": [],
      "assertion": "exit-zero",
      "timeout_seconds": 30
    }
  ]
}
</book:copy>

### Assemble `scripts/literate-ci.py`

<book:file path="scripts/literate-ci.py"><book:paste select="#ci-plan"/><book:paste select="#ci-execution"/><book:paste select="#ci-cli"/></book:file>

### Assemble `scripts/test-literate-ci.py`

<book:file path="scripts/test-literate-ci.py"><book:paste select="#ci-plan-tests"/><book:paste select="#ci-process-tests"/></book:file>

### Assemble `ci/checks/runner.json`

<book:file path="ci/checks/runner.json"><book:paste select="#ci-runner-check"/></book:file>


Read [the static export](site.md) to share selected captured documents and results.
<book:include file="site.md" />
</book:doc>
