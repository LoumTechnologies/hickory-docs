<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Verify real semantic readings of authored Markdown fences

For maintainers checking the usable path from an ordinary Markdown editor through installed Python and TypeScript providers. The controlled fixture uses pinned existing tool installations and a real engine, with Unicode, CRLF and indented Python beside TypeScript. No code output exists. These opt-in cases must execute exactly once without skips in their owned runner; normal model runs explicitly skip the absent live fixture.

## Build one owned workspace and retain truthful receipts

The runner pins provider versions, isolates server state and source files, and checks saved bytes and the complete workspace file roster afterward. Browser acceptance uses the existing Edit menu and semantic-location popup, opens each authored definition and checks retained editor geometry. Native acceptance calls the same production language controller and source navigation used by the regular popup; macOS mounts the real editor. Android exercises its controller, not physical touch input. The shared native runner retains exact XML or a labeled strict XCTest-console receipt. iOS compilation and physical input/accessibility remain separate acceptance.

<storycode:copy id="fenced-provider-acceptance-runner">
"""Owned genuine provider fixture for ordinary authored Markdown code readings."""
import argparse,importlib.util,json,os,socket,subprocess,sys,tempfile,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
SOURCE='Readable prose 😀\r\n\r\n  ```python\r\n  # Unicode 😀\r\n  def greet(name: str) -> str:\r\n      return name.upper()\r\n  result = greet("reader")\r\n  ```\r\n\r\n```typescript\r\nconst marker = "😀"; function greet(name: string) { return name.toUpperCase(); }\r\nconst result = greet("reader");\r\n```\r\n'
ROWS=[{'query':{'line':6,'character':12},'definition':{'line':4,'character':6}}, {'query':{'line':11,'character':16},'definition':{'line':10,'character':30}}]
def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--host',choices=['protocol','browser','apple','android'],default='protocol');args=parser.parse_args()
 spec=importlib.util.spec_from_file_location('fenced_simple_fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 tools=ROOT/'target/refactor-provider/node_modules';typescript=tools/'typescript-language-server/lib/cli.mjs';compiler=tools/'typescript/lib/tsserver.js';python=ROOT/'target/fenced-lsp-tools/basedpyright/dist/pyright-langserver.js'
 for path in [typescript,compiler,python]:assert path.is_file(),f'Pinned installed provider missing: {path}'
 assert json.loads((tools/'typescript-language-server/package.json').read_text())['version']=='6.1.0'
 assert json.loads((tools/'typescript/package.json').read_text())['version']=='6.0.3'
 with tempfile.TemporaryDirectory(prefix='fenced-source-reading-',dir=ROOT/'target') as temporary:
  folder=Path(temporary);root=folder/'workspace';root.mkdir();(root/'notes.md').write_bytes(SOURCE.encode());(root/'.hickory-native-template-fixture').write_text('owned-native-templates-v1\n')
  (root/'node_modules').mkdir();(root/'node_modules/typescript').symlink_to(tools/'typescript',target_is_directory=True)
  (root/'tsconfig.json').write_text(json.dumps({'compilerOptions':{'strict':True,'noEmit':True},'typeAcquisition':{'enable':False}}))
  (root/'.hick-lsp.json').write_text(json.dumps({'servers':{'python':{'command':['node',str(python),'--stdio']},'typescript':{'command':['node',str(typescript),'--stdio']}}}))
  data=folder/'fixture.json';data.write_text(json.dumps({'source':SOURCE,'rows':ROWS}));processes=[];driver=None
  try:
   with socket.socket() as reservation:reservation.bind(('127.0.0.1',0));port=reservation.getsockname()[1]
   env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state')};engine=fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(root),'window',f'http://127.0.0.1:{port}'],env=env);processes.append(engine);origin=fixture.origin(engine)
   if args.host=='protocol':subprocess.run(['node',str(ROOT/'scripts/fenced-source-reading-driver.mjs'),origin,str(data)],cwd=ROOT,check=True,timeout=180)
   elif args.host in ['apple','android']:
    spec=importlib.util.spec_from_file_location('native_fixture',ROOT/'scripts/test-native-templates.py');native=importlib.util.module_from_spec(spec);spec.loader.exec_module(native)
    native.run_host(args.host,origin,root,folder,fixture,acceptance=(('FencedSourceReadingTests','testLiveFencedSourceLanguageReadings','fenced-source') if args.host=='apple' else ('FencedSourceReadingTest','liveFencedSourceLanguageReadings','fenced-source')))
   else:
    vite=fixture.start(['node','node_modules/vite/bin/vite.js','--config','vite.simple.config.ts','--host','127.0.0.1','--port',str(port),'--strictPort'],cwd=ROOT/'apps/web',env={**env,'HICKORY_API_ORIGIN':origin});processes.append(vite);driver=fixture.BrowserDriver(fixture.origin(vite)+'/simple.html');driver.act('open',path='notes.md');driver.act('remember-editor',path='notes.md');size=driver.act('editor-size',path='notes.md')
    for row in ROWS:
     lines=SOURCE.splitlines(keepends=True);prefix=''.join(lines[:row['query']['line']])+lines[row['query']['line']][:row['query']['character']];offset=len(prefix.replace('\r\n','\n'));deadline=time.monotonic()+30
     while True:
      result=driver.act('semantic-query',path='notes.md',offset=offset,query='Go to definition');text=result['text']
      if 'notes.md' in text and '1 locations' in text:break
      driver.act('semantic-close');assert time.monotonic()<deadline,text;time.sleep(.1)
     assert driver.act('construction-source',path='notes.md')==SOURCE;definition=row['definition'];driver.act('semantic-open',path='notes.md',label=f"notes.md:{definition['line']+1}:{definition['character']+1}");before=''.join(lines[:definition['line']]);expected=len(before.encode('utf-16-le'))//2+definition['character'];assert driver.act('editor-selections',path='notes.md')['ranges']==[{'from':expected,'to':expected}]
    assert driver.act('editor-is-retained',path='notes.md');assert driver.act('editor-size',path='notes.md')==size
   assert(root/'notes.md').read_bytes()==SOURCE.encode()
   assert set(path.name for path in root.iterdir())=={'notes.md','.hickory-native-template-fixture','tsconfig.json','.hick-lsp.json','node_modules'},'Language readings materialized unexpected files'
   print('PASS: genuine providers read authored fences; saved source bytes and workspace file roster unchanged. Physical-input and accessibility acceptance remain unverified.')
  finally:
   if driver:driver.close()
   for process in reversed(processes):fixture.stop(process)
if __name__=='__main__':main()
</storycode:copy>
<storycode:file path="scripts/test-fenced-source-readings.py"><storycode:paste select="#fenced-provider-acceptance-runner"/></storycode:file>
## Ask the real provider without granting write authority

The WebSocket client awaits exact source preparation, then checks both provider readings against their authored coordinates. Dirty replacement and Undo must query fresh names. Inspection cannot authorize completion insertion, rename or formatting, and closing the source retires its read address.

<storycode:copy id="fenced-provider-protocol-client">
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
</storycode:copy>
<storycode:file path="scripts/fenced-source-reading-driver.mjs"><storycode:paste select="#fenced-provider-protocol-client"/></storycode:file>
## Follow the normal mounted macOS navigation path

Keep a real editor mounted while production inspect, navigate and open methods resolve both languages. The source, selection conversion, editor identity, undo manager and saved bytes are observed through the same objects used by the application, with bounded provider readiness independent of readable text.

<storycode:copy id="fenced-provider-apple-witness">
import XCTest
#if os(macOS)
import AppKit
#endif
@testable import HickorySimple
@MainActor final class FencedSourceReadingTests:XCTestCase {
 #if os(macOS)
 func testLiveFencedSourceLanguageReadings()async throws {
  guard let origin=ProcessInfo.processInfo.environment["HICKORY_NATIVE_TEMPLATE_ORIGIN"],let directory=ProcessInfo.processInfo.environment["HICKORY_NATIVE_TEMPLATE_WORKSPACE"] else{throw XCTSkip("Run test-fenced-source-readings.py --host apple")}
  let root=URL(fileURLWithPath:directory);XCTAssertEqual(try String(contentsOf:root.appendingPathComponent(".hickory-native-template-fixture"),encoding:.utf8),"owned-native-templates-v1\n")
  let fixture=try XCTUnwrap(JSONSerialization.jsonObject(with:Data(contentsOf:root.deletingLastPathComponent().appendingPathComponent("fixture.json"))) as? [String:Any]),rows=try XCTUnwrap(fixture["rows"] as? [[String:Any]])
  let engine=try Engine(address:origin),tree=try await engine.files();_ = NSApplication.shared
  func find(_ nodes:[FileNode])->FileNode?{for node in nodes{if node.path=="notes.md"{return node};if let result=find(node.children ?? []){return result}};return nil}
  let file=try XCTUnwrap(find(tree.tree)),saved=try await engine.file(file),buffer=DocumentBuffer(file:file,text:saved),editor=MacEditor(buffer)
  let window=NSWindow(contentRect:NSRect(x:0,y:0,width:640,height:420),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=editor.scroll
  let language=NativeLanguage(engine:engine,buffer:buffer);buffer.language=language;language.start();defer{language.close();window.contentView=nil}
  let identity=try XCTUnwrap(buffer.sourceEditorState?()).identity,manager=try XCTUnwrap(editor.text.undoManager),scroll=editor.scroll.contentView.bounds.origin
  for _ in 0..<500{if language.status.hasPrefix("LSP"){break};try await Task.sleep(for:.milliseconds(10))};XCTAssertTrue(language.status.hasPrefix("LSP"),language.status)
  for row in rows{
   let query=try XCTUnwrap(row["query"] as? [String:Int]),definition=try XCTUnwrap(row["definition"] as? [String:Int]),offset=try XCTUnwrap(languageOffset(buffer.text,query))
   editor.text.setSelectedRange(NSRange(location:offset,length:0));buffer.selection=editor.text.selectedRange()
   let expectedHover=definition["line"]==4 ? "def greet":"function greet"
   for _ in 0..<200{await language.inspect();if language.hover.contains(expectedHover){break};try await Task.sleep(for:.milliseconds(50))};XCTAssertTrue(language.hover.contains(expectedHover),language.hover)
   await language.navigate(.definition)
   let target=try XCTUnwrap(language.locations.first(where:{$0.path=="notes.md"}),language.locationStatus)
   XCTAssertEqual(target.line,definition["line"]);XCTAssertEqual(target.character,definition["character"])
   language.open(target);let destination=try XCTUnwrap(languageOffset(buffer.text,definition));XCTAssertEqual(buffer.selection,NSRange(location:destination,length:0));XCTAssertEqual(editor.text.selectedRange(),buffer.selection)
  }
  XCTAssertEqual(buffer.text,saved.content);XCTAssertEqual(buffer.base,saved.content);XCTAssertEqual(buffer.sourceEditorState?()?.identity,identity);XCTAssertTrue(editor.text.undoManager===manager);XCTAssertEqual(editor.scroll.contentView.bounds.origin,scroll);let disk=try await engine.file(file);XCTAssertEqual(disk.content,saved.content)
 }
 #endif
}
</storycode:copy>
<storycode:file path="apps/apple-simple/Tests/FencedSourceReadingTests.swift"><storycode:paste select="#fenced-provider-apple-witness"/></storycode:file>
## Follow the Android controller path

Use the production language connection and source-selection/navigation methods against the same real providers. This verifies controller integration and retained editor state; it does not substitute for physical touch, device IME or accessibility acceptance.

<storycode:copy id="fenced-provider-android-witness">
package com.hickorydocs.simple
import org.junit.Assert.*
import org.junit.Test
import org.json.JSONObject
class FencedSourceReadingTest {
 @Test(timeout=120000) fun liveFencedSourceLanguageReadings()=kotlinx.coroutines.runBlocking {
  val origin=System.getenv("HICKORY_NATIVE_TEMPLATE_ORIGIN");org.junit.Assume.assumeNotNull(origin)
  val root=java.io.File(System.getenv("HICKORY_NATIVE_TEMPLATE_WORKSPACE") ?: error("Use test-fenced-source-readings.py --host android")).canonicalFile
  check(root.resolve(".hickory-native-template-fixture").readText()=="owned-native-templates-v1\n")
  val rows=JSONObject(root.parentFile.resolve("fixture.json").readText()).getJSONArray("rows");val engine=Engine(origin!!)
  val (_,tree)=engine.files()
  fun find(nodes:List<FileNode>):FileNode?{for(node in nodes){if(node.path=="notes.md")return node;find(node.children)?.let{return it}};return null}
  val file=find(tree) ?: error("Missing authored notes.md");val saved=engine.file(file);val buffer=DocumentBuffer(file,saved);val state=buffer.state;val scroll=buffer.sourceScroll.value
  val language=NativeLanguage(engine,buffer,this);buffer.language=language;language.start()
  try{
   repeat(500){if(!language.connection.active||language.status!="LSP connected")kotlinx.coroutines.delay(10)};check(language.connection.active&&language.status=="LSP connected"){language.status}
   for(index in 0 until rows.length()){
    val row=rows.getJSONObject(index);val query=row.getJSONObject("query");val definition=row.getJSONObject("definition");val offset=languageOffset(buffer.text,query) ?: error("Invalid authored query")
    assertTrue(buffer.selectSourceRange(offset,offset,buffer.text));val expectedHover=if(definition.getInt("line")==4)"def greet" else "function greet";for(attempt in 0 until 200){language.inspect();if(language.hover.contains(expectedHover))break;kotlinx.coroutines.delay(50)};assertTrue(language.hover,language.hover.contains(expectedHover));language.navigate(SemanticQuery.Definition)
    val target=language.locations.firstOrNull{it.path=="notes.md"} ?: error(language.locationStatus)
    assertEquals(definition.getInt("line"),target.line);assertEquals(definition.getInt("character"),target.character);assertTrue(language.open(target))
    val destination=languageOffset(buffer.text,definition) ?: error("Invalid authored definition");assertEquals(destination,state.selection.min);assertEquals(destination,state.selection.max)
   }
   assertEquals(saved.content,buffer.text);assertEquals(saved.content,buffer.base);assertSame(state,buffer.state);assertEquals(scroll,buffer.sourceScroll.value);assertEquals(saved.content,engine.file(file).content)
  }finally{language.close()}
 }
}
</storycode:copy>
<storycode:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/FencedSourceReadingTest.kt"><storycode:paste select="#fenced-provider-android-witness"/></storycode:file>

## Evidence boundary

The maintained runner passed all four modes on the same fresh engine after exact inspection priority was fixed: `--host protocol`, `--host browser`, `--host apple` and `--host android`. Protocol acceptance used genuine basedpyright 1.40.2 and TypeScript Language Server 6.1.0 with TypeScript 6.0.3. It checked authored hover/definition coordinates, Unicode/CRLF and indentation, prose exclusion, fresh dirty source and Undo readings, close retirement, and refusal of completion, rename and formatting authority. Saved source bytes and the complete owned workspace file roster remained unchanged.

Rendered browser acceptance used Edit → Go to definition and clicked the exact authored location for both fences. It retained the original editor and geometry. Mounted macOS executed exactly one passing case without skips or failures (0.885 seconds), retained the editor and native undo manager, and opened both exact authored targets. Android executed exactly one passing controller case without skips, failures or errors (0.978 seconds), retained its editor state and scroll, and opened both targets. Native hover assertions distinguish Python's `def greet` from TypeScript's `function greet`, so a retained prior-language reading cannot satisfy the next case.

Executed logs are `target/fenced-source-{protocol,browser,apple,android}-final.log`. The macOS runner retained a truthful `xctest-console` receipt and matching log at `target/native-template-smoke/apple-fenced-source-last.{json,log}`; it did not invent framework XML. Android retained exact producer XML and log at `target/native-template-smoke/android-fenced-source-last.{xml,log}`. The official generated focused Rust checks passed five `fenced_` cases and eleven `position_map::tests` cases (`target/parity-fenced-priority-results.log`).

A first real protocol failure exposed randomized routing between the whole-document Markdown map and the exact fenced map; transparent wire tracing confirmed the genuine child never received the failed query. Read routing now prefers one exact current inspection before normal Markdown and refuses multiple inspection matches. The focused backend regression checks this overlap and preparation, equal-byte revision ABA and close retirement. Initial failing evidence remains in `target/fenced-source-protocol-actual.log` and the isolated wire trace; it was not declared a successful acceptance or worked around with a retry-only change.

These outcomes establish independent semantic readings of statically known, complete, pathless Markdown fences through installed providers. Unknown, dynamic and incomplete regions do not gain an inspection provider. A private inspection address grants no generated-file, edit, runtime or test ownership. Cross-fence/generated-output module inference and arbitrary-provider sandboxing are not provided by this inspection role. iOS runtime, physical pointer/keyboard input, device IME and accessibility acceptance remain unverified; current iOS product evidence is maintained separately by the integration owner.

</storycode:doc>
