import XCTest
#if os(macOS)
import AppKit
import SwiftUI
import Darwin
#endif
@testable import HickorySimple
@MainActor final class SourceHistoryTests:XCTestCase {
    func saved(_ text:String)->[String:Any]{["kind":"file","text":text,"sha256":nativeSourceHash(text),"readonly":false,"executable":false,"mode":420]}
    func reading()->[String:Any]{["version":1,"history_protocol":1,"workspace":"workspace","namespace":"/fixture","path":"a.txt","current":saved("saved"),"current_identity":"physical-current","restore_target":"current-source-draft","entries":[["id":String(repeating:"1",count:32),"at":1,"outcome":"published","fingerprint":String(repeating:"a",count:64),"before":saved("old"),"after":saved("saved"),"available":true]],"notices":[]]}
    func draft(_ source:String)->[String:Any]{["version":1,"history_protocol":1,"workspace":"workspace","namespace":"/fixture","path":"a.txt","id":String(repeating:"1",count:32),"fingerprint":String(repeating:"a",count:64),"current_identity":"physical-current","current_hash":nativeSourceHash("saved"),"draft_hash":nativeSourceHash(source),"side":"before","source":"old","restore_target":"current-source-draft","source_effects":false,"consequences":"unknown","notice":"Restored to draft; compiler consequences are unknown."]}
    func testHistoryEvidenceMustMatchExactPathsAndSavedHashes()throws {
        let bytes=try JSONSerialization.data(withJSONObject:reading()),value=try JSONDecoder().decode(SourceHistoryReading.self,from:bytes)
        XCTAssertThrowsError(try validateSourceTestJSON(Data(#"{"history_protocol":1e0}"#.utf8)));XCTAssertTrue(validSourceHistory(value,path:"a.txt"));XCTAssertFalse(validSourceHistory(value,path:"A.txt"))
        var corrupt=value;corrupt.entries[0].before.text="tampered";XCTAssertFalse(validSourceHistory(corrupt,path:"a.txt"));corrupt=value;corrupt.entries[0].outcome="failed";XCTAssertFalse(validSourceHistory(corrupt,path:"a.txt"))
        var malformed=value;malformed.entries[0].id=String(repeating:"😀",count:32);XCTAssertFalse(validSourceHistory(malformed,path:"a.txt"));malformed=value;malformed.entries[0].fingerprint=String(repeating:"A",count:64);XCTAssertFalse(validSourceHistory(malformed,path:"a.txt"))
        var alias=value;alias.path="é.txt";XCTAssertFalse(validSourceHistory(alias,path:"e\u{301}.txt"))
    }
    #if os(macOS)
    func native()->(Workspace,DocumentBuffer,MacEditor,NSWindow){
        _ = NSApplication.shared;let config=URLSessionConfiguration.ephemeral;config.protocolClasses=[ResponseProtocol.self]
        let space=Workspace(),engine=try! Engine(address:"http://fixture.invalid",session:URLSession(configuration:config)),file=FileNode(name:"a.txt",path:"a.txt",dir:false),buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:"draft",hash:"base")),editor=MacEditor(buffer)
        buffer.base="saved";space.engine=engine;space.sourceRootAddress="/fixture";space.buffers[file.path]=buffer;space.layout.activate(file.path,in:"document-group")
        let window=NSWindow(contentRect:NSRect(x:0,y:0,width:640,height:420),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=editor.scroll;editor.text.setSelectedRange(NSRange(location:5,length:0));buffer.selection=editor.text.selectedRange();return(space,buffer,editor,window)
    }
    func install(_ source:String)throws{ResponseProtocol.status=200;ResponseProtocol.beforeResponse=nil;ResponseProtocol.resetRequests();ResponseProtocol.responsesByPath=["/api/source-history":try JSONSerialization.data(withJSONObject:reading()),"/api/source-history/restore-draft":try JSONSerialization.data(withJSONObject:draft(source))]}
    func testStrictCreationRetainsExactConsumedIntentAndDirectConfirmedBaseline()async throws {
        let(space,buffer,editor,window)=native();defer{window.contentView=nil;ResponseProtocol.beforeResponse=nil;ResponseProtocol.responsesByPath=[:]}
        space.sourceWorkspaceID="runtime-workspace";buffer.savedGuard="captured-before";ResponseProtocol.resetRequests();ResponseProtocol.responsesByPath=[:];ResponseProtocol.status=500;ResponseProtocol.response=Data(#"{"error":"reply lost"}"#.utf8)
        do{_ = try await space.createSourceCopy(buffer,path:"copy.txt");XCTFail("Lost response cannot confirm creation")}catch{}
        let attempt=try XCTUnwrap(buffer.creationAttempt),original=try XCTUnwrap(ResponseProtocol.requestBodies("/api/source-history/create").last),identity=editor.text,selection=editor.text.selectedRange(),geometry=editor.scroll.frame
        XCTAssertTrue(buffer.dirty);XCTAssertEqual(buffer.base,"saved");XCTAssertNil(space.buffers["copy.txt"])
        ResponseProtocol.status=200;ResponseProtocol.response=try JSONSerialization.data(withJSONObject:["version":1,"history_protocol":1,"workspace":"runtime-workspace","namespace":"/fixture","path":"copy.txt","source":buffer.text,"source_hash":nativeSourceHash(buffer.text),"hash":"confirmed-hash","request_id":attempt.id,"save_guard":"confirmed-copy","history":["available":true,"outcome":"published"]])
        let created=try await space.createSourceCopy(buffer,path:"copy.txt")
        XCTAssertEqual(try JSONSerialization.data(withJSONObject:original,options:.sortedKeys),try JSONSerialization.data(withJSONObject:try XCTUnwrap(ResponseProtocol.requestBodies("/api/source-history/create").last),options:.sortedKeys));XCTAssertEqual(ResponseProtocol.requestCount("/api/file"),0);XCTAssertEqual(ResponseProtocol.requestCount("/api/files/op"),0)
        XCTAssertEqual(created.base,"draft");XCTAssertEqual(created.savedGuard,"confirmed-copy");XCTAssertFalse(created.dirty);XCTAssertNil(space.buffers["a.txt"]);XCTAssertTrue(space.buffers["copy.txt"] === buffer);XCTAssertTrue(created === buffer);XCTAssertTrue(editor.text === identity);XCTAssertEqual(editor.text.selectedRange(),selection);XCTAssertEqual(editor.scroll.frame,geometry)
    }
    func testCreationOwnershipCapturesDirtyConstructionAndRefusesUnboundBaseline()async throws {
        let(space,buffer,_,window)=native();defer{window.contentView=nil;ResponseProtocol.beforeResponse=nil;ResponseProtocol.responsesByPath=[:]}
        space.sourceWorkspaceID="runtime-workspace";let doc=DocumentBuffer(file:FileNode(name:"project.md",path:"project.md",dir:false,doc_id:"project"),text:FileText(path:"project.md",content:"saved construction",hash:"old"));doc.text="<h:file path=\"copy.txt\">draft output</h:file>";space.buffers["project.md"]=doc;ResponseProtocol.resetRequests();ResponseProtocol.status=409;ResponseProtocol.response=Data(#"{"error":"Original loaded physical guard is required for the selected construction."}"#.utf8)
        space.fileCommand="Save As";space.filePath="copy.txt";await space.performFileCommand();XCTAssertTrue(space.status.contains("Original loaded physical guard"),space.status);XCTAssertTrue(space.status.contains("Original edits are retained"));XCTAssertFalse(buffer.saving)
        let body=try XCTUnwrap(ResponseProtocol.requestBodies("/api/source-history/create").last),capture=try XCTUnwrap(body["ownership"] as? [String:Any]);XCTAssertEqual((capture["ownership_sources"] as? [String:String])?["project.md"],doc.text);XCTAssertNil((body["saved_guards"] as? [String:String])?["project.md"])
        XCTAssertEqual(ResponseProtocol.requestCount("/api/source-history/create"),1);XCTAssertEqual(ResponseProtocol.requestCount("/api/file"),0);XCTAssertEqual(ResponseProtocol.requestCount("/api/files/op"),0);XCTAssertNil(space.buffers["copy.txt"]);XCTAssertEqual(buffer.text,"draft");XCTAssertNil(doc.savedGuard);XCTAssertEqual(doc.text,"<h:file path=\"copy.txt\">draft output</h:file>")
    }
    func testOnlyReviewedDefinitiveFailureReleasesMatchingConsumedSaveAttempt()async throws {
        let(space,buffer,editor,window)=native();defer{window.contentView=nil;ResponseProtocol.beforeResponse=nil;ResponseProtocol.responsesByPath=[:]}
        buffer.savedGuard="captured-before";let id=String(repeating:"1",count:32);buffer.saveHistoryAttempt=SourceSaveAttempt(id:id,source:buffer.text,base:buffer.base,hash:buffer.hash,guardToken:"captured-before",revision:buffer.sourceEditIdentity)
        var payload=reading();var entries=payload["entries"] as! [[String:Any]];entries[0]["outcome"]="failed";entries[0]["available"]=false;entries[0]["retry_allowed"]=true;payload["entries"]=entries
        ResponseProtocol.status=200;ResponseProtocol.beforeResponse=nil;ResponseProtocol.responsesByPath=["/api/source-history":try JSONSerialization.data(withJSONObject:payload)];await space.sourceHistory.open(space)
        let chosen=try XCTUnwrap(space.sourceHistory.reading?.entries.first);space.sourceHistory.choose(chosen,side:"after");XCTAssertTrue(space.sourceHistory.allowNewSaveAttempt());XCTAssertNil(buffer.saveHistoryAttempt);XCTAssertEqual(buffer.text,"draft");XCTAssertEqual(buffer.base,"saved");XCTAssertEqual(buffer.savedGuard,"captured-before")
        buffer.saveHistoryAttempt=SourceSaveAttempt(id:id,source:buffer.text,base:buffer.base,hash:buffer.hash,guardToken:"captured-before",revision:buffer.sourceEditIdentity);var unknown=chosen;unknown.outcome="unknown";space.sourceHistory.choose(unknown,side:"after");XCTAssertFalse(space.sourceHistory.allowNewSaveAttempt());XCTAssertNotNil(buffer.saveHistoryAttempt);XCTAssertTrue(editor.text.string==buffer.text)
    }
    func testStrictSaveRetainsConsumedIntentAndNeverFallsBack()async throws {
        let(space,buffer,_,window)=native();defer{window.contentView=nil;ResponseProtocol.beforeResponse=nil;ResponseProtocol.responsesByPath=[:]}
        let engine=try XCTUnwrap(space.engine);buffer.savedGuard="captured-before";try install(buffer.text)
        ResponseProtocol.status=500;ResponseProtocol.response=Data(#"{"error":"response unavailable"}"#.utf8)
        await buffer.save(using:engine);let original=ResponseProtocol.captured
        XCTAssertEqual(buffer.base,"saved");XCTAssertEqual(buffer.savedGuard,"captured-before");XCTAssertTrue(buffer.dirty)
        ResponseProtocol.responsesByPath=[:];ResponseProtocol.status=200
        ResponseProtocol.response=try JSONSerialization.data(withJSONObject:["version":1,"history_protocol":1,"path":"a.txt","request_id":original["request_id"] ?? "","source_hash":nativeSourceHash("draft"),"save_guard":"confirmed-after","hash":"after-hash","history":["available":true,"outcome":"published"]])
        await buffer.save(using:engine)
        XCTAssertEqual(ResponseProtocol.requestedURL?.path,"/api/source-history/file");XCTAssertEqual(ResponseProtocol.captured["request_id"] as? String,original["request_id"] as? String);XCTAssertEqual(ResponseProtocol.captured["save_guard"] as? String,"captured-before");XCTAssertEqual(ResponseProtocol.requestCount("/api/file"),0)
        XCTAssertEqual(try JSONSerialization.data(withJSONObject:ResponseProtocol.captured,options:.sortedKeys),try JSONSerialization.data(withJSONObject:original,options:.sortedKeys))
        XCTAssertEqual(buffer.base,"draft");XCTAssertEqual(buffer.savedGuard,"confirmed-after");XCTAssertFalse(buffer.dirty)
    }
    func testMissingReadGuardAndOldDaemonKeepReadableDraft()async throws {
        let(space,buffer,_,window)=native();defer{window.contentView=nil;ResponseProtocol.beforeResponse=nil;ResponseProtocol.responsesByPath=[:]}
        let engine=try XCTUnwrap(space.engine);ResponseProtocol.resetRequests();await buffer.save(using:engine)
        XCTAssertEqual(ResponseProtocol.requestCount("/api/source-history/file"),0);XCTAssertTrue(buffer.dirty);XCTAssertEqual(buffer.text,"draft")
        buffer.savedGuard="captured";ResponseProtocol.status=404;ResponseProtocol.response=Data(#"{"error":"unsupported endpoint"}"#.utf8);ResponseProtocol.responsesByPath=[:]
        await buffer.save(using:engine);XCTAssertEqual(ResponseProtocol.requestedURL?.path,"/api/source-history/file");XCTAssertEqual(ResponseProtocol.requestCount("/api/file"),0);XCTAssertEqual(buffer.base,"saved");XCTAssertEqual(buffer.savedGuard,"captured");XCTAssertTrue(buffer.dirty)
    }
    func testHistoryPreviewAndRestoreRetainNativeDraftUndoAndEditor()async throws {
        let(space,buffer,editor,window)=native();defer{window.contentView=nil;ResponseProtocol.responsesByPath=[:]}
        editor.text.insertText("!",replacementRange:editor.text.selectedRange());let before=buffer.text,selection=editor.text.selectedRange(),identity=try XCTUnwrap(buffer.sourceEditorState?()).identity,geometry=editor.scroll.frame,manager=try XCTUnwrap(editor.text.undoManager)
        try install(before);await space.sourceHistory.open(space);space.sourceHistory.choose(try XCTUnwrap(space.sourceHistory.reading?.entries.first),side:"before")
        XCTAssertEqual(buffer.text,before);XCTAssertEqual(editor.text.selectedRange(),selection);XCTAssertTrue(manager.canUndo)
        let applied=await space.sourceHistory.restore();XCTAssertTrue(applied,space.sourceHistory.notice);XCTAssertEqual(buffer.text,"old");XCTAssertEqual(buffer.base,"saved");XCTAssertTrue(buffer.dirty)
        manager.undo();XCTAssertEqual(buffer.text,before);XCTAssertEqual(editor.text.selectedRange(),selection);manager.redo();XCTAssertEqual(buffer.text,"old")
        XCTAssertEqual(try XCTUnwrap(buffer.sourceEditorState?()).identity,identity);XCTAssertEqual(editor.scroll.frame,geometry)
    }
    func testMissingSourceRestoresOnlyAnUnmaterializedNativeDraft()async throws {
        let(space,buffer,editor,window)=native();defer{window.contentView=nil;ResponseProtocol.responsesByPath=[:]}
        var missing=reading();missing["current"]=["kind":"missing"];missing["restore_target"]="missing-source-draft"
        var reply=draft(buffer.text);reply["current_hash"]="missing";reply["restore_target"]="missing-source-draft"
        ResponseProtocol.status=200;ResponseProtocol.responsesByPath=["/api/source-history":try JSONSerialization.data(withJSONObject:missing),"/api/source-history/restore-draft":try JSONSerialization.data(withJSONObject:reply)]
        let identity=try XCTUnwrap(buffer.sourceEditorState?()).identity,geometry=editor.scroll.frame
        await space.sourceHistory.open(space);space.sourceHistory.choose(try XCTUnwrap(space.sourceHistory.reading?.entries.first),side:"before")
        let restored=await space.sourceHistory.restore();XCTAssertTrue(restored,space.sourceHistory.notice);XCTAssertEqual(buffer.text,"old");XCTAssertTrue(buffer.resourceDeleted);XCTAssertNil(buffer.savedGuard);XCTAssertEqual(buffer.sourceEditorState?()?.identity,identity);XCTAssertEqual(editor.scroll.frame,geometry)
        editor.text.undoManager?.undo();XCTAssertEqual(buffer.text,"draft");XCTAssertTrue(buffer.resourceDeleted)
        await space.save();XCTAssertEqual(space.fileCommand,"Save As")
    }
    func testClosedEditedAndReplacedEditorCannotRestoreHeldHistory()async throws {
        let(space,buffer,editor,window)=native();defer{window.contentView=nil;ResponseProtocol.beforeResponse=nil;ResponseProtocol.responsesByPath=[:]}
        try install(buffer.text);await space.sourceHistory.open(space);space.sourceHistory.choose(try XCTUnwrap(space.sourceHistory.reading?.entries.first),side:"before")
        editor.text.insertText("!",replacementRange:editor.text.selectedRange());let refused=await space.sourceHistory.restore();XCTAssertFalse(refused);XCTAssertEqual(buffer.text,"draft!")
        try install(buffer.text);await space.sourceHistory.open(space);space.sourceHistory.choose(try XCTUnwrap(space.sourceHistory.reading?.entries.first),side:"before")
        ResponseProtocol.beforeResponse={request in if request.url?.path=="/api/source-history/restore-draft"{space.sourceHistory.close()}}
        let closed=await space.sourceHistory.restore();XCTAssertFalse(closed);XCTAssertEqual(buffer.text,"draft!")
        ResponseProtocol.beforeResponse=nil;try install(buffer.text);await space.sourceHistory.open(space);space.sourceHistory.choose(try XCTUnwrap(space.sourceHistory.reading?.entries.first),side:"before");buffer.resourceDeleted=true
        let deleted=await space.sourceHistory.restore();XCTAssertFalse(deleted);XCTAssertEqual(buffer.text,"draft!")
    }
    func testLiveNativeOrdinarySaveHistoryRestoreAndUndo()async throws {
        guard let origin=ProcessInfo.processInfo.environment["HICKORY_NATIVE_HISTORY_ORIGIN"] else{throw XCTSkip("Opt in with an isolated ordinary-history engine fixture.")}
        _ = NSApplication.shared;NSApp.setActivationPolicy(.regular)
        guard let workspace=ProcessInfo.processInfo.environment["HICKORY_NATIVE_HISTORY_WORKSPACE"],let canonical=workspace.withCString({realpath($0,nil)}) else{throw EngineError(message:"Actual history writes require an owned workspace fixture.")};defer{free(canonical)}
        let engine=try Engine(address:origin),space=Workspace();space.engine=engine
        let marker=try await engine.file(FileNode(name:".hickory-native-history-fixture",path:".hickory-native-history-fixture",dir:false));XCTAssertEqual(marker.content,"owned-native-history-v1\n");guard marker.content=="owned-native-history-v1\n" else{return}
        let listing=try await engine.files();space.rootFiles=listing.tree;space.sourceRootAddress=listing.root_path;space.sourceWorkspaceID=listing.workspace_id;XCTAssertEqual(listing.root_path,String(cString:canonical));guard listing.root_path==String(cString:canonical) else{return}
        func find(_ nodes:[FileNode])->FileNode?{for node in nodes{if node.path=="history.txt"{return node};if let child=find(node.children ?? []){return child}};return nil}
        let file=try XCTUnwrap(find(space.rootFiles)),initial=try await engine.file(file),buffer=try await space.loadBuffer(file);XCTAssertFalse(buffer.readOnly);space.layout.activate(file.path,in:"document-group")
        let host=NSHostingView(rootView:WorkspaceView(workspace:space)),window=NSWindow(contentRect:NSRect(x:0,y:0,width:840,height:560),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=host;window.makeKeyAndOrderFront(nil);host.layoutSubtreeIfNeeded()
        defer{space.sourceHistory.close();space.debugger?.close();space.sourceTestIndex?.stop();for item in space.buffers.values{item.language?.close()};window.orderOut(nil)}
        func editor(_ view:NSView)->NSTextView?{if let text=view as? NSTextView{return text};return view.subviews.lazy.compactMap{editor($0)}.first}
        for _ in 0..<200{if editor(host) != nil{break};try await Task.sleep(for:.milliseconds(10))}
        let native=try XCTUnwrap(editor(host)),scroll=try XCTUnwrap(native.enclosingScrollView),identity=try XCTUnwrap(buffer.sourceEditorState?()).identity,geometry=scroll.convert(scroll.bounds,to:host)
        native.setSelectedRange(NSRange(location:0,length:native.string.utf16.count));native.insertText("saved-after\n",replacementRange:native.selectedRange());await buffer.save(using:engine);XCTAssertTrue(buffer.error.isEmpty,buffer.error);XCTAssertFalse(buffer.dirty)
        native.setSelectedRange(NSRange(location:native.string.utf16.count,length:0));native.insertText("unsaved",replacementRange:native.selectedRange());let draft=buffer.text,selection=native.selectedRange(),manager=try XCTUnwrap(native.undoManager)
        try XCTUnwrap(space.commands().first(where:{$0.id=="source.history"})).run();for _ in 0..<200{if !space.sourceHistory.busy&&space.sourceHistory.reading != nil{break};try await Task.sleep(for:.milliseconds(10))}
        let entry=try XCTUnwrap(space.sourceHistory.reading?.entries.first,space.sourceHistory.notice);XCTAssertEqual(entry.outcome,"published");XCTAssertTrue(entry.available);XCTAssertEqual(entry.before.text,initial.content);space.sourceHistory.choose(entry,side:"before");XCTAssertEqual(buffer.text,draft)
        let restored=await space.sourceHistory.restore();XCTAssertTrue(restored,space.sourceHistory.notice);XCTAssertEqual(buffer.text,initial.content);XCTAssertEqual(buffer.base,"saved-after\n");let disk=try await engine.file(file);XCTAssertEqual(disk.content,"saved-after\n")
        manager.undo();XCTAssertEqual(buffer.text,draft);XCTAssertEqual(native.selectedRange(),selection);manager.redo();XCTAssertEqual(buffer.text,initial.content)
        XCTAssertEqual(try XCTUnwrap(buffer.sourceEditorState?()).identity,identity);XCTAssertEqual(scroll.convert(scroll.bounds,to:host),geometry);XCTAssertTrue(editor(host) === native)
        space.sourceHistory.close();let copied=buffer.text;space.fileCommand="Save As";space.filePath="history-copy.txt";await space.performFileCommand()
        XCTAssertTrue(space.currentBuffer === buffer,space.status);XCTAssertNil(space.buffers["history.txt"]);XCTAssertEqual(buffer.file.path,"history-copy.txt");XCTAssertEqual(buffer.base,copied);XCTAssertNotNil(buffer.savedGuard);XCTAssertFalse(buffer.dirty)
        for _ in 0..<200{if editor(host) === native{break};try await Task.sleep(for:.milliseconds(10))}
        XCTAssertTrue(editor(host) === native);XCTAssertEqual(try XCTUnwrap(buffer.sourceEditorState?()).identity,identity);XCTAssertEqual(scroll.convert(scroll.bounds,to:host),geometry)
        manager.undo();XCTAssertEqual(buffer.text,draft);XCTAssertTrue(buffer.dirty);manager.redo();XCTAssertEqual(buffer.text,copied);XCTAssertFalse(buffer.dirty)
        let copiedDisk=try await engine.file(buffer.file),originalDisk=try await engine.file(file);XCTAssertEqual(copiedDisk.content,copied);XCTAssertEqual(originalDisk.content,"saved-after\n")
        await space.sourceHistory.open(space);let creation=try XCTUnwrap(space.sourceHistory.reading?.entries.first,space.sourceHistory.notice);XCTAssertEqual(creation.before.kind,"missing");XCTAssertEqual(creation.after.text,copied);XCTAssertTrue(creation.available)
        let missingPath=buffer.file.path,missingURL=URL(fileURLWithPath:workspace).appendingPathComponent(missingPath)
        native.setSelectedRange(NSRange(location:native.string.utf16.count,length:0));native.insertText("lost draft",replacementRange:native.selectedRange());let missingDraft=buffer.text
        try FileManager.default.removeItem(at:missingURL);await space.sourceHistory.open(space);let missingEntry=try XCTUnwrap(space.sourceHistory.reading?.entries.first,space.sourceHistory.notice);XCTAssertEqual(space.sourceHistory.reading?.current.kind,"missing");space.sourceHistory.choose(missingEntry,side:"after")
        let missingRestored=await space.sourceHistory.restore();XCTAssertTrue(missingRestored,space.sourceHistory.notice);XCTAssertEqual(buffer.text,copied);XCTAssertTrue(buffer.unmaterializedHistoryDraft);XCTAssertNil(buffer.savedGuard);XCTAssertFalse(FileManager.default.fileExists(atPath:missingURL.path))
        manager.undo();XCTAssertEqual(buffer.text,missingDraft);manager.redo();XCTAssertEqual(buffer.text,copied);XCTAssertFalse(FileManager.default.fileExists(atPath:missingURL.path));XCTAssertEqual(buffer.sourceEditorState?()?.identity,identity);XCTAssertEqual(scroll.convert(scroll.bounds,to:host),geometry)
        space.chooseFileCommand("Save As");space.filePath="recovered λ%2F?#.txt";await space.performFileCommand();XCTAssertEqual(buffer.file.path,"recovered λ%2F?#.txt",space.status);XCTAssertFalse(buffer.unmaterializedHistoryDraft);XCTAssertFalse(FileManager.default.fileExists(atPath:missingURL.path));XCTAssertEqual(try String(contentsOf:URL(fileURLWithPath:workspace).appendingPathComponent(buffer.file.path),encoding:.utf8),copied)
    }
    #endif
}
