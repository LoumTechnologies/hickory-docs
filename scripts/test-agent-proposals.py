#!/usr/bin/env python3
"""Owned-engine coherent proposal preparation and complete-intention admission."""
import hashlib,importlib.util,json,os,tempfile,urllib.error,urllib.request,uuid
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def call(origin,method,route,body=None):
 raw=None if body is None else json.dumps(body).encode()
 with urllib.request.urlopen(urllib.request.Request(origin+route,data=raw,method=method,headers={'Content-Type':'application/json'}),timeout=30) as response:return json.load(response)
def refused(action):
 try:action()
 except urllib.error.HTTPError as error:
  assert error.code in (400,403,404,409,422),(error.code,error.read());return
 raise AssertionError('A stale or forged proposal was admitted')
def suite(origin,root):
 before={path.name:path.read_bytes() for path in root.iterdir() if path.is_file()}
 capture={'context':'owned-proposal','project_entry':'project.md','drafts':{},'ownership_sources':{},'unaddressed_sources':{},'saved_guards':{},'edit':{'changes':{'hick:///note.py':[{'range':{'start':{'line':0,'character':0},'end':{'line':1,'character':0}},'newText':'value = 41\n'}]}}}
 intention={'version':1,'request_id':uuid.uuid4().hex,'origin':'owned provider fixture','title':'Explain and change one source','explanation':'Change the implementation together with its expected behavior. This is a proposal, not an observed verdict.','claims':['Agent claims tests passed; this remains unverified agent text.'],'check_ids':[],'capture':capture,'reading':{'version':1,'entry':'project.md','sources':{},'missing_sources':[],'unaddressed_sources':{},'saved_guards':{},'focus':'note.py'}}
 review=call(origin,'POST','/api/agent-proposals/prepare',intention);assert review['proposal_protocol']==1 and not review['source_effects'] and not review['checks_executed'];assert hashlib.sha256(review['material'].encode()).hexdigest()==review['fingerprint'];material=json.loads(review['material']);assert material['intention']==intention and material['namespace']==str(root) and material['workspace']==review['workspace'];assert material['origin_authority'].startswith('caller-provided')
 assert material['plan']['patches'][0]['before']['text']=='value = 1\n' and material['plan']['patches'][0]['after']['text']=='value = 41\n'
 assert material['consequences']['protected_targets']==[] and material['before']['version']==1 and material['after']['version']==1
 admission={'version':1,'token':review['token'],'fingerprint':review['fingerprint']}
 confirmed=call(origin,'POST','/api/agent-proposals/validate',admission);assert confirmed['admitted'] and confirmed['fingerprint']==review['fingerprint'] and confirmed['namespace']==str(root) and not confirmed['source_effects']
 refused(lambda:call(origin,'POST','/api/agent-proposals/validate',{**admission,'fingerprint':'f'*64}))
 listed=call(origin,'GET','/api/agent-proposals');assert listed['namespace']==str(root) and any(row['token']==review['token'] for row in listed['reviews']);assert call(origin,'GET','/api/agent-proposals/'+review['token'])==review
 report={'version':1,'token':review['token'],'fingerprint':review['fingerprint'],'workspace':review['workspace'],'namespace':str(root),'request_id':uuid.uuid4().hex,'host':'owned protocol fixture (host claim only)','after_hashes':{'note.py':hashlib.sha256(b'value = 41\n').hexdigest()}}
 refused(lambda:call(origin,'POST','/api/agent-proposals/report',{**report,'after_hashes':{}}))
 claimed=call(origin,'POST','/api/agent-proposals/report',report);assert claimed['status']=='host-reported draft application' and not claimed['saved'] and not claimed['checks_executed'];assert call(origin,'POST','/api/agent-proposals/report',report)==claimed
 refused(lambda:call(origin,'POST','/api/agent-proposals/report',{**report,'request_id':uuid.uuid4().hex}));assert call(origin,'GET','/api/agent-proposals/'+review['token'])['host_application']==claimed
 assert (root/'note.py').read_text()=='value = 1\n','A host report must not apply or save source'
 for field,value in [('explanation','changed reasoning'),('claims',['forged verdict']),('check_ids',['not-in-engine-reading'])]:
  refused(lambda field=field,value=value:call(origin,'POST','/api/agent-proposals/validate',{**admission,'intention':{**intention,field:value}}))
 refused(lambda:call(origin,'POST','/api/agent-proposals/prepare',{**intention,'request_id':uuid.uuid4().hex,'check_ids':['invented-check']}))
 generated={**intention,'request_id':uuid.uuid4().hex,'capture':{**capture,'edit':{'changes':{'hick:///generated.py':[{'range':{'start':{'line':0,'character':0},'end':{'line':1,'character':0}},'newText':'forbidden\n'}]}}}}
 refused(lambda:call(origin,'POST','/api/agent-proposals/prepare',generated))
 replacement=root/'replacement.tmp';replacement.write_bytes((root/'note.py').read_bytes());os.replace(replacement,root/'note.py');refused(lambda:call(origin,'POST','/api/agent-proposals/validate',admission))
 assert {path.name:path.read_bytes() for path in root.iterdir() if path.is_file()}==before
 print('PASS coherent proposal HTTP: complete explanation/implementation/consequence/readings, exact unchanged admission, sealed byte-exact material, immutable admission, retained review/list and host reports, changed claims/checks refusal, generated ownership and same-byte replacement refusal, no source/save/check effects',flush=True)
def main():
 spec=importlib.util.spec_from_file_location('fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 with tempfile.TemporaryDirectory(prefix='hickory-agent-proposal-') as folder:
  temporary=Path(folder);root=temporary/'workspace';root.mkdir();root=root.resolve()
  for path,text in {'project.md':'<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:project sources="construction.md"/></h:doc>\n','construction.md':'<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:file path="generated.py">generated = 1\n</h:file></h:doc>\n','generated.py':'generated = 1\n','note.py':'value = 1\n'}.items():(root/path).write_text(text)
  process=fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(root),'window','http://127.0.0.1:1'],env={**os.environ,'HICKORY_STATE_DIR':str(temporary/'state')})
  try:suite(fixture.origin(process),root)
  finally:fixture.stop(process)
if __name__=='__main__':main()
