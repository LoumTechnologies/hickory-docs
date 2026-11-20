<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Resolve source-bound snippet context through the real engine

For maintainers changing completion context negotiation or provider caching.

The controlled child supplies an opaque item and a lazy snippet. The driver uses the actual workspace wire and shared engine, binds the exact captured source proof, and resolves selected text, filename, local clock and seed variables together with a live transform. A valid clipboard demand retries the cached resolved item rather than calling the child again. Denial remains explicit, unused defaults never demand clipboard content, foreign source revisions refuse, and original source bytes remain unchanged. This establishes protocol and ownership behavior; host popup and physical input acceptance remain separate.

<ui:copy id="snippet-context-protocol-provider">
"""Controlled lazy child; provider data remains opaque to the engine."""
import json,sys

events=sys.argv[1]
def record(value):
 with open(events,'a') as output:output.write(json.dumps(value)+'\n')
def send(value):
 raw=json.dumps(value).encode();sys.stdout.buffer.write(b'Content-Length: '+str(len(raw)).encode()+b'\r\n\r\n'+raw);sys.stdout.buffer.flush()
issued={}
while True:
 headers={}
 while True:
  line=sys.stdin.buffer.readline()
  if not line:sys.exit()
  if line in (b'\r\n',b'\n'):break
  key,value=line.decode().split(':',1);headers[key.lower()]=value.strip()
 message=json.loads(sys.stdin.buffer.read(int(headers['content-length'])));identity=message.get('id');method=message.get('method');params=message.get('params',{})
 if identity is None:continue
 if method=='initialize':result={'capabilities':{'textDocumentSync':1,'documentSymbolProvider':True,'completionProvider':{'resolveProvider':True,'triggerCharacters':['.']}}}
 elif method=='textDocument/documentSymbol':result=[]
 elif method=='textDocument/completion':
  uri=params['textDocument']['uri'];item={'label':'context completion','filterText':'old','data':{'opaque':{'uri':uri,'unicode':'é😀','range':{'start':{'line':91,'character':2},'end':{'line':91,'character':4}}}}};issued[uri]=item;record({'query':uri});result=[item]
 elif method=='completionItem/resolve':
  uri=params.get('data',{}).get('opaque',{}).get('uri');assert params==issued.get(uri),'Original opaque provider item was changed';record({'resolve':uri})
  syntax='${TM_SELECTED_TEXT:${CLIPBOARD}}$0' if uri.endswith('/unused.py') else '${CLIPBOARD:fallback}$0' if uri.endswith('/denied.py') else '${TM_FILENAME}|${CURRENT_YEAR}|${RANDOM_HEX}|${1:${TM_SELECTED_TEXT}}|$1|${1/(.*)/${1:/upcase}/}|${CLIPBOARD:fallback}$0'
  if uri.endswith('/browser.py'):syntax=syntax.rsplit('|${CLIPBOARD:fallback}',1)[0]+'$0'
  if uri.endswith('/grouped.py'):syntax='${CURSOR_NUMBER}|${TM_SELECTED_TEXT}|${CLIPBOARD:fallback}$0'
  result={**params,'insertTextFormat':2,'textEdit':{'range':{'start':{'line':0,'character':0},'end':{'line':0,'character':3}},'newText':syntax}}
 elif method=='shutdown':result=None
 else:result=None
 send({'jsonrpc':'2.0','id':identity,'result':result})
</ui:copy>
<ui:file path="scripts/snippet-context-controlled-provider.py"><ui:paste select="#snippet-context-protocol-provider"/></ui:file>

<ui:copy id="snippet-context-protocol-driver">
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {pathToFileURL} from 'node:url';
import {join} from 'node:path';
const url=new URL('/api/ws?doc=workspace',process.argv[2]);url.protocol='ws:';
const socket=new WebSocket(url);socket.binaryType='arraybuffer';let sequence=0;const pending=new Map();
function send(body){const data=new TextEncoder().encode(JSON.stringify({jsonrpc:'2.0',...body}));const frame=new Uint8Array(data.length+1);frame[0]=2;frame.set(data,1);socket.send(frame);}
function request(method,params){return new Promise((resolve,reject)=>{const id=++sequence,timer=setTimeout(()=>{pending.delete(id);reject(Error(method+' timed out'));},20000);pending.set(id,{resolve,reject,timer,method});send({id,method,params});});}
socket.addEventListener('message',event=>{if(!(event.data instanceof ArrayBuffer))return;const raw=new Uint8Array(event.data);if(raw[0]!==2)return;const body=JSON.parse(new TextDecoder().decode(raw.subarray(1))),held=pending.get(body.id);if(!held)return;pending.delete(body.id);clearTimeout(held.timer);body.error?held.reject(Error(held.method+": "+body.error.message)):held.resolve(body.result);});
const source='old\r\nsecond\n',snapshot={clock:{unix_millis:1709228107000,utc_offset_minutes:330},random_hex:'000102030405060708090a0b0c0d0e0f',clipboard:null};
function bind(item,clipboard=null,carets){return {...item,data:{...item.data,hickory_context:{version:1,expected:item.data.hickory_context,selection:{from:0,to:3},...snapshot,clipboard,...(carets?{carets}:{})}}};}
function first(result){return Array.isArray(result)?result[0]:result?.items?.[0];}
async function completeCurrent(uri){const deadline=Date.now()+20000,expectedHash='sha256:'+createHash('sha256').update(source).digest('hex');let heldRevision;
 while(Date.now()<deadline){
  const prepared=await request('hick/captureDocument',{textDocument:{uri,languageId:'python',version:1,text:source}});
  if(prepared.processed&&prepared.current){assert.equal(prepared.source_files_modified,false);assert.equal(prepared.source_hash,expectedHash);assert.match(prepared.source_revision,/^[a-f0-9]{64}$/);if(heldRevision!==undefined)assert.equal(prepared.source_revision,heldRevision,'Startup query retry must retain the same document lifetime');else heldRevision=prepared.source_revision;
   try{const item=first(await request('textDocument/completion',{textDocument:{uri},position:{line:0,character:3},context:{triggerKind:1}}));if(item){assert.deepEqual(item.data.hickory_context,{source_hash:expectedHash,source_revision:heldRevision});return item;}}catch(error){if(!/source.*changed|prepar|closed|mapping/i.test(String(error)))throw error;}
  }
  await new Promise(resolve=>setTimeout(resolve,25));
 }throw Error('Current completion provider did not become ready');}

try{
 await new Promise((resolve,reject)=>{socket.addEventListener('open',resolve,{once:true});socket.addEventListener('error',reject,{once:true});});
 for(const name of['variables.py','unused.py','denied.py']){
  const uri='hick:///'+name;send({method:'textDocument/didOpen',params:{textDocument:{uri,languageId:'python',version:1,text:source}}});await request('textDocument/documentSymbol',{textDocument:{uri}});
  const original=await completeCurrent(uri);assert.match(original.data.hickory_context.source_hash,/^sha256:[a-f0-9]{64}$/);const untouched=JSON.stringify(original);
  let resolved=await request('completionItem/resolve',bind(original));assert.equal(JSON.stringify(original),untouched);
  if(name==='unused.py'){assert.equal(resolved.data.hickory_context_request,undefined);assert.equal(resolved.data.hickory_snippet.text,'old');}
  else{assert.deepEqual(resolved.data.hickory_context_request,{version:1,variables:['CLIPBOARD']});resolved=await request('completionItem/resolve',bind(resolved,name==='denied.py'?{available:false,text:null}:{available:true,text:'λ😀'}));
   assert.equal(resolved.data.hickory_context_request,undefined);assert.equal(resolved.data.hickory_snippet.text,name==='denied.py'?'fallback':'variables.py|2024|000102|old|old|OLD|λ😀');
   if(name==='denied.py')assert.deepEqual(resolved.data.hickory_context_unavailable,['CLIPBOARD']);else{assert.equal(resolved.data.hickory_snippet.version,2);assert.equal(resolved.data.hickory_snippet.fields.filter(field=>field.transform).length,1);}
  }
  const events=(await readFile(process.argv[3],'utf8')).split('\n').filter(Boolean).map(JSON.parse);assert.equal(events.filter(event=>event.resolve===pathToFileURL(join(process.argv[4],name)).href).length,1,'Clipboard retry must reuse resolved child data');
  const forged=bind(original);forged.data.hickory_context.expected={...forged.data.hickory_context.expected,source_revision:'b'.repeat(64)};await assert.rejects(request('completionItem/resolve',forged),/source|stamp|changed/i);
 }
 const uri='hick:///grouped.py',carets={index:1,count:2},denied={available:false,text:null};
 const original=await completeCurrent(uri);
 const foreignPosition=bind(original,null,carets);
 foreignPosition.data.hickory_context.selection={from:5,to:11};
 await assert.rejects(request('completionItem/resolve',foreignPosition),/captured caret/);
 const pending=await request('completionItem/resolve',bind(original,null,carets));
 assert.deepEqual(pending.data.hickory_context_request,{version:1,variables:['CLIPBOARD']});
 assert.notEqual(pending.data.hickory_completion,original.data.hickory_completion);
 await assert.rejects(request('completionItem/resolve',bind(pending,denied,{index:0,count:2})),/captured context/);
 const resolved=await request('completionItem/resolve',bind(pending,denied,carets));
 assert.equal(resolved.data.hickory_snippet.text,'2|old|fallback');
 assert.equal(resolved.data.hickory_context_request,undefined);
 const groupedEvents=(await readFile(process.argv[3],'utf8')).split('\n').filter(Boolean).map(JSON.parse);
 assert.equal(groupedEvents.filter(event=>event.resolve===pathToFileURL(join(process.argv[4],'grouped.py')).href).length,1);
 console.log('PASS: source-bound variables, demand-only clipboard retry, grouped caret and retry identity, cached child resolve and foreign proof refusal');
}finally{for(const held of pending.values()){clearTimeout(held.timer);held.reject(Error('Closed'));}pending.clear();socket.close();}
</ui:copy>
<ui:file path="scripts/snippet-context-protocol-driver.mjs"><ui:paste select="#snippet-context-protocol-driver"/></ui:file>

<ui:copy id="snippet-context-protocol-runner">
"""Run source-bound context resolution through the actual shared engine."""
import importlib.util,json,os,subprocess,sys,tempfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('simple_fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
def main():
 with tempfile.TemporaryDirectory(prefix='snippet-context-',dir=ROOT/'target') as temporary:
  folder=Path(temporary);root=folder/'workspace';root.mkdir();events=folder/'events.jsonl';source='old\r\nsecond\n';paths=['variables.py','unused.py','denied.py','grouped.py']
  for path in paths:(root/path).write_bytes(source.encode())
  provider=ROOT/'scripts/snippet-context-controlled-provider.py';driver=ROOT/'scripts/snippet-context-protocol-driver.mjs'
  (root/'.hick-lsp.json').write_text(json.dumps({'servers':{'python':{'command':[sys.executable,str(provider),str(events)]}}}))
  engine=fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/debug/examples/engine_client')),str(root),'window','http://127.0.0.1:1'],env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state')})
  try:
   subprocess.run(['node',str(driver),fixture.origin(engine),str(events),str(root)],check=True,timeout=90)
   for path in paths:assert (root/path).read_bytes()==source.encode(),'Context resolution wrote source bytes'
  finally:fixture.stop(engine)
if __name__=='__main__':main()
</ui:copy>
<ui:file path="scripts/test-snippet-context-protocol.py"><ui:paste select="#snippet-context-protocol-runner"/></ui:file>

## Drive the retained browser popup and linked source history

The browser witness opens the existing popup both at an empty prefix and over an authored selection. Its template mode separately opens the existing menu, previews the same captured path/time/seed/selected-text declaration and applies that reviewed expansion through the real HTTP service. It accepts the real lazy provider result, types into the linked field, and checks derived Unicode casing, exact mixed line endings and coordinated Undo/Redo. It also requires retained editor identity and geometry and unchanged disk bytes. Clipboard demand/denial remains covered by the separate protocol and template witnesses; this fixture does not depend on ambient system clipboard content.

<ui:copy id="snippet-context-browser-runner">
"""Actual blank-prefix popup, variable resolution, linked transforms and source history."""
import importlib.util,json,os,re,socket,sys,tempfile,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('simple_fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
def main():
 with tempfile.TemporaryDirectory(prefix='snippet-context-browser-',dir=ROOT/'target') as temporary:
  folder=Path(temporary);root=folder/'workspace';root.mkdir();template='--template' in sys.argv;selected='--selection' in sys.argv or template;path='browser.py';source='old\r\nsecond\n';(root/path).write_bytes(source.encode());events=folder/'events.jsonl'
  (root/'.hick-lsp.json').write_text(json.dumps({'servers':{'python':{'command':[sys.executable,str(ROOT/'scripts/snippet-context-controlled-provider.py'),str(events)]}}}))
  if template:
   (root/'.hickory').mkdir();(root/'.hickory/templates.json').write_text(json.dumps({'version':1,'templates':[{'id':'context.derived','label':'Context derived','description':'Captured source variables and computed mirrors','language':'python','syntax':'${TM_FILENAME}|${CURRENT_YEAR}|${RANDOM_HEX}|${1:${TM_SELECTED_TEXT}}|$1|${1/(.*)/${1:/upcase}/}$0'}]}))
  processes=[];driver=None
  try:
   with socket.socket() as reservation:reservation.bind(('127.0.0.1',0));port=reservation.getsockname()[1]
   env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state')};ui=f'http://127.0.0.1:{port}'
   engine=fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(root),'window',ui],env=env);processes.append(engine);backend=fixture.origin(engine)
   vite=fixture.start(['node','node_modules/vite/bin/vite.js','--config','vite.simple.config.ts','--host','127.0.0.1','--port',str(port),'--strictPort'],cwd=ROOT/'apps/web',env={**env,'HICKORY_API_ORIGIN':backend});processes.append(vite)
   driver=fixture.BrowserDriver(fixture.origin(vite)+'/simple.html');driver.act('open',path=path);driver.act('remember-editor',path=path);size=driver.act('editor-size',path=path);deadline=time.monotonic()+20
   if template:
    driver.act('editor-position',path=path,anchor=0,head=3);catalog=driver.act('templates-open');assert 'Context derived' in catalog,catalog;preview=driver.act('templates-preview',label='Context derived');assert 'browser.py|' in preview,preview;assert driver.act('construction-source',path=path)==source;driver.act('templates-apply',path=path)
   else:
    while True:
     options=driver.act('completion-open',path=path,offset=0,**({'selection':{'anchor':0,'head':3}} if selected else {}))
     if any('context completion' in option for option in options):break
     assert time.monotonic()<deadline,options;time.sleep(.1)
    notice=driver.act('completion-choose',title='context completion');assert 'Completion applied' in notice,notice
   after=driver.act('construction-source',path=path);pattern=r'browser\.py\|\d{4}\|[a-f0-9]{6}\|'+('old\|old\|OLD' if selected else '\|\|')+r'\r\nsecond\n';assert re.fullmatch(pattern,after),after
   assert driver.act('selected-source-text',path=path)==('old' if selected else '');driver.act('type-source-selection',path=path,text='ßa');typed=after.replace('old|old|OLD\r\n' if selected else '|||\r\n','ßa|ßa|SSA\r\n' if selected else '|ßa|ßa|SSA\r\n');assert driver.act('construction-source',path=path)==typed
   driver.act('undo',path=path);assert driver.act('construction-source',path=path)==after;driver.act('undo',path=path);assert driver.act('construction-source',path=path)==source
   driver.act('redo',path=path);assert driver.act('construction-source',path=path)==after;driver.act('redo',path=path);assert driver.act('construction-source',path=path)==typed
   assert driver.act('editor-is-retained',path=path);assert driver.act('editor-size',path=path)==size;assert (root/path).read_bytes()==source.encode()
   if not template:
    observed=[json.loads(row) for row in events.read_text().splitlines()];assert len([row for row in observed if row.get('resolve')])==1,observed
   print('PASS: actual '+('template preview/Apply' if template else 'explicit completion')+', captured source variables and selection, computed Unicode mirrors, atomic UndoRedo, retained editor geometry and unchanged CRLF disk bytes')
  finally:
   if driver:driver.close()
   for process in reversed(processes):fixture.stop(process)
if __name__=='__main__':main()
</ui:copy>
<ui:file path="scripts/test-snippet-context-browser.py"><ui:paste select="#snippet-context-browser-runner"/></ui:file>
</ui:doc>
