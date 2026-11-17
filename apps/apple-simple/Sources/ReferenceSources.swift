import Foundation
struct ReferenceSourceAcquisition {var sources:NativeSourceMap<SourceSnapshot>;var references:[String]}
struct SourceProposalAcquisition<Value> {var sources:NativeSourceMap<SourceSnapshot>;var paths:[String];var value:Value}
@MainActor func acquireSourceProposal<Value>(_ initial:NativeSourceMap<SourceSnapshot>,query:@escaping () async throws->(paths:[String],value:Value),acquire:@escaping (String) async throws->SourceSnapshot,current:@escaping ()->Bool,requireWritable:@escaping (String)->Bool={_ in true}) async throws->SourceProposalAcquisition<Value> {
    var sources=initial;var synchronized=Set<NativeSourceKey>()
    func check()throws{guard current() else{throw EngineError(message:"Source, selection or provider changed. Prepare again.")}}
    for _ in 0..<4 {
        try check();let proposal=try await query(),references=Set(proposal.paths.map(NativeSourceKey.init)).map(\.string).sorted();try check()
        guard references.count<=10000 else{throw EngineError(message:"Reference discovery exceeds 10,000 source targets. Narrow the change.")}
        let pending=references.filter {!synchronized.contains(NativeSourceKey($0))};if pending.isEmpty{return SourceProposalAcquisition(sources:sources,paths:references,value:proposal.value)}
        for path in pending {
            try check();let value=try await acquire(path);try check()
            if let original=sources[path],!original.text.utf8.elementsEqual(value.text.utf8)||original.writable != value.writable{throw EngineError(message:"Source changed while synchronizing: \(path). Prepare again.")}
            guard !requireWritable(path)||value.writable else{throw EngineError(message:"Source is generated, read-only or saving: \(path)")}
            if sources[path]==nil{sources[path]=value};synchronized.insert(NativeSourceKey(path))
        }
        try check()
    }
    throw EngineError(message:"Reference targets kept expanding. Refresh project analysis and prepare again. No edits were applied.")
}
@MainActor func acquireReferenceSources(_ initial:NativeSourceMap<SourceSnapshot>,query:@escaping () async throws->[String],acquire:@escaping (String) async throws->SourceSnapshot,current:@escaping ()->Bool,requireWritable:@escaping (String)->Bool={_ in true}) async throws->ReferenceSourceAcquisition {
    let result=try await acquireSourceProposal(initial,query:{(paths:try await query(),value:false)},acquire:acquire,current:current,requireWritable:requireWritable)
    return ReferenceSourceAcquisition(sources:result.sources,references:result.paths)
}
