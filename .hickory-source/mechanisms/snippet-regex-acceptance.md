<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Exercise negotiated regex transforms through real source actions

These witnesses are for the integration owner deciding whether version 3 may be advertised. The pure VM corpus establishes matching and formatting semantics; it does not establish a usable editor path. Start an isolated real engine, an external child process and the actual host controls/controllers. The fixture supplies authored snippet syntax, never compiled programs or canned engine replies.

Run `python3 scripts/test-snippet-regex-live.py --host protocol`, then `--host browser`, `--host apple` and `--host android`. Add `--template` for the existing source-template path only after that consumer negotiates version 3. The native runner reuses the repository's strict exact-one XML or identified SDK-console receipt checks. An absent opt-in environment skips normal model runs; this explicit runner must reject skips. iOS uses the same Swift evaluator and production bridge, but these macOS window witnesses do not establish device input or accessibility acceptance.

## Declare independent visible outcomes

The source retains mixed CRLF/LF bytes. The first template combines lookbehind, an escaped named capture and backreference, scoped flags, v-mode intersection, Unicode emoji string properties and standard format conditions/casing. The second starts with a cheap valid expansion, then deliberately exhausts the evaluator during field editing. That refusal must leave both primary and calculated occurrences unchanged and add no undo entry.
<ui:copy id="snippet-regex-live-cases">
{
  "version": 1,
  "source": "OldOld\r\nsecond\n",
  "syntax": "${1:${TM_SELECTED_TEXT}}|$1|${1/(?<=.{3})(?<tail>.+)/${1:/upcase}/u}|${1/(?<\\u0077ord>[a-z]{3})\\k<word>/${1:/upcase}/i}|${1/[\\p{L}&&\\p{ASCII}]+/${0:/snakecase}/v}|${1/(?i:old)/${0:/downcase}/g}|${1/(?:\\p{RGI_Emoji}|(?<word>[A-Za-z]+))/${1:?word:emoji}/gv}$0",
  "budget_syntax": "${1:${TM_SELECTED_TEXT}}|${1/(a+)+$/ok/}$0",
  "after": "OldOld|OldOld|OldOLD|OLD|old_old|oldold|word\r\nsecond\n",
  "typed": "NewNew|NewNew|NewNEW|NEW|new_new|NewNew|word\r\nsecond\n",
  "emoji": "😀|😀|😀|😀|😀|😀|emoji\r\nsecond\n",
  "budget_after": "OldOld|OldOld\r\nsecond\n"
}
</ui:copy>
<ui:file path="apps/simple-contract/snippet-regex-live-cases.json"><ui:paste select="#snippet-regex-live-cases"/></ui:file>

## Require truthful child initialization and retain opaque data

The child returns snippets only if the engine actually advertises support. It records query/resolve URIs and checks the exact original opaque item on resolve. No provider command is part of expansion; attempting one fails the fixture.

<ui:copy id="snippet-regex-child-fixture">
"""External lazy child fixture; never supplies a compiled program or writes source."""
import json,sys
EVENTS=sys.argv[1]
from pathlib import Path
CASES=json.loads((Path(__file__).resolve().parents[1]/'apps/simple-contract/snippet-regex-live-cases.json').read_text())
SYNTAX=CASES['syntax'];BUDGET=CASES['budget_syntax']
def record(value):
 with open(EVENTS,'a') as stream:stream.write(json.dumps(value)+'\n')
def send(value):
 raw=json.dumps(value).encode();sys.stdout.buffer.write(b'Content-Length: '+str(len(raw)).encode()+b'\r\n\r\n'+raw);sys.stdout.buffer.flush()
issued={};snippets=False
while True:
 headers={}
 while True:
  line=sys.stdin.buffer.readline()
  if not line:sys.exit()
  if line in (b'\r\n',b'\n'):break
  name,value=line.decode().split(':',1);headers[name.lower()]=value.strip()
 message=json.loads(sys.stdin.buffer.read(int(headers['content-length'])));identity=message.get('id');method=message.get('method');params=message.get('params',{})
 if identity is None:continue
 if method=='initialize':
  snippets=params.get('capabilities',{}).get('textDocument',{}).get('completion',{}).get('completionItem',{}).get('snippetSupport') is True;record({'initialize_snippets':snippets});result={'capabilities':{'textDocumentSync':1,'documentSymbolProvider':True,'completionProvider':{'resolveProvider':True,'triggerCharacters':['.']}}}
 elif method=='textDocument/documentSymbol':result=[]
 elif method=='textDocument/completion':
  uri=params['textDocument']['uri'];record({'query':uri});item={'label':'Regex mirrors','filterText':'OldOld','data':{'opaque':{'uri':uri,'unicode':'é😀','range':{'start':{'line':91,'character':2},'end':{'line':91,'character':4}}}}}
  if uri.endswith('/inline.py'):item.update(insertTextFormat=2,insertText=SYNTAX)
  issued[uri]=item;result=[item] if snippets else []
 elif method=='completionItem/resolve':
  uri=params.get('data',{}).get('opaque',{}).get('uri');assert params==issued.get(uri),'Original opaque provider item was changed';record({'resolve':uri});result={**params,'insertTextFormat':2,'textEdit':{'range':{'start':{'line':0,'character':0},'end':{'line':0,'character':6}},'newText':BUDGET if uri.endswith('/budget.py') else SYNTAX}}
 elif method=='workspace/executeCommand':raise AssertionError('Snippet evaluation must never execute provider commands')
 elif method=='shutdown':result=None
 else:result=None
 send({'jsonrpc':'2.0','id':identity,'result':result})
</ui:copy>
<ui:file path="scripts/snippet-regex-controlled-provider.py"><ui:paste select="#snippet-regex-child-fixture"/></ui:file>

## Refuse unsupported or forged program policies before child resolution

Standard and omitted/explicit legacy version-2 queries must not expose known unsupported inline edits. A version-3 query retains its own immutable policy. Wrong policies and source stamps refuse before any child resolve; the successful path performs exactly one resolve and returns engine-compiled version-3 metadata.

<ui:copy id="snippet-regex-protocol-driver">
import assert from 'node:assert/strict';import{readFile}from'node:fs/promises';import{createHash}from'node:crypto';import{pathToFileURL}from'node:url';import{join}from'node:path';
const source='OldOld\r\nsecond\n',expected='OldOld|OldOld|OldOLD|OLD|old_old|oldold|word';const url=new URL('/api/ws?doc=workspace',process.argv[2]);url.protocol='ws:';const socket=new WebSocket(url);socket.binaryType='arraybuffer';let serial=0;const pending=new Map();
function send(body){const bytes=new TextEncoder().encode(JSON.stringify({jsonrpc:'2.0',...body})),frame=new Uint8Array(bytes.length+1);frame[0]=2;frame.set(bytes,1);socket.send(frame);}
function request(method,params){return new Promise((resolve,reject)=>{const id=++serial,timer=setTimeout(()=>{pending.delete(id);reject(Error(method+' timed out'));},20000);pending.set(id,{resolve,reject,timer});send({id,method,params});});}
socket.addEventListener('message',event=>{if(!(event.data instanceof ArrayBuffer))return;const bytes=new Uint8Array(event.data);if(bytes[0]!==2)return;const value=JSON.parse(new TextDecoder().decode(bytes.subarray(1))),wait=pending.get(value.id);if(!wait)return;pending.delete(value.id);clearTimeout(wait.timer);value.error?wait.reject(Error(value.error.message)):wait.resolve(value.result);});
const items=value=>Array.isArray(value)?value:value?.items??[],query=(uri,programs)=>({textDocument:{uri},position:{line:0,character:0},context:{triggerKind:1},...(programs?{transform_program_versions:programs}:{})});
function bind(item,programs=[2,3]){return{...item,data:{...item.data,hickory_context:{version:1,expected:item.data.hickory_context,selection:{from:0,to:6},clock:null,random_hex:null,clipboard:null,transform_program_versions:programs}}};}
async function prepare(uri){const deadline=Date.now()+20000;let revision;while(Date.now()<deadline){const ack=await request('hick/captureDocument',{textDocument:{uri,languageId:'python',version:1,text:source}});if(ack.processed&&ack.current){assert.equal(ack.source_files_modified,false);assert.equal(ack.source_hash,'sha256:'+createHash('sha256').update(source).digest('hex'));if(revision)assert.equal(ack.source_revision,revision);revision=ack.source_revision;try{const offered=items(await request('hick/completion',query(uri,[2,3])));if(offered.length)return offered[0];}catch(error){if(!/prepar|source.*changed|closed|mapping/i.test(String(error)))throw error;}}await new Promise(resolve=>setTimeout(resolve,25));}throw Error('Current v3 provider did not become ready');}
try{await new Promise((resolve,reject)=>{socket.addEventListener('open',resolve,{once:true});socket.addEventListener('error',reject,{once:true});});
 for(const path of['regex.py','inline.py']){const uri='hick:///'+path;send({method:'textDocument/didOpen',params:{textDocument:{uri,languageId:'python',version:1,text:source}}});const item=await prepare(uri);assert.deepEqual(item.data.hickory_transform_program_versions,[2,3]);
  if(path==='inline.py'){assert.deepEqual(items(await request('textDocument/completion',query(uri))),[]);assert.deepEqual(items(await request('hick/completion',query(uri,[2]))),[]);assert.deepEqual(items(await request('hick/completion',query(uri))),[]);}
  for(const programs of[[],[4],[3,3],[true]])await assert.rejects(request('hick/completion',query(uri,programs)),/program|version/i);
  await assert.rejects(request('completionItem/resolve',bind(item,[2])),/program|support/i);const forged=bind(item);forged.data.hickory_transform_program_versions=[3];await assert.rejects(request('completionItem/resolve',forged),/policy|program/i);
  const stale=bind(item);stale.data.hickory_context.expected={...stale.data.hickory_context.expected,source_revision:'b'.repeat(64)};await assert.rejects(request('completionItem/resolve',stale),/source|stamp|changed/i);
  const untouched=JSON.stringify(item),resolved=await request('completionItem/resolve',bind(item));assert.equal(JSON.stringify(item),untouched);assert.equal(resolved.data.hickory_snippet.text,expected);assert.deepEqual(resolved.data.hickory_transform_program_versions,[2,3]);const fields=resolved.data.hickory_snippet.fields.filter(field=>field.transform);assert.equal(fields.length,5);for(const field of fields){assert.equal(field.transform.version,3);assert.equal(field.transform.unicode_map,'hickory-regex-ucd-17.0.0-v1');}
  const events=(await readFile(process.argv[3],'utf8')).trim().split('\n').map(JSON.parse);assert.ok(events.some(event=>event.initialize_snippets===true));assert.equal(events.filter(event=>event.resolve===pathToFileURL(join(process.argv[4],path)).href).length,1,'Forged support/source replies must not reach child resolve');
 }
 console.log('PASS: real child snippet initialization, v3 named/lookbehind/backref/v-set/string/modifier expansion, exact held policy, old/omitted2 refusal, original opaque data and source revision guards');
}finally{for(const wait of pending.values()){clearTimeout(wait.timer);wait.reject(Error('Closed'));}socket.close();}
</ui:copy>
<ui:file path="scripts/snippet-regex-protocol-driver.mjs"><ui:paste select="#snippet-regex-protocol-driver"/></ui:file>

## Drive the existing compact interfaces

The browser opens the file, chooses an existing completion/template preview, types linked fields and uses Undo/Redo. It retains editor identity and geometry and verifies disk bytes remain untouched. The native modes invoke named actual-host cases through the shared strict receipt runner; they do not synthesize framework XML. Fixtures carry an owned marker and are removed only by their own temporary-directory lifetime.

<ui:copy id="snippet-regex-live-runner">
"""Owned real-engine v3 completion/template host flows; no canned engine responses."""
import argparse,importlib.util,json,os,socket,subprocess,sys,tempfile,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def module(path,name):
 spec=importlib.util.spec_from_file_location(name,path);value=importlib.util.module_from_spec(spec);spec.loader.exec_module(value);return value
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--host',choices=['protocol','browser','apple','android'],required=True);parser.add_argument('--template',action='store_true');args=parser.parse_args();fixture=module(ROOT/'scripts/test-simple-browser.py','simple_fixture');native=module(ROOT/'scripts/test-native-templates.py','native_fixture');cases=json.loads((ROOT/'apps/simple-contract/snippet-regex-live-cases.json').read_text())
 with tempfile.TemporaryDirectory(prefix='snippet-regex-v3-',dir=ROOT/'target') as temporary:
  folder=Path(temporary);root=folder/'workspace';root.mkdir();(root/'.git').mkdir();(root/'.hickory').mkdir();(root/'.hickory-snippet-regex-fixture').write_text('owned-snippet-regex-v3\n');events=folder/'provider-events.jsonl'
  for path in ['regex.py','inline.py','budget.py']:(root/path).write_bytes(cases['source'].encode())
  (root/'.hick-lsp.json').write_text(json.dumps({'servers':{'python':{'command':[sys.executable,str(ROOT/'scripts/snippet-regex-controlled-provider.py'),str(events)]}}}))
  (root/'.hickory/templates.json').write_text(json.dumps({'version':1,'templates':[{'id':'regex.mirrors','label':'Regex mirrors','description':'Lookbehind, escaped names, references, v sets, modifiers and emoji strings','language':'python','syntax':cases['syntax']},{'id':'regex.budget','label':'Budget refusal','description':'A failed calculation must refuse the entire field edit','language':'python','syntax':cases['budget_syntax']}]}))
  processes=[];driver=None
  try:
   with socket.socket() as reserve:reserve.bind(('127.0.0.1',0));port=reserve.getsockname()[1]
   ui=f'http://127.0.0.1:{port}';env={**os.environ,'HICKORY_STATE_DIR':str(folder/'state')};engine=fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(root),'window',ui],env=env);processes.append(engine);origin=fixture.origin(engine)
   if args.host=='protocol':subprocess.run(['node',str(ROOT/'scripts/snippet-regex-protocol-driver.mjs'),origin,str(events),str(root)],check=True,timeout=120)
   elif args.host=='browser':
    vite=fixture.start(['node','node_modules/vite/bin/vite.js','--config','vite.simple.config.ts','--host','127.0.0.1','--port',str(port),'--strictPort'],cwd=ROOT/'apps/web',env={**env,'HICKORY_API_ORIGIN':origin});processes.append(vite);driver=fixture.BrowserDriver(fixture.origin(vite)+'/simple.html')
    for path in ['regex.py','budget.py']:
     driver.act('open',path=path);driver.act('remember-editor',path=path);size=driver.act('editor-size',path=path)
     if args.template:
      driver.act('editor-position',path=path,anchor=0,head=6);catalog=driver.act('templates-open');label='Regex mirrors' if path=='regex.py' else 'Budget refusal';assert label in catalog,catalog;driver.act('templates-preview',label=label);assert driver.act('construction-source',path=path)==cases['source'];driver.act('templates-apply',path=path)
     else:
      deadline=time.monotonic()+20
      while True:
       options=driver.act('completion-open',path=path,offset=0,selection={'anchor':0,'head':6})
       if any('Regex mirrors' in option for option in options):break
       assert time.monotonic()<deadline,options;time.sleep(.1)
      notice=driver.act('completion-choose',title='Regex mirrors');assert 'Completion applied' in notice,notice
     expected=cases['after'] if path=='regex.py' else cases['budget_after'];assert driver.act('construction-source',path=path)==expected;assert driver.act('selected-source-text',path=path)=='OldOld'
     if path=='regex.py':
      driver.act('type-source-selection',path=path,text='NewNew');assert driver.act('construction-source',path=path)==cases['typed'];driver.act('undo',path=path);assert driver.act('construction-source',path=path)==expected;driver.act('redo',path=path);assert driver.act('construction-source',path=path)==cases['typed'];driver.act('undo',path=path)
      driver.act('type-source-selection',path=path,text='😀');assert driver.act('construction-source',path=path)==cases['emoji'];driver.act('undo',path=path);assert driver.act('construction-source',path=path)==expected
     else:
      driver.act('type-source-selection',path=path,text='a'*100+'!');assert driver.act('construction-source',path=path)==expected,'Refused VM calculation changed a primary field';assert driver.act('selected-source-text',path=path)=='OldOld'
     driver.act('undo',path=path);assert driver.act('construction-source',path=path)==cases['source'];driver.act('redo',path=path);assert driver.act('construction-source',path=path)==expected
     assert driver.act('editor-is-retained',path=path);assert driver.act('editor-size',path=path)==size
     if not args.template:
      observed=[json.loads(row) for row in events.read_text().splitlines()];assert any(row.get('initialize_snippets') is True for row in observed),observed;assert sum(row.get('resolve')==(root/path).resolve().as_uri() for row in observed)==1,observed
    print('PASS: rendered v3 '+('template' if args.template else 'completion')+' expansion, linked Unicode input, whole-edit budget refusal, atomic UndoRedo and retained code space/identity')
   else:
    if args.template:case=('SnippetRegexTests','testLiveRegexTemplatesAndHistory','regex-template') if args.host=='apple' else ('SnippetRegexTest','liveRegexTemplatesAndHistory','regex-template')
    else:case=('SnippetRegexTests','testLiveRegexProviderAndHistory','regex-provider') if args.host=='apple' else ('SnippetRegexTest','liveRegexProviderAndHistory','regex-provider')
    native.run_host(args.host,origin,root,folder,fixture,acceptance=case)
   for path in ['regex.py','inline.py','budget.py']:assert (root/path).read_bytes()==cases['source'].encode(),'V3 source preview/typing wrote disk bytes'
  finally:
   if driver:driver.close()
   for process in reversed(processes):fixture.stop(process)
if __name__=='__main__':main()
</ui:copy>
<ui:file path="scripts/test-snippet-regex-live.py"><ui:paste select="#snippet-regex-live-runner"/></ui:file>

## Use a mounted AppKit editor and the production language/controller path

The same native text view receives the real resolved source action and subsequent programmatic text input. Check calculated fields, Unicode input, refusal, history, scroll, editor identity and unchanged base/disk text. Template selection uses the existing controller, with no extra feature chrome. Physical keyboard, pointer, IME and assistive-technology acceptance remains a separate reading.

<ui:copy id="snippet-regex-live-apple">
import XCTest

@testable import HickorySimple

#if os(macOS)
  import AppKit
#endif
@MainActor final class SnippetRegexTests: XCTestCase {
  #if os(macOS)
    private func exercise(templates: Bool) async throws {
      guard let origin = ProcessInfo.processInfo.environment["HICKORY_NATIVE_TEMPLATE_ORIGIN"],
        let directory = ProcessInfo.processInfo.environment["HICKORY_NATIVE_TEMPLATE_WORKSPACE"]
      else { throw XCTSkip("Run the maintained snippet regex fixture") }
      let root = URL(fileURLWithPath: directory)
      XCTAssertEqual(
        try String(
          contentsOf: root.appendingPathComponent(".hickory-snippet-regex-fixture"), encoding: .utf8
        ), "owned-snippet-regex-v3\n")
      let dataURL = URL(fileURLWithPath: #filePath).deletingLastPathComponent()
        .deletingLastPathComponent().deletingLastPathComponent().appendingPathComponent(
          "simple-contract/snippet-regex-live-cases.json")
      let rawCases =
        try JSONSerialization.jsonObject(with: Data(contentsOf: dataURL)) as! [String: Any]
      let cases = rawCases.compactMapValues { $0 as? String }
      let engine = try Engine(address: origin)
      let tree = try await engine.files()
      _ = NSApplication.shared
      func find(_ path: String, _ nodes: [FileNode]) -> FileNode? {
        for node in nodes {
          if node.path == path { return node }
          if let child = find(path, node.children ?? []) { return child }
        }
        return nil
      }
      let events = root.deletingLastPathComponent().appendingPathComponent("provider-events.jsonl")
      func records() throws -> [[String: Any]] {
        guard FileManager.default.fileExists(atPath: events.path) else { return [] }
        return try String(contentsOf: events, encoding: .utf8).split(separator: "\n").map {
          try JSONSerialization.jsonObject(with: Data($0.utf8)) as! [String: Any]
        }
      }
      for path in ["regex.py", "budget.py"] {
        let file = try XCTUnwrap(find(path, tree.tree))
        let saved = try await engine.file(file)
        XCTAssertEqual(saved.content, cases["source"])
        let space = Workspace()
        space.engine = engine
        space.rootFiles = tree.tree
        space.sourceRootAddress = tree.root_path
        space.sourceWorkspaceID = tree.workspace_id
        let buffer = DocumentBuffer(file: file, text: saved)
        space.buffers[path] = buffer
        space.layout.activate(path, in: "document-group")
        let editor = MacEditor(buffer)
        let window = NSWindow(
          contentRect: NSRect(x: 0, y: 0, width: 700, height: 480), styleMask: [.titled],
          backing: .buffered, defer: false)
        window.contentView = editor.scroll
        editor.text.setSelectedRange(NSRange(location: 0, length: 6))
        buffer.selection = editor.text.selectedRange()
        let identity = try XCTUnwrap(buffer.sourceEditorState?()).identity
        let manager = try XCTUnwrap(editor.text.undoManager)
        let scroll = editor.scroll.contentView.bounds.origin
        let geometry = editor.scroll.frame
        var clipboardReads = 0
        let language = NativeLanguage(engine: engine, buffer: buffer)
        buffer.language = language
        language.snippetClipboardReader = {
          clipboardReads += 1
          return "must not be read"
        }
        language.start()
        defer {
          space.sourceTemplates.close()
          language.close()
          window.contentView = nil
        }
        if templates {
          let model = space.sourceTemplates
          model.clipboardReader = {
            clipboardReads += 1
            return "must not be read"
          }
          await model.open(space)
          let entry = try XCTUnwrap(
            model.catalog?.entries.first(where: {
              $0.id == (path == "regex.py" ? "regex.mirrors" : "regex.budget")
            }), model.notice)
          XCTAssertTrue(entry.available, entry.reason ?? model.notice)
          await model.choose(entry)
          XCTAssertNotNil(model.preview, model.notice)
          XCTAssertEqual(buffer.text, saved.content)
          let applied = await model.apply()
          XCTAssertTrue(applied, model.notice)
        } else {
          for _ in 0..<500 {
            if language.status.hasPrefix("LSP") { break }
            try await Task.sleep(for: .milliseconds(10))
          }
          await language.complete()
          for _ in 0..<500 {
            if !language.suggestions.isEmpty { break }
            try await Task.sleep(for: .milliseconds(10))
          }
          let offered = try XCTUnwrap(
            language.suggestions.first(where: { $0.label == "Regex mirrors" }), language.status)
          let accepted = await language.accept(offered)
          XCTAssertTrue(accepted, language.completionNotice)
        }
        let expected = try XCTUnwrap(cases[path == "regex.py" ? "after" : "budget_after"])
        XCTAssertEqual(buffer.text, expected)
        XCTAssertNotNil(buffer.snippets.active)
        XCTAssertEqual(editor.text.selectedRange(), NSRange(location: 0, length: 6))
        XCTAssertEqual(clipboardReads, 0)
        if path == "regex.py" {
          let active = try XCTUnwrap(buffer.snippets.active)
          XCTAssertEqual(active.fields.filter { $0.transform != nil }.count, 5)
          editor.text.insertText("NewNew", replacementRange: editor.text.selectedRange())
          XCTAssertEqual(buffer.text, cases["typed"])
          manager.undo()
          XCTAssertEqual(buffer.text, expected)
          manager.redo()
          XCTAssertEqual(buffer.text, cases["typed"])
          manager.undo()
          editor.text.insertText("😀", replacementRange: editor.text.selectedRange())
          XCTAssertEqual(buffer.text, cases["emoji"])
          manager.undo()
          XCTAssertEqual(buffer.text, expected)
        } else {
          editor.text.insertText(
            String(repeating: "a", count: 100) + "!", replacementRange: editor.text.selectedRange())
          XCTAssertEqual(
            buffer.text, expected, "A refused transform must not leave a primary-only edit")
          XCTAssertEqual(editor.text.selectedRange(), NSRange(location: 0, length: 6))
        }
        manager.undo()
        XCTAssertEqual(buffer.text, saved.content)
        manager.redo()
        XCTAssertEqual(buffer.text, expected)
        XCTAssertEqual(buffer.sourceEditorState?()?.identity, identity)
        XCTAssertEqual(editor.scroll.contentView.bounds.origin, scroll)
        XCTAssertEqual(editor.scroll.frame, geometry)
        XCTAssertTrue(editor.text.undoManager === manager)
        XCTAssertEqual(buffer.base, saved.content)
        let disk = try await engine.file(file)
        XCTAssertEqual(disk.content, saved.content)
        if !templates {
          let observed = try records()
          let uri = root.appendingPathComponent(path).standardizedFileURL.absoluteString
          XCTAssertTrue(observed.contains { $0["initialize_snippets"] as? Bool == true })
          XCTAssertEqual(observed.filter { $0["resolve"] as? String == uri }.count, 1)
        }
      }
    }
    func testLiveRegexProviderAndHistory() async throws { try await exercise(templates: false) }
    func testLiveRegexTemplatesAndHistory() async throws { try await exercise(templates: true) }
  #endif
}
</ui:copy>
<ui:file path="apps/apple-simple/Tests/SnippetRegexTests.swift"><ui:paste select="#snippet-regex-live-apple"/></ui:file>

## Retain Android state and verify its host/controller integration

Resolve through NativeLanguage and apply through the same source snippet controller used by the compact text-field adapter. Keep TextFieldState and scroll identity, validate linked input/history and require whole-action refusal on exhausted work. This JVM host/controller witness does not claim physical device or accessibility coverage.

<ui:copy id="snippet-regex-live-android">
package com.hickorydocs.simple
import org.junit.Assert.*
import org.junit.Test
import org.json.JSONObject
import java.io.File
class SnippetRegexTest {
 private suspend fun exercise(scope:kotlinx.coroutines.CoroutineScope,templates:Boolean){val origin=System.getenv("HICKORY_NATIVE_TEMPLATE_ORIGIN");org.junit.Assume.assumeNotNull(origin);val root=File(System.getenv("HICKORY_NATIVE_TEMPLATE_WORKSPACE")?:error("Run maintained regex fixture")).canonicalFile;assertEquals("owned-snippet-regex-v3\n",root.resolve(".hickory-snippet-regex-fixture").readText())
  val contract=generateSequence(File(System.getProperty("user.dir")).absoluteFile){it.parentFile}.map{File(it,"apps/simple-contract/snippet-regex-live-cases.json")}.first{it.isFile};val cases=JSONObject(contract.readText());val engine=Engine(origin!!);val (_,tree)=engine.files();fun find(path:String,nodes:List<FileNode> = tree):FileNode?{for(node in nodes){if(node.path==path)return node;find(path,node.children)?.let{return it}};return null}
  val events=root.parentFile.resolve("provider-events.jsonl");fun records()=if(events.isFile)events.readLines().map{JSONObject(it)}else emptyList()
  for(path in listOf("regex.py","budget.py")){val file=find(path)?:error("Regex fixture input missing");val saved=engine.file(file);assertEquals(cases.getString("source"),saved.content);val workspace=Workspace(scope);workspace.engine=engine;workspace.rootFiles=tree;val buffer=DocumentBuffer(file,saved);workspace.buffers[path]=buffer;workspace.show(path);val state=buffer.state;val scroll=buffer.sourceScroll.value;buffer.selectSourceRange(0,6,buffer.text)
   val language=NativeLanguage(engine,buffer,scope);buffer.language=language;var clipboardReads=0;engine.snippetClipboardReader={clipboardReads++;"must not be read"};language.start()
   try{if(templates){val model=workspace.sourceTemplates;workspace.showTemplates=true;model.refresh(workspace);kotlinx.coroutines.yield();repeat(500){if(model.catalog==null&&model.busy)kotlinx.coroutines.delay(20)};val entry=model.catalog?.entries?.singleOrNull{it.id==if(path=="regex.py")"regex.mirrors" else "regex.budget"}?:error(model.notice);assertTrue(entry.reason?:model.notice,entry.available);model.prepare(workspace,entry);assertNotNull(model.notice,model.reviewed);assertEquals(saved.content,buffer.text);assertTrue(model.notice,model.apply(workspace))}else{repeat(500){if(!language.connection.active||language.status!="LSP connected")kotlinx.coroutines.delay(10)};check(language.connection.active&&language.status=="LSP connected"){language.status};language.complete();repeat(500){if(language.suggestions.isEmpty())kotlinx.coroutines.delay(10)};val offered=language.suggestions.singleOrNull{it.label=="Regex mirrors"}?:error(language.status);assertTrue(language.completionNotice,language.accept(offered))}
    val expected=cases.getString(if(path=="regex.py")"after" else "budget_after");assertEquals(expected,buffer.text);assertNotNull(buffer.snippets.active);assertEquals(0,state.selection.min);assertEquals(6,state.selection.max);assertEquals(0,clipboardReads)
    if(path=="regex.py"){assertEquals(5,buffer.snippets.active!!.fields.count{it.transform!=null});assertTrue(buffer.snippets.applyInput(buffer.text,state.selection.min,state.selection.max,"NewNew"));assertEquals(cases.getString("typed"),buffer.text);assertTrue(buffer.snippets.undoSource());assertEquals(expected,buffer.text);assertTrue(buffer.snippets.redoSource());assertEquals(cases.getString("typed"),buffer.text);assertTrue(buffer.snippets.undoSource());assertTrue(buffer.snippets.applyInput(buffer.text,state.selection.min,state.selection.max,"😀"));assertEquals(cases.getString("emoji"),buffer.text);assertTrue(buffer.snippets.undoSource());assertEquals(expected,buffer.text)}else{assertFalse(runCatching{buffer.snippets.applyInput(buffer.text,state.selection.min,state.selection.max,"a".repeat(100)+"!")}.getOrDefault(false));assertEquals("A refused calculation must preserve primary and computed fields",expected,buffer.text);assertEquals(0,state.selection.min);assertEquals(6,state.selection.max)}
    assertTrue(buffer.snippets.undoSource());assertEquals(saved.content,buffer.text);assertTrue(buffer.snippets.redoSource());assertEquals(expected,buffer.text);assertSame(state,buffer.state);assertEquals(scroll,buffer.sourceScroll.value);assertEquals(saved.content,buffer.base);assertEquals(saved.content,engine.file(file).content)
    if(!templates){val observed=records();assertTrue(observed.any{it.optBoolean("initialize_snippets")});assertEquals(1,observed.count{it.optString("resolve")==("file://"+root.resolve(path).canonicalFile.toURI().rawPath)})}
   }finally{language.close()}
  }
 }
 @Test(timeout=120000)fun liveRegexProviderAndHistory()=kotlinx.coroutines.runBlocking{exercise(this,false)}
 @Test(timeout=120000)fun liveRegexTemplatesAndHistory()=kotlinx.coroutines.runBlocking{exercise(this,true)}
}
</ui:copy>
<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/SnippetRegexTest.kt"><ui:paste select="#snippet-regex-live-android"/></ui:file>
</ui:doc>
