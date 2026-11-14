import Foundation
struct ReferenceSourceAcquisition {var sources:[String:SourceSnapshot];var references:[String]}
@MainActor func acquireReferenceSources(_ initial:[String:SourceSnapshot],query:@escaping () async throws->[String],acquire:@escaping (String) async throws->SourceSnapshot,current:@escaping ()->Bool) async throws->ReferenceSourceAcquisition {
    var sources=initial
    func check()throws{guard current() else{throw EngineError(message:"Source, selection or provider changed. Prepare again.")}}
    for _ in 0..<4 {
        try check();let references=Array(Set(try await query())).sorted();try check()
        guard references.count<=10000 else{throw EngineError(message:"Reference discovery exceeds 10,000 source targets. Narrow the change.")}
        let missing=references.filter {sources[$0]==nil};if missing.isEmpty{return ReferenceSourceAcquisition(sources:sources,references:references)}
        var cursor=0;var failure:Error?
        @MainActor func worker()async {
            while failure==nil && cursor<missing.count {
                do{try check();let path=missing[cursor];cursor+=1;let value=try await acquire(path);try check()
                    guard value.writable else{throw EngineError(message:"Source is generated, read-only or saving: \(path)")};sources[path]=value
                }catch{failure=error}
            }
        }
        let first=Task {await worker()},second=Task {await worker()};_ = await(first.value,second.value)
        if let failure{throw failure};try check()
    }
    throw EngineError(message:"Reference targets kept expanding. Refresh project analysis and prepare again. No edits were applied.")
}
