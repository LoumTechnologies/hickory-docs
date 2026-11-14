import XCTest
import Observation
@testable import HickorySimple
@MainActor final class SourceRevisionTests:XCTestCase {
    let before="let value = \"\u{e9}\"\n"
    let after="let value = \"e\u{301}\"\n"
    func buffer(_ text:String)->DocumentBuffer {DocumentBuffer(file:FileNode(name:"source.txt",path:"source.txt",dir:false),text:FileText(path:"source.txt",content:text,hash:nativeSourceHash(text)))}
    func testNormalizationOnlyEditIsDirtyAndPublishesARevision() async {
        XCTAssertEqual(before,after,"Swift's linguistic equality must not establish source identity")
        let value=buffer(before),changed=expectation(description:"byte-distinct source revision")
        withObservationTracking({_ = value.text},onChange:{changed.fulfill()})
        value.text=after
        XCTAssertNotEqual(nativeSourceHash(before),nativeSourceHash(after));XCTAssertTrue(value.text.utf8.elementsEqual(after.utf8));XCTAssertTrue(value.dirty)
        await fulfillment(of:[changed],timeout:0.2)
        value.base=after;XCTAssertFalse(value.dirty)
    }
    func testEqualLengthCanonicalFormsStillHaveDifferentRevisions() {
        let a="a\u{315}\u{300}",b="a\u{300}\u{315}"
        XCTAssertEqual(a,b);XCTAssertEqual(a.utf8.count,b.utf8.count)
        XCTAssertFalse(exactUTF8(a,b));XCTAssertNotEqual(ExactSourceText(a),ExactSourceText(b))
    }
    func testTestObservationRejectsDifferentSourceBytes() {
        let a=TestSourceSnapshot(workspace:"root",request:TestSourceRequest(path:"source.txt",source:before,projectEntry:nil))
        let b=TestSourceSnapshot(workspace:"root",request:TestSourceRequest(path:"source.txt",source:after,projectEntry:nil))
        let facts=SourceTestScan(version:1,input_path:"source.txt",fingerprint:"scan",source_hashes:["source.txt":nativeSourceHash(before)],candidates:[],scope:"plain",syntax_complete:true,notices:[],unsupported_languages:[],unclassified_outputs:[],unused_sources:[])
        XCTAssertFalse(testSourceCurrent(a,facts,b));XCTAssertNotEqual(a,b)
        var contributorA=a,contributorB=a,projectFacts=facts
        contributorA.request.sources["helper.md"]=before;contributorB.request.sources["helper.md"]=after
        projectFacts.source_hashes["helper.md"]=nativeSourceHash(before)
        XCTAssertFalse(testSourceCurrent(contributorA,projectFacts,contributorB))
    }
    func testNormalizationProposalAndStaleGuardUseSourceBytes()throws {
        let source="\u{e9}",replacement="e\u{301}"
        let edit:[String:Any]=["changes":[languageURI("source.txt"):[["range":["start":["line":0,"character":0],"end":["line":0,"character":1]],"newText":replacement]]]]
        let plan=try planWorkspaceEdit("Normalize source",raw:edit,sources:["source.txt":SourceSnapshot(text:source,writable:true)])
        XCTAssertTrue(try XCTUnwrap(plan.patches.first).after.utf8.elementsEqual(replacement.utf8))
        XCTAssertNotNil(validateChangePlan(plan,sources:["source.txt":SourceSnapshot(text:replacement,writable:true)]))
        XCTAssertNotEqual(SourceReplacement(from:0,to:1,text:source),SourceReplacement(from:0,to:1,text:replacement))
    }
    func testLanguageSnapshotPublishesCanonicalEquivalentBytes() async throws {
        let value=buffer(before),engine=try Engine(address:"http://127.0.0.1:1"),language=NativeLanguage(engine:engine,buffer:value)
        language.diagnosticSource=before
        let changed=expectation(description:"new diagnostic source bytes")
        withObservationTracking({_ = language.diagnosticSource},onChange:{changed.fulfill()})
        language.diagnosticSource=after
        await fulfillment(of:[changed],timeout:0.2)
    }
}
