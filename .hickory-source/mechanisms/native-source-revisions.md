<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Use exact bytes for native source revisions

For maintainers changing workspace services independently of the editor renderer.

Swift linguistic equality treats composed and decomposed Unicode forms as equal, but their UTF-8 bytes and UTF-16 offsets can differ. Source identity must preserve those differences. The witnesses require normalization-only edits to remain dirty and observable, retire older test observations, preserve real normalization proposals, reject superseded proposals and publish new language-snapshot bytes. They instantiate this product's own models; no language provider is simulated or started.

Use one explicit byte comparator at source admission, stale-response checks and source-derived cache keys. Shared contiguous storage can return immediately; other values compare their actual UTF-8 bytes rather than a normalized form. An exact-text value supplies byte-based Equatable semantics to Observation and SwiftUI change detection without replacing the String API used by editor renderers. Source maps compare UTF-8 path keys and exact values rather than linguistic dictionary equality. Nil equality alone grants no positional authority; callers still require a captured source.

<ui:copy id="native-exact-source-identity">
import Foundation
/** Linguistic equivalence cannot establish a code or protocol-byte revision. */
func exactUTF8(_ lhs:String?,_ rhs:String?)->Bool {
    switch (lhs,rhs) {
    case (.none,.none):return true
    case (.some(let lhs),.some(let rhs)):
        if let equal=lhs.utf8.withContiguousStorageIfAvailable({left in
            rhs.utf8.withContiguousStorageIfAvailable {right in
                guard left.count==right.count else {return false}
                if left.baseAddress==right.baseAddress {return true}
                return left.elementsEqual(right)
            }
        }).flatMap({$0}) {return equal}
        return lhs.utf8.elementsEqual(rhs.utf8)
    default:return false
    }
}
/** Observation must publish byte-distinct text even when String equality says equal. */
struct ExactSourceText:Equatable,Sendable {
    let text:String
    init(_ text:String){self.text=text}
    static func ==(lhs:Self,rhs:Self)->Bool {exactUTF8(lhs.text,rhs.text)}
}
func exactSourceValues(_ values:[String:String])->[Data:ExactSourceText] {
    Dictionary(uniqueKeysWithValues:values.map {(Data($0.key.utf8),ExactSourceText($0.value))})
}
</ui:copy>

<ui:copy id="native-source-revision-witness">
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
</ui:copy>

<ui:file path="apps/apple-simple/Tests/SourceRevisionTests.swift"><ui:paste select="#native-source-revision-witness"/></ui:file>
<ui:file path="apps/apple-simple/Sources/SourceRevision.swift"><ui:paste select="#native-exact-source-identity"/></ui:file>
</ui:doc>
