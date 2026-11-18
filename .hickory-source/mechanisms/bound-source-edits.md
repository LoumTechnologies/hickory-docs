<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Bind rename and formatting to the acknowledged source

For maintainers closing the interval between source preparation and a provider edit request.

A preparation acknowledgment cannot authorize a later standard LSP request after another editor replaces the shared source. The custom `hick/rename` and `hick/formatting` requests carry ordinary LSP fields plus `expected:{source_hash,source_revision}` from the exact acknowledgment. Check that origin before dispatch and after every response. Freeze mapping and document identities before provider work, then validate only the actual origin, contributors and affected preimages. An unrelated document preparing does not block coherent source work. No lock spans a child await.

## Hold coordinates through the whole operation

A successful result is an engine-stamped action envelope with an opaque held token, exact coordinate hashes and `source_files_modified:false`. Caller substitution cannot replace the held mapped action. Unknown sources return acquisition-only metadata and require another synchronized query. Formatting gathers all output responses and projects them as one complete operation; provider failure, malformed or unmappable edits refuse the entire result.

<ui:copy id="bound-source-edit-queries">
#[derive(Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectedSourceCapture {source_hash:String,source_revision:String}
#[derive(serde::Deserialize)]
pub struct BoundRenameParams {#[serde(flatten)]params:RenameParams,expected:ExpectedSourceCapture}
#[derive(serde::Deserialize)]
pub struct BoundFormattingParams {#[serde(flatten)]params:DocumentFormattingParams,expected:ExpectedSourceCapture}
struct BoundSourceSnapshot {
 maps:HashMap<Url,VFileMapping>,
 documents:HashMap<Url,(crate::source_identity::SourceRevision,Option<String>,bool)>,
}
impl BoundSourceSnapshot {
 fn index(&self)->HashMap<String,(Url,PositionMap)>{self.maps.iter().map(|(uri,map)|(uri_key(uri.as_str()),(map.hick_uri.clone(),map.position_map.clone()))).collect()}
 fn expected(&self,uri:&Url,expected:&ExpectedSourceCapture)->Result<()> {
  let valid=self.documents.get(uri).is_some_and(|(revision,hash,preparing)|{
   !preparing&&hash.as_deref()==Some(expected.source_hash.as_str())&&crate::source_identity::source_graph_fingerprint(std::iter::once((uri.as_str(),*revision)))==expected.source_revision
  });
  if !valid{return Err(tower_lsp::jsonrpc::Error::invalid_params("Source capture changed or is still preparing. Synchronize the intended source and prepare again."));}Ok(())
 }
 fn sources(&self,maps:&[Url])->Result<Arc<Vec<ProviderSourceProof>>> {
  let mut rows=std::collections::BTreeMap::<String,ProviderSourceProof>::new();
  for uri in maps {
   let mapping=self.maps.get(uri).ok_or_else(||tower_lsp::jsonrpc::Error::invalid_params("Provider mapping changed. Prepare again."))?;
   for mut proof in action_source_proofs(mapping)?.iter().cloned(){
    if let Some((revision,hash,preparing))=self.documents.get(&proof.uri){
     if *preparing||hash.as_deref()!=Some(proof.hash.as_str()){return Err(tower_lsp::jsonrpc::Error::invalid_params("Captured provider source changed or is still preparing. Prepare again."));}
     proof.open_revision=Some(*revision);
    }
    if rows.get(proof.uri.as_str()).is_some_and(|old|old!=&proof){return Err(tower_lsp::jsonrpc::Error::invalid_params("Provider mappings captured conflicting source bytes. Prepare again."));}
    rows.insert(proof.uri.to_string(),proof);
   }
  }
  Ok(Arc::new(rows.into_values().collect()))
 }
 fn affected_maps(&self,original:&Value,targets:&[Url])->Result<Vec<Url>> {
  let mut selected=targets.to_vec();
  for raw in action_input_uris(original).ok_or_else(||tower_lsp::jsonrpc::Error::invalid_params("Malformed source-bound provider edits."))? {
   for (uri,map) in &self.maps {
    if uri_key(uri.as_str())==uri_key(&raw)||map.hick_uri.as_str()==raw {selected.push(uri.clone());}
   }
  }
  selected.sort_by(|a,b|a.as_str().cmp(b.as_str()));selected.dedup();Ok(selected)
 }
}
impl HickBackend {
 async fn bound_snapshot(&self)->BoundSourceSnapshot {
  let maps=self.vfile_index.read().await.clone();
  let documents=self.documents.read().await.iter().map(|(uri,doc)|(uri.clone(),(doc.source_revision,doc.source_hash.clone(),doc.preparing))).collect();
  BoundSourceSnapshot{maps,documents}
 }
 async fn require_bound_capture(&self,uri:&Url,expected:&ExpectedSourceCapture)->Result<()> {
  let documents=self.documents.read().await;
  let valid=documents.get(uri).is_some_and(|doc|!doc.preparing&&doc.source_hash.as_deref()==Some(expected.source_hash.as_str())&&crate::source_identity::source_graph_fingerprint(std::iter::once((uri.as_str(),doc.source_revision)))==expected.source_revision);
  if !valid{return Err(tower_lsp::jsonrpc::Error::invalid_params("Source capture changed or is still preparing. Synchronize the intended source and prepare again."));}Ok(())
 }
 async fn require_bound_maps(&self,snapshot:&BoundSourceSnapshot,targets:&[Url])->Result<()> {
  let current=self.vfile_index.read().await;
  if targets.iter().any(|uri|snapshot.maps.get(uri).map(|map|map.source_revision)!=current.get(uri).map(|map|map.source_revision)) {return Err(tower_lsp::jsonrpc::Error::invalid_params("Provider ownership changed. Prepare again."));}Ok(())
 }
 async fn bound_child(&self,mapping:&VFileMapping)->Result<ChildLspHandle>{
  let dispatcher=self.dispatcher.lock().await;
  dispatcher.get_child(&mapping.language_id,&mapping.root_uri).cloned().map_err(|_|tower_lsp::jsonrpc::Error::invalid_params("The required provider is unavailable. No partial source change was prepared."))
 }
 async fn finish_bound_edit(&self,title:&str,edit:Value,snapshot:&BoundSourceSnapshot,targets:&[Url],handle:ChildLspHandle,origin:&Url,expected:&ExpectedSourceCapture)->Result<Value>{
  let original=json!({"title":title,"edit":edit});let targets=snapshot.affected_maps(&original,targets)?;
  let proofs=snapshot.sources(&targets)?;self.require_action_sources(&proofs).await?;self.require_bound_maps(snapshot,&targets).await?;self.require_bound_capture(origin,expected).await?;
  let index=snapshot.index();let root=self.workspace_root.read().await.clone();
  let mut action=bound_project_edit(original,&index,root.as_ref())?;
  self.require_action_sources(&proofs).await?;self.require_bound_maps(snapshot,&targets).await?;self.require_bound_capture(origin,expected).await?;
  let revision=self.provider_source_revision().await;
  let data=action_discovery_data(action.get("data"));
  let token=self.prepared_actions.lock().await.values.insert(Instant::now(),revision,PreparedAction{original:action.clone(),handle,resolve:false,refactor_query:None,already_mapped:true,display_title:None,source_proofs:proofs.clone()});
  action["data"]=data;action["data"]["hickory_action"]=json!(token);action["data"]["hickory_sources"]=action_source_data(&proofs);action["source_files_modified"]=json!(false);
  self.require_action_sources(&proofs).await?;self.require_bound_maps(snapshot,&targets).await?;self.require_bound_capture(origin,expected).await?;Ok(action)
 }
 pub async fn source_bound_rename(&self,params:BoundRenameParams)->Result<Value>{
  let origin=params.params.text_document_position.text_document.uri.clone();let snapshot=self.bound_snapshot().await;snapshot.expected(&origin,&params.expected)?;
  let Some((target,_,position))=self.virtual_target(&origin,params.params.text_document_position.position).await else{return Err(tower_lsp::jsonrpc::Error::invalid_params("This source position has no unambiguous provider rename target."));};
  let mapping=snapshot.maps.get(&target).ok_or_else(||tower_lsp::jsonrpc::Error::invalid_params("Provider mapping changed. Prepare again."))?;
  let sources=snapshot.sources(std::slice::from_ref(&target))?;self.require_action_sources(&sources).await?;self.require_bound_maps(&snapshot,std::slice::from_ref(&target)).await?;self.require_bound_capture(&origin,&params.expected).await?;
  let handle=self.bound_child(mapping).await?;
  let edit=handle.request("textDocument/rename",json!({"textDocument":{"uri":target},"position":position,"newName":params.params.new_name})).await.map_err(child_request_error)?;
  let edit=if edit.is_null(){json!({"documentChanges":[]})}else{edit};
  self.finish_bound_edit("Rename",edit,&snapshot,&[target],handle,&origin,&params.expected).await
 }
 pub async fn source_bound_formatting(&self,params:BoundFormattingParams)->Result<Value>{
  let origin=params.params.text_document.uri;let snapshot=self.bound_snapshot().await;snapshot.expected(&origin,&params.expected)?;
  let associated=snapshot.maps.iter().filter(|(_,map)|provider_target_contributes(map,&origin)).collect::<Vec<_>>();let project=associated.iter().any(|(_,map)|map.position_map.is_project());
  let mut targets=associated.into_iter().filter(|(_,map)|!project||map.position_map.is_project()).map(|(uri,_)|uri.clone()).collect::<Vec<_>>();targets.sort_by(|a,b|a.as_str().cmp(b.as_str()));
  if targets.is_empty()||targets.len()>512{return Err(tower_lsp::jsonrpc::Error::invalid_params("Complete formatting requires between one and 512 unambiguous provider outputs."));}
  let sources=snapshot.sources(&targets)?;self.require_action_sources(&sources).await?;self.require_bound_maps(&snapshot,&targets).await?;self.require_bound_capture(&origin,&params.expected).await?;
  let mut changes=Vec::new();let mut first=None;
  for target in &targets {
   self.require_action_sources(&sources).await?;self.require_bound_maps(&snapshot,&targets).await?;self.require_bound_capture(&origin,&params.expected).await?;
   let handle=self.bound_child(&snapshot.maps[target]).await?;if first.is_none(){first=Some(handle.clone());}
   let edits=handle.request("textDocument/formatting",json!({"textDocument":{"uri":target},"options":params.params.options})).await.map_err(child_request_error)?;
   if edits.is_null(){continue;}if !edits.is_array(){return Err(tower_lsp::jsonrpc::Error::invalid_params("A formatter returned malformed edits. No partial formatting was prepared."));}
   changes.push(json!({"textDocument":{"uri":target,"version":null},"edits":edits}));
  }
  self.finish_bound_edit("Format document",json!({"documentChanges":changes}),&snapshot,&targets,first.expect("nonempty complete formatting"),&origin,&params.expected).await
 }
}
</ui:copy>

## Refuse stale capture and partial formatting as observable outcomes

These focused tests change server-owned lifetimes without changing bytes, mutate an affected source after an immutable snapshot, and keep unrelated preparation independent. The pure complete projection gate must reject a valid edit combined with one unmappable edit; it cannot return the valid subset.

<ui:copy id="bound-source-edit-witness">
fn bound_project_edit(original:Value,index:&HashMap<String,(Url,PositionMap)>,root:Option<&Url>)->Result<Value>{
 if let Some(discovery)=discover_action_sources(&original,index,root){return Ok(discovery);}
 let value=translate_provider_edits(json!([original]),index,root).map_err(tower_lsp::jsonrpc::Error::invalid_params)?;
 let action=value.as_array().and_then(|rows|rows.first()).cloned().ok_or_else(||tower_lsp::jsonrpc::Error::invalid_params("Malformed complete source-bound edit."))?;
 if action.get("disabled").is_some_and(|value|!value.is_null()){return Err(tower_lsp::jsonrpc::Error::invalid_params("A complete provider edit cannot be projected safely. No partial change was prepared."));}Ok(action)
}
#[cfg(test)]
mod bound_edit_tests {
 use super::*;
 fn expected(uri:&Url,entry:&DocEntry)->ExpectedSourceCapture{ExpectedSourceCapture{source_hash:entry.source_hash.clone().unwrap(),source_revision:crate::source_identity::source_graph_fingerprint(std::iter::once((uri.as_str(),entry.source_revision)))}}
 fn mapping(uri:&Url,text:&str)->VFileMapping{VFileMapping{source_revision:crate::source_identity::SourceRevision::fresh(),hick_uri:uri.clone(),language_id:"python".into(),root_uri:"file:///workspace/".into(),position_map:PositionMap::identity(10),provider_diagnostics:vec![], provider_diagnostics_from_pull: false,provider_source:text.into(),provider_version:Some(1),primary_source_hash:diagnostic_source_digest(text)}}
 #[tokio::test]
 async fn exact_origin_capture_rejects_equal_byte_aba_and_preparing_without_dispatch(){
  let (service,_)=tower_lsp::LspService::new(HickBackend::new);let backend=service.inner();let uri=Url::parse("file:///workspace/a.py").unwrap();
  let mut entry=DocEntry::empty();entry.source_hash=Some(diagnostic_source_digest("same"));let receipt=expected(&uri,&entry);backend.documents.write().await.insert(uri.clone(),entry);
  backend.require_bound_capture(&uri,&receipt).await.unwrap();
  backend.documents.write().await.get_mut(&uri).unwrap().source_revision=crate::source_identity::SourceRevision::fresh();
  assert!(backend.require_bound_capture(&uri,&receipt).await.is_err());
  let params=BoundRenameParams{params:serde_json::from_value(json!({"textDocument":{"uri":uri},"position":{"line":0,"character":0},"newName":"later"})).unwrap(),expected:receipt};
  assert!(backend.source_bound_rename(params).await.unwrap_err().message.contains("capture changed"));
 }
 #[tokio::test]
 async fn touched_source_and_map_lifetimes_never_rebind_while_unrelated_preparation_remains_usable(){
  let (service,_)=tower_lsp::LspService::new(HickBackend::new);let backend=service.inner();let uri=Url::parse("file:///workspace/a.py").unwrap();let busy=Url::parse("file:///workspace/unrelated.md").unwrap();
  let mut entry=DocEntry::empty();entry.source_hash=Some(diagnostic_source_digest("same"));let receipt=expected(&uri,&entry);
  let mut preparing=DocEntry::empty();preparing.preparing=true;
  backend.documents.write().await.extend([(uri.clone(),entry),(busy,preparing)]);backend.vfile_index.write().await.insert(uri.clone(),mapping(&uri,"same"));
  let snapshot=backend.bound_snapshot().await;snapshot.expected(&uri,&receipt).unwrap();let proofs=snapshot.sources(std::slice::from_ref(&uri)).unwrap();backend.require_action_sources(&proofs).await.unwrap();backend.require_bound_maps(&snapshot,std::slice::from_ref(&uri)).await.unwrap();
  backend.documents.write().await.get_mut(&uri).unwrap().source_revision=crate::source_identity::SourceRevision::fresh();assert!(backend.require_action_sources(&proofs).await.is_err());
  backend.vfile_index.write().await.get_mut(&uri).unwrap().source_revision=crate::source_identity::SourceRevision::fresh();assert!(backend.require_bound_maps(&snapshot,std::slice::from_ref(&uri)).await.is_err());
 }
 #[tokio::test]
 async fn affected_preimages_come_from_the_original_snapshot_not_a_later_equal_byte_owner(){
  let (service,_)=tower_lsp::LspService::new(HickBackend::new);let backend=service.inner();let a=Url::parse("file:///workspace/a.py").unwrap();let b=Url::parse("file:///workspace/b.py").unwrap();
  for uri in [&a,&b]{let mut doc=DocEntry::empty();doc.source_hash=Some(diagnostic_source_digest("same"));backend.documents.write().await.insert(uri.clone(),doc);backend.vfile_index.write().await.insert(uri.clone(),mapping(uri,"same"));}
  let snapshot=backend.bound_snapshot().await;let key=b.as_str();let original=json!({"edit":{"changes":{key:[]}}});let targets=snapshot.affected_maps(&original,std::slice::from_ref(&a)).unwrap();assert_eq!(targets.len(),2);
  backend.documents.write().await.get_mut(&b).unwrap().source_revision=crate::source_identity::SourceRevision::fresh();let proofs=snapshot.sources(&targets).unwrap();assert!(backend.require_action_sources(&proofs).await.is_err());
 }
 #[test]
 fn complete_formatting_projection_refuses_a_valid_subset_when_any_output_is_unmappable(){
  let uri=Url::parse("hick:///a.py").unwrap();let map=PositionMap::build(&[crate::virtual_file::VirtualSegment{text:"x".into(),source_span:Some(hick_lang::SourceSpan::new(0,1,1,0)),source_line:1,source_column:0}]);let index=HashMap::from([(uri_key(uri.as_str()),(uri.clone(),map))]);let key=uri.as_str();
  let edit=json!({"title":"Format","edit":{"changes":{key:[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":1}},"newText":"valid"},{"range":{"start":{"line":900,"character":0},"end":{"line":900,"character":1}},"newText":"invalid"}]}}});
  assert!(bound_project_edit(edit,&index,None).is_err());
 }
}
</ui:copy>

## Exercise the real source-bound transport

The owned controlled provider holds real rename replies while the client changes and restores the origin or renews an affected source lifetime. Another document supplies one successful formatter and one failed formatter; no valid subset may escape. Unknown source acquisition must precede a completely fresh query. Disk sources remain unchanged.

<ui:copy id="bound-source-controlled-provider">
#!/usr/bin/env python3
import json,os,sys,threading,time
EVENTS,GATE=sys.argv[1:3];LOCK=threading.Lock();documents={}
def send(value):
 raw=json.dumps(value).encode()
 with LOCK:sys.stdout.buffer.write(f'Content-Length: {len(raw)}\r\n\r\n'.encode()+raw);sys.stdout.buffer.flush()
def record(value):
 with open(EVENTS,'a') as output:output.write(json.dumps(value)+'\n')
def reply(identity,method,params):
 uri=params.get('textDocument',{}).get('uri','');name=params.get('newName','');record({'method':method,'uri':uri,'name':name})
 if name.startswith('held_'):
  record({'held':name})
  while not os.path.exists(GATE):time.sleep(.005)
 target=uri.rsplit('/',1)[0]+'/b.py' if name=='held_target' else uri.rsplit('/',1)[0]+'/hidden.py' if name=='discover' else uri
 edit={'range':{'start':{'line':0,'character':0},'end':{'line':0,'character':5}},'newText':name or 'formatted'}
 if method=='textDocument/rename':result={'changes':{target:[edit]}}
 elif method=='textDocument/formatting':
  if uri.endswith('/z-bad.py'):send({'jsonrpc':'2.0','id':identity,'error':{'code':-32603,'message':'Owned formatter failure'}});return
  result=[edit]
  if uri.endswith('/unmapped.py'):result.append({'range':{'start':{'line':900,'character':0},'end':{'line':900,'character':1}},'newText':'must refuse whole operation'})
 else:result=[]
 send({'jsonrpc':'2.0','id':identity,'result':result})
while True:
 headers={}
 while True:
  line=sys.stdin.buffer.readline()
  if not line:sys.exit(0)
  if line in (b'\n',b'\r\n'):break
  key,value=line.decode().split(':',1);headers[key.lower()]=value.strip()
 request=json.loads(sys.stdin.buffer.read(int(headers['content-length'])));method=request.get('method');params=request.get('params',{});identity=request.get('id')
 if method=='textDocument/didOpen':documents[params['textDocument']['uri']]=params['textDocument']['text'];continue
 if method=='exit':sys.exit(0)
 if identity is None:continue
 if method=='initialize':send({'jsonrpc':'2.0','id':identity,'result':{'capabilities':{'textDocumentSync':1,'renameProvider':True,'documentFormattingProvider':True,'documentSymbolProvider':True}}});continue
 if method=='shutdown':send({'jsonrpc':'2.0','id':identity,'result':None});continue
 threading.Thread(target=reply,args=(identity,method,params),daemon=True).start()
</ui:copy>

<ui:file path="scripts/controlled-bound-source-provider.py"><ui:paste select="#bound-source-controlled-provider"/></ui:file>

<ui:copy id="bound-source-protocol-driver">
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFile,writeFile,unlink} from 'node:fs/promises';
const [origin,events,gate,fixturePath]=process.argv.slice(2),fixture=JSON.parse(await readFile(fixturePath,'utf8'));
const address=new URL('/api/ws?doc=workspace',origin);address.protocol='ws:';const socket=new WebSocket(address);socket.binaryType='arraybuffer';let sequence=0;const pending=new Map();
const opened=new Promise((resolve,reject)=>{socket.addEventListener('open',resolve,{once:true});socket.addEventListener('error',reject,{once:true});});
function send(value){const data=new TextEncoder().encode(JSON.stringify({jsonrpc:'2.0',...value})),frame=new Uint8Array(data.length+1);frame[0]=2;frame.set(data,1);socket.send(frame);}
function request(method,params){return new Promise((resolve,reject)=>{const id=++sequence,timer=setTimeout(()=>reject(new Error('Owned query timeout '+method)),20000);pending.set(id,{resolve,reject,timer});send({id,method,params});});}
socket.addEventListener('message',event=>{if(!(event.data instanceof ArrayBuffer))return;const frame=new Uint8Array(event.data);if(frame[0]!==2)return;const value=JSON.parse(new TextDecoder().decode(frame.subarray(1)));const call=pending.get(value.id);if(!call)return;pending.delete(value.id);clearTimeout(call.timer);value.error?call.reject(new Error(value.error.message)):call.resolve(value.result);});
const uri=path=>'hick:///'+path,hash=text=>'sha256:'+createHash('sha256').update(text).digest('hex');
async function capture(path,text=fixture[path]){const ack=await request('hick/captureDocument',{textDocument:{uri:uri(path),languageId:path.endsWith('.md')?'markdown':'python',version:1,text}});assert.equal(ack.current,true);assert.equal(ack.source_hash,hash(text));return {source_hash:ack.source_hash,source_revision:ack.source_revision};}
const rename=(expected,newName='renamed')=>request('hick/rename',{textDocument:{uri:uri('a.py')},position:{line:0,character:1},newName,expected});
const format=(path,expected)=>request('hick/formatting',{textDocument:{uri:uri(path)},options:{tabSize:4,insertSpaces:true},expected});
function proofs(action){assert.equal(action.source_files_modified,false);assert.ok(action.data.hickory_action);for(const row of action.data.hickory_sources){const path=decodeURIComponent(new URL(row.uri).pathname.slice(1));assert.equal(row.source_hash,hash(fixture[path]),row.uri);}}
async function held(name,operation,mutate){const answer=operation().then(value=>({value}),error=>({error}));let started=false;for(let n=0;n<300;n++){if((await readFile(events,'utf8').catch(()=>'' )).split('\n').filter(Boolean).map(JSON.parse).some(row=>row.held===name)){started=true;break;}await new Promise(resolve=>setTimeout(resolve,10));}assert.ok(started);await mutate();await writeFile(gate,'release');const value=await answer;await unlink(gate);assert.match(value.error?.message??'',/changed|preparing|ownership/i);}
try{
 await opened;let a=await capture('a.py');await capture('b.py');const good=await rename(a);proofs(good);assert.equal(good.edit.documentChanges[0].edits[0].newText,'renamed');const restored=await request('codeAction/resolve',{...good,title:'forged',edit:{changes:{}},data:{...good.data,hickory_sources:[]}});assert.equal(restored.title,'Rename');assert.equal(restored.edit.documentChanges[0].edits[0].newText,'renamed');
 const stale=a;a=await capture('a.py');await assert.rejects(rename(stale),/capture changed/i);
 await held('held_origin',()=>rename(a,'held_origin'),async()=>{await capture('a.py',fixture['a.py']+'# edit\n');a=await capture('a.py');});
 await held('held_target',()=>rename(a,'held_target'),async()=>{await capture('b.py');});
 const discovery=await rename(a,'discover');assert.equal(discovery.edit,undefined);assert.ok(discovery.data.hickory_acquire.length);await capture('hidden.py');const acquired=await rename(a,'discover');proofs(acquired);assert.equal(acquired.edit.documentChanges[0].textDocument.uri,uri('hidden.py'));
 const formatted=await format('a.py',a);proofs(formatted);assert.equal(formatted.edit.documentChanges[0].edits[0].newText,'formatted');
 await assert.rejects(format('a.py',stale),/capture changed/i);const document=await capture('document.md');await assert.rejects(format('document.md',document),/formatter failure/i);const calls=(await readFile(events,'utf8')).split('\n').filter(Boolean).map(JSON.parse).filter(row=>row.method==='textDocument/formatting');assert.ok(calls.findIndex(row=>row.uri.endsWith('/a-good.py'))<calls.findIndex(row=>row.uri.endsWith('/z-bad.py')),'Successful formatter must precede whole-operation failure');const unmapped=await capture('unmapped.md');await assert.rejects(format('unmapped.md',unmapped),/cannot be projected|partial|map/i);
 console.log('PASS: exact-capture rename/format, immutable synthetic resolve, pre-dispatch ABA refusal, held origin edit/Undo and affected-source lifetime refusal, unknown-source acquisition/requery, full formatting provider failure and unmappable-subset refusal. No source writes.');
}finally{for(const call of pending.values()){clearTimeout(call.timer);call.reject(new Error('Owned client closed'));}socket.close();}
</ui:copy>

<ui:file path="scripts/bound-source-edits-driver.mjs"><ui:paste select="#bound-source-protocol-driver"/></ui:file>

<ui:copy id="bound-source-protocol-runner">
#!/usr/bin/env python3
import importlib.util,json,os,subprocess,tempfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
SOURCE={'a.py':'value = 1\n','b.py':'value = 2\n','hidden.py':'value = 3\n','document.md':'\x3chick:doc xmlns:hick="http://www.hickorydocs.com/1.0">\n\x3chick:file path="a-good.py">value = 1\n\x3c/hick:file>\n\x3chick:file path="z-bad.py">value = 2\n\x3c/hick:file>\n\x3c/hick:doc>\n','unmapped.md':'\x3chick:doc xmlns:hick="http://www.hickorydocs.com/1.0">\n\x3chick:file path="unmapped.py">value = 3\n\x3c/hick:file>\n\x3c/hick:doc>\n'}
def main():
 spec=importlib.util.spec_from_file_location('fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 with tempfile.TemporaryDirectory(prefix='hickory-bound-edits-') as temporary:
  folder=Path(temporary);root=folder/'workspace';root.mkdir();events=folder/'events';gate=folder/'gate';source=folder/'source.json';source.write_text(json.dumps(SOURCE))
  for path,text in SOURCE.items():(root/path).write_text(text)
  (root/'.hick-lsp.json').write_text(json.dumps({'servers':{'python':{'command':[os.sys.executable,str(ROOT/'scripts/controlled-bound-source-provider.py'),str(events),str(gate)]}}}))
  engine=fixture.start([str(ROOT/'target/source-edit-focus/debug/examples/engine_client'),str(root),'window','http://127.0.0.1:1'],env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state')})
  try:
   subprocess.run(['node',str(ROOT/'scripts/bound-source-edits-driver.mjs'),fixture.origin(engine),str(events),str(gate),str(source)],check=True,timeout=120)
   for path,text in SOURCE.items():assert (root/path).read_text()==text,path
   print('PASS: all authored fixture disk bytes retained. Controlled provider protocol evidence does not certify language semantics or physical/accessibility acceptance.')
  finally:fixture.stop(engine)
if __name__=='__main__':main()
</ui:copy>

<ui:file path="scripts/test-bound-source-edits.py"><ui:paste select="#bound-source-protocol-runner"/></ui:file>

</ui:doc>
