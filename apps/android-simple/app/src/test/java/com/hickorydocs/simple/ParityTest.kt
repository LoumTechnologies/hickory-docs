// Contract: .hickory-source/frontends/contract.md
package com.hickorydocs.simple

import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Test

class ParityTest {
    @Test fun sharedLayoutScenarios() {
        val fixture = javaClass.classLoader!!.getResource("layout-cases.json")!!.readText()
        val scenarios = JSONObject(fixture).getJSONArray("cases")
        for (index in 0 until scenarios.length()) {
            val scenario = scenarios.getJSONObject(index); var layout = PaneLayout()
            val steps = scenario.getJSONArray("steps")
            for (stepIndex in 0 until steps.length()) {
                val step = steps.getJSONObject(stepIndex)
                val group = if (step.has("group")) layout.groups()[step.getInt("group")].id else ""
                layout = when (step.getString("op")) {
                    "open" -> layout.activate(step.getString("tab"), group)
                    "move" -> layout.move(step.getString("tab"), group)
                    "remove" -> layout.remove(step.getString("tab"))
                    "split" -> layout.split(group, step.getString("axis"), step.getString("id"))
                    else -> error("Unknown shared operation")
                }
            }
            val expected = scenario.getJSONArray("expected")
            val actual = layout.groups().map { it.tabs }
            val wanted = (0 until expected.length()).map { group -> val tabs = expected.getJSONArray(group); (0 until tabs.length()).map(tabs::getString) }
            assertEquals(scenario.getString("name"), wanted, actual)
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
