import XCTest
import SwiftUI
#if os(macOS)
import AppKit
#endif
@testable import HickorySimple
@MainActor final class ActionChoiceTests:XCTestCase {
    #if os(macOS)
    func testLiveOwnedBulkFixCommandReviewApplyAndUndo()async throws {
        guard let origin=ProcessInfo.processInfo.environment["HICKORY_BULK_ORIGIN"],let root=ProcessInfo.processInfo.environment["HICKORY_BULK_WORKSPACE"] else{throw XCTSkip("Use the maintained owned bulk-fix fixture.")}
        let directory=URL(fileURLWithPath:root);guard try String(contentsOf:directory.appendingPathComponent(".hickory-bulk-fixture"),encoding:.utf8)=="owned-bulk-v1\n" else{throw EngineError(message:"An owned disposable workspace is required.")}
        _ = NSApplication.shared;NSApp.setActivationPolicy(.regular);let e=try Engine(address:origin),w=Workspace();w.engine=e;let listing=try await e.files();w.rootFiles=listing.tree;w.sourceRootAddress=listing.root_path;w.sourceWorkspaceID=listing.workspace_id
        let f=try XCTUnwrap(w.rootFiles.first{$0.path=="a.py"}),b=try await w.loadBuffer(f),source=b.text;w.layout.activateSource(f.path,in:"document-group")
        let host=NSHostingView(rootView:WorkspaceView(workspace:w)),window=NSWindow(contentRect:NSRect(x:0,y:0,width:920,height:620),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=host;window.makeKeyAndOrderFront(nil);host.layoutSubtreeIfNeeded()
        defer{for buffer in w.buffers.values{buffer.language?.close()};w.sourceTestIndex?.stop();window.orderOut(nil);window.contentView=nil}
        for _ in 0..<200{if b.sourceEditorState?() != nil{break};try await Task.sleep(for:.milliseconds(20))};let editor=try XCTUnwrap(b.sourceEditorState?());let size=host.bounds.size
        _ = b.selectSourceRange?(NSRange(location:2,length:0),source);let selection=b.selection
        try XCTUnwrap(w.commands().first{$0.id=="symbol.fixAll"}).run();XCTAssertEqual(w.changes.mode,"fixAll")
        for _ in 0..<200{await w.changes.prepare(w);if !w.changes.choices.isEmpty{break};try await Task.sleep(for:.milliseconds(100))}
        XCTAssertEqual(w.changes.choices.count,1,w.changes.notice);await w.changes.choose(0,workspace:w);XCTAssertEqual(w.changes.plan?.patches.count,2,w.changes.notice);XCTAssertEqual(b.text,source);XCTAssertEqual(b.selection,selection);XCTAssertEqual(b.sourceEditorState?()?.identity,editor.identity);XCTAssertEqual(host.bounds.size,size)
        XCTAssertEqual(try String(contentsOf:directory.appendingPathComponent("a.py"),encoding:.utf8),source);XCTAssertEqual(try String(contentsOf:directory.appendingPathComponent("other.py"),encoding:.utf8),source)
        w.changes.apply(w);XCTAssertEqual(b.text,"python_fixed = 1\n");XCTAssertEqual(w.buffers["other.py"]?.text,"other_fixed = 1\n");w.changes.open("undo",workspace:w);XCTAssertEqual(b.text,source);XCTAssertEqual(w.buffers["other.py"]?.text,source);w.changes.open("redo",workspace:w);XCTAssertEqual(b.text,"python_fixed = 1\n");XCTAssertEqual(b.sourceEditorState?()?.identity,editor.identity);XCTAssertEqual(host.bounds.size,size)
        XCTAssertEqual(try String(contentsOf:directory.appendingPathComponent("a.py"),encoding:.utf8),source);XCTAssertEqual(try String(contentsOf:directory.appendingPathComponent("other.py"),encoding:.utf8),source)
    }
    #endif

    func testCompleteFileFixChoiceFiltersPreviewsAndUsesCoordinatedUndo()async throws {
        let w=Workspace(),e=try Engine(address:"http://127.0.0.1:1"),f=FileNode(name:"a.py",path:"a.py",dir:false)
        let b=DocumentBuffer(file:f,text:FileText(path:f.path,content:"old\n",hash:"base"));b.language=NativeLanguage(engine:e,buffer:b);w.buffers[f.path]=b;w.show(f.path)
        let action:[String:Any]=["title":"Fix all","kind":"source.fixAll.provider","edit":["changes":[languageURI(f.path):[["range":["start":["line":0,"character":0],"end":["line":0,"character":3]],"newText":"new"]]]]]
        let choices=[action,["title":"single","kind":"quickfix"],["title":"unclassified"],["title":"prefix","kind":"source.fixAllWrong"]]
        w.changes.open("fixAll",workspace:w);await w.changes.prepare(w,proposal:{kind,_ in XCTAssertEqual(kind,"fixAll");return choices})
        XCTAssertEqual(w.changes.choices.count,1);await w.changes.choose(0,workspace:w);XCTAssertEqual(w.changes.plan?.patches.first?.after,"new\n");XCTAssertEqual(b.text,"old\n")
        w.changes.apply(w);XCTAssertEqual(b.text,"new\n");w.changes.open("undo",workspace:w);XCTAssertEqual(b.text,"old\n");w.changes.open("redo",workspace:w);XCTAssertEqual(b.text,"new\n")
        w.changes.open("fixAll",workspace:w);await w.changes.prepare(w,proposal:{_,_ in [["title":"ordinary","kind":"quickfix"]]});XCTAssertTrue(w.changes.choices.isEmpty);XCTAssertTrue(w.changes.notice.contains("no file-wide"))
        w.changes.open("fixAll",workspace:w);await w.changes.prepare(w,proposal:{_,_ in b.text="later";b.text="new\n";return [action]});XCTAssertTrue(w.changes.choices.isEmpty);XCTAssertNil(w.changes.plan)
        w.changes.open("fixAll",workspace:w);await w.changes.prepare(w,proposal:{_,_ in [action]},resolution:{_ in var changed=action;changed["kind"]="quickfix";return changed});await w.changes.choose(0,workspace:w);XCTAssertNil(w.changes.plan);XCTAssertEqual(b.text,"new\n")
    }

    func testUnopenedActionTargetsAcquireThenRequeryAndRefuseAmbiguousChoices()async throws {
        let workspace=Workspace(),engine=try Engine(address:"http://127.0.0.1:1"),file=FileNode(name:"a.py",path:"a.py",dir:false)
        let buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:"old\n",hash:"base"));buffer.language=NativeLanguage(engine:engine,buffer:buffer);workspace.buffers[file.path]=buffer;workspace.show(file.path)
        let changes=workspace.changes
        let action:[String:Any]=["title":"Fix both","kind":"quickfix","data":["hickory_action":"first"]]
        var queries=0,resolutions=0,loads:[String]=[];var ambiguous=false
        let propose:(String,String) async throws->Any={_,_ in queries+=1;return ambiguous && queries>1 ? [action,action]:[action]}
        let resolve:([String:Any]) async throws->[String:Any]={_ in
            resolutions+=1
            if resolutions==1{return ["title":"Fix both","data":["hickory_acquire":[["uri":languageURI("other.py")]]]]}
            let replacement:[String:Any]=["range":["start":["line":0,"character":0],"end":["line":0,"character":3]],"newText":"new"]
            return ["title":"Fix both","edit":["changes":[languageURI("a.py"):[replacement],languageURI("other.py"):[replacement]]]]
        }
        let load:(String) async throws->SourceSnapshot={path in
            loads.append(path);let target=DocumentBuffer(file:FileNode(name:path,path:path,dir:false),text:FileText(path:path,content:"old\n",hash:"base"));workspace.buffers[path]=target
            return SourceSnapshot(text:target.text,writable:true)
        }
        changes.open("actions",workspace:workspace);await changes.prepare(workspace,proposal:propose,resolution:resolve,acquisition:load);await changes.choose(0,workspace:workspace)
        XCTAssertEqual(queries,2);XCTAssertEqual(resolutions,2);XCTAssertEqual(loads,["other.py"]);XCTAssertEqual(changes.plan?.patches.count,2);XCTAssertEqual(workspace.currentBuffer?.file.path,"a.py");XCTAssertEqual(buffer.text,"old\n")
        workspace.buffers["other.py"]?.text="later\n";changes.apply(workspace);XCTAssertTrue(changes.notice.contains("Source changed"));XCTAssertEqual(buffer.text,"old\n")
        workspace.buffers.removeValue(forKey:"other.py");queries=0;ambiguous=true
        changes.open("actions",workspace:workspace);await changes.prepare(workspace,proposal:propose,resolution:resolve,acquisition:load);await changes.choose(0,workspace:workspace)
        XCTAssertNil(changes.plan);XCTAssertTrue(changes.notice.contains("ambiguous"));XCTAssertEqual(buffer.text,"old\n")
    }

    func testLatestResolvedChoiceWinsAndRenderedSnapshotsCannotBorrowNewLists()async throws {
        let workspace=Workspace(),engine=try Engine(address:"http://127.0.0.1:1"),file=FileNode(name:"a.py",path:"a.py",dir:false)
        let buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:"old\n",hash:"base"));buffer.language=NativeLanguage(engine:engine,buffer:buffer);workspace.buffers[file.path]=buffer;workspace.show(file.path)
        let changes=workspace.changes;changes.open("actions",workspace:workspace)
        let actions:[[String:Any]]=["first","second"].map {["title":$0,"data":["hickory_action":$0]]}
        var held:[String:CheckedContinuation<[String:Any],Error>]=[:]
        let resolve:([String:Any]) async throws->[String:Any]={action in try await withCheckedThrowingContinuation {held[action["title"] as! String]=$0}}
        func prepare()async{await changes.prepare(workspace,proposal:{_,_ in actions},resolution:resolve)}
        func settle(_ condition:@escaping ()->Bool)async{let end=Date().addingTimeInterval(2);while !condition()&&Date()<end{try? await Task.sleep(for:.milliseconds(1))}}
        func resolved(_ name:String)->[String:Any]{["title":name,"edit":["changes":[languageURI(file.path):[["range":["start":["line":0,"character":0],"end":["line":0,"character":3]],"newText":name]]]]]}
        await prepare();let snapshot=changes.actionSnapshot
        let first=Task{await changes.choose(0,workspace:workspace,expectedSnapshot:snapshot)};await settle{held["first"] != nil}
        let second=Task{await changes.choose(1,workspace:workspace,expectedSnapshot:snapshot)};await settle{held["second"] != nil}
        try XCTUnwrap(held["second"]).resume(returning:resolved("second"));await second.value
        try XCTUnwrap(held["first"]).resume(returning:resolved("first"));await first.value;XCTAssertEqual(changes.plan?.patches.first?.after,"second\n")
        held=[:];await prepare();await changes.choose(0,workspace:workspace,expectedSnapshot:snapshot);XCTAssertTrue(held.isEmpty);XCTAssertNil(changes.plan)
        let current=changes.actionSnapshot;let closing=Task{await changes.choose(0,workspace:workspace,expectedSnapshot:current)};await settle{held["first"] != nil};changes.close(workspace)
        try XCTUnwrap(held["first"]).resume(returning:resolved("first"));await closing.value;XCTAssertNil(changes.plan);XCTAssertEqual(buffer.text,"old\n")
        changes.open("actions",workspace:workspace)
        await changes.prepare(workspace,proposal:{_,_ in actions},resolution:{_ in var value=resolved("partial");value["command"]=["title":"Effect","command":"provider.effect"];return value})
        await changes.choose(0,workspace:workspace,expectedSnapshot:changes.actionSnapshot);XCTAssertNil(changes.plan);XCTAssertTrue(changes.notice.contains("provider command"));XCTAssertEqual(buffer.text,"old\n")
    }
}
