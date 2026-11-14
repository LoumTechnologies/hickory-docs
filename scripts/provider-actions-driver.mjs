import assert from 'node:assert/strict';
import {readFile,writeFile} from 'node:fs/promises';
const url=new URL('/api/ws?doc=workspace',process.argv[2]);url.protocol=url.protocol==='https:'?'wss:':'ws:';
const socket=new WebSocket(url);socket.binaryType='arraybuffer';let sequence=0,capabilities;
const pending=new Map();let readyResolve;const ready=new Promise(resolve=>{readyResolve=resolve;});
function send(body){const data=new TextEncoder().encode(JSON.stringify({jsonrpc:'2.0',...body}));const frame=new Uint8Array(data.length+1);frame[0]=2;frame.set(data,1);socket.send(frame);}
function request(method,params){return new Promise((resolve,reject)=>{const id=++sequence,timer=setTimeout(()=>{pending.delete(id);reject(new Error('LSP request timed out: '+method));},20000);pending.set(id,{resolve,reject,timer});send({id,method,params});});}
socket.addEventListener('message',event=>{if(!(event.data instanceof ArrayBuffer))return;const bytes=new Uint8Array(event.data);if(bytes[0]!==2)return;const body=JSON.parse(new TextDecoder().decode(bytes.subarray(1)));
 if(body.method==='hick/serverCapabilities'){capabilities=body.params.capabilities;readyResolve();return;}
 if(body.method&&body.id!==undefined){send({id:body.id,error:{code:-32601,message:'Controlled client does not execute commands'}});return;}
 const held=pending.get(body.id);if(!held)return;pending.delete(body.id);clearTimeout(held.timer);if(body.error)held.reject(new Error(body.error.message));else held.resolve(body.result);
});
const open=new Promise((resolve,reject)=>{socket.addEventListener('open',resolve,{once:true});socket.addEventListener('error',reject,{once:true});});
const position={line:0,character:1};const a='hick:///a.py',b='hick:///b.ts';
try{
 await open;
 for(const [uri,languageId,text]of[[a,'python','value = 1\n'],[b,'typescript','value = 2;\n']])send({method:'textDocument/didOpen',params:{textDocument:{uri,languageId,version:1,text}}});
 await Promise.race([ready,new Promise((_,reject)=>setTimeout(()=>reject(new Error('No server capabilities')),20000).unref())]);assert.equal(capabilities.codeActionProvider.resolveProvider,true);
 for(const uri of[a,b])await request('textDocument/documentSymbol',{textDocument:{uri}});
 const actions=uri=>request('textDocument/codeAction',{textDocument:{uri},range:{start:position,end:position},context:{diagnostics:[]}});
 const first=(await actions(a))[0],second=(await actions(b))[0];assert.equal(first.edit,undefined);assert.ok(second.edit);assert.deepEqual(Object.keys(first.data),['hickory_action']);
 const changed={...first,title:'Client substitution',command:{title:'Injected',command:'must-never-execute',arguments:[b]}};
 const resolved=await request('codeAction/resolve',changed);assert.equal(resolved.title,'Controlled python fix');assert.equal(resolved.command,undefined);assert.equal(resolved.edit.documentChanges[0].textDocument.uri,a);assert.equal(resolved.edit.documentChanges[0].edits[0].newText,'python_fixed');
 const other=await request('codeAction/resolve',{...second,title:'Client eager substitution',edit:{changes:{[a]:[]}}});assert.equal(other.title,'Controlled typescript fix');assert.equal(other.edit.documentChanges[0].textDocument.uri,b);assert.equal(other.edit.documentChanges[0].edits[0].newText,'typescript_fixed');
 send({method:'textDocument/didChange',params:{textDocument:{uri:a,version:2},contentChanges:[{text:'other = 1\n'}]}});await request('textDocument/documentSymbol',{textDocument:{uri:a}});
 await assert.rejects(request('codeAction/resolve',first),/source changed|ownership changed|expired/i);
 await assert.rejects(request('codeAction/resolve',{...first,data:{hickory_action:'foreign'}}),/source changed|expired/i);
 const slow=(await request('textDocument/codeAction',{textDocument:{uri:a},range:{start:{line:0,character:2},end:{line:0,character:2}},context:{diagnostics:[]}}))[0];
 const held=request('codeAction/resolve',slow).then(value=>({value}),error=>({error}));
 const deadline=Date.now()+3000;let started=false;
 while(!started&&Date.now()<deadline){const text=await readFile(process.argv[3],'utf8').catch(()=> '');started=text.split('\n').filter(Boolean).some(line=>JSON.parse(line).slow===true);if(!started)await new Promise(resolve=>setTimeout(resolve,5));}
 assert.ok(started,'The controlled resolver must be held before changing source');
 send({method:'textDocument/didChange',params:{textDocument:{uri:a,version:3},contentChanges:[{text:'third = 1\n'}]}});await request('textDocument/documentSymbol',{textDocument:{uri:a}});
 await writeFile(process.argv[4],'release');const completed=await held;assert.match(completed.error?.message??'',/source changed|ownership changed/i);

 console.log('PASS: lazy action routing, original opaque payload, client substitution refusal, complete projection and stale/foreign token refusal');
}finally{for(const value of pending.values()){clearTimeout(value.timer);value.reject(new Error('Client closed'));}pending.clear();socket.close();}
