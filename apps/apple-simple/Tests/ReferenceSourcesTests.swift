import XCTest
@testable import HickorySimple
@MainActor final class ReferenceSourcesTests:XCTestCase {
    struct Read {var path:String;var continuation:CheckedContinuation<SourceSnapshot,Error>}
    func settle(_ condition:@escaping ()->Bool)async{let deadline=Date().addingTimeInterval(2);while !condition()&&Date()<deadline{try? await Task.sleep(for:.milliseconds(1))}}
    func testBoundedAcquisitionRequeriesWithoutReplacingRetainedDrafts()async throws {
        var queries=0;var reads:[Read]=[]
        let task=Task {try await acquireReferenceSources(["retained":SourceSnapshot(text:"original draft",writable:true)],query:{queries+=1;return ["a","b","a","c","retained"]},acquire:{path in try await withCheckedThrowingContinuation{reads.append(Read(path:path,continuation:$0))}},current:{true})}
        await settle{reads.count==2};XCTAssertEqual(reads.map(\.path),["a","b"]);XCTAssertEqual(queries,1)
        reads[0].continuation.resume(returning:SourceSnapshot(text:"one",writable:true));await settle{reads.count==3};XCTAssertEqual(reads[2].path,"c");XCTAssertEqual(queries,1)
        reads[1].continuation.resume(returning:SourceSnapshot(text:"two",writable:true));reads[2].continuation.resume(returning:SourceSnapshot(text:"three",writable:true));let value=try await task.value
        XCTAssertEqual(queries,2);XCTAssertEqual(value.references,["a","b","c","retained"]);XCTAssertEqual(value.sources["retained"]?.text,"original draft");XCTAssertEqual(value.sources.count,4)
    }
    func testStaleAcquisitionDrainsAndExpandingTargetsRefuse()async {
        var current=true;var reads:[Read]=[]
        let task=Task {try await acquireReferenceSources([:],query:{["a","b","c"]},acquire:{path in try await withCheckedThrowingContinuation{reads.append(Read(path:path,continuation:$0))}},current:{current})}
        await settle{reads.count==2};current=false;reads.forEach{$0.continuation.resume(returning:SourceSnapshot(text:"source",writable:true))}
        do{_ = try await task.value;XCTFail("obsolete request accepted")}catch{XCTAssertTrue(error.localizedDescription.contains("changed"))};XCTAssertEqual(reads.count,2)
        var round=0
        do{_ = try await acquireReferenceSources([:],query:{defer{round+=1};return ["new-\(round)"]},acquire:{_ in SourceSnapshot(text:"source",writable:true)},current:{true});XCTFail("expanding discovery accepted")}catch{XCTAssertTrue(error.localizedDescription.contains("kept expanding"))};XCTAssertEqual(round,4)
    }
}
