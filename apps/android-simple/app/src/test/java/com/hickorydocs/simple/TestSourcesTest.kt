package com.hickorydocs.simple
import org.junit.Assert.*
import org.junit.Test
class TestSourcesTest {
    private fun fixture():Pair<TestSourceSnapshot,SourceTestScan> {
        val before=TestSourceSnapshot("w1",TestSourceRequest("cases.md","draft","project.md",mapOf("helper.md" to "helper","unused.md" to "old")))
        val point=TestSourcePoint("test.py",1,4,"target");val candidate=SourceTestCandidate("case","python","file","test_case","test_case","syntax",null,point)
        return before to SourceTestScan(1,"cases.md","scan",mapOf("cases.md" to "draft-hash","helper.md" to "helper-hash"),listOf(candidate),"project",true,emptyList(),emptyList(),emptyList(),listOf("unused.md"))
    }
    @Test fun contributorAndWorkspaceIdentity() {
        val (before,facts)=fixture();assertTrue(testSourceCurrent(before,facts,before))
        assertFalse(testSourceCurrent(before,facts,before.copy(workspace="w2")))
        assertFalse(testSourceCurrent(before,facts,before.copy(request=before.request.copy(source="changed"))))
        assertFalse(testSourceCurrent(before,facts,before.copy(request=before.request.copy(sources=mapOf("helper.md" to "changed")))))
        assertTrue(testSourceCurrent(before,facts,before.copy(request=before.request.copy(sources=mapOf("helper.md" to "helper","unused.md" to "new")))))
        assertFalse(testSourceCurrent(before,facts.copy(version=2),before))
    }
    @Test fun explicitSelectionAndDynamicRefusal() {
        val (before,facts)=fixture();val value=sourceTestRun(before,facts,before,"case",TestSourceSelection.NAME_FILTER)
        assertEquals("saved",value.getJSONObject("input").getString("kind"));assertEquals("scan",value.getString("fingerprint"))
        val dynamic=facts.copy(candidates=listOf(facts.candidates[0].copy(nameFilter=null)))
        assertTrue(runCatching {sourceTestRun(before,dynamic,before,"case",TestSourceSelection.NAME_FILTER)}.isFailure)
        assertEquals("containing_scope",sourceTestRun(before,dynamic,before,"case",TestSourceSelection.CONTAINING_SCOPE).getJSONObject("selection").getString("kind"))
        assertTrue(runCatching {sourceTestRun(before,facts,before,"missing",TestSourceSelection.NAME_FILTER)}.isFailure)
    }
}
