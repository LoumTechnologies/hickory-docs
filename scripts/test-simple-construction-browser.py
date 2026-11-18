"""Drive the existing View menu against a real engine and retained authored draft."""
import importlib.util
import os
from pathlib import Path
import socket
import tempfile
ROOT=Path(__file__).resolve().parents[1]
def main():
 spec=importlib.util.spec_from_file_location("fixture",ROOT/"scripts/test-simple-browser.py");fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 with tempfile.TemporaryDirectory(prefix="simple-construction-") as temporary:
  folder=Path(temporary);root=folder/"workspace";root.mkdir()
  entry='<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:project sources="part.md"/></h:doc>'
  source='<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:copy id="body">one</h:copy><h:file path="output.txt"><h:paste select="#body"/></h:file></h:doc>'
  (root/"project.md").write_bytes(entry.encode());(root/"part.md").write_bytes(source.encode());draft=source.replace('>one<','>two<')
  environment={**os.environ,"HICKORY_STATE_DIR":str(folder/"state")};processes=[];driver=None
  try:
   with socket.socket() as reservation:reservation.bind(("127.0.0.1",0));port=reservation.getsockname()[1]
   ui=f"http://127.0.0.1:{port}"
   engine=fixture.start([os.environ.get("HICKORY_SIMPLE_TEST_ENGINE",str(ROOT/"target/source-edit-focus/debug/examples/engine_client")),str(root),"window",ui],env=environment);processes.append(engine);backend=fixture.origin(engine)
   vite=fixture.start(["node","node_modules/vite/bin/vite.js","--config","vite.simple.config.ts","--host","127.0.0.1","--port",str(port),"--strictPort"],cwd=ROOT/"apps/web",env={**environment,"HICKORY_API_ORIGIN":backend});processes.append(vite)
   driver=fixture.BrowserDriver(fixture.origin(vite)+"/simple.html");driver.act("open",path="part.md");driver.act("remember-editor",path="part.md");size=driver.act("editor-size",path="part.md")
   driver.act("construction-set-source",path="part.md",source=draft);actual=driver.act("construction-source",path="part.md");assert actual==draft,(draft,actual)
   driver.act("construction-open");text=driver.act("construction-read",entry="",focus="part.md");assert "modified" in text and "output.txt" in text and "not_queried" in text,text
   assert driver.act("construction-side",after=False)=="one";assert driver.act("construction-side",after=True)=="two"
   stale=driver.act("construction-origin",phase="before",path="part.md",stale=True);assert "Source changed" in stale and "declared copy #body" in stale,stale
   driver.act("construction-origin",phase="after",path="part.md",stale=False)
   selected=driver.act("construction-source",path="part.md",selection=True);assert selected["anchor"]==draft.index(">two<")+1,selected
   driver.act("construction-open");driver.act("construction-close")
   assert driver.act("editor-is-retained",path="part.md"), "Editor identity changed"
   after_size=driver.act("editor-size",path="part.md");assert after_size==size,(size,after_size)
   after_source=driver.act("construction-source",path="part.md");assert after_source==draft,(draft,after_source)
   driver.act("construction-history",redo=False);assert driver.act("construction-source",path="part.md")==source
   driver.act("construction-history",redo=True);assert driver.act("construction-source",path="part.md")==draft
   assert (root/"part.md").read_bytes()==source.encode();assert (root/"project.md").read_bytes()==entry.encode();assert not (root/"output.txt").exists()
   print("PASS: existing View menu, captured retained draft, compiler contribution/impact reading, saved/draft previews, stale-link refusal and exact matching-source reveal, retained editor/dimensions/undo/redo, unchanged source and no generated materialization")
  finally:
   if driver:driver.close()
   for process in reversed(processes):fixture.stop(process)
if __name__=="__main__":main()
