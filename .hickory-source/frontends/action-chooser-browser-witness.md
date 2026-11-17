<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Resolve a provider action through the existing preview

For maintainers changing workspace services independently of the editor renderer.

The real engine starts a controlled lazy provider, so this flow tests protocol and UI integration rather than Python analysis. The existing Edit menu prepares its choice, selection resolves it, and the complete before/after preview leaves editor source untouched. Explicit application changes only the retained draft; coordinated undo restores it while editor identity, geometry and disk stay unchanged.

The `--bulk` variant enters through Preview file-wide diagnostic fixes, asserts the whole-source `source.fixAll` child context on both queries, and uses the same complete review and draft-only history. A contextual quick-fix query cannot satisfy this witness.

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
def debug_resource_acceptance(driver,root,phase,fixture,source):
    # Stable phase files preserve the identical proposal during required provider reacquisition.
    size=driver.act("editor-size",path="a.py");driver.act("debug-breakpoint",line=2);driver.act("debug-start",path="a.py");fixture.eventually(lambda:any("paused" in status for status in driver.act("debug-state")),True)
    position=driver.act("editor-position",path="a.py",anchor=source.index("value = value"),y=100)
    trace=[]
    def propose(operation,path):
        trace.append((operation,"before",driver.act("editor-position",path=path)));phase.write_text(operation);driver.act("change-open",command="Refactor and quick fixes");deadline=time.monotonic()+20
        while "Controlled python fix" not in (reading:=driver.act("change-prepare",command="Prepare provider actions")):
            assert time.monotonic()<deadline,reading;time.sleep(.1)
        driver.act("action-choose",title="Controlled python fix",resources=True);trace.append((operation,"review",driver.act("editor-position",path=path)));result=driver.act("change-resource-save");assert "Saved source resources" in result,result;driver.act("change-close")
    def retained(path):
        assert driver.act("editor-is-retained",path=path,previous="a.py") and driver.act("editor-size",path=path)==size
        assert driver.act("construction-source",path=path)==source
        assert (actual:=driver.act("editor-position",path=path))==position,(path,actual,position,trace)
        assert any("paused" in status for status in driver.act("debug-state")),driver.act("debug-diagnostics")
    propose("move","a.py");retained("moved.py");assert not (root/"a.py").exists() and (root/"moved.py").read_text()==source
    propose("swap","moved.py");retained("other.py");assert (root/"moved.py").read_text()=="unrelated = 7\n" and (root/"other.py").read_text()==source
    propose("recreate-a","other.py");driver.act("open",path="a.py");assert not any("paused" in state for state in driver.act("debug-state")),"New buffer adopted old-spelling session";driver.act("open",path="other.py");retained("other.py")
    propose("delete","other.py");retained("other.py");assert not (root/"other.py").exists();assert any("deleted source" in tab for tab in driver.act("tabs"))
    driver.act("change-open",command="Undo workspace change");driver.act("change-resource-ready");driver.act("change-resource-save");driver.act("change-close");retained("other.py");assert (root/"other.py").read_text()==source
    driver.act("change-open",command="Redo workspace change");driver.act("change-resource-ready");driver.act("change-resource-save");driver.act("change-close");retained("other.py")
    driver.act("open",path="a.py");propose("recreate-other","a.py");driver.act("open",path="other.py");driver.act("remember-editor",path="other.py");assert driver.act("construction-source",path="other.py")=="replacement = 99\n";assert not any("paused" in state for state in driver.act("debug-state")),"Independent reused spelling adopted retained launch"
    driver.act("source-tab",label="other.py (deleted source)");retained("other.py");assert "2" in driver.act("debug-watch",expression="value + 1");driver.act("debug-stop");fixture.eventually(lambda:not any(any(word in state for word in ["paused","running","starting","unconfirmed","stopping"]) for state in driver.act("debug-state")),True)
    driver.act("construction-set-source",path="other.py",source=source+"# retained copy\n");driver.act("file-command",command="Save As",path="other.py");assert (root/"other.py").read_text()=="replacement = 99\n"
    driver.act("file-command",command="Save As",path="a.py");fixture.eventually(lambda:"Original edits are retained" in driver.act("debug-diagnostics")["text"],True);assert driver.act("construction-source",path="other.py")==source+"# retained copy\n"
    driver.act("file-command",command="Save As",path="copied.py");fixture.eventually(lambda:(root/"copied.py").read_text() if (root/"copied.py").exists() else None,source+"# retained copy\n");fixture.eventually(lambda:any("deleted source" in tab for tab in driver.act("tabs")),False);driver.act("open",path="other.py");assert driver.act("editor-is-retained",path="other.py") and driver.act("construction-source",path="other.py")=="replacement = 99\n"
    wire=driver.act("debug-wire");assert len([item for item in wire if item.get("op")=="start"])==1,wire;assert len([item for item in wire if item.get("event")=="started"])==1,wire
    print("PASS: actual provider move/swap/delete/recreate/undo with one controlled debugger snapshot, exact retained editor geometry, independent reused paths and guarded deleted-source Save As")

def main():
    spec=importlib.util.spec_from_file_location("simple_fixture",ROOT/"scripts/test-simple-browser.py")
    fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
    bulk="--bulk" in sys.argv;resources="--resources" in sys.argv;debug_resources="--debug-resources" in sys.argv
    with tempfile.TemporaryDirectory(prefix="hickory-action-chooser-") as temporary:
        folder=Path(temporary);root=folder/"workspace";root.mkdir();path="a.py";source="value = 1\n";(root/path).write_text(source);(root/"other.py").write_text(source);events=folder/"events.jsonl"
        if resources:(root/"delete.py").write_text("delete me\n")
        phase=folder/"resource-phase";phase.write_text("move")
        if debug_resources:
            source="value = 1\npause_marker = value\nwhile True:\n    value = value\n"+"# retained scroll context\n"*80;(root/path).write_text(source);(root/"other.py").write_text("unrelated = 7\n")
            virtualenv=Path(os.environ.get("HICKORY_THREAD_TEST_VENV",str(ROOT/"target/thread-debug-env"))).resolve();assert (virtualenv/"bin/python").exists();(root/".venv").symlink_to(virtualenv,target_is_directory=True)
            (root/".hick-launch.json").write_text(json.dumps({"version":1,"profiles":[{"id":"resource-debug","name":"Resource debugger","program":path,"default":True}]}))
        (root/".hick-lsp.json").write_text(json.dumps({"servers":{"python":{"command":[sys.executable,str(ROOT/"scripts/controlled-action-provider.py"),"python",str(events)]}}}))
        environment={**os.environ,"HICKORY_STATE_DIR":str(folder/"state"),"HICKORY_CONTROLLED_ACTION_OTHER":"1",**({"HICKORY_CONTROLLED_ACTION_RESOURCES":"1"} if resources else {}),**({"HICKORY_CONTROLLED_DEBUG_RESOURCE_PHASE":str(phase)} if debug_resources else {})};processes=[];driver=None
        try:
            with socket.socket() as reservation:reservation.bind(("127.0.0.1",0));port=reservation.getsockname()[1]
            ui=f"http://127.0.0.1:{port}"
            engine=fixture.start([os.environ.get("HICKORY_SIMPLE_TEST_ENGINE",str(ROOT/"target/source-edit-focus/debug/examples/engine_client")),str(root),"window",ui],env=environment);processes.append(engine);backend=fixture.origin(engine)
            vite=fixture.start(["node","node_modules/vite/bin/vite.js","--config","vite.simple.config.ts","--host","127.0.0.1","--port",str(port),"--strictPort"],cwd=ROOT/"apps/web",env={**environment,"HICKORY_API_ORIGIN":backend});processes.append(vite)
            driver=fixture.BrowserDriver(fixture.origin(vite)+"/simple.html");driver.act("open",path=path);driver.act("remember-editor",path=path);size=driver.act("editor-size",path=path)
            if debug_resources:debug_resource_acceptance(driver,root,phase,fixture,source);return
            original_draft=source+"# retained draft\n" if resources else source
            if resources:driver.act("construction-set-source",path=path,source=original_draft)
            driver.act("change-open",command="Preview file-wide diagnostic fixes" if bulk else "Refactor and quick fixes");deadline=time.monotonic()+20
            while True:
                result=driver.act("change-prepare",command="Prepare file-wide diagnostic fixes" if bulk else "Prepare provider actions")
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
            records=[json.loads(line) for line in events.read_text().splitlines()];assert len([record for record in records if record.get("resolved")=="python"])==2,records
            if bulk:
                queries=[record for record in records if record.get("bulk_query")=="python"];assert len(queries)==2 and all(record["context"].get("only")==["source.fixAll"] and record["range"]=={"start":{"line":0,"character":0},"end":{"line":1,"character":0}} for record in queries),records
            assert (root/"other.py").read_text()==source
            driver.act("open",path="other.py");assert driver.act("search-source",path="other.py")==source
            print("PASS: menu, actual lazy resolution, unopened-target acquisition and fresh resolution, complete preview, explicit draft application, coordinated undo, retained editor geometry and unchanged disk")
        finally:
            if driver:driver.close()
            for process in reversed(processes):fixture.stop(process)
if __name__=="__main__":main()
</ui:copy>

<ui:file path="scripts/test-action-chooser-browser.py"><ui:paste select="#simple-action-chooser-browser-witness"/></ui:file>

## Verify native complete-file fixing against the same owned provider

The native runner owns its workspace, engine and provider processes. Each host must enter through the existing command descriptor, review the complete two-file plan after unopened-target reacquisition, and coordinate draft Apply/Undo/Redo while saved files remain unchanged. The Apple witness mounts a real editor; Android checks retained TextFieldState and scroll ownership. Exact single-test execution and recorded child contexts prevent a skipped test or a contextual quick-fix query from masquerading as bulk acceptance. Physical input and assistive technology remain unverified.

<ui:copy id="bulk-fix-native-runner">
#!/usr/bin/env python3
"""Owned provider-to-native complete-file fix review and coordinated draft history."""
import argparse,importlib.util,json,os,re,subprocess,sys,tempfile,xml.etree.ElementTree as ET
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('host',choices=['apple','android']);parser.add_argument('--skip-build',action='store_true');args=parser.parse_args()
 spec=importlib.util.spec_from_file_location('fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 with tempfile.TemporaryDirectory(prefix='hickory-native-bulk-') as temporary:
  folder=Path(temporary);root=folder/'workspace';root.mkdir();(root/'.hickory-bulk-fixture').write_text('owned-bulk-v1\n');source='value = 1\n'
  for path in ['a.py','other.py']:(root/path).write_text(source)
  events=folder/'events.jsonl';(root/'.hick-lsp.json').write_text(json.dumps({'servers':{'python':{'command':[sys.executable,str(ROOT/'scripts/controlled-action-provider.py'),'python',str(events)]}}}))
  env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state'),'HICKORY_CONTROLLED_ACTION_OTHER':'1'}
  engine=fixture.start([str(ROOT/'target/source-edit-focus/debug/examples/engine_client'),str(root),'window','http://127.0.0.1:1'],env=env)
  try:
   env.update(HICKORY_BULK_ORIGIN=fixture.origin(engine),HICKORY_BULK_WORKSPACE=str(root))
   if args.host=='apple':
    name='testLiveOwnedBulkFixCommandReviewApplyAndUndo';command=['swift','test','--package-path','apps/apple-simple','--scratch-path','target/apple-simple-spm','--filter','ActionChoiceTests.'+name+'$']+(['--skip-build'] if args.skip_build else [])
   else:
    name='liveOwnedBulkFixCommandReviewApplyAndUndo';command=[str(ROOT/'target/simple-android-gradle'),'-Pandroid.builder.sdkDownload=false',':app:testDebugUnitTest','--tests','com.hickorydocs.simple.ActionChoiceTest.'+name,'--rerun']
   log=ROOT/'target'/f'native-bulk-{args.host}.log'
   with log.open('w') as output:result=subprocess.run(command,cwd=ROOT,env=env,stdout=output,stderr=subprocess.STDOUT)
   if result.returncode:raise RuntimeError(f'{args.host} failed: {log}')
   if args.host=='apple':
    text=log.read_text();assert re.search(re.escape(name)+r'.*passed',text) and 'Executed 1 test, with 0 failures' in text and 'Test skipped' not in text,str(log)
   else:
    report=ROOT/'apps/android-simple/app/build/test-results/testDebugUnitTest/TEST-com.hickorydocs.simple.ActionChoiceTest.xml';rows=list(ET.parse(report).getroot().iter('testcase'));assert len(rows)==1 and rows[0].get('name','').split('(')[0]==name and not any(rows[0].find(key) is not None for key in ['skipped','failure','error']),str(report)
   records=[json.loads(line) for line in events.read_text().splitlines()];queries=[record for record in records if record.get('bulk_query')=='python'];assert len(queries)>=2 and all(record['context'].get('only')==['source.fixAll'] and record['range']=={'start':{'line':0,'character':0},'end':{'line':1,'character':0}} for record in queries),records
   assert len([record for record in records if record.get('resolved')=='python'])==2,records
   for path in ['a.py','other.py']:assert (root/path).read_text()==source
   print(f'PASS {args.host}: real provider bulk family, existing command, complete two-file preview/reacquisition, draft Apply/Undo/Redo, retained editor state and unchanged disk; no skipped acceptance. Physical input/accessibility unverified.')
  finally:fixture.stop(engine)
if __name__=='__main__':main()
</ui:copy>
<ui:file path="scripts/test-native-bulk-fixes.py"><ui:paste select="#bulk-fix-native-runner"/></ui:file>
</ui:doc>
