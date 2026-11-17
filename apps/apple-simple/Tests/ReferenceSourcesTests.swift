import XCTest
@testable import HickorySimple
@MainActor final class ReferenceSourcesTests:XCTestCase {
    func testRetainedDraftSynchronizesOnceBeforeFreshProposal()async throws {
        var round=0;var reads:[String]=[]
        let result=try await acquireSourceProposal(["retained":SourceSnapshot(text:"original draft",writable:true)],query:{round+=1;return (paths:["retained","retained"],value:round)},acquire:{path in reads.append(path);return SourceSnapshot(text:"original draft",writable:true)},current:{true})
        XCTAssertEqual(result.value,2);XCTAssertEqual(reads,["retained"]);XCTAssertEqual(result.sources["retained"]?.text,"original draft")
    }
    struct Read {var path:String;var continuation:CheckedContinuation<SourceSnapshot,Error>}
    func settle(_ condition:@escaping ()->Bool)async{let deadline=Date().addingTimeInterval(2);while !condition()&&Date()<deadline{try? await Task.sleep(for:.milliseconds(1))}}
    func testSequentialAcquisitionDeduplicatesAndRequeries()async throws {
        var queries=0;var reads:[Read]=[]
        let task=Task {try await acquireReferenceSources(["retained":SourceSnapshot(text:"original draft",writable:true)],query:{queries+=1;return ["a","b","a","retained"]},acquire:{path in try await withCheckedThrowingContinuation{reads.append(Read(path:path,continuation:$0))}},current:{true})}
        await settle{reads.count==1};XCTAssertEqual(reads.map(\.path),["a"]);reads[0].continuation.resume(returning:SourceSnapshot(text:"one",writable:true));await settle{reads.count==2};XCTAssertEqual(reads.map(\.path),["a","b"]);XCTAssertEqual(queries,1)
        reads[1].continuation.resume(returning:SourceSnapshot(text:"two",writable:true));await settle{reads.count==3};XCTAssertEqual(reads[2].path,"retained");reads[2].continuation.resume(returning:SourceSnapshot(text:"original draft",writable:true));let result=try await task.value
        XCTAssertEqual(queries,2);XCTAssertEqual(result.references,["a","b","retained"])
    }
    func testTextWritableAndUnicodeByteDriftRefuse()async {
        for next in [SourceSnapshot(text:"changed",writable:true),SourceSnapshot(text:"é",writable:false),SourceSnapshot(text:"e\u{301}",writable:true)] {
            do{_ = try await acquireReferenceSources(["a":SourceSnapshot(text:"é",writable:true)],query:{["a"]},acquire:{_ in next},current:{true});XCTFail("drift accepted")}catch{XCTAssertTrue(error.localizedDescription.contains("changed"))}
        }
        do{_ = try await acquireReferenceSources([:],query:{["a"]},acquire:{_ in SourceSnapshot(text:"source",writable:false)},current:{true});XCTFail("readonly accepted")}catch{XCTAssertTrue(error.localizedDescription.contains("read-only"))}
    }
    func testStaleReadStopsSubsequentDispatchAndQuery()async {
        var current=true;var reads:[Read]=[];var queries=0
        let task=Task {try await acquireReferenceSources([:],query:{queries+=1;return ["a","b","c"]},acquire:{path in try await withCheckedThrowingContinuation{reads.append(Read(path:path,continuation:$0))}},current:{current})}
        await settle{reads.count==1};current=false;reads[0].continuation.resume(returning:SourceSnapshot(text:"source",writable:true))
        do{_ = try await task.value;XCTFail("obsolete accepted")}catch{XCTAssertTrue(error.localizedDescription.contains("changed"))};XCTAssertEqual(reads.count,1);XCTAssertEqual(queries,1)
    }
    func testExpandingAndOversizedDiscoveryRefuse()async {
        var round=0
        do{_ = try await acquireReferenceSources([:],query:{defer{round+=1};return ["new-\(round)"]},acquire:{_ in SourceSnapshot(text:"source",writable:true)},current:{true});XCTFail("expansion accepted")}catch{XCTAssertTrue(error.localizedDescription.contains("kept expanding"))};XCTAssertEqual(round,4)
        var reads=0
        do{_ = try await acquireReferenceSources([:],query:{(0...10000).map(String.init)},acquire:{_ in reads+=1;return SourceSnapshot(text:"",writable:true)},current:{true});XCTFail("oversized accepted")}catch{XCTAssertTrue(error.localizedDescription.contains("10,000"))};XCTAssertEqual(reads,0)
    }
    func testReadonlyContextRequiresExplicitPolicyAndStillRefusesDrift()async throws {
        let original=SourceSnapshot(text:"context",writable:false)
        let result=try await acquireSourceProposal(["context":original],query:{(paths:["context"],value:1)},acquire:{_ in original},current:{true},requireWritable:{_ in false});XCTAssertFalse(result.sources["context"]!.writable)
        do{_ = try await acquireSourceProposal(["context":original],query:{(paths:["context"],value:1)},acquire:{_ in SourceSnapshot(text:"context",writable:true)},current:{true},requireWritable:{_ in false});XCTFail("drift accepted")}catch{XCTAssertTrue(error.localizedDescription.contains("changed"))}
    }
}
