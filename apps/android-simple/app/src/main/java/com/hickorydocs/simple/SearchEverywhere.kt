package com.hickorydocs.simple
import java.util.Locale
data class SearchTarget(val id:String,val kind:String,val label:String,val detail:String,val path:String?=null,val location:SemanticLocation?=null,val source:String?=null,val commandID:String?=null,val reason:String?=null)
fun symbolSearchKind(kind:Int?):String=if(kind in listOf(5,10,11,23,26)) "type" else if(kind in listOf(6,7,8,9,12,13,14,22,25)) "member" else "symbol"
fun everywhereTargets(files:List<FileNode>,knowledge:List<AnalysisKnowledge>,current:(String)->String?,commands:List<WorkspaceCommand>):List<SearchTarget> {
    val targets=mutableListOf<SearchTarget>();val seen=mutableSetOf<String>()
    fun visit(files:List<FileNode>) {for(file in files){if(file.dir){visit(file.children);continue};if(!seen.add(file.path))continue;targets.add(SearchTarget("file:${file.path}","file",file.name,file.path+if(file.generatedBy!=null)" · generated, read-only" else "",path=file.path))}};visit(files)
    for(item in knowledge){val source=item.source;if(item.path !in seen||item.state!="ready"||source==null||current(item.path)!=source)continue
        for(location in item.locations){val name=location.name;if(location.path!=item.path||name.isNullOrEmpty())continue;targets.add(SearchTarget("symbol:${location.uri}:${location.line}:${location.character}",symbolSearchKind(location.kind),name,"${item.path}:${location.line+1}",path=item.path,location=location,source=source))}}
    for(command in commands)targets.add(SearchTarget("action:${command.id}","action",command.label,command.group,commandID=command.id,reason=command.reason))
    return targets.distinctBy {it.id}
}
private fun everywhereScore(value:String,query:String):Int? {
    val text=value.lowercase(Locale.ROOT);if(text==query)return 100;if(text.startsWith(query))return 80;if(query in text)return 60
    val scalars=query.codePoints().toArray();var cursor=0;for(scalar in text.codePoints().toArray()){if(cursor<scalars.size&&scalar==scalars[cursor])cursor++;if(cursor==scalars.size)return 20};return null
}
fun searchEverywhere(targets:List<SearchTarget>,query:String,kind:String="all",limit:Int=200):Pair<List<SearchTarget>,Int> {
    val terms=query.lowercase(Locale.ROOT).trim().split(Regex("\\s+")).filter {it.isNotEmpty()}
    val matches=targets.mapNotNull {target->if(kind!="all"&&target.kind!=kind)return@mapNotNull null;var total=0
        for(term in terms){val a=everywhereScore(target.label,term) ?: -1;val b=everywhereScore(target.detail,term)?.div(2) ?: -1;val best=maxOf(a,b);if(best<0)return@mapNotNull null;total+=best};target to total
    }.sortedWith(compareByDescending<Pair<SearchTarget,Int>> {it.second}.thenBy {it.first.label}.thenBy {it.first.id})
    return matches.take(limit.coerceIn(0,1000)).map {it.first} to matches.size
}
