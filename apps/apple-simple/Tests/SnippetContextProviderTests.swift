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
