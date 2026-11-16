import XCTest
#if os(macOS)
import AppKit
import SwiftUI
import Darwin
#endif
@testable import HickorySimple
@MainActor final class SourceTemplateTests:XCTestCase {
    func reply(_ source:String="value = 'old'\n")->([String:Any],[String:Any]) {
        let entry:[String:Any]=["id":"workspace.wrap","label":"Wrap selection","description":"Mirrored selected text","language":"python","origin":"workspace","available":true]
        let catalog:[String:Any]=["version":1,"template_protocol":1,"path":"main.py","selection":["from":9,"to":12],"source_hash":nativeSourceHash(source),"workspace_id":"fixture-workspace","language":"python","catalog_fingerprint":"catalog","available":true,"notices":[],"entries":[entry]]
        var plan=catalog;plan.removeValue(forKey:"entries");plan["template_id"]="workspace.wrap";plan["origin"]="workspace";plan["replacement"]=["from":9,"to":12]
        plan["snippet"]=["version":1,"syntax":"${1:${TM_SELECTED_TEXT}} + $1$0","text":"old + old","fields":[["id":0,"parent":NSNull(),"index":1,"from":0,"to":3,"choices":[]],["id":1,"parent":NSNull(),"index":1,"from":6,"to":9,"choices":[]],["id":2,"parent":NSNull(),"index":0,"from":9,"to":9,"choices":[]]]]
        return(catalog,plan)
    }
    func install(_ catalog:[String:Any],_ plan:[String:Any])throws {
        ResponseProtocol.status=200;ResponseProtocol.beforeResponse=nil;ResponseProtocol.resetRequests()
        ResponseProtocol.responsesByPath=["/api/templates/catalog":try JSONSerialization.data(withJSONObject:catalog),"/api/templates/plan":try JSONSerialization.data(withJSONObject:plan)]
    }
    func decode<T:Decodable>(_ value:[String:Any],as type:T.Type)throws->T {try JSONDecoder().decode(type,from:JSONSerialization.data(withJSONObject:value))}
    func testPlanBytesAndExactSelectionAreRequired()throws {
        let (raw,body)=reply(),catalog=try decode(raw,as:SourceTemplateCatalog.self),plan=try decode(body,as:SourceTemplatePlan.self),entry=try XCTUnwrap(catalog.entries.first)
        XCTAssertTrue(sourceTemplateCatalogMatches(catalog,path:"main.py",source:"value = 'old'\n",selection:NSRange(location:9,length:3)));XCTAssertTrue(sourceTemplatePlanMatches(plan,catalog:catalog,entry:entry))
        var changed=plan;changed.replacement.to=13;XCTAssertFalse(sourceTemplatePlanMatches(changed,catalog:catalog,entry:entry));changed=plan;changed.origin="builtin";XCTAssertFalse(sourceTemplatePlanMatches(changed,catalog:catalog,entry:entry))
        var composed=plan;composed.snippet.syntax="$0é";composed.snippet.text="é";composed.snippet.fields=[SourceTemplateField(id:0,parent:nil,index:0,from:0,to:0,choices:[])]
        var decomposed=composed;decomposed.snippet.text="e\u{301}";XCTAssertFalse(composed.same(decomposed));decomposed=composed;decomposed.snippet.syntax="$0e\u{301}";XCTAssertFalse(composed.same(decomposed))
        XCTAssertThrowsError(try validateSourceTestJSON(Data(#"{"template_protocol":1e0}"#.utf8)))
        changed=plan;changed.snippet.fields[0].to=99;XCTAssertThrowsError(try sourceTemplateDraft(changed,source:"value = 'old'\n",label:"Bad field"))
    }
    #if os(macOS)
    func native()->(Workspace,DocumentBuffer,MacEditor,NSWindow) {
        _ = NSApplication.shared
        let configuration=URLSessionConfiguration.ephemeral;configuration.protocolClasses=[ResponseProtocol.self]
        let space=Workspace(),engine=try! Engine(address:"http://fixture.invalid",session:URLSession(configuration:configuration)),file=FileNode(name:"main.py",path:"main.py",dir:false),buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:"value = 'old'\n",hash:"base")),editor=MacEditor(buffer)
        space.engine=engine;space.buffers[file.path]=buffer;space.layout.activate(file.path,in:"document-group")
        let window=NSWindow(contentRect:NSRect(x:0,y:0,width:640,height:420),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=editor.scroll
        editor.text.setSelectedRange(NSRange(location:9,length:3));buffer.selection=editor.text.selectedRange()
        return(space,buffer,editor,window)
    }
    func testPreviewApplyAndLinkedInputRetainActualNativeHistory()async throws {
        let (space,buffer,editor,window)=native(),(catalog,plan)=reply();try install(catalog,plan);defer{window.contentView=nil;ResponseProtocol.responsesByPath=[:]}
        let native=editor.text,manager=try XCTUnwrap(native.undoManager),source=buffer.text,identity=try XCTUnwrap(buffer.sourceEditorState?()).identity
        await space.sourceTemplates.open(space);await space.sourceTemplates.choose(try XCTUnwrap(space.sourceTemplates.catalog?.entries.first));XCTAssertNotNil(space.sourceTemplates.preview,space.sourceTemplates.notice);XCTAssertEqual(buffer.text,source);XCTAssertFalse(manager.canUndo)
        let applied=await space.sourceTemplates.apply();XCTAssertTrue(applied,space.sourceTemplates.notice);XCTAssertEqual(buffer.text,"value = 'old + old'\n");XCTAssertEqual(native.selectedRange(),NSRange(location:9,length:3));XCTAssertNotNil(buffer.snippets.active)
        native.insertText("new",replacementRange:native.selectedRange());XCTAssertEqual(buffer.text,"value = 'new + new'\n");XCTAssertEqual(buffer.base,source)
        manager.undo();XCTAssertEqual(buffer.text,"value = 'old + old'\n");XCTAssertNotNil(buffer.snippets.active);manager.undo();XCTAssertEqual(buffer.text,source);XCTAssertNil(buffer.snippets.active)
        manager.redo();XCTAssertEqual(buffer.text,"value = 'old + old'\n");XCTAssertNotNil(buffer.snippets.active);manager.redo();XCTAssertEqual(buffer.text,"value = 'new + new'\n")
        XCTAssertTrue(editor.text === native);XCTAssertTrue(native.undoManager === manager);XCTAssertEqual(buffer.sourceEditorState?()?.identity,identity);XCTAssertEqual(ResponseProtocol.requestCount("/api/templates/plan"),2)
    }
    func testActualNativeUnicodeMirrorsRetainUTF16FieldsThroughHistory()async throws {
        let (space,buffer,editor,window)=native(),(catalog,plan)=reply();try install(catalog,plan);defer{window.contentView=nil;ResponseProtocol.responsesByPath=[:]}
        await space.sourceTemplates.open(space);await space.sourceTemplates.choose(try XCTUnwrap(space.sourceTemplates.catalog?.entries.first));let accepted=await space.sourceTemplates.apply();XCTAssertTrue(accepted,space.sourceTemplates.notice)
        let value="😀e\u{301}",manager=try XCTUnwrap(editor.text.undoManager)
        editor.text.insertText(value,replacementRange:editor.text.selectedRange());XCTAssertTrue(exactUTF8(buffer.text,"value = '\(value) + \(value)'\n"));XCTAssertEqual(editor.text.selectedRange(),NSRange(location:13,length:0));XCTAssertEqual(buffer.snippets.active?.fields.map{[$0.from,$0.to]},[[9,13],[16,20],[20,20]])
        manager.undo();XCTAssertTrue(exactUTF8(buffer.text,"value = 'old + old'\n"));XCTAssertEqual(buffer.snippets.active?.fields.map{[$0.from,$0.to]},[[9,12],[15,18],[18,18]])
        manager.redo();XCTAssertTrue(exactUTF8(buffer.text,"value = '\(value) + \(value)'\n"));XCTAssertEqual(buffer.snippets.active?.fields.map{[$0.from,$0.to]},[[9,13],[16,20],[20,20]])
    }
    func testSourceEditUndoSelectionAndCompositionRetirePreview()async throws {
        for change in ["editUndo","selection","composition","editor","model","closed"] {
            let (space,buffer,editor,window)=native(),(catalog,plan)=reply();try install(catalog,plan)
            await space.sourceTemplates.open(space);await space.sourceTemplates.choose(try XCTUnwrap(space.sourceTemplates.catalog?.entries.first));XCTAssertNotNil(space.sourceTemplates.preview)
            let source=buffer.text,original=try XCTUnwrap(buffer.sourceEditorState),state=try XCTUnwrap(original()),reviewed=space.sourceTemplates
            switch change {
            case "editUndo":buffer.text="changed";buffer.text=source
            case "selection":editor.text.setSelectedRange(NSRange(location:0,length:0));buffer.selection=editor.text.selectedRange()
            case "composition":buffer.sourceEditorState={var value=state;value.composing=true;return value}
            case "editor":buffer.sourceEditorState={var value=state;value.identity=UUID();return value}
            case "model":space.sourceTemplates=SourceTemplatesModel()
            default:space.close("main.py");space.show("main.py")
            }
            let before=ResponseProtocol.requestCount("/api/templates/plan"),accepted=await reviewed.apply();XCTAssertFalse(accepted,change);XCTAssertEqual(buffer.text,source,change);XCTAssertEqual(ResponseProtocol.requestCount("/api/templates/plan"),before,change);window.contentView=nil
        }
        ResponseProtocol.responsesByPath=[:]
    }
    func testProviderRequeryCannotRerollReviewedUnicodeOrApplyRevokedPlan()async throws {
        for revoked in [false,true] {
            let (space,buffer,editor,window)=native(),(catalog,raw)=reply();defer{_ = editor};var plan=raw
            plan["snippet"]=["version":1,"syntax":"$0é","text":"é","fields":[["id":0,"parent":NSNull(),"index":0,"from":0,"to":0,"choices":[]]]];try install(catalog,plan)
            await space.sourceTemplates.open(space);await space.sourceTemplates.choose(try XCTUnwrap(space.sourceTemplates.catalog?.entries.first));XCTAssertNotNil(space.sourceTemplates.preview)
            if revoked{ResponseProtocol.status=409;ResponseProtocol.responsesByPath["/api/templates/plan"]=Data(#"{"error":"Template provider revoked"}"#.utf8)}else{var snippet=plan["snippet"] as! [String:Any];snippet["text"]="e\u{301}";plan["snippet"]=snippet;ResponseProtocol.responsesByPath["/api/templates/plan"]=try JSONSerialization.data(withJSONObject:plan)}
            let accepted=await space.sourceTemplates.apply();XCTAssertFalse(accepted);XCTAssertEqual(buffer.text,"value = 'old'\n");XCTAssertNil(buffer.snippets.active);window.contentView=nil
        }
        ResponseProtocol.status=200;ResponseProtocol.responsesByPath=[:]
    }
    func testLiveNativeSourceTemplateCatalogAndHistory()async throws {
        guard let origin=ProcessInfo.processInfo.environment["HICKORY_NATIVE_TEMPLATE_ORIGIN"],let directory=ProcessInfo.processInfo.environment["HICKORY_NATIVE_TEMPLATE_WORKSPACE"] else{throw XCTSkip("Set HICKORY_NATIVE_TEMPLATE_ORIGIN/WORKSPACE to the maintained isolated native-template fixture.")}
        func canonicalPath(_ path:String)throws->String{let pointer=try XCTUnwrap(realpath(path,nil));defer{free(pointer)};return String(cString:pointer)}
        let canonicalWorkspace=try canonicalPath(directory),owned=URL(fileURLWithPath:directory),marker=owned.appendingPathComponent(".hickory-native-template-fixture")
        XCTAssertTrue(FileManager.default.fileExists(atPath:marker.path));guard FileManager.default.fileExists(atPath:marker.path) else{return}
        let manifest=owned.appendingPathComponent(".hickory/templates.json"),canonicalManifest=try canonicalPath(manifest.path);XCTAssertTrue(canonicalManifest.hasPrefix(canonicalWorkspace+"/"));guard canonicalManifest.hasPrefix(canonicalWorkspace+"/") else{return}
        let originalManifest=try Data(contentsOf:manifest);defer{try? originalManifest.write(to:manifest,options:.atomic)}
        let engine=try Engine(address:origin),space=Workspace(),path=ProcessInfo.processInfo.environment["HICKORY_NATIVE_TEMPLATE_INPUT"] ?? "main.py",tree=try await engine.files()
        func find(_ files:[FileNode],_ path:String)->FileNode?{for file in files{if exactUTF8(file.path,path){return file};if let child=find(file.children ?? [],path){return child}};return nil}
        let file=try XCTUnwrap(find(tree.tree,path));space.engine=engine;space.rootFiles=tree.tree;space.debugger=NativeDebugger(engine:engine)
        let buffer=try await space.loadBuffer(file);XCTAssertFalse(buffer.readOnly);space.layout.activate(file.path,in:"document-group")
        _ = NSApplication.shared;NSApp.setActivationPolicy(.regular);NSApp.activate(ignoringOtherApps:true)
        let host=NSHostingView(rootView:WorkspaceView(workspace:space)),window=NSWindow(contentRect:NSRect(x:0,y:0,width:700,height:480),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=host;window.makeKeyAndOrderFront(nil);host.layoutSubtreeIfNeeded()
        defer{space.sourceTemplates.close();space.debugger?.close();space.sourceTestIndex?.stop();for item in space.buffers.values{item.language?.close()};window.orderOut(nil)}
        func editor(_ view:NSView)->NSTextView?{if let text=view as? NSTextView{return text};return view.subviews.lazy.compactMap{editor($0)}.first}
        for _ in 0..<200{if editor(host) != nil{break};try await Task.sleep(for:.milliseconds(10))}
        let native=try XCTUnwrap(editor(host)),manager=try XCTUnwrap(native.undoManager),scroll=try XCTUnwrap(native.enclosingScrollView),source=buffer.text
        XCTAssertEqual(source,"value = 'old'\n");native.insertText("# prior history\n",replacementRange:NSRange(location:native.string.utf16.count,length:0));manager.undo();XCTAssertEqual(buffer.text,source);XCTAssertTrue(manager.canRedo)
        native.setSelectedRange(NSRange(location:9,length:3));buffer.selection=native.selectedRange();let geometry=scroll.convert(scroll.bounds,to:host),identity=try XCTUnwrap(buffer.sourceEditorState?()).identity
        let command=try XCTUnwrap(space.commands().first(where:{$0.id=="source.templates"}));XCTAssertNil(command.reason);command.run()
        let model=space.sourceTemplates;for _ in 0..<400{if model.catalog != nil{break};try await Task.sleep(for:.milliseconds(25))}
        let catalog=try XCTUnwrap(model.catalog,model.notice),entry=try XCTUnwrap(catalog.entries.first(where:{exactUTF8($0.id,"workspace.wrap")}),model.notice)
        XCTAssertEqual(catalog.workspace_id,nativeSourceHash(canonicalWorkspace),"Owned fixture workspace must match the provider catalog");guard exactUTF8(catalog.workspace_id,nativeSourceHash(canonicalWorkspace)) else{return}
        let unsupported=try XCTUnwrap(catalog.entries.first(where:{exactUTF8($0.id,"workspace.unsupported")}));XCTAssertFalse(unsupported.available);XCTAssertFalse(unsupported.reason?.isEmpty ?? true)
        await model.choose(entry);XCTAssertEqual(model.preview?.snippet.text,"old + old",model.notice);XCTAssertEqual(buffer.text,source);XCTAssertTrue(manager.canRedo);XCTAssertEqual(native.selectedRange(),NSRange(location:9,length:3))
        let accepted=await model.apply();XCTAssertTrue(accepted,model.notice);guard accepted else{return}
        XCTAssertEqual(buffer.text,"value = 'old + old'\n");XCTAssertEqual(native.selectedRange(),NSRange(location:9,length:3));XCTAssertNotNil(buffer.snippets.active)
        native.insertText("new",replacementRange:native.selectedRange());XCTAssertEqual(buffer.text,"value = 'new + new'\n")
        manager.undo();XCTAssertEqual(buffer.text,"value = 'old + old'\n");XCTAssertNotNil(buffer.snippets.active);manager.undo();XCTAssertEqual(buffer.text,source);XCTAssertNil(buffer.snippets.active)
        manager.redo();manager.redo();XCTAssertEqual(buffer.text,"value = 'new + new'\n");XCTAssertNotNil(buffer.snippets.active)
        XCTAssertTrue(editor(host) === native);XCTAssertTrue(native.undoManager === manager);XCTAssertEqual(buffer.sourceEditorState?()?.identity,identity);XCTAssertEqual(scroll.convert(scroll.bounds,to:host),geometry);XCTAssertEqual(buffer.base,source)
        manager.undo();manager.undo();native.setSelectedRange(NSRange(location:9,length:3));buffer.selection=native.selectedRange()
        await model.open(space);await model.choose(try XCTUnwrap(model.catalog?.entries.first(where:{exactUTF8($0.id,"workspace.wrap")})));XCTAssertNotNil(model.preview,model.notice)
        try Data(#"{"version":1,"templates":[]}"#.utf8).write(to:manifest,options:.atomic)
        let revoked=await model.apply();XCTAssertFalse(revoked);XCTAssertEqual(buffer.text,source);XCTAssertTrue(manager.canRedo);XCTAssertNil(buffer.snippets.active)
        try originalManifest.write(to:manifest,options:.atomic)
        await model.open(space);await model.choose(try XCTUnwrap(model.catalog?.entries.first(where:{exactUTF8($0.id,"workspace.wrap")})));XCTAssertNotNil(model.preview,model.notice)
        native.setSelectedRange(NSRange(location:0,length:0));buffer.selection=native.selectedRange();let stale=await model.apply();XCTAssertFalse(stale);XCTAssertEqual(buffer.text,source);XCTAssertTrue(manager.canRedo)
        XCTAssertTrue(editor(host) === native);XCTAssertEqual(scroll.convert(scroll.bounds,to:host),geometry);model.close()
        let disk=try await engine.file(file);XCTAssertEqual(disk.content,source,"Template application is a local draft, not a save")
        if let generated=find((try await engine.files()).tree,"generated.py") {await space.open(generated);let readonly=try XCTUnwrap(space.currentBuffer);XCTAssertTrue(readonly.readOnly);let before=readonly.text;XCTAssertNotNil(space.commands().first(where:{$0.id=="source.templates"})?.reason);await model.open(space);XCTAssertNil(model.catalog);XCTAssertEqual(readonly.text,before)}else{XCTFail("Fixture must include generated.py for readonly refusal")}
    }
    #endif
}
