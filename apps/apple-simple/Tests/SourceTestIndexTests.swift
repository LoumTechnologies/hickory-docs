import XCTest
@testable import HickorySimple
@MainActor final class SourceTestIndexTests:XCTestCase {
    func facts(_ path:String)->SourceTestScan {SourceTestScan(version:1,input_path:path,fingerprint:"scan",source_hashes:[path:"hash"],candidates:[],scope:"plain",syntax_complete:true,notices:[],unsupported_languages:[],unclassified_outputs:[],unused_sources:[])}
    func settle(_ condition:@escaping ()->Bool) async {let deadline=Date().addingTimeInterval(2);while !condition(),Date()<deadline {try? await Task.sleep(for:.milliseconds(1))};XCTAssertTrue(condition())}
    struct Read {var path:String;var source:String;var continuation:CheckedContinuation<SourceTestScan,Error>}
    func testCoalescesDraftsAndDrainsAfterIndependentFailure() async {
        var texts=["a":"first","b":"source","c":"source"],reads:[Read]=[]
        let index=SourceTestIndex(debounce:0){path,source in try await withCheckedThrowingContinuation {reads.append(Read(path:path,source:source,continuation:$0))}}
        func enqueue(_ path:String){index.enqueue(path,source:texts[path]!,current:{texts[path]})}
        enqueue("a");enqueue("a");enqueue("b");await settle {reads.count==2};XCTAssertEqual(index.inFlight,2)
        texts["a"]="middle";enqueue("a");texts["a"]="latest";enqueue("a");enqueue("c");XCTAssertEqual(index.queued,2)
        let first=reads.first(where:{$0.path=="a"})!;first.continuation.resume(returning:facts("a"));await settle {reads.count==3}
        XCTAssertNil(index.current("a",source:"first"));XCTAssertEqual(reads[2].source,"latest");XCTAssertEqual(index.inFlight,2)
        reads.first(where:{$0.path=="b"})!.continuation.resume(throwing:EngineError(message:"unavailable"));await settle {reads.count==4}
        XCTAssertTrue(index.records["b"]?.notice.contains("unavailable")==true);XCTAssertEqual(reads[3].path,"c")
        reads[2].continuation.resume(returning:facts("a"));reads[3].continuation.resume(returning:facts("c"));await settle {index.inFlight==0}
        XCTAssertNotNil(index.current("a",source:"latest"));XCTAssertNil(index.current("a",source:"changed"));XCTAssertNotNil(index.current("c",source:"source"))
        enqueue("a");XCTAssertEqual(reads.count,4);index.stop()
    }
    func testStopAndCompilerOwnedRows() async {
        var reads:[Read]=[];let index=SourceTestIndex(debounce:0){path,source in try await withCheckedThrowingContinuation {reads.append(Read(path:path,source:source,continuation:$0))}}
        for path in ["a","b","c"] {index.enqueue(path,source:"source",current:{"source"})};await settle {reads.count==2};index.stop()
        for read in reads {read.continuation.resume(returning:facts(read.path))};await settle {index.inFlight==0};XCTAssertTrue(index.records.isEmpty);XCTAssertEqual(index.queued,0)
        var scan=facts("note.md");scan.scope="document"
        let point=TestSourcePoint(path:"generated.py",line:2,character:4,hash:"hash")
        scan.candidates=[SourceTestCandidate(id:"case",language:"python",run_scope:"file",name:"test_case",name_filter:"test_case",confidence:"syntax",source:nil,target:point)]
        func record()->IndexedSourceTests {IndexedSourceTests(path:"note.md",source:"\nsource",facts:scan,notice:"")}
        XCTAssertTrue(indexedTestRows(record()).isEmpty);scan.candidates[0].source=TestSourcePoint(path:"other.md",line:2,character:4,hash:"hash");XCTAssertTrue(indexedTestRows(record()).isEmpty)
        scan.candidates[0].source=TestSourcePoint(path:"note.md",line:100,character:4,hash:"hash");XCTAssertTrue(indexedTestRows(record()).isEmpty)
        scan.candidates[0].source=TestSourcePoint(path:"note.md",line:2,character:4,hash:"hash");XCTAssertEqual(indexedTestRows(record()).first?.line,1)
    }
}
