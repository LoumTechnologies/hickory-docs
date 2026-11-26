import XCTest
#if os(macOS)
import AppKit
import Vision
import SwiftUI
import Darwin
#endif
@testable import HickorySimple
final class AgentProposalTests:XCTestCase {
  func testConversationActivityLabelsShowTheTargetWithoutEditPayload() throws {
   let data = #"{"arguments":{"doc":"project.md","upstream":"chapters/code.md","input":"proposed replacement"}}"#
   let bytes = try JSONSerialization.data(withJSONObject: [
       "run_id": "turn", "exec_id": "agent",
       "event": ["kind": "tool_started", "name": "edit_doc", "data": data]
   ])
   let frame = try JSONDecoder().decode(NativeAgentEvent.self, from: bytes)
   let tool = NativeAgentActivity.Tool(name: try XCTUnwrap(frame.event.name), data: frame.event.data)
   XCTAssertEqual(tool.name, "edit_doc")
   XCTAssertEqual(tool.label, "Propose source edit · chapters/code.md")
   XCTAssertEqual(NativeAgentActivity.Tool(name: "read_doc", data: nil).label, "Read source")
   XCTAssertEqual(NativeAgentActivity.Tool(name: "custom", data: data).label, "custom")
  }
  func testConversationPreviewDoesNotMatchByteDistinctRevision() {
   let old = NativePendingAgentReview(id: "preview", name: "edit_doc", path: "project.md", oldText: "old", newText: "\u{e9}", editor: false, status: "pending")
   let revised = NativePendingAgentReview(id: "preview", name: "edit_doc", path: "project.md", oldText: "old", newText: "e\u{301}", editor: false, status: "pending")
   XCTAssertEqual(old.newText, revised.newText)
   XCTAssertTrue(old.matches(old))
   XCTAssertFalse(old.matches(revised))
  }
 #if os(macOS)
 @MainActor func testLiveHistoricalProposalKeepsDirtyEditorAndHasNoApply()async throws {
  let env=ProcessInfo.processInfo.environment;guard let origin=env["HICKORY_PROPOSAL_HISTORY_ORIGIN"],let root=env["HICKORY_PROPOSAL_HISTORY_WORKSPACE"],let token=env["HICKORY_PROPOSAL_HISTORY_TOKEN"],let fingerprint=env["HICKORY_PROPOSAL_HISTORY_FINGERPRINT"] else{throw XCTSkip("Run the owned proposal-history host fixture.")};let directory=URL(fileURLWithPath:root);XCTAssertEqual(try String(contentsOf:directory.appendingPathComponent(".hickory-proposal-archive-fixture"),encoding:.utf8),"owned-proposal-archive-v1\n")
  let engine=try Engine(address:origin),w=Workspace();w.engine=engine;let tree=try await engine.files();guard let physical=root.withCString({realpath($0,nil)}) else{throw EngineError(message:"Owned historical fixture root is unavailable")};defer{free(physical)};XCTAssertTrue(exactUTF8(tree.root_path,String(cString:physical)),"Engine and owned fixture must share the POSIX physical root");w.sourceRootAddress=tree.root_path;w.sourceWorkspaceID=tree.workspace_id;w.files=tree.tree;w.rootFiles=tree.tree;let b=try await w.loadBuffer(FileNode(name:"note.py",path:"note.py",dir:false));w.layout.activateSource("note.py",in:"document-group");let original=b.text
  _ = NSApplication.shared;NSApp.setActivationPolicy(.regular);let host=NSHostingView(rootView:WorkspaceView(workspace:w)),window=NSWindow(contentRect:NSRect(x:0,y:0,width:840,height:560),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=host;window.makeKeyAndOrderFront(nil);defer{w.buffers.values.forEach{$0.language?.close()};window.orderOut(nil)}
  func find(_ view:NSView)->NSTextView?{if let text=view as? NSTextView,exactUTF8(text.string,b.text){return text};return view.subviews.lazy.compactMap{find($0)}.first}
  for _ in 0..<400{host.layoutSubtreeIfNeeded();if find(host) != nil{break};try await Task.sleep(for:.milliseconds(10))};let editor=try XCTUnwrap(find(host));window.makeFirstResponder(editor);editor.setSelectedRange(NSRange(location:editor.string.utf16.count,length:0));editor.insertText("# retained archive draft\n",replacementRange:editor.selectedRange());let draft=b.text;XCTAssertNotEqual(draft,original);let selection=editor.selectedRange(),geometry=editor.frame,manager=try XCTUnwrap(editor.undoManager),m=w.agentProposals
  await m.open(w);let entry=try XCTUnwrap(m.entries.first{$0.token==token},m.notice);XCTAssertEqual(entry.fingerprint,fingerprint);await m.read(w,entry);XCTAssertTrue(m.value?.historical==true,m.notice);XCTAssertTrue(m.value?.sections.contains{$0.title.hasPrefix("Explanation")}==true);XCTAssertFalse(m.canApply);XCTAssertFalse(m.authorityCurrent);XCTAssertFalse(m.hasReport);await m.applyDraft(w);await m.prepareSave(w);XCTAssertEqual(b.text,draft);XCTAssertTrue(find(host) === editor);XCTAssertEqual(editor.selectedRange(),selection);XCTAssertEqual(editor.frame,geometry);m.close();window.makeFirstResponder(editor);manager.undo();XCTAssertEqual(b.text,original);XCTAssertFalse(m.canApply);XCTAssertEqual(try String(contentsOf:directory.appendingPathComponent("note.py"),encoding:.utf8),original)
 }
 @MainActor func testLiveOwnedManagedProposalRetainsEditorDraftUndoAndNamespaceRecovery()async throws {
  guard let origin=ProcessInfo.processInfo.environment["HICKORY_NATIVE_PROPOSAL_ORIGIN"],let root=ProcessInfo.processInfo.environment["HICKORY_NATIVE_PROPOSAL_WORKSPACE"] else{throw XCTSkip("Opt in with owned scripts/test-agent-proposals-browser.py --serve-only fixture.")}
  let directory=URL(fileURLWithPath:root),marker=try String(contentsOf:directory.appendingPathComponent(".hickory-native-agent-proposal-fixture"),encoding:.utf8);guard marker=="owned-native-agent-proposal-v1\n" else{throw EngineError(message:"An isolated owned proposal fixture is required.")}
  let engine=try Engine(address:origin),w=Workspace();w.engine=engine;let tree=try await engine.files();guard let resolved=root.withCString({realpath($0,nil)}) else{throw EngineError(message:"Owned proposal fixture root is unavailable.")};defer{free(resolved)};guard exactUTF8(tree.root_path,String(cString:resolved)) else{throw EngineError(message:"Server does not own this isolated proposal fixture namespace.")};w.sourceRootAddress=tree.root_path;w.sourceWorkspaceID=tree.workspace_id;w.files=tree.tree;w.rootFiles=tree.tree
  let b=try await w.loadBuffer(FileNode(name:"note.py",path:"note.py",dir:false));w.layout.activateSource("note.py",in:"document-group");let source=b.text,generated=try Data(contentsOf:directory.appendingPathComponent("generated.py"));XCTAssertEqual(source,"value=1\n")
  _ = NSApplication.shared;NSApp.setActivationPolicy(.regular);let host=NSHostingView(rootView:WorkspaceView(workspace:w)),window=NSWindow(contentRect:NSRect(x:0,y:0,width:840,height:560),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=host;window.makeKeyAndOrderFront(nil)
  defer{for buffer in w.buffers.values{buffer.language?.close()};window.orderOut(nil)}
  func settle(_ message:String,_ ready:()->Bool)async throws {for _ in 0..<400{if ready(){return};try await Task.sleep(for:.milliseconds(10))};throw EngineError(message:message)}
  func find(_ view:NSView)->NSTextView?{if let text=view as? NSTextView,exactUTF8(text.string,b.text){return text};return view.subviews.lazy.compactMap{find($0)}.first}
  try await settle("Native source editor did not mount"){host.layoutSubtreeIfNeeded();return find(host) != nil};let editor=try XCTUnwrap(find(host));editor.setSelectedRange(NSRange(location:2,length:0));b.selection=NSRange(location:2,length:0);let geometry=editor.frame,selection=editor.selectedRange()
  func prepare(_ edit:[String:Any])async throws->AgentProposalEntry {
   let intention:[String:Any]=["version":1,"request_id":UUID().uuidString.replacingOccurrences(of:"-",with:"").lowercased(),"origin":"owned controlled ACP fixture","title":"Explain and change implementation together","explanation":"The complete implementation and connected consequences are reviewed together.","claims":["Agent claim only; not an executed verdict"],"check_ids":[],"capture":["context":UUID().uuidString.replacingOccurrences(of:"-",with:"").lowercased(),"project_entry":"project.md","drafts":[:],"ownership_sources":[:],"unaddressed_sources":[:],"saved_guards":[:],"edit":edit],"reading":["version":1,"entry":"project.md","sources":[:],"missing_sources":[],"focus":b.file.path,"unaddressed_sources":[:],"saved_guards":[:]]]
   let parent=directory.deletingLastPathComponent(),calls=parent.appendingPathComponent("calls.json"),result=parent.appendingPathComponent("tool-result.json");try JSONSerialization.data(withJSONObject:["intention":intention]).write(to:calls,options:.atomic);if FileManager.default.fileExists(atPath:result.path){try FileManager.default.removeItem(at:result)}
   let _:ResourceJSON=try await engine.request("POST","/api/docs/workspace/agent/acp",body:["backend":"owned-proposal"])
   let _:ResourceJSON=try await engine.request("POST","/api/docs/workspace/agent",body:["backend":"owned-proposal","prompt":"Prepare the owned intention only; do not apply, save or run it.","context":try nativeAgentContext(w)])
   try await settle("Controlled ACP tool did not retain a complete proposal"){FileManager.default.fileExists(atPath:result.path)}
   let reply=try JSONSerialization.jsonObject(with:Data(contentsOf:result)) as? [String:Any],answer=reply?["result"] as? [String:Any],content=answer?["content"] as? [[String:Any]],text=try XCTUnwrap(content?.first?["text"] as? String),summary=try JSONSerialization.jsonObject(with:Data(text.utf8)) as? [String:Any];XCTAssertEqual(answer?["isError"] as? Bool,false);return AgentProposalEntry(token:try XCTUnwrap(summary?["token"] as? String),fingerprint:try XCTUnwrap(summary?["fingerprint"] as? String))
  }
  let edit:[String:Any]=["changes":[languageURI("note.py"):[["range":["start":["line":0,"character":6],"end":["line":0,"character":7]],"newText":"41"]]]]
  let entry=try await prepare(edit),m=w.agentProposals;try XCTUnwrap(w.commands().first{$0.id=="agent.proposals"}).run();try await settle("Proposal menu did not open"){!m.busy&&m.entries.contains(entry)};await m.read(w,entry);XCTAssertTrue(m.canApply,m.notice);XCTAssertTrue(m.value!.material.origin_authority.hasPrefix("engine-managed"));XCTAssertEqual(b.text,source);await m.applyDraft(w);XCTAssertEqual(b.text,"value=41\n",m.notice);XCTAssertEqual(try String(contentsOf:directory.appendingPathComponent("note.py"),encoding:.utf8),source);XCTAssertTrue(find(host) === editor);XCTAssertEqual(editor.selectedRange(),selection);XCTAssertEqual(editor.frame,geometry);XCTAssertFalse(m.canApply);XCTAssertFalse(m.hasReport,m.notice)
  w.changes.open("undo",workspace:w);XCTAssertEqual(b.text,source);w.changes.open("redo",workspace:w);XCTAssertEqual(b.text,"value=41\n");w.changes.open("undo",workspace:w);XCTAssertEqual(b.text,source)
  m.close();let moved="moved λ%2F?#.py",move:[String:Any]=["documentChanges":[["kind":"rename","oldUri":languageURI("note.py"),"newUri":languageURI(moved)]]],resource=try await prepare(move);await m.open(w);await m.read(w,resource);XCTAssertTrue(m.canApply,m.notice);await m.prepareSave(w);XCTAssertTrue(m.canSave,m.notice);await m.save(w);XCTAssertEqual(w.resourceRestoration.confirmedReceipt?.adoption_current,true,w.resourceRestoration.notice);XCTAssertTrue(w.buffers[moved] === b);XCTAssertTrue(find(host) === editor);XCTAssertFalse(FileManager.default.fileExists(atPath:directory.appendingPathComponent("note.py").path));XCTAssertEqual(try String(contentsOf:directory.appendingPathComponent(moved),encoding:.utf8),source);XCTAssertEqual(try Data(contentsOf:directory.appendingPathComponent("generated.py")),generated)
  await w.resourceRestoration.prepareInverse(w);XCTAssertNotNil(w.resourceRestoration.review,w.resourceRestoration.notice);await w.resourceRestoration.publish(w);XCTAssertTrue(w.buffers["note.py"] === b);XCTAssertTrue(find(host) === editor);XCTAssertEqual(b.text,source)
 }
 #endif

 @MainActor private func fixture(_ b:DocumentBuffer?=nil,managed:Bool=false,recordedAuthority:String?=nil)throws->(AgentProposalEntry,ResourceJSON) {
  let workspace=String(repeating:"b",count:32),before="value=1\n",after="value=41\n",guardToken="original-loaded",context="captured"
  func source(_ text:String)->[String:Any]{["kind":"file","identity":"source-note","text":text,"writable":true,"removable":true,"version":NSNull()]}
  func saved(_ text:String)->[String:Any]{["kind":"file","text":text,"sha256":nativeSourceHash(text),"readonly":false,"executable":false,"mode":420]}
  let layer:[String:Any]=["entries":[],"removed_entries":[],"source_hashes":[:],"output_paths":[]],guards=["note.py":guardToken]
  let edit:[String:Any]=["changes":[languageURI("note.py"):[["range":["start":["line":0,"character":0],"end":["line":1,"character":0]],"newText":after]]]]
  let capture:[String:Any]=["context":context,"edit":edit,"project_entry":NSNull(),"drafts":["note.py":["text":before,"base_sha256":nativeSourceHash(before),"save_guard":guardToken,"writable":true]],"ownership_sources":["note.py":before],"unaddressed_sources":[:],"saved_guards":guards]
  let plan:[String:Any]=["version":1,"context":context,"title":"Complete change","annotations":[],"patches":[["path":"note.py","before":source(before),"after":source(after)]]]
  let inputs:[String:Any]=["version":1,"context":context,"workspace":workspace,"sources":["note.py":source(before)],"saved_sources":["note.py":saved(before)],"saved_guards":guards,"source_files_modified":false,"refusals":[:],"ownership":["mode":"none","saved_sources":[:],"draft_sources":[:],"protected_entries":[]]]
  let facts:[String:Any]=["version":1,"context":context,"workspace":workspace,"fingerprint":String(repeating:"f",count:64),"saved":layer,"draft":layer,"protected_targets":[],"source_files_modified":false,"saved_sources":["note.py":saved(after)]]
  let reading:[String:Any]=["version":1,"behavior_protocol":1,"namespace":"/fixture","engine_workspace_id":workspace]
  var intention:[String:Any]=["version":1,"request_id":String(repeating:"c",count:32),"title":"Complete change","explanation":"Implementation and explanation remain one intention.","claims":["Unverified agent claim"],"check_ids":[],"capture":capture]
  if let b{intention["editor_snapshot"]=["live":false,"buffers":[["id":b.id.uuidString,"path":"note.py","source_hash":nativeSourceHash(b.text),"save_guard":guardToken,"source_revision":b.agentSourceRevision.map(String.init) as Any? ?? NSNull()]]]}
  let authority = recordedAuthority ?? (managed ? "engine-managed agent tool; non-live submitted editor snapshot; claims remain unverified" : "caller-provided intention; not an agent attestation")
  let material:[String:Any]=["version":1,"workspace":workspace,"namespace":"/fixture","origin_authority":authority,"intention":intention,"inputs":inputs,"plan":plan,"consequences":facts,"before":reading,"after":reading,"saved_guards":guards]
  let text=String(decoding:try JSONSerialization.data(withJSONObject:material,options:[.sortedKeys]),as:UTF8.self),entry=AgentProposalEntry(token:String(repeating:"a",count:32),fingerprint:nativeSourceHash(text))
  let raw:[String:Any]=["version":1,"proposal_protocol":1,"token":entry.token,"fingerprint":entry.fingerprint,"workspace":workspace,"namespace":"/fixture","material":text,"source_effects":false,"checks_executed":false,"host_application":NSNull()]
  return(entry,try JSONDecoder().decode(ResourceJSON.self,from:JSONSerialization.data(withJSONObject:raw)))
 }
 @MainActor private func host()throws->(Workspace,DocumentBuffer){let w=Workspace();w.engine=try Engine(address:"http://fixture.invalid");w.sourceWorkspaceID=String(repeating:"b",count:32);w.sourceRootAddress="/fixture";let b=DocumentBuffer(file:FileNode(name:"note.py",path:"note.py",dir:false),text:FileText(path:"note.py",content:"value=1\n",hash:"baseline",save_guard:"original-loaded"));w.buffers["note.py"]=b;return(w,b)}
 @MainActor func testRecordedPatchAdmissionUsesBothMessageSidesAndRetiresAppliedAuthorityOnEdit() async throws {
  let (w, target) = try host(), model = w.agentProposals, agent = w.agent, turn = "owned-review"
  let patch = "Recorded text\n```diff\n--- a/note.py\n+++ b/note.py\n@@ -1 +1 @@\n-value=1\n+value=41\n```\n"
  let answer = "Assistant patch\n" + patch
  agent.turns = [AgentTurn(id: turn, prompt: patch, answer: answer, status: "ok")]
  let selectedEntry: [String: Any] = ["version": 1, "entry_protocol": 1, "workspace": String(repeating: "b", count: 32),
   "namespace": "/fixture", "entry": ["version": 1, "identity": String(repeating: "c", count: 32), "generation": 1, "path": NSNull()],
   "guard": String(repeating: "d", count: 64), "movable": false, "source_effects": false]
  let entry = try JSONDecoder().decode(ResourceJSON.self, from: JSONSerialization.data(withJSONObject: selectedEntry))
  for (side, text, opening) in [(RecordedMessageSide.user, patch, 1), (.assistant, answer, 2)] {
   let message = agent.document("\(turn)-\(side.bufferSuffix)", text)
   let hash = nativeSourceHash(text), authority = "recorded \(side.rawValue) fence, source SHA-256 \(hash); claims remain unverified"
   let (seal, envelope) = try fixture(target, recordedAuthority: authority)
   var writes = 0, reviewed = 0
   model.transport = { method, path, body in
    if method == "GET", path == "/api/workspace/entry" { return entry }
    if method == "POST", path == "/api/docs/workspace/agent/turns/owned-review/fence-patch" {
     let fence = try XCTUnwrap(body?["fence"] as? [String: Any])
     XCTAssertEqual(fence["side"] as? String, side.rawValue)
     XCTAssertEqual(fence["source_hash"] as? String, hash)
     XCTAssertEqual(fence["opening_line"] as? Int, opening)
     let intention = try XCTUnwrap(body?["intention"] as? [String: Any])
     let capture = try XCTUnwrap(intention["capture"] as? [String: Any])
     XCTAssertEqual((capture["saved_guards"] as? [String: String])?["note.py"], "original-loaded")
     XCTAssertTrue(((capture["edit"] as? [String: Any])?["changes"] as? [String: Any])?.isEmpty == true)
     reviewed += 1
     return envelope
    }
    if method == "GET", path == "/api/agent-proposals/" + seal.token { return envelope }
    writes += 1
    throw EngineError(message: "A stale review cannot dispatch a write")
   }
   message.inlineMarkup = InlineSnapshot(source: text, debug: inlineDebugContext(text, path: message.file.path))
   XCTAssertEqual(message.inlineMarkup.debug.recordedPatches, [opening])
   let source = NativeRecordedMessageSource(turn: turn, side: side, workspace: "/fixture") { agent.owns(message, turn: turn, side: side) }
   await model.prepareRecordedPatch(w, message: message, source: source, openingLine: opening)
   XCTAssertEqual(reviewed, 1)
   XCTAssertEqual(model.value?.material.origin_authority, authority, model.notice)
   XCTAssertTrue(model.canApply, model.notice)
   XCTAssertEqual(target.text, "value=1\n")
   XCTAssertTrue(message.readOnly)
   XCTAssertTrue(exactUTF8(message.text, text))
   target.text = "intervening draft"; target.text = "value=1\n"
   XCTAssertFalse(model.canApply)
   await model.applyDraft(w)
   XCTAssertEqual(writes, 0)
   XCTAssertEqual(target.text, "value=1\n")
   model.close()
  }
 }
 @MainActor func testHistoricalProposalReadsOriginalSealWithoutAcquiringCurrentEditors()async throws {
  let(w,b)=try host(),(entry,raw)=try fixture();guard case .object(let original)=raw else{return XCTFail("Expected fixture envelope")};var fields=original;let runtime=String(repeating:"c",count:32);w.sourceWorkspaceID=runtime;fields["historical"] = .bool(true);fields["reading_scope"] = .object(["workspace":.string(runtime),"namespace":.string("/fixture")]);let historical=ResourceJSON.object(fields),m=w.agentProposals;var effects=0
  m.transport={method,path,_ in if path=="/api/agent-proposals"{return .object(["version":.integer(1),"proposal_protocol":.integer(1),"workspace":.string(runtime),"namespace":.string("/fixture"),"reviews":.array([.object(["token":.string(entry.token),"fingerprint":.string(entry.fingerprint),"historical":.bool(true)])])])};if method=="GET",path=="/api/agent-proposals/"+entry.token{return historical};effects+=1;throw EngineError(message:"Historical reading entered live preparation")}
  await m.open(w);await m.read(w,entry);XCTAssertTrue(m.value?.historical==true,m.notice);XCTAssertEqual(m.value?.material.workspace,String(repeating:"b",count:32));XCTAssertFalse(m.canApply);XCTAssertFalse(m.authorityCurrent);await m.applyDraft(w);XCTAssertEqual(effects,0);XCTAssertEqual(b.text,"value=1\n");XCTAssertTrue(m.phase.contains("Historical"))
  fields["reading_scope"] = .object(["workspace":.string(String(repeating:"d",count:32)),"namespace":.string("/fixture")]);let foreign=try JSONDecoder().decode(AgentProposalEnvelope.self,from:JSONEncoder().encode(ResourceJSON.object(fields)));XCTAssertThrowsError(try SealedAgentProposal(foreign,expected:entry,workspace:runtime,namespace:"/fixture"))
 }
 @MainActor func testLiveProposalReportsFailedDurabilityWithoutLosingItsSeal()throws {
  let(entry,raw)=try fixture();guard case .object(let original)=raw else{return XCTFail("Expected envelope")};var fields=original;fields["durability"] = .object(["retained":.bool(false),"notice":.string("Private archive unavailable")]);let envelope=try JSONDecoder().decode(AgentProposalEnvelope.self,from:JSONEncoder().encode(ResourceJSON.object(fields))),value=try SealedAgentProposal(envelope,expected:entry,workspace:String(repeating:"b",count:32),namespace:"/fixture");XCTAssertFalse(value.historical);XCTAssertEqual(value.retentionNotice,"Private archive unavailable")
 }
 @MainActor func testCompleteManagedReadRetiresAfterEqualTextUndoAndExtraEditor()async throws {
  let(w,b)=try host(),(entry,envelope)=try fixture(b,managed:true),m=w.agentProposals
  m.transport={method,path,body in if path=="/api/agent-proposals" {return .object(["version":.integer(1),"proposal_protocol":.integer(1),"workspace":.string(String(repeating:"b",count:32)),"namespace":.string("/fixture"),"reviews":.array([.object(["token":.string(entry.token),"fingerprint":.string(entry.fingerprint)])])])};return envelope}
  await m.open(w);await m.read(w,entry);XCTAssertTrue(m.canApply,m.notice);b.text="other";b.text="value=1\n";XCTAssertFalse(m.canApply);await m.read(w,entry);XCTAssertFalse(m.canApply);XCTAssertTrue(m.notice.contains("revision"),m.notice)
 }
 @MainActor func testSealedLeaseRetiresNewUnaddressedConstructionWithoutChangingCode()async throws {
  let(w,b)=try host(),(entry,envelope)=try fixture(),m=w.agentProposals
  m.transport={_,path,_ in path=="/api/agent-proposals" ? .object(["version":.integer(1),"proposal_protocol":.integer(1),"workspace":.string(String(repeating:"b",count:32)),"namespace":.string("/fixture"),"reviews":.array([.object(["token":.string(entry.token),"fingerprint":.string(entry.fingerprint)])])]):envelope}
  await m.open(w);await m.read(w,entry);XCTAssertTrue(m.canApply,m.notice)
  let added=DocumentBuffer(file:FileNode(name:"Untitled",path:"draft",dir:false),text:FileText(path:"draft",content:"<h:include path='other.md'/>",hash:""),isUntitledDraft:true);w.buffers["draft"]=added;XCTAssertFalse(m.canApply);await m.applyDraft(w);XCTAssertEqual(b.text,"value=1\n");XCTAssertEqual(added.text,"<h:include path='other.md'/>")
 }

 func testDigestFailurePrecedesInnerJSONAndExplanationAdmission()throws {
  let material="{not admitted explanation";let row=AgentProposalEntry(token:String(repeating:"a",count:32),fingerprint:String(repeating:"b",count:64))
  let envelope=AgentProposalEnvelope(version:1,proposal_protocol:1,token:row.token,fingerprint:row.fingerprint,workspace:"workspace",namespace:"namespace",material:material,source_effects:false,checks_executed:false,host_application:nil)
  XCTAssertThrowsError(try SealedAgentProposal(envelope,expected:row,workspace:"workspace",namespace:"namespace")){XCTAssertTrue($0.localizedDescription.contains("SHA256"))}
 }
 func testForeignNamespaceCannotAdmitEvenCorrectDigest()throws {
  let material="{}",row=AgentProposalEntry(token:String(repeating:"a",count:32),fingerprint:nativeSourceHash(material))
  let envelope=AgentProposalEnvelope(version:1,proposal_protocol:1,token:row.token,fingerprint:row.fingerprint,workspace:"workspace",namespace:"é",material:material,source_effects:false,checks_executed:false,host_application:nil)
  XCTAssertThrowsError(try SealedAgentProposal(envelope,expected:row,workspace:"workspace",namespace:"e\u{301}"))
 }
 func testOuterWireRefusesDeepUnknownStructureAndProtocolAliases()throws {let nested=String(repeating:"[",count:65)+"0"+String(repeating:"]",count:65);XCTAssertThrowsError(try validateNativeAgentProposalResponse(Data(nested.utf8)));for token in ["1.0","true","1e0"]{XCTAssertThrowsError(try validateNativeAgentProposalResponse(Data(("{\"version\":"+token+"}").utf8)))};XCTAssertNoThrow(try validateNativeAgentProposalResponse(Data("{\"version\":1,\"proposal_protocol\":1,\"material\":\"literal { [\"}".utf8)))}
 func testPagesNeverSplitUnicodeScalar()throws {let text=String(repeating:"🙂",count:4001),section=try AgentProposalSection("Literal",text);XCTAssertEqual(section.pages.count,2);XCTAssertEqual(section.page(0).unicodeScalars.count,4000);XCTAssertEqual(section.page(1),"🙂")}
 @MainActor func testSenderRevisionRetiresEqualTextUndoAndCloseContext()throws {
  let b=DocumentBuffer(file:FileNode(name:"main.py",path:"main.py",dir:false),text:FileText(path:"main.py",content:"value=1\n",hash:""));let original=b.agentSourceRevision;b.text="value=2\n";b.text="value=1\n";XCTAssertNotEqual(b.agentSourceRevision,original);let equal=b.agentSourceRevision;b.retireAgentContext();XCTAssertNotEqual(equal,b.agentSourceRevision)
 }
}
#if os(macOS)
extension AgentProposalTests {
 @MainActor func testMountedNativeAgentSendThroughCoordinatedDraftHistory() async throws {
  let environment = ProcessInfo.processInfo.environment
  guard let origin = environment["HICKORY_APPKIT_PROPOSAL_SEND_ORIGIN"],
        let root = environment["HICKORY_APPKIT_PROPOSAL_SEND_ROOT"] else {
   throw XCTSkip("Use the owned native Agent Send fixture.")
  }
  let directory = URL(fileURLWithPath: root).standardizedFileURL.resolvingSymlinksInPath()
  let marker = directory.appendingPathComponent(".hickory-native-agent-proposal-fixture")
  guard try String(contentsOf: marker, encoding: .utf8) == "owned-native-agent-proposal-v1\n" else {
   throw EngineError(message: "An isolated proposal workspace is required.")
  }
  _ = NSApplication.shared
  NSApp.setActivationPolicy(.regular)
  let workspace = Workspace()
  workspace.address = origin
  guard await workspace.connect(), let confirmed = workspace.sourceRootAddress,
        URL(fileURLWithPath: confirmed).standardizedFileURL.resolvingSymlinksInPath() == directory else {
   throw EngineError(message: "The server is not connected to the owned proposal workspace. \(workspace.status) · \(workspace.sourceRootAddress ?? "no root") · \(directory.path)")
  }
  let noteFile = try XCTUnwrap(workspace.workspaceFile("note.py")), constructionFile = try XCTUnwrap(workspace.workspaceFile("construction.md"))
  let note = try await workspace.loadBuffer(noteFile), construction = try await workspace.loadBuffer(constructionFile)
  let notePane = try XCTUnwrap(workspace.layout.split("document-group", axis: "horizontal", newID: "fixture-neighbor"))
  let constructionPane = try XCTUnwrap(workspace.layout.split("fixture-neighbor", axis: "vertical", newID: "fixture-agent"))
  workspace.layout.activateSource(note.file.path, in: notePane)
  workspace.layout.activateSource(construction.file.path, in: constructionPane)
  workspace.layout.activateUtility(.agent, in: "fixture-agent")
  workspace.focused = notePane
  let host = NSHostingView(rootView: WorkspaceView(workspace: workspace))
  let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 1240, height: 940),
   styleMask: [.titled], backing: .buffered, defer: false)
  window.contentView = host
  window.makeKeyAndOrderFront(nil)
  NSApp.activate()
  defer { workspace.debugger?.close(); workspace.editorRecovery.close(); window.orderOut(nil); window.contentView = nil }

  func wait(_ message: @autoclosure () -> String, ready: () -> Bool) async throws {
   let clock = ContinuousClock(), deadline = clock.now + .seconds(20)
   while clock.now < deadline {
    host.layoutSubtreeIfNeeded()
    if ready() { return }
    try await Task.sleep(for: .milliseconds(100))
   }
   let message = message()
   XCTFail(message)
   throw EngineError(message: message)
  }
  func editor(_ source: String, under view: NSView) -> NSTextView? {
   if let text = view as? NSTextView, text.isEditable, exactUTF8(text.string, source) { return text }
   return view.subviews.lazy.compactMap { editor(source, under: $0) }.first
  }
  func clickSend(around view: NSView) throws -> Bool {
   view.displayIfNeeded()
   guard let bitmap = view.bitmapImageRepForCachingDisplay(in: view.bounds) else { return false }
   view.cacheDisplay(in: view.bounds, to: bitmap)
   guard let raster = bitmap.cgImage,
         let composite = CGContext(data: nil, width: raster.width, height: raster.height,
          bitsPerComponent: 8, bytesPerRow: 0, space: CGColorSpaceCreateDeviceRGB(),
          bitmapInfo: CGImageAlphaInfo.noneSkipLast.rawValue) else { return false }
   var background = NSColor.white
   view.effectiveAppearance.performAsCurrentDrawingAppearance {
    background = NSColor.windowBackgroundColor.usingColorSpace(.deviceRGB) ?? .white
   }
   let area = CGRect(x: 0, y: 0, width: CGFloat(raster.width), height: CGFloat(raster.height))
   composite.setFillColor(background.cgColor)
   composite.fill(area)
   composite.draw(raster, in: area)
   guard let pixels = composite.makeImage() else { return false }
   if let directory = environment["HICKORY_APPKIT_PROPOSAL_SEND_IMAGES"],
      let png = NSBitmapImageRep(cgImage: pixels).representation(using: .png, properties: [:]) {
    let output = URL(fileURLWithPath: directory).appendingPathComponent("native-composer.png")
    try? png.write(to: output, options: .atomic)
   }
   let reading = VNRecognizeTextRequest()
   reading.recognitionLevel = .accurate
   reading.usesLanguageCorrection = false
   try VNImageRequestHandler(cgImage: pixels, options: [:]).perform([reading])
   let sendBoxes = (reading.results ?? []).compactMap { row -> CGRect? in
    guard let line = row.topCandidates(1).first,
          line.string == "Send" || line.string.hasPrefix("Send Stop"),
          let word = line.string.range(of: "Send") else { return nil }
    return (try? line.boundingBox(for: word))?.boundingBox
   }
   guard sendBoxes.count == 1, let box = sendBoxes.first else { return false }
   let local = NSPoint(x: view.bounds.minX + box.midX * view.bounds.width,
                       y: view.bounds.minY + (view.isFlipped ? 1 - box.midY : box.midY) * view.bounds.height)
   guard window.isVisible, view.bounds.contains(local) else { return false }
   if !window.isKeyWindow { window.makeKey() }
   let point = view.convert(local, to: nil)
   let hit = view.hitTest(view.convert(local, to: view.superview))
   NSLog("HickoryAppKitNativeSend button (%g, %g), key=%d, flipped=%d, hit=%@", point.x, point.y,
         window.isKeyWindow ? 1 : 0, view.isFlipped ? 1 : 0, hit.map { String(describing: type(of: $0)) } ?? "none")
   func mouse(_ type: NSEvent.EventType) throws -> NSEvent {
    try XCTUnwrap(NSEvent.mouseEvent(with: type, location: point, modifierFlags: [],
     timestamp: ProcessInfo.processInfo.systemUptime, windowNumber: window.windowNumber,
     context: nil, eventNumber: 0, clickCount: type == .mouseMoved ? 0 : 1,
     pressure: type == .leftMouseDown ? 1 : 0))
   }
   NSApp.sendEvent(try mouse(.mouseMoved))
   NSApp.postEvent(try mouse(.leftMouseUp), atStart: false)
   NSApp.sendEvent(try mouse(.leftMouseDown))
   if let up = NSApp.nextEvent(matching: .leftMouseUp, until: .distantPast, inMode: .default, dequeue: true) {
    NSApp.sendEvent(up)
   }
   return true
  }
  func command(_ id: String) throws { try XCTUnwrap(workspace.commands().first { $0.id == id }).run() }

  let before = [note.text, construction.text]
  let after = ["value=41\n", construction.text.replacingOccurrences(of: "generated = 1", with: "generated = 41")]
  let fileNames = ["note.py", "construction.md", "generated.py"]
  let saved = try fileNames.map { try Data(contentsOf: directory.appendingPathComponent($0)) }
  XCTAssertEqual(before[0], "value=1\n"); XCTAssertNotEqual(before[1], after[1])
  try await wait("Both source editors and the native composer must mount before Send") {
   before.allSatisfy { editor($0, under: host) != nil } && editor("", under: host) != nil
  }
  let noteEditor = try XCTUnwrap(editor(before[0], under: host)), constructionEditor = try XCTUnwrap(editor(before[1], under: host))
  let composer = try XCTUnwrap(editor("", under: host))
  let originalEditorIDs = try [note, construction].map { try XCTUnwrap($0.sourceEditorState?()).identity }
  let originalGuards = try [note, construction].map { try XCTUnwrap($0.savedGuard) }
  let editors = [noteEditor, constructionEditor]
  for text in editors { text.setSelectedRange(NSRange(location: 2, length: 3)) }
  let selections = editors.map { $0.selectedRange() }, frames = editors.map { $0.convert($0.bounds, to: host) }
  let prompt = "Prepare the owned two-source intention.", agent = workspace.agent
  XCTAssertTrue(window.makeFirstResponder(composer))
  composer.insertText(prompt, replacementRange: composer.selectedRange())
  try await wait("Native composer did not retain the exact entered prompt") { exactUTF8(agent.prompt, prompt) }
  try await wait("The visible Agent Send button could not be found") {
   (try? clickSend(around: host)) == true
  }
  try await wait("Native Send did not archive the exact prompt. Agent: \(agent.error)") {
   agent.turns.contains { exactUTF8($0.prompt, prompt) && $0.status == "ok" }
  }
  XCTAssertTrue(agent.prompt.isEmpty)
  let recorded = try XCTUnwrap(agent.turns.first { exactUTF8($0.prompt, prompt) })

  let review = workspace.agentProposals
  try command("agent.proposals")
  try await wait("The mounted proposal review did not list the sealed native Send result. Review: \(review.notice)") {
   review.showing && !review.busy && review.entries.count == 1
  }
  let entry = try XCTUnwrap(review.entries.first)
  await review.read(workspace, entry)
  let proposal = try XCTUnwrap(review.value, review.notice)
  XCTAssertTrue(proposal.material.origin_authority.hasPrefix("engine-managed"))
  XCTAssertEqual(nativeSourceHash(proposal.envelope.material), entry.fingerprint)
  XCTAssertEqual(Set(proposal.patches.map(\.path)), Set(fileNames.prefix(2)))
  XCTAssertTrue(proposal.sections.contains { $0.title.hasPrefix("Saved and draft generated consequences") && $0.text.contains("generated.py") })
  XCTAssertEqual([note.text, construction.text], before)
  XCTAssertTrue(review.canApply, review.notice)
  XCTAssertFalse(proposal.envelope.source_effects); XCTAssertFalse(proposal.envelope.checks_executed)
  await review.applyDraft(workspace)
  try await wait("The guarded proposal did not apply exactly both drafts. Review: \(review.notice)") {
   [note.text, construction.text] == after && zip(editors, after).allSatisfy { exactUTF8($0.0.string, $0.1) }
  }
  XCTAssertFalse(review.hasReport, review.notice)
  review.close()
  try command("workspace.undo")
  try await wait("One coordinated Undo did not restore both mounted editors") {
   [note.text, construction.text] == before && zip(editors, before).allSatisfy { exactUTF8($0.0.string, $0.1) }
  }
  XCTAssertEqual(editors.map { $0.selectedRange() }, selections)
  try command("workspace.redo")
  try await wait("One coordinated Redo did not restore both reviewed drafts") {
   [note.text, construction.text] == after && zip(editors, after).allSatisfy { exactUTF8($0.0.string, $0.1) }
  }
  XCTAssertTrue(editor(after[0], under: host) === noteEditor); XCTAssertTrue(editor(after[1], under: host) === constructionEditor)
  XCTAssertEqual(try [note, construction].map { try XCTUnwrap($0.sourceEditorState?()).identity }, originalEditorIDs)
  XCTAssertEqual(try [note, construction].map { try XCTUnwrap($0.savedGuard) }, originalGuards)
  XCTAssertEqual(editors.map { $0.convert($0.bounds, to: host) }, frames)
  XCTAssertEqual(try fileNames.map { try Data(contentsOf: directory.appendingPathComponent($0)) }, saved)
  XCTAssertEqual(agent.turns.first { $0.id == recorded.id }?.prompt, prompt)
 }
}
#endif
