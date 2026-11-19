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
