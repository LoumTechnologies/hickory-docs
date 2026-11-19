<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Exercise source tests through the existing menu

For maintainers changing the floating source-test interaction.

Start a dependency-free crate with isolated engine state and a local Vite server. Reuse the browser fixture only for process lifetime and the production command driver.

<ui:copy id="source-tests-browser-setup">
"""Focused real-engine source-test menu and retained-editor acceptance."""
import importlib.util
import os
from pathlib import Path
import socket
import sys
import tempfile
import time
ROOT=Path(__file__).resolve().parents[1]

def main():
    spec=importlib.util.spec_from_file_location("simple_fixture",ROOT/"scripts/test-simple-browser.py")
    fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
    with tempfile.TemporaryDirectory(prefix="hickory-source-test-ui-") as temporary:
        root=Path(temporary)/"workspace";root.mkdir()
        debug="--debug" in sys.argv;captured="--captured" in sys.argv
        if debug:
            native_spec=importlib.util.spec_from_file_location("native_debug_fixture",ROOT/"scripts/test-native-debug.py")
            native=importlib.util.module_from_spec(native_spec);native_spec.loader.exec_module(native)
            native.prepare_source_tests(root)
            venv=Path(os.environ.get("HICKORY_THREAD_TEST_VENV",str(ROOT/"target/thread-debug-env"))).resolve()
            assert (venv/"bin/python").is_file(),"Prepare the explicit debugpy/pytest fixture environment first"
            (root/".venv").symlink_to(venv,target_is_directory=True)
            path="test_doc.md" if captured else "test_choice.py";source=(root/path).read_text()
        else:
            (root/"Cargo.toml").write_text('[package]\nname="test_ui_witness"\nversion="0.1.0"\nedition="2024"\n[lib]\npath="lib.rs"\n')
            source="#[test] fn ui_source_witness() { assert_eq!(2 + 2, 4); }\n#[test] fn ui_extra_witness() { assert!(true); }\n"
            path="lib.rs";(root/path).write_text(source)
        environment={**os.environ,"HICKORY_STATE_DIR":str(Path(temporary)/"state")};processes=[];driver=None
        try:
            with socket.socket() as reservation:reservation.bind(("127.0.0.1",0));port=reservation.getsockname()[1]
            ui=f"http://127.0.0.1:{port}"
            engine=fixture.start([os.environ.get("HICKORY_SIMPLE_TEST_ENGINE",str(ROOT/"target/source-edit-focus/debug/examples/engine_client")),str(root),"window",ui],env=environment);processes.append(engine)
            backend=fixture.origin(engine)
            vite=fixture.start(["node","node_modules/vite/bin/vite.js","--config","vite.simple.config.ts","--host","127.0.0.1","--port",str(port),"--strictPort"],cwd=ROOT/"apps/web",env={**environment,"HICKORY_API_ORIGIN":backend});processes.append(vite)
            driver=fixture.BrowserDriver(fixture.origin(vite)+"/simple.html")
</ui:copy>

Use adjacent one-line tests to exercise a test action beside an existing debug action, then verify that opening the gutter menu does not move another populated row. The gutter opens a fresh inspection. Also use the actual Edit menu, inspect source candidates and run a name filter. Read only the selected disposable run and require the real runner's success output. The editor must retain its DOM identity and dimensions beneath the floating surface.

<ui:copy id="source-tests-browser-run">
            if debug:
                try:debug_flow(driver,root,path,source,fixture)
                except Exception:
                    print("DEBUG WIRE",driver.act("debug-wire"),flush=True)
                    print("DEBUG UI",driver.act("debug-diagnostics"),flush=True)
                    raise
                return
            driver.act("open",path=path);driver.act("remember-editor",path=path);size=driver.act("editor-size",path=path)
            gutter=driver.act("source-tests-gutter",name="ui_source_witness")
            assert gutter["before"]==gutter["after"],gutter
            assert "ui_source_witness" in gutter["text"],gutter
            assert driver.act("editor-size",path=path)==size
            driver.act("source-tests-close")
            driver.act("source-tests-open");result=driver.act("source-tests-read")
            assert "ui_source_witness" in result["text"] and result["runEnabled"],result
            assert driver.act("editor-size",path=path)==size
            assert driver.act("editor-is-retained",path=path)
            driver.act("source-tests-source")
            fixture.eventually(lambda:driver.act("caret-offset",path=path),len("#[test] fn "))
            driver.act("source-tests-open")
            driver.act("source-tests-run")
            deadline=time.monotonic()+40
            while time.monotonic()<deadline:
                result=driver.act("source-tests-refresh-run")
                if "ui_source_witness ... ok" in result["text"] and "test result: ok." in result["text"]:break
                time.sleep(.2)
            else:raise AssertionError(result)
</ui:copy>

Close and reopen the surface after a draft edit. Old candidates must be disabled; refreshing must read the draft without saving it; execution stays disabled for that draft while the earlier run and editor state remain available.

<ui:copy id="source-tests-browser-retention">
            assert "Files were not frozen during execution" in driver.act("source-tests-revisions")
            driver.act("source-tests-close");driver.act("append",path=path,text="// retained unsaved draft\n")
            draft=driver.act("search-source",path=path)
            driver.act("source-tests-open");result=driver.act("source-tests-state")
            assert "Stale observation" in result["text"] and not result["runEnabled"],result
            result=driver.act("source-tests-read");assert not result["runEnabled"] and "Save this current owned source" in result["text"],result
            assert "Retained test runs (1)" in result["text"],result
            # Already-dirty buffers publish capability revisions without rerendering the shell.
            driver.act("source-tests-edit",path=path,text="// another unsaved edit\n")
            result=driver.act("source-tests-state");assert not result["runEnabled"] and "Stale observation" in result["text"],result
            draft=driver.act("search-source",path=path)
            assert driver.act("editor-size",path=path)==size
            driver.act("source-tests-close")
            assert driver.act("editor-is-retained",path=path)
            assert driver.act("search-source",path=path)==draft
            assert (root/path).read_text()==source,"Inspection or execution saved a draft"
            print("PASS source tests UI: menu, real test output, retained sessions/editor/draft, stale refusal, unchanged code geometry and disk")
        finally:
            if driver:driver.close()
            for process in reversed(processes):fixture.stop(process)

</ui:copy>

The test Debug witness reuses the maintained Python family fixture. A real user
breakpoint is placed in the selected body before opening its source-adjacent
candidate action. The first run proves locals, watch evaluation and retained
session navigation; the second runs the same choice to completion. Only that
completed run can establish that the sibling sentinel was never written. These
are process observations, not collected per-case verdicts.

<ui:copy id="source-tests-browser-debug-flow">
def debug_flow(driver,root,path,source,fixture):
    def wait_state(fragment):
        deadline=time.monotonic()+30
        while time.monotonic()<deadline:
            state=driver.act("debug-state")
            if any(fragment.lower() in value.lower() for value in state):return state
            time.sleep(.1)
        raise AssertionError({"state":state,"diagnostic":driver.act("debug-diagnostics")})
    marker=next(index for index,line in enumerate(source.splitlines(),1) if "source-test-breakpoint" in line)
    driver.act("open",path=path);driver.act("remember-editor",path=path);size=driver.act("editor-size",path=path)
    fixture.eventually(lambda:driver.act("editor-write-state",path=path),{"readOnly":False,"editable":True})
    driver.act("debug-breakpoint",line=marker)
    gutter=driver.act("source-tests-gutter",name="test_selected")
    assert gutter["before"]==gutter["after"],gutter
    record=driver.act("source-tests-debug",filter="test_selected")
    assert "Process state and exit code are not collected test verdicts" in record,record
    driver.act("source-tests-debug-link");wait_state("paused")
    assert "41" in driver.act("debug-watch",expression="selected_value")
    assert "selected_value" in driver.act("debug-locals")
    assert driver.act("editor-is-retained",path=path)
    assert driver.act("editor-size",path=path)==size
    position=driver.act("editor-position",path=path)
    driver.act("source-tests-open");record=driver.act("source-tests-debug-records")
    assert "test_selected" in record and "name_filter" in record,record
    driver.act("source-tests-debug-link");assert driver.act("editor-position",path=path)==position
    driver.act("debug-stop");fixture.eventually(lambda:any(event.get("event")=="ended" for event in driver.act("debug-wire")),True)
    # Prepare a second explicit choice; never use ordinary Debug again.
    driver.act("source-tests-open");driver.act("source-tests-read")
    driver.act("source-tests-debug",filter="test_selected");driver.act("source-tests-debug-link");wait_state("paused")
    driver.act("debug-continue");wait_state("finished")
    driver.act("source-tests-open");records=driver.act("source-tests-debug-records")
    assert "Retained test debug sessions (2)" in records and "process exit 0" in records,records
    assert not (root/"sibling-ran.txt").exists(),"Filtered Debug widened to the sibling test"
    assert driver.act("editor-is-retained",path=path)
    assert driver.act("editor-size",path=path)==size
    assert driver.act("construction-source",path=path)==source
    assert (root/path).read_text()==source
    assert not driver.act("debug-diagnostics")["errors"]
    print("PASS source-test Debug UI: source gutter, selected family, body breakpoint, locals/watch, retained link, Stop, selected completion without sibling, unchanged editor/geometry/disk; process-only outcomes")

if __name__=="__main__":main()
</ui:copy>

<ui:file path="scripts/test-source-tests-browser.py"><ui:paste select="#source-tests-browser-setup"/><ui:paste select="#source-tests-browser-run"/><ui:paste select="#source-tests-browser-retention"/><ui:paste select="#source-tests-browser-debug-flow"/></ui:file>
## Focused verification receipt

On the fresh engine with dedicated `start_test` admission, both `--debug` and
`--debug --captured` passed in Chromium against isolated workspaces. Each flow
opened the source-adjacent gutter candidate, selected its exact pytest family,
stopped at the existing user body breakpoint, read locals and a watch value of
41, reopened the retained debugger link, and stopped the first session. A fresh
second choice completed with process exit 0 while its sibling sentinel remained
absent. The authored flow used `test_doc.md` and mapped runtime frames through
its construction. Both retained the editor, exact selection/scroll across the
popup roundtrip, dimensions, source text and disk bytes, with no page errors.
These observations establish process behavior, not collected test verdicts.

The original no-flag Run witness also passed after enforcing saved-source
execution: candidate reads remain available for a dirty draft, Run stays
disabled with an explicit Save notice, and the earlier run remains readable.
TypeScript checking and 110 focused checks passed across source-test model/view,
debugger hook/options, host registry and API client. Physical keyboard/pointer,
assistive technology, other browser engines and unsupported language providers
were not exercised by these checks. Python pytest availability does not close
the remaining provider breadth in the canonical parity contract.

</ui:doc>
