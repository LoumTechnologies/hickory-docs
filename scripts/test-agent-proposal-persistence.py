#!/usr/bin/env python3
"""Owned real-engine historical proposal reading, restart refusal and storage-failure isolation."""
import hashlib,importlib.util,json,os,tempfile,time,urllib.error,urllib.request,uuid
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
FILES={'project.md':'\x3ch:doc xmlns:h="http://www.hickorydocs.com/1.0">\x3ch:project sources="construction.md"/>\x3c/h:doc>\n','construction.md':'\x3ch:doc xmlns:h="http://www.hickorydocs.com/1.0">\x3ch:file path="generated.py">generated = 1\n\x3c/h:file>\x3c/h:doc>\n','generated.py':'generated = 1\n','note.py':'value = 1\n'}
def call(origin,method,route,body=None):
 request=urllib.request.Request(origin+route,data=None if body is None else json.dumps(body).encode(),method=method,headers={'Content-Type':'application/json'})
 with urllib.request.urlopen(request,timeout=90) as response:return json.load(response)
def refused(action):
 try:action()
 except urllib.error.HTTPError as error:assert error.code in (400,403,404,409,422),(error.code,error.read());return
 raise AssertionError('Historical or tampered proposal was admitted')
def intention():
 return {'version':1,'request_id':uuid.uuid4().hex,'origin':'owned restart fixture','title':'Retain an exact historical intention','explanation':'The expected transition is an agent claim. Restarted readings carry no admission authority.','claims':['This claim was not verified by an executed check.'],'check_ids':[],'capture':{'context':'owned-restart','project_entry':'project.md','drafts':{},'ownership_sources':{},'unaddressed_sources':{},'saved_guards':{},'edit':{'changes':{'hick:///note.py':[{'range':{'start':{'line':0,'character':0},'end':{'line':1,'character':0}},'newText':'value = 41\n'}]}}},'reading':{'version':1,'entry':'project.md','sources':{},'missing_sources':[],'focus':'note.py'}}
def stop_engine(fixture,process,state):
 # Closing a window proxy does not itself end the shared engine process.
 fixture.stop(process);endpoint=Path(state)/'engine/endpoint.json';deadline=time.monotonic()+30
 while endpoint.exists() and time.monotonic()<deadline:time.sleep(.1)
 assert not endpoint.exists(),'Owned engine did not finish its lease shutdown; a true engine restart was not established'
def main():
 spec=importlib.util.spec_from_file_location('fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 with tempfile.TemporaryDirectory(prefix='hickory-proposal-history-') as temporary:
  folder=Path(temporary).resolve();root=folder/'workspace';root.mkdir();state=folder/'private-state'
  files=FILES
  for path,text in files.items():(root/path).write_text(text)
  def start():return fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(root),'window','http://127.0.0.1:1'],env={**os.environ,'HICKORY_STATE_DIR':str(state)})
  engine=start()
  try:
   origin=fixture.origin(engine);review=call(origin,'POST','/api/agent-proposals/prepare',intention());assert review['historical'] is False and review['durability']['retained'] is True,review.get('durability');assert hashlib.sha256(review['material'].encode()).hexdigest()==review['fingerprint']
   admission={'version':1,'token':review['token'],'fingerprint':review['fingerprint']};assert call(origin,'POST','/api/agent-proposals/validate',admission)['admitted'] is True
   report={'version':1,'token':review['token'],'fingerprint':review['fingerprint'],'workspace':review['workspace'],'namespace':str(root),'request_id':uuid.uuid4().hex,'host':'owned fixture host claim, not an observed draft','after_hashes':{'note.py':hashlib.sha256(b'value = 41\n').hexdigest()}}
   claimed=call(origin,'POST','/api/agent-proposals/report',report);assert claimed['evidence_authority']=='host report; not independently observed by the engine';assert claimed['saved'] is False and claimed['checks_executed'] is False
   assert call(origin,'GET','/api/agent-proposals/'+review['token'])['host_application']==claimed
  finally:stop_engine(fixture,engine,state)
  engine=start()
  try:
   origin=fixture.origin(engine);listing=call(origin,'GET','/api/agent-proposals');row=next(row for row in listing['reviews'] if row['token']==review['token']);assert row['historical'] is True and listing['workspace']!=review['workspace'];assert listing['retention_notice'] is None
   historical=call(origin,'GET','/api/agent-proposals/'+review['token']);assert historical['historical'] is True;assert historical['material']==review['material'];assert historical['fingerprint']==review['fingerprint'];assert historical['workspace']==review['workspace'] and historical['namespace']==review['namespace'];assert historical['reading_scope']=={'workspace':listing['workspace'],'namespace':listing['namespace']};assert historical['host_application']==claimed;assert historical['source_effects'] is False and historical['checks_executed'] is False
   refused(lambda:call(origin,'POST','/api/agent-proposals/validate',admission));refused(lambda:call(origin,'POST','/api/agent-proposals/report',report));refused(lambda:call(origin,'POST','/api/agent-proposals/report',{**report,'workspace':listing['workspace'],'request_id':uuid.uuid4().hex}))
   fresh=call(origin,'POST','/api/agent-proposals/prepare',intention());assert fresh['historical'] is False and fresh['token']!=review['token'];assert call(origin,'POST','/api/agent-proposals/validate',{'version':1,'token':fresh['token'],'fingerprint':fresh['fingerprint']})['admitted'] is True
  finally:stop_engine(fixture,engine,state)
  catalogues=list(state.rglob('readings.json'));assert len(catalogues)==1;catalogue=json.loads(catalogues[0].read_text());catalogue['payload']['records'][0]['material']+='forged';catalogues[0].write_text(json.dumps(catalogue))
  engine=start()
  try:
   origin=fixture.origin(engine);listing=call(origin,'GET','/api/agent-proposals');assert listing['reviews']==[] and 'unavailable' in listing['retention_notice'].lower();refused(lambda:call(origin,'GET','/api/agent-proposals/'+review['token']));refused(lambda:call(origin,'POST','/api/agent-proposals/validate',admission))
   live=call(origin,'POST','/api/agent-proposals/prepare',intention());assert live['historical'] is False and live['durability']['retained'] is False;assert 'failed' in live['durability']['notice'].lower();assert call(origin,'GET','/api/agent-proposals/'+live['token'])==live;assert call(origin,'POST','/api/agent-proposals/validate',{'version':1,'token':live['token'],'fingerprint':live['fingerprint']})['admitted'] is True
  finally:stop_engine(fixture,engine,state)
  for path,text in files.items():assert (root/path).read_text()==text,path
  assert not (root/'.hick-cache').exists()
  print('PASS: real-engine live preparation/report, restart historical list/read with exact original material and current reading scope, no archived admission/report or automatic source effect, fresh live proposal independently admitted, tampered archive refusal with truthful live-only durability failure, original source bytes retained. Host reports remain unverified claims; physical/accessibility input not exercised.')
if __name__=='__main__':main()
