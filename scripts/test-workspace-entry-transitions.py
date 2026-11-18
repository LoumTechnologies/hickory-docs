#!/usr/bin/env python3
"""Owned real engine: reviewed main relocation, saved baselines, restart and inverse."""
import hashlib,importlib.util,json,os,tempfile,urllib.parse
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
MAIN='<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\n<h:project sources="part.md"/>\n<h:file path="safe.txt">safe baseline</h:file>\n</h:doc>\n'
DIRTY=MAIN.replace('safe baseline','retained dirty body')
PART='<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:file path="part.txt">old scope</h:file></h:doc>\n'
NEXT_PART=PART.replace('old scope','new selected scope')
def module(name,path):
 spec=importlib.util.spec_from_file_location(name,path);value=importlib.util.module_from_spec(spec);spec.loader.exec_module(value);return value
def main():
 fixture=module('browser_fixture',ROOT/'scripts/test-simple-browser.py');support=module('owned_history',ROOT/'scripts/test-agent-proposal-persistence.py');call=support.call
 def start(root,state):return fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(root),'entry-transition','http://127.0.0.1:1'],env={**os.environ,'HICKORY_STATE_DIR':str(state)})
 def post(origin,path,body):return call(origin,'POST','/api/'+path,body)
 def entry(origin):return call(origin,'GET','/api/workspace/entry')
 def intent(origin,operation,selection=None):return post(origin,'workspace/entry/intent',{'version':1,'entry_selection':selection or entry(origin),'operation':operation})
 def prepare(origin,review,drafts=None,guards=None):
  capture={'context':review['context'],'edit':review['edit'],'drafts':drafts or {},'ownership_sources':{path:row['text'] for path,row in (drafts or {}).items()},'saved_guards':guards or {}}
  inputs=None
  for _ in range(2):inputs=post(origin,'workspace/entry/inputs',{'version':1,'token':review['token'],'capture':capture});assert not inputs['refusals'],inputs
  if review['kind']=='move':
   plan=post(origin,'changes/plan-resources',{'context':capture['context'],'title':review['title'],'edit':review['edit'],'sources':inputs['sources']})['plan'];proposal={'inputs':inputs,'capture':capture,'plan':plan};facts=post(origin,'changes/resource-consequences',proposal);return proposal,facts,None
  proposal={'recovery_id':review['_record']['id'],'inputs':inputs,'capture':capture};prepared=post(origin,'changes/prepare-restoration',proposal);return proposal,prepared['consequences'],prepared['restoration']['recovery_fingerprint']
 def publish(origin,review,proposal,facts,identifier,recovery_fingerprint=None):
  body={'request_id':identifier,'reviewed_fingerprint':facts['fingerprint'],'proposal':proposal};path='changes/resource-save'
  if recovery_fingerprint is not None:path='changes/restore-resources';body['reviewed_recovery_fingerprint']=recovery_fingerprint
  result=post(origin,path,body);assert result['outcome']['complete'],result;outcome=result['entry_outcome'];assert outcome['state']=='confirmed' and outcome['before_entry']==review['before_entry'] and outcome['after_entry']==review['after_entry'] and outcome['current_entry']==review['after_entry'];return result
 def record(origin,result):
  identifier=Path(result['outcome']['recovery']).parent.name;value=call(origin,'GET','/api/changes/resource-recovery?id='+identifier);assert value['entry_transition']['state']=='retained';return value
 def restore(origin,result,identifier):
  old=record(origin,result);proof=entry(origin);review=intent(origin,{'kind':'restoration','recovery_id':old['id'],'recovery_fingerprint':old['fingerprint']},proof);review['_record']=old
  manifest=json.loads((root/result['outcome']['recovery']).read_text());path=proof['entry']['path'];after=next(patch['after'] for patch in manifest['reviewed_plan']['patches'] if patch['path']==path);saved=(root/path).read_text();guard=proof['source_guard'];drafts={path:{'text':after['text'],'base_sha256':hashlib.sha256(saved.encode()).hexdigest(),'version':after.get('version'),'writable':True,'save_guard':guard}}
  proposal,facts,fingerprint=prepare(origin,review,drafts,{path:guard});return publish(origin,review,proposal,facts,identifier,fingerprint)
 with tempfile.TemporaryDirectory(prefix='hickory-entry-transitions-') as temporary:
  folder=Path(temporary).resolve();root=folder/'workspace';root.mkdir();state=folder/'state';(root/'next').mkdir();(root/'project.md').write_text(MAIN);(root/'part.md').write_text(PART);(root/'next/part.md').write_text(NEXT_PART);(root/'note.py').write_text('readable = True\n');engine=start(root,state)
  try:
   origin=fixture.origin(engine);before=entry(origin);review=intent(origin,{'kind':'move','destination':'next/main.spec'},before)
   guard=before['source_guard'];drafts={'project.md':{'text':DIRTY,'base_sha256':hashlib.sha256(MAIN.encode()).hexdigest(),'version':7,'writable':True,'save_guard':guard}}
   proposal,facts,_=prepare(origin,review,drafts,{'project.md':guard});assert 'next/main.spec' in facts['saved']['entries'] and 'next/safe.txt' in facts['saved']['output_paths'] and 'next/part.txt' in facts['saved']['output_paths'];assert facts['saved']['source_hashes']['next/part.md']==hashlib.sha256(NEXT_PART.encode()).hexdigest()
   moved=publish(origin,review,proposal,facts,'a'*32);assert not (root/'project.md').exists() and (root/'next/main.spec').read_text()==MAIN
   recovery=json.loads((root/moved['outcome']['recovery']).read_text());assert recovery['before']['project.md']['text']==MAIN and recovery['after']['next/main.spec']['text']==MAIN;assert next(patch['after']['text'] for patch in recovery['reviewed_plan']['patches'] if patch['path']=='next/main.spec')==DIRTY
   after=entry(origin);assert after['entry']==review['after_entry'] and after['entry']['identity']==before['entry']['identity'];support.refused(lambda:post(origin,'workspace/entry/inputs',{'version':1,'token':review['token'],'capture':proposal['capture']}))
   (root/'project.md').write_text('<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:project/><h:file path="wrong.txt">not main</h:file></h:doc>');assert entry(origin)['entry']==after['entry'];(root/'project.md').unlink()
  finally:support.stop_engine(fixture,engine,state)
  engine=start(root,state)
  try:
   origin=fixture.origin(engine);fresh=entry(origin);assert fresh['entry']==after['entry'] and fresh['workspace']!=after['workspace'];support.refused(lambda:post(origin,'workspace/entry/inputs',{'version':1,'token':review['token'],'capture':proposal['capture']}))
   (root/'unselected.spec').write_text(MAIN)
   files=call(origin,'GET','/api/files')['tree']
   moved_node=next(child for folder in files if folder['path']=='next' for child in folder['children'] if child['path']=='next/main.spec')
   ordinary=next(node for node in files if node['path']=='unselected.spec')
   assert moved_node['doc_id'] and moved_node['language']=='markdown',moved_node
   assert ordinary.get('doc_id') is None and ordinary.get('language') is None,ordinary
   document=call(origin,'GET','/api/docs/'+urllib.parse.quote(moved_node['doc_id'],safe=''))
   assert document['path']=='next/main.spec' and document['source']==MAIN and document['save_guard']==fresh['source_guard'],document
   undone=restore(origin,moved,'b'*32);assert (root/'project.md').read_text()==MAIN and not (root/'next/main.spec').exists();assert entry(origin)['entry']['path']=='project.md'
   redone=restore(origin,undone,'c'*32);assert (root/'next/main.spec').read_text()==MAIN and not (root/'project.md').exists();assert entry(origin)['entry']['generation']==before['entry']['generation']+3
   frozen=entry(origin);late=intent(origin,{'kind':'move','destination':'next/final.spec'},frozen);replacement=root/'replacement';replacement.write_text(MAIN);os.replace(replacement,root/'next/main.spec');support.refused(lambda:post(origin,'workspace/entry/inputs',{'version':1,'token':late['token'],'capture':{'context':late['context'],'edit':late['edit']}}));assert call(origin,'GET','/api/file?path=note.py')['content']=='readable = True\n'
   assert not (root/'next/final.spec').exists()
  finally:support.stop_engine(fixture,engine,state)
 print('PASS real reviewed main Move: exact dirty source, separate saved baseline, selected new compilation, old alias not default, genuine restart, inverse/redo, stale/old token refusal, readable independent source. Older multi-step rebase and physical/accessibility input are not claimed by this witness.')
if __name__=='__main__':main()
