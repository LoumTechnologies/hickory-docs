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
        folder=Path(temporary);root=folder/"workspace";root.mkdir();path="a.py";source="value = 1\n";(root/path).write_text(source);events=folder/"events.jsonl"
        (root/".hick-lsp.json").write_text(json.dumps({"servers":{"python":{"command":[sys.executable,str(ROOT/"scripts/controlled-action-provider.py"),"python",str(events)]}}}))
        environment={**os.environ,"HICKORY_STATE_DIR":str(folder/"state")};processes=[];driver=None
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
            driver.act("action-choose",title="Controlled python fix")
            result=driver.act("change-state",path=path);assert "python_fixed" in result["text"] and "1 source files" in result["text"],result
            assert driver.act("editor-size",path=path)==size;assert driver.act("editor-is-retained",path=path);assert driver.act("search-source",path=path)==source
            driver.act("change-apply");driver.act("change-close");assert driver.act("search-source",path=path)=="python_fixed = 1\n";assert (root/path).read_text()==source
            driver.act("change-open",command="Undo workspace change");driver.act("change-close");assert driver.act("search-source",path=path)==source
            assert driver.act("editor-is-retained",path=path);assert driver.act("editor-size",path=path)==size
            records=[json.loads(line) for line in events.read_text().splitlines()];assert len(records)==1 and records[0]["resolved"]=="python",records
            print("PASS: menu, actual lazy resolution, complete preview, explicit draft application, coordinated undo, retained editor geometry and unchanged disk")
        finally:
            if driver:driver.close()
            for process in reversed(processes):fixture.stop(process)
if __name__=="__main__":main()
