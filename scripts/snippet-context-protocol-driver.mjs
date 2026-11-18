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
function bind(item,clipboard=null){return {...item,data:{...item.data,hickory_context:{version:1,expected:item.data.hickory_context,selection:{from:0,to:3},...snapshot,clipboard}}};}
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
 console.log('PASS: source-bound v2 variables/transforms, demand-only clipboard retry, denied fallback, unused default privacy, cached child resolve and foreign proof refusal');
}finally{for(const held of pending.values()){clearTimeout(held.timer);held.reject(Error('Closed'));}pending.clear();socket.close();}
