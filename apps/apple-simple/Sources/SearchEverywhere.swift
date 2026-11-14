import Foundation
struct SearchTarget: Identifiable {
    var id:String;var kind:String;var label:String;var detail:String;var path:String?=nil;var location:SemanticLocation?=nil;var source:String?=nil;var commandID:String?=nil;var reason:String?=nil
}
func symbolSearchKind(_ kind:Int?)->String { [5,10,11,23,26].contains(kind ?? 0) ? "type":[6,7,8,9,12,13,14,22,25].contains(kind ?? 0) ? "member":"symbol" }
func everywhereTargets(_ files:[FileNode],knowledge:[AnalysisKnowledge],current:(String)->String?,commands:[WorkspaceCommand])->[SearchTarget] {
    var targets:[SearchTarget]=[],seen=Set<String>()
    func visit(_ files:[FileNode]) {for file in files {
        if file.dir {visit(file.children ?? []);continue};guard seen.insert(file.path).inserted else {continue}
        targets.append(SearchTarget(id:"file:\(file.path)",kind:"file",label:file.name,detail:file.path+(file.generated_by != nil ? " · generated, read-only":""),path:file.path))
    }};visit(files)
    for item in knowledge {
        guard seen.contains(item.path),item.state=="ready",let source=item.source,current(item.path)==source else {continue}
        for location in item.locations {
            guard location.path==item.path,let name=location.name,!name.isEmpty else {continue}
            targets.append(SearchTarget(id:"symbol:\(location.id)",kind:symbolSearchKind(location.kind),label:name,detail:"\(item.path):\(location.line+1)",path:item.path,location:location,source:source))
        }
    }
    for command in commands {targets.append(SearchTarget(id:"action:\(command.id)",kind:"action",label:command.label,detail:command.group,commandID:command.id,reason:command.reason))}
    var ids=Set<String>();return targets.filter {ids.insert($0.id).inserted}
}
private func everywhereScore(_ value:String,_ query:String)->Int? {
    let value=value.lowercased();if value==query {return 100};if value.hasPrefix(query){return 80};if value.contains(query){return 60}
    let scalars=Array(query.unicodeScalars);var cursor=0
    for scalar in value.unicodeScalars {if cursor<scalars.count && scalar==scalars[cursor] {cursor += 1};if cursor==scalars.count {return 20}}
    return nil
}
func searchEverywhere(_ targets:[SearchTarget],query:String,kind:String="all",limit:Int=200)->(items:[SearchTarget],total:Int) {
    let terms=query.lowercased().split(whereSeparator:{$0.isWhitespace}).map(String.init)
    let matches=targets.compactMap {target->(SearchTarget,Int)? in
        guard kind=="all" || kind==target.kind else {return nil};var total=0
        for term in terms {let a=everywhereScore(target.label,term) ?? -1,b=everywhereScore(target.detail,term).map {$0/2} ?? -1,best=max(a,b);if best<0{return nil};total += best}
        return (target,total)
    }.sorted {a,b in a.1 != b.1 ? a.1>b.1:a.0.label != b.0.label ? a.0.label<b.0.label:a.0.id<b.0.id}
    return (Array(matches.prefix(max(0,min(1000,limit)))).map {$0.0},matches.count)
}
