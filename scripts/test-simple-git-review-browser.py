"""Existing View menu, actual engine and temporary Git state; no source save."""
import importlib.util
import os
from pathlib import Path
import socket
import subprocess
import tempfile
ROOT=Path(__file__).resolve().parents[1]
def main():
 spec=importlib.util.spec_from_file_location("fixture",ROOT/"scripts/test-simple-browser.py");fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 with tempfile.TemporaryDirectory(prefix="simple-git-review-") as temporary:
  folder=Path(temporary);root=folder/"workspace";root.mkdir()
  def git(*args):return subprocess.run(["git","-C",str(root),"--literal-pathspecs",*args],check=True,capture_output=True)
  source="value = 1\n";(root/"main.py").write_text(source);old='quoted "é -> .py';new=':(glob)*.py';(root/old).write_text("renamed = True\n")
  git("init","-q");git("add","--",".");git("-c","user.name=Fixture","-c","user.email=fixture@example.invalid","commit","-qm","base")
  (root/"removed.py").write_text("temporary\n");git("mv","--",old,new);saved="value = 2\n";(root/"main.py").write_text(saved)
  environment={**os.environ,"HICKORY_STATE_DIR":str(folder/"state")};processes=[];driver=None
  try:
   with socket.socket() as reservation:reservation.bind(("127.0.0.1",0));port=reservation.getsockname()[1]
   ui=f"http://127.0.0.1:{port}"
   engine=fixture.start([os.environ.get("HICKORY_SIMPLE_TEST_ENGINE",str(ROOT/"target/source-edit-focus/debug/examples/engine_client")),str(root),"window",ui],env=environment);processes.append(engine);backend=fixture.origin(engine)
   vite=fixture.start(["node","node_modules/vite/bin/vite.js","--config","vite.simple.config.ts","--host","127.0.0.1","--port",str(port),"--strictPort"],cwd=ROOT/"apps/web",env={**environment,"HICKORY_API_ORIGIN":backend});processes.append(vite)
   driver=fixture.BrowserDriver(fixture.origin(vite)+"/simple.html");driver.act("open",path="main.py");driver.act("remember-editor",path="main.py");size=driver.act("editor-size",path="main.py")
   driver.act("source-tests-edit",path="main.py",text="# unsaved draft\n");draft=driver.act("search-source",path="main.py");assert "unsaved draft" in draft
   caret=driver.act("caret-offset",path="main.py");text=driver.act("review-open");assert new in text and old in text and "main.py" in text,text
   (root/"removed.py").unlink();assert driver.act("review-select",path="removed.py",staged=False)=="";assert "Git diff refused" in driver.act("review-status")
   diff=driver.act("review-select",path="main.py",staged=False);assert "+value = 2" in diff and "unsaved draft" not in diff,diff
   driver.act("review-stage");staged=git("diff","--cached","--","main.py").stdout.decode();assert "+value = 2" in staged and "unsaved draft" not in staged,staged
   driver.act("review-select",path="main.py",staged=True);driver.act("review-unstage");assert git("diff","--cached","--","main.py").stdout==b"";assert (root/"main.py").read_text()==saved
   diff=driver.act("review-select",path=new,staged=True);assert "+renamed = True" in diff,diff
   driver.act("review-unstage");assert git("diff","--cached","--name-only").stdout==b"";assert (root/new).read_text()=="renamed = True\n";assert not (root/old).exists()
   diff=driver.act("review-select",path="main.py",staged=True);assert diff=="",diff
   driver.act("review-close");sha=git("rev-parse","HEAD").stdout.decode().strip()
   for filtered in [False,True]:
    text=driver.act("history-open",file=filtered);assert "base" in text,text
    text=driver.act("history-read",sha=sha);assert "Immutable commit reading" in text,text;driver.act("history-file",path="main.py")
    assert driver.act("history-side",side="after")==source;assert driver.act("history-side",side="before")=="";driver.act("history-close")
   driver.act("open",path=new);text=driver.act("history-open",file=True);assert "0 reported commits" in text,text;driver.act("history-close");driver.act("open",path="main.py")
   driver.act("focus-source",path="main.py");actual_caret=driver.act("caret-offset",path="main.py");assert actual_caret==caret,{"before_caret":caret,"after_caret":actual_caret}
   driver.act("undo",path="main.py");assert driver.act("search-source",path="main.py")==saved,{"undo":driver.act("search-source",path="main.py")}
   driver.act("redo",path="main.py");assert driver.act("search-source",path="main.py")==draft,{"redo":driver.act("search-source",path="main.py")}
   assert driver.act("editor-is-retained",path="main.py");assert driver.act("editor-size",path="main.py")==size;assert driver.act("search-source",path="main.py")==draft;assert (root/"main.py").read_text()==saved;assert git("rev-parse","HEAD").stdout.decode().strip()==sha;assert git("diff","--cached","--name-only").stdout==b""

   print("PASS: existing View menu, literal renamed paths, working-tree/index diffs, saved-file stage/unstage and complete rename unstaging, immutable repository/file history and literal path filtering, unsaved draft retained, stable code dimensions and unchanged disk")
  finally:
   if driver:driver.close()
   for process in reversed(processes):fixture.stop(process)
if __name__=="__main__":main()
