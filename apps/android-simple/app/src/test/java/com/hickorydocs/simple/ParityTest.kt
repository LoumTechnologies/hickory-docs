// Contract: .hickory-source/frontends/contract.md
package com.hickorydocs.simple

import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Test

class ParityTest {
    @Test fun detectedIndentationAndPartialTabStops() {
        // Contract: .hickory-source/frontends/contract.md
        assertEquals("    ", simpleIndent("root\n    one\n        two\n"))
        assertEquals("  ", simpleIndent("root\n  one\n    two\n"))
        assertEquals("\t", simpleIndent("root\n\tone\n\t\ttwo\n"))
        assertEquals("    ", simpleIndent("\n   \n"))
        val source = "root\n    child\n "
        assertEquals("   ", simpleTabText(source, source.length))
        assertEquals("    ", simpleTabText(source + "   ", source.length + 3))
        assertEquals("    ", simpleTabText("\n", 0))
        val edits = simpleTabEdits("one\ntwo", 0, 7)
        assertEquals(listOf("    ", "    "), edits.map { it.text })
        assertTrue(edits.all { it.from == it.to })
        val tabs = editorReplacements(emptyList(), emptyList(), "a\tb")
        assertEquals("a   b", editorDisplay("a\tb", tabs))
        assertEquals(2, editorSourceOffset(4, tabs))
    }

    @Test fun tableArrowEntryKeepsCollapsedCaret() {
        // Contract: .hickory-source/frontends/contract.md
        assertEquals(androidx.compose.ui.text.TextRange(2), gridCaretSelection("wide", 10f, 0) { 2 })
        assertEquals(androidx.compose.ui.text.TextRange(1), gridCaretSelection("x", 100f, 0) { 99 })
        assertEquals(androidx.compose.ui.text.TextRange(0), gridCaretSelection("", 100f, 0) { 99 })
        assertEquals(androidx.compose.ui.text.TextRange(4), gridCaretSelection("left", 0f, -1) { error("Not a vertical move") })
        assertEquals(androidx.compose.ui.text.TextRange(0), gridCaretSelection("right", 0f, 1) { error("Not a vertical move") })
    }
    @Test fun diffPresentationKeepsPatchTextLiteral() {
        // Contract: .hickory-source/frontends/contract.md
        val source = "```diff\n-old **text**\n+new **text**\n```"
        val snapshot = computeInlineMarkup(source, "workbook.md")
        assertEquals(listOf("diff-remove", "diff-add"), snapshot.marks.map { it.kind })
        assertEquals(source, snapshot.source)
        assertTrue(snapshot.debug.code.isEmpty())
    }
    @Test fun csvGridsAndContextualCodeLines() {
        // Contract: .hickory-source/frontends/contract.md
        val csv = "Name,Note\r\nCafé,\"two\nlines\"\r\nlast,"
        val grid = inlineTables(computeInlineMarkup(csv, "stock.csv"))
        assertEquals(1, grid.size)
        assertEquals(listOf(listOf("Name", "Note"), listOf("Café", "two\nlines"), listOf("last", "")), grid.single().rows.map { it.map { it.value } })
        val source = "| A | B |\n| --- | --- |\n| 1 | 2 |\n\n<h:table path=\"stock.csv\">\nName,Note\nPen,Blue\n</h:table>\n<h:file path=\"coupon.py\" language=\"python\">\ndef coupon():\n    return 1\n</h:file>\nProse"
        assertEquals(2, inlineTables(computeInlineMarkup(source, "workbook.md")).size)
        assertNull(inlineDebugPath(source, "workbook.md", 12))
        assertNull(inlineDebugPath(source, "workbook.md", 0))
        assertEquals("coupon.py", inlineDebugPath(source, "workbook.md", 8, true))
        assertEquals("coupon.py", inlineDebugPath(source, "workbook.md", 10))
        assertNull(inlineDebugPath("```xml\n<h:file path=\"fake.py\">\nprint(1)\n</h:file>\n```", "workbook.md", 2))
    }
    @Test fun markdownTableProjectionRetainsSourceAndFollowingLineOffsets() {
        val source = "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\">\r\nBefore 😀\r\n| A | B |\r\n| --- | ---: |\r\n| café | a\\|b |\r\nAfter\r\n```md\r\n| X | Y |\r\n| --- | --- |\r\n```\r\n</h:doc>"
        val tables = inlineTables(computeInlineMarkup(source, "workbook.md"))
        assertEquals(1, tables.size)
        val table = tables.single()
        assertEquals(listOf(listOf("A", "B"), listOf("café", "a|b")), table.rows.map { row -> row.map { it.value } })
        assertEquals("| A | B |\r\n| --- | ---: |\r\n| café | a\\|b |", source.substring(table.from, table.to))
        val replacements = editorReplacements(emptyList(), tables)
        val display = editorDisplay(source, replacements)
        assertFalse(display.contains("| A | B |"))
        assertEquals(source.indexOf("After"), editorSourceOffset(display.indexOf("After"), replacements))
        assertEquals(display.indexOf("After"), editorDisplayOffset(source.indexOf("After"), replacements))
        assertEquals(source, editorDisplay(source, emptyList()))
        assertEquals("AA", tableColumnName(26))
    }

    @Test fun languageCacheMapsUnchangedUtf16Regions() {
        val original = "def f():\n    value = 1\n"
        val spans = listOf(NativeLanguageSpan(0, 3, "keyword"), NativeLanguageSpan(13, 18, "variable"))
        val inserted = "# 😀\n" + original
        val shifted = rebaseNativeTokens(spans, original, inserted)
        assertEquals(listOf(5, 18), shifted.map { it.from })
        assertEquals(spans, rebaseNativeTokens(shifted, inserted, original))
        assertEquals(listOf(spans[0]), rebaseNativeTokens(spans, original, "def f():\n  other = 1\n"))
        assertEquals(NativeLanguageSpan(13, 19, "variable"), rebaseNativeTokens(spans, original, "def f():\n    valxue = 1\n").last())
        assertTrue(rebaseNativeTokens(spans, original, "").isEmpty())
        assertEquals(spans, rebaseNativeTokens(spans, original, original))
    }

    @OptIn(kotlinx.coroutines.DelicateCoroutinesApi::class, kotlinx.coroutines.ExperimentalCoroutinesApi::class)
    @Test fun realNativeLanguageChannel() {
        val origin = System.getenv("HICKORY_NATIVE_LSP_ORIGIN")
        org.junit.Assume.assumeTrue("Set HICKORY_NATIVE_LSP_ORIGIN for the real backend witness", !origin.isNullOrEmpty())
        val dispatcher = kotlinx.coroutines.newSingleThreadContext("native-language-witness")
        try { kotlinx.coroutines.runBlocking(dispatcher) {
            val scope = kotlinx.coroutines.CoroutineScope(dispatcher + kotlinx.coroutines.SupervisorJob())
            val file = FileNode("native-language-check.py", "native-language-check.py", false, emptyList(), null, null)
            val buffer = DocumentBuffer(file, FileText("counter = 1\nprint(counter)\n", "base"))
            val service = NativeLanguage(Engine(origin!!), buffer, scope); buffer.language = service
            try {
                service.start()
                for (attempt in 0 until 100) { if (service.tokens.isNotEmpty()) break; androidx.compose.runtime.snapshots.Snapshot.sendApplyNotifications(); kotlinx.coroutines.delay(100) }
                assertTrue(service.status, service.tokens.isNotEmpty())
                buffer.state.edit { selection = androidx.compose.ui.text.TextRange(2) }
                service.inspect(); assertTrue("Real Python hover should describe counter", service.hover.isNotEmpty())
                buffer.state.edit { replace(0, length, "counter = 1\ncou"); selection = androidx.compose.ui.text.TextRange(length) }
                androidx.compose.runtime.snapshots.Snapshot.sendApplyNotifications()
                service.complete(); assertTrue(service.status, service.suggestions.any { it.label == "counter" })
            } finally { service.close(); scope.coroutineContext[kotlinx.coroutines.Job]?.cancel() }
        } } finally { dispatcher.close() }
    }
    @OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
    @Test fun nativeSourceEditsAreOneUndoAndRejectStaleAndReadonly() {
        val source = "name=12\nname=34\n"
        val file = FileNode("search.txt", "search.txt", false, emptyList(), null, null)
        val buffer = DocumentBuffer(file, FileText(source, "base"))
        val edits = nativeSearch(NativeSearchRequest(source, "name=(\\d+)", "$1", true, true)).edits
        assertTrue(buffer.applySourceEdits(edits, source)); assertEquals("12\n34\n", buffer.text)
        assertFalse(buffer.applySourceEdits(edits, source))
        buffer.state.undoState.undo(); assertEquals(source, buffer.text)
        val generated = DocumentBuffer(file.copy(generatedBy = "owner"), FileText(source, "base"))
        assertFalse(generated.applySourceEdits(edits, source)); assertEquals(source, generated.text)
        assertTrue(generated.selectSourceRange(8, 12, source)); assertEquals(8, generated.state.selection.min)
    }
    @Test fun inlineMarkupKeepsSourceCoordinatesAndLiteralPayloads() {
        val source = "# Heading\r\n**bold** and `**literal**`\r\n| Name | Value |\r\n| --- | --- |\r\n| 😀 | a\\|b |\r\n```md\r\n# raw\r\n```\r\n<h:math>\r\nx^2\r\n</h:math>\r\n"
        val result = computeInlineMarkup(source, "source.md")
        assertEquals(4, inlineLineAt(source.indexOf("😀"), result.lineStarts))
        assertEquals(source, result.source); assertEquals(1, result.marks.count { it.kind == "heading" }); assertEquals(1, result.marks.count { it.kind == "strong" })
        assertEquals(listOf("Name", "Value", "😀", "a|b"), result.cells.map { it.value })
        assertEquals("😀", source.substring(result.cells[2].from, result.cells[2].to))
        assertFalse(result.cells.any { it.value.contains('\r') }); assertEquals(1, result.equations.size)
        assertEquals("\r\nx^2\r\n", result.equations[0].body)
        assertTrue(computeInlineMarkup("$5 and $6", "money.md").equations.isEmpty())
        assertTrue(computeInlineMarkup(source, "code.py").marks.isEmpty()); assertEquals("a\\|b", inlineCellText("a|b"))
    }
    @OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
    @Test fun inlineCSVRetainsQuotedMultilineRanges() {
        val body = "\nName,Value\r\n\"a,b\",\"two\nlines\"\r\nlast,"
        val cells = inlineCSVCells(body, 0, 10, ",")
        assertEquals(listOf("Name", "Value", "a,b", "two\nlines", "last", ""), cells.map { it.value })
        assertEquals(2, inlineCells("| ``a|`b`` | c |", 0, 0).size)
        assertEquals(4, cells.last().line)
        assertEquals("\"a,\"\"b\"", inlineCellText("a,\"b", "csv"))
        assertTrue(inlineCSVCells("\"unterminated", 0, 0, ",").isEmpty())
    }
    @OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
    @Test fun inlineCellEditsAreSourceGuardedAndUndoable() {
        val source = "# Heading\n\n| A | B |\n| --- | --- |\n| x | y |\n"
        val buffer = DocumentBuffer(FileNode("styled.md", "styled.md", false, emptyList(), null, null), FileText(source, "base"))
        val cell = computeInlineMarkup(source, "styled.md").cells.last()
        assertTrue(buffer.applySourceEdits(listOf(NativeSourceEdit(cell.from, cell.to, inlineCellText("a|b"))), source))
        assertEquals(source.replace("| x | y |", "| x | a\\|b |"), buffer.text)
        assertFalse(buffer.applySourceEdits(listOf(NativeSourceEdit(cell.from, cell.to, "stale")), source))
        buffer.state.undoState.undo(); assertEquals(source, buffer.text)
    }
    @Test fun languageDiagnosticPositionsRequireCurrentVersion() {
        val item = JSONObject().put("message", "error").put("range", JSONObject().put("start", JSONObject().put("line", 0).put("character", 3)).put("end", JSONObject().put("line", 0).put("character", 4)))
        val params = JSONObject().put("diagnostics", org.json.JSONArray().put(item))
        assertNull(decodeNativeDiagnostics(params, "😀 x", 2).first().from)
        params.put("version", 1); assertNull(decodeNativeDiagnostics(params, "😀 x", 2).first().from)
        params.put("version", 2); assertEquals(3, decodeNativeDiagnostics(params, "😀 x", 2).first().from)
        assertEquals("hick:///a%20space.py", languageURI("a space.py"))
        assertNull(languageOffset("😀 x", JSONObject().put("line", 0).put("character", 5)))
    }
    @OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
    @Test fun multipleSelectionsReplaceAndUndoAtomically() {
        val source = "one one"
        val buffer = DocumentBuffer(FileNode("multi.txt", "multi.txt", false, emptyList(), null, null), FileText(source, "base"))
        assertTrue(buffer.selectMultiple(listOf(NativeSourceEdit(0, 3, ""), NativeSourceEdit(4, 7, "")), source))
        assertTrue(buffer.replaceMultiple("two")); assertEquals("two two", buffer.text)
        assertEquals(listOf(3, 7), buffer.multiRanges.map { it.from })
        buffer.state.undoState.undo(); assertEquals(source, buffer.text)
        assertFalse(buffer.selectMultiple(listOf(NativeSourceEdit(0, 3, ""), NativeSourceEdit(1, 4, "")), source))
    }
    @Test fun nativeSearchUnicodeCapturesAndRefusals() {
        val request = NativeSearchRequest("😀 NAME=12\nname=34\n", "name=(\\d+)", "$1 $$ $&", false, true)
        val result = nativeSearch(request)
        assertEquals(listOf(3, 11), result.edits.map { it.from })
        assertEquals(listOf("12 $ NAME=12", "34 $ name=34"), result.edits.map { it.text })
        assertTrue(nativeSearch(request.copy(query = "[")).error.isNotEmpty())
        assertTrue(nativeSearch(request.copy(query = "^")).edits.isEmpty())
        assertTrue(nativeSearch(request.copy(regex = false, query = "\\n")).edits.isEmpty())
        val many = nativeSearch(request.copy(source = "a".repeat(10_001), query = "a", regex = false))
        assertTrue(many.error.isNotEmpty()); assertTrue(many.edits.isEmpty())
        assertFalse(validSourceEdits(listOf(NativeSourceEdit(4, 20, "")), 5))
    }
    @Test fun foldProjectionPreservesSourceAndOffsets() {
        val source = "😀 header\n\tchild\n        nested\n\nnext\n"
        val ranges = simpleFoldRanges(source)
        assertEquals(listOf(0, 1), ranges.map { it.line })
        assertEquals(listOf("\n\tchild\n        nested", "\n        nested"), ranges.map { source.substring(it.from, it.to) })
        val display = foldDisplay(source, ranges)
        assertEquals("😀 header …\n\nnext\n", display)
        assertEquals(source.indexOf("next"), foldSourceOffset(display.indexOf("next"), ranges))
        assertEquals(source, foldDisplay(source, emptyList()))
        assertTrue(simpleFoldRanges("one\ntwo\n").isEmpty())
    }
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
