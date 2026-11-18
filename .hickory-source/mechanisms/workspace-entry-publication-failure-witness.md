<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Prove a live foreign room cannot reserve an unwritten entry Move

For maintainers verifying the actual local HTTP publisher and Yjs room transport, independently of physical keyboard or accessibility acceptance.

## Retain a foreign room without scheduling source persistence

An owned temporary destination is first opened through the real collaboration socket. The peer sends only sync-step1 and verifies the returned source bytes. It ignores the server's step1: even an empty client step2 would count as a room mutation and introduce debounce persistence. The peer therefore sends no source updates, and no sleep races against the 750ms writer.

The fixture removes its temporary destination before acquiring the Move. The room remains genuinely attached to that now-missing path. The first destination publisher must refuse before its persist callback. The response must bind the original request and entry proof, report no reservation consumed and zero source effects, and expose no saved adoption. Original saved bytes and the retained dirty plan remain exact. Replaying that request returns the same refusal evidence; it never invokes another writer.

A genuine owned engine restart then retires the room. One fresh reviewed Move must publish the saved baseline while retaining the dirty editor plan independently. This is actual transport and engine behavior; it does not claim rendered host controls, physical input, accessibility or interrupted repair acceptance.

<ui:copy id="workspace-entry-failure-http-driver">
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
</ui:copy>

## Keep the room peer read-only

The peer reports its exact synchronized text and zero writes sent. It keeps the socket alive until the driver explicitly closes it, so an absent filesystem source cannot silently retire the foreign room before publication admission.

<ui:copy id="workspace-entry-failure-room-peer">
import * as Y from '../apps/web/node_modules/yjs/dist/yjs.mjs';
import * as encoding from '../apps/web/node_modules/lib0/encoding.js';
import * as decoding from '../apps/web/node_modules/lib0/decoding.js';
import * as sync from '../apps/web/node_modules/y-protocols/sync.js';
import WebSocket from '../apps/web/node_modules/ws/wrapper.mjs';
const [origin, identifier, expected] = process.argv.slice(2);
const doc = new Y.Doc();
const socket = new WebSocket(origin.replace(/^http/, 'ws') + '/api/ws?doc=' + encodeURIComponent('doc:' + identifier));
const deadline = setTimeout(() => { console.error('Owned room did not synchronize'); process.exit(1); }, 15000);
let ready = false;
socket.on('open', () => {
  const encoder = encoding.createEncoder();
  encoding.writeVarUint(encoder, 0);
  encoding.writeVarUint(encoder, 0);
  sync.writeSyncStep1(encoder, doc);
  socket.send(encoding.toUint8Array(encoder));
});
socket.on('message', (bytes) => {
  const frame = new Uint8Array(bytes);
  if (frame[0] !== 0) return;
  const decoder = decoding.createDecoder(frame.subarray(1));
  if (decoding.readVarUint(decoder) !== 0) return;
  const type = decoding.readVarUint(decoder);
  // Deliberately do not answer server step1: even a no-op client step2 would
  // mark the production room mutated and schedule its debounce persistence.
  if (type !== sync.messageYjsSyncStep2) return;
  Y.applyUpdate(doc, decoding.readVarUint8Array(decoder), socket);
  const source = doc.getText('source').toString();
  if (source !== expected) throw new Error('Foreign room bytes differ from fixture');
  if (!ready) {
    ready = true;
    clearTimeout(deadline);
    process.stdout.write(JSON.stringify({ready: true, source, writes_sent: 0}) + '\n');
  }
});
socket.on('error', (error) => { console.error(error); process.exitCode = 1; });
process.stdin.on('data', () => socket.close());
socket.on('close', () => { clearTimeout(deadline); doc.destroy(); process.exit(ready ? 0 : 1); });
</ui:copy>

<ui:file path="scripts/test-entry-publication-failures.py"><ui:paste select="#workspace-entry-failure-http-driver"/></ui:file>
<ui:file path="scripts/control-entry-room.mjs"><ui:paste select="#workspace-entry-failure-room-peer"/></ui:file>
</ui:doc>
