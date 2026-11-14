<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Index test declarations without holding the editor on apple

For maintainers replacing source discovery without coupling it to a gutter renderer.

Keep one observed source value per file. Repeated requests for the same active or successful observation do not multiply reads. A changed draft retires precise access immediately through the source equality check. Debounce pending work while retaining only its latest text.

<ui:copy id="source-test-index-apple-observations">
import Foundation
import Observation
struct IndexedSourceTests {var path:String;var source:String;var facts:SourceTestScan?;var notice:String;var rows:[(line:Int,candidate:SourceTestCandidate)]?=nil}
@MainActor @Observable final class SourceTestIndex {
    private(set) var records:[String:IndexedSourceTests]=[:]
    private struct Job {var path:String;var source:String;var due:Date;var current:()->String?}
    @ObservationIgnored private var pending:[String:Job]=[:]
    @ObservationIgnored private var active:[String:Job]=[:]
    @ObservationIgnored private var timer:Task<Void,Never>?
    @ObservationIgnored private var closed=false
    private let debounce:Double
    private let read:(String,String) async throws->SourceTestScan
    init(debounce:Double=0.25,read:@escaping (String,String) async throws->SourceTestScan){self.debounce=debounce;self.read=read}
    var inFlight:Int {active.count};var queued:Int {pending.count}
    func current(_ path:String,source:String)->IndexedSourceTests? {guard let item=records[path],exactUTF8(item.source,source),item.facts != nil else {return nil};return item}
    func enqueue(_ path:String,source:String,current:@escaping ()->String?){
        guard !closed else {return}
        if self.current(path,source:source) != nil || exactUTF8(active[path]?.source,source) {pending[path]=nil;pump();return}
        if exactUTF8(pending[path]?.source,source) {return}
        pending[path]=Job(path:path,source:source,due:Date().addingTimeInterval(debounce),current:current);pump()
    }
    func stop(){closed=true;timer?.cancel();timer=nil;pending=[:]}
</ui:copy>

Admit at most two reads and never overlap reads of the same path. Only ready jobs consume workers; another file can progress while one fails. The supplied current-source callback and absence of a newer queued value decide whether a reply can be installed. Stop refuses new work and late replies without claiming the underlying provider is immediately terminated.

<ui:copy id="source-test-index-apple-workers">
    private func pump(){
        timer?.cancel();timer=nil;guard !closed else {return}
        let now=Date()
        for job in pending.values.sorted(by:{$0.due<$1.due}) {
            if active.count>=2 {break}
            if active[job.path] != nil || job.due>now {continue}
            pending[job.path]=nil;active[job.path]=job
            Task {await perform(job)}
        }
        if active.count<2,let next=pending.values.filter({active[$0.path]==nil}).map(\.due).min(){
            timer=Task { [weak self] in
                do {try await Task.sleep(for:.seconds(max(0,next.timeIntervalSinceNow)))}catch{return}
                guard !Task.isCancelled else {return};self?.pump()
            }
        }
    }
    private func perform(_ job:Job) async {
        var facts:SourceTestScan?;var notice=""
        do {let value=try await read(job.path,job.source)
            guard value.version==1,exactUTF8(value.input_path,job.path) else {throw EngineError(message:"Unsupported source test observation. Refresh the app and engine together.")};facts=value
        }catch{notice=error.localizedDescription}
        if !closed,exactUTF8(job.current(),job.source),pending[job.path]==nil {var record=IndexedSourceTests(path:job.path,source:job.source,facts:facts,notice:notice);record.rows=indexedTestRows(record);records[job.path]=record}
        active[job.path]=nil;pump()
    }
}
</ui:copy>

Use compiler-owned authored locations. An unmapped constructed runtime line cannot become a source gutter mark, and a foreign authored owner belongs to its own file. Plain files may use their runtime location because that is the supplied source. Validate line and UTF-16 column bounds once, and cache rows on the installed observation so each rendered gutter line does not split the whole source again. This primary-source index supplies inspection hints; opening a hint must refresh the full source-test view before offering execution against current contributors.

<ui:copy id="source-test-index-apple-locations">
func indexedTestRows(_ record:IndexedSourceTests?)->[(line:Int,candidate:SourceTestCandidate)] {
    if let rows=record?.rows {return rows}
    guard let record,let facts=record.facts else {return []}
    let lines=record.source.components(separatedBy:"\n")
    return facts.candidates.compactMap {candidate in
        guard let point=candidate.source ?? (facts.scope=="plain" ? candidate.target:nil),exactUTF8(point.path,record.path),point.line>0,point.line<=lines.count,point.character>=0,point.character<=lines[point.line-1].utf16.count else {return nil}
        return (point.line-1,candidate)
    }
}
</ui:copy>

These focused witnesses hold real asynchronous reads to prove the two-worker bound, duplicate suppression, edit coalescing, stale refusal, independent failure and stop behavior. Location witnesses distinguish primary, foreign and missing compiler ownership. They do not execute a language runner or establish native pointer acceptance.

<ui:copy id="source-test-index-apple-witness">
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
</ui:copy>

<ui:file path="apps/apple-simple/Sources/SourceTestIndex.swift"><ui:paste select="#source-test-index-apple-observations"/><ui:paste select="#source-test-index-apple-workers"/><ui:paste select="#source-test-index-apple-locations"/></ui:file>
<ui:file path="apps/apple-simple/Tests/SourceTestIndexTests.swift"><ui:paste select="#source-test-index-apple-witness"/></ui:file>
</ui:doc>
