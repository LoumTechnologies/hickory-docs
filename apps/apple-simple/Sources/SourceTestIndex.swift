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
    func current(_ path:String,source:String)->IndexedSourceTests? {guard let item=records[path],item.source==source,item.facts != nil else {return nil};return item}
    func enqueue(_ path:String,source:String,current:@escaping ()->String?){
        guard !closed else {return}
        if self.current(path,source:source) != nil || active[path]?.source==source {pending[path]=nil;pump();return}
        if pending[path]?.source==source {return}
        pending[path]=Job(path:path,source:source,due:Date().addingTimeInterval(debounce),current:current);pump()
    }
    func stop(){closed=true;timer?.cancel();timer=nil;pending=[:]}
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
            guard value.version==1,value.input_path==job.path else {throw EngineError(message:"Unsupported source test observation. Refresh the app and engine together.")};facts=value
        }catch{notice=error.localizedDescription}
        if !closed,job.current()==job.source,pending[job.path]==nil {var record=IndexedSourceTests(path:job.path,source:job.source,facts:facts,notice:notice);record.rows=indexedTestRows(record);records[job.path]=record}
        active[job.path]=nil;pump()
    }
}
func indexedTestRows(_ record:IndexedSourceTests?)->[(line:Int,candidate:SourceTestCandidate)] {
    if let rows=record?.rows {return rows}
    guard let record,let facts=record.facts else {return []}
    let lines=record.source.components(separatedBy:"\n")
    return facts.candidates.compactMap {candidate in
        guard let point=candidate.source ?? (facts.scope=="plain" ? candidate.target:nil),point.path==record.path,point.line>0,point.line<=lines.count,point.character>=0,point.character<=lines[point.line-1].utf16.count else {return nil}
        return (point.line-1,candidate)
    }
}
