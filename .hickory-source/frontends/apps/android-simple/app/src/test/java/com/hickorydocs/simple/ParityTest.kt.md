<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Execute the same layout scenarios on Android

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

The Gradle test resource root points at the shared JSON fixture. These witnesses check pane commands without an emulator; gesture and text-input behavior still require device tests.

Opening witnesses serve real loopback HTTP responses and assert the requested endpoint as well as the returned source/hash. This catches a managed document accidentally routed to the intentionally refusing plain-file API.

<ui:copy id="simple-de002213e5-0">
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
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/ParityTest.kt"><ui:paste select="#simple-de002213e5-0"/></ui:file>

</ui:doc>
