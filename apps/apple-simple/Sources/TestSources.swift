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
func validateSourceTestJSON(_ data:Data)throws {try validateNativeJSON(data,canonicalVersionKeys:["version","test_debug_protocol","wait_ms","template_protocol","history_protocol","behavior_protocol","resolution_protocol"])}
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
