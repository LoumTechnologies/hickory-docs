<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Resolve a provider action through the existing preview

For maintainers changing workspace services independently of the editor renderer.

The real engine starts a controlled lazy provider, so this flow tests protocol and UI integration rather than Python analysis. The existing Edit menu prepares its choice, selection resolves it, and the complete before/after preview leaves editor source untouched. Explicit application changes only the retained draft; coordinated undo restores it while editor identity, geometry and disk stay unchanged.

The `--resources` variant uses ordered source edits, creation, movement, further edits and deletion through the same lazy provider. Its complete review precedes explicit saved publication, and the existing workspace Undo/Redo commands prepare guarded inverse reviews. The witness checks all saved paths and a separately retained original dirty draft, editor identity and geometry.

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
    resources="--resources" in sys.argv
    with tempfile.TemporaryDirectory(prefix="hickory-action-chooser-") as temporary:
        folder=Path(temporary);root=folder/"workspace";root.mkdir();path="a.py";source="value = 1\n";(root/path).write_text(source);(root/"other.py").write_text(source);events=folder/"events.jsonl"
        if resources:(root/"delete.py").write_text("delete me\n")
        (root/".hick-lsp.json").write_text(json.dumps({"servers":{"python":{"command":[sys.executable,str(ROOT/"scripts/controlled-action-provider.py"),"python",str(events)]}}}))
        environment={**os.environ,"HICKORY_STATE_DIR":str(folder/"state"),"HICKORY_CONTROLLED_ACTION_OTHER":"1",**({"HICKORY_CONTROLLED_ACTION_RESOURCES":"1"} if resources else {})};processes=[];driver=None
        try:
            with socket.socket() as reservation:reservation.bind(("127.0.0.1",0));port=reservation.getsockname()[1]
            ui=f"http://127.0.0.1:{port}"
            engine=fixture.start([os.environ.get("HICKORY_SIMPLE_TEST_ENGINE",str(ROOT/"target/source-edit-focus/debug/examples/engine_client")),str(root),"window",ui],env=environment);processes.append(engine);backend=fixture.origin(engine)
            vite=fixture.start(["node","node_modules/vite/bin/vite.js","--config","vite.simple.config.ts","--host","127.0.0.1","--port",str(port),"--strictPort"],cwd=ROOT/"apps/web",env={**environment,"HICKORY_API_ORIGIN":backend});processes.append(vite)
            driver=fixture.BrowserDriver(fixture.origin(vite)+"/simple.html");driver.act("open",path=path);driver.act("remember-editor",path=path);size=driver.act("editor-size",path=path)
            original_draft=source+"# retained draft\n" if resources else source
            if resources:driver.act("construction-set-source",path=path,source=original_draft)
            driver.act("change-open",command="Refactor and quick fixes");deadline=time.monotonic()+20
            while True:
                result=driver.act("change-prepare",command="Prepare provider actions")
                if "Controlled python fix" in result:break
                assert time.monotonic()<deadline,result;time.sleep(.1)
            assert all("other.py" not in tab for tab in driver.act("tabs"))
            if resources:
                preview=driver.act("action-choose",title="Controlled python fix",resources=True);assert "5 reviewed paths" in preview,preview
                assert (root/path).read_text()==source and not (root/"created.py").exists() and (root/"other.py").read_text()==source and (root/"delete.py").read_text()=="delete me\n"
                assert driver.act("construction-source",path=path)==original_draft and driver.act("editor-size",path=path)==size and driver.act("editor-is-retained",path=path)
                result=driver.act("change-resource-save");assert "Saved source resources" in result,result;driver.act("change-close")
                expected=original_draft.replace("value","python_fixed",1)
                assert (root/path).read_text()==expected and (root/"created.py").read_text()=="created = 1\n" and (root/"moved.py").read_text()=="other_fixed = 1\n" and not (root/"other.py").exists() and not (root/"delete.py").exists()
                assert driver.act("construction-source",path=path)==expected and driver.act("editor-is-retained",path=path) and driver.act("editor-size",path=path)==size
                driver.act("change-open",command="Undo workspace change");driver.act("change-resource-ready");result=driver.act("change-resource-save");assert "Saved source resources" in result,result;driver.act("change-close")
                assert (root/path).read_text()==source and (root/"other.py").read_text()==source and (root/"delete.py").read_text()=="delete me\n" and not (root/"created.py").exists() and not (root/"moved.py").exists()
                assert driver.act("construction-source",path=path)==original_draft and driver.act("editor-is-retained",path=path) and driver.act("editor-size",path=path)==size
                driver.act("change-open",command="Redo workspace change");driver.act("change-resource-ready");result=driver.act("change-resource-save");assert "Saved source resources" in result,result;driver.act("change-close")
                assert (root/path).read_text()==expected and (root/"created.py").exists() and (root/"moved.py").exists() and not (root/"other.py").exists() and not (root/"delete.py").exists()
                assert driver.act("construction-source",path=path)==expected and driver.act("editor-is-retained",path=path) and driver.act("editor-size",path=path)==size
                records=[json.loads(line) for line in events.read_text().splitlines()];assert len(records)>=2 and all(record.get("resolved")=="python" for record in records),records
                print("PASS: actual provider ordered text/create/move/delete proposal, unopened-source synchronization and requery, complete review, explicit saved publication, source-adjacent retained dirty baseline, guarded workspace undo/redo and retained editor geometry/identity")
                return
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
