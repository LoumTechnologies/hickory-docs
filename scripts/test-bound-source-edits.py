#!/usr/bin/env python3
import importlib.util,json,os,subprocess,tempfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
SOURCE={'a.py':'value = 1\n','b.py':'value = 2\n','hidden.py':'value = 3\n','document.md':'\x3chick:doc xmlns:hick="http://www.hickorydocs.com/1.0">\n\x3chick:file path="a-good.py">value = 1\n\x3c/hick:file>\n\x3chick:file path="z-bad.py">value = 2\n\x3c/hick:file>\n\x3c/hick:doc>\n','unmapped.md':'\x3chick:doc xmlns:hick="http://www.hickorydocs.com/1.0">\n\x3chick:file path="unmapped.py">value = 3\n\x3c/hick:file>\n\x3c/hick:doc>\n'}
def main():
 spec=importlib.util.spec_from_file_location('fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 with tempfile.TemporaryDirectory(prefix='hickory-bound-edits-') as temporary:
  folder=Path(temporary);root=folder/'workspace';root.mkdir();events=folder/'events';gate=folder/'gate';source=folder/'source.json';source.write_text(json.dumps(SOURCE))
  for path,text in SOURCE.items():(root/path).write_text(text)
  (root/'.hick-lsp.json').write_text(json.dumps({'servers':{'python':{'command':[os.sys.executable,str(ROOT/'scripts/controlled-bound-source-provider.py'),str(events),str(gate)]}}}))
  engine=fixture.start([str(ROOT/'target/source-edit-focus/debug/examples/engine_client'),str(root),'window','http://127.0.0.1:1'],env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state')})
  try:
   subprocess.run(['node',str(ROOT/'scripts/bound-source-edits-driver.mjs'),fixture.origin(engine),str(events),str(gate),str(source)],check=True,timeout=120)
   for path,text in SOURCE.items():assert (root/path).read_text()==text,path
   print('PASS: all authored fixture disk bytes retained. Controlled provider protocol evidence does not certify language semantics or physical/accessibility acceptance.')
  finally:fixture.stop(engine)
if __name__=='__main__':main()
