"""Actual engine protocol: read-only preparation followed by explicit reviewed resource saves."""
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
   consequences=call("changes/resource-consequences",{"inputs":inputs,"capture":body,"plan":proposal["plan"]});assert consequences["protected_targets"]==[] and not consequences["source_files_modified"] and len(consequences["fingerprint"])==64
   reviewed_save=json.loads(json.dumps({"request_id":"1"*32,"reviewed_fingerprint":consequences["fingerprint"],"proposal":{"inputs":inputs,"capture":body,"plan":proposal["plan"]}}))
   candidate_part=part.replace("main-generated.py","candidate-generated.py")
   edit2={"documentChanges":[{"textDocument":{"uri":"hick:///part.md","version":None},"edits":[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":len(part.encode("utf-16-le"))//2}},"newText":candidate_part}]},{"kind":"create","uri":"hick:///candidate-generated.py"}]}
   capture2={"context":"candidate-owned-output","edit":edit2};inputs2=call("changes/resource-inputs",capture2);plan2=call("changes/plan-resources",{"context":capture2["context"],"title":"Candidate output guard","edit":edit2,"sources":inputs2["sources"]})["plan"]
   consequences2=call("changes/resource-consequences",{"inputs":inputs2,"capture":capture2,"plan":plan2});assert consequences2["protected_targets"]==["candidate-generated.py"] and "candidate-generated.py" in consequences2["saved"]["output_paths"] and "candidate-generated.py" in consequences2["draft"]["output_paths"]
   assert (root/"part.md").read_text()==part and not (root/"candidate-generated.py").exists()
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
   records=root/".hick-cache"/"resource-changes"
   existing=set(records.iterdir()) if records.exists() else set()
   rejected={"request_id":"2"*32,"reviewed_fingerprint":consequences2["fingerprint"],"proposal":{"inputs":inputs2,"capture":capture2,"plan":plan2}}
   try:call("changes/resource-save",rejected)
   except urllib.error.HTTPError as error:assert error.code==409
   else:raise AssertionError("Candidate generated destination was published")
   assert not (root/"candidate-generated.py").exists() and (root/"part.md").read_text()==part
   saved_result=call("changes/resource-save",reviewed_save);outcome=saved_result["outcome"]
   assert outcome["complete"] and outcome["source_files_modified"] and not outcome["source_files_uncertain"]
   assert outcome["applied"]==["a.py","b.py"]
   assert not (root/"a.py").exists() and (root/"b.py").read_text()=="renamed\n# unsaved draft\n"
   record=json.loads((root/outcome["recovery"]).read_text());assert record["version"]==2 and record["before"]["a.py"]["text"]==saved
   assert next(patch for patch in record["reviewed_plan"]["patches"] if patch["path"]=="a.py")["before"]["text"]==draft
   after=set(records.iterdir());assert len(after-existing)==1
   assert call("changes/resource-save",reviewed_save)==saved_result and set(records.iterdir())==after
   with urllib.request.urlopen(origin+"/api/changes/resource-save?request_id="+reviewed_save["request_id"],timeout=30) as reply:assert json.load(reply)==saved_result
   changed=json.loads(json.dumps(reviewed_save));changed["reviewed_fingerprint"]="0"*64
   try:call("changes/resource-save",changed)
   except urllib.error.HTTPError as error:assert error.code==409
   else:raise AssertionError("A save request identity accepted different review content")
   assert set(records.iterdir())==after
   stale_edit={"documentChanges":[{"kind":"rename","oldUri":"hick:///b.py","newUri":"hick:///c.py"}]}
   stale_capture={"context":"stale-save","edit":stale_edit};stale_inputs=call("changes/resource-inputs",stale_capture)
   stale_plan=call("changes/plan-resources",{"context":stale_capture["context"],"title":"Stale move","edit":stale_edit,"sources":stale_inputs["sources"]})["plan"]
   stale_proposal={"inputs":stale_inputs,"capture":stale_capture,"plan":stale_plan};stale_facts=call("changes/resource-consequences",stale_proposal)
   (root/"b.py").write_text("independent edit\n")
   try:call("changes/resource-save",{"request_id":"3"*32,"reviewed_fingerprint":stale_facts["fingerprint"],"proposal":stale_proposal})
   except urllib.error.HTTPError as error:assert error.code==409
   else:raise AssertionError("Stale save changed a source")
   assert (root/"b.py").read_text()=="independent edit\n" and not (root/"c.py").exists() and set(records.iterdir())==after
   note='<h:doc xmlns:h="http://www.hickorydocs.com/1.0">Original note</h:doc>'
   (root/"note.md").write_text(note)
   note_edit={"documentChanges":[{"kind":"rename","oldUri":"hick:///note.md","newUri":"hick:///moved-note.md"}]}
   note_capture={"context":"move-authored-document","edit":note_edit};note_inputs=call("changes/resource-inputs",note_capture)
   note_plan=call("changes/plan-resources",{"context":note_capture["context"],"title":"Move note","edit":note_edit,"sources":note_inputs["sources"]})["plan"]
   note_proposal={"inputs":note_inputs,"capture":note_capture,"plan":note_plan};note_facts=call("changes/resource-consequences",note_proposal)
   note_result=call("changes/resource-save",{"request_id":"4"*32,"reviewed_fingerprint":note_facts["fingerprint"],"proposal":note_proposal})
   assert note_result["outcome"]["complete"] and not (root/"note.md").exists() and (root/"moved-note.md").read_text()==note,note_result
   root_edit={"documentChanges":[{"kind":"rename","oldUri":"hick:///project.md","newUri":"hick:///moved-project.md"}]}
   root_capture={"context":"canonical-entry-save-guard","edit":root_edit};root_inputs=call("changes/resource-inputs",root_capture)
   root_plan=call("changes/plan-resources",{"context":root_capture["context"],"title":"Move canonical entry","edit":root_edit,"sources":root_inputs["sources"]})["plan"]
   root_proposal={"inputs":root_inputs,"capture":root_capture,"plan":root_plan};root_facts=call("changes/resource-consequences",root_proposal)
   root_records=set(records.iterdir())
   try:call("changes/resource-save",{"request_id":"5"*32,"reviewed_fingerprint":root_facts["fingerprint"],"proposal":root_proposal})
   except urllib.error.HTTPError as error:assert error.code==409 and "canonical project.md" in error.read().decode()
   else:raise AssertionError("Resource save silently removed canonical entry ownership")
   assert (root/"project.md").read_text()==entry and not (root/"moved-project.md").exists() and set(records.iterdir())==root_records
   print("PASS: actual resource acquisition, planning and candidate ownership; explicit guarded save of a moved dirty draft, retained saved/draft recovery, duplicate/status identity, changed-request refusal, stale-source and generated-destination refusal, authored document retirement/move, canonical-entry ownership guard")
  finally:
   if engine:fixture.stop(engine)
if __name__=="__main__":main()
