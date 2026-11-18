#!/usr/bin/env python3
"""Owned older main-Move Undo/Redo through real review, producer and editor paths."""
import argparse,hashlib,importlib.util,json,os,socket,tempfile,urllib.parse,urllib.error,uuid
from contextlib import ExitStack
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def load(path,name):
 spec=importlib.util.spec_from_file_location(name,ROOT/path);value=importlib.util.module_from_spec(spec);spec.loader.exec_module(value);return value
move=load('scripts/test-workspace-entry-move-hosts.py','lineage_move_helpers')
A=move.ORIGINAL;B=move.DESTINATION;C='second #main.md';SAVED=move.PUBLISHED_SOURCE;DRAFT=move.DRAFT_SOURCE;EARLIER=SAVED.replace('saved.txt','before.txt')
def call(origin,method,path,body=None):return load('scripts/test-agent-proposal-persistence.py','lineage_http').call(origin,method,path,body)
def proof(origin):return call(origin,'GET','/api/workspace/entry')
def history(origin,path):return call(origin,'GET','/api/source-history?'+urllib.parse.urlencode({'path':path}))
def seed_history(origin):
 tree=call(origin,'GET','/api/files')['tree'];identifier=next(node['doc_id'] for node in tree if node['path']==A)
 document=call(origin,'GET','/api/docs/'+urllib.parse.quote(identifier,safe=''));assert document['source']==EARLIER
 identifier_save=uuid.uuid4().hex
 saved=call(origin,'PUT','/api/source-history/docs/'+urllib.parse.quote(identifier,safe=''),{'version':1,'request_id':identifier_save,'save_guard':document['save_guard'],'source':SAVED,'expected':{'source':EARLIER,'disk':EARLIER}})
 assert saved['history']['available'] is True and saved['history']['outcome']=='published' and saved['history']['id']==identifier_save,saved
 return identifier_save

def browser(driver,root,origin,seed):
 first=proof(origin)['entry'];driver.act('open',path=A);driver.act('remember-editor',path=A)
 location=SAVED.index('saved.txt');driver.act('source-text-select',path=A,anchor=location,head=location+len('saved.txt'));driver.act('type-source-selection',path=A,text='draft.txt')
 selection=driver.act('editor-position',path=A);size=driver.act('editor-size',path=A)
 def check(path,generation,position=None):
  stamp=move.entry(origin);assert stamp=={**first,'path':path,'generation':first['generation']+generation},stamp
  assert(root/path).read_text()==SAVED;assert all(not(root/other).exists() for other in [A,B,C] if other!=path)
  assert driver.act('construction-source',path=path)==DRAFT
  assert driver.act('editor-is-retained',path=path,previous=A)
  actual_position=driver.act('editor-position',path=path)
  expected_position=selection if position is None else position
  assert actual_position==expected_position,(actual_position,expected_position)
  assert driver.act('editor-size',path=path)==size
  assert not(root/'saved.txt').exists() and not(root/'draft.txt').exists()
 def publish(destination):
  driver.act('change-open',command='Move main construction…');driver.act('change-prepare',name=destination,input='New workspace-relative path',command='Prepare move');preview=driver.act('change-resource-ready');assert destination in preview,preview
  result=driver.act('change-resource-save');assert 'Saved source resources' in result,result;driver.act('change-close')
 check(A,0);publish(B);check(B,1);publish(C);check(C,2)
 for direction,path,generation in [('Undo',B,3),('Undo',A,4),('Redo',B,5),('Redo',C,6)]:
  driver.act('change-open',command=direction+' workspace change');preview=driver.act('change-resource-ready');assert path in preview,preview
  result=driver.act('change-resource-save');assert 'Saved source resources' in result,result;driver.act('change-close');check(path,generation)
 value=driver.act('local-history-open');assert 'unavailable' not in value.lower(),value
 selected=driver.act('local-history-select',id=seed,side='before');assert selected['id']==seed and selected['text']==EARLIER and selected['enabled'],selected;driver.act('local-history-close')
 before_history=driver.act('source-text-state',path=C)
 driver.act('undo',path=C)
 assert driver.act('construction-source',path=C)==SAVED
 driver.act('redo',path=C)
 # CodeMirror restores the replacement range on Redo. The publications above
 # preserve the original post-typing caret independently of this native edit.
 check(C,6,{**selection,'anchor':location,'head':location+len('draft.txt')})
 after_history=driver.act('source-text-state',path=C)
 assert (after_history['undo'],after_history['redo'])==(before_history['undo'],before_history['redo'])
 # A fresh review is held before an unrelated equal-byte inode replaces its source.
 driver.act('change-open',command='Undo workspace change');driver.act('change-resource-ready');state=driver.act('source-text-state',path=C)
 previous=(root/C).stat().st_ino;replacement=root/'independent-replacement';replacement.write_text(SAVED);os.replace(replacement,root/C);assert(root/C).stat().st_ino!=previous
 result=driver.act('change-resource-save');assert 'Saved source resources' not in result and any(word in result.lower() for word in ['changed','replac','physical','stale','inspect','publish','publication']),result
 assert driver.act('source-text-state',path=C)==state;assert(root/C).read_text()==SAVED;assert all(not(root/path).exists() for path in [A,B]);driver.act('change-close')
 print('PASS browser: Move A→B→C, reviewed Undo twice/Redo twice, exact role generations, saved/draft separation, original editor/selection/geometry/local history, native text Undo retained, equal-byte foreign inode refusal and no generated output materialization')

def protocol(root,origin,seed):
 first=proof(origin)['entry'];outcomes=[]
 def prepared(operation):
  current=proof(origin);review=call(origin,'POST','/api/workspace/entry/intent',{'version':1,'entry_selection':current,'operation':operation})
  path=current['entry']['path'];guard=current['source_guard'];capture={'context':review['context'],'edit':review['edit'],'drafts':{path:{'text':DRAFT,'base_sha256':hashlib.sha256(SAVED.encode()).hexdigest(),'version':99,'writable':True,'save_guard':guard}},'ownership_sources':{path:DRAFT},'saved_guards':{path:guard}}
  for _ in range(2):inputs=call(origin,'POST','/api/workspace/entry/inputs',{'version':1,'token':review['token'],'capture':capture});assert not inputs['refusals'],inputs
  proposal={'inputs':inputs,'capture':capture}
  if operation['kind']=='move':
   proposal['plan']=call(origin,'POST','/api/changes/plan-resources',{'context':review['context'],'title':review['title'],'edit':review['edit'],'sources':inputs['sources']})['plan'];facts=call(origin,'POST','/api/changes/resource-consequences',proposal);route='/api/changes/resource-save';extra={}
  else:
   proposal['recovery_id']=operation['recovery_id'];restoration=call(origin,'POST','/api/changes/prepare-restoration',proposal);facts=restoration['consequences'];route='/api/changes/restore-resources';extra={'reviewed_recovery_fingerprint':restoration['restoration']['recovery_fingerprint']}
  return route,{'request_id':uuid.uuid4().hex,'reviewed_fingerprint':facts['fingerprint'],'proposal':proposal,**extra},review,capture
 def run(operation,expected,generation):
  route,body,review,_=prepared(operation);result=call(origin,'POST',route,body);assert result['outcome']['complete'] and result['entry_outcome']['state']=='confirmed',result
  assert result['entry_outcome']['current_entry']==review['after_entry']=={**first,'generation':first['generation']+generation,'path':expected};assert(root/expected).read_text()==SAVED
  assert all(not(root/path).exists() for path in [A,B,C] if path!=expected)
  record=call(origin,'GET','/api/changes/resource-recovery?id='+Path(result['outcome']['recovery']).parent.name);outcomes.append(record);return record
 def inverse(record):return {'kind':'restoration','recovery_id':record['id'],'recovery_fingerprint':record['fingerprint']}
 m1=run({'kind':'move','destination':B},B,1);m2=run({'kind':'move','destination':C},C,2);u2=run(inverse(m2),B,3);u1=run(inverse(m1),A,4);run(inverse(u1),B,5);run(inverse(u2),C,6)
 assert any(row['id']==seed and row['available'] for row in history(origin,C)['entries'])
 def refused(route,body):
  try:result=call(origin,'POST',route,body)
  except urllib.error.HTTPError as error:
   assert error.code==409,(error.code,error.read());reason=error.read(4096).decode();assert any(word in reason.lower() for word in ['source','entry','changed','physical','producer','generation']),reason
  else:
   assert result['outcome']['complete'] is False and result['outcome']['source_files_modified'] is False and result['outcome']['source_files_uncertain'] is False,result
 held_route,held_body,_,_=prepared(inverse(outcomes[-1]));extra='unrelated main.md';run({'kind':'move','destination':extra},extra,7);run({'kind':'move','destination':C},C,8);refused(held_route,held_body);assert not(root/extra).exists()
 route,body,_,_=prepared(inverse(outcomes[-1]));previous=(root/C).stat().st_ino;replacement=root/'independent-replacement';replacement.write_text(SAVED);os.replace(replacement,root/C);assert(root/C).stat().st_ino!=previous;refused(route,body)
 assert(root/C).read_text()==SAVED and all(not(root/path).exists() for path in [A,B]);print('PASS protocol: exact complete older producer Undo/Redo, saved/dirty plans, retained local history, independent same-path role ABA and equal-byte inode refusals; no UI or physical-input claim')

def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--host',choices=['protocol','browser','apple','android'],required=True);parser.add_argument('--engine',default=os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')));args=parser.parse_args()
 fixture=move.module('scripts/test-simple-browser.py','lineage_browser_fixture');native=move.module('scripts/test-native-templates.py','lineage_native_fixture');owned=move.module('scripts/test-agent-proposal-persistence.py','lineage_owned_engine')
 with tempfile.TemporaryDirectory(prefix='entry-lineage-host-',dir=ROOT/'target') as temporary:
  folder=Path(temporary);root=folder/'workspace';root.mkdir();(root/'.git').mkdir();(root/'.hickory-entry-move-fixture').write_text('owned-entry-move-v1\n');(root/A).write_text(EARLIER);(root/'part.md').write_text('\x3ch:doc xmlns:h="http://www.hickorydocs.com/1.0">\x3ch:copy id="part">source\x3c/h:copy>\x3c/h:doc>\n')
  with socket.socket() as reservation:reservation.bind(('127.0.0.1',0));port=reservation.getsockname()[1]
  ui=f'http://127.0.0.1:{port}';env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state')}
  with ExitStack() as cleanup:
   engine=fixture.start([args.engine,str(root),'window',ui],env=env);cleanup.callback(owned.stop_engine,fixture,engine,folder/'state');origin=fixture.origin(engine);seed=seed_history(origin);baseline=proof(origin)['entry'];os.environ['HICKORY_ENTRY_LINEAGE_HISTORY']=seed
   if args.host=='protocol':protocol(root,origin,seed)
   elif args.host=='browser':
    vite=fixture.start(['node','node_modules/vite/bin/vite.js','--config','vite.simple.config.ts','--host','127.0.0.1','--port',str(port),'--strictPort'],cwd=ROOT/'apps/web',env={**env,'HICKORY_API_ORIGIN':origin});cleanup.callback(fixture.stop,vite);driver=fixture.BrowserDriver(fixture.origin(vite)+'/simple.html');cleanup.callback(driver.close);browser(driver,root,origin,seed)
   else:
    case=('WorkspaceEntryLineageTests','testLiveOlderMainMoveUndoRedo','entry-lineage') if args.host=='apple' else ('WorkspaceEntryLineageTest','liveOlderMainMoveUndoRedo','entry-lineage');native.run_host(args.host,origin,root,folder,fixture,acceptance=case)
   assert(root/C).read_text()==SAVED and all(not(root/path).exists() for path in [A,B]);assert not(root/'saved.txt').exists() and not(root/'draft.txt').exists()
   try:retained=proof(origin)['entry']
   except urllib.error.HTTPError as error:assert error.code==409,(error.code,error.read())
   else:assert retained['identity']==baseline['identity'] and retained['path']==C and retained['generation']>=baseline['generation']+6,retained
if __name__=='__main__':main()
