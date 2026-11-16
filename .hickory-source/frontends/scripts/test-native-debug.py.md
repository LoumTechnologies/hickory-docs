<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Reproduce actual native debugger acceptance

For maintainers verifying the native controller path against the same guarded engine.
The contract is [simple frontend IDE parity](../contract.md). This fixture does
not certify pointer, touch, device IME or assistive-technology interaction.

Run `just native-debug-smoke apple` or `just native-debug-smoke android` from
this checkout or a nested directory. The Rust preparation adapter builds the
locked engine, reuses a pinned debugpy 1.8.22 environment or prepares one below
`target/native-debug-smoke`, and starts this existing isolated fixture runner.
Python 3.10+ with venv, Rust 1.89+, Cargo and a C compiler are machine
prerequisites. An installed interpreter on PATH or one found by the installed
uv tool with downloads disabled is sufficient; use
`HICKORY_NATIVE_DEBUG_PYTHON=/path/to/python` for an explicit interpreter.

Apple requires macOS and full Xcode with its macOS SDK. `DEVELOPER_DIR` is honored;
otherwise an installed full Xcode is selected for child processes only. Android
requires its configured SDK platform/build-tools and JDK 17+, not a connected
device. The wrapper's Gradle version/checksum are authoritative: an installed or
cached matching distribution is reused, otherwise that exact distribution is
verified and unpacked under `target/native-debug-smoke`. No SDK packages,
licenses, global Xcode selection or global Python environments are changed.
Initial missing dependencies need network access; satisfied dependencies are
not reinstalled. Invalid pre-existing caches/environments are preserved with an
explicit failure instead of being deleted or silently replaced.

The direct `python3 scripts/test-native-debug.py` entry remains the internal
fixture adapter for explicitly prepared engine/venv paths. Its `--serve-only`
mode prints an isolated origin for separate native adapters; it does not run
acceptance.
For Apple, select a full Xcode with `DEVELOPER_DIR` when the shell defaults to
Command Line Tools. The focused run passes only when every named case executed
and passed once. SwiftBuild may ignore the XML option; its individual XCTest
start/pass receipts are checked explicitly, never inferred from exit status.

<a id="simple-native-debug-fixture"></a>
## Build one isolated source snapshot

The same source fixtures support Apple and Android: ordinary startup, a named
active worker, nested values, a later run-to line and literal URI characters.
Use the existing real-engine process adapter. A read-only catalog check rejects
an older engine before native launches; it does not establish feature acceptance.

<ui:copy id="simple-native-debug-fixture">
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

</ui:copy>

<a id="simple-native-debug-host-runners"></a>
## Execute only actual native debugger witnesses

Select the four opt-in native controller/transport witnesses and require their
XML results to contain exactly those passing cases without skips. The platform
runners remain adapters; they do not turn a skipped physical-input or AX test
into acceptance. Failures do not replay mutations or launch another session.

<ui:copy id="simple-native-debug-host-runners">
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

</ui:copy>

The optional `--source-tests --serve-only` mode adds `test_choice.py` and
`test_doc.md` without changing the default four-flow smoke. Its independent `test_authored_choice.py` output cannot claim the physical input; startup verifies the physical file has no generated owner. Explicit `[tool.uv]` metadata selects the fixture-linked Python environment without changing product environment policy. The column-zero file tag
preserves the physical `TestChoice` body: `test_selected` assigns 41, pauses at the marked increment,
and asserts 41; `test_sibling` writes an absolute, fixture-owned sentinel. This
makes accidental widening observable even for a scratch-launched authored test.
The runner checks both actual source scans for class-qualified filters and the
physical written-body hash before its JSON descriptor reports the input options
and sentinel. Serving this
fixture does not execute or certify test debugging; consumers still require the
engine's `test_debug_protocol: 1` and their actual host acceptance checks.

<a id="simple-native-debug-runner-entry"></a>
## Own the fixture lifetime and cleanup

The runner owns a temporary workspace, state directory and engine process.
It never tests this checkout’s source files. Serve-only mode supports separate
agent-driven native adapters and explicitly reports that acceptance has not run.
Normal completion and interruption both stop the owned engine. The native host
runner also uses the existing process-group adapter, so timeout or interruption
stops its owned compiler/test worker tree before the fixture is removed.

<ui:copy id="simple-native-debug-runner-entry">
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
</ui:copy>

<ui:file path="scripts/test-native-debug.py"><ui:paste select="#simple-native-debug-fixture"/><ui:paste select="#simple-native-debug-host-runners"/><ui:paste select="#simple-native-debug-runner-entry"/></ui:file>
## Add a scoped smoke without changing the existing test surface

This is an additive public recipe, not a command-name migration. Every existing
recipe keeps its name and coverage. `simple-test` remains the normal platform
parity/safety test entry: its optional debugger cases may skip without an engine
origin, so its success does not establish these four actual debugpy/controller
flows. `native-debug-smoke` supplies that real-system fixture and insists on all
four executed receipts. It does not run browser acceptance, the general native
suite or APK/device interaction, and does not establish pointer, keyboard/IME
or accessibility acceptance.

| Public interface | Change | Active callers |
| --- | --- | --- |
| `simple-test browser/apple/android` | Preserved, no rename or coverage removal | `.github/workflows/simple-frontends.yml`, `scripts/checks/plan.rs`, developer guide |
| `native-debug-smoke apple/android` | New scoped smoke, with preparation | Just discovery and developer guide added here |
| `scripts/test-native-debug.py` | Reused internal fixture/test runner; explicit prepared inputs remain supported | New Rust adapter; retained execution receipts remain historical |

## Check installed machine tools before preparing project dependencies

Keep the recipe thin: rustc bootstraps this dependency-free Rust adapter. Resolve
all paths from the installed helper's checkout, not the shell's current directory,
and pass process arguments as values. Unsupported hosts and missing SDK/compiler
prerequisites fail before package preparation. Xcode and Android/JDK choices
exist only in child-process environments.

<ui:copy id="native-debug-smoke-process-tools">
//! Self-preparing acceptance adapter; the Python runner owns native fixtures.
use std::{
    env,
    ffi::{OsStr, OsString},
    fs::{self, OpenOptions},
    path::{Path, PathBuf},
    process::{Command, ExitCode, Output},
    time::{SystemTime, UNIX_EPOCH},
};
const DEBUGPY: &str = "1.8.22";
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Host {
    Apple,
    Android,
}
impl Host {
    fn parse(value: &OsStr) -> Result<Self> {
        match value.to_str() {
            Some("apple") => Ok(Self::Apple),
            Some("android") => Ok(Self::Android),
            _ => Err("Choose apple or android: just native-debug-smoke <platform>".into()),
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::Apple => "apple",
            Self::Android => "android",
        }
    }
}
#[derive(Clone)]
struct Machine {
    python: PathBuf,
    variables: Vec<(OsString, OsString)>,
}
impl Default for Machine {
    fn default() -> Self {
        Self {
            python: PathBuf::from("python3"),
            variables: Vec::new(),
        }
    }
}
fn installed_python(root: &Path) -> Result<PathBuf> {
    let valid = |path: &Path| {
        capture(
            path,
            &words(&[
                "-I",
                "-c",
                "import sys,venv; assert sys.version_info >= (3,10)",
            ]),
            root,
            &Machine::default(),
        )
        .is_ok()
    };
    if let Some(path) = env::var_os("HICKORY_NATIVE_DEBUG_PYTHON") {
        let path = PathBuf::from(path);
        if valid(&path) {
            return Ok(path);
        }
        return Err(
            "HICKORY_NATIVE_DEBUG_PYTHON must name an installed Python 3.10+ with venv support"
                .into(),
        );
    }
    let mut candidates = vec![root.join("target/thread-debug-env/bin/python")];
    for name in [
        "python3",
        "python3.14",
        "python3.13",
        "python3.12",
        "python3.11",
        "python3.10",
    ] {
        if let Some(path) = find_tool(name) {
            candidates.push(path)
        }
    }
    for path in candidates {
        if valid(&path) {
            return Ok(path);
        }
    }
    if let Some(uv) = find_tool("uv") {
        if let Ok(out) = capture(
            uv,
            &words(&["python", "find", "--no-python-downloads", ">=3.10"]),
            root,
            &Machine::default(),
        ) {
            let path = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim());
            if valid(&path) {
                return Ok(path);
            }
        }
    }
    Err("Machine prerequisite missing: installed Python 3.10+ with venv support (debugpy1.8.22 requires it). Install Python or set HICKORY_NATIVE_DEBUG_PYTHON; this smoke never installs global interpreters.".into())
}
fn command(
    program: impl AsRef<OsStr>,
    args: &[OsString],
    root: &Path,
    machine: &Machine,
) -> Command {
    let mut c = Command::new(program);
    c.args(args).current_dir(root);
    for (k, v) in &machine.variables {
        c.env(k, v);
    }
    c
}
fn run(
    label: &str,
    program: impl AsRef<OsStr>,
    args: &[OsString],
    root: &Path,
    machine: &Machine,
) -> Result<()> {
    eprintln!("Native debug smoke: {label}");
    let status = command(program, args, root, machine).status()?;
    if !status.success() {
        return Err(format!("{label} failed ({status}); the smoke did not pass.").into());
    }
    Ok(())
}
fn capture(
    program: impl AsRef<OsStr>,
    args: &[OsString],
    root: &Path,
    machine: &Machine,
) -> Result<Output> {
    let out = command(program, args, root, machine).output()?;
    if !out.status.success() {
        return Err(format!(
            "Tool probe failed ({}): {}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        )
        .into());
    }
    Ok(out)
}
fn words(values: &[&str]) -> Vec<OsString> {
    values.iter().map(OsString::from).collect()
}
fn find_tool(name: &str) -> Option<PathBuf> {
    env::var_os("PATH").and_then(|value| {
        env::split_paths(&value)
            .map(|dir| dir.join(name))
            .find(|path| path.is_file())
    })
}
fn require_tool(name: &str) -> Result<PathBuf> {
    find_tool(name).ok_or_else(|| {
        format!("Machine prerequisite missing: {name}. Install it and retry native-debug-smoke.")
            .into()
    })
}
fn repository(executable: &Path) -> Result<PathBuf> {
    executable.ancestors().find(|path|path.join("Cargo.lock").is_file()&&path.join("scripts/test-native-debug.py").is_file()).map(Path::to_path_buf).ok_or_else(||"Run the repository's native-debug-smoke recipe; its helper must remain under this checkout's target directory.".into())
}
fn xcode_installations(directory: &Path) -> Vec<PathBuf> {
    let mut result = fs::read_dir(directory)
        .into_iter()
        .flatten()
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with("Xcode"))
                && path.extension() == Some(OsStr::new("app"))
        })
        .map(|path| path.join("Contents/Developer"))
        .filter(|path| {
            path.join("Platforms/MacOSX.platform/Developer/SDKs")
                .is_dir()
        })
        .collect::<Vec<_>>();
    result.sort();
    result.reverse();
    result
}
fn full_xcode(root: &Path) -> Result<PathBuf> {
    if let Some(value) = env::var_os("DEVELOPER_DIR") {
        let path = PathBuf::from(value);
        if path
            .join("Platforms/MacOSX.platform/Developer/SDKs")
            .is_dir()
        {
            return Ok(path);
        }
        return Err("DEVELOPER_DIR must name a full Xcode Contents/Developer directory with the macOS SDK; no global Xcode selection was changed.".into());
    }
    let selected = capture("xcode-select", &words(&["-p"]), root, &Machine::default())
        .ok()
        .map(|out| PathBuf::from(String::from_utf8_lossy(&out.stdout).trim()));
    let mut candidates = selected.into_iter().collect::<Vec<_>>();
    candidates.push(PathBuf::from("/Applications/Xcode.app/Contents/Developer"));
    if let Some(home) = env::var_os("HOME") {
        candidates.push(PathBuf::from(home).join("Applications/Xcode.app/Contents/Developer"));
    }
    candidates.extend(xcode_installations(Path::new("/Applications")));
    if let Some(home) = env::var_os("HOME") {
        candidates.extend(xcode_installations(
            &PathBuf::from(home).join("Applications"),
        ));
    }
    candidates.into_iter().find(|path|path.join("Platforms/MacOSX.platform/Developer/SDKs").is_dir()).ok_or_else(||"Apple smoke requires full Xcode and its macOS SDK. Install Xcode, then select it for this invocation with DEVELOPER_DIR=/path/to/Xcode.app/Contents/Developer. This command never runs xcode-select --switch.".into())
}
// AGP's major/minor release pin may install as android-37.0. A major-only
// spelling is equivalent only for minor zero; never accept another minor.
fn android_platform_available(sdk:&Path,major:&str,minor:&str)->bool {
    let exact=sdk.join(format!("platforms/android-{major}.{minor}/android.jar"));
    exact.is_file() || minor=="0"&&sdk.join(format!("platforms/android-{major}/android.jar")).is_file()
}
fn android_sdk(root: &Path) -> Result<PathBuf> {
    let mut candidates = ["ANDROID_SDK_ROOT", "ANDROID_HOME"]
        .into_iter()
        .filter_map(env::var_os)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    if let Some(home) = env::var_os("HOME") {
        let home = PathBuf::from(home);
        candidates.extend([home.join("Library/Android/sdk"), home.join("Android/Sdk")]);
    }
    let build = fs::read_to_string(root.join("apps/android-simple/app/build.gradle.kts"))?;
    let version = build
        .split("compileSdk")
        .nth(1)
        .and_then(|value| value.split("release(").nth(1))
        .map(|value| {
            value
                .chars()
                .take_while(char::is_ascii_digit)
                .collect::<String>()
        })
        .filter(|value| !value.is_empty())
        .ok_or("Cannot read the project's compileSdk pin")?;
    let minor=build.split("minorApiLevel").nth(1).and_then(|value|value.split('=').nth(1)).map(str::trim_start).map(|value|value.chars().take_while(char::is_ascii_digit).collect::<String>()).filter(|value|!value.is_empty()).ok_or("Cannot read the project's compileSdk minor pin")?;
    candidates.into_iter().find(|sdk|android_platform_available(sdk,&version,&minor)&&sdk.join("build-tools").is_dir()).ok_or_else(||format!("Android smoke requires installed Android SDK platform {version}.{minor} and build-tools. Install them with Android Studio/SDK Manager, and set ANDROID_SDK_ROOT. SDK packages/licenses are never installed or accepted by this smoke.").into())
}
fn java_home() -> Result<PathBuf> {
    if let Some(value) = env::var_os("JAVA_HOME") {
        let path = PathBuf::from(value);
        if path.join("bin/java").is_file() && path.join("bin/javac").is_file() {
            return Ok(path);
        }
        return Err("JAVA_HOME must name a JDK (including bin/java and bin/javac).".into());
    }
    let mut studios = env::var_os("ANDROID_STUDIO_HOME")
        .map(PathBuf::from)
        .into_iter()
        .collect::<Vec<_>>();
    studios.push(PathBuf::from("/Applications/Android Studio.app"));
    if let Some(home) = env::var_os("HOME") {
        studios.push(PathBuf::from(home).join("Applications/Android Studio.app"));
    }
    for studio in studios {
        for path in [
            studio.join("Contents/jbr/Contents/Home"),
            studio.join("jbr"),
        ] {
            if path.join("bin/javac").is_file() {
                return Ok(path);
            }
        }
    }
    let javac = require_tool("javac")?.canonicalize()?;
    javac
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| "Set JAVA_HOME to an installed JDK 17 or later.".into())
}
fn supported_host(host: Host, os: &str) -> Result<()> {
    if !matches!(os, "macos" | "linux") {
        return Err("Native debugger smoke supports macOS (apple/android) and Linux (android); this host is unsupported.".into());
    }
    if host == Host::Apple && os != "macos" {
        return Err("The apple debugger smoke requires macOS and full Xcode; use android on a configured Linux host.".into());
    }
    Ok(())
}
fn machine(root: &Path, host: Host) -> Result<Machine> {
    supported_host(host, env::consts::OS)?;
    for tool in ["rustc", "cargo", "cc"] {
        let executable = require_tool(tool)?;
        capture(
            executable,
            &words(&["--version"]),
            root,
            &Machine::default(),
        )?;
    }
    let mut result = Machine {
        python: installed_python(root)?,
        ..Machine::default()
    };
    if host == Host::Apple {
        result
            .variables
            .push(("DEVELOPER_DIR".into(), full_xcode(root)?.into_os_string()));
        require_tool("swift")?;
        capture(
            "xcrun",
            &words(&["--sdk", "macosx", "--show-sdk-path"]),
            root,
            &result,
        )?;
    } else {
        let sdk = android_sdk(root)?;
        let java = java_home()?;
        result.variables.extend([
            ("ANDROID_SDK_ROOT".into(), sdk.clone().into_os_string()),
            ("ANDROID_HOME".into(), sdk.into_os_string()),
            ("JAVA_HOME".into(), java.clone().into_os_string()),
        ]);
        let output = capture(java.join("bin/javac"), &words(&["-version"]), root, &result)?;
        let version = format!(
            "{} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let major = version
            .split_whitespace()
            .nth(1)
            .unwrap_or("")
            .split('.')
            .next()
            .and_then(|part| part.parse::<u32>().ok())
            .unwrap_or(0);
        if major < 17 {
            return Err(
                format!("Android smoke requires JDK 17 or later; reported {version}").into(),
            );
        }
    }
    Ok(result)
}
</ui:copy>

## Reuse the exact adapter pin without modifying user Python

Probe the target-local environment before installing anything. A matching
existing `target/thread-debug-env` is reusable; otherwise create the versioned
project-owned environment. Pip runs through that environment's interpreter, in
isolated mode with an explicit package pin and virtualenv requirement. A wrong
pre-existing package version is preserved and refused, rather than uninstalled.

<ui:copy id="native-debug-smoke-python">
fn python_version(python: &Path, root: &Path, machine: &Machine) -> Result<String> {
    let out = capture(
        python,
        &words(&[
            "-I",
            "-c",
            "import importlib.metadata as m\ntry: print(m.version('debugpy'))\nexcept m.PackageNotFoundError: print('')",
        ]),
        root,
        machine,
    )?;
    Ok(String::from_utf8(out.stdout)?.trim().into())
}
fn verify_python(python: &Path, root: &Path, machine: &Machine) -> Result<()> {
    let script =
        format!("import debugpy; assert debugpy.__version__ == {DEBUGPY:?}, debugpy.__version__");
    capture(
        python,
        &["-I".into(), "-c".into(), script.into()],
        root,
        machine,
    )?;
    Ok(())
}
fn prepare_python(root: &Path, machine: &Machine) -> Result<PathBuf> {
    let owned = root.join(format!("target/native-debug-smoke/debugpy-{DEBUGPY}"));
    let legacy = root.join("target/thread-debug-env");
    for candidate in [&owned, &legacy] {
        let python = candidate.join("bin/python");
        if python.is_file()
            && python_version(&python, root, machine).ok().as_deref() == Some(DEBUGPY)
            && verify_python(&python, root, machine).is_ok()
        {
            eprintln!(
                "Native debug smoke: reuse pinned debugpy {DEBUGPY} at {}",
                candidate.display()
            );
            return Ok(candidate.to_path_buf());
        }
    }
    if owned.exists() && !owned.join("pyvenv.cfg").is_file() {
        return Err(format!("Preserved unrecognized preparation directory {}. Move it aside explicitly before retrying.",owned.display()).into());
    }
    if !owned.join("bin/python").is_file() {
        fs::create_dir_all(owned.parent().unwrap())?;
        run(
            "prepare target-local Python environment",
            &machine.python,
            &["-m".into(), "venv".into(), owned.as_os_str().into()],
            root,
            machine,
        )?;
    }
    let python = owned.join("bin/python");
    let existing = python_version(&python, root, machine)?;
    if !existing.is_empty() {
        return Err(format!("Preserved existing debugpy {existing} at {}; required pin is {DEBUGPY}. Repair/move this target-local environment explicitly; no dependencies were removed.",owned.display()).into());
    }
    run(
        "install pinned debugpy into target-local environment",
        &python,
        &[
            "-I".into(),
            "-m".into(),
            "pip".into(),
            "--isolated".into(),
            "install".into(),
            "--require-virtualenv".into(),
            "--disable-pip-version-check".into(),
            "--no-deps".into(),
            format!("debugpy=={DEBUGPY}").into(),
        ],
        root,
        machine,
    )?;
    verify_python(&python, root, machine)?;
    Ok(owned)
}
</ui:copy>

## Honor the wrapper's Gradle version and checksum

The native runner already reuses the existing Gradle adapter. Select a matching
installed/cache distribution or prepare the wrapper-pinned archive locally. A
checksum failure preserves the artifact and prevents extraction. Verify every
archive path before extraction, then install only the new private staging tree;
never remove an existing dependency directory to simulate a clean start.

<ui:copy id="native-debug-smoke-gradle">
#[derive(Debug)]
struct GradlePin {
    version: String,
    url: String,
    sha: String,
}
fn gradle_pin(root: &Path) -> Result<GradlePin> {
    let text = fs::read_to_string(
        root.join("apps/android-simple/gradle/wrapper/gradle-wrapper.properties"),
    )?;
    let get = |key: &str| {
        text.lines()
            .find_map(|line| line.strip_prefix(key))
            .map(str::trim)
            .map(str::to_string)
    };
    let url = get("distributionUrl=")
        .ok_or("Missing pinned Gradle URL")?
        .replace("\\:", ":");
    let sha = get("distributionSha256Sum=").ok_or("Missing pinned Gradle checksum")?;
    if sha.len() != 64 || !sha.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("Invalid Gradle SHA-256 pin".into());
    }
    if !url.starts_with("https://") && !url.starts_with("file://") {
        return Err(
            "Pinned Gradle URL must use https:// (or a local file:// verification fixture)".into(),
        );
    }
    let version = url
        .rsplit("gradle-")
        .next()
        .and_then(|value| value.strip_suffix("-bin.zip"))
        .ok_or("Expected pinned gradle-VERSION-bin.zip URL")?
        .to_string();
    if version.is_empty()
        || !version
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b".-".contains(&byte))
    {
        return Err("Invalid pinned Gradle version spelling".into());
    }
    Ok(GradlePin { version, url, sha })
}
fn gradle_matches(home: &Path, pin: &GradlePin, root: &Path, machine: &Machine) -> Result<bool> {
    let executable = home.join("bin/gradle");
    if !executable.is_file() {
        return Ok(false);
    }
    let out = capture(executable, &words(&["--version"]), root, machine)?;
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .any(|line| line.trim() == format!("Gradle {}", pin.version)))
}
fn checksum(path: &Path, sha: &str, root: &Path, machine: &Machine) -> Result<()> {
    capture(&machine.python,&["-I".into(),"-c".into(),"import hashlib,pathlib,sys; actual=hashlib.sha256(pathlib.Path(sys.argv[1]).read_bytes()).hexdigest(); assert actual==sys.argv[2].lower(), 'Gradle archive checksum mismatch: '+actual".into(),path.as_os_str().into(),sha.into()],root,machine)?;
    Ok(())
}
fn prepare_gradle(root: &Path, machine: &Machine) -> Result<PathBuf> {
    let pin = gradle_pin(root)?;
    let storage = root.join("target/native-debug-smoke");
    let owned = storage.join(format!("gradle-{}", pin.version));
    let mut candidates = vec![owned.clone()];
    if let Some(home) = env::var_os("GRADLE_HOME") {
        candidates.push(home.into())
    }
    if let Some(path) = find_tool("gradle").and_then(|path| path.canonicalize().ok()) {
        if let Some(home) = path.parent().and_then(Path::parent) {
            candidates.push(home.into())
        }
    }
    let cache = env::var_os("GRADLE_USER_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".gradle")));
    if let Some(cache) = cache {
        for kind in ["bin", "all"] {
            let folder = cache
                .join("wrapper/dists")
                .join(format!("gradle-{}-{kind}", pin.version));
            if let Ok(entries) = fs::read_dir(folder) {
                for entry in entries.flatten() {
                    candidates.push(entry.path().join(format!("gradle-{}", pin.version)));
                }
            }
        }
    }
    for home in candidates {
        if gradle_matches(&home, &pin, root, machine)? {
            eprintln!(
                "Native debug smoke: reuse pinned Gradle {} at {}",
                pin.version,
                home.display()
            );
            return Ok(home);
        }
    }
    if owned.exists() {
        return Err(format!(
            "Preserved invalid Gradle preparation at {}; move it aside explicitly before retrying.",
            owned.display()
        )
        .into());
    }
    fs::create_dir_all(&storage)?;
    let archive = storage.join(format!("gradle-{}-bin.zip", pin.version));
    if !archive.exists() {
        require_tool("curl")?;
        let temporary = storage.join(format!(
            "gradle-{}.download-{}",
            pin.version,
            std::process::id()
        ));
        run(
            "download checksum-pinned Gradle distribution",
            "curl",
            &[
                "--fail".into(),
                "--location".into(),
                "--proto".into(),
                "=https,file".into(),
                "--output".into(),
                temporary.as_os_str().into(),
                pin.url.clone().into(),
            ],
            root,
            machine,
        )?;
        checksum(&temporary, &pin.sha, root, machine)?;
        fs::rename(&temporary, &archive)?;
    } else {
        checksum(&archive, &pin.sha, root, machine)?;
    }
    let stage = storage.join(format!(
        "gradle-stage-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir(&stage)?;
    let extract = "import pathlib,sys,zipfile\narchive,destination,prefix=sys.argv[1:]\nwith zipfile.ZipFile(archive) as z:\n for member in z.infolist():\n  parts=pathlib.PurePosixPath(member.filename).parts\n  assert parts and parts[0]==prefix and not member.filename.startswith('/') and '..' not in parts, 'Unsafe Gradle archive path'\n z.extractall(destination)";
    run(
        "extract verified Gradle distribution",
        &machine.python,
        &[
            "-I".into(),
            "-c".into(),
            extract.into(),
            archive.as_os_str().into(),
            stage.as_os_str().into(),
            format!("gradle-{}", pin.version).into(),
        ],
        root,
        machine,
    )?;
    let extracted = stage.join(format!("gradle-{}", pin.version));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(
            extracted.join("bin/gradle"),
            fs::Permissions::from_mode(0o755),
        )?;
    }
    if !gradle_matches(&extracted, &pin, root, machine)? {
        return Err("Verified Gradle archive did not contain its pinned working launcher".into());
    }
    fs::rename(extracted, &owned)?;
    fs::remove_dir(&stage)?;
    Ok(owned)
}
</ui:copy>

## Serialize preparation and own one platform smoke at a time

OS file locks release even if a preparation process dies. The shared preparation
lock protects the venv/Gradle cache; a separate platform lock prevents two same-
platform runs from sharing a report or log. Cargo's locked incremental build
provides freshness without a second hand-maintained source fingerprint. The
existing Python runner owns engine/fixture cleanup, and any preparation or native
runner failure propagates without replaying a debug mutation. Full native output
is retained in `target/native-debug-smoke/<platform>-last.log`.

<ui:copy id="native-debug-smoke-preparation">
fn prepare(root: &Path, host: Host, machine: &mut Machine) -> Result<(PathBuf, PathBuf)> {
    fs::create_dir_all(root.join("target/native-debug-smoke"))?;
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(root.join("target/native-debug-smoke/preparation.lock"))?;
    eprintln!("Native debug smoke: waiting for this checkout's preparation lock");
    lock.lock()?;
    if host == Host::Android {
        let gradle = prepare_gradle(root, machine)?;
        machine
            .variables
            .push(("GRADLE_HOME".into(), gradle.into_os_string()));
    }
    let venv = prepare_python(root, machine)?;
    let target = root.join("target/source-edit-focus");
    run(
        "prepare locked engine build",
        "cargo",
        &[
            "build".into(),
            "--locked".into(),
            "--package".into(),
            "hickory-cli".into(),
            "--example".into(),
            "engine_client".into(),
            "--target-dir".into(),
            target.as_os_str().into(),
        ],
        root,
        machine,
    )?;
    let engine = target.join("debug/examples/engine_client");
    if !engine.is_file() {
        return Err("Engine build succeeded without producing engine_client".into());
    }
    drop(lock);
    Ok((engine, venv))
}
fn smoke(root: &Path, host: Host, machine: &mut Machine) -> Result<()> {
    fs::create_dir_all(root.join("target/native-debug-smoke"))?;
    let lease = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(root.join(format!(
            "target/native-debug-smoke/{}.run.lock",
            host.name()
        )))?;
    eprintln!(
        "Native debug smoke: waiting for this checkout's {} smoke lock",
        host.name()
    );
    lease.lock()?;
    let (engine, venv) = prepare(root, host, machine)?;
    run(
        "execute exactly four real native debugger flows",
        &machine.python,
        &[
            root.join("scripts/test-native-debug.py").into_os_string(),
            host.name().into(),
            "--engine".into(),
            engine.into_os_string(),
            "--venv".into(),
            venv.into_os_string(),
        ],
        root,
        machine,
    )
}
fn entry() -> Result<()> {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    if args.len() != 1 {
        return Err("Usage: just native-debug-smoke <apple|android>".into());
    }
    let host = Host::parse(&args[0])?;
    let root = repository(&env::current_exe()?.canonicalize()?)?;
    let mut machine = machine(&root, host)?;
    smoke(&root, host, &mut machine)
}
fn main() -> ExitCode {
    match entry() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Native debug smoke: {error}");
            ExitCode::FAILURE
        }
    }
}
</ui:copy>

## Verify cold preparation and warm reuse in an owned fixture

The isolated preparation check uses a tiny locked Rust example, a real pinned
debugpy environment, a local checksum-pinned Gradle launcher fixture and a test
runner adapter that records exact arguments. It checks orchestration and quoting,
not native debugger acceptance. The repeated invocation must preserve dependency
and engine timestamps; a deliberate later compiler failure must prevent the
runner and preserve those dependencies. Only fixture-owned directories are
removed. Run the four real native flows separately through the public command.

<ui:copy id="native-debug-smoke-preparation-tests">
#[cfg(test)]
mod tests {
    use super::*;
    static FIXTURE_SEQUENCE:std::sync::atomic::AtomicU64=std::sync::atomic::AtomicU64::new(0);
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let root = env::temp_dir().join(format!(
                "native smoke fixture # {}-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                FIXTURE_SEQUENCE.fetch_add(1,std::sync::atomic::Ordering::Relaxed)
            ));
            fs::create_dir(&root).unwrap();
            Self(root)
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn fixture_pin(root: &Path, sha: &str, url: &str) {
        let folder = root.join("apps/android-simple/gradle/wrapper");
        fs::create_dir_all(&folder).unwrap();
        fs::write(
            folder.join("gradle-wrapper.properties"),
            format!("distributionUrl={url}\ndistributionSha256Sum={sha}\n"),
        )
        .unwrap();
    }
    #[test]
    fn scoped_platform_and_root_resolution_do_not_interpret_shell_text() {
        assert_eq!(Host::parse(OsStr::new("apple")).unwrap(), Host::Apple);
        assert_eq!(Host::parse(OsStr::new("android")).unwrap(), Host::Android);
        assert!(Host::parse(OsStr::new("android; touch unexpected")).is_err());
        assert!(Host::parse(OsStr::new("browser")).is_err());
        assert!(supported_host(Host::Apple, "macos").is_ok());
        assert!(supported_host(Host::Android, "linux").is_ok());
        assert!(supported_host(Host::Apple, "linux").is_err());
        assert!(supported_host(Host::Android, "windows").is_err());
        assert!(supported_host(Host::Android, "freebsd").is_err());
        let fixture = Fixture::new();
        fs::write(fixture.0.join("Cargo.lock"), "").unwrap();
        fs::create_dir(fixture.0.join("scripts")).unwrap();
        fs::write(fixture.0.join("scripts/test-native-debug.py"), "").unwrap();
        let nested = fixture.0.join("target/nested/helper");
        assert_eq!(repository(&nested).unwrap(), fixture.0);
    }
    #[test]
    fn versioned_xcode_discovery_ignores_manager_and_incomplete_apps() {
        let fixture = Fixture::new();
        let expected = fixture.0.join("Xcode-27.0.0.app/Contents/Developer");
        fs::create_dir_all(expected.join("Platforms/MacOSX.platform/Developer/SDKs")).unwrap();
        fs::create_dir(fixture.0.join("Xcodes.app")).unwrap();
        fs::create_dir(fixture.0.join("Xcode-incomplete.app")).unwrap();
        assert_eq!(xcode_installations(&fixture.0), vec![expected]);
    }
    #[test]
    fn sdk_platform_matches_the_declared_major_and_minor() {
        let fixture=Fixture::new();let sdk=&fixture.0;
        fs::create_dir_all(sdk.join("platforms/android-37.0")).unwrap();
        fs::write(sdk.join("platforms/android-37.0/android.jar"),"fixture").unwrap();
        assert!(android_platform_available(sdk,"37","0"));
        assert!(!android_platform_available(sdk,"37","1"));
        assert!(!android_platform_available(sdk,"36","0"));
        fs::remove_file(sdk.join("platforms/android-37.0/android.jar")).unwrap();
        fs::create_dir_all(sdk.join("platforms/android-37")).unwrap();
        fs::write(sdk.join("platforms/android-37/android.jar"),"fixture").unwrap();
        assert!(android_platform_available(sdk,"37","0"));
        assert!(!android_platform_available(sdk,"37","1"));
    }
    #[test]
    fn wrapper_requires_a_pinned_archive_and_safe_version() {
        let fixture = Fixture::new();
        fixture_pin(
            &fixture.0,
            &"a".repeat(64),
            "https\\://services.gradle.org/distributions/gradle-9.3.1-bin.zip",
        );
        let pin = gradle_pin(&fixture.0).unwrap();
        assert_eq!(pin.version, "9.3.1");
        assert!(pin.url.starts_with("https://"));
        fixture_pin(
            &fixture.0,
            "not-a-sha",
            "https://example.invalid/gradle-9.3.1-bin.zip",
        );
        assert!(gradle_pin(&fixture.0).is_err());
        fixture_pin(
            &fixture.0,
            &"a".repeat(64),
            "http://example.invalid/gradle-9.3.1-bin.zip",
        );
        assert!(gradle_pin(&fixture.0).is_err());
        fixture_pin(
            &fixture.0,
            &"a".repeat(64),
            "https://example.invalid/gradle-../../bad-bin.zip",
        );
        assert!(gradle_pin(&fixture.0).is_err());
    }
    #[test]
    fn invalid_cached_archive_is_preserved_and_rejected() {
        let fixture = Fixture::new();
        let archive = fixture.0.join("archive with spaces.zip");
        fs::write(&archive, b"invalid cached bytes").unwrap();
        assert!(checksum(&archive, &"0".repeat(64), &fixture.0, &Machine::default()).is_err());
        assert_eq!(fs::read(&archive).unwrap(), b"invalid cached bytes");
    }
    #[cfg(unix)]
    #[test]
    fn existing_gradle_probe_failure_is_not_replaced_by_a_download() {
        use std::os::unix::fs::PermissionsExt;
        let fixture = Fixture::new();
        fixture_pin(
            &fixture.0,
            &"a".repeat(64),
            "https://example.invalid/gradle-9.3.1-bin.zip",
        );
        let launcher = fixture
            .0
            .join("target/native-debug-smoke/gradle-9.3.1/bin/gradle");
        fs::create_dir_all(launcher.parent().unwrap()).unwrap();
        let text = "#!/bin/sh\necho deliberate-sdk-probe-failure >&2\nexit 17\n";
        fs::write(&launcher, text).unwrap();
        fs::set_permissions(&launcher, fs::Permissions::from_mode(0o755)).unwrap();
        let error = prepare_gradle(&fixture.0, &Machine::default()).unwrap_err();
        assert!(error.to_string().contains("deliberate-sdk-probe-failure"));
        assert_eq!(fs::read_to_string(&launcher).unwrap(), text);
        assert!(
            !fixture
                .0
                .join("target/native-debug-smoke/gradle-9.3.1-bin.zip")
                .exists()
        );
    }
    #[test]
    #[ignore = "prepares a private actual Cargo example, pinned debugpy venv and local checksum-pinned Gradle fixture"]
    fn cold_preparation_then_reuse_and_failure_preserve_existing_dependencies() {
        let fixture = Fixture::new();
        let root = &fixture.0;
        let machine = Machine {
            python: installed_python(root).unwrap(),
            ..Machine::default()
        };
        fs::create_dir(root.join("scripts")).unwrap();
        fs::write(root.join("Cargo.toml"),"[package]\nname='hickory-cli'\nversion='0.0.0'\nedition='2024'\n[[example]]\nname='engine_client'\npath='engine_client.rs'\n[workspace]\n").unwrap();
        fs::write(root.join("engine_client.rs"), "fn main() {}\n").unwrap();
        run(
            "create isolated fixture lockfile",
            "cargo",
            &words(&["generate-lockfile", "--offline"]),
            root,
            &machine,
        )
        .unwrap();
        let lock = fs::read(root.join("Cargo.lock")).unwrap();
        let fake_runner = "import argparse,pathlib\np=argparse.ArgumentParser();p.add_argument('host');p.add_argument('--engine',type=pathlib.Path);p.add_argument('--venv',type=pathlib.Path);a=p.parse_args();assert a.host=='android' and a.engine.is_file() and (a.venv/'bin/python').is_file()\nwith pathlib.Path('smoke-calls').open('a') as output: output.write(a.host+'\\n')\n";
        fs::write(root.join("scripts/test-native-debug.py"), fake_runner).unwrap();
        let archive = root.join("gradle-0.0.0-fixture-bin.zip");
        let out=capture(&machine.python,&["-I".into(),"-c".into(),"import hashlib,pathlib,sys,zipfile\np=pathlib.Path(sys.argv[1])\nwith zipfile.ZipFile(p,'w') as z:z.writestr('gradle-0.0.0-fixture/bin/gradle', '#!/bin/sh\\nprintf \\\'Gradle 0.0.0-fixture\\\\n\\\'\\n')\nprint(p.as_uri());print(hashlib.sha256(p.read_bytes()).hexdigest())".into(),archive.as_os_str().into()],root,&machine).unwrap();
        let text = String::from_utf8(out.stdout).unwrap();
        let rows = text.lines().collect::<Vec<_>>();
        fixture_pin(root, rows[1], rows[0]);
        let mut first = machine.clone();
        smoke(root, Host::Android, &mut first).unwrap();
        let engine = root.join("target/source-edit-focus/debug/examples/engine_client");
        let venv = root.join(format!("target/native-debug-smoke/debugpy-{DEBUGPY}"));
        let gradle = root.join("target/native-debug-smoke/gradle-0.0.0-fixture/bin/gradle");
        assert!(engine.is_file());
        assert_eq!(
            python_version(&venv.join("bin/python"), root, &machine).unwrap(),
            DEBUGPY
        );
        let engine_time = fs::metadata(&engine).unwrap().modified().unwrap();
        let python_time = fs::metadata(venv.join("pyvenv.cfg"))
            .unwrap()
            .modified()
            .unwrap();
        let gradle_time = fs::metadata(&gradle).unwrap().modified().unwrap();
        let mut again = machine.clone();
        smoke(root, Host::Android, &mut again).unwrap();
        assert_eq!(
            fs::metadata(&engine).unwrap().modified().unwrap(),
            engine_time
        );
        assert_eq!(
            fs::metadata(venv.join("pyvenv.cfg"))
                .unwrap()
                .modified()
                .unwrap(),
            python_time
        );
        assert_eq!(
            fs::metadata(&gradle).unwrap().modified().unwrap(),
            gradle_time
        );
        assert_eq!(
            fs::read_to_string(root.join("smoke-calls")).unwrap(),
            "android\nandroid\n"
        );
        assert_eq!(fs::read(root.join("Cargo.lock")).unwrap(), lock);
        fs::write(
            root.join("engine_client.rs"),
            "compile_error!(\"deliberate preparation refusal\"); fn main(){}\n",
        )
        .unwrap();
        let mut failed = machine.clone();
        assert!(smoke(root, Host::Android, &mut failed).is_err());
        assert_eq!(
            fs::read_to_string(root.join("smoke-calls")).unwrap(),
            "android\nandroid\n"
        );
        assert!(venv.join("bin/python").exists());
        assert!(gradle.exists());
        assert_eq!(
            fs::metadata(venv.join("pyvenv.cfg"))
                .unwrap()
                .modified()
                .unwrap(),
            python_time
        );
    }
}
</ui:copy>

## Focused preparation and public-command evidence

On 2026-11-16 the generated Rust helper compiled and all six fast tests passed; the normally opt-in cold/reuse/failure fixture also executed explicitly and passed, with zero skips. That fixture prepared a private locked Cargo example, a real pinned debugpy environment and a checksum-pinned local Gradle archive. Reuse preserved dependency timestamps; a deliberate compiler failure preserved dependencies and prevented driver invocation. Its fake native driver does not establish native debugger acceptance. Parallel test directories include an atomic suffix, so simultaneous fixture creation cannot share an owned directory.

From `apps/android-simple`, the actual public `just native-debug-smoke android` then passed. It reused existing Gradle 9.3.1 and debugpy 1.8.22 without reinstalling them, completed the locked engine build, and executed exactly four real native debugger methods with zero skips (6.682 seconds of JUnit execution). The strict receipt check admitted only startup, thread/scoped-values, literal-source identity and paused resource-lineage flows. The owned fixture and service were cleaned up; complete native output was retained at `target/native-debug-smoke/android-last.log`. The earlier SDK preflight refusal exposed and led to the exact major/minor platform-layout fix; it never launched a native test or installed SDK packages.

The Apple owner then ran the same public `just native-debug-smoke apple` command after its source-test acceptance and reported exit zero with exactly four executed real controller/transport witnesses and zero skips; the locked engine build reused its existing artifacts in 1.11 seconds. It used the validated installed versioned Xcode without changing global selection, cleaned its owned fixture, and retained output at `target/native-debug-smoke/apple-last.log`. Both supported host commands therefore have actual public-path evidence. Neither preparation tests nor controller tests establish physical gutter/popover input, device keyboard/IME or accessibility acceptance.

<ui:file path="scripts/native-debug-smoke.rs"><ui:paste select="#native-debug-smoke-process-tools"/><ui:paste select="#native-debug-smoke-python"/><ui:paste select="#native-debug-smoke-gradle"/><ui:paste select="#native-debug-smoke-preparation"/><ui:paste select="#native-debug-smoke-preparation-tests"/></ui:file>

</ui:doc>
