import assert from 'node:assert/strict';
import {readFile,writeFile} from 'node:fs/promises';
const url=new URL('/api/ws?doc=workspace',process.argv[2]);url.protocol=url.protocol==='https:'?'wss:':'ws:';
const socket=new WebSocket(url);socket.binaryType='arraybuffer';let sequence=0;
const pending=new Map();let readyResolve;const ready=new Promise(resolve=>{readyResolve=resolve;});
function send(body){const data=new TextEncoder().encode(JSON.stringify({jsonrpc:'2.0',...body}));const frame=new Uint8Array(data.length+1);frame[0]=2;frame.set(data,1);socket.send(frame);}
function request(method,params){return new Promise((resolve,reject)=>{const id=++sequence,timer=setTimeout(()=>{pending.delete(id);reject(new Error('LSP request timed out: '+method));},20000);pending.set(id,{resolve,reject,timer});send({id,method,params});});}
socket.addEventListener('message',event=>{if(!(event.data instanceof ArrayBuffer))return;const bytes=new Uint8Array(event.data);if(bytes[0]!==2)return;const body=JSON.parse(new TextDecoder().decode(bytes.subarray(1)));
 if(body.method==='hick/serverCapabilities'){assert.equal(body.params.capabilities.experimental.hickoryTypeHierarchy,true);readyResolve();return;}
 const held=pending.get(body.id);if(!held)return;pending.delete(body.id);clearTimeout(held.timer);if(body.error)held.reject(new Error(body.error.message));else held.resolve(body.result);
});
const open=new Promise((resolve,reject)=>{socket.addEventListener('open',resolve,{once:true});socket.addEventListener('error',reject,{once:true});});
const a='hick:///a.py',b='hick:///b.ts',position={line:0,character:1};
try{
 await open;
 for(const [uri,languageId,text]of[[a,'python','value = 1\n'],[b,'typescript','value = 2;\n']])send({method:'textDocument/didOpen',params:{textDocument:{uri,languageId,version:1,text}}});
 await Promise.race([ready,new Promise((_,reject)=>setTimeout(()=>reject(new Error('No server capabilities')),20000).unref())]);
 for(const uri of[a,b])await request('textDocument/documentSymbol',{textDocument:{uri}});
 const prepare=uri=>request('textDocument/prepareTypeHierarchy',{textDocument:{uri},position});
 const first=(await prepare(a))[0],second=(await prepare(b))[0];assert.equal(first.tags,undefined);assert.deepEqual(Object.keys(first.data),['hickory_type']);
 const changed={...first,name:'Caller substitution',uri:b,range:second.range};
 const bases=await request('typeHierarchy/supertypes',{item:changed});assert.equal(bases[0].name,'pythonBase');assert.equal(bases[0].uri,a);assert.equal(bases[0].tags,undefined);
 const derived=await request('typeHierarchy/subtypes',{item:second});assert.equal(derived[0].name,'typescriptDerived');assert.equal(derived[0].uri,b);
 const followed=await request('typeHierarchy/subtypes',{item:bases[0]});assert.equal(followed[0].name,'pythonDerived');
 send({method:'textDocument/didChange',params:{textDocument:{uri:a,version:2},contentChanges:[{text:'other = 1\n'}]}});await request('textDocument/documentSymbol',{textDocument:{uri:a}});
 await assert.rejects(request('typeHierarchy/supertypes',{item:first}),/source changed|ownership changed|expired/i);
 await assert.rejects(request('typeHierarchy/subtypes',{item:{...second,data:{hickory_type:'foreign'}}}),/source changed|expired/i);
 const slow=(await request('textDocument/prepareTypeHierarchy',{textDocument:{uri:a},position:{line:0,character:2}}))[0];
 const held=request('typeHierarchy/supertypes',{item:slow}).then(value=>({value}),error=>({error}));
 const deadline=Date.now()+3000;let started=false;
 while(!started&&Date.now()<deadline){const events=await readFile(process.argv[3],'utf8').catch(()=> '');started=events.split('\n').filter(Boolean).some(line=>JSON.parse(line).slow_type===true);if(!started)await new Promise(resolve=>setTimeout(resolve,5));}
 assert.ok(started,'Hold the type follow-up before changing source');
 send({method:'textDocument/didChange',params:{textDocument:{uri:a,version:3},contentChanges:[{text:'third = 1\n'}]}});await request('textDocument/documentSymbol',{textDocument:{uri:a}});
 await writeFile(process.argv[4],'release');const completed=await held;assert.match(completed.error?.message??'',/source changed|ownership changed/i);
 console.log('PASS: type hierarchy across two providers, original opaque data, caller substitution refusal, fresh follow-up tokens, original child tag arrays and stale/in-flight source refusal');
}finally{for(const value of pending.values()){clearTimeout(value.timer);value.reject(new Error('Client closed'));}pending.clear();socket.close();}
