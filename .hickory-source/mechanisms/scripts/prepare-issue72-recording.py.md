<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain scripts/prepare-issue72-recording.py

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-6307e15986a3"></a>
## `scripts/prepare-issue72-recording.py`

<a id="f-6307e15986a3-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-6307e15986a3-expression_statement">
#!/usr/bin/env python3
"""Prepare the selected Linux build for a human-owned issue 72 recording."""
</storycode:copy>
<a id="f-6307e15986a3-argparse"></a>
### `argparse`

<storycode:copy id="f-6307e15986a3-argparse">
import argparse
</storycode:copy>
<a id="f-6307e15986a3-hashlib"></a>
### `hashlib`

<storycode:copy id="f-6307e15986a3-hashlib">
import hashlib
</storycode:copy>
<a id="f-6307e15986a3-json"></a>
### `json`

<storycode:copy id="f-6307e15986a3-json">
import json
</storycode:copy>
<a id="f-6307e15986a3-os"></a>
### `os`

<storycode:copy id="f-6307e15986a3-os">
import os
</storycode:copy>
<a id="f-6307e15986a3-Path"></a>
### `Path`

<storycode:copy id="f-6307e15986a3-Path">
from pathlib import Path
</storycode:copy>
<a id="f-6307e15986a3-re"></a>
### `re`

<storycode:copy id="f-6307e15986a3-re">
import re
</storycode:copy>
<a id="f-6307e15986a3-subprocess"></a>
### `subprocess`

<storycode:copy id="f-6307e15986a3-subprocess">
import subprocess
</storycode:copy>
<a id="f-6307e15986a3-tempfile"></a>
### `tempfile`

<storycode:copy id="f-6307e15986a3-tempfile">
import tempfile

</storycode:copy>
<a id="f-6307e15986a3-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-6307e15986a3-expression_statement-2">
ROOT = Path(__file__).resolve().parents[1]
</storycode:copy>
<a id="f-6307e15986a3-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-6307e15986a3-expression_statement-3">
REVISION = "c2d64095"
</storycode:copy>
<a id="f-6307e15986a3-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-6307e15986a3-expression_statement-4">
PACKAGE_HASH = "68c2041baacf7e04e6c659d698df8e2d06ef7a90fc374a3bce8e67cbf7614b63"
</storycode:copy>
<a id="f-6307e15986a3-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-6307e15986a3-expression_statement-5">
PACKAGE = ROOT / ".hick-cache/issue72-recording/hickory-docs-0.1.0-issue72.c2d64095-x86_64-unknown-linux-gnu.deb"


</storycode:copy>
<a id="f-6307e15986a3-credential"></a>
### `credential`

<storycode:copy id="f-6307e15986a3-credential">
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


</storycode:copy>
<a id="f-6307e15986a3-prepare"></a>
### `prepare`

<storycode:copy id="f-6307e15986a3-prepare">
def prepare(package, output):
    if not package.is_file():
        raise SystemExit("Selected recording package is missing; restore the pinned local package before launching")
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


</storycode:copy>
<a id="f-6307e15986a3-main"></a>
### `main`

<storycode:copy id="f-6307e15986a3-main">
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


</storycode:copy>
<a id="f-6307e15986a3-if_statement"></a>
### `if_statement`

<storycode:copy id="f-6307e15986a3-if_statement">
if __name__ == "__main__":
    main()
</storycode:copy>

### Assemble `scripts/prepare-issue72-recording.py`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="scripts/prepare-issue72-recording.py"><storycode:paste select="#f-6307e15986a3-expression_statement"/><storycode:paste select="#f-6307e15986a3-argparse"/><storycode:paste select="#f-6307e15986a3-hashlib"/><storycode:paste select="#f-6307e15986a3-json"/><storycode:paste select="#f-6307e15986a3-os"/><storycode:paste select="#f-6307e15986a3-Path"/><storycode:paste select="#f-6307e15986a3-re"/><storycode:paste select="#f-6307e15986a3-subprocess"/><storycode:paste select="#f-6307e15986a3-tempfile"/><storycode:paste select="#f-6307e15986a3-expression_statement-2"/><storycode:paste select="#f-6307e15986a3-expression_statement-3"/><storycode:paste select="#f-6307e15986a3-expression_statement-4"/><storycode:paste select="#f-6307e15986a3-expression_statement-5"/><storycode:paste select="#f-6307e15986a3-credential"/><storycode:paste select="#f-6307e15986a3-prepare"/><storycode:paste select="#f-6307e15986a3-main"/><storycode:paste select="#f-6307e15986a3-if_statement"/></storycode:file>

</storycode:doc>
