#!/usr/bin/env python3
"""Owned HTTP/Yjs proof of an unreserved no-effect Move refusal and fresh retry."""
import hashlib,importlib.util,json,os,select,subprocess,tempfile,urllib.parse
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def module(name,path):
 spec=importlib.util.spec_from_file_location(name,path);value=importlib.util.module_from_spec(spec);spec.loader.exec_module(value);return value
def main():
 fixture=module('entry_failure_fixture',ROOT/'scripts/test-simple-browser.py');support=module('entry_failure_support',ROOT/'scripts/test-agent-proposal-persistence.py');move=module('entry_failure_move',ROOT/'scripts/test-workspace-entry-transitions.py');call=support.call
 def start(root,state):return fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(root),'entry-failure-owned','http://127.0.0.1:1'],env={**os.environ,'HICKORY_STATE_DIR':str(state)})
 def post(origin,path,value):return call(origin,'POST','/api/'+path,value)
 def prepare(origin):
  proof=call(origin,'GET','/api/workspace/entry');review=post(origin,'workspace/entry/intent',{'version':1,'entry_selection':proof,'operation':{'kind':'move','destination':'next.md'}})
  guard=proof['source_guard'];capture={'context':review['context'],'edit':review['edit'],'drafts':{'project.md':{'text':move.DIRTY,'base_sha256':hashlib.sha256(move.MAIN.encode()).hexdigest(),'version':7,'writable':True,'save_guard':guard}},'ownership_sources':{'project.md':move.DIRTY},'saved_guards':{'project.md':guard}}
  for _ in range(2):inputs=post(origin,'workspace/entry/inputs',{'version':1,'token':review['token'],'capture':capture});assert not inputs['refusals'],inputs
  plan=post(origin,'changes/plan-resources',{'context':capture['context'],'title':review['title'],'edit':review['edit'],'sources':inputs['sources']})['plan'];proposal={'inputs':inputs,'capture':capture,'plan':plan};facts=post(origin,'changes/resource-consequences',proposal);return proof,review,proposal,facts
 with tempfile.TemporaryDirectory(prefix='hickory-entry-noeffect-') as temporary:
  folder=Path(temporary).resolve();root=folder/'workspace';root.mkdir();state=folder/'state';foreign='<h:doc xmlns:h="http://www.hickorydocs.com/1.0">foreign live room</h:doc>\n';original={'project.md':move.MAIN,'part.md':move.PART,'note.py':'readable = True\n'}
  for path,text in original.items():(root/path).write_bytes(text.encode())
  (root/'next.md').write_bytes(foreign.encode());engine=start(root,state);room=None
  try:
   origin=fixture.origin(engine);nodes=call(origin,'GET','/api/files')['tree'];node=next(node for node in nodes if node['path']=='next.md');identifier=node['doc_id'];assert identifier,node
   helper=Path(os.environ.get('HICKORY_ENTRY_ROOM_HELPER',str(ROOT/'scripts/control-entry-room.mjs')))
   room=subprocess.Popen(['node',str(helper),origin,identifier,foreign],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
   assert select.select([room.stdout],[],[],20)[0],'Owned room handshake timed out';ready=json.loads(room.stdout.readline());assert ready=={'ready':True,'source':foreign,'writes_sent':0},ready
   # Establish a missing destination while retaining a genuine foreign room.
   # Nothing is sent to mutate Yjs, so no debounce can republish the removed file.
   (root/'next.md').unlink();proof,review,proposal,facts=prepare(origin);identifier='d'*32;request={'request_id':identifier,'reviewed_fingerprint':facts['fingerprint'],'proposal':proposal};refused=post(origin,'changes/resource-save',request)
   effect=refused['outcome'];entry=refused['entry_outcome'];assert refused['request_id']==identifier,refused;assert effect['complete'] is False and effect['source_files_modified'] is False and effect['source_files_uncertain'] is False and effect['applied']==[],refused
   assert entry['state']=='no_effect' and entry['request_id']==identifier and entry['reservation_consumed'] is False and entry['before_entry']==proof['entry'] and entry['current_entry']==proof['entry'] and entry['after_entry']==review['after_entry'],refused
   assert refused['source_files']==[] and refused.get('saved_sources') is None,refused
   assert 'live editor owns the missing source path' in effect['error'].lower(),refused
   assert call(origin,'GET','/api/workspace/entry')['entry']==proof['entry'];assert not (root/'next.md').exists()
   for path,text in original.items():assert (root/path).read_bytes()==text.encode(),path
   record=json.loads((root/effect['recovery']).read_bytes());assert record['before']['project.md']['text']==move.MAIN;assert next(patch['before']['text'] for patch in record['reviewed_plan']['patches'] if patch['path']=='project.md')==move.DIRTY
   repeated=post(origin,'changes/resource-save',request);assert all(repeated[key]==refused[key] for key in ['request_id','entry_outcome','outcome','intended_saved_sources','saved_sources','source_files']),'Same request must retain exact refusal evidence without another writer';assert repeated['adoption_current'] is False
   room.stdin.write('close\n');room.stdin.flush();room.wait(timeout=15);assert room.returncode==0,room.stderr.read();room=None
  finally:
   if room is not None:room.terminate();room.wait(timeout=10)
   support.stop_engine(fixture,engine,state)
  # A genuine restart retires the read-only orphan room deterministically.
  engine=start(root,state)
  try:
   origin=fixture.origin(engine);fresh,review,proposal,facts=prepare(origin);assert fresh['entry']==proof['entry'] and fresh['workspace']!=proof['workspace'];assert not (root/'next.md').exists()
   result=post(origin,'changes/resource-save',{'request_id':'e'*32,'reviewed_fingerprint':facts['fingerprint'],'proposal':proposal});assert result['outcome']['complete'] is True and result['entry_outcome']['state']=='confirmed',result
   assert (root/'next.md').read_bytes()==move.MAIN.encode() and not (root/'project.md').exists();assert result['entry_outcome']['current_entry']==review['after_entry'];assert call(origin,'GET','/api/workspace/entry')['entry']==review['after_entry']
   saved=json.loads((root/result['outcome']['recovery']).read_bytes());assert next(patch['after']['text'] for patch in saved['reviewed_plan']['patches'] if patch['path']=='next.md')==move.DIRTY
   assert len(list((root/'.hick-cache/resource-changes').glob('*/manifest.json')))==2,'One refused record and exactly one fresh published record'
   assert call(origin,'GET','/api/file?path=note.py')['content']==original['note.py']
  finally:support.stop_engine(fixture,engine,state)
 print('PASS real foreign Yjs room + HTTP: exact unreserved no-effect receipt, zero source/adoption effects, original dirty evidence retained, exact request refusal replay, genuine restart and one fresh reviewed Move. No Yjs mutations/debounce assumptions; physical input/accessibility and interrupted repair are not claimed.')
if __name__=='__main__':main()
