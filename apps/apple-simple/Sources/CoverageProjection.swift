import Foundation
struct CoverageProjectionRecord:Decodable {
    struct Identity:Decodable {var source_hashes:[String:String]}
    struct File:Decodable {var line_counts:Bool;var lines:[String:UInt64]}
    struct Report:Decodable {var files:[String:File]}
    struct Evidence:Decodable {var report:Report}
    struct Mapping:Decodable {var path:String;var line:Int}
    var id:String;var source_changed_during_run:Bool?;var identity:Identity;var evidence:Evidence?;var mappings:[String:[String:Mapping]]?
}
struct CoverageProjectedLine:Decodable,Equatable {
    var line:Int;var state:String;var count:UInt64?;var origins:[String]
}
func coverageProjection(_ record:CoverageProjectionRecord)->[String:[CoverageProjectedLine]] {
    guard record.source_changed_during_run==false else {return [:]}
    struct Aggregate {var hit:Bool;var miss:Bool;var count:UInt64?;var origins:[String]}
    var owners:[String:[Int:Aggregate]]=[:]
    func add(_ path:String,_ line:Int,_ count:UInt64,_ available:Bool,_ origin:String){
        guard let hash=record.identity.source_hashes[path],hash.count==64,hash.allSatisfy({$0.isASCII && $0.isHexDigit}),line>0,line<=Int(Int32.max) else {return}
        let known:UInt64?=available && count<=9_007_199_254_740_991 ? count:nil
        let old=owners[path]?[line-1]
        let minimum:UInt64?
        if let old {if let left=old.count,let right=known {minimum=min(left,right)}else{minimum=nil}}
        else {minimum=known}
        owners[path,default:[:]][line-1]=Aggregate(hit:(old?.hit ?? false)||count>0,miss:(old?.miss ?? false)||count==0,count:minimum,origins:(old?.origins ?? [])+[origin])
    }
    for (path,file) in record.evidence?.report.files ?? [:] {
        for (value,count) in file.lines {
            guard let line=Int(value) else {continue};let origin="\(path):\(value)";add(path,line,count,file.line_counts,origin)
            if let mapping=record.mappings?[path]?[value],mapping.path != path {add(mapping.path,mapping.line,count,file.line_counts,origin)}
        }
    }
    return owners.mapValues {rows in rows.keys.sorted().map {line in let value=rows[line]!;return CoverageProjectedLine(line:line,state:value.hit && value.miss ? "mixed":value.hit ? "covered":"uncovered",count:value.count,origins:value.origins.sorted())}}
}
