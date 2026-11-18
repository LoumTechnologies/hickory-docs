import XCTest
@testable import HickorySimple
final class TestSourcesTests:XCTestCase {
    func fixture()->(TestSourceSnapshot,SourceTestScan) {
        let before=TestSourceSnapshot(workspace:"w1",request:TestSourceRequest(path:"cases.md",source:"draft",projectEntry:"project.md",sources:["helper.md":"helper","unused.md":"old"]))
        let point=TestSourcePoint(path:"test.py",line:1,character:4,hash:"target")
        let candidate=SourceTestCandidate(id:"case",language:"python",run_scope:"file",name:"test_case",name_filter:"test_case",confidence:"syntax",source:nil,target:point)
        return (before,SourceTestScan(version:1,input_path:"cases.md",fingerprint:"scan",source_hashes:["cases.md":"draft-hash","helper.md":"helper-hash"],candidates:[candidate],scope:"project",syntax_complete:true,notices:[],unsupported_languages:[],unclassified_outputs:[],unused_sources:["unused.md"]))
    }
    func testContributorAndWorkspaceIdentity() {
        let (before,facts)=fixture();XCTAssertTrue(testSourceCurrent(before,facts,before));var now=before
        now.workspace="w2";XCTAssertFalse(testSourceCurrent(before,facts,now));now=before
        now.request.source="changed";XCTAssertFalse(testSourceCurrent(before,facts,now));now=before
        now.request.sources["helper.md"]="changed";XCTAssertFalse(testSourceCurrent(before,facts,now));now=before
        now.request.sources["unused.md"]="new";XCTAssertTrue(testSourceCurrent(before,facts,now))
        var future=facts;future.version=2;XCTAssertFalse(testSourceCurrent(before,future,before))
    }
    func debugFixture()throws->(TestSourceSnapshot,SourceTestScan,SourceTestDebugPlan,[String:Any]) {
        var (before,facts)=fixture();before.request.sources=[before.request.path:before.request.source]
        facts.source_hashes=[before.request.path:nativeSourceHash(before.request.source)];facts.workspace_id="saved-workspace";facts.input_hash=nativeSourceHash(before.request.source)
        let plan=SourceTestDebugPlan(version:1,test_debug_protocol:1,available:true,reason:nil,workspace_id:"saved-workspace",input_path:before.request.path,input_hash:facts.input_hash!,analysis_fingerprint:facts.fingerprint,candidate_id:"case",candidate_name:"test_case",selection:SourceTestDebugSelection(kind:"name_filter"),source_uri:languageURI(before.request.path),source_hash:facts.input_hash!,language:"python",framework:"pytest",name:"pytest filter",notices:[])
        let binding:[String:Any]=["version":1,"workspace_id":plan.workspace_id,"input_path":plan.input_path,"input_hash":plan.input_hash,"analysis_fingerprint":plan.analysis_fingerprint,"candidate_id":plan.candidate_id,"candidate_name":plan.candidate_name,"selection":["kind":"name_filter"],"checked_sources":facts.source_hashes,"language":"python","framework":"pytest"]
        return(before,facts,plan,binding)
    }
    func testDebugPlanAndEntireAdmissionBinding()throws {
        let (before,facts,plan,raw)=try debugFixture();XCTAssertTrue(plan.matches(before,facts,facts.candidates[0],.nameFilter))
        let binding=try JSONDecoder().decode(SourceTestDebugBinding.self,from:JSONSerialization.data(withJSONObject:raw));XCTAssertTrue(plan.admits(binding,facts))
        var stale=plan;stale.candidate_name="other";XCTAssertFalse(stale.matches(before,facts,facts.candidates[0],.nameFilter))
        stale=plan;stale.selection.kind="containing_scope";XCTAssertFalse(stale.matches(before,facts,facts.candidates[0],.nameFilter))
        stale=plan;stale.source_uri="hick:///runtime.py";XCTAssertFalse(stale.matches(before,facts,facts.candidates[0],.nameFilter))
        stale=plan;stale.test_debug_protocol=2;XCTAssertFalse(stale.matches(before,facts,facts.candidates[0],.nameFilter))
        var changed=binding;changed.checked_sources["new-helper"]="hash";XCTAssertFalse(plan.admits(changed,facts))
        changed=binding;changed.input_hash="other";XCTAssertFalse(plan.admits(changed,facts))
        var bool=raw;bool["version"]=true;XCTAssertThrowsError(try JSONDecoder().decode(SourceTestDebugBinding.self,from:JSONSerialization.data(withJSONObject:bool)))
    }
    func testRawContributorAliasesRefuseBeforeFoundationMergesThem()throws {
        let alias=Data(#"{"checked_sources":{"é.py":"same","e\u0301.py":"same"}}"#.utf8)
        XCTAssertThrowsError(try validateSourceTestJSON(alias))
        for raw in [#"{"version":1.0}"#,#"{"test_debug_protocol":1e0}"#,#"{"source_test":{"version":1E+0}}"#]{XCTAssertThrowsError(try validateSourceTestJSON(Data(raw.utf8)))}
        XCTAssertNoThrow(try validateSourceTestJSON(Data(#"{"version":1,"test_debug_protocol":1,"checked_sources":{"version":"hash"}}"#.utf8)))
        XCTAssertThrowsError(try validateSourceTestJSON(Data(#"{"source_hashes":{"helper.py":"same","helper.py":"same"}}"#.utf8)))
        XCTAssertNoThrow(try validateSourceTestJSON(Data(#"{"checked_sources":{"é.py":"same"},"nested":{"é.py":"same"},"value":"escaped \" { } : text"}"#.utf8)))
        XCTAssertFalse(unambiguousTestSourcePaths(["é.py","e\u{301}.py"]));XCTAssertTrue(unambiguousTestSourcePaths(["é.py","é.py"]))
        let (before,facts,plan,_)=try debugFixture();var named=facts;named.input_path="é.py";named.source_hashes=["é.py":plan.source_hash];var choice=plan;choice.input_path="e\u{301}.py";choice.source_uri=languageURI(choice.input_path)
        XCTAssertFalse(choice.matches(before,named,facts.candidates[0],.nameFilter))
    }
    @MainActor func testUnavailableLegacyAndDirtyChoicesNeverStart()async throws {
        let (_,facts,plan,_)=try debugFixture(),config=URLSessionConfiguration.ephemeral;config.protocolClasses=[ResponseProtocol.self]
        let engine=try Engine(address:"http://fixture.invalid",session:URLSession(configuration:config)),space=Workspace(),buffer=DocumentBuffer(file:FileNode(name:"cases.md",path:"cases.md",dir:false),text:FileText(path:"cases.md",content:"draft",hash:"base")),debug=NativeDebugger(engine:engine)
        space.engine=engine;space.debugger=debug;space.buffers["cases.md"]=buffer;space.layout.activate("cases.md",in:"document-group");space.showTests=true
        let model=space.testSources,before=try XCTUnwrap(model.snapshot(space));model.entry="project.md";model.observation=(try XCTUnwrap(model.snapshot(space)),facts)
        var sent:[[String:Any]]=[];debug.emitForTest={sent.append($0)};ResponseProtocol.status=200;defer{ResponseProtocol.responsesByPath=[:];debug.close()}
        var unavailable=plan;unavailable.available=false;unavailable.reason="Provider unavailable for this explicit filter"
        ResponseProtocol.responsesByPath["/api/tests/debug-plan"]=try JSONEncoder().encode(unavailable)
        await model.debug(facts.candidates[0],.nameFilter,workspace:space);XCTAssertTrue(sent.isEmpty);XCTAssertTrue(model.notice.contains("Provider unavailable"),model.notice);XCTAssertFalse(model.preparingDebug)
        var raw=try XCTUnwrap(JSONSerialization.jsonObject(with:JSONEncoder().encode(plan)) as? [String:Any]);raw["test_debug_protocol"]=true
        ResponseProtocol.responsesByPath["/api/tests/debug-plan"]=try JSONSerialization.data(withJSONObject:raw)
        await model.debug(facts.candidates[0],.nameFilter,workspace:space);XCTAssertTrue(sent.isEmpty);XCTAssertFalse(model.preparingDebug)
        ResponseProtocol.resetRequests();buffer.text="unsaved"
        await model.debug(facts.candidates[0],.nameFilter,workspace:space);XCTAssertTrue(sent.isEmpty);XCTAssertEqual(ResponseProtocol.requestCount("/api/tests/debug-plan"),0)
        buffer.text=before.request.source;space.showTests=false;XCTAssertFalse(model.preparingDebug)
    }
    @MainActor func testHeldPlanCannotReviveClosedOrForeignContext()async throws {
        for change in ["closed","editor","draft","saving","deleted","engine","observation","debugger","contributor"] {
            let (_,base,plan,_)=try debugFixture(),configuration=URLSessionConfiguration.ephemeral;configuration.protocolClasses=[ResponseProtocol.self]
            let engine=try Engine(address:"http://fixture.invalid",session:URLSession(configuration:configuration)),space=Workspace(),buffer=DocumentBuffer(file:FileNode(name:"cases.md",path:"cases.md",dir:false),text:FileText(path:"cases.md",content:"draft",hash:"base")),helper=DocumentBuffer(file:FileNode(name:"helper.md",path:"helper.md",dir:false),text:FileText(path:"helper.md",content:"helper",hash:"base")),debug=NativeDebugger(engine:engine)
            space.engine=engine;space.debugger=debug;space.buffers=["cases.md":buffer,"helper.md":helper];space.layout.activate("cases.md",in:"document-group");space.showTests=true
            var facts=base;facts.source_hashes["helper.md"]=nativeSourceHash(helper.text)
            let model=space.testSources;model.entry="project.md";model.observation=(try XCTUnwrap(model.snapshot(space)),facts)
            var sent:[[String:Any]]=[];debug.emitForTest={sent.append($0)}
            let entered=DispatchSemaphore(value:0),release=DispatchSemaphore(value:0)
            ResponseProtocol.status=200;ResponseProtocol.responsesByPath["/api/tests/debug-plan"]=try JSONEncoder().encode(plan)
            ResponseProtocol.beforeResponse={request in if request.url?.path=="/api/tests/debug-plan"{entered.signal();_ = release.wait(timeout:.now()+5)}}
            let task=Task{await model.debug(facts.candidates[0],.nameFilter,workspace:space)}
            var waiting=false;for _ in 0..<200{if entered.wait(timeout:.now()) == .success{waiting=true;break};try await Task.sleep(for:.milliseconds(5))}
            XCTAssertTrue(waiting,change)
            switch change {
            case "closed":space.close("cases.md");space.show("cases.md")
            case "editor":space.buffers["cases.md"]=DocumentBuffer(file:buffer.file,text:FileText(path:buffer.file.path,content:buffer.text,hash:"base"))
            case "draft":buffer.text="changed"
            case "saving":buffer.saving=true
            case "deleted":buffer.resourceDeleted=true
            case "engine":space.engine=try Engine(address:"http://fixture.invalid",session:URLSession(configuration:.ephemeral))
            case "observation":model.observation=nil
            case "debugger":debug.close()
            default:space.buffers["helper.md"]=DocumentBuffer(file:helper.file,text:FileText(path:helper.file.path,content:helper.text,hash:"base"))
            }
            release.signal();await task.value;ResponseProtocol.beforeResponse=nil;ResponseProtocol.responsesByPath=[:]
            XCTAssertTrue(sent.isEmpty,change);XCTAssertFalse(model.preparingDebug,change);XCTAssertTrue(model.debugRuns.isEmpty,change);XCTAssertEqual(buffer.text,change=="draft" ? "changed":"draft");debug.close()
        }
    }
    @MainActor func testSameBytesCannotAdoptAnotherEditorObservation()throws {
        let (_,facts,_,_)=try debugFixture(),space=Workspace(),engine=try Engine(address:"http://fixture.invalid",session:URLSession(configuration:.ephemeral)),file=FileNode(name:"cases.md",path:"cases.md",dir:false),buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:"draft",hash:"base"))
        space.engine=engine;space.buffers[file.path]=buffer;space.layout.activate(file.path,in:"document-group");let model=space.testSources;model.entry="project.md";model.observation=(try XCTUnwrap(model.snapshot(space)),facts)
        XCTAssertTrue(model.current(space));space.buffers[file.path]=DocumentBuffer(file:file,text:FileText(path:file.path,content:"draft",hash:"base"));XCTAssertFalse(model.current(space))
        space.buffers[file.path]=buffer;XCTAssertTrue(model.current(space));space.engine=try Engine(address:"http://fixture.invalid",session:URLSession(configuration:.ephemeral));XCTAssertFalse(model.current(space))
    }
    @MainActor func testTestChoiceRetiresOlderOrdinaryProfileRead()async throws {
        let configuration=URLSessionConfiguration.ephemeral;configuration.protocolClasses=[ResponseProtocol.self]
        let engine=try Engine(address:"http://fixture.invalid",session:URLSession(configuration:configuration)),buffer=DocumentBuffer(file:FileNode(name:"main.py",path:"main.py",dir:false),text:FileText(path:"main.py",content:"print(1)\n",hash:"base")),debug=NativeDebugger(engine:engine)
        var sent:[[String:Any]]=[];debug.emitForTest={sent.append($0)}
        ResponseProtocol.status=200;ResponseProtocol.responsesByPath["/api/launch-profiles"]=Data(#"{"debug_protocol":3,"profiles":[{"id":"python","name":"Python","can_debug":true,"program":"main.py"}],"fingerprint":"current"}"#.utf8)
        let entered=DispatchSemaphore(value:0),release=DispatchSemaphore(value:0)
        ResponseProtocol.beforeResponse={request in if request.url?.path=="/api/launch-profiles"{entered.signal();_ = release.wait(timeout:.now()+5)}}
        defer{ResponseProtocol.beforeResponse=nil;ResponseProtocol.responsesByPath=[:];debug.close()}
        let older=Task{await debug.prepare(buffer)};var waiting=false
        for _ in 0..<200{if entered.wait(timeout:.now()) == .success{waiting=true;break};try await Task.sleep(for:.milliseconds(5))}
        XCTAssertTrue(waiting);let choice=try XCTUnwrap(debug.claimTestPreparation());release.signal();await older.value
        XCTAssertTrue(debug.testPreparationCurrent(choice));XCTAssertTrue(sent.isEmpty);XCTAssertTrue(debug.profiles.isEmpty);XCTAssertFalse(debug.active)
    }
    @MainActor func testOwnedTestStartMismatchAndClosedPreparationAreStopped()throws {
        let (before,facts,plan,binding)=try debugFixture(),buffer=DocumentBuffer(file:FileNode(name:before.request.path,path:before.request.path,dir:false),text:FileText(path:before.request.path,content:before.request.source,hash:"base")),debug=NativeDebugger(engine:try Engine(address:"http://fixture.invalid"))
        var sent:[[String:Any]]=[];debug.emitForTest={sent.append($0)};var admitted:[String]=[],rejected:[String]=[]
        let body=try sourceTestRun(before,facts,before,id:"case",selection:.nameFilter)
        let helper=DocumentBuffer(file:FileNode(name:"helper.py",path:"helper.py",dir:false),text:FileText(path:"helper.py",content:"helper",hash:"base"));debug.toggle(helper,line:0)
        debug.launchSourceTest(buffer,body:body,plan:plan,facts:facts,current:{true},admitted:{admitted.append($0)},rejected:{rejected.append($0)})
        let start=try XCTUnwrap(sent.last);XCTAssertEqual(start["op"] as? String,"start_test");XCTAssertNotNil(start["source_test"]);XCTAssertNil(start["profile"]);XCTAssertNil(start["program"]);XCTAssertNil(start["project"]);XCTAssertEqual((start["source_hashes"] as? [String:String])?[languageURI("helper.py")],nativeSourceHash("helper"));XCTAssertEqual((start["sources"] as? [String:[[String:Any]]])?[languageURI("helper.py")]?.count,1)
        var event:[String:Any]=["event":"started","doc":plan.source_uri,"session":"owned","request_id":try XCTUnwrap(debug.launchRequestID),"source_hash":plan.source_hash,"source_test":binding,"capabilities":[:]]
        var wrong=binding;wrong["candidate_id"]="foreign";event["source_test"]=wrong;debug.receive(event)
        XCTAssertEqual(sent.last?["op"] as? String,"stop");XCTAssertEqual(sent.last?["session"] as? String,"owned");XCTAssertTrue(admitted.isEmpty);XCTAssertEqual(rejected.count,1);XCTAssertNil(debug.sessionID)
        debug.launchSourceTest(buffer,body:body,plan:plan,facts:facts,current:{true},admitted:{admitted.append($0)},rejected:{rejected.append($0)})
        event["request_id"]=debug.launchRequestID;event["source_test"]=binding;debug.retireSourceTestPreparation();debug.receive(event);XCTAssertEqual(sent.last?["op"] as? String,"stop");XCTAssertTrue(admitted.isEmpty)
        let count=sent.count;event["request_id"]="foreign-launch";debug.receive(event);XCTAssertEqual(sent.count,count)
        debug.launchSourceTest(buffer,body:body,plan:plan,facts:facts,current:{true},admitted:{admitted.append($0)},rejected:{rejected.append($0)})
        let timed=try XCTUnwrap(debug.launchRequestID);debug.expireSourceTestPreparation("foreign");XCTAssertEqual(debug.status,"starting");debug.expireSourceTestPreparation(timed);XCTAssertEqual(debug.status,"failed");event["request_id"]=timed;event["session"]="late-owned";debug.receive(event);XCTAssertEqual(sent.last?["session"] as? String,"late-owned");XCTAssertEqual(sent.last?["op"] as? String,"stop");XCTAssertTrue(admitted.isEmpty)
        debug.launchSourceTest(buffer,body:body,plan:plan,facts:facts,current:{true},admitted:{admitted.append($0)},rejected:{rejected.append($0)});event["request_id"]=debug.launchRequestID;event["session"]="accepted";debug.receive(event)
        XCTAssertEqual(admitted,["accepted"]);debug.retireSourceTestPreparation();XCTAssertEqual(debug.sessionID,"accepted");XCTAssertEqual(debug.retainedSessionID,"accepted")
        let admittedCount=sent.count;debug.launchSourceTest(buffer,body:body,plan:plan,facts:facts,current:{true},admitted:{_ in XCTFail("replaced active runtime")},rejected:{rejected.append($0)});XCTAssertEqual(sent.count,admittedCount)
    }
    @MainActor func testOwnedPreparationReceiptRetiresOnlyTheInitialDeadline()throws {
        let (before,facts,plan,binding)=try debugFixture()
        let file=FileNode(name:before.request.path,path:before.request.path,dir:false)
        let buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:before.request.source,hash:"base"))
        let debug=NativeDebugger(engine:try Engine(address:"http://fixture.invalid"))
        var sent:[[String:Any]]=[],admitted:[String]=[],rejected:[String]=[]
        debug.emitForTest={sent.append($0)}
        defer{debug.close()}
        let body=try sourceTestRun(before,facts,before,id:"case",selection:.nameFilter)
        debug.launchSourceTest(buffer,body:body,plan:plan,facts:facts,current:{true},
            admitted:{admitted.append($0)},rejected:{rejected.append($0)})
        let request=try XCTUnwrap(debug.launchRequestID)
        let preparation:[String:Any]=["event":"test_preparing","doc":plan.source_uri,"request_id":request,
                                      "wait_ms":180_000,"source_test":binding]
        var foreign=preparation;foreign["request_id"]="foreign"
        var oversized=preparation;oversized["wait_ms"]=180_001
        var masquerading=preparation;masquerading["session"]="fake"
        var scalar=preparation;scalar["source_test"]="not a binding"
        var boolean=preparation;boolean["wait_ms"]=true
        debug.receive(foreign);debug.receive(oversized);debug.receive(masquerading)
        debug.receive(scalar);debug.receive(boolean)
        XCTAssertTrue(debug.message.isEmpty)
        debug.receive(preparation);debug.receive(preparation)
        XCTAssertTrue(debug.message.contains("Preparing the saved test"))
        XCTAssertTrue(admitted.isEmpty);XCTAssertNil(debug.sessionID)
        // Drive the callback that was armed for 30 seconds without a wall-clock sleep.
        debug.expireSourceTestPreparation(request)
        XCTAssertEqual(debug.status,"starting")
        debug.expireSourceTestPreparation(request,preparing:true)
        XCTAssertEqual(debug.status,"failed");XCTAssertTrue(rejected.last?.contains("preparation wait") == true)
        let late:[String:Any]=["event":"started","doc":plan.source_uri,"request_id":request,
            "session":"late-owned","source_hash":plan.source_hash,"source_test":binding,"capabilities":[:]]
        debug.receive(late);debug.receive(late)
        XCTAssertTrue(admitted.isEmpty)
        XCTAssertEqual(sent.compactMap{$0["op"] as? String},["start_test","stop"])
        XCTAssertEqual(sent.last?["session"] as? String,"late-owned")
    }
    func testExplicitSelectionAndDynamicRefusal()throws {
        let (before,facts)=fixture();let value=try sourceTestRun(before,facts,before,id:"case",selection:.nameFilter)
        XCTAssertEqual(value["fingerprint"] as? String,"scan");XCTAssertEqual((value["input"] as? [String:String])?["kind"],"saved")
        var dynamic=facts;dynamic.candidates[0].confidence="dynamic"
        XCTAssertThrowsError(try sourceTestRun(before,dynamic,before,id:"case",selection:.nameFilter))
        dynamic.candidates[0].name_filter=nil
        XCTAssertThrowsError(try sourceTestRun(before,dynamic,before,id:"case",selection:.nameFilter))
        XCTAssertNoThrow(try sourceTestRun(before,dynamic,before,id:"case",selection:.containingScope))
        XCTAssertThrowsError(try sourceTestRun(before,facts,before,id:"missing",selection:.nameFilter))
        var scope=facts.candidates[0];scope.run_scope="file_filter";XCTAssertEqual(scope.containingScopeName,"file")
    }
}
