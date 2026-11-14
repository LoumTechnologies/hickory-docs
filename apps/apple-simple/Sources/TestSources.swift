import Foundation
struct TestSourceRequest: Equatable {
    var path:String;var source:String;var projectEntry:String?;var sources:[String:String]=[:]
    var body:[String:Any] {var value:[String:Any]=["path":path,"source":source,"sources":sources];if let projectEntry {value["project_entry"]=projectEntry};return value}
}
struct TestSourceSnapshot:Equatable {var workspace:String;var request:TestSourceRequest}
struct TestSourcePoint:Decodable {var path:String;var line:Int;var character:Int;var hash:String}
struct SourceTestCandidate:Decodable,Identifiable {
    var id:String;var language:String;var run_scope:String;var name:String;var name_filter:String?;var confidence:String;var source:TestSourcePoint?;var target:TestSourcePoint
    var point:TestSourcePoint {source ?? target}
}
struct SourceTestScan:Decodable {
    var version:Int;var input_path:String;var fingerprint:String;var source_hashes:[String:String];var candidates:[SourceTestCandidate]
    var scope:String;var syntax_complete:Bool;var notices:[String];var unsupported_languages:[String];var unclassified_outputs:[String];var unused_sources:[String]
}
func testSourceCurrent(_ before:TestSourceSnapshot,_ facts:SourceTestScan,_ now:TestSourceSnapshot)->Bool {
    let a=before.request,b=now.request
    guard facts.version==1,facts.input_path==a.path,before.workspace==now.workspace,a.path==b.path,a.source==b.source,a.projectEntry==b.projectEntry else {return false}
    return facts.source_hashes.keys.allSatisfy {$0==a.path || a.sources[$0]==b.sources[$0]}
}
enum TestSourceSelection:String {case nameFilter="name_filter",containingScope="containing_scope"}
func sourceTestRun(_ before:TestSourceSnapshot,_ facts:SourceTestScan,_ now:TestSourceSnapshot,id:String,selection:TestSourceSelection)throws->[String:Any] {
    guard testSourceCurrent(before,facts,now) else {throw EngineError(message:"Test source changed. Refresh candidates before running.")}
    guard let candidate=facts.candidates.first(where:{$0.id==id}) else {throw EngineError(message:"Candidate unavailable. Refresh test candidates.")}
    guard selection != .nameFilter || candidate.name_filter != nil else {throw EngineError(message:"This declaration needs runner discovery or an explicit containing-scope run.")}
    return ["version":1,"input":["kind":"saved"],"analysis":before.request.body,"fingerprint":facts.fingerprint,"candidate_id":id,"selection":["kind":selection.rawValue]]
}
