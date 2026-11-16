"""Isolated actual-engine native debugger witnesses; no physical-input claim."""
import argparse
import importlib.util
import json
import os
import re
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

def confirmed_catalog(origin):
    request = urllib.request.Request(origin + "/api/launch-profiles", data=json.dumps({"uri": "hick:///debug.py", "program": "debug.py"}).encode(), headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(request, timeout=15) as response:
        catalog = json.load(response)
    if not isinstance(catalog.get("debug_protocol"), int) or catalog["debug_protocol"] < 3:
        raise RuntimeError("Build the guarded debugger engine (protocol >=3) before running this witness; no native launch was requested.")
    if not any(profile.get("can_debug") and profile.get("program") == "debug.py" for profile in catalog["profiles"]):
        raise RuntimeError("The isolated Python launch profile was not discovered; inspect fixture source and adapter setup.")

def check_results(path, expected):
    cases = ET.parse(path).getroot().iter("testcase")
    rows = list(cases)
    observed = {row.get("name", "").split("(")[0] for row in rows}
    if len(rows) != len(expected) or observed != set(expected) or any(row.find("skipped") is not None or row.find("failure") is not None or row.find("error") is not None for row in rows):
        details=[{"name":row.get("name"),"skipped":row.find("skipped") is not None,"failure":(row.findtext("failure") or row.findtext("error") or "")[-1500:]} for row in rows]
        raise AssertionError(f"Expected exactly these executed passing native witnesses: {expected}. Observed {details}")

def run_host(host, origin, temporary):
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
        command = [str(launcher), ":app:testDebugUnitTest", "--rerun-tasks"]
        for name in names:
            command += ["--tests", "com.hickorydocs.simple.NativeDebugTest." + name]
        report = ROOT / "apps/android-simple/app/build/test-results/testDebugUnitTest/TEST-com.hickorydocs.simple.NativeDebugTest.xml"
    result = subprocess.run(command, cwd=ROOT, env=environment, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=600)
    if result.returncode:
        diagnostics = [line for line in result.stdout.splitlines() if "error:" in line or "failed" in line.lower()]
        raise RuntimeError("Native debugger witness failed:\n" + "\n".join(diagnostics)[-8000:] + "\n" + result.stdout[-2000:])
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
    parser.add_argument("--serve-only", action="store_true", help="Keep the isolated fixture alive for separate native adapters; does not execute acceptance.")
    args = parser.parse_args()
    engine, venv = args.engine.resolve(), args.venv.resolve()
    if not engine.is_file() or not (venv / "bin/python").is_file():
        parser.error("Build the engine_client example and prepare an explicit virtualenv with debugpy; pass --engine and --venv to select them.")
    fixture = fixture_api()
    with tempfile.TemporaryDirectory(prefix=f"hickory-native-{args.host}-debug-") as folder:
        temporary = Path(folder)
        workspace = temporary / "workspace"
        prepare(workspace, venv)
        process = fixture.start([str(engine), str(workspace), "window", "http://127.0.0.1:1"], env={**os.environ, "HICKORY_STATE_DIR": str(temporary / "state")})
        try:
            origin = fixture.origin(process)
            confirmed_catalog(origin)
            print(json.dumps({"host": args.host, "origin": origin, "workspace": str(workspace), "acceptance": "not run"}), flush=True)
            if args.serve_only:
                threading.Event().wait()
            else:
                run_host(args.host, origin, temporary)
        finally:
            fixture.stop(process)

if __name__ == "__main__":
    main()
