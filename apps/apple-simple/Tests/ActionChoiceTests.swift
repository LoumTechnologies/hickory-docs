import XCTest
import CoreFoundation
import SwiftUI
#if os(macOS)
import AppKit
#endif
@testable import HickorySimple
@MainActor final class ActionChoiceTests:XCTestCase {
    func testOriginalTargetOwnersNeverRenewDuringQueriesOrApproval()async throws {
        for change in ["undo","guard","baseline","handle","unknown"] {
            let w=Workspace(),e=try Engine(address:"http://127.0.0.1:1"),f=FileNode(name:"a.py",path:"a.py",dir:false),other=FileNode(name:"other.py",path:"other.py",dir:false)
            let b=DocumentBuffer(file:f,text:FileText(path:f.path,content:"old\n",hash:"base")),target=DocumentBuffer(file:other,text:FileText(path:other.path,content:"old\n",hash:"base"));b.language=NativeLanguage(engine:e,buffer:b);w.buffers[f.path]=b;w.buffers[other.path]=target;w.show(f.path)
            let action:[String:Any]=["title":"Fix all","kind":"source.fixAll.provider"]
            w.changes.open("fixAll",workspace:w);await w.changes.prepare(w,proposal:{_,_ in
                switch change{case "undo":target.text="later";target.text="old\n";case "guard":target.savedGuard="foreign";case "baseline":target.base="foreign";case "handle":w.buffers[other.path]=DocumentBuffer(file:other,text:FileText(path:other.path,content:target.text,hash:target.hash));default:w.buffers["unknown.md"]=DocumentBuffer(file:FileNode(name:"unknown.md",path:"unknown.md",dir:false),text:FileText(path:"unknown.md",content:"unmodeled construction",hash:"new"))};return [action]
            });XCTAssertTrue(w.changes.choices.isEmpty,change);XCTAssertNil(w.changes.plan,change);XCTAssertEqual(b.text,"old\n")
        }
    }
    func testAcquiredEditUndoCannotBecomeANewProviderBaseline()async throws {
        let w=Workspace(),e=try Engine(address:"http://127.0.0.1:1"),f=FileNode(name:"a.py",path:"a.py",dir:false),b=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:"old\n",hash:"base"));b.language=NativeLanguage(engine:e,buffer:b);w.buffers[f.path]=b;w.show(f.path)
        let action:[String:Any]=["title":"Fix all","kind":"source.fixAll.provider","data":["hickory_acquire":[["uri":languageURI("other.py")]]]]
        w.changes.open("fixAll",workspace:w);await w.changes.prepare(w,proposal:{_,_ in [action]},resolution:{$0},acquisition:{path in let target=DocumentBuffer(file:FileNode(name:path,path:path,dir:false),text:FileText(path:path,content:"old\n",hash:"base"));w.buffers[path]=target;target.text="later";target.text="old\n";return SourceSnapshot(text:target.text,writable:true)});await w.changes.choose(0,workspace:w);XCTAssertNil(w.changes.plan);XCTAssertEqual(b.text,"old\n");XCTAssertTrue(w.changes.notice.contains("acquired target") || w.changes.notice.contains("sealed"),w.changes.notice)
    }

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
        workspace.buffers["other.py"]?.text="later\n";changes.apply(workspace);XCTAssertTrue(changes.notice.contains("Source, selection or provider changed"));XCTAssertEqual(buffer.text,"old\n")
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
    func testProviderProofsSynchronizeReadonlyContextAndRefuseUnprovedOrChangedEdits()async throws {
        for mode in ["success","hash","missing","unproved","readonly"] {
            let w=Workspace(),engine=try Engine(address:"http://127.0.0.1:1"),f=FileNode(name:"a.py",path:"a.py",dir:false),b=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:"old\n",hash:"base"));b.language=NativeLanguage(engine:engine,buffer:b);w.buffers[f.path]=b;w.show(f.path)
            var queries=0;var loaded:[String]=[]
            func action()->[String:Any]{
                let target=mode=="readonly" && queries>1 ? "context.py":"a.py",replacement:[String:Any]=["range":["start":["line":0,"character":0],"end":["line":0,"character":3]],"newText":"new"]
                var rows:[[String:Any]]=[["uri":languageURI("a.py"),"source_hash":"sha256:"+(mode=="hash" ? String(repeating:"0",count:64):nativeSourceHash("old\n"))],["uri":languageURI("context.py"),"source_hash":"sha256:"+nativeSourceHash("old\n")]]
                if mode=="unproved"{rows.removeFirst()}
                var result:[String:Any]=["title":"Proof fix","kind":"quickfix","edit":["changes":[languageURI(target):[replacement]]],"data":["hickory_action":"owned","hickory_sources":rows]]
                if mode=="missing"{result.removeValue(forKey:"data")};return result
            }
            w.changes.open("actions",workspace:w)
            await w.changes.prepare(w,proposal:{_,_ in queries+=1;return [action()]},resolution:{$0},acquisition:{path in
                loaded.append(path);if let current=w.buffers[path]{return SourceSnapshot(text:current.text,writable:!current.readOnly)}
                let file=FileNode(name:path,path:path,dir:false,generated_by:"construction.md"),target=DocumentBuffer(file:FileNode(name:path,path:path,dir:false,generated_by:"construction.md"),text:FileText(path:path,content:"old\n",hash:"base"));w.buffers[file.path]=target;return SourceSnapshot(text:target.text,writable:false)
            },enforceProviderProofs:true)
            await w.changes.choose(0,workspace:w)
            XCTAssertEqual(b.text,"old\n",mode);XCTAssertEqual(w.currentBuffer?.file.path,"a.py",mode)
            if mode=="success"{XCTAssertEqual(w.changes.plan?.patches.first?.after,"new\n",w.changes.notice);XCTAssertTrue(loaded.contains("context.py"));XCTAssertTrue(w.buffers["context.py"]?.readOnly==true);w.changes.apply(w);XCTAssertEqual(b.text,"new\n");w.changes.open("undo",workspace:w);XCTAssertEqual(b.text,"old\n")}
            else{XCTAssertNil(w.changes.plan,mode+": "+w.changes.notice);XCTAssertFalse(w.changes.notice.isEmpty,mode)}
        }
    }
    func testBoundRenameFormattingRequireTypedNoEffectAndSourceProofs()async throws {
        for effect:Any in [false,true,0,"false",NSNull()] {
            let w=Workspace(),e=try Engine(address:"http://127.0.0.1:1"),f=FileNode(name:"a.py",path:"a.py",dir:false),b=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:"old\n",hash:"base"));b.language=NativeLanguage(engine:e,buffer:b);w.buffers[f.path]=b;w.show(f.path)
            let action:[String:Any]=["title":"Bound format","edit":["changes":[languageURI(f.path):[["range":["start":["line":0,"character":0],"end":["line":0,"character":3]],"newText":"new"]]]],"data":["hickory_action":"owned","hickory_sources":[["uri":languageURI(f.path),"source_hash":"sha256:"+nativeSourceHash(b.text)]]],"source_files_modified":effect]
            w.changes.open("format",workspace:w);await w.changes.prepare(w,proposal:{_,_ in action},acquisition:{_ in SourceSnapshot(text:b.text,writable:true)},enforceProviderProofs:true)
            if let number=effect as? NSNumber,CFGetTypeID(number)==CFBooleanGetTypeID(),!number.boolValue{XCTAssertEqual(w.changes.plan?.patches.first?.after,"new\n",w.changes.notice)}else{XCTAssertNil(w.changes.plan);XCTAssertTrue(w.changes.notice.contains("no-effect"),w.changes.notice)};XCTAssertEqual(b.text,"old\n")
        }
    }

    #if os(macOS)
    func testLiveOwnedBoundRenameFormatPreviewUndoAndStaleRefusal()async throws {
        let environment=ProcessInfo.processInfo.environment
        guard let address=environment["HICKORY_NATIVE_BOUND_ORIGIN"],let root=environment["HICKORY_NATIVE_BOUND_WORKSPACE"],let events=environment["HICKORY_NATIVE_BOUND_EVENTS"],let gate=environment["HICKORY_NATIVE_BOUND_GATE"] else{throw XCTSkip("Use the maintained owned native-bound fixture.")}
        let directory=URL(fileURLWithPath:root),parent=URL(fileURLWithPath:root).deletingLastPathComponent().standardizedFileURL
        guard try String(contentsOf:directory.appendingPathComponent(".hickory-native-bound-fixture"),encoding:.utf8)=="owned-native-bound-v1\n",URL(fileURLWithPath:events).deletingLastPathComponent().standardizedFileURL==parent,URL(fileURLWithPath:gate).deletingLastPathComponent().standardizedFileURL==parent else{throw EngineError(message:"An isolated owned bound-source workspace is required.")}
        _=NSApplication.shared;NSApp.setActivationPolicy(.regular);let e=try Engine(address:address),w=Workspace();w.engine=e;let listing=try await e.files();w.rootFiles=listing.tree;w.sourceRootAddress=listing.root_path;w.sourceWorkspaceID=listing.workspace_id
        let f=try XCTUnwrap(w.rootFiles.first{$0.path=="a.py"}),b=try await w.loadBuffer(f),source=b.text;w.showSource(f.path)
        let host=NSHostingView(rootView:WorkspaceView(workspace:w)),window=NSWindow(contentRect:NSRect(x:0,y:0,width:980,height:700),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=host;window.makeKeyAndOrderFront(nil);host.layoutSubtreeIfNeeded()
        defer{for item in w.buffers.values{item.language?.close()};w.sourceTestIndex?.stop();window.orderOut(nil);window.contentView=nil}
        func settle(_ message:String,_ condition:()->Bool)async throws{for _ in 0..<600{if condition(){return};try await Task.sleep(for:.milliseconds(20))};throw EngineError(message:message)}
        func native(_ view:NSView)->NSTextView?{if let value=view as? NSTextView{return value};return view.subviews.lazy.compactMap{native($0)}.first}
        try await settle("Original code editor did not mount"){b.sourceEditorState?()?.attached==true};let editor=try XCTUnwrap(native(host)),identity=try XCTUnwrap(b.sourceEditorState?()).identity,scroll=try XCTUnwrap(editor.enclosingScrollView),geometry=scroll.convert(scroll.bounds,to:host)
        func prepare(_ command:String,_ name:String)async throws {try XCTUnwrap(w.commands().first{$0.id==command}).run();w.changes.name=name;for _ in 0..<100{await w.changes.prepare(w);if w.changes.plan != nil{return};try await Task.sleep(for:.milliseconds(30))};throw EngineError(message:"Bound proposal unavailable: "+w.changes.notice)}
        editor.setSelectedRange(NSRange(location:2,length:0));let selection=b.selection
        try await prepare("symbol.rename","native_renamed");XCTAssertEqual(w.changes.plan?.patches.first?.before,source);XCTAssertEqual(b.text,source);XCTAssertEqual(b.selection,selection);w.changes.apply(w);XCTAssertEqual(b.text,"native_renamed = 1\n",w.changes.notice);w.changes.open("undo",workspace:w);XCTAssertEqual(b.text,source);w.changes.open("redo",workspace:w);XCTAssertEqual(b.text,"native_renamed = 1\n");w.changes.open("undo",workspace:w);XCTAssertEqual(b.text,source)
        try await prepare("document.format","");XCTAssertEqual(b.text,source);w.changes.apply(w);XCTAssertEqual(b.text,"formatted = 1\n",w.changes.notice);w.changes.open("undo",workspace:w);XCTAssertEqual(b.text,source);w.changes.open("redo",workspace:w);XCTAssertEqual(b.text,"formatted = 1\n");w.changes.open("undo",workspace:w);XCTAssertEqual(b.text,source)
        XCTAssertEqual(b.sourceEditorState?()?.identity,identity);XCTAssertEqual(scroll.convert(scroll.bounds,to:host),geometry)
        try XCTUnwrap(w.commands().first{$0.id=="symbol.rename"}).run();w.changes.name="held_native";let held=Task{await w.changes.prepare(w)}
        try await settle("Controlled rename did not reach its gate"){(try? String(contentsOfFile:events,encoding:.utf8))?.contains("\"held\": \"held_native\"")==true}
        editor.setSelectedRange(NSRange(location:editor.string.utf16.count,length:0));editor.insertText("!",replacementRange:editor.selectedRange());XCTAssertNotEqual(b.text,source);editor.undoManager?.undo();XCTAssertEqual(b.text,source);try Data("release".utf8).write(to:URL(fileURLWithPath:gate));await held.value
        XCTAssertNil(w.changes.plan);XCTAssertFalse(w.changes.notice.isEmpty);XCTAssertEqual(b.text,source);XCTAssertEqual(b.sourceEditorState?()?.identity,identity);XCTAssertEqual(scroll.convert(scroll.bounds,to:host),geometry);XCTAssertEqual(try String(contentsOf:directory.appendingPathComponent("a.py"),encoding:.utf8),source)
    }
    #endif

    #if os(macOS)
    func testLiveOwnedLiterateBulkFixAcquiresContributorAndKeepsProseEditor()async throws {
        let environment=ProcessInfo.processInfo.environment
        guard let address=environment["HICKORY_NATIVE_BOUND_ORIGIN"],let root=environment["HICKORY_NATIVE_BOUND_WORKSPACE"],let events=environment["HICKORY_NATIVE_BOUND_EVENTS"] else{throw XCTSkip("Use the maintained native-bound --literate fixture.")}
        let directory=URL(fileURLWithPath:root);guard try String(contentsOf:directory.appendingPathComponent(".hickory-native-bound-fixture"),encoding:.utf8)=="owned-native-bound-v1\n" else{throw EngineError(message:"An owned source fixture is required.")}
        _=NSApplication.shared;NSApp.setActivationPolicy(.regular);let e=try Engine(address:address),w=Workspace();w.engine=e;let listing=try await e.files();w.rootFiles=listing.tree;w.sourceRootAddress=listing.root_path;w.sourceWorkspaceID=listing.workspace_id
        let file=try XCTUnwrap(w.rootFiles.first{$0.path=="project.md"}),b=try await w.loadBuffer(file),source=b.text;w.showSource(file.path);let contributor=try String(contentsOf:directory.appendingPathComponent("a.md"),encoding:.utf8)
        let host=NSHostingView(rootView:WorkspaceView(workspace:w)),window=NSWindow(contentRect:NSRect(x:0,y:0,width:980,height:700),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=host;window.makeKeyAndOrderFront(nil);host.layoutSubtreeIfNeeded();defer{for item in w.buffers.values{item.language?.close()};w.sourceTestIndex?.stop();window.orderOut(nil);window.contentView=nil}
        func native(_ view:NSView)->NSTextView?{if let value=view as? NSTextView{return value};return view.subviews.lazy.compactMap{native($0)}.first}
        for _ in 0..<300{if b.sourceEditorState?()?.attached==true{break};try await Task.sleep(for:.milliseconds(20))};let editor=try XCTUnwrap(native(host)),identity=try XCTUnwrap(b.sourceEditorState?()).identity,scroll=try XCTUnwrap(editor.enclosingScrollView),geometry=scroll.convert(scroll.bounds,to:host)
        editor.setSelectedRange(NSRange(location:(source as NSString).range(of:"# Readable").location+3,length:0));let selection=b.selection;XCTAssertNil(w.buffers["a.md"])
        try XCTUnwrap(w.commands().first{$0.id=="symbol.fixAll"}).run()
        for _ in 0..<100{await w.changes.prepare(w);if w.changes.choices.contains(where:{($0["title"] as? String)?.hasSuffix(" · out/a.py")==true}){break};try await Task.sleep(for:.milliseconds(30))}
        let index=try XCTUnwrap(w.changes.choices.firstIndex{($0["title"] as? String)?.hasSuffix(" · out/a.py")==true});await w.changes.choose(index,workspace:w)
        XCTAssertEqual(w.changes.plan?.patches.map(\.path),["a.md"],w.changes.notice);XCTAssertEqual(w.buffers["a.md"]?.text,contributor);XCTAssertEqual(b.text,source);XCTAssertEqual(b.selection,selection);XCTAssertEqual(w.currentBuffer?.file.path,"project.md")
        w.changes.apply(w);XCTAssertEqual(w.buffers["a.md"]?.text,contributor.replacingOccurrences(of:"value",with:"fixed"),w.changes.notice);w.changes.open("undo",workspace:w);XCTAssertEqual(w.buffers["a.md"]?.text,contributor);w.changes.open("redo",workspace:w);XCTAssertEqual(w.buffers["a.md"]?.text,contributor.replacingOccurrences(of:"value",with:"fixed"));w.changes.open("undo",workspace:w)
        XCTAssertEqual(b.text,source);XCTAssertEqual(b.selection,selection);XCTAssertEqual(b.sourceEditorState?()?.identity,identity);XCTAssertEqual(scroll.convert(scroll.bounds,to:host),geometry);XCTAssertEqual(try String(contentsOf:directory.appendingPathComponent("project.md"),encoding:.utf8),source);XCTAssertEqual(try String(contentsOf:directory.appendingPathComponent("a.md"),encoding:.utf8),contributor);XCTAssertFalse(FileManager.default.fileExists(atPath:directory.appendingPathComponent("out").path));XCTAssertTrue((try String(contentsOfFile:events,encoding:.utf8)).contains("\"whole\": true"))
    }
    #endif

}
