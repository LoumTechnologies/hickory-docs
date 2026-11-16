import Foundation
struct TestSourceRequest: Equatable {
    static func ==(lhs:Self,rhs:Self)->Bool {exactUTF8(lhs.path,rhs.path)&&exactUTF8(lhs.source,rhs.source)&&exactUTF8(lhs.projectEntry,rhs.projectEntry)&&exactSourceValues(lhs.sources)==exactSourceValues(rhs.sources)}
    var path:String;var source:String;var projectEntry:String?;var sources:[String:String]=[:]
    var body:[String:Any] {var value:[String:Any]=["path":path,"source":source,"sources":sources];if let projectEntry {value["project_entry"]=projectEntry};return value}
}
struct TestSourceSnapshot:Equatable {var workspace:String;var request:TestSourceRequest
    static func ==(lhs:Self,rhs:Self)->Bool {exactUTF8(lhs.workspace,rhs.workspace)&&lhs.request==rhs.request}
}
struct TestSourcePoint:Decodable {var path:String;var line:Int;var character:Int;var hash:String}
struct SourceTestCandidate:Decodable,Identifiable {
    var id:String;var language:String;var run_scope:String;var name:String;var name_filter:String?;var confidence:String;var source:TestSourcePoint?;var target:TestSourcePoint
    var point:TestSourcePoint {source ?? target}
    var containingScopeName:String {run_scope=="file_filter" ? "file":run_scope.replacingOccurrences(of:"_",with:" ")}
}
struct SourceTestScan:Decodable {
    var version:Int;var input_path:String;var fingerprint:String;var source_hashes:[String:String];var candidates:[SourceTestCandidate]
    var scope:String;var syntax_complete:Bool;var notices:[String];var unsupported_languages:[String];var unclassified_outputs:[String];var unused_sources:[String]
    var workspace_id:String?=nil;var input_hash:String?=nil
}
func testSourceCurrent(_ before:TestSourceSnapshot,_ facts:SourceTestScan,_ now:TestSourceSnapshot)->Bool {
    let a=before.request,b=now.request
    guard facts.version==1,exactUTF8(facts.input_path,a.path),exactUTF8(before.workspace,now.workspace),exactUTF8(a.path,b.path),exactUTF8(a.source,b.source),exactUTF8(a.projectEntry,b.projectEntry) else {return false}
    let previous=exactSourceValues(a.sources),current=exactSourceValues(b.sources)
    return facts.source_hashes.keys.allSatisfy {exactUTF8($0,a.path) || previous[Data($0.utf8)]==current[Data($0.utf8)]}
}
enum TestSourceSelection:String {case nameFilter="name_filter",containingScope="containing_scope"}
func sourceTestRun(_ before:TestSourceSnapshot,_ facts:SourceTestScan,_ now:TestSourceSnapshot,id:String,selection:TestSourceSelection)throws->[String:Any] {
    guard testSourceCurrent(before,facts,now) else {throw EngineError(message:"Test source changed. Refresh candidates before running.")}
    guard let candidate=facts.candidates.first(where:{exactUTF8($0.id,id)}) else {throw EngineError(message:"Candidate unavailable. Refresh test candidates.")}
    guard selection != .nameFilter || (candidate.name_filter?.isEmpty==false && !exactUTF8(candidate.confidence,"dynamic")) else {throw EngineError(message:"This declaration needs runner discovery or an explicit containing-scope run.")}
    return ["version":1,"input":["kind":"saved"],"analysis":before.request.body,"fingerprint":facts.fingerprint,"candidate_id":id,"selection":["kind":selection.rawValue]]
}
// Foundation dictionaries merge canonical-equivalent keys before Decodable sees them.
// Audit raw object keys and numeric version lexemes before Foundation can merge or coerce aliases.
func validateSourceTestJSON(_ data:Data)throws {
    let bytes=Array(data);var objects:[[String:Data]]=[];var index=0
    while index<bytes.count {
        let byte=bytes[index]
        if byte==123{objects.append([:]);index+=1;continue}
        if byte==125{if !objects.isEmpty{objects.removeLast()};index+=1;continue}
        guard byte==34 else{index+=1;continue}
        let start=index;index+=1
        while index<bytes.count {if bytes[index]==92{index+=2;continue};if bytes[index]==34{break};index+=1}
        guard index<bytes.count else{throw EngineError(message:"Malformed source-test JSON string.")}
        index+=1;var next=index;while next<bytes.count,[UInt8(9),10,13,32].contains(bytes[next]){next+=1}
        if next<bytes.count,bytes[next]==58,!objects.isEmpty {
            let key=try JSONDecoder().decode(String.self,from:Data(bytes[start..<index])),position=objects.count-1
            guard objects[position][key]==nil else{throw EngineError(message:"Duplicate or canonically equivalent JSON keys are unsupported by this native host. Source-test admission was refused without merging contributors.")}
            objects[position][key]=Data(key.utf8)
            if key=="version" || key=="test_debug_protocol" {
                var value=next+1;while value<bytes.count,[UInt8(9),10,13,32].contains(bytes[value]){value+=1}
                if value<bytes.count,(bytes[value]==45 || (48...57).contains(bytes[value])) {
                    var end=value;while end<bytes.count,![UInt8(9),10,13,32,44,125,93].contains(bytes[end]){end+=1}
                    let token=bytes[value..<end]
                    guard !token.isEmpty,token.allSatisfy({(48...57).contains($0)}),token.count==1 || token.first != 48 else{throw EngineError(message:"Test Debug protocol requires a canonical integer version; numeric aliases were refused.")}
                }
            }
        }
    }
}
func unambiguousTestSourcePaths(_ paths:[String])->Bool {
    var seen:[String:Data]=[:]
    for path in paths{let bytes=Data(path.utf8);if let previous=seen[path],previous != bytes{return false};seen[path]=bytes}
    return true
}
struct SourceTestDebugBinding:Decodable {
    var version:Int;var workspace_id:String;var input_path:String;var input_hash:String;var analysis_fingerprint:String;var candidate_id:String;var candidate_name:String;var selection:SourceTestDebugSelection;var checked_sources:[String:String];var language:String;var framework:String
}
struct SourceTestDebugSelection:Codable {var kind:String}
struct SourceTestDebugPlan:Codable {
    var version:Int;var test_debug_protocol:Int;var available:Bool;var reason:String?;var workspace_id:String;var input_path:String;var input_hash:String;var analysis_fingerprint:String;var candidate_id:String;var candidate_name:String;var selection:SourceTestDebugSelection;var source_uri:String;var source_hash:String;var language:String;var framework:String;var name:String;var notices:[String]
    func matches(_ before:TestSourceSnapshot,_ facts:SourceTestScan,_ candidate:SourceTestCandidate,_ choice:TestSourceSelection)->Bool {
        version==1 && test_debug_protocol==1 && exactUTF8(workspace_id,facts.workspace_id) && exactUTF8(input_path,facts.input_path) && exactUTF8(input_hash,facts.input_hash) && exactUTF8(input_hash,facts.source_hashes.first(where:{exactUTF8($0.key,input_path)})?.value) && exactUTF8(source_hash,input_hash) && exactUTF8(input_hash,nativeSourceHash(before.request.source)) && exactUTF8(analysis_fingerprint,facts.fingerprint) && exactUTF8(candidate_id,candidate.id) && exactUTF8(candidate_name,candidate.name) && exactUTF8(selection.kind,choice.rawValue) && exactUTF8(source_uri,languageURI(input_path)) && exactUTF8(language,candidate.language)
    }
    func admits(_ binding:SourceTestDebugBinding,_ facts:SourceTestScan)->Bool {
        binding.version==1 && exactUTF8(binding.workspace_id,workspace_id) && exactUTF8(binding.input_path,input_path) && exactUTF8(binding.input_hash,input_hash) && exactUTF8(binding.analysis_fingerprint,analysis_fingerprint) && exactUTF8(binding.candidate_id,candidate_id) && exactUTF8(binding.candidate_name,candidate_name) && exactUTF8(binding.selection.kind,selection.kind) && exactUTF8(binding.language,language) && exactUTF8(binding.framework,framework) && exactSourceValues(binding.checked_sources)==exactSourceValues(facts.source_hashes)
    }
}
