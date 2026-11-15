"""Read immutable Git conflict stages while retaining an unrelated unsaved editor draft."""
import importlib.util
import os
from pathlib import Path
import socket
import subprocess
import tempfile
ROOT=Path(__file__).resolve().parents[1]
def main():
 spec=importlib.util.spec_from_file_location("fixture",ROOT/"scripts/test-simple-browser.py");fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 with tempfile.TemporaryDirectory(prefix="simple-conflict-") as temporary:
  folder=Path(temporary);root=folder/"workspace";root.mkdir();path="literal[ab] name.txt"
  def git(*args,check=True):return subprocess.run(["git","-C",str(root),"--literal-pathspecs",*args],check=check,stdout=subprocess.PIPE,stderr=subprocess.PIPE).stdout
  git("init","-b","main");git("config","user.email","test@example.invalid");git("config","user.name","Conflict fixture")
  (root/path).write_text("base\n");git("add","--",path);git("commit","-m","base");git("checkout","-b","incoming")
  (root/path).write_text("incoming\n");git("commit","-am","incoming");git("checkout","main");(root/path).write_text("current\n");git("commit","-am","current");git("merge","incoming",check=False)
  saved=(root/path).read_bytes();draft=saved.decode()+"# unsaved\n";index=(root/".git/index").read_bytes();head=git("rev-parse","HEAD")
  environment={**os.environ,"HICKORY_STATE_DIR":str(folder/"state")};processes=[];driver=None
  try:
   with socket.socket() as reservation:reservation.bind(("127.0.0.1",0));port=reservation.getsockname()[1]
   ui=f"http://127.0.0.1:{port}"
   engine=fixture.start([os.environ.get("HICKORY_SIMPLE_TEST_ENGINE",str(ROOT/"target/source-edit-focus/debug/examples/engine_client")),str(root),"window",ui],env=environment);processes.append(engine);backend=fixture.origin(engine)
   vite=fixture.start(["node","node_modules/vite/bin/vite.js","--config","vite.simple.config.ts","--host","127.0.0.1","--port",str(port),"--strictPort"],cwd=ROOT/"apps/web",env={**environment,"HICKORY_API_ORIGIN":backend});processes.append(vite)
   driver=fixture.BrowserDriver(fixture.origin(vite)+"/simple.html");driver.act("open",path=path);driver.act("remember-editor",path=path);size=driver.act("editor-size",path=path)
   driver.act("construction-set-source",path=path,source=draft);driver.act("review-open");driver.act("review-select",path=path,staged=False)
   text=driver.act("review-conflict-read");assert "Captured immutable Git stages" in text and "rebase" in text,text
   for stage,expected in [(1,"base\n"),(2,"current\n"),(3,"incoming\n")]:assert driver.act("review-conflict-stage",stage=stage)==expected
   driver.act("review-close");assert driver.act("construction-source",path=path)==draft
   assert driver.act("editor-is-retained",path=path);assert driver.act("editor-size",path=path)==size
   driver.act("construction-history",redo=False);assert driver.act("construction-source",path=path)==saved.decode();driver.act("construction-history",redo=True);assert driver.act("construction-source",path=path)==draft
   assert (root/path).read_bytes()==saved;assert (root/".git/index").read_bytes()==index;assert git("rev-parse","HEAD")==head
   print("PASS: real conflict stage 1/2/3 inspection through repository review, literal path, retained dirty source/editor/dimensions/undo, unchanged HEAD/index/working bytes")
  finally:
   if driver:driver.close()
   for process in reversed(processes):fixture.stop(process)
if __name__=="__main__":main()
