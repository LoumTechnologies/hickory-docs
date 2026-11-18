import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
const [origin,fixtureFile]=process.argv.slice(2), fixture=JSON.parse(await readFile(fixtureFile,'utf8'));
const url=new URL('/api/ws?doc=workspace',origin);url.protocol=url.protocol==='https:'?'wss:':'ws:';
const socket=new WebSocket(url);socket.binaryType='arraybuffer';let sequence=0;const pending=new Map();
function send(body){const raw=new TextEncoder().encode(JSON.stringify({jsonrpc:'2.0',...body}));const bytes=new Uint8Array(raw.length+1);bytes[0]=2;bytes.set(raw,1);socket.send(bytes);}
function request(method,params){return new Promise((resolve,reject)=>{const id=++sequence,timer=setTimeout(()=>{pending.delete(id);reject(new Error('Fenced reading request timeout: '+method));},30000);pending.set(id,{resolve,reject,timer});send({id,method,params});});}
socket.addEventListener('message',event=>{if(!(event.data instanceof ArrayBuffer))return;const bytes=new Uint8Array(event.data);if(bytes[0]!==2)return;const body=JSON.parse(new TextDecoder().decode(bytes.subarray(1)));if(body.method&&body.id!==undefined){send({id:body.id,error:{code:-32601,message:'Owned client does not execute provider commands'}});return;}const call=pending.get(body.id);if(!call)return;pending.delete(body.id);clearTimeout(call.timer);body.error?call.reject(new Error(body.error.message)):call.resolve(body.result);});
await new Promise((resolve,reject)=>{socket.addEventListener('open',resolve,{once:true});socket.addEventListener('error',reject,{once:true});});
const uri='hick:///notes.md';
async function capture(text,version){const reply=await request('hick/captureDocument',{textDocument:{uri,languageId:'markdown',version,text}});assert.equal(reply.current,true,JSON.stringify(reply));assert.equal(reply.processed,true);assert.equal(reply.source_files_modified,false);return reply;}
const locations=value=>Array.isArray(value)?value:[value];
try {
 const source=fixture.source;send({method:'textDocument/didOpen',params:{textDocument:{uri,languageId:'markdown',version:1,text:source}}});await capture(source,1);
 for(const row of fixture.rows){
  const params={textDocument:{uri},position:row.query};
  const hover=await request('textDocument/hover',params);assert.ok(hover?.contents,JSON.stringify(hover));assert.match(JSON.stringify(hover),/greet/);
  const definition=await request('textDocument/definition',params);assert.ok(definition,JSON.stringify(definition));
  const target=locations(definition).find(value=>(value.targetUri??value.uri)===uri);assert.ok(target,JSON.stringify(definition));
  assert.deepEqual((target.targetSelectionRange??target.range).start,row.definition);
  assert.equal(await request('textDocument/rename',{...params,newName:'must_not_change'}),null,'A read-only address must not grant rename');
  assert.equal(await request('textDocument/completion',params),null,'Inspection has no insertion authority');
 }
 assert.deepEqual(await request('textDocument/formatting',{textDocument:{uri},options:{tabSize:4,insertSpaces:true}}),null);
 assert.equal(await request('textDocument/hover',{textDocument:{uri},position:{line:0,character:0}}),null,'Prose is not code');
 const changed=source.replaceAll('greet','welcome');await capture(changed,2);
 for(const row of fixture.rows){const hover=await request('textDocument/hover',{textDocument:{uri},position:row.query});assert.ok(hover?.contents);assert.match(JSON.stringify(hover),/welcome/);assert.doesNotMatch(JSON.stringify(hover),/\bgreet\b/);}
 await capture(source,3);
 for(const row of fixture.rows){const result=await request('textDocument/definition',{textDocument:{uri},position:row.query});const target=locations(result).find(value=>(value.targetUri??value.uri)===uri);assert.deepEqual((target.targetSelectionRange??target.range).start,row.definition);}
 send({method:'textDocument/didClose',params:{textDocument:{uri}}});
 assert.equal(await request('textDocument/hover',{textDocument:{uri},position:fixture.rows[0].query}),null);
 console.log('PASS: installed Python and TypeScript inspect independent static fences, authored Unicode/CRLF definition coordinates, prose exclusion, dirty source replacement/Undo refresh, close retirement, and no completion/rename/format authority.');
} finally {for(const call of pending.values()){clearTimeout(call.timer);call.reject(new Error('Owned client closed'));}pending.clear();socket.close();}
