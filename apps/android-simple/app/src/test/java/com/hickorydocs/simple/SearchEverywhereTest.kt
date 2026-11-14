package com.hickorydocs.simple
import org.junit.Assert.*
import org.junit.Test
import org.json.JSONArray
import org.json.JSONObject
class SearchEverywhereTest {
    @Test fun sourceKnowledgeAndFallbacks() {
        val file=FileNode("Widget.py","src/Widget.py",false,emptyList(),null,null);val generated=FileNode("out.py","out.py",false,emptyList(),null,"owner");var source="source"
        val values=JSONArray();listOf(5,6,99).forEachIndexed {i,kind->values.put(JSONObject().put("name",listOf("Widget","render","unknown")[i]).put("kind",kind).put("range",JSONObject().put("start",JSONObject().put("line",0).put("character",i)).put("end",JSONObject().put("line",0).put("character",i))))}
        val locations=semanticLocations(values,"hick:///src/Widget.py")+SemanticLocation("hick:///foreign.py",0,0,"foreign",5)
        val knowledge=listOf(AnalysisKnowledge(file.path,"source",locations,"ready",""));val commands=listOf(WorkspaceCommand("symbol.rename","Rename symbol","Edit","Open source first.",{}))
        fun targets()=everywhereTargets(listOf(file,file,generated),knowledge,{source},commands)
        assertEquals(6,targets().size);assertEquals("Widget",searchEverywhere(targets(),"wdgt","type").first.first().label)
        assertEquals(1,searchEverywhere(targets(),"render","member").second);assertEquals(1,searchEverywhere(targets(),"unknown","symbol").second)
        assertEquals("Open source first.",searchEverywhere(targets(),"rename","action").first.first().reason);assertTrue(searchEverywhere(targets(),"out.py").first.first().detail.contains("read-only"))
        assertEquals(1,searchEverywhere(targets(),"",limit=1).first.size);assertEquals(6,searchEverywhere(targets(),"",limit=1).second)
        source="draft";assertEquals(3,targets().size);assertEquals(0,searchEverywhere(targets(),"","type").second)
        assertEquals(1,searchEverywhere(listOf(SearchTarget("emoji","file","😀x😀","")),"😀😀").second)
    }
}
