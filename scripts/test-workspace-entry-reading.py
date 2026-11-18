#!/usr/bin/env python3
"""Owned engine readings: stable role, fresh physical proofs and truthful refusal."""
import hashlib,importlib.util,json,os,tempfile,urllib.error
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
ENTRY='<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:project/></h:doc>\n'
def module(name,path):
 spec=importlib.util.spec_from_file_location(name,path);value=importlib.util.module_from_spec(spec);spec.loader.exec_module(value);return value
def main():
 fixture=module('fixture',ROOT/'scripts/test-simple-browser.py');history=module('history',ROOT/'scripts/test-agent-proposal-persistence.py');call=history.call
 def start(root,state):return fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(root),'window','http://127.0.0.1:1'],env={**os.environ,'HICKORY_STATE_DIR':str(state)})
 def get(origin):
  proof=call(origin,'GET','/api/workspace/entry');assert proof['version']==proof['entry_protocol']==1 and proof['source_effects'] is False
  assert len(proof['guard'])==64 and all(c in '0123456789abcdef' for c in proof['guard']);return proof
 def rejected(origin):history.refused(lambda:call(origin,'GET','/api/workspace/entry'))
 def note(origin):assert call(origin,'GET','/api/file?path=note.py')['content']=='value = 1\n'
 def doc(origin):
  tree=call(origin,'GET','/api/files')['tree']
  def leaves(nodes):
   for node in nodes:
    yield node
    yield from leaves(node.get('children',[]))
  node=next(node for node in leaves(tree) if node.get('path')=='project.md');return call(origin,'GET','/api/docs/'+node['doc_id'])['source']
 with tempfile.TemporaryDirectory(prefix='hickory-entry-reading-') as temporary:
  folder=Path(temporary).resolve();root=folder/'workspace';root.mkdir();state=folder/'state';(root/'project.md').write_text(ENTRY);(root/'note.py').write_text('value = 1\n');engine=start(root,state)
  try:
   origin=fixture.origin(engine);original=get(origin);assert original['entry']['path']=='project.md' and original['movable'] is True;assert original['source_hash']=='sha256:'+hashlib.sha256(ENTRY.encode()).hexdigest();assert get(origin)==original
   (root/'project.md').write_text(ENTRY+'edited\n');edited=get(origin);assert edited['entry']==original['entry'] and edited['guard']!=original['guard'] and edited['source_hash']!=original['source_hash']
   replacement=root/'replacement';replacement.write_text(ENTRY);os.replace(replacement,root/'project.md');undone=get(origin);assert undone['entry']==original['entry'] and undone['source_hash']==original['source_hash'] and undone['source_guard']!=original['source_guard'] and undone['guard']!=original['guard']
   (root/'project.md').write_text('<h:doc');malformed=get(origin);assert malformed['entry']==original['entry'] and malformed['movable'] is False;assert doc(origin)=='<h:doc';note(origin)
   (root/'project.md').write_text(ENTRY);retired=folder/'retired-main';os.replace(root/'project.md',retired);os.symlink(retired,root/'project.md');rejected(origin);note(origin);(root/'project.md').unlink();os.replace(retired,root/'project.md')
   (root/'project.md').unlink();rejected(origin);note(origin);(root/'project.md').write_text(ENTRY);current=get(origin);assert current['entry']==original['entry']
   records=list(state.rglob('entry.json'));assert len(records)==1;record=records[0];saved=record.read_bytes();forged=json.loads(saved);forged['payload']['current']['generation']+=1;record.write_text(json.dumps(forged));rejected(origin);note(origin);record.write_bytes(saved);before_restart=get(origin)
  finally:history.stop_engine(fixture,engine,state)
  engine=start(root,state)
  try:
   origin=fixture.origin(engine);fresh=get(origin);assert fresh['workspace']!=before_restart['workspace'] and fresh['guard']!=before_restart['guard'];assert fresh['entry']==before_restart['entry'] and fresh['namespace']==str(root);assert fresh['source_guard']==before_restart['source_guard'];note(origin)
  finally:history.stop_engine(fixture,engine,state)
  renamed=folder/'renamed';root.rename(renamed);engine=start(renamed,state)
  try:origin=fixture.origin(engine);rejected(origin);note(origin);assert doc(origin)==ENTRY
  finally:history.stop_engine(fixture,engine,state)
  assert (renamed/'project.md').read_text()==ENTRY and (renamed/'note.py').read_text()=='value = 1\n';assert not (renamed/'.hick-cache').exists()
  plain=folder/'plain';plain.mkdir();(plain/'note.py').write_text('value = 1\n');plain_state=folder/'plain-state';engine=start(plain,plain_state)
  try:
   origin=fixture.origin(engine);absent=get(origin);assert absent['entry']['path'] is None and absent['movable'] is False and 'source_guard' not in absent and 'source_hash' not in absent;(plain/'project.md').write_text(ENTRY);bound=get(origin);assert bound['entry']['identity']==absent['entry']['identity'] and bound['entry']['generation']==absent['entry']['generation']+1;assert bound['entry']['path']=='project.md';(plain/'project.md').unlink();rejected(origin);note(origin)
  finally:history.stop_engine(fixture,engine,plain_state)
 print('PASS real readonly entry path: stable retained role, repeated exact proof, edited/Undo equal-byte physical proof retirement, malformed source still readable, missing/symlink/tampered binding refusal, genuine restart new runtime proof, renamed physical root namespace refusal, one-time initial no-main discovery. No entry Move/admission/application or physical/accessibility interaction is claimed.')
if __name__=='__main__':main()
