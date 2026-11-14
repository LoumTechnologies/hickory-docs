import XCTest
@testable import HickorySimple
@MainActor final class SourceDigestsTests:XCTestCase {
    func testCoalescedHashingRejectsObsoleteUnicodeBytesAndClosedReplies() async {
        var jobs:[(String,CheckedContinuation<String,Error>)]=[]
        let index=SourceDigests {source in try await withCheckedThrowingContinuation {jobs.append((source,$0))}}
        var source="é";index.enqueue("a",source:source,current:{source});try? await Task.sleep(for:.milliseconds(1))
        source="e\u{301}";index.enqueue("a",source:"intermediate",current:{source});index.enqueue("a",source:source,current:{source});index.enqueue("b",source:"other",current:{"other"})
        XCTAssertEqual(jobs.count,1);jobs[0].1.resume(returning:"old")
        for _ in 0..<10 {try? await Task.sleep(for:.milliseconds(1))}
        XCTAssertNil(index.current("a",source:"é"));XCTAssertEqual(jobs.count,2);XCTAssertTrue(exactUTF8(jobs[1].0,source))
        jobs[1].1.resume(returning:"new");for _ in 0..<10 {try? await Task.sleep(for:.milliseconds(1))}
        XCTAssertEqual(index.current("a",source:source),"new");XCTAssertNil(index.current("a",source:"é"));XCTAssertEqual(jobs.count,3)
        index.stop();jobs[2].1.resume(returning:"late");for _ in 0..<10 {try? await Task.sleep(for:.milliseconds(1))};XCTAssertNil(index.current("b",source:"other"))
    }
    func testFailureDoesNotBlockAnotherSource() async {
        let index=SourceDigests {source in if source=="bad"{throw EngineError(message:"unavailable")};return "good"}
        index.enqueue("a",source:"bad",current:{"bad"});index.enqueue("b",source:"ok",current:{"ok"});for _ in 0..<20 {try? await Task.sleep(for:.milliseconds(1))}
        XCTAssertNil(index.current("a",source:"bad"));XCTAssertEqual(index.current("b",source:"ok"),"good");index.stop()
    }
}
