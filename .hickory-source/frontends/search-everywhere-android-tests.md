<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Keep search useful while source knowledge changes

For maintainers changing workspace services independently of the editor renderer.

Exercise a mixed workspace rather than the ranking implementation. Duplicate file listings collapse; a generated file stays discoverable and read-only; a foreign-source symbol is rejected. Only the current source contributes type/member/general-symbol targets. Disabled actions remain visible. After a draft changes, file and action search survives while every old symbol disappears. A capped view retains the full match count, and fuzzy matching handles an emoji scalar without splitting its surrogate pair.

<ui:copy id="simple-search-everywhere-android-tests">
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
</ui:copy>

<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/SearchEverywhereTest.kt"><ui:paste select="#simple-search-everywhere-android-tests"/></ui:file>
</ui:doc>
