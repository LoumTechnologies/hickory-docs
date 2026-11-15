"""Existing simple-menu type navigation against a controlled child, real engine."""
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
    with tempfile.TemporaryDirectory(prefix="hickory-type-navigation-") as temporary:
        folder=Path(temporary);root=folder/"workspace";root.mkdir();path="a.py";source="value = 1\n";(root/path).write_text(source);events=folder/"events.jsonl"
        (root/".hick-lsp.json").write_text(json.dumps({"servers":{"python":{"command":[sys.executable,str(ROOT/"scripts/controlled-action-provider.py"),"python",str(events)]}}}))
        environment={**os.environ,"HICKORY_STATE_DIR":str(folder/"state")};processes=[];driver=None
        try:
            with socket.socket() as reservation:reservation.bind(("127.0.0.1",0));port=reservation.getsockname()[1]
            ui=f"http://127.0.0.1:{port}"
            engine=fixture.start([os.environ.get("HICKORY_SIMPLE_TEST_ENGINE",str(ROOT/"target/source-edit-focus/debug/examples/engine_client")),str(root),"window",ui],env=environment);processes.append(engine);backend=fixture.origin(engine)
            vite=fixture.start(["node","node_modules/vite/bin/vite.js","--config","vite.simple.config.ts","--host","127.0.0.1","--port",str(port),"--strictPort"],cwd=ROOT/"apps/web",env={**environment,"HICKORY_API_ORIGIN":backend});processes.append(vite)
            driver=fixture.BrowserDriver(fixture.origin(vite)+"/simple.html");driver.act("open",path=path);driver.act("source-tests-edit",path=path,text="# retained draft\n")
            draft=driver.act("search-source",path=path);driver.act("remember-editor",path=path);size=driver.act("editor-size",path=path)
            deadline=time.monotonic()+20
            while True:
                result=driver.act("semantic-query",path=path,offset=1,query="Find supertypes")
                if "pythonBase" in result["text"]:break
                driver.act("semantic-close");assert time.monotonic()<deadline,result;time.sleep(.1)
            assert result["size"]==size,result;driver.act("semantic-open",path=path,label="pythonBase · a.py:1:1");assert driver.act("caret-offset",path=path)==0
            result=driver.act("semantic-query",path=path,offset=1,query="Find subtypes");assert "pythonDerived" in result["text"],result;assert result["size"]==size;driver.act("semantic-close")
            assert driver.act("search-source",path=path)==draft;assert driver.act("editor-is-retained",path=path);assert driver.act("editor-size",path=path)==size;assert (root/path).read_text()==source
            observed=[json.loads(line) for line in events.read_text().splitlines()];assert {item.get("type") for item in observed}>={"typeHierarchy/supertypes","typeHierarchy/subtypes"},observed
            print("PASS: existing type-navigation menus, actual provider follow-ups, precise name navigation, retained draft/editor/geometry and unchanged disk")
        finally:
            if driver:driver.close()
            for process in reversed(processes):fixture.stop(process)
if __name__=="__main__":main()
