"""Actual recovery admission keeps the address-bound canonical entry coherent."""
import importlib.util,json,os,tempfile,urllib.request,urllib.error
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def main():
 spec=importlib.util.spec_from_file_location("fixture",ROOT/"scripts/test-simple-browser.py");fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 with tempfile.TemporaryDirectory(prefix="entry-restoration-") as temporary:
  folder=Path(temporary);root=folder/"workspace";root.mkdir();engine=None
  source='<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:project sources=""/></h:doc>'
  try:
   engine=fixture.start([os.environ.get("HICKORY_SIMPLE_TEST_ENGINE",str(ROOT/"target/source-edit-focus/debug/examples/engine_client")),str(root),"window","http://127.0.0.1:1"],env={**os.environ,"HICKORY_STATE_DIR":str(folder/"state")});origin=fixture.origin(engine)
   def call(route,body):
    request=urllib.request.Request(origin+"/api/"+route,data=json.dumps(body).encode(),headers={"Content-Type":"application/json"},method="POST")
    with urllib.request.urlopen(request,timeout=30) as reply:return json.load(reply)
   def save(edit,context,identifier,entry=None):
    capture={"context":context,"edit":edit}
    if entry is not None:capture["project_entry"]=entry
    inputs=call("changes/resource-inputs",capture)
    plan=call("changes/plan-resources",{"context":context,"title":context,"edit":edit,"sources":inputs["sources"]})["plan"]
    proposal={"inputs":inputs,"capture":capture,"plan":plan};facts=call("changes/resource-consequences",proposal)
    return call("changes/resource-save",{"request_id":identifier,"reviewed_fingerprint":facts["fingerprint"],"proposal":proposal})
   def restoration(result,context,identifier):
    record=json.loads((root/result["outcome"]["recovery"]).read_text())
    capture={"context":context,"edit":{"changes":{"hick:///"+path:[] for path in record["after"]}}}
    proposal={"recovery_id":Path(result["outcome"]["recovery"]).parent.name,"inputs":call("changes/resource-inputs",capture),"capture":capture}
    prepared=call("changes/prepare-restoration",proposal)
    return {"request_id":identifier,"proposal":proposal,"reviewed_fingerprint":prepared["consequences"]["fingerprint"],"reviewed_recovery_fingerprint":prepared["restoration"]["recovery_fingerprint"]}
   created=save({"documentChanges":[{"kind":"create","uri":"hick:///project.md"},{"textDocument":{"uri":"hick:///project.md","version":None},"edits":[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":0}},"newText":source}]}]},"Create canonical entry","a"*32)
   assert created["outcome"]["complete"] and (root/"project.md").read_text()==source
   removed=call("changes/restore-resources",restoration(created,"Undo entry creation","b"*32))
   assert removed["outcome"]["complete"] and not (root/"project.md").exists()
   (root/"entry.md").write_text(source)
   moved=save({"documentChanges":[{"kind":"rename","oldUri":"hick:///entry.md","newUri":"hick:///project.md"}]},"Move selected entry to canonical address","c"*32,"entry.md")
   assert moved["outcome"]["complete"] and not (root/"entry.md").exists()
   record_folder=root/".hick-cache/resource-changes";before=set(record_folder.iterdir())
   try:call("changes/restore-resources",restoration(moved,"Refuse implicit canonical entry move","d"*32))
   except urllib.error.HTTPError as error:assert error.code==409 and "canonical project.md" in error.read().decode()
   else:raise AssertionError("Historical restoration silently moved canonical entry ownership")
   assert (root/"project.md").read_text()==source and not (root/"entry.md").exists() and set(record_folder.iterdir())==before
   print("PASS: canonical creation can be restored to absence, but historical moves cannot silently discard canonical entry ownership")
  finally:
   if engine:fixture.stop(engine)
if __name__=="__main__":main()
