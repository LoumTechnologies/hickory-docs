#!/usr/bin/env python3
"""Export selected local CI evidence as a static Hickory viewer."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys


def digest(text):
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def contained_file(root, relative):
    if not isinstance(relative, str) or Path(relative).is_absolute():
        raise ValueError("Expected a relative captured log path")
    path = (root / relative).resolve()
    if not path.is_relative_to(root.resolve()) or not path.is_file():
        raise ValueError("Captured log is missing or escapes the run")
    return path


def bundle(run, selection):
    report = json.loads((run / "report.json").read_text())
    if report.get("version") != 1 or selection.get("version") != 1:
        raise ValueError("Unsupported run or selection version")
    selected = selection.get("checks")
    if not isinstance(selected, list) or not selected or len(set(selected)) != len(selected):
        raise ValueError("Select unique public check IDs")
    captured = report.get("documents", {})
    checks = {c["id"]: c for c in report["checks"]}
    if len(checks) != len(report["checks"]):
        raise ValueError("Duplicate captured check IDs")
    published = []
    for identity in selected:
        if not isinstance(identity, str) or not re.fullmatch(r"[a-z][a-z0-9-]*", identity) or identity not in checks:
            raise ValueError("Selected check is absent from the receipt")
        check = checks[identity]
        if check["status"] not in ("passed", "failed", "blocked", "timed_out", "unavailable"):
            raise ValueError("Incomplete check outcome")
        document = captured.get(check["owner"])
        if not document or digest(document["source"]) != document["source_hash"] or report["definition_hashes"].get(check["owner"]) != document["source_hash"]:
            raise ValueError("Missing or changed captured owner")
        logs = {}
        for kind in ("stdout", "stderr"):
            if selection.get("include_logs") is True and check.get(kind):
                data = contained_file(run, check[kind]).read_bytes()
                if len(data) > 1024 * 1024:
                    raise ValueError("Selected log exceeds the 1 MiB preview limit")
                logs[kind] = data.decode("utf-8", errors="replace")
        argv = list(check["argv"])
        if argv and Path(argv[0]).is_absolute():
            argv[0] = Path(argv[0]).name
        published.append(dict(id=identity, owner=check["owner"], source=document["source"],
            source_hash=document["source_hash"], status=check["status"], argv=argv,
            needs=check.get("needs", []), assertion=check["assertion"],
            exit_code=check.get("exit_code"), elapsed_seconds=check.get("elapsed_seconds"), logs=logs))
    return dict(format="hickory-ci-publication", version=1, title="Literate CI, run locally",
                captured_at=report["finished_at"], base_revision=report["base_revision"],
                tested_revision=report.get("tested_revision"), checks=published,
                scope="Selected historical command outcomes; opening this page executes no checks.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--selection", type=Path, default=Path("ci/public-preview.json"))
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    value = bundle(args.run.resolve(), json.loads(args.selection.read_text()))
    destination = args.output.resolve()
    if destination.exists():
        raise ValueError("Use a fresh output directory")
    subprocess.run(["npm", "exec", "--", "vite", "build", "--config", "vite.ci.config.ts"], cwd=root / "apps/web", check=True)
    shutil.copytree(root / "apps/web/dist-ci", destination)
    (destination / "index.html").write_bytes((destination / "ci.html").read_bytes())
    data = destination / "ci-preview/bundle.json"
    data.parent.mkdir()
    data.write_text(json.dumps(value, indent=2) + "\n")
    print("Static CI site: " + str(destination))
    print("Serve this directory over HTTP, or upload it to your static host.")


if __name__ == "__main__":
    try:
        main()
    except (ValueError, KeyError, OSError, subprocess.SubprocessError) as error:
        print("Export refused: " + str(error), file=sys.stderr)
        sys.exit(1)
