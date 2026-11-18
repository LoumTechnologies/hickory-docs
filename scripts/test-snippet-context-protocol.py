"""Run source-bound context resolution through the actual shared engine."""
import importlib.util,json,os,subprocess,sys,tempfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('simple_fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
def main():
 with tempfile.TemporaryDirectory(prefix='snippet-context-',dir=ROOT/'target') as temporary:
  folder=Path(temporary);root=folder/'workspace';root.mkdir();events=folder/'events.jsonl';source='old\r\nsecond\n';paths=['variables.py','unused.py','denied.py']
  for path in paths:(root/path).write_bytes(source.encode())
  provider=ROOT/'scripts/snippet-context-controlled-provider.py';driver=ROOT/'scripts/snippet-context-protocol-driver.mjs'
  (root/'.hick-lsp.json').write_text(json.dumps({'servers':{'python':{'command':[sys.executable,str(provider),str(events)]}}}))
  engine=fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(root),'window','http://127.0.0.1:1'],env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state')})
  try:
   subprocess.run(['node',str(driver),fixture.origin(engine),str(events),str(root)],check=True,timeout=90)
   for path in paths:assert (root/path).read_bytes()==source.encode(),'Context resolution wrote source bytes'
  finally:fixture.stop(engine)
if __name__=='__main__':main()
