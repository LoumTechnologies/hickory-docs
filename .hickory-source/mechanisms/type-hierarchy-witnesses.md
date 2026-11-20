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

Use the actual engine for both browser paths. The default controlled child
supplies predictable items and records the follow-ups. The optional
`--clangd` fixture opens a C++ translation unit whose superclass is declared
in an initially closed header. Its C++ hierarchy comes from installed clangd.

<ui:copy id="type-hierarchy-browser-witness">
"""Read type relationships through the Simple Browser menus and real engine."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import shutil
import socket
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]


def prepare_workspace(root, folder, clangd):
    if clangd:
        if not shutil.which("clangd"):
            raise RuntimeError("The genuine hierarchy witness requires installed clangd.")
        (root / ".git").mkdir()
        return {
            "hierarchy.cpp": '#include "base.hpp"\nstruct Mid : Root {};\nstruct Leaf : Mid {};\n',
            "base.hpp": "struct Root {};\n",
        }

    events = folder / "events.jsonl"
    command = [sys.executable, str(ROOT / "scripts/controlled-action-provider.py"), "python", str(events)]
    (root / ".hick-lsp.json").write_text(json.dumps({"servers": {"python": {"command": command}}}))
    return {"a.py": "value = 1\n"}
</ui:copy>

Wait for the named relationship through the existing Edit command and select
its exact source name. Returning from the header must recover the original
editor and its caret. A second query follows the subtype while the original
unsaved draft and both saved files retain their bytes.

<ui:copy id="type-hierarchy-browser-reading">


def query_relationship(driver, path, offset, size, query, expected):
    deadline = time.monotonic() + 20
    while True:
        result = driver.act("semantic-query", path=path, offset=offset, query=query)
        if expected in result["text"]:
            assert result["size"] == size, result
            return
        driver.act("semantic-close")
        assert time.monotonic() < deadline, result
        time.sleep(.1)


def exercise_relationships(driver, fixture, clangd, sources):
    path = "hierarchy.cpp" if clangd else "a.py"
    source = sources[path]
    offset = source.index("Mid") if clangd else 1
    driver.act("open", path=path)
    change = "// retained draft\n" if clangd else "# retained draft\n"
    driver.act("source-tests-edit", path=path, text=change)
    draft = driver.act("search-source", path=path)
    assert draft == source + change, draft
    driver.act("remember-editor", path=path)
    size = driver.act("editor-size", path=path)

    base = "Root · base.hpp:1:8" if clangd else "pythonBase · a.py:1:1"
    query_relationship(driver, path, offset, size, "Find supertypes", base)
    base_path = "base.hpp" if clangd else path
    driver.act("semantic-open", path=base_path, label=base)
    base_offset = sources[base_path].index("Root") if clangd else 0
    fixture.eventually(lambda: driver.act("caret-offset", path=base_path), base_offset)
    if clangd:
        driver.act("navigate-history", direction="back", path=path)
        fixture.eventually(lambda: driver.act("caret-offset", path=path), offset)

    derived = "Leaf · hierarchy.cpp:3:8" if clangd else "pythonDerived"
    query_relationship(driver, path, offset, size, "Find subtypes", derived)
    if clangd:
        driver.act("semantic-open", path=path, label=derived)
        fixture.eventually(lambda: driver.act("caret-offset", path=path), source.index("Leaf"))
    else:
        driver.act("semantic-close")

    assert driver.act("editor-is-retained", path=path)
    assert driver.act("editor-size", path=path) == size
    assert driver.act("search-source", path=path) == draft
</ui:copy>

The engine and browser each run within the disposable fixture. Compare the
saved files after both reads. The controlled case also checks that the real
engine sent each follow-up to the child.

<ui:copy id="type-hierarchy-browser-run">


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--clangd", action="store_true", help="Use installed clangd for C++ relationships")
    args = parser.parse_args()
    spec = importlib.util.spec_from_file_location("simple_fixture", ROOT / "scripts/test-simple-browser.py")
    fixture = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(fixture)
    with tempfile.TemporaryDirectory(prefix="hickory-type-navigation-") as temporary:
        folder = Path(temporary)
        root = folder / "workspace"
        root.mkdir()
        sources = prepare_workspace(root, folder, args.clangd)
        for path, source in sources.items():
            (root / path).write_text(source)
        environment = {**os.environ, "HICKORY_STATE_DIR": str(folder / "state")}
        processes = []
        driver = None
        try:
            with socket.socket() as reservation:
                reservation.bind(("127.0.0.1", 0))
                port = reservation.getsockname()[1]
            ui = f"http://127.0.0.1:{port}"
            engine_path = os.environ.get("HICKORY_SIMPLE_TEST_ENGINE", str(ROOT / "target/source-edit-focus/debug/examples/engine_client"))
            engine = fixture.start([engine_path, str(root), "window", ui], env=environment)
            processes.append(engine)
            vite = fixture.start(
                ["node", "node_modules/vite/bin/vite.js", "--config", "vite.simple.config.ts",
                 "--host", "127.0.0.1", "--port", str(port), "--strictPort"],
                cwd=ROOT / "apps/web", env={**environment, "HICKORY_API_ORIGIN": fixture.origin(engine)},
            )
            processes.append(vite)
            driver = fixture.BrowserDriver(fixture.origin(vite) + "/simple.html")
            exercise_relationships(driver, fixture, args.clangd, sources)
            for path, source in sources.items():
                assert (root / path).read_bytes() == source.encode()
            if not args.clangd:
                events = [json.loads(line) for line in (folder / "events.jsonl").read_text().splitlines()]
                assert {item.get("type") for item in events} >= {
                    "typeHierarchy/supertypes", "typeHierarchy/subtypes",
                }, events
            provider = "genuine clangd" if args.clangd else "controlled child"
            print(f"PASS {provider} type navigation, exact name positions, retained draft/editor and unchanged disk")
        finally:
            if driver:
                driver.close()
            for process in reversed(processes):
                fixture.stop(process)


if __name__ == "__main__":
    main()
</ui:copy>

<ui:file path="scripts/test-type-navigation-browser.py"><ui:paste select="#type-hierarchy-browser-witness"/><ui:paste select="#type-hierarchy-browser-reading"/><ui:paste select="#type-hierarchy-browser-run"/></ui:file>

</ui:doc>
