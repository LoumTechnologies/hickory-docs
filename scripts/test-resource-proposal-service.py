"""Actual engine protocol: resource preparation never publishes the saved or draft source."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import urllib.error
import urllib.request
ROOT=Path(__file__).resolve().parents[1]
def main():
 spec=importlib.util.spec_from_file_location("fixture",ROOT/"scripts/test-simple-browser.py");fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 with tempfile.TemporaryDirectory(prefix="resource-service-") as temporary:
  folder=Path(temporary);root=folder/"workspace";root.mkdir();saved="saved\n";draft=saved+"# unsaved draft\n";(root/"a.py").write_bytes(saved.encode());engine=None
  entry='<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:project sources="part.md"/></h:doc>'
  part='<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:file path="main-generated.py">main</h:file></h:doc>'
  (root/"project.md").write_text(entry);(root/"part.md").write_text(part)
  (root/"other.md").write_text(entry.replace("part.md","other-part.md"));(root/"other-part.md").write_text(part.replace("main-generated.py","other-generated.py"))
  try:
   engine=fixture.start([os.environ.get("HICKORY_SIMPLE_TEST_ENGINE",str(ROOT/"target/source-edit-focus/debug/examples/engine_client")),str(root),"window","http://127.0.0.1:1"],env={**os.environ,"HICKORY_STATE_DIR":str(folder/"state")});origin=fixture.origin(engine)
   def call(route,body):
    request=urllib.request.Request(origin+"/api/"+route,data=json.dumps(body,ensure_ascii=False).encode(),headers={"Content-Type":"application/json"},method="POST")
    with urllib.request.urlopen(request,timeout=30) as reply:return json.load(reply)
   edit={"documentChanges":[{"kind":"rename","oldUri":"hick:///a.py","newUri":"hick:///b.py"},{"textDocument":{"uri":"hick:///b.py","version":7},"edits":[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":5}},"newText":"renamed"}]}]}
   targets=call("changes/resource-targets",{"edit":edit});assert targets["paths"]==["a.py","b.py"] and not targets["acquired"]
   body={"context":"service-fixture","edit":edit,"drafts":{"a.py":{"text":draft,"base_sha256":hashlib.sha256(saved.encode()).hexdigest(),"version":7,"writable":True}}}
   inputs=call("changes/resource-inputs",body);assert inputs["context"]==body["context"] and inputs["workspace"] and not inputs["source_files_modified"];assert inputs["sources"]["a.py"]["text"]==draft;assert inputs["saved_sources"]["a.py"]["text"]==saved;assert inputs["saved_sources"]["b.py"]["kind"]=="missing"
   proposal=call("changes/plan-resources",{"context":body["context"],"title":"Move draft","edit":edit,"sources":inputs["sources"]});assert not proposal["applied"] and not proposal["saved"];patches={patch["path"]:patch for patch in proposal["plan"]["patches"]};assert patches["a.py"]["after"]["kind"]=="missing";assert patches["b.py"]["after"]["text"]=="renamed\n# unsaved draft\n";assert patches["b.py"]["after"]["identity"]==inputs["sources"]["a.py"]["identity"]
   body["drafts"]["a.py"]["base_sha256"]=hashlib.sha256(b"old").hexdigest()
   try:call("changes/resource-inputs",body)
   except urllib.error.HTTPError as error:assert error.code==422
   else:raise AssertionError("Stale saved base was accepted")
   protected=["main-generated.py","draft-generated.py","other-generated.py"]
   inputs=call("changes/resource-inputs",{"context":"narrow-reading","project_entry":"other.md","ownership_sources":{"part.md":part.replace("main-generated.py","draft-generated.py")},"edit":{"documentChanges":[{"kind":"create","uri":"hick:///"+path} for path in protected]}})
   assert inputs["ownership"]["protected_entries"]==["other.md","project.md"]
   for path in protected:assert inputs["sources"][path]=={"kind":"missing","creatable":False} and not (root/path).exists()
   try:call("construction/reading",{"entry":"project.md","sources":{},"missing_sources":["part.md"]})
   except urllib.error.HTTPError as error:assert error.code==422
   else:raise AssertionError("Deleted construction source fell back to saved bytes")
   reading=call("construction/reading",{"entry":"project.md","sources":{"project.md":entry.replace("part.md","moved.md"),"moved.md":part.replace(">main<",">moved<")},"missing_sources":["part.md"]})
   output=next(item for item in reading["outputs"] if item["path"]=="main-generated.py");assert output["before"]["preview"]=="main" and output["after"]["preview"]=="moved"
   assert (root/"part.md").read_text()==part and (root/"project.md").read_text()==entry and not (root/"moved.md").exists()
   assert (root/"a.py").read_bytes()==saved.encode();assert not (root/"b.py").exists()
   print("PASS: actual resource target/acquisition/planning routes, held root identity, separate saved/draft text, complete move/text delta, stale-base refusal, main/selected saved/draft ownership union, explicit deleted/moved construction namespace, unchanged saved files")
  finally:
   if engine:fixture.stop(engine)
if __name__=="__main__":main()
