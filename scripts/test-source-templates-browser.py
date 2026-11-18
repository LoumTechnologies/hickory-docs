"""Focused live-engine reusable source-template browser acceptance."""
import importlib.util
import json
import os
from pathlib import Path
import socket
import tempfile
import urllib.error
import urllib.request
ROOT=Path(__file__).resolve().parents[1]
def main():
    spec=importlib.util.spec_from_file_location("simple_fixture",ROOT/"scripts/test-simple-browser.py")
    fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
    with tempfile.TemporaryDirectory(prefix="hickory-source-template-ui-") as temporary:
        root=Path(temporary)/"workspace";root.mkdir();(root/".hickory").mkdir()
        source="old";path="a.py";(root/path).write_text(source)
        (root/"generated.py").write_text("generated = True\n")
        (root/"generator.md").write_text('<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\n<h:file path="generated.py">\ngenerated = True\n</h:file>\n</h:doc>\n')
        manifest={"version":1,"templates":[{"id":"workspace.linked","label":"Workspace linked","description":"Mirrors, Unicode and selected text","language":"python","syntax":"${1|first,second|} + $1\n${2:😀 value} ${TM_FILENAME} ${TM_SELECTED_TEXT}$0"},{"id":"workspace.unsupported","label":"Unsupported transform","description":"Explicit unsupported capability","language":"python","syntax":"${1/(?=x)/y/}"}]}
        (root/".hickory/templates.json").write_text(json.dumps(manifest,ensure_ascii=False))
        environment={**os.environ,"HICKORY_STATE_DIR":str(Path(temporary)/"state")};processes=[];driver=None
        try:
            with socket.socket() as reservation:reservation.bind(("127.0.0.1",0));port=reservation.getsockname()[1]
            ui=f"http://127.0.0.1:{port}"
            engine=fixture.start([os.environ.get("HICKORY_SIMPLE_TEST_ENGINE",str(ROOT/"target/source-edit-focus/debug/examples/engine_client")),str(root),"window",ui],env=environment);processes.append(engine)
            backend=fixture.origin(engine)
            vite=fixture.start(["node","node_modules/vite/bin/vite.js","--config","vite.simple.config.ts","--host","127.0.0.1","--port",str(port),"--strictPort"],cwd=ROOT/"apps/web",env={**environment,"HICKORY_API_ORIGIN":backend});processes.append(vite)
            driver=fixture.BrowserDriver(fixture.origin(vite)+"/simple.html")
            driver.act("open",path=path);driver.act("remember-editor",path=path);size=driver.act("editor-size",path=path)
            before=driver.act("editor-position",path=path,anchor=0,head=3)
            driver.act("templates-open");preview=driver.act("templates-preview",label="Workspace linked")
            assert "first + first\n😀 value a.py old" in preview,preview
            assert driver.act("construction-source",path=path)==source
            assert driver.act("editor-size",path=path)==size and driver.act("editor-is-retained",path=path)
            driver.act("templates-hold-apply")
            altered=json.loads(json.dumps(manifest));altered["templates"][0]["syntax"]="revoked$0"
            (root/".hickory/templates.json").write_text(json.dumps(altered,ensure_ascii=False))
            refused=driver.act("templates-release-apply");assert refused["status"]==422 and "changed" in refused["body"].lower(),refused
            assert driver.act("construction-source",path=path)==source
            (root/".hickory/templates.json").write_text(json.dumps(manifest,ensure_ascii=False))
            driver.act("templates-read");driver.act("templates-preview",label="Workspace linked")
            state=driver.act("templates-apply",path=path);base="first + first\n😀 value a.py old"
            assert state=={"source":base,"anchor":0,"head":5},state
            driver.act("type-source-selection",path=path,text="longer");expanded="longer + longer\n😀 value a.py old"
            fixture.eventually(lambda:driver.act("construction-source",path=path),expanded)
            driver.act("template-next");selected=driver.act("construction-source",path=path,selection=True)
            at=len("longer + longer\n".encode("utf-16-le"))//2
            assert selected["anchor"]==at and selected["head"]==at+len("😀 value".encode("utf-16-le"))//2,selected
            driver.act("construction-history");fixture.eventually(lambda:driver.act("construction-source",path=path),base)
            driver.act("construction-history");restored=driver.act("construction-source",path=path,selection=True)
            assert restored=={"source":source,"anchor":before["anchor"],"head":before["head"]},restored
            driver.act("construction-history",redo=True);driver.act("construction-history",redo=True)
            fixture.eventually(lambda:driver.act("construction-source",path=path),expanded)
            driver.act("template-choices");driver.act("completion-pick",title="second")
            fixture.eventually(lambda:driver.act("construction-source",path=path),"second + second\n😀 value a.py old")
            driver.act("template-finish")
            driver.act("templates-open",via="search");driver.act("templates-preview",label="Workspace linked")
            driver.act("source-tests-edit",path=path,text="\n# intervening draft\n")
            assert not driver.act("templates-apply-enabled"),"Obsolete preview remained enabled"
            changed=driver.act("construction-source",path=path);driver.act("templates-close")
            assert driver.act("construction-source",path=path)==changed
            assert driver.act("editor-size",path=path)==size and driver.act("editor-is-retained",path=path)
            assert (root/path).read_text()==source,"Templates unexpectedly saved the draft"
            driver.act("open",path="generated.py");fixture.eventually(lambda:driver.act("editor-write-state",path="generated.py"),{"readOnly":True,"editable":False})
            refused=driver.act("templates-command-state");assert refused["disabled"] and "editable" in refused["reason"],refused
            request=urllib.request.Request(backend+"/api/templates/catalog",data=json.dumps({"version":1,"path":"generated.py","source":"generated = True\n","selection":{"from":0,"to":0}}).encode(),headers={"Content-Type":"application/json"})
            try:urllib.request.urlopen(request,timeout=15);raise AssertionError("Generated template source was admitted")
            except urllib.error.HTTPError as error:assert error.code==422 and "generated" in error.read().decode().lower(),error
            assert not driver.act("templates-apply-enabled")
            assert (root/"generated.py").read_text()=="generated = True\n"
            assert not driver.act("debug-diagnostics")["errors"]
            print("PASS source templates UI: workspace catalog, explicit preview/Apply, selected-text/Unicode fields, mirrors/choices/traversal, exact undo/redo, revoked-manifest/stale/readonly refusal, retained editor/geometry/draft and unchanged disk")
        finally:
            if driver:driver.close()
            for process in reversed(processes):fixture.stop(process)
if __name__=="__main__":main()
