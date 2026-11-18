<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Accept context-bound provider snippets through native editors

For maintainers checking the path from a real language child through native completion admission, local variable capture and coordinated source history. These opt-in witnesses use only the marked fixture created by the maintained native template runner. They do not replace provider resolution with a test closure. The controlled child asserts its original opaque item is unchanged; event counts establish one child resolve even when the engine asks the host for clipboard context. On 2026-11-17 both exact real-engine cases passed with zero skips using `python3 scripts/test-native-templates.py apple --provider-context` and its Android counterpart, with the current context engine explicitly selected. Each case accepted three provider offers: captured variables plus an editable/derived group, explicit clipboard denial with fallback and notice, and a selected-text branch whose unused clipboard default never reads the clipboard. The macOS case mounts the real editor; Android exercises its native source-state controller. Both retain authored CRLF bytes on disk and ordinary Undo/Redo. Durable logs and native receipts are `target/native-template-smoke/apple-provider-last.{log,json}` and `android-provider-last.{log,xml}`. The JSON is an explicitly identified XCTest-console receipt with a verified log hash; it is not framework-produced XML. Physical native input, IME, operating-system clipboard permissions, accessibility and iOS runtime remain unverified.

<ui:copy id="native-snippet-context-provider-apple">
import XCTest
#if os(macOS)
import AppKit
#endif
@testable import HickorySimple
@MainActor final class SnippetContextProviderTests:XCTestCase {
 #if os(macOS)
 func testLiveProviderContextCompletionAndHistory()async throws {
  guard let origin=ProcessInfo.processInfo.environment["HICKORY_NATIVE_TEMPLATE_ORIGIN"],let directory=ProcessInfo.processInfo.environment["HICKORY_NATIVE_TEMPLATE_WORKSPACE"] else{throw XCTSkip("Run the maintained native template fixture with --provider-context")}
  let root=URL(fileURLWithPath:directory),marker=root.appendingPathComponent(".hickory-native-template-fixture")
  XCTAssertEqual(try String(contentsOf:marker,encoding:.utf8),"owned-native-templates-v1\n")
  let events=root.deletingLastPathComponent().appendingPathComponent("provider-events.jsonl")
  func resolves(_ path:String)throws->Int{guard FileManager.default.fileExists(atPath:events.path) else{return 0};return try String(contentsOf:events,encoding:.utf8).split(separator:"\n").filter{line in guard let data=String(line).data(using:.utf8),let value=try? JSONSerialization.jsonObject(with:data) as? [String:Any],let uri=value["resolve"] as? String else{return false};return uri.hasSuffix("/"+path)}.count}
  let engine=try Engine(address:origin),tree=try await engine.files();_ = NSApplication.shared
  func find(_ path:String,_ nodes:[FileNode])->FileNode?{for node in nodes{if node.path==path{return node};if let match=find(path,node.children ?? []){return match}};return nil}
  for path in ["variables.py","denied.py","unused.py"]{
   let file=try XCTUnwrap(find(path,tree.tree)),saved=try await engine.file(file),buffer=DocumentBuffer(file:file,text:saved),editor=MacEditor(buffer)
   XCTAssertEqual(saved.content,"old\r\nsecond\n")
   let window=NSWindow(contentRect:NSRect(x:0,y:0,width:640,height:420),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=editor.scroll
   let language=NativeLanguage(engine:engine,buffer:buffer);buffer.language=language;language.start()
   defer{language.close();window.contentView=nil}
   var reads=0;language.snippetClipboardReader={reads+=1;return path=="denied.py" ? nil:"copied"}
   editor.text.setSelectedRange(NSRange(location:0,length:3));buffer.selection=editor.text.selectedRange()
   let identity=try XCTUnwrap(buffer.sourceEditorState?()).identity,manager=try XCTUnwrap(editor.text.undoManager),before=try resolves(path),scroll=editor.scroll.contentView.bounds.origin
   for _ in 0..<500{if language.status.hasPrefix("LSP"){break};try await Task.sleep(for:.milliseconds(10))};XCTAssertTrue(language.status.hasPrefix("LSP"),language.status)
   await language.complete();for _ in 0..<500{if !language.suggestions.isEmpty{break};try await Task.sleep(for:.milliseconds(10))};XCTAssertEqual(buffer.text,saved.content);XCTAssertEqual(buffer.selection,NSRange(location:0,length:3));let offered=try XCTUnwrap(language.suggestions.first(where:{$0.label=="context completion"}),language.status)
   let accepted=await language.accept(offered);XCTAssertTrue(accepted,language.completionNotice);XCTAssertEqual(try resolves(path),before+1)
   if path=="variables.py"{
    let line=buffer.text.components(separatedBy:"\r\n")[0],parts=line.components(separatedBy:"|");XCTAssertEqual(parts.count,7);guard parts.count==7 else{continue}
    XCTAssertEqual(parts[0],path);XCTAssertEqual(parts[1],String(Calendar.current.component(.year,from:Date())));XCTAssertNotNil(parts[2].range(of:"^[0-9a-f]{6}$",options:.regularExpression));XCTAssertEqual(Array(parts[3...6]),["old","old","OLD","copied"]);XCTAssertEqual(reads,1)
    let active=try XCTUnwrap(buffer.snippets.active,"Provider transform metadata must survive draft offset mapping");XCTAssertEqual(active.fields.filter{$0.transform != nil}.count,1);XCTAssertNotNil(buffer.snippets.commit)
    editor.text.insertText("new",replacementRange:editor.text.selectedRange());XCTAssertEqual(buffer.text,parts[0...2].joined(separator:"|")+"|new|new|NEW|copied\r\nsecond\n")
    manager.undo();XCTAssertEqual(buffer.text,line+"\r\nsecond\n");manager.undo();XCTAssertEqual(buffer.text,saved.content);manager.redo();manager.redo();XCTAssertEqual(buffer.text,parts[0...2].joined(separator:"|")+"|new|new|NEW|copied\r\nsecond\n")
   }else if path=="denied.py"{XCTAssertEqual(reads,1);XCTAssertEqual(buffer.text,"fallback\r\nsecond\n");XCTAssertTrue(language.completionNotice.contains("Clipboard unavailable"));manager.undo();XCTAssertEqual(buffer.text,saved.content);manager.redo();XCTAssertEqual(buffer.text,"fallback\r\nsecond\n")}
   else{XCTAssertEqual(reads,0);XCTAssertEqual(buffer.text,saved.content)}
   let disk=try await engine.file(file);XCTAssertEqual(buffer.base,saved.content);XCTAssertEqual(disk.content,saved.content);XCTAssertEqual(buffer.sourceEditorState?()?.identity,identity);XCTAssertEqual(editor.scroll.contentView.bounds.origin,scroll)
  }
 }
 #endif
}
</ui:copy>

The Android witness uses its real native language transport and existing source draft controller. It retains the TextFieldState, source scroll and base bytes while ordinary Undo/Redo restores derived occurrences. This controller acceptance does not establish physical Compose pointer, IME, clipboard permission prompts or accessibility behavior.

<ui:copy id="native-snippet-context-provider-android">
package com.hickorydocs.simple
import org.junit.Assert.*
import org.junit.Test
import org.json.JSONObject
import androidx.compose.ui.text.TextRange
class SnippetContextProviderTest {
 @Test(timeout=120000) fun liveProviderContextCompletionAndHistory()=kotlinx.coroutines.runBlocking {
  val origin=System.getenv("HICKORY_NATIVE_TEMPLATE_ORIGIN");org.junit.Assume.assumeNotNull(origin)
  val root=java.io.File(System.getenv("HICKORY_NATIVE_TEMPLATE_WORKSPACE") ?: error("Use maintained native template fixture --provider-context")).canonicalFile
  check(root.resolve(".hickory-native-template-fixture").readText()=="owned-native-templates-v1\n")
  val events=root.parentFile.resolve("provider-events.jsonl")
  fun resolves(path:String)=if(!events.exists())0 else events.readLines().count{runCatching{JSONObject(it).optString("resolve").endsWith("/$path")}.getOrDefault(false)}
  val engine=Engine(origin!!);val (_,tree)=engine.files()
  fun find(path:String,nodes:List<FileNode> = tree):FileNode?{for(node in nodes){if(node.path==path)return node;find(path,node.children)?.let{return it}};return null}
  for(path in listOf("variables.py","denied.py","unused.py")){
   val file=find(path) ?: error("Provider fixture input missing: $path");val saved=engine.file(file);assertEquals("old\r\nsecond\n",saved.content)
   val buffer=DocumentBuffer(file,saved);val state=buffer.state;val scroll=buffer.sourceScroll.value;buffer.selectSourceRange(0,3,buffer.text)
   val language=NativeLanguage(engine,buffer,this);buffer.language=language;language.start();var reads=0;engine.snippetClipboardReader={reads++;if(path=="denied.py")null else "copied"}
   try{
    repeat(500){if(!language.connection.active||language.status!="LSP connected")kotlinx.coroutines.delay(10)};check(language.connection.active&&language.status=="LSP connected"){language.status}
    val before=resolves(path);language.complete();val offered=language.suggestions.singleOrNull{it.label=="context completion"} ?: error(language.status)
    assertTrue(language.completionNotice,language.accept(offered));assertEquals(before+1,resolves(path))
    if(path=="variables.py"){
     val line=buffer.text.substringBefore("\r\n");val parts=line.split('|');assertEquals(7,parts.size);assertEquals(path,parts[0]);assertEquals(java.time.ZonedDateTime.now().year.toString(),parts[1]);assertTrue(parts[2].matches(Regex("[0-9a-f]{6}")));assertEquals(listOf("old","old","OLD","copied"),parts.drop(3));assertEquals(1,reads)
     assertTrue(buffer.snippets.applyInput(buffer.text,state.selection.min,state.selection.max,"new"));val after=parts.take(3).joinToString("|")+"|new|new|NEW|copied\r\nsecond\n";assertEquals(after,buffer.text)
     assertTrue(buffer.snippets.undoSource());assertEquals(line+"\r\nsecond\n",buffer.text);assertTrue(buffer.snippets.undoSource());assertEquals(saved.content,buffer.text);repeat(2){assertTrue(buffer.snippets.redoSource())};assertEquals(after,buffer.text)
    }else if(path=="denied.py"){assertEquals(1,reads);assertEquals("fallback\r\nsecond\n",buffer.text);assertTrue(language.completionNotice.contains("Clipboard unavailable"));assertTrue(buffer.snippets.undoSource());assertEquals(saved.content,buffer.text);assertTrue(buffer.snippets.redoSource());assertEquals("fallback\r\nsecond\n",buffer.text)}
    else{assertEquals(0,reads);assertEquals(saved.content,buffer.text)}
    assertSame(state,buffer.state);assertEquals(scroll,buffer.sourceScroll.value);assertEquals(saved.content,buffer.base);assertEquals(saved.content,engine.file(file).content)
   }finally{language.close()}
  }
 }
}
</ui:copy>

<ui:file path="apps/apple-simple/Tests/SnippetContextProviderTests.swift"><ui:paste select="#native-snippet-context-provider-apple"/></ui:file>
<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/SnippetContextProviderTest.kt"><ui:paste select="#native-snippet-context-provider-android"/></ui:file>
</ui:doc>
