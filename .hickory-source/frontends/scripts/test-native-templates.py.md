<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Exercise native templates in an owned workspace

For maintainers verifying the actual Android and Apple template adapters against the shared service. This fixture is separate from the four-flow debugger smoke: templates need no debugpy, pytest or project virtual environment. Reuse the existing fixture process ownership helpers and an explicitly built engine. Serving the fixture is preparation, not acceptance.

<ui:copy id="native-template-fixture-prepare">
#!/usr/bin/env python3
"""Focused actual-engine native source-template fixture and witnesses."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import signal
import subprocess
import tempfile
import threading
import urllib.request
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
SOURCE = "value = 'old'\n"
SYNTAX = "${1:${TM_SELECTED_TEXT}} + $1$0"

def fixture_api():
    spec = importlib.util.spec_from_file_location("simple_fixture", ROOT / "scripts/test-simple-browser.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

def prepare(root):
    root.mkdir()
    (root / ".git").mkdir()
    (root / ".hickory").mkdir()
    (root / ".hickory-native-template-fixture").write_text("owned-native-templates-v1\n")
    (root / "main.py").write_text(SOURCE)
    (root / "generated.py").write_text("generated = True\n")
    (root / "generator.md").write_text('<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\n<h:file path="generated.py">\ngenerated = True\n</h:file>\n</h:doc>\n')
    entries = [
        {"id": "workspace.wrap", "label": "Wrap selection", "description": "Linked selected text", "language": "python", "syntax": SYNTAX},
        {"id": "workspace.unsupported", "label": "Unsupported transform", "description": "An explicit unavailable transform", "language": "python", "syntax": "${1/x/y/}"},
    ]
    (root / ".hickory/templates.json").write_text(json.dumps({"version": 1, "templates": entries}))

def request(origin, path, body=None):
    call = urllib.request.Request(origin + path, data=None if body is None else json.dumps(body).encode(), headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(call, timeout=15) as response:
        return json.load(response)

def confirm(origin):
    context = {"version": 1, "path": "main.py", "source": SOURCE, "selection": {"from": 9, "to": 12}}
    catalog = request(origin, "/api/templates/catalog", context)
    if type(catalog.get("version")) is not int or catalog["version"] != 1 or type(catalog.get("template_protocol")) is not int or catalog["template_protocol"] != 1:
        raise RuntimeError("Build the engine with template protocol 1 before using this fixture; no native action was requested.")
    wrap = next((entry for entry in catalog["entries"] if entry["id"] == "workspace.wrap"), None)
    unsupported = next((entry for entry in catalog["entries"] if entry["id"] == "workspace.unsupported"), None)
    if not catalog["available"] or not wrap or not wrap["available"] or not unsupported or unsupported["available"] or not unsupported.get("reason"):
        raise RuntimeError("The owned workspace template catalog did not retain supported and unavailable entries truthfully.")
    plan = request(origin, "/api/templates/plan", {**context, "catalog_fingerprint": catalog["catalog_fingerprint"], "template_id": "workspace.wrap"})
    if plan["source_hash"] != hashlib.sha256(SOURCE.encode()).hexdigest() or plan["replacement"] != context["selection"] or plan["snippet"]["syntax"] != SYNTAX or plan["snippet"]["text"] != "old + old":
        raise RuntimeError("The source-template fixture did not produce the exact selected-text replacement.")
    def rows(nodes):
        for node in nodes:
            yield node
            yield from rows(node.get("children", []))
    files = {node["path"]: node for node in rows(request(origin, "/api/files")["tree"])}
    if files["main.py"].get("generated_by") or files["main.py"].get("doc_id") or not files["generated.py"].get("generated_by"):
        raise RuntimeError("Template fixture must expose an independent writable primary and a distinct generated, read-only source.")
</ui:copy>

The runtime owns only its temporary workspace, engine and selected native process group. Pass the workspace path to native tests so they can verify the engine root and the fixture marker before temporarily revoking the data-only catalog. Always preserve native failures and inspect exactly one executed test receipt; an old report or skipped optional case cannot establish acceptance. The server helper consumes its own output for readiness, while native output is retained separately.

<ui:copy id="native-template-fixture-runtime">
def run_host(host, origin, workspace, temporary, fixture):
    environment = {**os.environ, "HICKORY_NATIVE_TEMPLATE_ORIGIN": origin, "HICKORY_NATIVE_TEMPLATE_WORKSPACE": str(workspace), "HICKORY_NATIVE_TEMPLATE_INPUT": "main.py"}
    if host == "apple":
        name = "testLiveNativeSourceTemplateCatalogAndHistory"
        report = temporary / "native-template.xml"
        command = ["swift", "test", "--package-path", "apps/apple-simple", "--scratch-path", "target/apple-simple-spm", "--filter", "SourceTemplateTests." + name + "$", "--xunit-output", str(report)]
    else:
        name = "liveNativeSourceTemplateCatalogAndHistory"
        launcher = ROOT / "target/simple-android-gradle"
        if not launcher.is_file():
            subprocess.run(["rustc", "--edition=2021", "scripts/simple-android-gradle.rs", "-o", str(launcher)], cwd=ROOT, check=True)
        command = [str(launcher), "-Pandroid.builder.sdkDownload=false", ":app:testDebugUnitTest", "--tests", "com.hickorydocs.simple.SourceTemplateTest." + name, "--rerun-tasks"]
        report = ROOT / "apps/android-simple/app/build/test-results/testDebugUnitTest/TEST-com.hickorydocs.simple.SourceTemplateTest.xml"
    report.unlink(missing_ok=True)
    log = ROOT / "target/native-template-smoke" / (host + "-last.log")
    log.parent.mkdir(parents=True, exist_ok=True)
    process = fixture.start(command, cwd=ROOT, env=environment)
    try:
        output, _ = process.communicate(timeout=600)
    except BaseException:
        fixture.stop(process)
        output, _ = process.communicate(timeout=15)
        log.write_text(output or "")
        raise
    log.write_text(output or "")
    if process.returncode:
        raise RuntimeError(f"Native template witness failed ({process.returncode}); retained output: {log}")
    rows = list(ET.parse(report).getroot().iter("testcase"))
    if len(rows) != 1 or rows[0].get("name", "").split("(")[0] != name or any(rows[0].find(key) is not None for key in ("skipped", "failure", "error")):
        raise RuntimeError(f"Expected exactly one executed passing native template witness; inspect {report} and {log}")
    print(f"PASS {host}: one actual source-template host/controller witness executed without skips. Physical pointer, device IME and accessibility remain unverified.")

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("host", choices=["apple", "android"])
    parser.add_argument("--engine", type=Path, default=ROOT / "target/source-edit-focus/debug/examples/engine_client")
    parser.add_argument("--serve-only", action="store_true", help="Keep the owned fixture alive; this does not execute acceptance.")
    args = parser.parse_args()
    def interrupted(number, _frame):
        raise SystemExit(128 + number)
    signal.signal(signal.SIGTERM, interrupted)
    engine = args.engine.resolve()
    if not engine.is_file():
        parser.error("Build the existing hickory-cli engine_client example first, or pass --engine; this fixture does not install tools or change machine configuration.")
    fixture = fixture_api()
    with tempfile.TemporaryDirectory(prefix=f"hickory-native-{args.host}-templates-") as folder:
        temporary = Path(folder)
        workspace = temporary / "workspace"
        prepare(workspace)
        process = fixture.start([str(engine), str(workspace), "window", "http://127.0.0.1:1"], env={**os.environ, "HICKORY_STATE_DIR": str(temporary / "state")})
        try:
            origin = fixture.origin(process)
            confirm(origin)
            print(json.dumps({"host": args.host, "origin": origin, "workspace": str(workspace), "input": "main.py", "selection": {"from": 9, "to": 12}, "template_id": "workspace.wrap", "acceptance": "not run"}), flush=True)
            if args.serve_only:
                threading.Event().wait()
            else:
                run_host(args.host, origin, workspace, temporary, fixture)
        finally:
            fixture.stop(process)

if __name__ == "__main__":
    main()
</ui:copy>

## Executed fixture evidence

On 2026-11-16 the maintained runner parsed, prepared an isolated workspace with the exact source/selection/catalog and distinct generated target, and required no Python project environment. Its canonical Android invocation passed exactly one real `SourceTemplateTest.liveNativeSourceTemplateCatalogAndHistory` receipt with zero skips, failures or errors (0.294 seconds of native test execution). That path included actual catalog/plan startup checks and the command/preview/Apply/history/refusal acceptance described in the [Android owning reading](../source-templates-android.md). A failed early fixture-root assertion was corrected to use the canonical workspace fingerprint; a display label never authorizes catalog mutation. Native failure output remained available, and the final successful run cleaned the owned service and workspace. Apple execution is tracked separately by its owner. Serving alone, geometry, physical input, device IME and accessibility are not certified by this fixture.

<ui:file path="scripts/test-native-templates.py"><ui:paste select="#native-template-fixture-prepare"/><ui:paste select="#native-template-fixture-runtime"/></ui:file>
</ui:doc>
