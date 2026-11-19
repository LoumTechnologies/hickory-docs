package com.hickorydocs.simple
import org.junit.Assert.*
import org.junit.Test
import org.json.JSONObject
import kotlinx.coroutines.async
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
    @Test fun lateTestPointCannotReplaceNewerTabOrReopenedReading() = kotlinx.coroutines.runBlocking {
        val targetSource = "header\nsource test\n"
        val paths = listOf("reopened.py", "newer_tab.py")
        val arrived = paths.associateWith { kotlinx.coroutines.CompletableDeferred<Unit>() }
        val gates = paths.associateWith { java.util.concurrent.CountDownLatch(1) }
        val listener = com.sun.net.httpserver.HttpServer.create(java.net.InetSocketAddress("127.0.0.1", 0), 0)
        listener.createContext("/api/file") { exchange ->
            val path = exchange.requestURI.query.removePrefix("path=")
            arrived.getValue(path).complete(Unit)
            check(gates.getValue(path).await(8, java.util.concurrent.TimeUnit.SECONDS))
            val response = JSONObject().put("content", targetSource).put("hash", "saved").toString().toByteArray()
            exchange.sendResponseHeaders(200, response.size.toLong())
            exchange.responseBody.use { it.write(response) }
        }
        listener.start()
        fun file(path: String) = FileNode(path, path, false, emptyList(), null, null)
        val workspace = Workspace(this)
        workspace.engine = Engine("http://127.0.0.1:${listener.address.port}")
        val draft = DocumentBuffer(file("draft.py"), FileText("existing", "saved"))
        val other = DocumentBuffer(file("other.py"), FileText("other source", "saved"))
        workspace.rootFiles = (paths + draft.file.path + other.file.path).map(::file)
        workspace.buffers[draft.file.path] = draft
        workspace.buffers[other.file.path] = other
        workspace.show(sourceTab(other.file.path))
        workspace.show(sourceTab(draft.file.path))
        val selection = androidx.compose.ui.text.TextRange(2, 5)
        draft.state.edit { append(" dirty draft"); this.selection = selection }
        draft.sourceScroll.scrollTo(11)
        workspace.showTests = true
        val reading = workspace.testSources
        val trail = workspace.navigationTrail
        try {
            for (path in paths) {
                val point = TestSourcePoint(path, 2, 3, debugHash(targetSource))
                val reveal = workspace.testPointNavigation(point)
                val pending = async { reveal() }
                kotlinx.coroutines.withTimeout(4000) { arrived.getValue(path).await() }
                if (path == "reopened.py") {
                    workspace.showTests = false
                    workspace.showTests = true
                } else {
                    workspace.show(sourceTab(other.file.path))
                    workspace.show(sourceTab(draft.file.path))
                }
                val panes = workspace.layout
                val focused = workspace.focused
                gates.getValue(path).countDown()
                pending.await()
                assertSame(panes, workspace.layout)
                assertEquals(focused, workspace.focused)
                assertSame(trail, workspace.navigationTrail)
                assertSame(draft, workspace.currentBuffer)
                assertEquals("existing dirty draft", draft.text)
                assertEquals(selection, draft.state.selection)
                assertEquals(11, draft.sourceScroll.value)
                assertSame(reading, workspace.testSources)
                assertTrue(workspace.showTests)
                assertEquals(targetSource, workspace.buffers.getValue(path).text)
            }
            val target = workspace.buffers.getValue(paths.last())
            workspace.testPointNavigation(TestSourcePoint(paths.last(), 2, 3, debugHash(targetSource)))()
            assertSame(target, workspace.currentBuffer)
            assertEquals(androidx.compose.ui.text.TextRange(10), target.state.selection)
            assertFalse(workspace.showTests)
            assertEquals(selection, draft.state.selection)
        } finally {
            gates.values.forEach { it.countDown() }
            workspace.buffers.values.forEach { it.language?.close() }
            listener.stop(0)
        }
    }
}
