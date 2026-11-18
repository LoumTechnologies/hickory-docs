from pathlib import Path
import argparse,importlib.util,json,os,subprocess,tempfile
NODE = r"""
import fs from 'node:fs';
const [origin,root,casesFile,receipt]=process.argv.slice(2);const cases=JSON.parse(fs.readFileSync(casesFile));
async function post(path,body){const response=await fetch(origin+'/api/'+path,{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(body)});const text=await response.text();if(!response.ok)throw Error(path+': '+response.status+' '+text);return JSON.parse(text)}
const socket=new WebSocket(origin.replace('http:','ws:')+'/api/ws?doc=workspace');socket.binaryType='arraybuffer';const rows=[],arrival=[];let socketError;socket.addEventListener('message',event=>{const bytes=new Uint8Array(event.data);if(bytes[0]===3){const row=JSON.parse(new TextDecoder().decode(bytes.slice(1)));rows.push(row);arrival.push(performance.now());console.log(JSON.stringify(row));}});socket.addEventListener('error',()=>{socketError='Debug socket failed'});await new Promise((resolve,reject)=>{socket.addEventListener('open',resolve,{once:true});socket.addEventListener('error',reject,{once:true})});
function send(value){const json=new TextEncoder().encode(JSON.stringify(value));const frame=new Uint8Array(json.length+1);frame[0]=3;frame.set(json,1);socket.send(frame)}
async function wait(test,label){const end=Date.now()+120000;while(Date.now()<end){if(socketError)throw Error(socketError);const value=test();if(value)return value;await new Promise(resolve=>setTimeout(resolve,20))}throw Error(label+': '+JSON.stringify(rows.slice(-5)))}
const outcomes=[];
try{for(const [index,fixture]of cases.entries()){
 const text=fs.readFileSync(root+'/'+fixture.path,'utf8');const analysis={path:fixture.path,source:text,project_entry:null,sources:{},capture:false};const scan=await post('tests/source-candidates',analysis);const candidate=scan.candidates.find(c=>c.name==='test_selected'&&(!fixture.negative||c.target.path.endsWith('selected.rs')));if(!candidate)throw Error('Candidate absent: '+JSON.stringify(scan));const request={version:1,input:{kind:'saved'},analysis,fingerprint:scan.fingerprint,candidate_id:candidate.id,selection:{kind:'name_filter'}};const plan=await post('tests/debug-plan',request);if(!plan.available)throw Error('Plan unavailable: '+JSON.stringify(plan));const line=text.split('\n').findIndex(line=>line.includes('owned-proof-stop'));const id='proof-'+index;const launch={op:'start_test',doc:plan.source_uri,source_hash:plan.source_hash,request_id:id,source_test:request,breakpoints:[{line}]};
 if(fixture.slow){
  const wrong=id+'-stale';const staleStart=rows.length;
  send({...launch,request_id:wrong,source_hash:'0'.repeat(64)});
  await wait(()=>rows.slice(staleStart).find(r=>r.event==='failed'&&r.request_id===wrong),'Changed source hash did not refuse');
  if(rows.slice(staleStart).some(r=>r.request_id===wrong&&r.event!=='failed'))throw Error('Wrong source received preparation authority');
 }
 const start=rows.length,sentAt=performance.now();
 send(launch);
 const preparing=await wait(()=>{
  const receipt=rows.slice(start).find(r=>r.event==='test_preparing'&&r.request_id===id);
  if(receipt)return receipt;
  const failed=rows.slice(start).find(r=>r.event==='failed'&&r.request_id===id);
  if(failed)throw Error('Failed before saved preparation: '+JSON.stringify(failed));
 },'Missing bounded preparation receipt');
 if(preparing.doc!==plan.source_uri||preparing.wait_ms!==180_000||'session' in preparing||
    preparing.source_test.candidate_id!==candidate.id||preparing.source_test.input_hash!==plan.input_hash||
    preparing.source_test.analysis_fingerprint!==scan.fingerprint)throw Error('Preparation receipt changed saved Start authority');
 const preparingMs=arrival[rows.indexOf(preparing)]-sentAt;
 if(fixture.negative){const failed=await wait(()=>rows.slice(start).find(r=>r.event==='failed'),'Disabled declaration did not refuse');if(rows.slice(start).some(r=>r.event==='started'))throw Error('Disabled declaration started sibling');if(!JSON.stringify(failed).includes('no proven active module/target identity'))throw Error('Wrong refusal: '+JSON.stringify(failed));outcomes.push({path:fixture.path,outcome:'refused-disabled-source-before-session'});}
 else{const stop=await wait(()=>{const failed=rows.slice(start).find(r=>r.event==='failed');if(failed)throw Error(JSON.stringify(failed));return rows.slice(start).find(r=>r.event==='stopped')},'Selected source did not pause');if(stop.line!==line)throw Error('Wrong source line '+JSON.stringify(stop));const admitted=rows.slice(start).find(r=>r.event==='started'&&r.request_id===id);const session=admitted?.session;if(!session)throw Error('Missing started receipt');
 const startedMs=arrival[rows.indexOf(admitted)]-sentAt;
 const launchEvents=rows.slice(start).filter(r=>r.request_id===id);
 const prefix=launchEvents.filter(r=>['test_preparing','build','started'].includes(r.event));
 if(prefix.map(r=>r.event).join(',')!=='test_preparing,build,started'||!prefix[1].events?.length||
    admitted.breakpoints?.length!==1||admitted.breakpoints[0].line!==line||
    JSON.stringify(preparing.source_test)!==JSON.stringify(admitted.source_test))throw Error('Startup receipts were dropped, replayed or changed: '+JSON.stringify(prefix));
 if(fixture.slow&&(preparingMs>=30_000||startedMs<=30_000))throw Error('Slow fixture did not cross the former client deadline: '+JSON.stringify({preparingMs,startedMs}));
 const clearIndex=rows.length;send({op:'breakpoints',session,source_hash:plan.source_hash,request_id:id+'-clear',breakpoints:[]});await wait(()=>rows.slice(clearIndex).find(r=>r.event==='breakpoints'),'Mark did not clear');send({op:'step',session,how:'continue',stop:stop.stop,thread:stop.thread,request_id:id+'-finish'});const finish=await wait(()=>rows.slice(start).find(r=>r.event==='finished'&&r.session===session),'Selected test did not finish');if(finish.exit_code!==0)throw Error('Wrong process exit '+JSON.stringify(finish));outcomes.push({path:fixture.path,outcome:'selected-source-paused-exit0',preparing_ms:preparingMs,started_ms:startedMs});}
 if(fs.existsSync(root+'/wrong-sibling-ran'))throw Error('Sibling body executed');if(fs.readFileSync(root+'/'+fixture.path,'utf8')!==text)throw Error('Authored source changed');
 }
 fs.writeFileSync(receipt,JSON.stringify({outcomes,events:rows},null,2));console.log('PASS direct Rust source association '+outcomes.length+' physical/authored inline/external/disabled flows');
}finally{socket.close()}
"""
parser=argparse.ArgumentParser()
parser.add_argument('--slow',action='store_true',help='Cross the old native deadline with one real Cargo build')
args=parser.parse_args()
ROOT=Path(__file__).resolve().parents[1];spec=importlib.util.spec_from_file_location('fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
with tempfile.TemporaryDirectory(prefix='rust-source-proof-')as temp:
 folder=Path(temp);root=folder/'workspace';root.mkdir();(root/'.git').mkdir();manifest='[workspace]\n[package]\nname="owned-source-proof"\nversion="0.1.0"\nedition="2024"\n';cases=[]
 for kind in ['inline','external','disabled']:
  selected='#[test]\nfn test_selected(){\n let selected_value=41;\n assert_eq!(selected_value,41); // owned-proof-stop\n}\n';sibling='#[test]fn test_selected(){std::fs::write('+json.dumps(str(root/'wrong-sibling-ran'))+',b"wrong").unwrap();}\n'
  if kind=='inline':files={'Cargo.toml':manifest,'src/lib.rs':'mod selected {\n'+selected+'}\nmod sibling {\n'+sibling+'}\n'};target='src/lib.rs'
  else:files={'Cargo.toml':manifest,'src/lib.rs':('#[cfg(any())]'if kind=='disabled'else'')+'mod selected;\nmod sibling;\n','src/selected.rs':selected,'src/sibling.rs':sibling};target='src/selected.rs'
  for path,text in files.items():file=root/kind/path;file.parent.mkdir(parents=True,exist_ok=True);file.write_text(text)
  cases.append({'path':kind+'/'+target,'negative':kind=='disabled'});book='<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\n'+''.join('<h:file path="'+path+'">\n'+text+'</h:file>\n'for path,text in files.items())+'</h:doc>\n';name=kind+'.md';(root/name).write_text(book);cases.append({'path':name,'negative':kind=='disabled'})
 if args.slow:
  (root/'inline/build.rs').write_text('fn main() { std::thread::sleep(std::time::Duration::from_secs(32)); }\n')
  cases=[{**cases[0],'slow':True}]
 receipt_name='slow-protocol-receipt.json' if args.slow else 'protocol-receipt.json'
 file=folder/'cases.json';file.write_text(json.dumps(cases));script=folder/'protocol.mjs';script.write_text(NODE);receipt=ROOT/'target/rust-test-debug/source-proof'/receipt_name;receipt.parent.mkdir(parents=True,exist_ok=True);env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state'),'DEVELOPER_DIR':'/Applications/Xcode-27.0.0.app/Contents/Developer'};engine=fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(root),'window','http://127.0.0.1:43981'],env=env)
 try:
  origin=fixture.origin(engine);result=subprocess.run(['node',str(script),origin,str(root),str(file),str(receipt)],env=env);raise SystemExit(result.returncode)
 finally:fixture.stop(engine)
