"""Isolated actual-engine native debugger witnesses; no physical-input claim."""
import argparse
import hashlib
import importlib.util
import json
import os
import re
import signal
from pathlib import Path
import subprocess
import tempfile
import threading
import urllib.request
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
SOURCE = """value = 1
value += 1
print(value)
import threading
ready = threading.Event()
release = threading.Event()
def worker():
    worker_value = 41
    worker_nested = {'items': [{'value': 41}], 'nested': {'answer': 41}}
    ready.set()
    while not release.is_set():
        worker_value += 0
worker_thread = threading.Thread(target=worker, name='hickory-worker')
worker_thread.start()
ready.wait()
main_value = 7
release.set()  # thread-stop # hickory-thread-breakpoint
worker_thread.join()
"""
LITERAL_SOURCE = "value = 1\nvalue += 1\nprint(value)\n"

def fixture_api():
    spec = importlib.util.spec_from_file_location("simple_fixture", ROOT / "scripts/test-simple-browser.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

def prepare(root, venv):
    root.mkdir()
    (root / ".git").mkdir()
    (root / ".venv").symlink_to(venv, target_is_directory=True)
    (root / "project.md").write_text('<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:project sources=""/></h:doc>')
    (root / "debug.py").write_text(SOURCE)
    for name in ("%2F λ?#.py", "literal %2F λ?#.py"):
        (root / name).write_text(LITERAL_SOURCE)

def prepare_source_tests(root):
    # The project environment resolver recognizes this fixture's linked venv
    # through explicit project metadata, rather than guessing from its presence.
    (root / "pyproject.toml").write_text('[tool.uv]\n')
    sentinel = (root / "sibling-ran.txt").resolve()
    source = ("from pathlib import Path\n\n"
              "class TestChoice:\n"
              "    def test_selected(self):\n"
              "        selected_value = 41\n"
              "        selected_value += 0  # source-test-breakpoint\n"
              "        assert selected_value == 41\n\n"
              "    def test_sibling(self):\n"
              f"        Path({str(sentinel)!r}).write_text('unexpected sibling execution')\n")
    (root / "test_choice.py").write_text(source)
    # A column-zero file tag preserves Python class indentation. Inline tags
    # inherit their source column and are not a byte-equivalent construction.
    (root / "test_doc.md").write_text('<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\n<h:file path="test_authored_choice.py">\n' + source + '</h:file>\n</h:doc>\n')
    return {"physical": "test_choice.py", "authored": "test_doc.md", "sentinel": str(sentinel)}

def confirmed_catalog(origin, source_tests=False):
    request = urllib.request.Request(origin + "/api/launch-profiles", data=json.dumps({"uri": "hick:///debug.py", "program": "debug.py"}).encode(), headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(request, timeout=15) as response:
        catalog = json.load(response)
    if not isinstance(catalog.get("debug_protocol"), int) or catalog["debug_protocol"] < 3:
        raise RuntimeError("Build the guarded debugger engine (protocol >=3) before running this witness; no native launch was requested.")
    if source_tests and (type(catalog.get("test_debug_protocol")) is not int or catalog["test_debug_protocol"] != 1):
        raise RuntimeError("Source-test fixture requires test_debug_protocol 1; no test-debug launch was requested.")
    if not any(profile.get("can_debug") and profile.get("program") == "debug.py" for profile in catalog["profiles"]):
        raise RuntimeError("The isolated Python launch profile was not discovered; inspect fixture source and adapter setup.")

def confirmed_source_tests(origin, workspace, descriptor):
    # The authored construction must not turn the independent physical input
    # into its generated output, which would make its editor read-only.
    with urllib.request.urlopen(origin + "/api/files", timeout=15) as response:
        inventory = json.load(response)
    def rows(nodes):
        for node in nodes:
            yield node
            yield from rows(node.get("children", []))
    physical = next((node for node in rows(inventory["tree"]) if node["path"] == descriptor["physical"]), None)
    if physical is None or physical.get("generated_by") or physical.get("doc_id"):
        raise RuntimeError("Physical source-test fixture must remain a plain authored input, independent of the document output.")
    expected_hash = hashlib.sha256((workspace / descriptor["physical"]).read_bytes()).hexdigest()
    expected_filters = {"test_selected": "TestChoice::test_selected", "test_sibling": "TestChoice::test_sibling"}
    for name in (descriptor["physical"], descriptor["authored"]):
        request = urllib.request.Request(origin + "/api/tests/source-candidates", data=json.dumps({"path": name, "source": (workspace / name).read_text()}).encode(), headers={"Content-Type": "application/json"})
        with urllib.request.urlopen(request, timeout=15) as response:
            facts = json.load(response)
        candidates = facts["candidates"]
        if len(candidates) != 2 or {item["name"]: item["name_filter"] for item in candidates} != expected_filters or any(item["target"]["hash"] != expected_hash for item in candidates):
            raise RuntimeError(f"Source-test fixture {name} did not retain the physical body and class-qualified filters: {candidates}")

def check_results(path, expected):
    cases = ET.parse(path).getroot().iter("testcase")
    rows = list(cases)
    observed = {row.get("name", "").split("(")[0] for row in rows}
    if len(rows) != len(expected) or observed != set(expected) or any(row.find("skipped") is not None or row.find("failure") is not None or row.find("error") is not None for row in rows):
        details=[{"name":row.get("name"),"skipped":row.find("skipped") is not None,"failure":(row.findtext("failure") or row.findtext("error") or "")[-1500:]} for row in rows]
        raise AssertionError(f"Expected exactly these executed passing native witnesses: {expected}. Observed {details}")

def run_host(host, origin, temporary, fixture=None):
    environment = {**os.environ, "HICKORY_NATIVE_DEBUG_ORIGIN": origin}
    if host == "apple":
        names = ["testLiveNativeDebugChannel", "testLiveNativeThreadSelectionAndScopedEvaluation", "testLiveNativeLiteralPhysicalSourceIdentity", "testLivePausedResourceLineageAndMenuUndoRetainNativeEditor"]
        report = temporary / "native-debug.xml"
        command = ["swift", "test", "--package-path", "apps/apple-simple", "--scratch-path", "target/apple-simple-spm", "--filter", "NativeDebugTests.(" + "|".join(names) + ")$", "--xunit-output", str(report)]
    else:
        names = ["liveNativeDebugChannel", "liveNativeThreadChannel", "liveLiteralPathBreakpointChannel", "livePausedResourceLineageChannel"]
        launcher = ROOT / "target/simple-android-gradle"
        if not launcher.is_file():
            subprocess.run(["rustc", "--edition=2021", "scripts/simple-android-gradle.rs", "-o", str(launcher)], cwd=ROOT, check=True)
        command = [str(launcher), "-Pandroid.builder.sdkDownload=false", ":app:testDebugUnitTest", "--rerun-tasks"]
        for name in names:
            command += ["--tests", "com.hickorydocs.simple.NativeDebugTest." + name]
        report = ROOT / "apps/android-simple/app/build/test-results/testDebugUnitTest/TEST-com.hickorydocs.simple.NativeDebugTest.xml"
    # A previous report cannot stand in for this invocation's executed cases.
    report.unlink(missing_ok=True)
    log = ROOT / "target/native-debug-smoke" / f"{host}-last.log"
    log.parent.mkdir(parents=True, exist_ok=True)
    fixture = fixture or fixture_api()
    runner = fixture.start(command, cwd=ROOT, env=environment)
    try:
        output, _ = runner.communicate(timeout=600)
    except BaseException as error:
        try:
            fixture.stop(runner)
        except ProcessLookupError:
            pass  # The owned process group already exited.
        output, _ = runner.communicate(timeout=15)
        log.write_text(output or "")
        if isinstance(error, subprocess.TimeoutExpired):
            raise RuntimeError(f"Native debugger witness timed out; retained output: {log}") from error
        raise
    result = subprocess.CompletedProcess(command, runner.returncode, output)
    log.write_text(result.stdout)
    if result.returncode:
        diagnostics = [line for line in result.stdout.splitlines() if "error:" in line or "failed" in line.lower()]
        raise RuntimeError(f"Native debugger witness failed; retained output: {log}\n" + "\n".join(diagnostics)[-8000:] + "\n" + result.stdout[-2000:])
    if report.is_file():
        check_results(report, names)
    elif host == "apple":
        # SwiftBuild currently prints XCTest receipts but ignores xunit-output.
        # Require every selected case to start and pass exactly once; an exit
        # status or a selected-suite summary alone is not execution evidence.
        cases = re.findall(r"Test Case '-\[HickorySimpleTests\.NativeDebugTests (\w+)\]' (started|passed|failed|skipped)", result.stdout)
        started = [name for name, state in cases if state == "started"]
        passed = [name for name, state in cases if state == "passed"]
        if sorted(started) != sorted(names) or sorted(passed) != sorted(names) or len(cases) != 2 * len(names):
            raise AssertionError("Native runner lacks exact executed passing XCTest receipts:\n" + result.stdout[-12000:])
    else:
        raise RuntimeError("Native runner did not produce its required executed-test report:\n" + result.stdout[-12000:])
    print(f"PASS {host}: {len(names)} actual debugger controller/transport witnesses executed without skips. Physical input, device IME and accessibility remain unverified.")

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("host", choices=["apple", "android"])
    parser.add_argument("--engine", type=Path, default=ROOT / "target/source-edit-focus/debug/examples/engine_client")
    parser.add_argument("--venv", type=Path, default=ROOT / "target/thread-debug-env")
    parser.add_argument("--source-tests", action="store_true", help="Add physical/authored TestChoice sources and an absolute sibling sentinel; currently requires --serve-only.")
    parser.add_argument("--serve-only", action="store_true", help="Keep the isolated fixture alive for separate native adapters; does not execute acceptance.")
    args = parser.parse_args()
    if args.source_tests and not args.serve_only:
        parser.error("--source-tests currently requires --serve-only; it does not replace the four-flow smoke or establish test-debug acceptance.")
    def interrupted(signum, _frame):
        raise SystemExit(128 + signum)
    signal.signal(signal.SIGTERM, interrupted)
    engine, venv = args.engine.resolve(), args.venv.resolve()
    if not engine.is_file() or not (venv / "bin/python").is_file():
        parser.error("Use just native-debug-smoke apple/android for automatic preparation, or pass explicit prepared --engine and --venv paths.")
    fixture = fixture_api()
    with tempfile.TemporaryDirectory(prefix=f"hickory-native-{args.host}-debug-") as folder:
        temporary = Path(folder)
        workspace = temporary / "workspace"
        prepare(workspace, venv)
        source_tests = prepare_source_tests(workspace) if args.source_tests else None
        process = fixture.start([str(engine), str(workspace), "window", "http://127.0.0.1:1"], env={**os.environ, "HICKORY_STATE_DIR": str(temporary / "state")})
        try:
            origin = fixture.origin(process)
            confirmed_catalog(origin, args.source_tests)
            if source_tests is not None:
                confirmed_source_tests(origin, workspace, source_tests)
            descriptor = {"host": args.host, "origin": origin, "workspace": str(workspace), "acceptance": "not run"}
            if source_tests is not None:
                descriptor["source_tests"] = source_tests
            print(json.dumps(descriptor), flush=True)
            if args.serve_only:
                threading.Event().wait()
            else:
                run_host(args.host, origin, temporary, fixture)
        finally:
            fixture.stop(process)

if __name__ == "__main__":
    main()
