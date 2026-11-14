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
import tempfile
import time
ROOT=Path(__file__).resolve().parents[1]

def main():
    spec=importlib.util.spec_from_file_location("simple_fixture",ROOT/"scripts/test-simple-browser.py")
    fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
    with tempfile.TemporaryDirectory(prefix="hickory-source-test-ui-") as temporary:
        root=Path(temporary)/"workspace";root.mkdir()
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
            driver.act("open",path=path);driver.act("remember-editor",path=path);size=driver.act("editor-size",path=path)
            gutter=driver.act("source-tests-gutter",path=path,line=1)
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

Close and reopen the surface after a draft edit. Old candidates must be disabled; refreshing must read the draft without saving it; execution against stale disk must be refused while the earlier run and editor state remain available.

<ui:copy id="source-tests-browser-retention">
            assert "Files were not frozen during execution" in driver.act("source-tests-revisions")
            driver.act("source-tests-close");driver.act("append",path=path,text="// retained unsaved draft\n")
            draft=driver.act("search-source",path=path)
            driver.act("source-tests-open");result=driver.act("source-tests-state")
            assert "Stale observation" in result["text"] and not result["runEnabled"],result
            result=driver.act("source-tests-read");assert result["runEnabled"],result
            result=driver.act("source-tests-run");assert "differs from the requested source" in result["text"],result
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

if __name__=="__main__":main()
</ui:copy>

<ui:file path="scripts/test-source-tests-browser.py"><ui:paste select="#source-tests-browser-setup"/><ui:paste select="#source-tests-browser-run"/><ui:paste select="#source-tests-browser-retention"/></ui:file>
</ui:doc>
