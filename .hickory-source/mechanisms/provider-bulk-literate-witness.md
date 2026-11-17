<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Read complete provider fixes from literate output authority

For maintainers following a document-wide provider action from prose through compiler-owned virtual outputs and back to exact authored contributors.

A file-wide fix request is not a caret query. The owned engine fixture opens Markdown with prose at line zero and several outputs, including repeated source copies and a file crossing contributors. The controlled child proves protocol and projection boundaries; it does not pretend to certify Python semantics. Each action remains an explicit output choice with a stable title across query/resolve. Core-issued coordinate source hashes must match every captured fixture input; caller- or child-supplied metadata cannot replace that evidence. Unmappable edits refuse as complete actions, never partial source changes.

## Supply an opaque, source-bound child action

The provider records complete child ranges and action context, and requires its exact original payload on lazy resolution. A held reply lets the real client change a copy-only contributor during the await. A deliberate oversized list tests whole-list refusal. Two dirty contributors and a replacement root inventory prove the actual compiler graph changes coherently; a literal `%2F#` source name must retain its exact encoded identity. It writes only its owned event log; it never executes commands or saves authored text.

<ui:copy id="bulk-literate-controlled-provider">
#!/usr/bin/env python3
"""Controlled complete-file provider: protocol timing/projection, not language semantics."""
import json,os,sys,threading,time
EVENTS,GATE=sys.argv[1:3]
LOCK=threading.Lock();documents={};issued={};sequence=0
def send(value):
 raw=json.dumps(value,separators=(',',':')).encode()
 with LOCK:sys.stdout.buffer.write(f'Content-Length: {len(raw)}\r\n\r\n'.encode()+raw);sys.stdout.buffer.flush()
def event(value):
 descriptor=os.open(EVENTS,os.O_WRONLY|os.O_CREAT|os.O_APPEND,0o600)
 try:os.write(descriptor,(json.dumps(value)+'\n').encode())
 finally:os.close(descriptor)
def edits(uri):
 text=documents[uri];rows=[]
 for line,content in enumerate(text.split('\n')):
  offset=content.find('value')
  if offset>=0:rows.append({'range':{'start':{'line':line,'character':len(content[:offset].encode('utf-16-le'))//2},'end':{'line':line,'character':len(content[:offset+5].encode('utf-16-le'))//2}},'newText':'fixed'})
 if uri.endswith('/cross.py') and len(rows)>1:return [{'range':{'start':rows[0]['range']['start'],'end':rows[-1]['range']['end']},'newText':'must not erase contributors'}]
 return rows
while True:
 headers={}
 while True:
  line=sys.stdin.buffer.readline()
  if not line:sys.exit(0)
  if line in (b'\n',b'\r\n'):break
  name,value=line.decode().split(':',1);headers[name.lower()]=value.strip()
 message=json.loads(sys.stdin.buffer.read(int(headers['content-length'])));method=message.get('method');params=message.get('params',{});identity=message.get('id')
 if method=='textDocument/didOpen':documents[params['textDocument']['uri']]=params['textDocument']['text'];continue
 if method=='textDocument/didChange':documents[params['textDocument']['uri']]=params['contentChanges'][-1]['text'];continue
 if method=='textDocument/didClose':documents.pop(params['textDocument']['uri'],None);continue
 if method=='exit':sys.exit(0)
 if identity is None:continue
 if method=='initialize':result={'capabilities':{'textDocumentSync':1,'documentSymbolProvider':True,'codeActionProvider':{'resolveProvider':True,'codeActionKinds':['source.fixAll.controlled','quickfix']}}}
 elif method=='textDocument/documentSymbol':result=[]
 elif method=='textDocument/codeAction':
  uri=params['textDocument']['uri'];text=documents[uri];expected={'start':{'line':0,'character':0},'end':{'line':text.count('\n'),'character':len(text.rsplit('\n',1)[-1].encode('utf-16-le'))//2}}
  context=params.get('context',{});event({'query':uri,'range':params['range'],'context':context,'whole':params['range']==expected,'source':text})
  if context.get('only')!=['source.fixAll'] or params['range']!=expected:result=[{'title':'Contextual only','kind':'quickfix'}]
  else:
   result=[]
   for index in range(513 if os.path.exists(GATE+'.overbudget') else 1):
    sequence+=1;action={'title':'Fix complete file'+(f' {index}' if index else ''),'kind':'source.fixAll.controlled','data':{'sequence':sequence,'uri':uri,'slow':context.get('triggerKind')==2,'vendor':{'range':{'line':97,'character':3},'literal':'é😀?#%'}}};issued[sequence]=action;result.append(action)
 elif method=='codeAction/resolve':
  original=issued.get(params.get('data',{}).get('sequence'))
  if params!=original:
   event({'error':'original opaque provider payload was changed'});send({'jsonrpc':'2.0','id':identity,'error':{'code':-32602,'message':'Original action payload required'}});continue
  uri=params['data']['uri'];result={**params,'edit':{'changes':{uri:edits(uri)}}};event({'resolved_started':uri,'slow':params['data']['slow'],'original':True})
  if params['data']['slow']:
   def release(identity=identity,result=result):
    deadline=time.monotonic()+15
    while not os.path.exists(GATE) and time.monotonic()<deadline:time.sleep(.01)
    send({'jsonrpc':'2.0','id':identity,'result':result} if os.path.exists(GATE) else {'jsonrpc':'2.0','id':identity,'error':{'code':-32603,'message':'Owned gate timed out'}})
   threading.Thread(target=release,daemon=True).start();continue
 else:result=None
 send({'jsonrpc':'2.0','id':identity,'result':result})
</ui:copy>

## Exercise the real WebSocket language path

The client verifies explicit multi-output choices, actual contributor membership rather than all loaded sources, deduplicated repeated origins, and complete cross-fragment refusal. It also checks copy-only edit/Undo and close/reopen token retirement, a dirty contributor whose captured project is stale, and a held resolution that becomes obsolete. These checks operate through the engine protocol without editing implementation objects or invoking an in-process server harness.

<ui:copy id="bulk-literate-protocol-driver">
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFile,writeFile,unlink} from 'node:fs/promises';
const [origin,events,gate,fixturePath]=process.argv.slice(2),fixture=JSON.parse(await readFile(fixturePath,'utf8')),currentSources={...fixture};
const url=new URL('/api/ws?doc=workspace',origin);url.protocol=url.protocol==='https:'?'wss:':'ws:';
const socket=new WebSocket(url);socket.binaryType='arraybuffer';let sequence=0;const pending=new Map();let readyResolve;const ready=new Promise(resolve=>{readyResolve=resolve;});
function send(body){const raw=new TextEncoder().encode(JSON.stringify({jsonrpc:'2.0',...body}));const bytes=new Uint8Array(raw.length+1);bytes[0]=2;bytes.set(raw,1);socket.send(bytes);}
function request(method,params){return new Promise((resolve,reject)=>{const id=++sequence,timer=setTimeout(()=>{pending.delete(id);reject(new Error('Owned LSP timeout: '+method));},20000);pending.set(id,{resolve,reject,timer});send({id,method,params});});}
socket.addEventListener('message',event=>{if(!(event.data instanceof ArrayBuffer))return;const bytes=new Uint8Array(event.data);if(bytes[0]!==2)return;const body=JSON.parse(new TextDecoder().decode(bytes.subarray(1)));if(body.method==='hick/serverCapabilities'){readyResolve();return;}if(body.method&&body.id!==undefined){send({id:body.id,error:{code:-32601,message:'This controlled client never executes provider commands'}});return;}const call=pending.get(body.id);if(!call)return;pending.delete(body.id);clearTimeout(call.timer);if(body.error)call.reject(new Error(body.error.message));else call.resolve(body.result);});
const opened=new Promise((resolve,reject)=>{socket.addEventListener('open',resolve,{once:true});socket.addEventListener('error',reject,{once:true});});
const uri=path=>'hick:///'+path.split('/').map(encodeURIComponent).join('/');
const prepare=async(path,text,version)=>{const result=await request('hick/captureDocument',{textDocument:{uri:uri(path),languageId:'markdown',version,text}});assert.equal(result.parsed,true,JSON.stringify(result));assert.equal(result.current,true);assert.equal(result.processed,true);assert.equal(result.source_hash,'sha256:'+createHash('sha256').update(text).digest('hex'));assert.match(result.source_revision,/^[a-f0-9]{64}$/);assert.equal(result.source_files_modified,false);};
const open=async(path,text=fixture[path])=>{currentSources[path]=text;send({method:'textDocument/didOpen',params:{textDocument:{uri:uri(path),languageId:'markdown',version:1,text}}});await prepare(path,text,1);};
const change=async(path,text,version)=>{currentSources[path]=text;send({method:'textDocument/didChange',params:{textDocument:{uri:uri(path),version},contentChanges:[{text}]}});await prepare(path,text,version);};
const query=async(path,triggerKind=1)=>await request('textDocument/codeAction',{textDocument:{uri:uri(path)},range:{start:{line:0,character:0},end:{line:fixture[path].split('\n').length-1,character:fixture[path].split('\n').at(-1).length}},context:{diagnostics:[],only:['source.fixAll'],triggerKind}})??[];
const one=(actions,path)=>{const found=actions.filter(action=>action.title.endsWith(' · '+path));assert.equal(found.length,1,JSON.stringify(actions));return found[0];};
const verifySources=action=>{const rows=action.data.hickory_sources;assert.ok(Array.isArray(rows)&&rows.length);assert.equal(new Set(rows.map(row=>row.uri)).size,rows.length);assert.deepEqual(rows.map(row=>row.uri),[...rows.map(row=>row.uri)].sort());for(const row of rows){assert.deepEqual(Object.keys(row).sort(),['source_hash','uri']);const path=decodeURIComponent(new URL(row.uri).pathname.slice(1));assert.ok(Object.hasOwn(fixture,path),row.uri);assert.equal(row.source_hash,'sha256:'+createHash('sha256').update(currentSources[path]).digest('hex'));}};
const resolved=async action=>{const value=await request('codeAction/resolve',action);verifySources(value);return value;};
const textEdits=action=>action.edit.documentChanges.flatMap(change=>change.edits.map(edit=>({uri:change.textDocument.uri,...edit})));
try{
 await opened;await open('project.md');await Promise.race([ready,new Promise((_,reject)=>setTimeout(()=>reject(new Error('No server capabilities')),20000).unref())]);
 const root=await query('project.md');assert.equal(root.length,4,JSON.stringify(root));assert.equal(new Set(root.map(action=>action.title)).size,4);for(const action of root){assert.equal(action.kind,'source.fixAll.controlled');assert.deepEqual(Object.keys(action.data).sort(),['hickory_action','hickory_sources']);verifySources(action);assert.equal(action.edit,undefined);}
 const original=one(root,'out/a.py');const forged={...original,title:'Caller substitution',command:{title:'Injected',command:'must.not.execute'},data:{...original.data,vendor:'caller',hickory_sources:[{uri:'hick:///foreign.md',source_hash:'sha256:'+'0'.repeat(64)}]}};const projected=await resolved(forged);assert.equal(projected.title,original.title);assert.equal(projected.command,undefined);assert.equal(textEdits(projected).length,1);assert.equal(textEdits(projected)[0].uri,uri('a.md'));assert.equal(textEdits(projected)[0].newText,'fixed');
 const repeated=await resolved(one(root,'out/repeated.py'));assert.equal(textEdits(repeated).length,1,'duplicate emitted copies must become one exact authored edit');assert.equal(textEdits(repeated)[0].uri,uri('a.md'));
 const cross=await resolved(one(root,'out/cross.py'));assert.equal(cross.edit,undefined);assert.match(cross.disabled?.reason??'',/source fragments|source origin|cannot map/i);
 const again=await query('project.md');assert.deepEqual(again.map(action=>[action.title,action.kind]),root.map(action=>[action.title,action.kind]));assert.notEqual(again[0].data.hickory_action,root[0].data.hickory_action);assert.equal((await resolved(original)).title,original.title);
 await open('b.md');const b=await query('b.md');assert.equal(b.length,2,JSON.stringify(b));one(b,'out/b.py');one(b,'out/cross.py');assert.deepEqual(await query('unused.md'),[],'loaded-but-unemitted sources are not contributors');
 await open('a.md');const a=await query('a.md');assert.equal(a.length,3);const stale=one(a,'out/a.py');await change('a.md',fixture['a.md']+'\n# intervening draft\n',2);await change('a.md',fixture['a.md'],3);await assert.rejects(resolved(stale),/source changed|contributor changed|expired|ownership changed/i);
 const reopenedToken=one(await query('a.md'),'out/a.py');send({method:'textDocument/didClose',params:{textDocument:{uri:uri('a.md')}}});await request('textDocument/documentSymbol',{textDocument:{uri:uri('project.md')}});await open('a.md');await assert.rejects(resolved(reopenedToken),/source changed|contributor changed|expired|ownership changed/i,'closing and reopening an equal-byte contributor must not revive a token');
 await change('a.md',fixture['a.md'].replace('value','later_value'),4);const dirtyRoot=await query('project.md');dirtyRoot.forEach(verifySources);const dirtyFix=await resolved(one(dirtyRoot,'out/a.py'));assert.equal(textEdits(dirtyFix)[0].uri,uri('a.md'));assert.equal(textEdits(dirtyFix)[0].newText,'fixed');assert.ok(textEdits(dirtyFix)[0].range.start.character>6,'Dirty contributor positions must reflect the exact current draft');await change('a.md',fixture['a.md'],5);
 const slow=one(await query('a.md',2),'out/a.py'),held=resolved(slow).then(value=>({value}),error=>({error}));let started=false;for(let n=0;n<300;n++){const rows=(await readFile(events,'utf8').catch(()=>'' )).split('\n').filter(Boolean).map(JSON.parse);if(rows.some(row=>row.resolved_started&&row.slow)){started=true;break;}await new Promise(resolve=>setTimeout(resolve,10));}assert.ok(started,'The owned provider resolve must be held before the contributor changes');await change('a.md',fixture['a.md']+'\n# during resolve\n',6);await writeFile(gate,'release');const answer=await held;assert.match(answer.error?.message??'',/source changed|contributor changed|expired|ownership changed/i);await change('a.md',fixture['a.md'],7);
 await writeFile(gate+'.overbudget','overbudget');await assert.rejects(query('project.md'),/review budget|too many|partial list/i);await unlink(gate+'.overbudget');
 await open('c.md',fixture['c.md'].replace('30','31'));await open('literal%2F#.md',fixture['literal%2F#.md'].replace('41','42'));
 const inventory=fixture['project.md'].replace('a.md b.md unused.md','b.md c.md literal%2F#.md unused.md').split('\n').filter(line=>!line.includes('path="out/a.py"')&&!line.includes('path="out/repeated.py"')&&!line.includes('path="out/cross.py"')).join('\n').replace('\x3c/hick:doc>','\x3chick:file path="out/new.py">\x3chick:paste select="#third"/>\x3c/hick:file>\n\x3chick:file path="out/literal.py">\x3chick:paste select="#literal"/>\x3c/hick:file>\n\x3c/hick:doc>');
 await change('project.md',inventory,2);const changedInventory=await query('project.md');assert.equal(changedInventory.length,3,JSON.stringify(changedInventory));changedInventory.forEach(verifySources);one(changedInventory,'out/b.py');one(changedInventory,'out/new.py');const literal=await resolved(one(changedInventory,'out/literal.py'));assert.equal(textEdits(literal)[0].uri,uri('literal%2F#.md'));assert.equal(new URL(textEdits(literal)[0].uri).hash,'');assert.equal((await query('literal%2F#.md')).length,1);assert.deepEqual(await query('a.md'),[],'Removed source outputs must not survive the replacement inventory');
 await change('project.md',fixture['project.md'],3);await change('c.md',fixture['c.md'],2);await change('literal%2F#.md',fixture['literal%2F#.md'],2);
 const records=(await readFile(events,'utf8')).split('\n').filter(Boolean).map(JSON.parse);assert.ok(records.some(row=>row.query?.endsWith('/out/new.py')&&row.source?.includes('value = 31')));assert.ok(records.some(row=>row.query?.endsWith('/out/literal.py')&&row.source?.includes('value = 42')));assert.ok(records.filter(row=>row.query).every(row=>row.whole&&JSON.stringify(row.context.only)==='["source.fixAll"]'));assert.ok(records.some(row=>row.context?.triggerKind===2));assert.ok(!records.some(row=>row.error));
 console.log('PASS: prose-start complete provider fan-out, stable explicit output choices, contributor/repeated-origin projection, standalone-duplicate/unrelated-output exclusion, opaque resolution, unmappable whole-action refusal, edit/Undo and close/reopen retirement, coherent dirty contributor fixes, dirty inventory replacement, literal percent/hash source projection, held resolve retirement and complete-list budget refusal. No source writes or commands.');
}finally{for(const call of pending.values()){clearTimeout(call.timer);call.reject(new Error('Owned client closed'));}pending.clear();socket.close();}
</ui:copy>

## Own and stop every process and saved fixture

The runner creates one disposable project and configures only its own controlled Python provider. The project source, contributors and unrelated loaded document are checked byte-for-byte after the client exits. The engine and child lifecycle belong to this runner, and no external publication occurs. Native host pointer, IME and accessibility acceptance remain unverified by this protocol reading.

<ui:copy id="bulk-literate-protocol-runner">
#!/usr/bin/env python3
"""Owned engine/child-provider acceptance for document-wide literate diagnostic fixes."""
import importlib.util,json,os,signal,subprocess,sys,tempfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
# Escaped tag openings keep executable fixture syntax literal in its owning construction.
SOURCE={
 'project.md':"\x3chick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\n# Readable prose precedes every construction\n\x3chick:project sources=\"a.md b.md unused.md\"/>\n\x3chick:file path=\"out/a.py\">\x3chick:paste select=\"#first\"/>\x3c/hick:file>\n\x3chick:file path=\"out/repeated.py\">\x3chick:paste select=\"#first\"/>\x3chick:paste select=\"#first\"/>\x3c/hick:file>\n\x3chick:file path=\"out/cross.py\">\x3chick:paste select=\"#first\"/>\x3chick:paste select=\"#second\"/>\x3c/hick:file>\n\x3c/hick:doc>\n",
 'a.md':"\x3chick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\n# Copy-only Unicode contributor\n\x3chick:copy id=\"first\">value = '😀'\n\x3c/hick:copy>\n\x3c/hick:doc>\n",
 'b.md':"\x3chick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\n# Another contributor with a standalone duplicate\n\x3chick:copy id=\"second\">value = 2\n\x3c/hick:copy>\n\x3chick:file path=\"out/b.py\">\x3chick:paste select=\"#second\"/>\x3c/hick:file>\n\x3c/hick:doc>\n",
 'unused.md':"\x3chick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\n# Loaded, but no output bytes originate here\n\x3c/hick:doc>\n",
 'c.md':"\x3chick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\n\x3chick:copy id=\"third\">value = 30\n\x3c/hick:copy>\n\x3c/hick:doc>\n",
 'literal%2F#.md':"\x3chick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\n\x3chick:copy id=\"literal\">value = 41\n\x3c/hick:copy>\n\x3c/hick:doc>\n",
}
def main():
 def interrupted(number,_frame):raise SystemExit(128+number)
 signal.signal(signal.SIGTERM,interrupted)
 spec=importlib.util.spec_from_file_location('fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 with tempfile.TemporaryDirectory(prefix='hickory-bulk-literate-') as temporary:
  folder=Path(temporary);root=folder/'workspace';root.mkdir();events=folder/'events.jsonl';gate=folder/'release';source=folder/'source.json';source.write_text(json.dumps(SOURCE))
  for path,text in SOURCE.items():(root/path).write_text(text)
  (root/'.hick-lsp.json').write_text(json.dumps({'servers':{'python':{'command':[sys.executable,str(ROOT/'scripts/controlled-bulk-literate-provider.py'),str(events),str(gate)]}}}))
  engine=fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(root),'window','http://127.0.0.1:1'],env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state')})
  try:
   subprocess.run(['node',str(ROOT/'scripts/provider-bulk-literate-driver.mjs'),fixture.origin(engine),str(events),str(gate),str(source)],cwd=ROOT,check=True,timeout=120)
   for path,text in SOURCE.items():assert (root/path).read_text()==text,f'Provider action changed saved source: {path}'
   print('PASS: all original Markdown source bytes retained; controlled-provider acceptance is not language semantic or physical-input/accessibility evidence.')
  finally:fixture.stop(engine)
if __name__=='__main__':main()
</ui:copy>

<ui:file path="scripts/controlled-bulk-literate-provider.py"><ui:paste select="#bulk-literate-controlled-provider"/></ui:file>
<ui:file path="scripts/provider-bulk-literate-driver.mjs"><ui:paste select="#bulk-literate-protocol-driver"/></ui:file>
<ui:file path="scripts/test-provider-bulk-literate.py"><ui:paste select="#bulk-literate-protocol-runner"/></ui:file>
</ui:doc>
