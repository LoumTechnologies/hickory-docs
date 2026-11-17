#!/usr/bin/env python3
"""Owned engine/child-provider acceptance for document-wide literate diagnostic fixes."""
import importlib.util,json,os,signal,subprocess,sys,tempfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
# Escaped tag openings keep executable fixture syntax literal in its owning construction.
SOURCE={
 'project.md':"\x3chick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\n# Readable prose precedes every construction\n\x3chick:project sources=\"a.md b.md unused.md\"/>\n\x3chick:file path=\"out/a.py\">\x3chick:paste select=\"#first\"/>\x3c/hick:file>\n\x3chick:file path=\"out/repeated.py\">\x3chick:paste select=\"#first\"/>\x3chick:paste select=\"#first\"/>\x3c/hick:file>\n\x3chick:file path=\"out/cross.py\">\x3chick:paste select=\"#first\"/>\x3chick:paste select=\"#second\"/>\x3c/hick:file>\n\x3c/hick:doc>\n",
 'a.md':"\x3chick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\n# Copy-only Unicode contributor\n\x3chick:copy id=\"first\">value = '😀'\n\x3c/hick:copy>\n\x3c/hick:doc>\n",
 'b.md':"\x3chick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\n# Another contributor with a standalone duplicate\n\x3chick:copy id=\"second\">value = 2\n\x3c/hick:copy>\n\x3chick:file path=\"out/b.py\">\x3chick:paste select=\"#second\"/>\x3c/hick:file>\n\x3c/hick:doc>\n",
 'unused.md':"\x3chick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\n# Loaded, but no output bytes originate here\n\x3c/hick:doc>\n",
}
def main():
 def interrupted(number,_frame):raise SystemExit(128+number)
 signal.signal(signal.SIGTERM,interrupted)
 spec=importlib.util.spec_from_file_location('fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 with tempfile.TemporaryDirectory(prefix='hickory-bulk-literate-') as temporary:
  folder=Path(temporary);root=folder/'workspace';root.mkdir();events=folder/'events.jsonl';gate=folder/'release';source=folder/'source.json';source.write_text(json.dumps(SOURCE))
  for path,text in SOURCE.items():(root/path).write_text(text)
  (root/'.hick-lsp.json').write_text(json.dumps({'servers':{'python':{'command':[sys.executable,str(ROOT/'scripts/controlled-bulk-literate-provider.py'),str(events),str(gate)]}}}))
  engine=fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(root),'window','http://127.0.0.1:1'],env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state')})
  try:
   subprocess.run(['node',str(ROOT/'scripts/provider-bulk-literate-driver.mjs'),fixture.origin(engine),str(events),str(gate),str(source)],cwd=ROOT,check=True,timeout=120)
   for path,text in SOURCE.items():assert (root/path).read_text()==text,f'Provider action changed saved source: {path}'
   print('PASS: all original Markdown source bytes retained; controlled-provider acceptance is not language semantic or physical-input/accessibility evidence.')
  finally:fixture.stop(engine)
if __name__=='__main__':main()
