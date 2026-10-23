#!/usr/bin/env python3
"""Prepare the selected Linux build for a human-owned issue 72 recording."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
REVISION = "c2d64095"
PACKAGE_HASH = "68c2041baacf7e04e6c659d698df8e2d06ef7a90fc374a3bce8e67cbf7614b63"
PACKAGE = Path("/tmp/hickory-issue72-native-1806f045/dist/desktop/hickory-docs-0.1.0-issue72.c2d64095-x86_64-unknown-linux-gnu.deb")


def credential():
    value = os.environ.get("DEEPSEEK_API_KEY", "").strip()
    if not value:
        # Parse exactly one value. Never execute .env or load unrelated secrets.
        source = ROOT / ".env"
        match = re.search(r"^[ \t]*(?:export[ \t]+)?DEEPSEEK_API_KEY[ \t]*=[ \t]*(.*?)[ \t]*$", source.read_text(), re.M) if source.is_file() else None
        if match:
            value = match.group(1).strip()
            if value[:1] in ("'", '"'):
                quote = value[0]
                if len(value) < 2 or quote not in value[1:]:
                    raise SystemExit("Malformed DEEPSEEK_API_KEY quotation")
                value = value[1:value.rfind(quote)]
            else:
                value = value.split(" #", 1)[0].strip()
    if not value:
        raise SystemExit("DEEPSEEK_API_KEY is missing from the environment and repository .env")
    return value


def prepare(package, output):
    if hashlib.sha256(package.read_bytes()).hexdigest() != PACKAGE_HASH:
        raise SystemExit("Package checksum differs from the selected build; nothing launched")
    python = ROOT / "apps/agent/.venv/bin/python"
    adapter = ROOT / ".hick-cache/adapters/python"
    if not python.is_file() or not (adapter / "bin/python").is_file():
        raise SystemExit("Configured Python worker/debugpy runtime is missing; nothing launched")
    # A fresh directory prevents accidental reuse of a failed/ended practice.
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    subprocess.run(["dpkg-deb", "-x", str(package), str(output / "unpacked")], check=True)
    project = output / "project"
    project.mkdir()
    (project / "note.md").write_text("# Native recording workspace\n\nOpen AI engineer tour, then create a private prediction practice.\n")
    (project / ".hick-cache/adapters").mkdir(parents=True)
    (project / ".hick-cache/adapters/python").symlink_to(adapter, target_is_directory=True)
    config = output / "config/com.loumtechnologies.hickorydocs/agents"
    config.mkdir(parents=True)
    (config / "python.json").write_text(json.dumps({"python": str(python)}))
    spec = subprocess.check_output(["git", "show", f"{REVISION}:apps/web/e2e/privatePredictionCoach.spec.ts"], cwd=ROOT, text=True)
    prompt = re.search(r'\.fill\("(Run this coached prediction exercise\..*?)"\);', spec)
    if not prompt:
        raise SystemExit("The selected request moved; update this recording helper before use")
    (output / "request.txt").write_text(json.loads('"' + prompt.group(1) + '"') + "\n")
    manifest = {"functional_revision": subprocess.check_output(["git", "rev-parse", REVISION], cwd=ROOT, text=True).strip(),
                "package_sha256": PACKAGE_HASH, "package": str(package),
                "python": str(python), "debug_adapter": str(adapter),
                "fixture_source_sha256": hashlib.sha256(subprocess.check_output(["git", "show", f"{REVISION}:examples/ai-engineer-private-deadline.md"], cwd=ROOT)).hexdigest(),
                "request_sha256": hashlib.sha256((output / "request.txt").read_bytes()).hexdigest(),
                "provider": "deepseek", "model": "deepseek-chat",
                "human_review": "pending", "recording": "pending", "published": False}
    (output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    return project


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--package", type=Path, default=PACKAGE)
    parser.add_argument("--output", type=Path, help="New directory; existing directories are refused")
    parser.add_argument("--launch", action="store_true", help="Launch the owned native app; only your Delegate action sends a model request")
    args = parser.parse_args()
    # Resolve the credential before making files when a launch was requested.
    key = credential() if args.launch else None
    output = args.output.absolute() if args.output else Path(tempfile.mkdtemp(prefix="hickory-recording-parent-")) / "take-1"
    project = prepare(args.package.resolve(), output)
    print(f"Recording directory: {output}\nRequest to paste: {output / 'request.txt'}")
    if args.launch:
        # Keep desktop session variables; withhold unrelated provider/account keys.
        env = {k: v for k, v in os.environ.items() if not k.startswith("HICKORY_") and not k.endswith(("_API_KEY", "_TOKEN"))}
        env.update(HICKORY_EXECUTOR="local", HICKORY_STATE_DIR=str(output / "state"),
                   HICKORY_PROJECT_DIR=str(project), XDG_CONFIG_HOME=str(output / "config"),
                   XDG_DATA_HOME=str(output / "data"), XDG_CACHE_HOME=str(output / "cache"),
                   HICKORY_LLM_PROVIDER="deepseek", DEEPSEEK_MODEL="deepseek-chat", DEEPSEEK_API_KEY=key)
        with (output / "native.log").open("wb") as log:
            process = subprocess.Popen([str(output / "unpacked/usr/bin/hickory-desktop"), str(project / "note.md")], env=env, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
        print(f"Owned native PID: {process.pid}. End the private practice before closing its window.")
        (output / "launch.json").write_text(json.dumps({"pid": process.pid, "provider_request_authorized_by": "human Delegate", "credentials_printed": False}) + "\n")


if __name__ == "__main__":
    main()
