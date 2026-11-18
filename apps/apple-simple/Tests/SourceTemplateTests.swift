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
        var catalog:[String:Any]=["version":1,"template_protocol":1,"path":"main.py","selection":["from":9,"to":12],"source_hash":nativeSourceHash(source),"workspace_id":"fixture-workspace","language":"python","catalog_fingerprint":"catalog","available":true,"notices":[],"entries":[entry]]
        catalog["transform_program_versions"]=supportedSnippetTransformPrograms;catalog["source_context"]=["version":1,"scope":"authored-source","language":"python","languages":["python"],"targets":["main.py"],"reason":NSNull(),"project_entry":NSNull(),"source_hashes":["main.py":nativeSourceHash(source)],"graph_fingerprint":String(repeating:"a",count:64),"fingerprint":String(repeating:"b",count:64)]
        var plan=catalog;plan.removeValue(forKey:"entries");plan["template_id"]="workspace.wrap";plan["origin"]="workspace";plan["replacement"]=["from":9,"to":12]
        plan["snippet"]=["version":1,"syntax":"${1:${TM_SELECTED_TEXT}} + $1$0","text":"old + old","fields":[["id":0,"parent":NSNull(),"index":1,"from":0,"to":3,"choices":[]],["id":1,"parent":NSNull(),"index":1,"from":6,"to":9,"choices":[]],["id":2,"parent":NSNull(),"index":0,"from":9,"to":9,"choices":[]]]]
        return(catalog,plan)
    }
    static let variables=SnippetVariableSnapshot(clock:SnippetVariableClock(unix_millis:1704067200000,utc_offset_minutes:0),random_hex:"0123456789abcdef0123456789abcdef")
    func install(_ originalCatalog:[String:Any],_ originalPlan:[String:Any])throws {
        var catalog=originalCatalog,plan=originalPlan
        catalog["template_protocol"]=2;plan["template_protocol"]=2;catalog["variable_context"]=Self.variables.body;plan["variable_context"]=Self.variables.body;catalog["source_files_modified"]=false;plan["source_files_modified"]=false
        ResponseProtocol.status=200;ResponseProtocol.beforeResponse=nil;ResponseProtocol.resetRequests()
        ResponseProtocol.responsesByPath=["/api/templates/catalog":try JSONSerialization.data(withJSONObject:catalog),"/api/templates/plan":try JSONSerialization.data(withJSONObject:plan)]
    }
    func decode<T:Decodable>(_ value:[String:Any],as type:T.Type)throws->T {try JSONDecoder().decode(type,from:JSONSerialization.data(withJSONObject:value))}
    func batchReply()->([String:Any],[String:Any]) {
        let source="old two";var (catalog,plan)=reply(source)
        let ranges:[[String:Int]]=[["from":0,"to":3],["from":4,"to":7]]
        let carets:[String:Any]=["version":1,"selections":ranges,"primary":0,"continuing":false]
        for isPlan in [false,true] {
            var value=isPlan ? plan:catalog;value["template_protocol"]=2;value["variable_context"]=Self.variables.body;value["source_files_modified"]=false;value["selection"]=ranges[0];value["template_batch_protocol"]=1;value["carets"]=carets
            var binding=value["source_context"] as! [String:Any];binding["scope"]="caret-batch";value["source_context"]=binding;binding.removeValue(forKey:"source_hashes")
            value["readings"]=ranges.map{["selection":$0,"context":binding] as [String:Any]}
            if isPlan {
                value.removeValue(forKey:"snippet");value.removeValue(forKey:"replacement")
                value["expansions"]=ranges.enumerated().map{index,range -> [String:Any] in
                    let selected=index==0 ? "old":"two",text=selected+":"+selected
                    return ["selection":range,"replacement":range,"snippet":["version":1,"syntax":"${1:"+selected+"}:$1$0","text":text,"fields":[["id":0,"parent":NSNull(),"index":1,"from":0,"to":3,"choices":[]],["id":1,"parent":NSNull(),"index":1,"from":4,"to":7,"choices":[]],["id":2,"parent":NSNull(),"index":0,"from":7,"to":7,"choices":[]]]]]
                };plan=value
            }else{catalog=value}
        }
        return(catalog,plan)
    }
    func testCompleteBatchRetainsDistinctOriginalSelectionsAndRejectsPartialReply()throws {
        let (raw,body)=batchReply(),catalog=try decode(raw,as:SourceTemplateCatalog.self),plan=try decode(body,as:SourceTemplateBatchReply.self)
        XCTAssertTrue(plan.matches(catalog,entry:catalog.entries[0]));let prepared=try plan.prepared(source:"old two",continuing:nil)
        XCTAssertEqual(prepared.plan.state.source,"old:old two:two");XCTAssertEqual(prepared.expansions.count,2)
        for mode in ["missing","range","primary","provisional","language","transform"]{
            var changed=body
            switch mode {
            case "primary":var carets=body["carets"] as! [String:Any];carets["primary"]=1;changed["carets"]=carets
            case "provisional":changed["snippet"]=["version":1]
            case "language":var rows=body["readings"] as! [[String:Any]];var facts=rows[1]["context"] as! [String:Any];facts["language"]="javascript";rows[1]["context"]=facts;changed["readings"]=rows
            default:
                var rows=body["expansions"] as! [[String:Any]]
                if mode=="missing"{rows.removeLast()}
                else if mode=="range"{rows[1]["replacement"]=["from":0,"to":3]}
                else{var snippet=rows[1]["snippet"] as! [String:Any];var fields=snippet["fields"] as! [[String:Any]];fields[0]["transform"]=["version":3];snippet["fields"]=fields;rows[1]["snippet"]=snippet}
                changed["expansions"]=rows
            }
            do {let candidate=try decode(changed,as:SourceTemplateBatchReply.self);if candidate.matches(catalog,entry:catalog.entries[0]){XCTAssertThrowsError(try candidate.prepared(source:"old two",continuing:nil),mode)}}catch{}
        }
    }
    func testTemplateProgramPolicyIsCapturedAndLegacyRejectsUnnegotiatedMetadata()throws {
        let (raw,body)=batchReply();XCTAssertEqual(raw["transform_program_versions"] as? [Int],[2,3])
        var legacyCatalog=raw,legacyBody=body;legacyCatalog["transform_program_versions"]=[2];legacyBody["transform_program_versions"]=[2]
        let catalog=try decode(legacyCatalog,as:SourceTemplateCatalog.self),fresh=try decode(body,as:SourceTemplateBatchReply.self);XCTAssertFalse(fresh.matches(catalog,entry:catalog.entries[0]))
        var rows=legacyBody["expansions"] as! [[String:Any]],snippet=rows[1]["snippet"] as! [String:Any],fields=snippet["fields"] as! [[String:Any]];fields[0]["transform"]=["version":3];snippet["fields"]=fields;rows[1]["snippet"]=snippet;legacyBody["expansions"]=rows
        let unnegotiated=try decode(legacyBody,as:SourceTemplateBatchReply.self)
        XCTAssertTrue(unnegotiated.matches(catalog,entry:catalog.entries[0]))
        XCTAssertThrowsError(try unnegotiated.prepared(source:"old two",continuing:nil)){error in XCTAssertTrue(error.localizedDescription.contains("not negotiated"),error.localizedDescription)}
        var invalid=body;invalid["transform_program_versions"]=[2,99];let foreign=try decode(invalid,as:SourceTemplateBatchReply.self);XCTAssertFalse(foreign.matches(catalog,entry:catalog.entries[0]))
    }

    func testCompleteBatchPreflightRejectsAggregateOutputAndChoiceMetadata()throws {
        let (_,body)=batchReply()
        for kind in ["output","choices"] {
            var changed=body;var rows=body["expansions"] as! [[String:Any]]
            for index in rows.indices {
                var snippet=rows[index]["snippet"] as! [String:Any]
                if kind=="output"{snippet["text"]=String(repeating:"x",count:600000)}
                else{var fields=snippet["fields"] as! [[String:Any]];fields[0]["choices"]=[String(repeating:"x",count:600000)];snippet["fields"]=fields}
                rows[index]["snippet"]=snippet
            }
            changed["expansions"]=rows;let plan=try decode(changed,as:SourceTemplateBatchReply.self);XCTAssertThrowsError(try plan.prepared(source:"old two",continuing:nil))
        }
    }

    func testEmbeddedBindingRetainsLanguageAndRejectsMalformedContext()throws {
        let (raw,body)=reply(),catalog=try decode(raw,as:SourceTemplateCatalog.self),plan=try decode(body,as:SourceTemplatePlan.self)
        let binding=try XCTUnwrap(catalog.source_context);XCTAssertTrue(binding.valid(path:"main.py",source:"value = 'old'\n",project:nil,language:"python"));XCTAssertFalse(binding.valid(path:"main.py",source:"changed",project:nil,language:"python"));XCTAssertFalse(binding.valid(path:"main.py",source:"value = 'old'\n",project:nil,language:"javascript"))
        var changed=plan;changed.source_context?.fingerprint=String(repeating:"c",count:64);XCTAssertFalse(sourceTemplatePlanMatches(changed,catalog:catalog,entry:catalog.entries[0]))
        var bad=raw,context=raw["source_context"] as! [String:Any];context["extra"]=true;bad["source_context"]=context;XCTAssertThrowsError(try decode(bad,as:SourceTemplateCatalog.self))
        bad=raw;bad["reading_context_request"]=["version":true,"kind":"construction_entry"];XCTAssertThrowsError(try decode(bad,as:SourceTemplateCatalog.self))
    }
    func testPlanBytesAndExactSelectionAreRequired()throws {
        let (raw,body)=reply(),catalog=try decode(raw,as:SourceTemplateCatalog.self),plan=try decode(body,as:SourceTemplatePlan.self),entry=try XCTUnwrap(catalog.entries.first)
        XCTAssertTrue(sourceTemplateCatalogMatches(catalog,path:"main.py",source:"value = 'old'\n",selection:NSRange(location:9,length:3)));XCTAssertTrue(sourceTemplatePlanMatches(plan,catalog:catalog,entry:entry))
        var changed=plan;changed.replacement.to=13;XCTAssertFalse(sourceTemplatePlanMatches(changed,catalog:catalog,entry:entry));changed=plan;changed.origin="builtin";XCTAssertFalse(sourceTemplatePlanMatches(changed,catalog:catalog,entry:entry))
        var composed=plan;composed.snippet.syntax="$0é";composed.snippet.text="é";composed.snippet.fields=[SourceTemplateField(id:0,parent:nil,index:0,from:0,to:0,choices:[])]
        var decomposed=composed;decomposed.snippet.text="e\u{301}";XCTAssertFalse(composed.same(decomposed));decomposed=composed;decomposed.snippet.syntax="$0e\u{301}";XCTAssertFalse(composed.same(decomposed))
        XCTAssertThrowsError(try validateSourceTestJSON(Data(#"{"template_protocol":1e0}"#.utf8)))
        changed=plan;changed.snippet.fields[0].to=99;XCTAssertThrowsError(try sourceTemplateDraft(changed,source:"value = 'old'\n",label:"Bad field"))
    }
    func testVariableContextRequiresExactStructureAndTypedNoEffects()throws {
        let (original,originalPlan)=reply();var raw=original,body=originalPlan
        for key in ["template_protocol","variable_context","source_files_modified"]{raw[key]=key=="template_protocol" ? 2 as Any:key=="variable_context" ? Self.variables.body as Any:false as Any;body[key]=raw[key]}
        let catalog=try decode(raw,as:SourceTemplateCatalog.self),plan=try decode(body,as:SourceTemplatePlan.self)
        XCTAssertTrue(sourceTemplateCatalogMatches(catalog,path:"main.py",source:"value = 'old'\n",selection:NSRange(location:9,length:3),variables:Self.variables));XCTAssertTrue(sourceTemplatePlanMatches(plan,catalog:catalog,entry:catalog.entries[0]))
        for effect in ["command","edit","workspaceEdit","additionalTextEdits"]{var changed=raw;changed[effect]=NSNull();XCTAssertThrowsError(try decode(changed,as:SourceTemplateCatalog.self));changed=body;changed[effect]=NSNull();XCTAssertThrowsError(try decode(changed,as:SourceTemplatePlan.self))}
        for value in [0 as Any,"false" as Any,true as Any]{var changed=raw;changed["source_files_modified"]=value;if let parsed=try? decode(changed,as:SourceTemplateCatalog.self){XCTAssertFalse(sourceTemplateCatalogMatches(parsed,path:"main.py",source:"value = 'old'\n",selection:NSRange(location:9,length:3),variables:Self.variables))}}
        var context=Self.variables.body;context["extra"]=true;raw["variable_context"]=context;XCTAssertThrowsError(try decode(raw,as:SourceTemplateCatalog.self))
        context=Self.variables.body;var clock=context["clock"] as! [String:Any];clock["extra"]=true;context["clock"]=clock;body["variable_context"]=context;XCTAssertThrowsError(try decode(body,as:SourceTemplatePlan.self))
    }
    #if os(macOS)
    func native()->(Workspace,DocumentBuffer,MacEditor,NSWindow) {
        _ = NSApplication.shared
        let configuration=URLSessionConfiguration.ephemeral;configuration.protocolClasses=[ResponseProtocol.self]
        let space=Workspace(),engine=try! Engine(address:"http://fixture.invalid",session:URLSession(configuration:configuration)),file=FileNode(name:"main.py",path:"main.py",dir:false),buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:"value = 'old'\n",hash:"base")),editor=MacEditor(buffer)
        space.engine=engine;space.buffers[file.path]=buffer;space.layout.activate(file.path,in:"document-group");space.sourceTemplates.captureVariables={Self.variables}
        let window=NSWindow(contentRect:NSRect(x:0,y:0,width:640,height:420),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=editor.scroll
        editor.text.setSelectedRange(NSRange(location:9,length:3));buffer.selection=editor.text.selectedRange()
        return(space,buffer,editor,window)
    }
    func testDemandedClipboardIsCapturedOnceDeniedExplicitlyAndCannotOutliveSourceLease()async throws {
        for mode in ["available","denied","stale","repeated"] {
            let (space,buffer,editor,window)=native(),(catalog,basePlan)=reply();try install(catalog,basePlan)
            defer{window.contentView=nil;ResponseProtocol.beforeResponse=nil;ResponseProtocol.responsesByPath=[:]}
            let model=space.sourceTemplates;var reads=0
            model.clipboardReader={reads+=1;if mode=="stale"{editor.text.setSelectedRange(NSRange(location:0,length:0));buffer.selection=editor.text.selectedRange()};return mode=="denied" ? nil:"copied"}
            var requested=basePlan;requested.removeValue(forKey:"snippet");requested["template_protocol"]=2;requested["source_files_modified"]=false;requested["variable_context"]=Self.variables.body;requested["available"]=false;requested["context_request"]=["version":1,"variables":["CLIPBOARD"]]
            var resolved=basePlan;resolved["template_protocol"]=2;resolved["source_files_modified"]=false
            var snapshot=Self.variables;snapshot.clipboard=SnippetClipboard(available:mode != "denied",text:mode=="denied" ? nil:"copied");resolved["variable_context"]=snapshot.body;resolved["context_unavailable"]=mode=="denied" ? ["CLIPBOARD"]:[]
            ResponseProtocol.beforeResponse={request in if request.url?.path=="/api/templates/plan"{let value=ResponseProtocol.requestCount("/api/templates/plan")==1||mode=="repeated" ? requested:resolved;ResponseProtocol.responsesByPath["/api/templates/plan"]=try! JSONSerialization.data(withJSONObject:value)}}
            await model.open(space);await model.choose(try XCTUnwrap(model.catalog?.entries.first));XCTAssertEqual(reads,1)
            if mode=="stale"||mode=="repeated"{XCTAssertNil(model.preview);XCTAssertEqual(buffer.text,"value = 'old'\n");continue}
            XCTAssertNotNil(model.preview,model.notice);let applied=await model.apply();XCTAssertTrue(applied,model.notice);XCTAssertEqual(reads,1);XCTAssertEqual(buffer.base,"value = 'old'\n")
            let requests=ResponseProtocol.requestBodies("/api/templates/plan");XCTAssertEqual(requests.count,3)
            let initial=requests[0]["variable_context"] as! [String:Any],retry=requests[1]["variable_context"] as! [String:Any]
            XCTAssertTrue(initial["clipboard"] is NSNull);XCTAssertEqual((retry["clipboard"] as? [String:Any])?["available"] as? Bool,mode != "denied")
            XCTAssertEqual(try JSONSerialization.data(withJSONObject:retry,options:.sortedKeys),try JSONSerialization.data(withJSONObject:requests[2]["variable_context"]!,options:.sortedKeys))
        }
    }
    func testReadonlyContributorRemainsReadableAndABAChangeRetiresWholePreview()async throws {
        let (space,buffer,editor,window)=native(),(catalog,plan)=reply();try install(catalog,plan);defer{window.contentView=nil;ResponseProtocol.responsesByPath=[:]}
        let file=FileNode(name:"context.md",path:"context.md",dir:false,generated_by:"fixture.md"),contributor=DocumentBuffer(file:file,text:FileText(path:file.path,content:"context",hash:"saved"));space.buffers[file.path]=contributor
        await space.sourceTemplates.open(space);await space.sourceTemplates.choose(try XCTUnwrap(space.sourceTemplates.catalog?.entries.first));XCTAssertNotNil(space.sourceTemplates.preview,space.sourceTemplates.notice)
        let request=try XCTUnwrap(ResponseProtocol.requestBodies("/api/templates/catalog").first),reading=try XCTUnwrap(request["reading_context"] as? [String:Any]);XCTAssertEqual((reading["sources"] as? [String:String])?[file.path],"context")
        contributor.text="changed";contributor.text="context";let before=ResponseProtocol.requestCount("/api/templates/plan");let applied=await space.sourceTemplates.apply();XCTAssertFalse(applied);XCTAssertEqual(ResponseProtocol.requestCount("/api/templates/plan"),before);XCTAssertEqual(buffer.text,"value = 'old'\n");XCTAssertTrue(contributor.readOnly);_ = editor
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
        for change in ["editUndo","selection","selectionUndo","deleted","composition","editor","model","closed"] {
            let (space,buffer,editor,window)=native(),(catalog,plan)=reply();try install(catalog,plan)
            await space.sourceTemplates.open(space);await space.sourceTemplates.choose(try XCTUnwrap(space.sourceTemplates.catalog?.entries.first));XCTAssertNotNil(space.sourceTemplates.preview)
            let source=buffer.text,original=try XCTUnwrap(buffer.sourceEditorState),state=try XCTUnwrap(original()),reviewed=space.sourceTemplates
            switch change {
            case "editUndo":buffer.text="changed";buffer.text=source
            case "selection":editor.text.setSelectedRange(NSRange(location:0,length:0));buffer.selection=editor.text.selectedRange()
            case "selectionUndo":let original=buffer.selection;editor.text.setSelectedRange(NSRange(location:0,length:0));buffer.selection=editor.text.selectedRange();editor.text.setSelectedRange(original);buffer.selection=editor.text.selectedRange()
            case "deleted":buffer.resourceDeleted=true
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
        let file=try XCTUnwrap(find(tree.tree,path));space.engine=engine;space.rootFiles=tree.tree;space.sourceRootAddress=tree.root_path;space.sourceWorkspaceID=tree.workspace_id;space.debugger=NativeDebugger(engine:engine)
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
        model.captureVariables={Self.variables};var clipboardReads=0;model.clipboardReader={clipboardReads+=1;return "copied"}
        await model.open(space);await model.choose(try XCTUnwrap(model.catalog?.entries.first(where:{$0.id=="workspace.context"})));XCTAssertEqual(model.preview?.snippet.text,"main.py|2024|012345|old|old|OLD",model.notice);XCTAssertEqual(clipboardReads,0)
        let contextApplied=await model.apply();XCTAssertTrue(contextApplied,model.notice);XCTAssertEqual(buffer.text,"value = 'main.py|2024|012345|old|old|OLD'\n")
        let installed=try XCTUnwrap(buffer.snippets.active,"Derived template metadata must install an active native session");XCTAssertEqual(installed.fields.filter{$0.transform != nil}.count,1);XCTAssertNotNil(buffer.snippets.commit)
        let selectedField=try XCTUnwrap(installed.fields.first(where:{$0.id==installed.current}));XCTAssertNil(selectedField.transform);XCTAssertEqual(native.selectedRange(),NSRange(location:selectedField.from,length:selectedField.to-selectedField.from))
        native.insertText("new",replacementRange:native.selectedRange());XCTAssertEqual(buffer.text,"value = 'main.py|2024|012345|new|new|NEW'\n")
        manager.undo();manager.undo();XCTAssertEqual(buffer.text,source);manager.redo();manager.redo();XCTAssertEqual(buffer.text,"value = 'main.py|2024|012345|new|new|NEW'\n");manager.undo();manager.undo()
        for denied in [false,true]{
            native.setSelectedRange(NSRange(location:9,length:3));buffer.selection=native.selectedRange();let before=clipboardReads
            model.clipboardReader={clipboardReads+=1;return denied ? nil:"copied"}
            await model.open(space);XCTAssertEqual(clipboardReads,before);await model.choose(try XCTUnwrap(model.catalog?.entries.first(where:{$0.id=="workspace.clipboard"})))
            XCTAssertEqual(clipboardReads,before+1);XCTAssertEqual(model.preview?.snippet.text,denied ? "fallback":"copied",model.notice);XCTAssertEqual(model.preview?.context_unavailable ?? [],denied ? ["CLIPBOARD"]:[])
            let applied=await model.apply();XCTAssertTrue(applied,model.notice);XCTAssertEqual(clipboardReads,before+1);XCTAssertEqual(buffer.text,"value = '\(denied ? "fallback":"copied")'\n");manager.undo();XCTAssertEqual(buffer.text,source);manager.redo();XCTAssertEqual(buffer.text,"value = '\(denied ? "fallback":"copied")'\n");manager.undo()
        }
        native.setSelectedRange(NSRange(location:9,length:3));buffer.selection=native.selectedRange();let beforeUnused=clipboardReads
        await model.open(space);await model.choose(try XCTUnwrap(model.catalog?.entries.first(where:{$0.id=="workspace.unused-clipboard"})));XCTAssertEqual(model.preview?.snippet.text,"old",model.notice);XCTAssertEqual(clipboardReads,beforeUnused);model.close()
        model.clipboardReader={clipboardReads+=1;native.setSelectedRange(NSRange(location:0,length:0));buffer.selection=native.selectedRange();return "stale"}
        await model.open(space);await model.choose(try XCTUnwrap(model.catalog?.entries.first(where:{$0.id=="workspace.clipboard"})));XCTAssertNil(model.preview);XCTAssertEqual(buffer.text,source)
        native.setSelectedRange(NSRange(location:9,length:3));buffer.selection=native.selectedRange();XCTAssertTrue(editor(host) === native);XCTAssertEqual(buffer.sourceEditorState?()?.identity,identity);XCTAssertEqual(buffer.base,source)
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
