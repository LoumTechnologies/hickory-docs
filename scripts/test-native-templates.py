#!/usr/bin/env python3
"""Focused actual-engine native source-template fixture and witnesses."""
import argparse
import hashlib
import importlib.util
import json
import os
import re
from pathlib import Path
import signal
import sys
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

def prepare(root, provider_context=False):
    root.mkdir()
    (root / ".git").mkdir()
    (root / ".hickory").mkdir()
    (root / ".hickory-native-template-fixture").write_text("owned-native-templates-v1\n")
    (root / "main.py").write_text(SOURCE)
    (root / "generated.py").write_text("generated = True\n")
    (root / "generator.md").write_text('<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\n<h:file path="generated.py">\ngenerated = True\n</h:file>\n</h:doc>\n')
    if provider_context:
        for name in ("variables.py", "denied.py", "unused.py"):
            (root / name).write_bytes(b"old\r\nsecond\n")
        provider = ROOT / "scripts/snippet-context-controlled-provider.py"
        (root / ".hick-lsp.json").write_text(json.dumps({"servers": {"python": {"command": [sys.executable, str(provider), str(root.parent / "provider-events.jsonl")]}}}))
    entries = [
        {"id": "workspace.wrap", "label": "Wrap selection", "description": "Linked selected text", "language": "python", "syntax": SYNTAX},
        {"id": "workspace.context", "label": "Captured context and transform", "description": "Filename, frozen time/random and linked uppercase transform", "language": "python", "syntax": "${TM_FILENAME}|${CURRENT_YEAR}|${RANDOM_HEX}|${1:${TM_SELECTED_TEXT}}|$1|${1/(.*)/${1:/upcase}/}$0"},
        {"id": "workspace.clipboard", "label": "Demand clipboard", "description": "Clipboard is requested only while preparing this explicit choice", "language": "python", "syntax": "${CLIPBOARD:fallback}$0"},
        {"id": "workspace.unused-clipboard", "label": "Selected text before clipboard default", "description": "A used selected-text variable never reads its unused clipboard default", "language": "python", "syntax": "${TM_SELECTED_TEXT:${CLIPBOARD}}$0"},
        {"id": "workspace.unsupported", "label": "Unsupported transform", "description": "An explicit unavailable transform", "language": "python", "syntax": "${1:old}${1/(?<=o)ld/x/}$0"},
    ]
    (root / ".hickory/templates.json").write_text(json.dumps({"version": 1, "templates": entries}))

def request(origin, path, body=None):
    call = urllib.request.Request(origin + path, data=None if body is None else json.dumps(body).encode(), headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(call, timeout=15) as response:
        return json.load(response)

def confirm(origin):
    context = {"version": 2, "path": "main.py", "source": SOURCE, "selection": {"from": 9, "to": 12}, "variable_context": {"clock": {"unix_millis": 1704067200000, "utc_offset_minutes": 0}, "random_hex": "0123456789abcdef0123456789abcdef", "clipboard": None}}
    catalog = request(origin, "/api/templates/catalog", context)
    if type(catalog.get("version")) is not int or catalog["version"] != 1 or type(catalog.get("template_protocol")) is not int or catalog["template_protocol"] != 2:
        raise RuntimeError("Build the engine with template protocol 2 before using this fixture; no native action was requested.")
    wrap = next((entry for entry in catalog["entries"] if entry["id"] == "workspace.wrap"), None)
    unsupported = next((entry for entry in catalog["entries"] if entry["id"] == "workspace.unsupported"), None)
    if not catalog["available"] or not wrap or not wrap["available"] or not unsupported or unsupported["available"] or not unsupported.get("reason"):
        raise RuntimeError("The owned workspace template catalog did not retain supported and unavailable entries truthfully.")
    plan = request(origin, "/api/templates/plan", {**context, "catalog_fingerprint": catalog["catalog_fingerprint"], "template_id": "workspace.wrap"})
    if plan["source_hash"] != hashlib.sha256(SOURCE.encode()).hexdigest() or plan["replacement"] != context["selection"] or plan["snippet"]["syntax"] != SYNTAX or plan["snippet"]["text"] != "old + old":
        raise RuntimeError("The source-template fixture did not produce the exact selected-text replacement.")
    if catalog.get("variable_context") != context["variable_context"] or catalog.get("source_files_modified") is not False or plan.get("variable_context") != context["variable_context"] or plan.get("source_files_modified") is not False:
        raise RuntimeError("Template protocol2 did not echo the exact captured context without effects.")
    variable_plan = request(origin, "/api/templates/plan", {**context, "catalog_fingerprint": catalog["catalog_fingerprint"], "template_id": "workspace.context"})
    if variable_plan["snippet"]["text"] != "main.py|2024|012345|old|old|OLD" or variable_plan["snippet"]["version"] != 2:
        raise RuntimeError("Captured variables and derived transforms did not expand as one complete proposal.")
    demand = request(origin, "/api/templates/plan", {**context, "catalog_fingerprint": catalog["catalog_fingerprint"], "template_id": "workspace.clipboard"})
    if demand.get("available") is not False or demand.get("context_request") != {"version": 1, "variables": ["CLIPBOARD"]} or demand.get("variable_context") != context["variable_context"] or demand.get("source_files_modified") is not False:
        raise RuntimeError("Clipboard was not demanded through the original immutable template lease.")
    denied = {**context, "variable_context": {**context["variable_context"], "clipboard": {"available": False, "text": None}}}
    fallback = request(origin, "/api/templates/plan", {**denied, "catalog_fingerprint": catalog["catalog_fingerprint"], "template_id": "workspace.clipboard"})
    if fallback["snippet"]["text"] != "fallback" or fallback.get("context_unavailable") != ["CLIPBOARD"]:
        raise RuntimeError("Explicit clipboard denial did not preserve the fallback and unavailable explanation.")
    unused = request(origin, "/api/templates/plan", {**context, "catalog_fingerprint": catalog["catalog_fingerprint"], "template_id": "workspace.unused-clipboard"})
    if unused["snippet"]["text"] != "old" or "context_request" in unused:
        raise RuntimeError("An unused clipboard default requested unnecessary access.")
    def rows(nodes):
        for node in nodes:
            yield node
            yield from rows(node.get("children", []))
    files = {node["path"]: node for node in rows(request(origin, "/api/files")["tree"])}
    if files["main.py"].get("generated_by") or files["main.py"].get("doc_id") or not files["generated.py"].get("generated_by"):
        raise RuntimeError("Template fixture must expose an independent writable primary and a distinct generated, read-only source.")
def run_host(host, origin, workspace, temporary, fixture, provider_context=False):
    environment = {**os.environ, "HICKORY_NATIVE_TEMPLATE_ORIGIN": origin, "HICKORY_NATIVE_TEMPLATE_WORKSPACE": str(workspace), "HICKORY_NATIVE_TEMPLATE_INPUT": "main.py"}
    if host == "apple":
        name = "testLiveProviderContextCompletionAndHistory" if provider_context else "testLiveNativeSourceTemplateCatalogAndHistory"
        test_class = "SnippetContextProviderTests" if provider_context else "SourceTemplateTests"
        report = temporary / "native-template.xml"
        command = ["swift", "test", "--package-path", "apps/apple-simple", "--scratch-path", "target/apple-simple-spm", "--filter", test_class + "." + name + "$", "--xunit-output", str(report)]
    else:
        name = "liveProviderContextCompletionAndHistory" if provider_context else "liveNativeSourceTemplateCatalogAndHistory"
        test_class = "SnippetContextProviderTest" if provider_context else "SourceTemplateTest"
        launcher = ROOT / "target/simple-android-gradle"
        if not launcher.is_file():
            subprocess.run(["rustc", "--edition=2021", "scripts/simple-android-gradle.rs", "-o", str(launcher)], cwd=ROOT, check=True)
        command = [str(launcher), "-Pandroid.builder.sdkDownload=false", ":app:testDebugUnitTest", "--tests", "com.hickorydocs.simple." + test_class + "." + name, "--rerun-tasks"]
        report = ROOT / ("apps/android-simple/app/build/test-results/testDebugUnitTest/TEST-com.hickorydocs.simple." + test_class + ".xml")
    report.unlink(missing_ok=True)
    log = ROOT / "target/native-template-smoke" / (host + ("-provider-last.log" if provider_context else "-last.log"))
    log.parent.mkdir(parents=True, exist_ok=True)
    log.with_suffix(".xml").unlink(missing_ok=True)
    log.with_suffix(".json").unlink(missing_ok=True)
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
    if host == "apple" and not report.is_file():
        # Xcode 27's XCTest runner may omit --xunit-output. Admit only its exact
        # executed case and retain a clearly identified normalized console receipt.
        started = re.findall(r"^Test Case '.+' started\.$", output or "", re.MULTILINE)
        expected_start = re.findall(r"^Test Case '-\[(?:[A-Za-z0-9_]+\.)?" + re.escape(test_class) + " " + re.escape(name) + r"\]' started\.$", output or "", re.MULTILINE)
        finished = re.findall(r"^Test Case '.+' (?:passed|failed|skipped) \([0-9.]+ seconds\)\.$", output or "", re.MULTILINE)
        exact = re.findall(r"^Test Case '-\[(?:[A-Za-z0-9_]+\.)?" + re.escape(test_class) + " " + re.escape(name) + r"\]' passed \(([0-9.]+) seconds\)\.$", output or "", re.MULTILINE)
        if len(started) != 1 or len(expected_start) != 1 or len(finished) != 1 or len(exact) != 1 or "Executed 1 test, with 0 failures" not in output or "Test skipped" in output:
            raise RuntimeError(f"Missing native XCTest XML and no exact executed passing console case; inspect {log}")
        receipt = {"version": 1, "producer": "xctest-console", "class": test_class, "case": name, "tests": 1, "failures": 0, "skipped": 0, "seconds": float(exact[0]), "log_sha256": hashlib.sha256((output or "").encode()).hexdigest()}
        log.with_suffix(".json").write_text(json.dumps(receipt, indent=2) + "\n")
    else:
        retained_report = log.with_suffix(".xml")
        retained_report.write_bytes(report.read_bytes())
        rows = list(ET.parse(report).getroot().iter("testcase"))
        if len(rows) != 1 or rows[0].get("name", "").split("(")[0] != name or any(rows[0].find(key) is not None for key in ("skipped", "failure", "error")):
            raise RuntimeError(f"Expected exactly one executed passing native template witness; inspect {report} and {log}")
    kind = "provider-completion context" if provider_context else "source-template"
    print(f"PASS {host}: one actual {kind} host/controller witness executed without skips. Physical pointer, device IME and accessibility remain unverified.")

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("host", choices=["apple", "android"])
    parser.add_argument("--engine", type=Path, default=ROOT / "target/source-edit-focus/debug/examples/engine_client")
    parser.add_argument("--serve-only", action="store_true", help="Keep the owned fixture alive; this does not execute acceptance.")
    parser.add_argument("--provider-context", action="store_true", help="Execute the exact native provider-completion context/transform/clipboard witness.")
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
        prepare(workspace, args.provider_context)
        process = fixture.start([str(engine), str(workspace), "window", "http://127.0.0.1:1"], env={**os.environ, "HICKORY_STATE_DIR": str(temporary / "state")})
        try:
            origin = fixture.origin(process)
            confirm(origin)
            print(json.dumps({"host": args.host, "origin": origin, "workspace": str(workspace), "input": "main.py", "selection": {"from": 9, "to": 12}, "template_id": "workspace.wrap", "acceptance": "not run"}), flush=True)
            if args.serve_only:
                threading.Event().wait()
            else:
                run_host(args.host, origin, workspace, temporary, fixture, args.provider_context)
        finally:
            fixture.stop(process)

if __name__ == "__main__":
    main()
