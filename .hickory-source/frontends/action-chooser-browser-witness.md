<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Resolve a provider action through the existing preview

For maintainers changing workspace services independently of the editor renderer.

The real engine starts a controlled lazy provider, so this flow tests protocol and UI integration rather than Python analysis. The existing Edit menu prepares its choice, selection resolves it, and the complete before/after preview leaves editor source untouched. Explicit application changes only the retained draft; coordinated undo restores it while editor identity, geometry and disk stay unchanged.

This controlled two-file action starts with its second target unopened. Two actual child resolutions prove the discovering result was replaced after source synchronization. The fixture checks draft-only apply and coordinated undo while disk and editor geometry remain unchanged; it does not establish a language server's semantic completeness.

<ui:copy id="simple-action-chooser-browser-witness">
"""Real-engine browser flow for a controlled lazy provider action."""
import importlib.util
import json
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
    with tempfile.TemporaryDirectory(prefix="hickory-action-chooser-") as temporary:
        folder=Path(temporary);root=folder/"workspace";root.mkdir();path="a.py";source="value = 1\n";(root/path).write_text(source);(root/"other.py").write_text(source);events=folder/"events.jsonl"
        (root/".hick-lsp.json").write_text(json.dumps({"servers":{"python":{"command":[sys.executable,str(ROOT/"scripts/controlled-action-provider.py"),"python",str(events)]}}}))
        environment={**os.environ,"HICKORY_STATE_DIR":str(folder/"state"),"HICKORY_CONTROLLED_ACTION_OTHER":"1"};processes=[];driver=None
        try:
            with socket.socket() as reservation:reservation.bind(("127.0.0.1",0));port=reservation.getsockname()[1]
            ui=f"http://127.0.0.1:{port}"
            engine=fixture.start([os.environ.get("HICKORY_SIMPLE_TEST_ENGINE",str(ROOT/"target/source-edit-focus/debug/examples/engine_client")),str(root),"window",ui],env=environment);processes.append(engine);backend=fixture.origin(engine)
            vite=fixture.start(["node","node_modules/vite/bin/vite.js","--config","vite.simple.config.ts","--host","127.0.0.1","--port",str(port),"--strictPort"],cwd=ROOT/"apps/web",env={**environment,"HICKORY_API_ORIGIN":backend});processes.append(vite)
            driver=fixture.BrowserDriver(fixture.origin(vite)+"/simple.html");driver.act("open",path=path);driver.act("remember-editor",path=path);size=driver.act("editor-size",path=path)
            driver.act("change-open",command="Refactor and quick fixes");deadline=time.monotonic()+20
            while True:
                result=driver.act("change-prepare",command="Prepare provider actions")
                if "Controlled python fix" in result:break
                assert time.monotonic()<deadline,result;time.sleep(.1)
            assert all("other.py" not in tab for tab in driver.act("tabs"))
            driver.act("action-choose",title="Controlled python fix")
            result=driver.act("change-state",path=path);assert "python_fixed" in result["text"] and "2 source files" in result["text"],result
            assert driver.act("editor-size",path=path)==size;assert driver.act("editor-is-retained",path=path);assert driver.act("search-source",path=path)==source
            assert all("other.py" not in tab for tab in driver.act("tabs"))
            driver.act("change-apply");driver.act("change-close");assert driver.act("search-source",path=path)=="python_fixed = 1\n";assert (root/path).read_text()==source
            driver.act("change-open",command="Undo workspace change");driver.act("change-close");assert driver.act("search-source",path=path)==source
            assert driver.act("editor-is-retained",path=path);assert driver.act("editor-size",path=path)==size
            records=[json.loads(line) for line in events.read_text().splitlines()];assert len(records)==2 and all(record["resolved"]=="python" for record in records),records
            assert (root/"other.py").read_text()==source
            driver.act("open",path="other.py");assert driver.act("search-source",path="other.py")==source
            print("PASS: menu, actual lazy resolution, unopened-target acquisition and fresh resolution, complete preview, explicit draft application, coordinated undo, retained editor geometry and unchanged disk")
        finally:
            if driver:driver.close()
            for process in reversed(processes):fixture.stop(process)
if __name__=="__main__":main()
</ui:copy>

<ui:file path="scripts/test-action-chooser-browser.py"><ui:paste select="#simple-action-chooser-browser-witness"/></ui:file>
</ui:doc>
