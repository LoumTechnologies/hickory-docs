"""Drive source recovery from the View menu without replacing the live editor."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import socket
import tempfile
import urllib.request
ROOT=Path(__file__).resolve().parents[1]
def main():
 spec=importlib.util.spec_from_file_location("fixture",ROOT/"scripts/test-simple-browser.py");fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 with tempfile.TemporaryDirectory(prefix="simple-recovery-") as temporary:
  folder=Path(temporary);root=folder/"workspace";root.mkdir();saved="old saved 😀\n";draft="retained draft 😀\n";(root/"a #😀.txt").write_text(saved)
  environment={**os.environ,"HICKORY_STATE_DIR":str(folder/"state")};processes=[];driver=None
  try:
   with socket.socket() as reservation:reservation.bind(("127.0.0.1",0));port=reservation.getsockname()[1]
   ui=f"http://127.0.0.1:{port}"
   engine=fixture.start([os.environ.get("HICKORY_SIMPLE_TEST_ENGINE",str(ROOT/"target/source-edit-focus/debug/examples/engine_client")),str(root),"window",ui],env=environment);processes.append(engine);backend=fixture.origin(engine)
   def call(route,body):
    request=urllib.request.Request(backend+"/api/changes/"+route,data=json.dumps(body).encode(),headers={"Content-Type":"application/json"},method="POST")
    with urllib.request.urlopen(request,timeout=30) as reply:return json.load(reply)
   edit={"documentChanges":[{"kind":"rename","oldUri":"hick:///a%20%23%F0%9F%98%80.txt","newUri":"hick:///b.txt"}]}
   capture={"context":"browser-recovery","edit":edit,"drafts":{"a #😀.txt":{"text":draft,"base_sha256":hashlib.sha256(saved.encode()).hexdigest(),"version":None,"writable":True}}}
   inputs=call("resource-inputs",capture);plan=call("plan-resources",{"context":capture["context"],"title":"Move retained draft","edit":edit,"sources":inputs["sources"]})["plan"];proposal={"inputs":inputs,"capture":capture,"plan":plan};facts=call("resource-consequences",proposal)
   result=call("resource-save",{"request_id":"a"*32,"reviewed_fingerprint":facts["fingerprint"],"proposal":proposal});assert result["outcome"]["complete"];record=Path(result["outcome"]["recovery"]).parent.name
   vite=fixture.start(["node","node_modules/vite/bin/vite.js","--config","vite.simple.config.ts","--host","127.0.0.1","--port",str(port),"--strictPort"],cwd=ROOT/"apps/web",env={**environment,"HICKORY_API_ORIGIN":backend});processes.append(vite)
   driver=fixture.BrowserDriver(fixture.origin(vite)+"/simple.html");driver.act("open",path="b.txt");driver.act("remember-editor",path="b.txt");size=driver.act("editor-size",path="b.txt");current=draft+"local editor change\n";driver.act("construction-set-source",path="b.txt",source=current)
   driver.act("recovery-open");assert driver.act("recovery-read",record=record,path="a #😀.txt",snapshot="saved_before")==saved;assert driver.act("recovery-read",snapshot="draft_before")==draft;assert driver.act("recovery-read",snapshot="saved_after") is None
   driver.act("recovery-close");assert driver.act("editor-is-retained",path="b.txt");assert driver.act("editor-size",path="b.txt")==size;assert driver.act("construction-source",path="b.txt")==current
   driver.act("construction-history",redo=False);assert driver.act("construction-source",path="b.txt")==draft;driver.act("construction-history",redo=True);assert driver.act("construction-source",path="b.txt")==current
   assert (root/"b.txt").read_text()==draft and not (root/"a #😀.txt").exists()
   print("PASS: View menu recovery discovery, exact named saved/dirty/absent snapshot reading, retained editor geometry/identity/draft/undo/redo, unchanged saved source")
  finally:
   if driver:driver.close()
   for process in reversed(processes):fixture.stop(process)
if __name__=="__main__":main()
