<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Debug one Rust test from its saved source

For maintainers proving that a source-test choice reaches the installed LLDB adapter through a real Cargo test harness. The [preparation mechanism](test-debug-session.md) admits compiler artifacts and one qualified identity; these witnesses connect that mechanism to saved physical and constructed source admission. The [compiled source proof](rust-test-source-proof.md) also binds exact ordinary external module routes and refuses cfg-disabled declarations whose leaf name belongs to another compiled source. A test-process exit remains a process outcome, not a collected verdict. Physical pointer, device input and accessibility acceptance require separate evidence.

## Inspect the immutable manifest without creating files

The ordinary checks exercise the actual debug-plan authority. A Cargo manifest that exists only in the compiler graph can establish a typed preparation, while a missing manifest stays unavailable. Neither query may materialize the future outputs or start Cargo. The opt-in live case then runs both physical and constructed source through installed Cargo and LLDB, stops on the chosen author line, excludes a sibling sentinel and continues to successful completion. Constructed outputs must remain absent from the source workspace.

<ui:copy id="rust-selected-test-debug-service-witnesses">
#[cfg(test)]
mod rust_selected_test_live {
    use super::*;
    use serde_json::{Value, json};
    use std::sync::Arc;
    async fn ask(
        registry: &Arc<crate::debug_sessions::Registry>,
        root: &Path,
        request: Value,
    ) -> Vec<Value> {
        crate::serve::debug_bridge::handle(registry, root, serde_json::from_value(request).unwrap())
            .await
            .into_iter()
            .map(|row| serde_json::to_value(row).unwrap())
            .collect()
    }
    fn event<'a>(rows: &'a [Value], kind: &str) -> &'a Value {
        rows.iter()
            .find(|row| row["event"] == kind)
            .unwrap_or_else(|| panic!("Missing {kind}: {rows:?}"))
    }
    fn saved_request(root: &Path, path: &str, text: &str) -> RunRequest {
        std::fs::write(root.join(path), text).unwrap();
        let analysis = source_tests::Request {
            path: path.into(),
            source: text.into(),
            project_entry: None,
            sources: BTreeMap::new(),
            capture: false,
        };
        let scan = source_tests::scan(root, &analysis).unwrap();
        let candidate = scan
            .candidates
            .iter()
            .find(|c| c.name == "test_selected")
            .unwrap();
        RunRequest {
            version: 1,
            input: source_tests::RunInput::Saved {},
            analysis,
            fingerprint: scan.fingerprint,
            candidate_id: candidate.id.clone(),
            selection: Selection::NameFilter {},
        }
    }
    #[test]
    fn constructed_rust_plan_reads_the_compiler_manifest_without_materializing_it() {
        let dir = tempfile::tempdir().unwrap();
        let text = "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\">\n<h:file path=\"Cargo.toml\">\n[package]\nname=\"owned\"\nversion=\"0.1.0\"\n</h:file>\n<h:file path=\"src/lib.rs\">\n#[test]fn test_selected(){}\n</h:file>\n</h:doc>\n";
        let request = saved_request(dir.path(), "cases.md", text);
        let value = admit(dir.path(), &request).unwrap();
        assert!(value.plan.available, "{:?}", value.plan.reason);
        assert_eq!(value.launch.rust_manifest.as_ref().unwrap().0, "Cargo.toml");
        assert!(!dir.path().join("Cargo.toml").exists());
        assert!(!dir.path().join("src/lib.rs").exists());
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }
    #[test]
    fn constructed_rust_missing_manifest_stays_unavailable_without_output() {
        let dir = tempfile::tempdir().unwrap();
        let text = "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\">\n<h:file path=\"src/lib.rs\">\n#[test]fn test_selected(){}\n</h:file>\n</h:doc>\n";
        let request = saved_request(dir.path(), "cases.md", text);
        let value = admit(dir.path(), &request).unwrap();
        assert!(!value.plan.available);
        assert!(
            value
                .plan
                .reason
                .unwrap()
                .contains("no admitted Cargo.toml")
        );
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }
    #[cfg(unix)]
    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "requires installed Cargo and LLDB DAP; owned physical and compiler-author fixture"]
    async fn genuine_rust_selected_test_uses_exact_cargo_harness_and_preserves_authored_sources() {
        for authored in [false, true] {
            let directory = tempfile::tempdir().unwrap();
            let root = directory.path().canonicalize().unwrap();
            std::fs::create_dir(root.join(".git")).unwrap();
            let sibling = root.join("sibling-ran");
            let code = format!(
                "#[cfg(test)]\nmod cases {{\n #[test]\n fn test_selected() {{\n  let selected_value=41;\n  assert_eq!(selected_value,41); // owned-rust-stop\n }}\n #[test]\n fn test_sibling() {{std::fs::write({:?},b\"wrong\").unwrap();}}\n}}\n",
                sibling.to_str().unwrap()
            );
            let manifest = "[workspace]\n[package]\nname=\"owned-selected-debug\"\nversion=\"0.1.0\"\nedition=\"2024\"\n";
            let (path, text) = if authored {
                (
                    "tests.md",
                    format!(
                        "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\">\n<h:file path=\"Cargo.toml\">\n{manifest}</h:file>\n<h:file path=\"src/lib.rs\">\n{code}</h:file>\n</h:doc>\n"
                    ),
                )
            } else {
                std::fs::write(root.join("Cargo.toml"), manifest).unwrap();
                std::fs::create_dir(root.join("src")).unwrap();
                ("src/lib.rs", code)
            };
            let request = saved_request(&root, path, &text);
            let admitted = admit(&root, &request).unwrap();
            assert!(admitted.plan.available, "{:?}", admitted.plan.reason);
            assert_eq!(admitted.plan.framework, "libtest");
            let line = text
                .lines()
                .position(|line| line.contains("owned-rust-stop"))
                .unwrap();
            let registry = Arc::new(crate::debug_sessions::Registry::with_projects(
                Default::default(),
                root.clone(),
            ));
            let rows=ask(&registry,&root,json!({"op":"start_test","doc":admitted.plan.source_uri,"source_hash":admitted.plan.source_hash,"request_id":"rust-selected","source_test":request,"breakpoints":[{"line":line}]})).await;
            let started = event(&rows, "started");
            let stopped = event(&rows, "stopped");
            assert_eq!(stopped["line"], line);
            let session = started["session"].as_str().unwrap();
            assert_eq!(
                started["source_test"]["candidate_id"],
                admitted.binding.candidate_id
            );
            assert!(!sibling.exists());
            let cleared=ask(&registry,&root,json!({"op":"breakpoints","session":session,"source_hash":admitted.plan.source_hash,"request_id":"clear-rust-mark","breakpoints":[]})).await;
            assert!(
                cleared.iter().all(|row| row["event"] != "failed"),
                "{cleared:?}"
            );
            let done=ask(&registry,&root,json!({"op":"step","session":session,"how":"continue","stop":stopped["stop"],"thread":stopped["thread"],"request_id":"finish-rust-selected"})).await;
            assert_eq!(event(&done, "finished")["exit_code"], 0);
            assert!(!sibling.exists());
            assert_eq!(std::fs::read_to_string(root.join(path)).unwrap(), text);
            if authored {
                assert!(!root.join("Cargo.toml").exists());
                assert!(!root.join("src/lib.rs").exists())
            }
            assert!(registry.get(session).await.is_err());
        }
    }
    #[cfg(unix)]
    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "requires installed Cargo and LLDB DAP; exact ordinary external module and cfg-disabled sibling fixtures"]
    async fn genuine_rust_external_module_and_disabled_sibling_have_proven_source_owners() {
        for disabled in [false, true] {
            let directory = tempfile::tempdir().unwrap();
            let root = directory.path().canonicalize().unwrap();
            std::fs::create_dir(root.join(".git")).unwrap();
            std::fs::create_dir(root.join("src")).unwrap();
            std::fs::write(root.join("Cargo.toml"),"[workspace]\n[package]\nname='owned-source-proof'\nversion='0.1.0'\nedition='2024'\n").unwrap();
            let sentinel = root.join("wrong-sibling-ran");
            let code = "#[test]\nfn test_selected(){\n let selected_value=41;\n assert_eq!(selected_value,41); // owned-external-stop\n}\n";
            let declarations = if disabled {
                "#[cfg(any())]mod selected;\nmod sibling;\n"
            } else {
                "mod selected;\nmod sibling;\n"
            };
            std::fs::write(root.join("src/lib.rs"), declarations).unwrap();
            std::fs::write(
                root.join("src/sibling.rs"),
                format!(
                    "#[test]fn test_selected(){{std::fs::write({:?},b\"wrong\").unwrap();}}\n",
                    sentinel.to_str().unwrap()
                ),
            )
            .unwrap();
            let request = saved_request(&root, "src/selected.rs", code);
            let admitted = admit(&root, &request).unwrap();
            let registry = Arc::new(crate::debug_sessions::Registry::with_projects(
                Default::default(),
                root.clone(),
            ));
            let line = code
                .lines()
                .position(|line| line.contains("owned-external-stop"))
                .unwrap();
            let rows=ask(&registry,&root,json!({"op":"start_test","doc":admitted.plan.source_uri,"source_hash":admitted.plan.source_hash,"request_id":"rust-source-proof","source_test":request,"breakpoints":[{"line":line}]})).await;
            if disabled {
                assert!(!rows.iter().any(|r| r["event"] == "started"), "{rows:?}");
                assert!(rows.iter().any(|r| r["event"] == "failed"), "{rows:?}");
                assert!(
                    serde_json::to_string(&rows)
                        .unwrap()
                        .contains("no proven active module/target identity"),
                    "{rows:?}"
                );
            } else {
                let started = event(&rows, "started");
                let stopped = event(&rows, "stopped");
                assert_eq!(stopped["line"], line);
                let session = started["session"].as_str().unwrap();
                let cleared=ask(&registry,&root,json!({"op":"breakpoints","session":session,"source_hash":admitted.plan.source_hash,"request_id":"clear-external-mark","breakpoints":[]})).await;
                assert!(cleared.iter().all(|r| r["event"] != "failed"));
                let done=ask(&registry,&root,json!({"op":"step","session":session,"how":"continue","stop":stopped["stop"],"thread":stopped["thread"],"request_id":"finish-external"})).await;
                assert_eq!(event(&done, "finished")["exit_code"], 0);
            }
            assert!(!sentinel.exists());
            assert_eq!(
                std::fs::read_to_string(root.join("src/selected.rs")).unwrap(),
                code
            );
        }
    }
}
</ui:copy>

## Exercise the connected host choice

The owned runner starts the actual engine and installed Cargo/LLDB in a marked temporary workspace. The browser uses the existing Source tests popover and retained debugger link; macOS mounts its editor, and Android exercises the production native controller and source state. Each runs root-inline and ordinary external module cases in both physical and constructed saved inputs, pauses only the chosen test and continues without running a sibling. Editor identity, code geometry where mounted, saved bytes and available history state are retained. These automated host/controller flows do not establish physical pointer, device IME or accessibility acceptance.

<ui:copy id="rust-selected-test-debug-runner">
"""Owned real Cargo/LLDB source-test Debug flows. No physical-input claim."""
import argparse,importlib.util,json,os,socket,tempfile,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def module(path,name):
 spec=importlib.util.spec_from_file_location(name,path);value=importlib.util.module_from_spec(spec);spec.loader.exec_module(value);return value
def wait(test,label):
 deadline=time.monotonic()+45
 while time.monotonic()<deadline:
  if test():return
  time.sleep(.05)
 raise RuntimeError(label)
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--host',choices=['browser','apple','android'],required=True);args=parser.parse_args();fixture=module(ROOT/'scripts/test-simple-browser.py','simple_fixture');native=module(ROOT/'scripts/test-native-templates.py','native_fixture')
 with tempfile.TemporaryDirectory(prefix='rust-selected-debug-') as temporary:
  folder=Path(temporary);root=folder/'workspace';root.mkdir();(root/'.git').mkdir();(root/'.hickory').mkdir();(root/'.hickory-rust-test-debug-fixture').write_text('owned-rust-test-debug-v1\n');sibling=json.dumps(str(root/'sibling-ran'));source=f'''#[cfg(test)]
mod cases {{
 #[test]
 fn test_selected() {{
  let selected_value=41;
  assert_eq!(selected_value,41); // owned-rust-stop
 }}
 #[test]
 fn test_sibling() {{ std::fs::write({sibling},b"wrong").unwrap(); }}
}}
''';manifest='[workspace]\n[package]\nname="owned-selected-debug"\nversion="0.1.0"\nedition="2024"\n';(root/'physical/src').mkdir(parents=True);(root/'physical/Cargo.toml').write_text(manifest);(root/'physical/src/lib.rs').write_text(source);authored=f'<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\n<h:file path="Cargo.toml">\n{manifest}</h:file>\n<h:file path="src/lib.rs">\n{source}</h:file>\n</h:doc>\n';(root/'tests.md').write_text(authored);(root/'external/src').mkdir(parents=True);(root/'external/Cargo.toml').write_text(manifest);(root/'external/src/lib.rs').write_text('mod cases;\n');(root/'external/src/cases.rs').write_text(source);external_authored=f'<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\n<h:file path="Cargo.toml">\n{manifest}</h:file>\n<h:file path="src/lib.rs">\nmod cases;\n</h:file>\n<h:file path="src/cases.rs">\n{source}</h:file>\n</h:doc>\n';(root/'external.md').write_text(external_authored);processes=[];driver=None
  try:
   with socket.socket() as reserve:reserve.bind(('127.0.0.1',0));port=reserve.getsockname()[1]
   ui=f'http://127.0.0.1:{port}';env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state')};engine=fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(root),'window',ui],env=env);processes.append(engine);origin=fixture.origin(engine)
   if args.host=='browser':
    vite=fixture.start(['node','node_modules/vite/bin/vite.js','--config','vite.simple.config.ts','--host','127.0.0.1','--port',str(port),'--strictPort'],cwd=ROOT/'apps/web',env={**env,'HICKORY_API_ORIGIN':origin});processes.append(vite);driver=fixture.BrowserDriver(fixture.origin(vite)+'/simple.html')
    for path,text in [('physical/src/lib.rs',source),('tests.md',authored),('external/src/cases.rs',source),('external.md',external_authored)]:
     driver.act('file-command',command='Open File',path=path);driver.act('remember-editor',path=path);geometry=driver.act('editor-size',path=path);before=driver.act('source-text-state',path=path);line=next(i+1 for i,row in enumerate(text.splitlines())if'owned-rust-stop'in row);driver.act('debug-breakpoint',line=line);driver.act('source-tests-open');driver.act('source-tests-read');driver.act('source-tests-debug',filter='test_selected');driver.act('source-tests-debug-link');wait(lambda:any('paused'in row.lower()for row in driver.act('debug-state')),'Selected Rust test did not pause');assert not(root/'sibling-ran').exists();paused=driver.act('source-text-state',path=path);assert all(paused[key]==before[key] for key in ['source','undo','redo']);driver.act('debug-breakpoint-remove',path=path);driver.act('debug-continue');wait(lambda:any('finished'in row.lower()for row in driver.act('debug-state')),'Selected Rust test did not finish');driver.act('source-tests-open');records=driver.act('source-tests-debug-records');assert'process exit 0'in records and'test_selected'in records;driver.act('source-tests-close');assert driver.act('source-text-state',path=path)==paused;assert driver.act('construction-source',path=path)==text;assert(root/path).read_text()==text;assert driver.act('editor-is-retained',path=path);assert driver.act('editor-size',path=path)==geometry;assert not(root/'sibling-ran').exists();assert not(root/'Cargo.toml').exists();assert not(root/'src/lib.rs').exists()
    assert not driver.act('debug-diagnostics')['errors'];print('PASS browser: saved physical/authored Rust test selection through real Cargo and LLDB, sibling exclusion, retained source/geometry/history')
   else:
    acceptance=('RustSelectedTestDebugTests','testLiveSavedRustSelectionAndHistory','rust-selected-test')if args.host=='apple'else('RustSelectedTestDebugTest','liveSavedRustSelectionAndHistory','rust-selected-test');native.run_host(args.host,origin,root,folder,fixture,acceptance=acceptance)
  finally:
   if driver:driver.close()
   for process in reversed(processes):fixture.stop(process)
if __name__=='__main__':main()
</ui:copy>
<ui:file path="scripts/test-rust-test-debug.py"><ui:paste select="#rust-selected-test-debug-runner"/></ui:file>
<ui:copy id="rust-selected-test-debug-apple">
import XCTest
@testable import HickorySimple
#if os(macOS)
import AppKit
import SwiftUI
@MainActor final class RustSelectedTestDebugTests:XCTestCase{
 func testLiveSavedRustSelectionAndHistory()async throws{
  guard let origin=ProcessInfo.processInfo.environment["HICKORY_NATIVE_TEMPLATE_ORIGIN"],let path=ProcessInfo.processInfo.environment["HICKORY_NATIVE_TEMPLATE_WORKSPACE"]else{throw XCTSkip("Use the maintained Rust test Debug fixture")}
  let root=URL(fileURLWithPath:path);XCTAssertEqual(try String(contentsOf:root.appendingPathComponent(".hickory-rust-test-debug-fixture"),encoding:.utf8),"owned-rust-test-debug-v1\n");_ = NSApplication.shared;NSApp.setActivationPolicy(.regular);let engine=try Engine(address:origin),listing=try await engine.files()
  for path in ["physical/src/lib.rs","tests.md","external/src/cases.rs","external.md"]{
   let w=Workspace();w.engine=engine;w.rootFiles=listing.tree;w.sourceRootAddress=listing.root_path;w.sourceWorkspaceID=listing.workspace_id
   func find(_ path:String,_ nodes:[FileNode])->FileNode?{for node in nodes{if node.path==path{return node};if let value=find(path,node.children ?? []){return value}};return nil}
   let file=try XCTUnwrap(find(path,listing.tree)),b=try await w.loadBuffer(file),source=b.text;w.layout.activateSource(path,in:"document-group");let debug=NativeDebugger(engine:engine);w.debugger=debug;debug.sourceBuffers={Array(w.buffers.values)}
   let host=NSHostingView(rootView:WorkspaceView(workspace:w)),window=NSWindow(contentRect:NSRect(x:0,y:0,width:920,height:620),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=host;window.makeKeyAndOrderFront(nil);host.layoutSubtreeIfNeeded()
   defer{if debug.active{debug.stop()};debug.close();w.sourceTestIndex?.stop();for value in w.buffers.values{value.language?.close()};window.orderOut(nil);window.contentView=nil}
   func native(_ view:NSView)->NSTextView?{if let text=view as? NSTextView{return text};return view.subviews.lazy.compactMap{native($0)}.first}
   func wait(_ label:String,_ ready:()->Bool)async throws{for _ in 0..<1500{host.layoutSubtreeIfNeeded();if ready(){return};try await Task.sleep(for:.milliseconds(20))};throw EngineError(message:label+": "+debug.message+" · "+debug.status)}
   try await wait("Native editor did not mount"){native(host) != nil};let text=try XCTUnwrap(native(host)),scroll=try XCTUnwrap(text.enclosingScrollView),identity=try XCTUnwrap(b.sourceEditorState?()).identity,geometry=scroll.convert(scroll.bounds,to:host);let selection=text.selectedRange(),position=scroll.contentView.bounds.origin
   let line=try XCTUnwrap(source.components(separatedBy:"\n").firstIndex{$0.contains("owned-rust-stop")});debug.toggle(b,line:line);w.showTests=true;await w.testSources.refresh(w);let facts=try XCTUnwrap(w.testSources.observation?.1),candidate=try XCTUnwrap(facts.candidates.first{$0.name=="test_selected"});XCTAssertEqual(candidate.language,"rust");await w.testSources.debug(candidate,.nameFilter,workspace:w)
   try await wait("Selected Rust test did not pause"){debug.status=="paused"||debug.status=="failed"};XCTAssertEqual(debug.status,"paused",debug.message);XCTAssertEqual(debug.pausedLine,line);XCTAssertEqual(w.testSources.debugRuns.count,1);XCTAssertEqual(b.text,source);XCTAssertFalse(FileManager.default.fileExists(atPath:root.appendingPathComponent("sibling-ran").path));debug.toggle(b,line:line);debug.step("continue");try await wait("Selected Rust test did not finish"){debug.status=="finished"||debug.status=="failed"};XCTAssertEqual(debug.status,"finished",debug.message);XCTAssertEqual(b.text,source);XCTAssertEqual(try String(contentsOf:root.appendingPathComponent(path),encoding:.utf8),source);XCTAssertEqual(try XCTUnwrap(b.sourceEditorState?()).identity,identity);XCTAssertEqual(text.selectedRange(),selection);XCTAssertEqual(scroll.contentView.bounds.origin,position);XCTAssertEqual(scroll.convert(scroll.bounds,to:host),geometry);XCTAssertFalse(FileManager.default.fileExists(atPath:root.appendingPathComponent("sibling-ran").path));XCTAssertFalse(FileManager.default.fileExists(atPath:root.appendingPathComponent("Cargo.toml").path))
  }
 }
}
#endif
</ui:copy>
<ui:file path="apps/apple-simple/Tests/RustSelectedTestDebugTests.swift"><ui:paste select="#rust-selected-test-debug-apple"/></ui:file>
<ui:copy id="rust-selected-test-debug-android">
package com.hickorydocs.simple
import java.io.File
import kotlinx.coroutines.*
import org.junit.Assert.*
import org.junit.Test
class RustSelectedTestDebugTest{
 @Test(timeout=180000)fun liveSavedRustSelectionAndHistory()=runBlocking{
  val origin=System.getenv("HICKORY_NATIVE_TEMPLATE_ORIGIN");org.junit.Assume.assumeNotNull(origin);val root=File(System.getenv("HICKORY_NATIVE_TEMPLATE_WORKSPACE")?:error("Use maintained Rust test Debug fixture")).canonicalFile;assertEquals("owned-rust-test-debug-v1\n",root.resolve(".hickory-rust-test-debug-fixture").readText());val scope=CoroutineScope(SupervisorJob()+Dispatchers.Default);val engine=Engine(origin!!)
  try{val(_,tree)=engine.files();fun find(path:String,nodes:List<FileNode> = tree):FileNode?{for(node in nodes){if(node.path==path)return node;find(path,node.children)?.let{return it}};return null}
   for(path in listOf("physical/src/lib.rs","tests.md")){val w=Workspace(scope);w.engine=engine;w.rootFiles=tree;val file=find(path)?:error("Owned Rust source missing");val b=w.loadBuffer(file);w.show(path);val source=b.text;val state=b.state;val selection=state.selection;val scroll=b.sourceScroll;val debug=NativeDebugger(engine,scope);w.debugger=debug
    try{suspend fun wait(label:String,ready:()->Boolean){withTimeout(45000){while(!ready())delay(20)}}
     val line=source.lines().indexOfFirst{it.contains("owned-rust-stop")};assertTrue(line>=0);debug.toggle(b,line);w.showTests=true;w.testSources.refresh(w);val facts=w.testSources.observation?.second?:error(w.testSources.notice);val candidate=facts.candidates.single{it.name=="test_selected"};assertEquals("rust",candidate.language);w.testSources.debug(candidate,TestSourceSelection.NAME_FILTER,w);wait("Selected Rust pause"){debug.status=="paused"||debug.status=="failed"};assertEquals(debug.message,"paused",debug.status);assertEquals(line,debug.line(path)?:-1);assertEquals(1,w.testSources.debugRuns.size);assertFalse(root.resolve("sibling-ran").exists());debug.toggle(b,line);debug.step("continue");wait("Selected Rust completion"){debug.status=="finished"||debug.status=="failed"};assertEquals(debug.message,"finished",debug.status);assertSame(state,b.state);assertSame(scroll,b.sourceScroll);assertEquals(selection,state.selection);assertEquals(source,b.text);assertEquals(source,root.resolve(path).readText());assertFalse(root.resolve("sibling-ran").exists());assertFalse(root.resolve("Cargo.toml").exists())
    }finally{if(debug.active)debug.stop();debug.close();w.buffers.values.forEach{it.language?.close()}}
   }
  }finally{scope.cancel()}
 }
}
</ui:copy>
<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/RustSelectedTestDebugTest.kt"><ui:paste select="#rust-selected-test-debug-android"/></ui:file>

## Prove the source route over the real workspace protocol

This unattended service witness uses the installed Cargo/LLDB route through actual HTTP candidate admission and channel3 workspace messages. It covers inline and ordinary external modules in saved physical and constructed inputs. Each positive stops at its selected author line, clears the mark and exits0; a same-leaf sibling never runs. Both cfg-disabled selections refuse before a session starts. It retains the actual event records rather than inferring launch success from a model or menu. It does not exercise a rendered host; the connected host witness above supplies that separate evidence.

<ui:copy id="rust-selected-test-source-proof-protocol">
from pathlib import Path
import importlib.util,json,os,subprocess,tempfile
NODE = r"""
import fs from 'node:fs';
const [origin,root,casesFile,receipt]=process.argv.slice(2);const cases=JSON.parse(fs.readFileSync(casesFile));
async function post(path,body){const response=await fetch(origin+'/api/'+path,{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(body)});const text=await response.text();if(!response.ok)throw Error(path+': '+response.status+' '+text);return JSON.parse(text)}
const socket=new WebSocket(origin.replace('http:','ws:')+'/api/ws?doc=workspace');socket.binaryType='arraybuffer';const rows=[];let socketError;socket.addEventListener('message',event=>{const bytes=new Uint8Array(event.data);if(bytes[0]===3){const row=JSON.parse(new TextDecoder().decode(bytes.slice(1)));rows.push(row);console.log(JSON.stringify(row));}});socket.addEventListener('error',()=>{socketError='Debug socket failed'});await new Promise((resolve,reject)=>{socket.addEventListener('open',resolve,{once:true});socket.addEventListener('error',reject,{once:true})});
function send(value){const json=new TextEncoder().encode(JSON.stringify(value));const frame=new Uint8Array(json.length+1);frame[0]=3;frame.set(json,1);socket.send(frame)}
async function wait(test,label){const end=Date.now()+120000;while(Date.now()<end){if(socketError)throw Error(socketError);const value=test();if(value)return value;await new Promise(resolve=>setTimeout(resolve,20))}throw Error(label+': '+JSON.stringify(rows.slice(-5)))}
const outcomes=[];
try{for(const [index,fixture]of cases.entries()){
 const text=fs.readFileSync(root+'/'+fixture.path,'utf8');const analysis={path:fixture.path,source:text,project_entry:null,sources:{},capture:false};const scan=await post('tests/source-candidates',analysis);const candidate=scan.candidates.find(c=>c.name==='test_selected'&&(!fixture.negative||c.target.path.endsWith('selected.rs')));if(!candidate)throw Error('Candidate absent: '+JSON.stringify(scan));const request={version:1,input:{kind:'saved'},analysis,fingerprint:scan.fingerprint,candidate_id:candidate.id,selection:{kind:'name_filter'}};const plan=await post('tests/debug-plan',request);if(!plan.available)throw Error('Plan unavailable: '+JSON.stringify(plan));const line=text.split('\n').findIndex(line=>line.includes('owned-proof-stop'));const start=rows.length;const id='proof-'+index;send({op:'start_test',doc:plan.source_uri,source_hash:plan.source_hash,request_id:id,source_test:request,breakpoints:[{line}]});
 if(fixture.negative){const failed=await wait(()=>rows.slice(start).find(r=>r.event==='failed'),'Disabled declaration did not refuse');if(rows.slice(start).some(r=>r.event==='started'))throw Error('Disabled declaration started sibling');if(!JSON.stringify(failed).includes('no proven active module/target identity'))throw Error('Wrong refusal: '+JSON.stringify(failed));outcomes.push({path:fixture.path,outcome:'refused-disabled-source-before-session'});}
 else{const stop=await wait(()=>{const failed=rows.slice(start).find(r=>r.event==='failed');if(failed)throw Error(JSON.stringify(failed));return rows.slice(start).find(r=>r.event==='stopped')},'Selected source did not pause');if(stop.line!==line)throw Error('Wrong source line '+JSON.stringify(stop));const session=rows.slice(start).find(r=>r.event==='started')?.session;if(!session)throw Error('Missing started receipt');const clearIndex=rows.length;send({op:'breakpoints',session,source_hash:plan.source_hash,request_id:id+'-clear',breakpoints:[]});await wait(()=>rows.slice(clearIndex).find(r=>r.event==='breakpoints'),'Mark did not clear');send({op:'step',session,how:'continue',stop:stop.stop,thread:stop.thread,request_id:id+'-finish'});const finish=await wait(()=>rows.slice(start).find(r=>r.event==='finished'&&r.session===session),'Selected test did not finish');if(finish.exit_code!==0)throw Error('Wrong process exit '+JSON.stringify(finish));outcomes.push({path:fixture.path,outcome:'selected-source-paused-exit0'});}
 if(fs.existsSync(root+'/wrong-sibling-ran'))throw Error('Sibling body executed');if(fs.readFileSync(root+'/'+fixture.path,'utf8')!==text)throw Error('Authored source changed');
 }
 fs.writeFileSync(receipt,JSON.stringify({outcomes,events:rows},null,2));console.log('PASS direct Rust source association '+outcomes.length+' physical/authored inline/external/disabled flows');
}finally{socket.close()}
"""
ROOT=Path(__file__).resolve().parents[1];spec=importlib.util.spec_from_file_location('fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
with tempfile.TemporaryDirectory(prefix='rust-source-proof-')as temp:
 folder=Path(temp);root=folder/'workspace';root.mkdir();(root/'.git').mkdir();manifest='[workspace]\n[package]\nname="owned-source-proof"\nversion="0.1.0"\nedition="2024"\n';cases=[]
 for kind in ['inline','external','disabled']:
  selected='#[test]\nfn test_selected(){\n let selected_value=41;\n assert_eq!(selected_value,41); // owned-proof-stop\n}\n';sibling='#[test]fn test_selected(){std::fs::write('+json.dumps(str(root/'wrong-sibling-ran'))+',b"wrong").unwrap();}\n'
  if kind=='inline':files={'Cargo.toml':manifest,'src/lib.rs':'mod selected {\n'+selected+'}\nmod sibling {\n'+sibling+'}\n'};target='src/lib.rs'
  else:files={'Cargo.toml':manifest,'src/lib.rs':('#[cfg(any())]'if kind=='disabled'else'')+'mod selected;\nmod sibling;\n','src/selected.rs':selected,'src/sibling.rs':sibling};target='src/selected.rs'
  for path,text in files.items():file=root/kind/path;file.parent.mkdir(parents=True,exist_ok=True);file.write_text(text)
  cases.append({'path':kind+'/'+target,'negative':kind=='disabled'});book='<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\n'+''.join('<h:file path="'+path+'">\n'+text+'</h:file>\n'for path,text in files.items())+'</h:doc>\n';name=kind+'.md';(root/name).write_text(book);cases.append({'path':name,'negative':kind=='disabled'})
 file=folder/'cases.json';file.write_text(json.dumps(cases));script=folder/'protocol.mjs';script.write_text(NODE);receipt=ROOT/'target/rust-test-debug/source-proof/protocol-receipt.json';receipt.parent.mkdir(parents=True,exist_ok=True);env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state'),'DEVELOPER_DIR':'/Applications/Xcode-27.0.0.app/Contents/Developer'};engine=fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(root),'window','http://127.0.0.1:43981'],env=env)
 try:
  origin=fixture.origin(engine);result=subprocess.run(['node',str(script),origin,str(root),str(file),str(receipt)],env=env);raise SystemExit(result.returncode)
 finally:fixture.stop(engine)
</ui:copy>
<ui:file path="scripts/test-rust-test-source-proof.py"><ui:paste select="#rust-selected-test-source-proof-protocol"/></ui:file>

</ui:doc>
