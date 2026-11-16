<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Witness retained values across provider boundaries

For maintainers changing protocol routing and source-bound provider artifacts.

These controlled child providers test protocol behavior, not Python or TypeScript analysis. An isolated real engine starts both configured children. Open both documents before obtaining actions, resolve the older Python action after the TypeScript query, and require the exact original opaque payload. Caller-supplied title/command substitutions never reach a child. A later source edit retires the old token; another edit while resolution is held must reject its late answer, and neither resolution nor refusal changes disk. No provider command is executed.

## Frame requests and retain independent child records

Speak actual Content-Length LSP framing and retain separate original values for actions, types and completions. Capability declarations distinguish lazy Python from eager TypeScript behavior; event and gate files live outside the source workspace.

<ui:copy id="controlled-action-provider">
"""Controlled child LSP for routing/opaque-action tests, not language semantics."""
import json
import os
import sys
import threading
import time
language,events=sys.argv[1:3]
gate=sys.argv[3] if len(sys.argv)>3 else events+".release"
output_lock=threading.Lock()
documents={}
issued={}
issued_types={}
issued_completions={}
def send(value):
    raw=json.dumps(value,separators=(",",":")).encode()
    with output_lock:sys.stdout.buffer.write(f"Content-Length: {len(raw)}\r\n\r\n".encode()+raw);sys.stdout.buffer.flush()
def event(value):
    descriptor=os.open(events,os.O_WRONLY|os.O_CREAT|os.O_APPEND,0o600)
    try:os.write(descriptor,(json.dumps(value)+"\n").encode())
    finally:os.close(descriptor)
while True:
    headers={}
    while True:
        line=sys.stdin.buffer.readline()
        if not line:sys.exit(0)
        if line in (b"\r\n",b"\n"):break
        key,value=line.decode().split(":",1);headers[key.lower()]=value.strip()
    message=json.loads(sys.stdin.buffer.read(int(headers["content-length"])));method=message.get("method");params=message.get("params",{});identity=message.get("id")
    if method=="textDocument/didOpen":documents[params["textDocument"]["uri"]]=params["textDocument"]["text"];continue
    if method=="textDocument/didChange":documents[params["textDocument"]["uri"]]=params["contentChanges"][-1]["text"];continue
    if method=="exit":sys.exit(0)
    if identity is None:continue
    if method=="initialize":result={"capabilities":{"textDocumentSync":1,"documentSymbolProvider":True,"typeHierarchyProvider":True,"completionProvider":{"resolveProvider":language=="python","triggerCharacters":["."]},"codeActionProvider":{"resolveProvider":language=="python"}}}
</ui:copy>

## Witness standards-expanded completion items

Supply list defaults with opaque, Unicode and range-shaped data. A lazy resolver requires the complete defaults-expanded original and returns primary plus import edits; a held reply allows source to change before completion.

<ui:copy id="controlled-provider-completion-cases">
    elif method=="textDocument/completion":
        uri=params["textDocument"]["uri"];opaque={"producer":language,"uri":uri,"range":{"start":{"line":91,"character":2},"end":{"line":91,"character":4}},"unicode":"é😀"}
        if params["position"]["character"]==2:opaque["slow"]=True
        edit_range={"start":{"line":0,"character":0},"end":{"line":0,"character":5}}
        item={"label":language+" completion","filterText":"value","textEditText":language+"_initial"}
        original={**item,"data":opaque,"insertTextFormat":1,"textEdit":{"range":edit_range,"newText":item["textEditText"]}}
        issued_completions[uri]=original
        result={"isIncomplete":False,"itemDefaults":{"editRange":edit_range,"data":opaque,"insertTextFormat":1},"items":[item]}
        event({"completion_query":language,"context":params.get("context")})
    elif method=="completionItem/resolve":
        uri=params.get("data",{}).get("uri")
        if language!="python" or params!=issued_completions.get(uri):
            event({"error":"altered completion or wrong provider","language":language});send({"jsonrpc":"2.0","id":identity,"error":{"code":-32602,"message":"Original completion payload was not preserved"}});continue
        event({"completion_resolved":language,"slow_completion":params["data"].get("slow",False)})
        result={**params,"textEdit":{**params["textEdit"],"newText":"python_completed"},"additionalTextEdits":[{"range":{"start":{"line":1,"character":0},"end":{"line":1,"character":0}},"newText":"# imported\n"}]}
        if uri.endswith("/snippet.py"):result.update(insertTextFormat=2,textEdit={**params["textEdit"],"newText":"${1:😀 value} + $1$0"})
        if uri.endswith("/nested.py"):result.update(insertTextFormat=2,textEdit={**params["textEdit"],"newText":"${1:value} + $1$0"})
        if params["data"].get("slow"):
            def release_completion(identity=identity,result=result):
                deadline=time.monotonic()+10
                while not os.path.exists(gate) and time.monotonic()<deadline:time.sleep(.01)
                if os.path.exists(gate):send({"jsonrpc":"2.0","id":identity,"result":result});event({"completion_released":language})
                else:send({"jsonrpc":"2.0","id":identity,"error":{"code":-32603,"message":"Controlled completion gate timed out"}})
            threading.Thread(target=release_completion,daemon=True).start();continue
</ui:copy>

## Witness opaque type follow-ups

Require exact original type items and the producing language, retain returned items for another follow-up, and hold a requested answer until its source can be changed.

<ui:copy id="controlled-provider-type-cases">
    elif method=="textDocument/prepareTypeHierarchy":
        uri=params["textDocument"]["uri"]
        item={"name":language+"Type","kind":5,"uri":uri,"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":5}},"selectionRange":{"start":{"line":0,"character":0},"end":{"line":0,"character":5}},"tags":[1],"data":{"producer":language,"uri":uri,"nested":["opaque",{"id":17}]}}
        if params["position"]["character"]==2:item["data"]["slow"]=True
        issued_types[(uri,item["name"])]=item;result=[item]
    elif method in ("typeHierarchy/supertypes","typeHierarchy/subtypes"):
        item=params.get("item",{});uri=item.get("uri")
        if item!=issued_types.get((uri,item.get("name"))) or item.get("data",{}).get("producer")!=language:
            event({"error":"altered type item or wrong provider","language":language});send({"jsonrpc":"2.0","id":identity,"error":{"code":-32602,"message":"Original type payload was not preserved"}});continue
        event({"type":method,"language":language,"slow_type":item["data"].get("slow",False)})
        answer={**item,"name":language+("Base" if method.endswith("supertypes") else "Derived"),"data":{"producer":language,"uri":uri,"nested":["returned",{"id":23}]}}
        issued_types[(uri,answer["name"])]=answer;result=[answer]
        if item["data"].get("slow"):
            def release_type(identity=identity,result=result):
                deadline=time.monotonic()+10
                while not os.path.exists(gate) and time.monotonic()<deadline:time.sleep(.01)
                if os.path.exists(gate):send({"jsonrpc":"2.0","id":identity,"result":result})
                else:send({"jsonrpc":"2.0","id":identity,"error":{"code":-32603,"message":"Controlled type gate timed out"}})
            threading.Thread(target=release_type,daemon=True).start();continue
</ui:copy>

## Witness lazy and eager action values

Require exact original actions and producing provider identity, return complete edits and keep a deliberately held action independent of later protocol requests.

<ui:copy id="controlled-provider-action-cases">
    elif method=="textDocument/documentSymbol":result=[]
    elif method=="textDocument/codeAction":
        uri=params["textDocument"]["uri"];opaque={"producer":language,"hickory_acquire":[{"uri":"file:///untrusted-child-path"}],"uri":uri,"nested":[1,{"uri":"file:///opaque-child-value"}]}
        if params["range"]["start"]["character"]==2:opaque["slow"]=True
        result=[{"title":f"Controlled {language} fix","kind":"quickfix","data":opaque}]
        if language!="python":result[0]["edit"]={"changes":{uri:[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":5}},"newText":language+"_fixed"}]}}
        issued[uri]=result[0]
    elif method=="codeAction/resolve":
        opaque=params.get("data",{});uri=opaque.get("uri");expected=issued.get(uri)
        if params!=expected or opaque.get("producer")!=language:
            event({"error":"altered opaque action or wrong provider","language":language,"received":params});send({"jsonrpc":"2.0","id":identity,"error":{"code":-32602,"message":"Original provider payload was not preserved"}});continue
        event({"resolved":language,"uri":uri,"slow":opaque.get("slow",False)})
        result={**params,"edit":{"changes":{uri:[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":5}},"newText":language+"_fixed"}]}}}
        if os.environ.get("HICKORY_CONTROLLED_ACTION_OTHER"):
            other=uri.rsplit("/",1)[0]+"/other.py"
            result["edit"]["changes"][other]=[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":5}},"newText":"other_fixed"}]
        if os.environ.get("HICKORY_CONTROLLED_ACTION_RESOURCES"):
            base=uri.rsplit("/",1)[0]
            result["edit"]={"documentChanges":[
                {"textDocument":{"uri":uri,"version":91},"edits":[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":5}},"newText":"python_fixed"}]},
                {"kind":"create","uri":base+"/created.py"},
                {"textDocument":{"uri":base+"/created.py","version":None},"edits":[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":0}},"newText":"created = 1\n"}]},
                {"kind":"rename","oldUri":base+"/other.py","newUri":base+"/moved.py"},
                {"textDocument":{"uri":base+"/moved.py","version":None},"edits":[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":5}},"newText":"other_fixed"}]},
                {"kind":"delete","uri":base+"/delete.py"}
            ]}
        if opaque.get("slow"):
            def release(identity=identity,result=result):
                deadline=time.monotonic()+10
                while not os.path.exists(gate) and time.monotonic()<deadline:time.sleep(.01)
                if not os.path.exists(gate):send({"jsonrpc":"2.0","id":identity,"error":{"code":-32603,"message":"Controlled gate timed out"}})
                else:send({"jsonrpc":"2.0","id":identity,"result":result})
            threading.Thread(target=release,daemon=True).start();continue
</ui:copy>

## Complete the framed response

Unimplemented optional queries return null. Every ordinary result uses the request ID and Content-Length response writer; deliberately held replies use that same writer independently.

<ui:copy id="controlled-provider-response">
    else:result=None
    send({"jsonrpc":"2.0","id":identity,"result":result})
</ui:copy>

<ui:file path="scripts/controlled-action-provider.py"><ui:paste select="#controlled-action-provider"/><ui:paste select="#controlled-provider-completion-cases"/><ui:paste select="#controlled-provider-type-cases"/><ui:paste select="#controlled-provider-action-cases"/><ui:paste select="#controlled-provider-response"/></ui:file>

## provider-action-protocol-driver

Use the existing binary LSP WebSocket channel against the real engine. Match request IDs, reject server requests rather than executing them, and check complete authored edit projection plus stale and foreign token refusals. A controlled gate holds one resolver while another request synchronizes changed source, proving that the late result cannot attach.

<ui:copy id="provider-action-protocol-driver">
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
 const resolved=await request('codeAction/resolve',changed);assert.equal(resolved.title,'Controlled python fix');assert.equal(resolved.command,undefined);assert.deepEqual(Object.keys(resolved.data),['hickory_action']);assert.equal(resolved.edit.documentChanges[0].textDocument.uri,a);assert.equal(resolved.edit.documentChanges[0].edits[0].newText,'python_fixed');
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
</ui:copy>

<ui:file path="scripts/provider-actions-driver.mjs"><ui:paste select="#provider-action-protocol-driver"/></ui:file>

## provider-action-isolated-engine-witness

Give each run a disposable workspace and process-owned server configuration. The metrics must contain exactly two Python resolutions, one held across a source edit; the eager TypeScript action must use its held complete value without invoking an unsupported resolver; source files must retain their original bytes.

<ui:copy id="provider-action-isolated-engine-witness">
"""Isolated-engine protocol witness with controlled child providers."""
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
ROOT=Path(__file__).resolve().parents[1]
def run_provider_witness(driver="provider-actions-driver.mjs",expected_actions=("python","python")):
    spec=importlib.util.spec_from_file_location("simple_fixture",ROOT/"scripts/test-simple-browser.py")
    fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
    with tempfile.TemporaryDirectory(prefix="hickory-provider-actions-") as temporary:
        folder=Path(temporary);root=folder/"workspace";root.mkdir();events=folder/"events.jsonl";gate=folder/"release"
        sources={"a.py":"value = 1\n","b.ts":"value = 2;\n"}
        for path,text in sources.items():(root/path).write_text(text)
        provider=ROOT/"scripts/controlled-action-provider.py"
        (root/".hick-lsp.json").write_text(json.dumps({"servers":{language:{"command":[sys.executable,str(provider),language,str(events),str(gate)]} for language in ["python","typescript"]}}))
        engine=fixture.start([os.environ.get("HICKORY_SIMPLE_TEST_ENGINE",str(ROOT/"target/source-edit-focus/debug/examples/engine_client")),str(root),"window","http://127.0.0.1:1"],env={**os.environ,"HICKORY_STATE_DIR":str(folder/"state")})
        try:
            origin=fixture.origin(engine);subprocess.run(["node",str(ROOT/"scripts"/driver),origin,str(events),str(gate)],check=True,timeout=90)
            observed=[json.loads(line) for line in events.read_text().splitlines()];assert [item["resolved"] for item in observed if "resolved" in item]==list(expected_actions),observed
            for path,text in sources.items():assert (root/path).read_text()==text,"Resolution wrote source"
        finally:fixture.stop(engine)
if __name__=="__main__":run_provider_witness()
</ui:copy>

<ui:file path="scripts/test-provider-actions.py"><ui:paste select="#provider-action-isolated-engine-witness"/></ui:file>

</ui:doc>
