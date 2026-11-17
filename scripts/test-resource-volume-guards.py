#!/usr/bin/env python3
"""Actual saved/draft/candidate output-folder refusals, with no source publication."""
import copy,importlib.util,json,os,tempfile,urllib.error,urllib.request
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def main():
 spec=importlib.util.spec_from_file_location('fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 with tempfile.TemporaryDirectory(prefix='hickory-volume-guards-') as temporary:
  folder=Path(temporary).resolve();root=folder/'workspace';root.mkdir();entry='<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:project sources="part.md"/></h:doc>';part='<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:volume name="generated" output="old-out"/><h:file path="existing.py">x</h:file></h:doc>';(root/'project.md').write_text(entry);(root/'part.md').write_text(part)
  for name in ['old-out','new-out','new-out-sibling']:(root/name).mkdir()
  engine=fixture.start([str(ROOT/'target/source-edit-focus/debug/examples/engine_client'),str(root),'window','http://127.0.0.1:1'],env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state')})
  try:
   origin=fixture.origin(engine)
   def get(route):
    with urllib.request.urlopen(origin+'/api/'+route,timeout=30) as reply:return json.load(reply)
   def call(route,body):
    req=urllib.request.Request(origin+'/api/changes/'+route,data=json.dumps(body).encode(),headers={'Content-Type':'application/json'},method='POST')
    with urllib.request.urlopen(req,timeout=30) as reply:return json.load(reply)
   def refused(route,body,status=409,contains=None):
    try:call(route,body)
    except urllib.error.HTTPError as error:
     detail=error.read().decode();assert error.code==status,(error.code,detail)
     if contains:assert contains in detail,detail
    else:raise AssertionError('A protected output-folder intention was admitted: '+route)
   node=next(row for row in get('files')['tree'] if row['path']=='part.md');guard=get('docs/'+node['doc_id'])['save_guard']
   names=['old-out/fresh.ts','new-out/fresh.ts','new-out-sibling/fresh.ts'];edit={'documentChanges':[{'kind':'create','uri':'hick:///'+name} for name in names]};capture={'context':'dirty-prefix','ownership_sources':{'part.md':part.replace('old-out','new-out')},'saved_guards':{'part.md':guard},'edit':edit};inputs=call('resource-inputs',capture)
   for name in names[:2]:assert inputs['sources'][name]=={'kind':'missing','creatable':False}
   assert inputs['sources'][names[2]]=={'kind':'missing','creatable':True}
   forged=copy.deepcopy(inputs)
   for name in names[:2]:forged['sources'][name]['creatable']=True
   plan=call('plan-resources',{'context':capture['context'],'title':'Forged permissions','edit':edit,'sources':forged['sources']})['plan'];refused('resource-consequences',{'inputs':forged,'capture':capture,'plan':plan},409)
   changes={'documentChanges':[{'textDocument':{'uri':'hick:///part.md','version':None},'edits':[{'range':{'start':{'line':0,'character':0},'end':{'line':0,'character':len(part)}},'newText':part.replace('old-out','new-out')}]},{'kind':'create','uri':'hick:///new-out/fresh.ts'}]};capture={'context':'candidate-prefix','edit':changes};inputs=call('resource-inputs',capture);plan=call('plan-resources',{'context':capture['context'],'title':'Candidate folder guard','edit':changes,'sources':inputs['sources']})['plan'];facts=call('resource-consequences',{'capture':capture,'inputs':inputs,'plan':plan});assert facts['protected_targets']==['new-out/fresh.ts'];assert not facts['source_files_modified']
   for layer in ['saved','draft']:assert 'new-out/fresh.ts' not in facts[layer]['output_paths'],'A folder declaration was presented as produced output'
   refused('resource-save',{'request_id':'d'*32,'reviewed_fingerprint':facts['fingerprint'],'proposal':{'capture':capture,'inputs':inputs,'plan':plan}},contains='generated')
   assert get('changes/resource-recoveries')['ids']==[]
   for name in names:assert not(root/name).exists()
   assert(root/'part.md').read_text()==part and(root/'project.md').read_text()==entry
   print('PASS: saved and dirty folder claims, candidate folder change, forged creation permission and explicit publication refusal; no output materialization or source/recovery publication. Folder claims remain separate from observed compiler outputs.')
  finally:fixture.stop(engine)
if __name__=='__main__':main()
