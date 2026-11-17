#!/usr/bin/env python3
"""Owned-engine recovery checkpoints: restart, exact receipts and physical source refusal."""
import argparse,hashlib,importlib.util,json,os,signal,socket,subprocess,tempfile,time,urllib.error,urllib.parse,urllib.request,uuid
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def load(name,path):
 spec=importlib.util.spec_from_file_location(name,path);module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module
def wait(check,message):
 deadline=time.monotonic()+30
 while time.monotonic()<deadline:
  value=check()
  if value:return value
  time.sleep(.04)
 raise AssertionError(message)
def stopped(pid):
 try:os.kill(pid,0);return False
 except ProcessLookupError:return True
def main():
 signal.signal(signal.SIGTERM,lambda signum,_frame:exit_signal(signum))
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--engine',type=Path,default=ROOT/'target/source-edit-focus/debug/examples/engine_client');args=parser.parse_args()
 fixture=load('fixture',ROOT/'scripts/test-simple-browser.py');api=load('save_fixture',ROOT/'scripts/test-source-save-history.py')
 with tempfile.TemporaryDirectory(prefix='hickory-editor-recovery-') as folder:
  temporary=Path(folder);root=temporary/'workspace';root.mkdir();(root/'.git').mkdir();path='source λ%?#.txt';source='saved source 😀\n';(root/path).write_text(source);state=temporary/'state';processes=[];engines=[]
  environment={**os.environ,'HICKORY_STATE_DIR':str(state)}
  def start():
   process=fixture.start([str(args.engine.resolve()),str(root),'window','http://127.0.0.1:1'],env=environment);processes.append(process);origin=fixture.origin(process)
   manifest=json.loads((state/'engine/endpoint.json').read_text());pid=manifest['pid']
   command=subprocess.check_output(['ps','-p',str(pid),'-o','command='],text=True)
   assert str(args.engine.resolve()) in command and '__engine' in command and os.getpgid(pid)==pid
   engines.append((process,pid));return origin
  def crash():
   process,pid=engines[-1]
   if not stopped(pid):
    command=subprocess.check_output(['ps','-p',str(pid),'-o','command='],text=True);assert str(args.engine.resolve()) in command and '__engine' in command and os.getpgid(pid)==pid
    os.killpg(pid,signal.SIGTERM);wait(lambda:stopped(pid),'Owned engine did not exit')
   for proxy,owned_pid in engines:
    if owned_pid==pid:fixture.stop(proxy)
  def catalog(origin):
   def ready():
    try:return api.request(origin,'GET','/api/workspace/recovery')
    except urllib.error.HTTPError as error:
     if error.code!=409:raise
   return wait(ready,'Recovery root capability did not become ready')
  def buffer(identifier,address,text,baseline=None):return {'id':identifier,'address':address,'text':text,'baseline':baseline,'view':{'selection':{'anchor':0,'head':len(text.encode('utf-16-le'))//2},'scroll_x':0,'scroll_y':29}}
  try:
   origin=start();scope=catalog(origin);loaded=api.guard_file(origin,path);client=uuid.uuid4().hex;identifier=uuid.uuid4().hex
   buffers=[buffer(identifier,{'kind':'source','path':path},'unsaved λ 😀\n',{'text':source,'save_guard':loaded['save_guard']}),buffer(uuid.uuid4().hex,{'kind':'unnamed','name':'ordinary note'},'unassigned retained text'),buffer(uuid.uuid4().hex,{'kind':'generated','path':'out.py','owner':'construction.md'},'generated read-only text'),buffer(uuid.uuid4().hex,{'kind':'source','path':'café.py'},'first raw name'),buffer(uuid.uuid4().hex,{'kind':'source','path':'cafe\u0301.py'},'second raw name')]
   snapshot={'host':'browser','buffers':buffers,'layout':{'root':'main','groups':[{'id':'main','tabs':[{'kind':'source','buffer_id':row['id']} for row in buffers],'active_buffer':identifier}]}}
   body={'version':1,'workspace':scope['engine_workspace_id'],'window_slot':scope['window_slot'],'client_id':client,'request_id':uuid.uuid4().hex,'base_revision':0,'snapshot':snapshot};route='/api/workspace/recovery/checkpoint'
   first=api.request(origin,'POST',route,body);assert first['revision']==1 and first['source_effects'] is False and first['source_saved'] is False
   records=list(state.glob('workspaces/**/editor-recovery-v1/'+client+'.json'));assert len(records)==1;record=records[0];before=record.read_bytes();inode=record.stat().st_ino
   retry=api.request(origin,'POST',route,body);assert retry['revision']==1 and retry['replayed'] and record.read_bytes()==before and record.stat().st_ino==inode
   api.refused(lambda:api.request(origin,'POST',route,{**body,'snapshot':{**snapshot,'layout':{}}}));api.refused(lambda:api.request(origin,'POST',route,{**body,'workspace':'f'*32,'request_id':uuid.uuid4().hex}))
   other_origin=start();other_scope=catalog(other_origin);assert other_scope['window_slot']!=scope['window_slot'];assert scope['window_slot'] in other_scope['window_slots']
   previous_window=api.request(other_origin,'GET','/api/workspace/recovery?window_slot='+scope['window_slot']);assert any(row['client_id']==client for row in previous_window['sessions']);assert previous_window['current_window_slot']==other_scope['window_slot']
   api.refused(lambda:api.request(other_origin,'POST',route,body));assert record.read_bytes()==before
   restored_other=api.request(other_origin,'GET','/api/workspace/recovery/'+client+'?window_slot='+scope['window_slot']);assert json.loads(restored_other['material'])==snapshot and restored_other['window_slot']==scope['window_slot']
   second={**body,'request_id':uuid.uuid4().hex,'base_revision':1};second['snapshot']=json.loads(json.dumps(snapshot));second['snapshot']['buffers'][0]['text']='newer unsaved checkpoint';second['snapshot']['buffers'][0]['view']['selection']={'anchor':0,'head':3}
   api.request(origin,'POST',route,second);api.refused(lambda:api.request(origin,'POST',route,{**body,'request_id':uuid.uuid4().hex}))
   def read():
    value=api.request(origin,'GET','/api/workspace/recovery/'+client);assert hashlib.sha256(value['material'].encode()).hexdigest()==value['snapshot_hash'];assert json.loads(value['material'])==second['snapshot'];assert value['source_effects'] is False and value['source_saved'] is False and all(row['write_permission'] is False for row in value['authority']);return value
   value=read();assert value['current_workspace'] and value['authority'][0]['baseline_current'] and not value['authority'][2]['baseline_current'];runtime=scope['engine_workspace_id'];disk=record.read_bytes()
   crash();origin=start();scope=catalog(origin);assert scope['engine_workspace_id']!=runtime;assert read()['authority'][0]['baseline_current'] and record.read_bytes()==disk
   replacement=root/'replacement';replacement.write_text(source);os.replace(replacement,root/path);assert not read()['authority'][0]['baseline_current'];assert (root/path).read_text()==source
   crash();old=temporary/'previous-workspace';root.rename(old);root.mkdir();(root/'.git').mkdir();(root/path).write_text(source);origin=start();scope=catalog(origin);historical=read();assert not historical['current_workspace'] and all(not row['baseline_current'] for row in historical['authority']);assert record.read_bytes()==disk
   api.refused(lambda:api.request(origin,'POST',route,{**second,'workspace':scope['engine_workspace_id'],'request_id':uuid.uuid4().hex,'base_revision':2}));assert record.read_bytes()==disk
   damaged=record.parent/(uuid.uuid4().hex+'.json');damaged.write_text('{damaged');listing=catalog(origin);assert any(row['client_id']==client for row in listing['sessions']) and listing['notices'];assert (root/path).read_text()==source
   print('PASS owned recovery HTTP: exact private retry/no second inode, stale/reused/root request refusal, raw Unicode/unnamed/generated/layout retention, real engine restart, raw-material seal, original physical baseline classification, same-byte source/root recreation refusal, corrupt sibling visibility and unchanged source files',flush=True)
  finally:
   for process,pid in reversed(engines):
    if not stopped(pid):
     command=subprocess.check_output(['ps','-p',str(pid),'-o','command='],text=True)
     if str(args.engine.resolve()) in command and '__engine' in command and os.getpgid(pid)==pid:os.killpg(pid,signal.SIGTERM)
   for process in reversed(processes):fixture.stop(process)
def exit_signal(signum):raise SystemExit(128+signum)
if __name__=='__main__':main()
