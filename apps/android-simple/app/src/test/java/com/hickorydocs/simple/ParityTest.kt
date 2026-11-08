// Contract: .hickory-source/frontends/contract.md
package com.hickorydocs.simple

import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Test

class ParityTest {
    @Test fun sharedLayoutScenarios() {
        val fixture = javaClass.classLoader!!.getResource("layout-cases.json")!!.readText()
        val decoded = JSONObject(fixture)
        val points = decoded.getJSONArray("edgeCases")
        for (i in 0 until points.length()) {
            val point = points.getJSONObject(i)
            assertEquals(if (point.isNull("edge")) null else point.getString("edge"), dropEdge(point.getDouble("x"), point.getDouble("y")))
        }
        val scenarios = decoded.getJSONArray("cases")
        for (index in 0 until scenarios.length()) {
            val scenario = scenarios.getJSONObject(index); var layout = PaneLayout()
            val steps = scenario.getJSONArray("steps")
            for (stepIndex in 0 until steps.length()) {
                val step = steps.getJSONObject(stepIndex)
                val group = if (step.has("target")) step.getString("target") else if (step.has("group")) layout.groups()[step.getInt("group")].id else ""
                layout = when (step.getString("op")) {
                    "open" -> layout.activate(step.getString("tab"), group)
                    "move" -> layout.move(step.getString("tab"), group, step.optString("before").takeIf { it.isNotEmpty() })
                    "dock" -> layout.dock(step.getString("tab"), group, step.getString("edge"), step.getString("id"))
                    "remove" -> layout.remove(step.getString("tab"))
                    "split" -> layout.split(group, step.getString("axis"), step.getString("id"))
                    else -> error("Unknown shared operation")
                }
            }
            val expected = scenario.getJSONArray("expected")
            val actual = layout.groups().map { it.tabs }
            val wanted = (0 until expected.length()).map { group -> val tabs = expected.getJSONArray(group); (0 until tabs.length()).map(tabs::getString) }
            assertEquals(scenario.getString("name"), wanted, actual)
            fun axes(id: String): List<String> {
                val node = layout.nodes.getValue(id)
                return if (node.axis == null) emptyList() else listOf(node.axis) + axes(node.first!!) + axes(node.second!!)
            }
            if (scenario.has("axes")) {
                val expectedAxes = scenario.getJSONArray("axes")
                assertEquals((0 until expectedAxes.length()).map(expectedAxes::getString), axes(layout.rootID))
            }
            layout.groups().forEach { assertTrue(it.active == null || it.active in it.tabs) }
        }
    }
    @Test fun documentAndPlainFileReadsUseTheirOwningEndpoints() = kotlinx.coroutines.runBlocking {
        for (managed in listOf(true, false)) {
            val server = java.net.ServerSocket(0, 1, java.net.InetAddress.getByName("127.0.0.1"))
            server.soTimeout = 5000
            val requested = java.util.concurrent.CompletableFuture<String>()
            val worker = Thread {
                try {
                    server.use { listener -> listener.accept().use { socket ->
                        socket.soTimeout = 5000
                        val reader = socket.getInputStream().bufferedReader()
                        val first = reader.readLine()
                        while (!reader.readLine().isNullOrEmpty()) { }
                        val json = if (managed) """{"source":"# Readable source"}""" else """{"content":"Plain text","hash":"plain-hash"}"""
                        val bytes = json.toByteArray(Charsets.UTF_8)
                        socket.getOutputStream().apply {
                            write("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: ${bytes.size}\r\nConnection: close\r\n\r\n".toByteArray())
                            write(bytes); flush()
                        }
                        requested.complete(first)
                    } }
                } catch (failure: Throwable) { requested.completeExceptionally(failure) }
            }.apply { isDaemon = true; start() }
            try {
                val file = FileNode("notes", if (managed) "notes.md" else "notes.txt", false, emptyList(), if (managed) "managed" else null, null)
                val text = Engine("http://127.0.0.1:${server.localPort}").file(file)
                assertEquals(if (managed) "# Readable source" else "Plain text", text.content)
                assertEquals(if (managed) "" else "plain-hash", text.hash)
                assertEquals(if (managed) "GET /api/docs/managed HTTP/1.1" else "GET /api/file?path=notes.txt HTTP/1.1", requested.get(5, java.util.concurrent.TimeUnit.SECONDS))
            } finally { server.close(); worker.join(5000) }
        }
    }
    @Test fun invalidMoveRetainsDocument() {
        val layout = PaneLayout().activate("notes.md", "document-group")
        assertEquals(layout, layout.move("notes.md", "missing"))
    }
}
class NativeDebugTest {
    private fun file() = FileNode("main.py", "main.py", false, emptyList(), null, null)
    @Test fun nativeDebugSessionAndPendingStop() = kotlinx.coroutines.runBlocking {
        val listener = com.sun.net.httpserver.HttpServer.create(java.net.InetSocketAddress("127.0.0.1", 0), 0)
        var captured = JSONObject()
        listener.createContext("/api/launch-profiles") { exchange ->
            captured = JSONObject(exchange.requestBody.bufferedReader().readText())
            val bytes = """{"profiles":[{"id":"python","name":"Python","can_debug":true,"program":"main.py"}],"fingerprint":"current"}""".toByteArray()
            exchange.sendResponseHeaders(200, bytes.size.toLong()); exchange.responseBody.use { it.write(bytes) }
        }
        listener.createContext("/api/docs/managed/render") { exchange ->
            val bytes = """{"blocks":[{"kind":"file","path":"main.py","span":[0,1]}]}""".toByteArray()
            exchange.sendResponseHeaders(200, bytes.size.toLong()); exchange.responseBody.use { it.write(bytes) }
        }; listener.start()
        val debug = NativeDebugger(Engine("http://127.0.0.1:${listener.address.port}"), this)
        try {
            val buffer = DocumentBuffer(file(), FileText("value = 1\nprint(value)\n", "hash"))
            val requests = mutableListOf<JSONObject>(); debug.emitForTest = { requests.add(it) }
            debug.prepare(buffer); assertEquals(buffer.text, captured.getString("expected_source"))
            debug.toggle("main.py", 0, "value > 0"); debug.launch(debug.profiles.first())
            debug.receive(JSONObject("""{"event":"started","doc":"hick:///other.py","session":"wrong"}""")); assertNull(debug.sessionID)
            debug.receive(JSONObject("""{"event":"started","doc":"hick:///main.py","session":"ours","breakpoints":[{"line":0,"moved_to":1,"state":"bound"}]}"""))
            assertEquals("value > 0", debug.breakpoints["main.py"]!!.first().getString("condition"))
            assertEquals(1, debug.breakpoints["main.py"]!!.first().getInt("line"))
            debug.receive(JSONObject("""{"event":"stopped","session":"old","line":0}""")); assertEquals("running", debug.status)
            var revealed = ""; debug.onLocation = { path, line -> revealed = "$path:$line" }
            debug.receive(JSONObject("""{"event":"stopped","session":"ours","line":null,"frames":[{"id":7,"name":"helper","source":"helper.py","source_line":2}],"variables":[{"name":"value","value":"1"}]}"""))
            assertEquals("helper.py:2", revealed); assertEquals(2, debug.line("helper.py"))
            val helper = DocumentBuffer(FileNode("helper.py", "helper.py", false, emptyList(), null, null), FileText("before", "base"))
            debug.rememberSource(helper); assertTrue(debug.sourceMatches(helper))
            helper.state.edit { replace(0, length, "saved replacement") }; helper.base = helper.text
            assertFalse(debug.sourceMatches(helper))
            debug.evaluate("value", true); assertEquals(7, requests.last().getInt("frame"))
            debug.step("over"); assertEquals("over", requests.last().getString("how")); assertTrue(debug.variables.isEmpty())
            debug.receive(JSONObject("""{"event":"value","session":"ours","expression":"value","value":"old"}""")); assertTrue(debug.values.isEmpty())
            debug.close(); debug.prepare(buffer); debug.launch(debug.profiles.first()); requests.clear(); debug.stop()
            assertEquals("stopping", debug.status); assertTrue(requests.isEmpty())
            debug.receive(JSONObject("""{"event":"started","doc":"hick:///main.py","session":"late","breakpoints":[]}"""))
            assertEquals("stop", requests.last().getString("op")); assertEquals("late", requests.last().getString("session"))
            buffer.state.edit { append("edited") }; debug.close(); debug.prepare(buffer)
            assertTrue(debug.message.contains("Save this file")); assertFalse(debug.sourceMatches(buffer))
            val managed = DocumentBuffer(FileNode("source.md", "source.md", false, emptyList(), "managed", null), FileText("<h:file path=\"main.py\">print(1)</h:file>", "base"))
            debug.prepare(managed)
            assertEquals("main.py", captured.getString("program")); assertEquals(managed.text, captured.getString("expected_document")); assertFalse(captured.has("expected_source"))
        } finally { debug.close(); listener.stop(0) }
    }
    @Test fun liveNativeDebugChannel() = kotlinx.coroutines.runBlocking {
        val origin = System.getenv("HICKORY_NATIVE_DEBUG_ORIGIN")
        org.junit.Assume.assumeNotNull(origin)
        val engine = Engine(origin!!); val file = FileNode("debug.py", "debug.py", false, emptyList(), null, null)
        val loaded = engine.file(file); val buffer = DocumentBuffer(file, loaded); val debug = NativeDebugger(engine, this)
        try {
            debug.prepare(buffer); assertTrue(debug.message, debug.profiles.isNotEmpty()); debug.toggle(file.path, 1); debug.launch(debug.profiles.first())
            suspend fun paused() { repeat(200) { if (debug.status == "paused" || debug.status == "failed") return; kotlinx.coroutines.delay(50) } }
            paused(); assertEquals(debug.message, "paused", debug.status); debug.step("over"); paused()
            assertEquals(debug.message, "paused", debug.status); assertTrue(debug.variables.isNotEmpty())
            debug.evaluate("value", true); repeat(100) { if (debug.values["value"] == null) kotlinx.coroutines.delay(50) }
            assertNotNull(debug.values["value"]); assertEquals(loaded.content, buffer.text); assertFalse(buffer.dirty)
            debug.stop(); repeat(100) { if (debug.active) kotlinx.coroutines.delay(50) }; assertFalse(debug.message, debug.active)
        } finally { debug.close() }
    }
}
