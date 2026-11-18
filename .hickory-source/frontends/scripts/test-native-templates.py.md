<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Exercise native templates in an owned workspace

For maintainers verifying the actual Android and Apple template adapters against the shared service. This fixture is separate from the four-flow debugger smoke: templates need no debugpy, pytest or project virtual environment. Reuse the existing fixture process ownership helpers and an explicitly built engine. Serving the fixture is preparation, not acceptance.

Normal mode verifies protocol 2 reusable templates; `--provider-context` selects a distinct exact native completion case and installs a controlled child only inside the marked temporary workspace. Both retain separate logs and native reports. When this Xcode runner omits XCTest XML, require its exact started/passed case, successful process, one-test summary and no skips or extra cases, then retain an explicitly identified console JSON receipt with its log hash. Never synthesize a framework report or accept an old report after a failed run. The final four host/mode replays passed with zero skips on 2026-11-17; physical device input remains separate.

A separate focused native protocol fixture can reuse `run_host(..., acceptance=(test_class, method, label))`. The explicit identifiers choose exactly one method, and the bounded label gives its retained log/report a separate address. This does not widen the native test filter or weaken the executed-case and producer-report checks; omitted overrides keep the existing template, embedded and provider modes unchanged.

The separate --program3 mode declares a valid lookbehind/Unicode-property template and retains its exact native receipt under a distinct template3 label. The deliberately unavailable declaration now uses genuinely malformed syntax, preserving that negative case as valid lookbehind becomes supported. Existing fixture modes and the producer-report admission rules remain intact.

<ui:copy id="native-template-fixture-prepare">
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

def prepare(root, provider_context=False, embedded=False, program3=False):
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
    if embedded:
        ns = 'http://www.hickorydocs.com/1.0'
        def doc(body): return '<h:doc xmlns:h="'+ns+'">'+body+'</h:doc>'
        fence = '<h:paste select="#fence_tick"/>' * 3
        (root / "project.md").write_text(doc('<h:project sources="part.md"/><h:copy id="fence_tick">`</h:copy><h:file path="saved.py"><h:paste select="#body"/></h:file><h:file path="second.py"><h:paste select="#body"/></h:file><h:file path="guide.md">'+fence+'python\n<h:paste select="#body"/>\n'+fence+'\n</h:file>'))
        (root / "part.md").write_text(doc('<h:copy id="body">saved</h:copy>'))
        (root / "notes.md").write_text("Narrative\n\n```python\nold\n```\n")
    entries = [
        {"id": "workspace.wrap", "label": "Wrap selection", "description": "Linked selected text", "language": "python", "syntax": SYNTAX},
        {"id": "workspace.context", "label": "Captured context and transform", "description": "Filename, frozen time/random and linked uppercase transform", "language": "python", "syntax": "${TM_FILENAME}|${CURRENT_YEAR}|${RANDOM_HEX}|${1:${TM_SELECTED_TEXT}}|$1|${1/(.*)/${1:/upcase}/}$0"},
        {"id": "workspace.clipboard", "label": "Demand clipboard", "description": "Clipboard is requested only while preparing this explicit choice", "language": "python", "syntax": "${CLIPBOARD:fallback}$0"},
        {"id": "workspace.unused-clipboard", "label": "Selected text before clipboard default", "description": "A used selected-text variable never reads its unused clipboard default", "language": "python", "syntax": "${TM_SELECTED_TEXT:${CLIPBOARD}}$0"},
        {"id": "workspace.unsupported", "label": "Unsupported transform", "description": "An explicit unavailable transform", "language": "python", "syntax": "${1:old}${1/(?<=/x/}$0"},
    ]
    if embedded: entries.append({"id":"workspace.embedded","label":"Embedded context","description":"Compiler-bound language and linked transforms","language":"python","syntax":"${LINE_COMMENT}${TM_FILENAME}|${1:${TM_SELECTED_TEXT}}|$1|${1/(.*)/${1:/upcase}/}$0"})
    if embedded: entries.extend([
        {"id":"workspace.batch-context","label":"Batch context","description":"One complete graph/clock snapshot with distinct cursor variables","language":"python","syntax":(r"${LINE_COMMENT}${CURSOR_INDEX}:${CURSOR_NUMBER}:${CURRENT_YEAR}:${RANDOM_HEX}:${1:${TM_SELECTED_TEXT}}|$1|${1/(?<=\p{L})\p{L}/X/gu}$0" if program3 else "${LINE_COMMENT}${CURSOR_INDEX}:${CURSOR_NUMBER}:${CURRENT_YEAR}:${RANDOM_HEX}:${1:${TM_SELECTED_TEXT}}|$1|${1/(.*)/${1:/upcase}/}$0")},
        {"id":"workspace.batch-nested","label":"Batch nested","description":"Continue every original outer mirror group","language":"python","syntax":"${1:kid}-$1$0"},
    ])
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
</ui:copy>

The runtime owns only its temporary workspace, engine and selected native process group. Pass the workspace path to native tests so they can verify the engine root and the fixture marker before temporarily revoking the data-only catalog. Always preserve native failures and inspect exactly one executed test receipt; an old report or skipped optional case cannot establish acceptance. The server helper consumes its own output for readiness, while native output is retained separately.

<ui:copy id="native-template-fixture-runtime">
def run_host(host, origin, workspace, temporary, fixture, provider_context=False, embedded=False, acceptance=None):
    environment = {**os.environ, "HICKORY_NATIVE_TEMPLATE_ORIGIN": origin, "HICKORY_NATIVE_TEMPLATE_WORKSPACE": str(workspace), "HICKORY_NATIVE_TEMPLATE_INPUT": "main.py"}
    if acceptance is not None:
        if not isinstance(acceptance, tuple) or len(acceptance) != 3:
            raise ValueError("Choose exactly one explicit native acceptance class, method and receipt label.")
        test_class, name, label = acceptance
        if not all(isinstance(value, str) and re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", value) for value in (test_class, name)) or not isinstance(label, str) or not re.fullmatch(r"[a-z][a-z0-9-]{0,63}", label):
            raise ValueError("Native acceptance identifiers or receipt label are invalid.")
    if host == "apple":
        name = name if acceptance is not None else ("testLiveEmbeddedTemplateCatalogAndHistory" if embedded else "testLiveProviderContextCompletionAndHistory" if provider_context else "testLiveNativeSourceTemplateCatalogAndHistory")
        test_class = test_class if acceptance is not None else ("EmbeddedTemplateTests" if embedded else "SnippetContextProviderTests" if provider_context else "SourceTemplateTests")
        report = temporary / "native-template.xml"
        command = ["swift", "test", "--package-path", "apps/apple-simple", "--scratch-path", "target/apple-simple-spm", "--filter", test_class + "." + name + "$", "--xunit-output", str(report)]
    else:
        name = name if acceptance is not None else ("liveEmbeddedTemplateCatalogAndHistory" if embedded else "liveProviderContextCompletionAndHistory" if provider_context else "liveNativeSourceTemplateCatalogAndHistory")
        test_class = test_class if acceptance is not None else ("EmbeddedTemplateTest" if embedded else "SnippetContextProviderTest" if provider_context else "SourceTemplateTest")
        launcher = ROOT / "target/simple-android-gradle"
        if not launcher.is_file():
            subprocess.run(["rustc", "--edition=2021", "scripts/simple-android-gradle.rs", "-o", str(launcher)], cwd=ROOT, check=True)
        command = [str(launcher), "-Pandroid.builder.sdkDownload=false", ":app:testDebugUnitTest", "--tests", "com.hickorydocs.simple." + test_class + "." + name, "--rerun-tasks"]
        report = ROOT / ("apps/android-simple/app/build/test-results/testDebugUnitTest/TEST-com.hickorydocs.simple." + test_class + ".xml")
    report.unlink(missing_ok=True)
    suffix = "-" + label + "-last.log" if acceptance is not None else "-embedded-last.log" if embedded else "-provider-last.log" if provider_context else "-last.log"
    log = ROOT / "target/native-template-smoke" / (host + suffix)
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
    kind = label if acceptance is not None else "provider-completion context" if provider_context else "source-template"
    print(f"PASS {host}: one actual {kind} host/controller witness executed without skips. Physical pointer, device IME and accessibility remain unverified.")

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("host", choices=["apple", "android"])
    parser.add_argument("--engine", type=Path, default=ROOT / "target/source-edit-focus/debug/examples/engine_client")
    parser.add_argument("--serve-only", action="store_true", help="Keep the owned fixture alive; this does not execute acceptance.")
    parser.add_argument("--provider-context", action="store_true", help="Execute the exact native provider-completion context/transform/clipboard witness.")
    parser.add_argument("--embedded", action="store_true", help="Execute exact native code-fence/reused-fragment template acceptance.")
    parser.add_argument("--batch", action="store_true", help="Execute the exact complete multi-selection template group witness.")
    parser.add_argument("--program3", action="store_true", help="Execute the exact multi-selection lookbehind/Unicode-property template witness.")
    args = parser.parse_args()
    if args.program3: args.batch = True
    if args.batch and args.provider_context: parser.error("Choose one exact template or provider acceptance mode.")
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
        prepare(workspace, args.provider_context, args.embedded or args.batch, args.program3)
        if args.program3: os.environ["HICKORY_NATIVE_TEMPLATE_PROGRAM3"] = "1"
        process = fixture.start([str(engine), str(workspace), "window", "http://127.0.0.1:1"], env={**os.environ, "HICKORY_STATE_DIR": str(temporary / "state")})
        try:
            origin = fixture.origin(process)
            confirm(origin)
            print(json.dumps({"host": args.host, "origin": origin, "workspace": str(workspace), "input": "main.py", "selection": {"from": 9, "to": 12}, "template_id": "workspace.wrap", "acceptance": "not run"}), flush=True)
            if args.serve_only:
                threading.Event().wait()
            else:
                run_host(args.host, origin, workspace, temporary, fixture, args.provider_context, args.embedded or args.batch, acceptance=(("BatchTemplateTests", "testLiveCompleteTemplateGroupAndHistory", "template3" if args.program3 else "batch") if args.host=="apple" else ("BatchTemplateTest", "liveCompleteTemplateGroupAndHistory", "template3" if args.program3 else "batch")) if args.batch else None)
        finally:
            fixture.stop(process)

if __name__ == "__main__":
    main()
</ui:copy>

## Executed fixture evidence

On 2026-11-16 the maintained runner parsed, prepared an isolated workspace with the exact source/selection/catalog and distinct generated target, and required no Python project environment. Its canonical Android invocation passed exactly one real `SourceTemplateTest.liveNativeSourceTemplateCatalogAndHistory` receipt with zero skips, failures or errors (0.294 seconds of native test execution). That path included actual catalog/plan startup checks and the command/preview/Apply/history/refusal acceptance described in the [Android owning reading](../source-templates-android.md). A failed early fixture-root assertion was corrected to use the canonical workspace fingerprint; a display label never authorizes catalog mutation. Native failure output remained available, and the final successful run cleaned the owned service and workspace. Apple execution is tracked separately by its owner. Serving alone, geometry, physical input, device IME and accessibility are not certified by this fixture.

<ui:file path="scripts/test-native-templates.py"><ui:paste select="#native-template-fixture-prepare"/><ui:paste select="#native-template-fixture-runtime"/></ui:file>

The negotiated `--program3` mode and ordinary `--batch` compatibility replay both passed on 2026-11-18 through each native host with exact one-case, zero-skip receipts. Their distinct `template3-last` and `template3-compatibility` artifacts retain the producer evidence under `target/native-template-smoke/`; the [batch witness reading](../batch-template-host-acceptance.md) records the exact scope and times. Serving alone remains preparation, and physical input/accessibility/device IME are not certified by these controller/mounted tests.

</ui:doc>
