<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Witness type navigation without adding editor chrome

For maintainers changing retained provider protocols or simple navigation adapters.

## Keep original values through the actual protocol

Ask two child languages before following the older item. Caller substitutions must not alter the child request. Follow a newly returned token, preserve original child tag arrays through follow-up payloads, and refuse foreign/stale tokens and a held response after source changes.

<ui:copy id="type-hierarchy-protocol-driver">
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
</ui:copy>

<ui:file path="scripts/type-hierarchy-driver.mjs"><ui:paste select="#type-hierarchy-protocol-driver"/></ui:file>

## Run inside an isolated actual engine

Reuse the controlled-provider engine fixture with its own workspace and event/gate files. Type queries perform no action resolutions and must leave every file unchanged.

<ui:copy id="type-hierarchy-protocol-runner">
"""Run the type-hierarchy protocol against an isolated real engine."""
import importlib.util
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location("provider_fixture",ROOT/"scripts/test-provider-actions.py")
fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
if __name__=="__main__":fixture.run_provider_witness("type-hierarchy-driver.mjs",())
</ui:copy>

<ui:file path="scripts/test-type-hierarchy.py"><ui:paste select="#type-hierarchy-protocol-runner"/></ui:file>

## Follow the existing menus in a retained editor

The browser uses the actual engine and controlled child through the existing Edit menu and floating semantic results. Navigate to the precise type-name occurrence while preserving an unsaved draft, editor identity, dimensions and disk. This establishes UI/protocol behavior rather than language analysis completeness.

<ui:copy id="type-hierarchy-browser-witness">
"""Existing simple-menu type navigation against a controlled child, real engine."""
import importlib.util
import json
import os
from pathlib import Path
import socket
import sys
import tempfile
import time
ROOT=Path(__file__).resolve().parents[1]
def main():
    spec=importlib.util.spec_from_file_location("simple_fixture",ROOT/"scripts/test-simple-browser.py")
    fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
    with tempfile.TemporaryDirectory(prefix="hickory-type-navigation-") as temporary:
        folder=Path(temporary);root=folder/"workspace";root.mkdir();path="a.py";source="value = 1\n";(root/path).write_text(source);events=folder/"events.jsonl"
        (root/".hick-lsp.json").write_text(json.dumps({"servers":{"python":{"command":[sys.executable,str(ROOT/"scripts/controlled-action-provider.py"),"python",str(events)]}}}))
        environment={**os.environ,"HICKORY_STATE_DIR":str(folder/"state")};processes=[];driver=None
        try:
            with socket.socket() as reservation:reservation.bind(("127.0.0.1",0));port=reservation.getsockname()[1]
            ui=f"http://127.0.0.1:{port}"
            engine=fixture.start([os.environ.get("HICKORY_SIMPLE_TEST_ENGINE",str(ROOT/"target/source-edit-focus/debug/examples/engine_client")),str(root),"window",ui],env=environment);processes.append(engine);backend=fixture.origin(engine)
            vite=fixture.start(["node","node_modules/vite/bin/vite.js","--config","vite.simple.config.ts","--host","127.0.0.1","--port",str(port),"--strictPort"],cwd=ROOT/"apps/web",env={**environment,"HICKORY_API_ORIGIN":backend});processes.append(vite)
            driver=fixture.BrowserDriver(fixture.origin(vite)+"/simple.html");driver.act("open",path=path);driver.act("source-tests-edit",path=path,text="# retained draft\n")
            draft=driver.act("search-source",path=path);driver.act("remember-editor",path=path);size=driver.act("editor-size",path=path)
            deadline=time.monotonic()+20
            while True:
                result=driver.act("semantic-query",path=path,offset=1,query="Find supertypes")
                if "pythonBase" in result["text"]:break
                driver.act("semantic-close");assert time.monotonic()<deadline,result;time.sleep(.1)
            assert result["size"]==size,result;driver.act("semantic-open",path=path,label="pythonBase · a.py:1:1");assert driver.act("caret-offset",path=path)==0
            result=driver.act("semantic-query",path=path,offset=1,query="Find subtypes");assert "pythonDerived" in result["text"],result;assert result["size"]==size;driver.act("semantic-close")
            assert driver.act("search-source",path=path)==draft;assert driver.act("editor-is-retained",path=path);assert driver.act("editor-size",path=path)==size;assert (root/path).read_text()==source
            observed=[json.loads(line) for line in events.read_text().splitlines()];assert {item.get("type") for item in observed}>={"typeHierarchy/supertypes","typeHierarchy/subtypes"},observed
            print("PASS: existing type-navigation menus, actual provider follow-ups, precise name navigation, retained draft/editor/geometry and unchanged disk")
        finally:
            if driver:driver.close()
            for process in reversed(processes):fixture.stop(process)
if __name__=="__main__":main()
</ui:copy>

<ui:file path="scripts/test-type-navigation-browser.py"><ui:paste select="#type-hierarchy-browser-witness"/></ui:file>

</ui:doc>
