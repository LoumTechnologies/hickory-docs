import XCTest
#if os(macOS)
import AppKit
#endif
@testable import HickorySimple
@MainActor final class CompletionAcceptanceTests:XCTestCase {
    func settle(_ condition:@escaping ()->Bool)async{let end=Date().addingTimeInterval(2);while !condition()&&Date()<end{try? await Task.sleep(for:.milliseconds(1))}}
    func edit(_ line:Int,_ from:Int,_ to:Int,_ text:String)->[String:Any]{["range":["start":["line":line,"character":from],"end":["line":line,"character":to]],"newText":text]}
    func testBackgroundCompletionKeepsRetainedRowsAndIgnoresObsoleteErrors() async throws {
        let file = FileNode(name: "a.py", path: "a.py", dir: false)
        let buffer = DocumentBuffer(file: file, text: FileText(path: file.path, content: "va", hash: "base"))
        let language = NativeLanguage(engine: try Engine(address: "http://fixture.invalid"), buffer: buffer)
        buffer.language = language
        buffer.selection = NSRange(location: 2, length: 0)
        defer { language.close() }

        await language.complete(fetch: { _ in [["label": "value"]] })
        XCTAssertEqual(language.suggestions.map(\.label), ["value"])
        await language.complete(fetch: { _ in NSNull() })
        XCTAssertEqual(language.suggestions.map(\.label), ["value"])
        XCTAssertEqual(language.status, "Showing last received completions. Request completions again.")

        var held: CheckedContinuation<Any, Error>?
        let obsolete = Task {
            await language.complete(fetch: { _ in
                try await withCheckedThrowingContinuation { held = $0 }
            })
        }
        await settle { held != nil }
        let reply = try XCTUnwrap(held)
        buffer.resourceDeleted = true
        language.status = "Source is now read-only."
        reply.resume(throwing: EngineError(message: "obsolete completion transport"))
        await obsolete.value
        XCTAssertEqual(language.status, "Source is now read-only.")

        buffer.resourceDeleted = false
        await language.complete(fetch: { _ in throw EngineError(message: "fixture provider failure") })
        XCTAssertEqual(language.status, "Completion unavailable: fixture provider failure")
        XCTAssertEqual(buffer.text, "va")
        XCTAssertEqual(buffer.selection, NSRange(location: 2, length: 0))
    }

    func testActualMacEditorAcceptsImportsAndCaretInOneUndo()async throws {
        #if os(macOS)
        let source="header\nva\n",file=FileNode(name:"a.py",path:"a.py",dir:false)
        let buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:source,hash:"base")),service=NativeLanguage(engine:try Engine(address:"http://fixture.invalid"),buffer:buffer);buffer.language=service
        let editor=MacEditor(buffer),window=NSWindow(contentRect:NSRect(x:0,y:0,width:500,height:300),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=editor.scroll
        defer{service.close();window.contentView=nil}
        editor.text.setSelectedRange(NSRange(location:9,length:0));buffer.selection=editor.text.selectedRange();let original:[String:Any]=["label":"value","data":["hickory_completion":"held"]]
        let item=NativeCompletion(id:1,label:"value",detail:"",range:NSRange(location:7,length:2),insert:"value",source:source,selection:buffer.selection,original:original)
        let accepted=await service.accept(item,resolution:{_ in ["label":"value","textEdit":self.edit(1,0,2,"value"),"additionalTextEdits":[self.edit(0,0,6,"longheader")]]})
        XCTAssertTrue(accepted);XCTAssertEqual(buffer.text,"longheader\nvalue\n");XCTAssertEqual(editor.text.selectedRange().location,16);XCTAssertEqual(buffer.base,source);XCTAssertEqual(buffer.hash,"base")
        buffer.saving=true;editor.text.undoManager?.undo();XCTAssertEqual(buffer.text,source);buffer.saving=false
        editor.text.undoManager?.redo();XCTAssertEqual(buffer.text,"longheader\nvalue\n");XCTAssertEqual(editor.text.selectedRange().location,16)
        buffer.text="external";editor.text.string="external";editor.text.undoManager?.undo()
        XCTAssertEqual(buffer.text,"external");XCTAssertEqual(editor.text.string,"external");XCTAssertTrue(service.completionNotice.contains("history retired"));XCTAssertFalse(editor.text.undoManager?.canUndo ?? false)
        #endif
    }
    private func caretOffer(_ source:String,token:String)->[String:Any] {
        ["label":"value","kind":6,"detail":"./dep",
         "data":["hickory_completion":token,"hickory_context":[
            "source_hash":"sha256:"+nativeSourceHash(source),"source_revision":String(repeating:"a",count:64)
         ]]]
    }

    private func resolvedCaretProof(_ offer:[String:Any]) throws -> [String:Any] {
        var resolved=offer
        var data=try XCTUnwrap(offer["data"] as? [String:Any])
        let context=try XCTUnwrap(data["hickory_context"] as? [String:Any])
        data["hickory_context"]=try XCTUnwrap(context["expected"] as? [String:String])
        data.removeValue(forKey:"hickory_context_request")
        resolved["data"]=data
        return resolved
    }

    func testActualMacMultiCaretCompletionSharesImportAndRestoresEveryCaret() async throws {
        #if os(macOS)
        let source="😀\nva\nva\n",file=FileNode(name:"a.py",path:"a.py",dir:false)
        let buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:source,hash:"base"))
        let service=NativeLanguage(engine:try Engine(address:"http://fixture.invalid"),buffer:buffer)
        buffer.language=service
        let editor=MacEditor(buffer),window=NSWindow(contentRect:NSRect(x:0,y:0,width:500,height:300),styleMask:[.titled],backing:.buffered,defer:false)
        window.contentView=editor.scroll
        defer{service.close();window.contentView=nil}
        let before=[5,8].map{NSRange(location:$0,length:0)}
        XCTAssertTrue(buffer.selectMultiple(before,source:source))
        await service.complete(fetch:{_ in [self.caretOffer(source,token:"primary")]})
        let choice=try XCTUnwrap(service.suggestions.first)
        var queries:[Int]=[],tokens:[String]=[]
        let accepted=await service.accept(choice,fetch:{params in
            let position=try XCTUnwrap(params["position"] as? [String:Int])
            queries.append(try XCTUnwrap(position["line"]))
            XCTAssertEqual(position["character"],2)
            return [self.caretOffer(source,token:"peer")]
        },resolution:{offer in
            let data=try XCTUnwrap(offer["data"] as? [String:Any])
            tokens.append(try XCTUnwrap(data["hickory_completion"] as? String))
            var resolved=try self.resolvedCaretProof(offer)
            resolved["additionalTextEdits"]=[self.edit(0,0,0,"import x\n")]
            return resolved
        })
        let after="import x\n😀\nvalue\nvalue\n"
        let carets=[17,23].map{NSRange(location:$0,length:0)}
        XCTAssertTrue(accepted,service.completionNotice)
        XCTAssertEqual(queries,[2]);XCTAssertEqual(tokens,["primary","peer"])
        XCTAssertEqual(buffer.text,after);XCTAssertEqual(editor.text.string,after)
        XCTAssertEqual(buffer.multiRanges,carets);XCTAssertEqual(editor.text.selectedRange(),carets[0])
        XCTAssertEqual(buffer.base,source);XCTAssertEqual(buffer.hash,"base")
        editor.text.undoManager?.undo()
        XCTAssertEqual(buffer.text,source);XCTAssertEqual(buffer.multiRanges,before)
        XCTAssertEqual(editor.text.selectedRange(),before[0])
        editor.text.undoManager?.redo()
        XCTAssertEqual(buffer.text,after);XCTAssertEqual(buffer.multiRanges,carets)
        XCTAssertEqual(editor.text.selectedRange(),carets[0])
        #endif
    }

    func testActualMacGroupedSnippetKeepsIndependentFieldsAndOneClipboardOutcome() async throws {
        #if os(macOS)
        let source="header\nva\nvo\n",file=FileNode(name:"a.py",path:"a.py",dir:false)
        let buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:source,hash:"base"))
        let service=NativeLanguage(engine:try Engine(address:"http://fixture.invalid"),buffer:buffer)
        buffer.language=service
        let editor=MacEditor(buffer),window=NSWindow(contentRect:NSRect(x:0,y:0,width:500,height:300),styleMask:[.titled],backing:.buffered,defer:false)
        window.contentView=editor.scroll
        defer{service.close();window.contentView=nil}
        let before=[NSRange(location:7,length:2),NSRange(location:10,length:2)]
        XCTAssertTrue(buffer.selectMultiple(before,source:source))
        let syntax="${1:${TM_SELECTED_TEXT}}+$1!$0${CLIPBOARD:}"
        var reads=0,requests:[String]=[],randoms:[String]=[],clocks:[[String:Int64]]=[]
        service.snippetClipboardReader={reads+=1;return nil}
        await service.complete(fetch:{_ in [self.caretOffer(source,token:"primary")]})
        let choice=try XCTUnwrap(service.suggestions.first)
        let accepted=await service.accept(choice,fetch:{params in
            XCTAssertEqual(params["position"] as? [String:Int],["line":2,"character":0])
            return [self.caretOffer(source,token:"peer")]
        },resolution:{offer in
            let data=try XCTUnwrap(offer["data"] as? [String:Any])
            let token=try XCTUnwrap(data["hickory_completion"] as? String)
            let context=try XCTUnwrap(data["hickory_context"] as? [String:Any])
            let row=token.hasPrefix("primary") ? 0 : 1
            let original=before[row]
            XCTAssertEqual(context["selection"] as? [String:Int],["from":original.location,"to":NSMaxRange(original)])
            XCTAssertEqual(context["carets"] as? [String:Int],["index":row,"count":2])
            randoms.append(try XCTUnwrap(context["random_hex"] as? String))
            clocks.append(try XCTUnwrap(context["clock"] as? [String:Int64]))
            requests.append(token)
            var result=try self.resolvedCaretProof(offer)
            var reply=try XCTUnwrap(result["data"] as? [String:Any])
            result["insertTextFormat"]=2
            result["textEdit"]=self.edit(row+1,0,2,syntax)
            result["additionalTextEdits"]=[self.edit(0,0,0,"import x\n")]
            if !token.hasSuffix("-retry") {
                XCTAssertEqual(reads,0)
                XCTAssertTrue(context["clipboard"] is NSNull)
                reply["hickory_completion"]=token+"-retry"
                reply["hickory_context_request"]=["version":1,"variables":["CLIPBOARD"]]
            }else{
                let clipboard=try XCTUnwrap(context["clipboard"] as? [String:Any])
                XCTAssertEqual(clipboard["available"] as? Bool,false)
                XCTAssertTrue(clipboard["text"] is NSNull)
                let value=row==0 ? "😀" : "β",width=value.utf16.count,exit=2*width+2
                let fields:[[String:Any]]=[
                    ["index":1,"from":0,"to":width,"choices":[],"id":0,"parent":NSNull()],
                    ["index":1,"from":width+1,"to":2*width+1,"choices":[],"id":1,"parent":NSNull()],
                    ["index":0,"from":exit,"to":exit,"choices":[],"id":2,"parent":NSNull()],
                ]
                reply["hickory_context_unavailable"]=["CLIPBOARD"]
                reply["hickory_snippet"]=["version":1,"syntax":syntax,"text":"\(value)+\(value)!","fields":fields]
            }
            result["data"]=reply
            return result
        })
        let expanded="import x\nheader\n😀+😀!\nβ+β!\n"
        let initialFields=[NSRange(location:16,length:2),NSRange(location:23,length:1)]
        guard accepted else{XCTFail(service.completionNotice);return}
        XCTAssertTrue(service.completionNotice.contains("Clipboard unavailable"))
        XCTAssertEqual(reads,1)
        XCTAssertEqual(requests,["primary","peer","primary-retry","peer-retry"])
        XCTAssertEqual(Set(randoms).count,1);XCTAssertTrue(clocks.allSatisfy{$0==clocks.first})
        XCTAssertEqual(buffer.text,expanded);XCTAssertEqual(buffer.multiRanges,initialFields)
        XCTAssertEqual(buffer.snippets.parallel?.selections,initialFields)
        editor.text.insertText("Z",replacementRange:editor.text.selectedRange())
        let typed="import x\nheader\nZ+Z!\nZ+Z!\n"
        XCTAssertEqual(buffer.text,typed)
        XCTAssertTrue(buffer.snippets.move(1))
        XCTAssertEqual(buffer.multiRanges,[20,25].map{NSRange(location:$0,length:0)})
        let manager=try XCTUnwrap(editor.text.undoManager)
        manager.undo();XCTAssertEqual(buffer.text,expanded);XCTAssertEqual(buffer.multiRanges,initialFields)
        manager.undo();XCTAssertEqual(buffer.text,source);XCTAssertEqual(buffer.multiRanges,before);XCTAssertNil(buffer.snippets.parallel)
        manager.redo();XCTAssertEqual(buffer.text,expanded);XCTAssertEqual(buffer.multiRanges,initialFields);XCTAssertNotNil(buffer.snippets.parallel)
        manager.redo();XCTAssertEqual(buffer.text,typed);XCTAssertNotNil(buffer.snippets.parallel)
        XCTAssertEqual(buffer.base,source);XCTAssertEqual(buffer.hash,"base")
        #endif
    }

    func testActualMacSourceEqualGroupedSnippetStartsFieldsWithoutTextUndo() async throws {
        #if os(macOS)
        let source="va\nva\n",file=FileNode(name:"a.py",path:"a.py",dir:false)
        let buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:source,hash:"base"))
        let service=NativeLanguage(engine:try Engine(address:"http://fixture.invalid"),buffer:buffer)
        buffer.language=service
        let editor=MacEditor(buffer),window=NSWindow(contentRect:NSRect(x:0,y:0,width:500,height:300),styleMask:[.titled],backing:.buffered,defer:false)
        window.contentView=editor.scroll
        defer{service.close();window.contentView=nil}
        let before=[2,5].map{NSRange(location:$0,length:0)},fields=[0,3].map{NSRange(location:$0,length:2)}
        XCTAssertTrue(buffer.selectMultiple(before,source:source))
        let manager=try XCTUnwrap(editor.text.undoManager)
        manager.removeAllActions()
        await service.complete(fetch:{_ in [self.caretOffer(source,token:"primary")]})
        let choice=try XCTUnwrap(service.suggestions.first)
        let accepted=await service.accept(choice,fetch:{_ in [self.caretOffer(source,token:"peer")]},resolution:{offer in
            var reply=try self.resolvedCaretProof(offer)
            var data=try XCTUnwrap(reply["data"] as? [String:Any])
            let row=data["hickory_completion"] as? String=="primary" ? 0 : 1
            let syntax="${1:va}$0"
            let fields:[[String:Any]]=[
                ["index":1,"from":0,"to":2,"choices":[],"id":0,"parent":NSNull()],
                ["index":0,"from":2,"to":2,"choices":[],"id":1,"parent":NSNull()],
            ]
            data["hickory_snippet"]=["version":1,"syntax":syntax,"text":"va","fields":fields]
            reply["insertTextFormat"]=2;reply["textEdit"]=self.edit(row,0,2,syntax);reply["data"]=data
            return reply
        })
        guard accepted else{XCTFail(service.completionNotice);return}
        XCTAssertEqual(buffer.text,source);XCTAssertFalse(buffer.dirty);XCTAssertFalse(manager.canUndo)
        XCTAssertEqual(buffer.multiRanges,fields);XCTAssertEqual(buffer.snippets.parallel?.selections,fields)
        editor.text.insertText("edited",replacementRange:editor.text.selectedRange())
        XCTAssertEqual(buffer.text,"edited\nedited\n");XCTAssertTrue(manager.canUndo)
        manager.undo()
        XCTAssertEqual(buffer.text,source);XCTAssertEqual(buffer.multiRanges,fields)
        XCTAssertEqual(buffer.snippets.parallel?.selections,fields);XCTAssertFalse(manager.canUndo)
        #endif
    }

    func testInvalidGroupedPeerPreventsClipboardAndCommit() async throws {
        let source="//\nva\nva\n",file=FileNode(name:"a.py",path:"a.py",dir:false)
        let buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:source,hash:"base"))
        let service=NativeLanguage(engine:try Engine(address:"http://fixture.invalid"),buffer:buffer)
        buffer.language=service
        defer{service.close()}
        let before=[5,8].map{NSRange(location:$0,length:0)},editorID=UUID()
        buffer.selection=before[0];buffer.multiSource=source;buffer.multiRanges=before
        buffer.sourceEditorState={ [weak buffer] in buffer.map {
            NativeSourceEditorState(identity:editorID,source:$0.text,selection:$0.selection,composing:false,editable:true,attached:true,selections:$0.multiRanges.count)
        }}
        var reads=0,commits=0
        service.snippetClipboardReader={reads+=1;return "private clipboard"}
        buffer.applyCompletionDraft={_,_ in commits+=1;return true}
        for mixed in [true,false] {
            await service.complete(fetch:{_ in [self.caretOffer(source,token:"primary")]})
            let choice=try XCTUnwrap(service.suggestions.first)
            var requests:[String]=[]
            let accepted=await service.accept(choice,fetch:{_ in [self.caretOffer(source,token:"peer")]},resolution:{offer in
                var reply=try self.resolvedCaretProof(offer)
                var data=try XCTUnwrap(reply["data"] as? [String:Any])
                let token=try XCTUnwrap(data["hickory_completion"] as? String)
                requests.append(token)
                if token=="peer",mixed{return reply}
                data["hickory_completion"]=token+"-retry"
                data["hickory_context_request"]=["version":1,"variables":["CLIPBOARD"]]
                reply["data"]=data
                reply["insertTextFormat"]=2
                reply["textEdit"]=self.edit(token=="primary" ? 1 : 2,0,2,"$1$0${CLIPBOARD:}")
                reply["additionalTextEdits"]=[self.edit(0,0,0,token=="primary" ? "import x\n" : "import y\n")]
                return reply
            })
            XCTAssertFalse(accepted)
            XCTAssertEqual(requests,["primary","peer"])
            XCTAssertEqual(reads,0);XCTAssertEqual(commits,0)
            XCTAssertEqual(buffer.text,source);XCTAssertEqual(buffer.multiRanges,before)
        }
    }

    func testSecondaryCaretRoundTripAndAmbiguousPeerRefuseTheWholeCompletion() async throws {
        let source="va\nva\n",file=FileNode(name:"a.py",path:"a.py",dir:false)
        let buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:source,hash:"base"))
        let service=NativeLanguage(engine:try Engine(address:"http://fixture.invalid"),buffer:buffer)
        buffer.language=service
        defer{service.close()}
        let origin=[2,5].map{NSRange(location:$0,length:0)},editorID=UUID()
        buffer.selection=origin[0];buffer.multiSource=source;buffer.multiRanges=origin
        buffer.sourceEditorState={ [weak buffer] in buffer.map {
            NativeSourceEditorState(identity:editorID,source:$0.text,selection:$0.selection,composing:false,editable:true,attached:true,selections:$0.multiRanges.count)
        }}
        var commits=0
        buffer.applyCompletionDraft={_,_ in commits+=1;return true}
        for ambiguous in [true,false] {
            await service.complete(fetch:{_ in [self.caretOffer(source,token:"primary")]})
            let choice=try XCTUnwrap(service.suggestions.first)
            let accepted=await service.accept(choice,fetch:{_ in
                var offers=[self.caretOffer(source,token:"peer")]
                if ambiguous {offers.append(self.caretOffer(source,token:"competitor"))}
                return offers
            },resolution:{offer in
                if (offer["data"] as? [String:Any])?["hickory_completion"] as? String=="peer" {
                    buffer.multiRanges=[origin[0],NSRange(location:4,length:0)]
                    buffer.multiRanges=origin
                }
                return try self.resolvedCaretProof(offer)
            })
            XCTAssertFalse(accepted)
            XCTAssertEqual(buffer.text,source);XCTAssertEqual(buffer.multiRanges,origin)
            XCTAssertEqual(commits,0)
        }
    }

    func testActualMacSnippetSelectionAndImportsUndoTogether()async throws {
        #if os(macOS)
        let apps=URL(fileURLWithPath:#filePath).deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
        let vectors=try JSONSerialization.jsonObject(with:Data(contentsOf:apps.appendingPathComponent("simple-contract/snippet-expansion-cases.json"))) as! [String:Any]
        let resolved=(vectors["cases"] as! [[String:Any]])[0]["item"] as! [String:Any]
        let source="va",file=FileNode(name:"a.py",path:"a.py",dir:false),buffer=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:"va",hash:"base"))
        let service=NativeLanguage(engine:try Engine(address:"http://fixture.invalid"),buffer:buffer);buffer.language=service
        let editor=MacEditor(buffer),window=NSWindow(contentRect:NSRect(x:0,y:0,width:500,height:300),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=editor.scroll
        defer{service.close();window.contentView=nil}
        editor.text.setSelectedRange(NSRange(location:2,length:0));buffer.selection=editor.text.selectedRange()
        let item=NativeCompletion(id:1,label:file.name,detail:"",range:NSRange(location:0,length:2),insert:"",source:source,selection:buffer.selection,original:resolved)
        let accepted=await service.accept(item,resolution:{_ in resolved});XCTAssertTrue(accepted)
        XCTAssertEqual(buffer.text,"#\n😀 + 😀");XCTAssertEqual(editor.text.selectedRange(),NSRange(location:2,length:2));XCTAssertEqual(buffer.base,source)
        editor.text.undoManager?.undo();XCTAssertEqual(buffer.text,source);editor.text.undoManager?.redo();XCTAssertEqual(editor.text.selectedRange(),NSRange(location:2,length:2))
        #endif
    }
    func testNewerChoiceAndDismissalRetireHeldResolutionWithoutOverwritingSource()async throws {
        let source="va\n",buffer=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:"va\n",hash:"base"))
        let service=NativeLanguage(engine:try Engine(address:"http://fixture.invalid"),buffer:buffer);buffer.language=service;buffer.selection=NSRange(location:2,length:0);defer{service.close()}
        var commits=0
        buffer.applyCompletionDraft={draft,initial in
            let edits=(draft.plan.patches.first?.edits ?? []).map{NativeSourceEdit(range:NSRange(location:$0.from,length:$0.to-$0.from),text:$0.text)},before=draft.before,selected=NSRange(location:draft.selection.from,length:draft.selection.to-draft.selection.from)
            guard exactUTF8(buffer.text,before),buffer.selection==initial else{return false};commits+=1
            let text=NSMutableString(string:before);for edit in edits.sorted(by:{$0.range.location>$1.range.location}){text.replaceCharacters(in:edit.range,with:edit.text)};buffer.text=text as String;buffer.selection=selected;return true
        }
        var held:[String:CheckedContinuation<[String:Any],Error>]=[:]
        let resolve:([String:Any]) async throws->[String:Any]={item in try await withCheckedThrowingContinuation{held[item["label"] as! String]=$0}}
        func item(_ label:String)->NativeCompletion{NativeCompletion(id:1,label:label,detail:"",range:NSRange(location:0,length:2),insert:label,source:source,selection:NSRange(location:2,length:0),original:["label":label,"data":["hickory_completion":label]])}
        func result(_ label:String)->[String:Any]{["label":label,"textEdit":edit(0,0,2,label),"additionalTextEdits":[edit(0,0,0,"# import\n")]]}
        let first=Task{await service.accept(item("old"),resolution:resolve)};await settle{held["old"] != nil}
        let second=Task{await service.accept(item("new"),resolution:resolve)};await settle{held["new"] != nil}
        try XCTUnwrap(held["new"]).resume(returning:result("new"));let secondAccepted=await second.value;XCTAssertTrue(secondAccepted);let notice=service.completionNotice
        try XCTUnwrap(held["old"]).resume(returning:result("old"));let firstAccepted=await first.value;XCTAssertFalse(firstAccepted);XCTAssertEqual(commits,1);XCTAssertEqual(buffer.text,"# import\nnew\n");XCTAssertEqual(service.completionNotice,notice)
        buffer.text=source;buffer.selection=NSRange(location:2,length:0)
        let pending=Task{await service.accept(item("dismissed"),resolution:resolve)};await settle{held["dismissed"] != nil};service.cancelCompletion()
        try XCTUnwrap(held["dismissed"]).resume(returning:result("dismissed"));let pendingAccepted=await pending.value;XCTAssertFalse(pendingAccepted);XCTAssertEqual(buffer.text,source);XCTAssertEqual(commits,1)
    }
}
