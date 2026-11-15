<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Verify explicit cleanup through the existing menu

For maintainers changing workspace services independently of the editor renderer.

Drive a real engine and browser with a dirty Unicode source that has no language provider. Preview must not edit; application and workspace undo/redo must retain the saved baseline, editor identity and code dimensions. A no-op remains an observation. The shared Rust tests cover CRLF, other Unicode whitespace, bounds and invalid scope; native change-model witnesses cover held/closed replies. This does not establish physical native input or accessibility acceptance.

<ui:copy id="source-cleanup-browser-witness">
"""Explicit cleanup uses the existing menu, preview and workspace undo against a real engine."""
import importlib.util
import os
from pathlib import Path
import socket
import tempfile
ROOT=Path(__file__).resolve().parents[1]
def main():
 spec=importlib.util.spec_from_file_location("fixture",ROOT/"scripts/test-simple-browser.py");fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 with tempfile.TemporaryDirectory(prefix="simple-cleanup-") as temporary:
  folder=Path(temporary);root=folder/"workspace";root.mkdir();saved="😀 value \t\n  keep\nlast \t";draft=saved+" ";after="😀 value\n  keep\nlast";(root/"sample.txt").write_bytes(saved.encode())
  environment={**os.environ,"HICKORY_STATE_DIR":str(folder/"state")};processes=[];driver=None
  try:
   with socket.socket() as reservation:reservation.bind(("127.0.0.1",0));port=reservation.getsockname()[1]
   ui=f"http://127.0.0.1:{port}"
   engine=fixture.start([os.environ.get("HICKORY_SIMPLE_TEST_ENGINE",str(ROOT/"target/source-edit-focus/debug/examples/engine_client")),str(root),"window",ui],env=environment);processes.append(engine);backend=fixture.origin(engine)
   vite=fixture.start(["node","node_modules/vite/bin/vite.js","--config","vite.simple.config.ts","--host","127.0.0.1","--port",str(port),"--strictPort"],cwd=ROOT/"apps/web",env={**environment,"HICKORY_API_ORIGIN":backend});processes.append(vite)
   driver=fixture.BrowserDriver(fixture.origin(vite)+"/simple.html");driver.act("open",path="sample.txt");driver.act("remember-editor",path="sample.txt");size=driver.act("editor-size",path="sample.txt")
   driver.act("construction-set-source",path="sample.txt",source=draft)
   driver.act("change-open",command="Trim Trailing Whitespace…");preview=driver.act("change-prepare",command="Prepare cleanup");assert "Trim trailing whitespace" in preview and "Apply to source drafts" in preview,preview
   assert driver.act("construction-source",path="sample.txt")==draft
   driver.act("change-apply");driver.act("change-close");assert driver.act("construction-source",path="sample.txt")==after
   driver.act("change-open",command="Undo workspace change");driver.act("change-close");assert driver.act("construction-source",path="sample.txt")==draft
   driver.act("change-open",command="Redo workspace change");driver.act("change-close");assert driver.act("construction-source",path="sample.txt")==after
   driver.act("change-open",command="Trim Trailing Whitespace…");noop=driver.act("change-prepare",command="Prepare cleanup");assert "No trailing ASCII spaces or tabs" in noop,noop;driver.act("change-close")
   assert driver.act("editor-is-retained",path="sample.txt");assert driver.act("editor-size",path="sample.txt")==size
   assert (root/"sample.txt").read_bytes()==saved.encode()
   print("PASS: real engine provider-independent cleanup, existing menu/preview, exact Unicode draft changes and workspace undo/redo, no-op, retained editor/dimensions, unchanged saved bytes")
  finally:
   if driver:driver.close()
   for process in reversed(processes):fixture.stop(process)
if __name__=="__main__":main()
</ui:copy>

<ui:file path="scripts/test-simple-cleanup-browser.py"><ui:paste select="#source-cleanup-browser-witness"/></ui:file>
</ui:doc>
