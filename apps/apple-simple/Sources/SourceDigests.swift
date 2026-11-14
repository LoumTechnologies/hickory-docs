import Foundation
import Observation
@MainActor @Observable final class SourceDigests {
    private struct Record {var source:ExactSourceText;var hash:String}
    private struct Job {var source:String;var current:()->String?}
    private var records:[String:Record]=[:]
    @ObservationIgnored private var pending:[String:Job]=[:]
    @ObservationIgnored private var order:[String]=[]
    @ObservationIgnored private var active=false
    @ObservationIgnored private var closed=false
    private let hash:(String) async throws->String
    init(hash:@escaping (String) async throws->String={source in await Task.detached {nativeSourceHash(source)}.value}){self.hash=hash}
    func current(_ path:String,source:String)->String? {guard let record=records[path],exactUTF8(record.source.text,source) else{return nil};return record.hash}
    func enqueue(_ path:String,source:String,current:@escaping ()->String?){guard !closed,self.current(path,source:source)==nil else{return};if pending[path]==nil{order.append(path)};pending[path]=Job(source:source,current:current);pump()}
    func stop(){closed=true;pending=[:];order=[]}
    private func pump(){guard !closed,!active,!order.isEmpty else{return};let path=order.removeFirst();guard let job=pending.removeValue(forKey:path) else{return};active=true
        Task {do{let value=try await hash(job.source)
            if !closed,exactUTF8(job.current(),job.source),pending[path]==nil{records[path]=Record(source:ExactSourceText(job.source),hash:value)}
        }catch{};active=false;pump()}
    }
}
