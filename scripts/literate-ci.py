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
