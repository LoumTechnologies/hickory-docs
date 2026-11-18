import assert from 'node:assert/strict';
import {readFile,writeFile} from 'node:fs/promises';
const url=new URL('/api/ws?doc=workspace',process.argv[2]);url.protocol=url.protocol==='https:'?'wss:':'ws:';
const socket=new WebSocket(url);socket.binaryType='arraybuffer';let sequence=0;
const pending=new Map();let readyResolve;const ready=new Promise(resolve=>{readyResolve=resolve;});
function send(body){const data=new TextEncoder().encode(JSON.stringify({jsonrpc:'2.0',...body}));const frame=new Uint8Array(data.length+1);frame[0]=2;frame.set(data,1);socket.send(frame);}
function request(method,params){return new Promise((resolve,reject)=>{const id=++sequence,timer=setTimeout(()=>{pending.delete(id);reject(new Error('LSP request timed out: '+method));},20000);pending.set(id,{resolve,reject,timer});send({id,method,params});});}
socket.addEventListener('message',event=>{if(!(event.data instanceof ArrayBuffer))return;const bytes=new Uint8Array(event.data);if(bytes[0]!==2)return;const body=JSON.parse(new TextDecoder().decode(bytes.subarray(1)));
 if(body.method==='hick/serverCapabilities'){readyResolve();return;}
 const held=pending.get(body.id);if(!held)return;pending.delete(body.id);clearTimeout(held.timer);if(body.error)held.reject(new Error(body.error.message));else held.resolve(body.result);
});
const open=new Promise((resolve,reject)=>{socket.addEventListener('open',resolve,{once:true});socket.addEventListener('error',reject,{once:true});});
const a='hick:///a.py',b='hick:///b.ts',position={line:0,character:1};
try{
 await open;
 for(const [uri,languageId,text]of[[a,'python','value = 1\n'],[b,'typescript','value = 2;\n']])send({method:'textDocument/didOpen',params:{textDocument:{uri,languageId,version:1,text}}});
 await Promise.race([ready,new Promise((_,reject)=>setTimeout(()=>reject(new Error('No server capabilities')),20000).unref())]);
 for(const uri of[a,b])await request('textDocument/documentSymbol',{textDocument:{uri}});
 const readEvents=async()=> (await readFile(process.argv[3],'utf8').catch(()=> '')).split('\n').filter(Boolean).map(line=>JSON.parse(line));
 const beforeUnsupported=(await readEvents()).filter(event=>event.completion_query).length;
 const unsupported=await request('textDocument/completion',{textDocument:{uri:a},position,context:{triggerKind:2,triggerCharacter:':'}});
 assert.deepEqual(Array.isArray(unsupported)?unsupported:unsupported?.items??[],[]);
 assert.equal((await readEvents()).filter(event=>event.completion_query).length,beforeUnsupported,'An unsupported implicit child trigger must not issue a child RPC');
 await assert.rejects(request('textDocument/completion',{textDocument:{uri:a},position,context:{triggerKind:99}}),/trigger kind|invalid params/i);
 assert.equal((await readEvents()).filter(event=>event.completion_query).length,beforeUnsupported,'Invalid trigger metadata must not issue a child RPC');
 const explicit=await request('textDocument/completion',{textDocument:{uri:a},position,context:{triggerKind:1}});assert.ok((Array.isArray(explicit)?explicit:explicit.items).length>0);
 assert.ok((await readEvents()).some(event=>event.completion_query==='python'&&event.context?.triggerKind===1&&!Object.hasOwn(event.context,'triggerCharacter')),'Explicit invocation remains explicit without a forged trigger character');
 const complete=uri=>request('textDocument/completion',{textDocument:{uri},position,context:{triggerKind:2,triggerCharacter:'.'}});
 const first=(await complete(a)).items[0],second=(await complete(b)).items[0];assert.deepEqual(Object.keys(first.data).sort(),['hickory_completion','hickory_context']);assert.match(first.data.hickory_context.source_hash,/^sha256:[a-f0-9]{64}$/);assert.match(first.data.hickory_context.source_revision,/^[a-f0-9]{64}$/);assert.equal(first.textEdit.newText,'python_initial');
 const changed={...first,label:'Caller substitution',insertText:'must-not-insert',command:{title:'Injected',command:'must-not-execute'}};
 const resolved=await request('completionItem/resolve',changed);assert.equal(resolved.label,'python completion');assert.equal(resolved.command,undefined);assert.equal(resolved.textEdit.newText,'python_completed');assert.equal(resolved.additionalTextEdits[0].range.start.line,1);assert.equal(resolved.additionalTextEdits[0].newText,'# imported\n');assert.deepEqual(Object.keys(resolved.data).sort(),['hickory_completion','hickory_context']);assert.deepEqual(resolved.data.hickory_context,first.data.hickory_context);
 const eager=await request('completionItem/resolve',{...second,label:'Changed eager value'});assert.equal(eager.label,'typescript completion');assert.equal(eager.textEdit.newText,'typescript_initial');
 send({method:'textDocument/didChange',params:{textDocument:{uri:a,version:2},contentChanges:[{text:'other = 1\n'}]}});await request('textDocument/documentSymbol',{textDocument:{uri:a}});
 await assert.rejects(request('completionItem/resolve',first),/source changed|ownership changed|expired/i);
 await assert.rejects(request('completionItem/resolve',{...second,data:{hickory_completion:'foreign'}}),/source changed|expired/i);
 const slow=(await request('textDocument/completion',{textDocument:{uri:a},position:{line:0,character:2}})).items[0];
 const held=request('completionItem/resolve',slow).then(value=>({value}),error=>({error}));
 const deadline=Date.now()+3000;let started=false;
 while(!started&&Date.now()<deadline){const events=await readFile(process.argv[3],'utf8').catch(()=> '');started=events.split('\n').filter(Boolean).some(line=>JSON.parse(line).slow_completion===true);if(!started)await new Promise(resolve=>setTimeout(resolve,5));}
 assert.ok(started,'Hold the completion resolver before changing source');
 send({method:'textDocument/didChange',params:{textDocument:{uri:a,version:3},contentChanges:[{text:'third = 1\n'}]}});await request('textDocument/documentSymbol',{textDocument:{uri:a}});
 await writeFile(process.argv[4],'release');const completed=await held;assert.match(completed.error?.message??'',/source changed|ownership changed/i);
 const events=(await readFile(process.argv[3],'utf8')).split('\n').filter(Boolean).map(line=>JSON.parse(line));assert.deepEqual(events.filter(item=>item.completion_resolved).map(item=>item.completion_resolved),['python','python']);assert.ok(events.some(item=>item.completion_query==='python'&&item.context?.triggerCharacter==='.'&&item.context?.triggerKind===2));assert.ok(!events.some(item=>item.error));
 console.log('PASS: completion defaults, actual producing provider, original opaque data, complete resolved import edit, eager fallback and stale/in-flight source refusal');
}finally{for(const value of pending.values()){clearTimeout(value.timer);value.reject(new Error('Client closed'));}pending.clear();socket.close();}
