package com.hickorydocs.simple
import org.json.JSONObject

data class CoverageProjectedLine(val line:Int,val state:String,val count:Long?,val origins:List<String>)
fun coverageProjection(record:JSONObject):Map<String,List<CoverageProjectedLine>> {
    if(record.opt("source_changed_during_run")!=false)return emptyMap()
    data class Aggregate(val hit:Boolean,val miss:Boolean,val count:Long?,val origins:List<String>)
    val hashes=record.optJSONObject("identity")?.optJSONObject("source_hashes") ?: return emptyMap()
    val owners=mutableMapOf<String,MutableMap<Int,Aggregate>>()
    fun add(path:String,line:Int,countText:String,available:Boolean,origin:String){
        if(!hashes.optString(path).matches(Regex("[0-9a-fA-F]{64}"))||line<1||countText.isEmpty()||!countText.all {it in '0'..'9'})return
        val hit=countText.any {it!='0'};val parsed=countText.toLongOrNull();val known=parsed?.takeIf {available&&it<=9_007_199_254_740_991}
        val rows=owners.getOrPut(path){mutableMapOf()};val old=rows[line-1]
        rows[line-1]=Aggregate(old?.hit==true||hit,old?.miss==true||!hit,if(old==null)known else if(old.count==null||known==null)null else minOf(old.count,known),old?.origins.orEmpty()+origin)
    }
    val files=record.optJSONObject("evidence")?.optJSONObject("report")?.optJSONObject("files") ?: return emptyMap()
    val mappings=record.optJSONObject("mappings")
    for(path in files.keys()){
        val file=files.getJSONObject(path);val lines=file.getJSONObject("lines")
        for(value in lines.keys()){
            val line=value.toIntOrNull() ?: continue;val count=lines.get(value).toString();val available=file.optBoolean("line_counts");val origin="$path:$value"
            add(path,line,count,available,origin)
            val mapping=mappings?.optJSONObject(path)?.optJSONObject(value)
            if(mapping!=null&&mapping.optString("path")!=path)add(mapping.getString("path"),mapping.optInt("line"),count,available,origin)
        }
    }
    return owners.mapValues {(_,rows)->rows.keys.sorted().map {line->val value=rows.getValue(line);CoverageProjectedLine(line,if(value.hit&&value.miss)"mixed" else if(value.hit)"covered" else "uncovered",value.count,value.origins.sorted())}}
}
