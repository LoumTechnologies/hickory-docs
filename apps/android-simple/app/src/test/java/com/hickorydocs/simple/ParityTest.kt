// Contract: .hickory-source/frontends/contract.md
package com.hickorydocs.simple

import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Test
import kotlinx.coroutines.async
import kotlinx.coroutines.launch
import kotlinx.coroutines.delay
import kotlinx.coroutines.withTimeout

class ParityTest {
    @Test(timeout = 16000)
    fun startupReadIsIndependentAndPromotesOnlySameConfirmedSource() = kotlinx.coroutines.runBlocking {
        val listener = com.sun.net.httpserver.HttpServer.create(java.net.InetSocketAddress("127.0.0.1", 0), 0)
        val workers = java.util.concurrent.Executors.newFixedThreadPool(2)
        listener.executor = workers
        val filesReady = java.util.concurrent.CountDownLatch(1)
        val guardReady = java.util.concurrent.CountDownLatch(1)
        val guardReached = java.util.concurrent.atomic.AtomicBoolean()
        val confirmed = java.util.concurrent.atomic.AtomicReference("# Raw source\n\nExact café 😀\n")
        val raw = confirmed.get()
        val path = "nested/café + #.md"
        val identity = "a".repeat(32)
        val reads = java.util.concurrent.CopyOnWriteArrayList<String>()
        fun respond(exchange: com.sun.net.httpserver.HttpExchange, body: JSONObject) {
            val bytes = body.toString().toByteArray()
            exchange.sendResponseHeaders(200, bytes.size.toLong())
            exchange.responseBody.use { it.write(bytes) }
        }
        listener.createContext("/api/startup-source") { exchange ->
            reads += exchange.requestURI.rawQuery
            respond(exchange, JSONObject().put("version", 1).put("path", path).put("content", raw)
                .put("root_path", "/owned").put("workspace_id", JSONObject.NULL))
        }
        listener.createContext("/api/files") { exchange ->
            filesReady.await(8, java.util.concurrent.TimeUnit.SECONDS)
            val file = JSONObject().put("name", path.substringAfterLast('/')).put("path", path).put("doc_id", "wanted").put("dir", false)
            val sibling = JSONObject().put("name", "sibling.txt").put("path", "nested/sibling.txt").put("dir", false)
            val directory = JSONObject().put("name", "nested").put("path", "nested").put("dir", true)
                .put("children", org.json.JSONArray().put(file).put(sibling))
            respond(exchange, JSONObject().put("root", "Owned").put("root_path", "/owned").put("workspace_id", identity)
                .put("tree", org.json.JSONArray().put(directory)))
        }
        listener.createContext("/api/docs/wanted") { exchange ->
            guardReached.set(true)
            guardReady.await(8, java.util.concurrent.TimeUnit.SECONDS)
            respond(exchange, JSONObject().put("source", confirmed.get()).put("save_guard", "owned guard"))
        }
        listener.start()
        val origin = "http://127.0.0.1:${listener.address.port}"
        val workspace = Workspace(this)
        try {
            workspace.connectFromLaunch(origin, path)
            withTimeout(3000) { while (workspace.currentBuffer == null) delay(10) }
            val early = workspace.currentBuffer!!
            val state = early.state
            val scroll = early.sourceScroll
            early.state.edit { selection = androidx.compose.ui.text.TextRange(2, 5) }
            scroll.scrollTo(120)
            assertEquals(raw, early.text)
            assertTrue(early.readOnly)
            assertNull(early.saveGuard)
            assertNull(early.language)
            assertEquals(1L, filesReady.count)
            assertEquals(1, reads.size)
            assertEquals("path=$path", java.net.URLDecoder.decode(reads.single(), "UTF-8"))
            assertFalse(early.applySourceEdits(listOf(NativeSourceEdit(0, 1, "X")), raw))
            filesReady.countDown()
            withTimeout(3000) { while (!guardReached.get()) delay(10) }
            assertEquals(listOf(path, "nested/sibling.txt"), workspace.files.map { it.path })
            assertSame(early, workspace.currentBuffer)
            assertTrue(early.readOnly)
            val changed = "$raw\nChanged on disk.\n"
            confirmed.set(changed)
            guardReady.countDown()
            withTimeout(3000) { while (workspace.connecting) delay(10) }
            assertSame(early, workspace.currentBuffer)
            assertSame(state, early.state)
            assertSame(scroll, early.sourceScroll)
            assertTrue(early.readOnly)
            assertEquals(raw, early.text)
            assertNull(early.saveGuard)
            assertTrue(workspace.status.contains("changed during first open"))

            confirmed.set(raw)
            workspace.refresh()
            assertSame(early, workspace.currentBuffer)
            assertSame(state, early.state)
            assertSame(scroll, early.sourceScroll)
            assertEquals(120, early.sourceScroll.value)
            assertEquals(androidx.compose.ui.text.TextRange(2, 5), early.state.selection)
            assertFalse(early.readOnly)
            assertEquals("owned guard", early.saveGuard)
        } finally {
            filesReady.countDown()
            guardReady.countDown()
            workspace.currentBuffer?.language?.close()
            workspace.debugger?.close()
            workspace.agent.debugger?.close()
            listener.stop(0)
            workers.shutdownNow()
        }
    }

    @Test(timeout = 12000)
    fun recordedFenceRunReadsOnlyItsAdmittedCurrentTerminal() = kotlinx.coroutines.runBlocking {
        val server = com.sun.net.httpserver.HttpServer.create(java.net.InetSocketAddress("127.0.0.1", 0), 0)
        val root = "/owned-run"
        val endpoint = "/api/docs/workspace/agent/turns/owned/fence-run"
        val received = java.util.concurrent.CopyOnWriteArrayList<JSONObject>()
        val screens = java.util.concurrent.CopyOnWriteArrayList<String>()
        val hold = java.util.concurrent.CountDownLatch(1)
        val assistantId = "term-assistant"
        val userId = "term-user"
        fun summary(id: String, exit: Int?) = JSONObject().put("id", id).put("state", if (exit == null) "working" else "finished")
            .put("exit_code", exit ?: JSONObject.NULL)
        fun respond(exchange: com.sun.net.httpserver.HttpExchange, reply: JSONObject) {
            val bytes = reply.toString().toByteArray()
            exchange.sendResponseHeaders(200, bytes.size.toLong())
            exchange.responseBody.use { it.write(bytes) }
        }
        server.createContext("/api/files") { exchange ->
            respond(exchange, JSONObject().put("root", "Owned").put("root_path", root).put("tree", org.json.JSONArray()))
        }
        server.createContext(endpoint) { exchange ->
            assertEquals("POST", exchange.requestMethod)
            val body = JSONObject(exchange.requestBody.bufferedReader().use { it.readText() })
            received.add(body)
            if (received.size == 3) hold.await(4, java.util.concurrent.TimeUnit.SECONDS)
            val id = if (body.getString("side") == "user") userId else assistantId
            respond(exchange, JSONObject(body.toString()).put("version", 1).put("workspace", root).put("turn", "owned")
                .put("terminal", summary(id, null)))
        }
        server.createContext("/api/terminals") { exchange ->
            val path = exchange.requestURI.path
            if (path == "/api/terminals") {
                respond(exchange, JSONObject().put("sessions", org.json.JSONArray().put(summary(userId, 0)).put(summary(assistantId, 0))))
            } else {
                val id = path.removePrefix("/api/terminals/").removeSuffix("/output")
                assertTrue(id in listOf(userId, assistantId))
                screens.add(id)
                val output = if (id == userId) "41\n" else "52\n"
                respond(exchange, JSONObject().put("bytes", org.json.JSONArray(output.toByteArray().map { it.toInt() }))
                    .put("revision", "screen-revision").put("next", JSONObject.NULL))
            }
        }
        server.start()
        val engine = Engine("http://127.0.0.1:${server.address.port}")
        val session = AgentSession()
        val turn = AgentTurn("owned", "~~~py\nprint(41)\n~~~", "~~~py\nprint(52)\n~~~", null, "ok")
        try {
            engine.files()
            session.turns = listOf(turn)
            session.prompt = "Unsent draft"
            for (assistant in listOf(false, true)) {
                val reading = session.fenceReading(turn, assistant)!!
                val buffer = session.document(if (assistant) "owned-agent" else "owned-user", reading.source)
                buffer.inlineMarkup = computeInlineMarkup(buffer.text, buffer.file.path, true)
                val state = buffer.state
                session.runRecordedFence(engine, buffer, reading, 0) { true }
                assertEquals(reading.side, received.last().getString("side"))
                assertEquals(debugHash(reading.source), received.last().getString("source_hash"))
                assertEquals(0, received.last().getInt("opening_line"))
                assertEquals(setOf("side", "source_hash", "opening_line"), received.last().keys().asSequence().toSet())
                assertEquals(if (assistant) "52\n" else "41\n", session.recordedRun(buffer)?.output)
                assertEquals(0, session.recordedRun(buffer)?.terminal?.exit)
                assertSame(state, buffer.state)
                assertEquals(reading.source, buffer.text)
            }
            val stale = session.fenceReading(turn, true)!!
            val source = session.document("owned-agent", stale.source)
            val pending = launch { session.runRecordedFence(engine, source, stale, 0) { true } }
            withTimeout(3000) { while (received.size < 3) delay(10) }
            session.turns = listOf(turn.copy(answer = "~~~py\nprint(99)\n~~~"))
            hold.countDown()
            pending.join()
            assertNull(session.recordedRun(source))
            assertEquals("41\n", session.recordedRun(session.document("owned-user", turn.prompt))?.output)
            assertEquals(listOf(userId, assistantId), screens)
            assertEquals("Unsent draft", session.prompt)
        } finally {
            hold.countDown()
            server.stop(0)
        }
    }

    @Test(timeout = 12000)
    fun recordedConstructionReadsExactSidesAndRetiresClosedDisclosures() = kotlinx.coroutines.runBlocking {
        val server = com.sun.net.httpserver.HttpServer.create(java.net.InetSocketAddress("127.0.0.1", 0), 0)
        val root = "/owned-construction"
        val workspace = "a".repeat(32)
        val endpoint = "/api/docs/workspace/agent/turns/owned/fence-construction"
        val requests = java.util.concurrent.CopyOnWriteArrayList<JSONObject>()
        val hold = java.util.concurrent.CountDownLatch(1)
        fun respond(exchange: com.sun.net.httpserver.HttpExchange, reply: JSONObject) {
            val bytes = reply.toString().toByteArray()
            exchange.sendResponseHeaders(200, bytes.size.toLong())
            exchange.responseBody.use { it.write(bytes) }
        }
        fun entry(text: String) = JSONObject().put("kind", "file").put("executable", false)
            .put("bytes", text.toByteArray(Charsets.UTF_8).size).put("sha256", debugHash(text))
            .put("text", text).put("text_status", "exact")
        server.createContext("/api/files") { exchange ->
            respond(exchange, JSONObject().put("root", "Owned").put("root_path", root)
                .put("workspace_id", workspace).put("tree", org.json.JSONArray()))
        }
        server.createContext(endpoint) { exchange ->
            assertEquals("POST", exchange.requestMethod)
            val address = JSONObject(exchange.requestBody.bufferedReader().use { it.readText() })
            requests.add(address)
            if (requests.size == 4) hold.await(4, java.util.concurrent.TimeUnit.SECONDS)
            val changes = org.json.JSONArray().put(JSONObject().put("path", "out.txt")
                .put("before", JSONObject.NULL).put("after", entry("captured é\r\n")))
            respond(exchange, JSONObject(address.toString()).put("version", 1).put("read_only", true)
                .put("turn", "owned").put("workspace", root).put("workspace_id", if (requests.size == 3) "b".repeat(32) else workspace)
                .put("entry", "entry.md").put("document", "recorded.md").put("operation_id", "exact")
                .put("opening_byte", 7).put("completed_fingerprint", "c".repeat(64)).put("changes", changes))
        }
        server.start()
        val engine = Engine("http://127.0.0.1:${server.address.port}")
        val session = AgentSession()
        val body = "\r\n<hick:patch id=\"exact\">\r\nchange\r\n</hick:patch>\r\n" +
            "~~~patch\r\nbare\r\n~~~\r\n~~~patch id=\"another\"\r\nchange\r\n~~~\r\n" +
            "~~~text\r\n<hick:patch id=\"quoted\">\r\n~~~"
        val turn = AgentTurn("owned", "User$body", "Assistant$body", null, "ok")
        try {
            engine.files()
            session.turns = listOf(turn)
            session.prompt = "Unsent draft"
            val readers = listOf(false, true).map { assistant ->
                val reading = session.fenceReading(turn, assistant)!!
                val buffer = session.document(if (assistant) "owned-agent" else "owned-user", reading.source)
                buffer.inlineMarkup = computeInlineMarkup(buffer.text, buffer.file.path, true)
                assertEquals(setOf(1, 7), buffer.inlineMarkup.debug.declaredPatches)
                assertEquals(setOf(4), buffer.inlineMarkup.debug.recordedPatches)
                val state = buffer.state
                val reader = RecordedConstructionReader(engine, buffer, reading) { true }
                assertNull(reader.read(4) { true })
                val receipt = reader.read(if (assistant) 7 else 1) { true }!!
                assertEquals(setOf("side", "source_hash", "opening_line"), requests.last().keys().asSequence().toSet())
                assertEquals(reading.side, requests.last().getString("side"))
                assertEquals(debugHash(reading.source), requests.last().getString("source_hash"))
                assertEquals(if (assistant) 7 else 1, requests.last().getInt("opening_line"))
                assertEquals("recorded.md", receipt.document)
                assertEquals("exact", receipt.operation)
                val change = receipt.changes.single()
                assertEquals("out.txt", change.path)
                assertNull(change.before)
                assertEquals("captured é\r\n", change.after?.text)
                assertSame(state, buffer.state)
                assertEquals(reading.source, buffer.text)
                reader
            }
            val reader = readers.last()
            assertTrue(runCatching { reader.read(1) { true } }.exceptionOrNull()?.message.orEmpty().contains("different recorded patch or workspace"))
            var open = true
            val pending = async { reader.read(1) { open } }
            withTimeout(3000) { while (requests.size < 4) delay(10) }
            open = false
            hold.countDown()
            assertNull(pending.await())
            assertEquals("Unsent draft", session.prompt)
        } finally {
            hold.countDown()
            server.stop(0)
        }
    }

    @Test(timeout = 12000)
    fun recordedAgentFencesUseTheExactSideRevisionAndMappedFrames() = kotlinx.coroutines.runBlocking {
        val listener = com.sun.net.httpserver.HttpServer.create(java.net.InetSocketAddress("127.0.0.1", 0), 0)
        val path = "/api/docs/workspace/agent/turns/owned/fence-debug"
        val id = "0123456789abcdef"
        val uri = "hick-project:///$id/message.md"
        val workspace = "/owned-workspace"
        val posts = java.util.concurrent.CopyOnWriteArrayList<JSONObject>()
        val released = java.util.concurrent.atomic.AtomicInteger()
        val corrupt = java.util.concurrent.atomic.AtomicBoolean()
        val heldReceipt = java.util.concurrent.atomic.AtomicReference<java.util.concurrent.CountDownLatch?>()
        fun respond(exchange: com.sun.net.httpserver.HttpExchange, body: JSONObject) {
            val bytes = body.toString().toByteArray()
            exchange.sendResponseHeaders(200, bytes.size.toLong())
            exchange.responseBody.use { it.write(bytes) }
        }
        listener.createContext("/api/files") { exchange ->
            respond(exchange, JSONObject().put("root", "Owned").put("root_path", workspace).put("tree", org.json.JSONArray()))
        }
        listener.createContext(path) { exchange ->
            val request = JSONObject(exchange.requestBody.bufferedReader().use { it.readText() })
            val held = heldReceipt.get()
            posts.add(request)
            held?.await(3, java.util.concurrent.TimeUnit.SECONDS)
            respond(exchange, JSONObject(request.toString()).put("id", id).put("uri", uri).put("turn", "owned")
                .put("version", 1).put("started", false).put("workspace", workspace)
                .apply { if (corrupt.get()) put("source_hash", "0".repeat(64)) })
        }
        listener.createContext("/api/project-contexts/$id") { exchange ->
            assertEquals("DELETE", exchange.requestMethod)
            released.incrementAndGet()
            respond(exchange, JSONObject().put("removed", true))
        }
        listener.start()
        val engine = Engine("http://127.0.0.1:${listener.address.port}")
        val debug = NativeDebugger(engine, this, recordedMessages = true)
        val agent = AgentSession()
        val user = "Prose 😀\r\n\r\n  ```python\r\n  value = 41\r\n  print(value)\r\n  ```\r\n"
        val answer = "Answer\n\n~~~py\nvalue = 52\nprint(value)\n~~~\n"
        val turn = AgentTurn("owned", user, answer, null, "ok")
        val sent = mutableListOf<JSONObject>()
        debug.emitForTest = { sent.add(it) }
        try {
            engine.files()
            agent.turns = listOf(turn)
            agent.prompt = "independent unsent draft"
            assertNull(agent.fenceReading(turn.copy(status = "running"), true))
            assertTrue(recordedFenceDebugContext("~~~py\nvalue = 1\n").launch.isEmpty())
            for (assistant in listOf(false, true)) {
                val reading = agent.fenceReading(turn, assistant)!!
                val message = agent.document(if (assistant) "owned-agent" else "owned-user", reading.source)
                message.inlineMarkup = computeInlineMarkup(message.text, message.file.path, true)
                assertTrue(message.readOnly)
                assertEquals(mapOf(2 to 5), message.inlineMarkup.debug.recordedClosing)
                assertEquals(setOf(3, 4), message.inlineMarkup.debug.code.keys)
                val originalState = message.state
                debug.toggle(message, 3)
                val before = sent.size
                debug.prepareRecordedFence(message, reading, 2)
                val start = sent[before]
                val hash = debugHash(reading.source)
                assertEquals(reading.side, posts.last().getString("side"))
                assertEquals(hash, posts.last().getString("source_hash"))
                assertEquals(2, posts.last().getInt("opening_line"))
                assertEquals("start", start.getString("op"))
                assertEquals(uri, start.getString("doc"))
                assertEquals(hash, start.getString("source_hash"))
                assertFalse(start.has("profile"))
                assertEquals(3, start.getJSONArray("breakpoints").getJSONObject(0).getInt("line"))
                val session = "debug-${before}"
                debug.receive(JSONObject().put("event", "started").put("request_id", start.getString("request_id"))
                    .put("doc", uri).put("session", session).put("source_hash", hash))
                fun stop(address: String) = JSONObject().put("event", "stopped").put("stop", 1).put("thread", 0).put("line", 3)
                    .put("session", session).put("frames", org.json.JSONArray().put(JSONObject().put("id", 0)
                        .put("line", 3).put("source", address).put("source_hash", hash).put("in_document", true)))
                debug.receive(stop("hick-project:///fedcba9876543210/message.md"))
                assertNull(debug.line(message.file.path))
                debug.receive(stop(uri))
                assertEquals(3, debug.line(message.file.path))
                debug.step("over")
                assertEquals("over", sent.last().getString("how"))
                assertEquals(0, sent.last().getInt("frame"))
                debug.receive(JSONObject().put("event", "finished").put("session", session).put("exit_code", 0))
                assertSame(originalState, message.state)
                assertEquals(reading.source, message.text)
                assertEquals("independent unsent draft", agent.prompt)
            }
            withTimeout(3000) { while (released.get() != 2) delay(10) }
            val oldReading = agent.fenceReading(turn, false)!!
            val message = agent.document("owned-user", oldReading.source)
            agent.turns = listOf(turn.copy(prompt = "replaced"))
            val postCount = posts.size
            debug.prepareRecordedFence(message, oldReading, 2)
            assertEquals(postCount, posts.size)
            agent.turns = listOf(turn)
            val barrier = java.util.concurrent.CountDownLatch(1)
            heldReceipt.set(barrier)
            val beforeDelayed = sent.size
            val delayed = async { debug.prepareRecordedFence(message, oldReading, 2) }
            withTimeout(3000) { while (posts.size == postCount) delay(10) }
            agent.turns = listOf(turn.copy(prompt = "changed during admission"))
            barrier.countDown()
            delayed.await()
            heldReceipt.set(null)
            assertEquals(beforeDelayed, sent.size)
            withTimeout(3000) { while (released.get() != 3) delay(10) }
            agent.turns = listOf(turn)
            corrupt.set(true)
            val before = sent.size
            debug.prepareRecordedFence(message, oldReading, 2)
            assertEquals(before, sent.size)
            assertEquals("failed", debug.status)
        } finally {
            heldReceipt.get()?.countDown()
            debug.close()
            listener.stop(0)
        }
    }

    @Test(timeout = 12000)
    fun launchOpensTheRequestedFileBesideItsContainingTreeAndRetainsTheEditor() = kotlinx.coroutines.runBlocking {
        val listener = com.sun.net.httpserver.HttpServer.create(java.net.InetSocketAddress("127.0.0.1", 0), 0)
        val reads = java.util.Collections.synchronizedList(mutableListOf<String>())
        val path = "src/notes.txt"
        val source = "Exact saved text\n"
        listener.createContext("/api/files") { exchange ->
            val tree = """{"root":"Fixture","tree":[{"name":"root.txt","path":"root.txt","dir":false},{"name":"src","path":"src","dir":true,"children":[{"name":"notes.txt","path":"src/notes.txt","dir":false},{"name":"sibling.txt","path":"src/sibling.txt","dir":false}]}]}"""
            val bytes = tree.toByteArray()
            exchange.sendResponseHeaders(200, bytes.size.toLong())
            exchange.responseBody.use { it.write(bytes) }
        }
        listener.createContext("/api/file") { exchange ->
            reads += exchange.requestURI.query
            val bytes = JSONObject().put("content", source).put("hash", "base").toString().toByteArray()
            exchange.sendResponseHeaders(200, bytes.size.toLong())
            exchange.responseBody.use { it.write(bytes) }
        }
        listener.start()
        val origin = "http://127.0.0.1:${listener.address.port}"
        val workspace = Workspace(this)
        try {
            workspace.connectFromLaunch(origin, path)
            withTimeout(3000) { while (workspace.currentBuffer == null) delay(10) }
            val buffer = workspace.currentBuffer!!
            assertEquals(path, buffer.file.path)
            assertEquals(source, buffer.text)
            assertEquals("src", workspace.directoryScope)
            assertEquals(listOf(path, "src/sibling.txt"), workspace.files.map { it.path })
            assertEquals(listOf("root.txt", "src"), workspace.rootFiles.map { it.path })
            assertEquals(listOf("files", path), workspace.layout.groups().map { it.active })

            buffer.state.edit { append("draft") }
            workspace.connectFromLaunch(origin, "root.txt")
            assertSame(buffer, workspace.currentBuffer)
            assertEquals(source + "draft", buffer.text)

            val missing = Workspace(this)
            missing.connectFromLaunch(origin, "outside.txt")
            withTimeout(3000) { while (missing.connecting || missing.engine == null) delay(10) }
            assertNull(missing.currentBuffer)
            assertEquals(listOf("root.txt", "src"), missing.files.map { it.path })
            assertTrue(missing.status.contains("outside.txt"))
            assertEquals(listOf("path=$path"), reads.toList())
        } finally {
            workspace.buffers.values.forEach { it.language?.close() }
            listener.stop(0)
        }
    }

    @Test fun reservedSourceTabsRemainIndependentFromUtilities() {
        val w=Workspace()
        for(path in listOf("files","agent","terminal","Untitled-real.txt","\u0000source:literal")) {
            val b=DocumentBuffer(FileNode(path,path,false,emptyList(),null,null),FileText("saved literal","hash","guard"))
            w.buffers[path]=b;w.show(sourceTab(path));assertSame(b,w.currentBuffer)
            assertEquals(SourceTabIdentity.Source(path),tabIdentity(w.active!!))
            assertFalse(b.isUntitledDraft)
            w.close(sourceTab(path));assertTrue(w.layout.groups().any{"files" in it.tabs})
        }
        w.show("files");assertNull(w.currentBuffer)
        w.newTab();assertTrue(w.currentBuffer!!.isUntitledDraft)
    }

    @Test(timeout=20000) fun sharedLanguageWireRoutesRepliesDetachesOneSourceAndReplaysAfterFailure() = kotlinx.coroutines.runBlocking {
        // Contract: .hickory-source/frontends/contract.md. Own protocol fixture, real OkHttp WebSocket.
        val root=generateSequence(java.io.File(System.getProperty("user.dir")).absoluteFile) {it.parentFile}.first {it.resolve("scripts/simple-language-wire-fixture.py").isFile}
        val process=ProcessBuilder("python3",root.resolve("scripts/simple-language-wire-fixture.py").path).redirectErrorStream(true).start()
        val connection:NativeLanguageConnection
        try {
            val greeting=JSONObject(process.inputStream.bufferedReader().readLine());val engine=Engine(greeting.getString("origin"));connection=NativeLanguageConnection(engine,this)
            try {
                fun params(uri:String)=JSONObject().put("textDocument",JSONObject().put("uri",uri))
                suspend fun metrics()=engine.request("GET","/state")
                val a="hick:///a.py";val b="hick:///b.py";val c="hick:///c.py"
                val reportsA=mutableListOf<String>();val reportsB=mutableListOf<String>();val reportsC=mutableListOf<String>();var failuresA=0;var failuresB=0;var failuresC=0
                val ownerA=connection.attach(a,"first",1,{it.optJSONObject("params")?.optString("uri")?.takeIf {it.isNotEmpty()}?.let(reportsA::add)},{failuresA++})
                val ownerB=connection.attach(b,"second",1,{it.optJSONObject("params")?.optString("uri")?.takeIf {it.isNotEmpty()}?.let(reportsB::add)},{failuresB++})
                connection.attach(c,"third",1,{it.optJSONObject("params")?.optString("uri")?.takeIf {it.isNotEmpty()}?.let(reportsC::add)},{failuresC++})
                val slow=async {connection.request("fixture/slow",params(a),ownerA) as JSONObject};val fast=async {connection.request("fixture/fast",params(b),ownerB) as JSONObject}
                assertEquals(b,fast.await().getString("uri"));assertEquals("first",slow.await().getString("text"))
                assertEquals(listOf(a),reportsA);assertEquals(listOf(b),reportsB);assertEquals(listOf(c),reportsC);assertEquals(1,metrics().getInt("connections"))
                var rejected=false
                val duplicate=connection.attach(a,"conflicting draft",2,{fail("Duplicate source received a notification")},{rejected=true})
                assertTrue(rejected);connection.change(duplicate,"corrupt",3)
                assertTrue(runCatching {connection.request("fixture/fast",params(a),duplicate)}.isFailure)
                assertFalse((connection.request("fixture/askApply",params(b),ownerB) as JSONObject).getBoolean("applied"))
                val preceding=metrics().getJSONArray("ids").length()
                val blocking=async {runCatching {connection.request("fixture/block",params(a),ownerA)}}
                connection.request("fixture/fast",params(b),ownerB)
                withTimeout(2000) {while(metrics().getJSONArray("ids").length()<preceding+2) delay(10)}
                connection.detach(ownerA);assertTrue(blocking.await().isFailure)
                assertEquals("second",(connection.request("fixture/fast",params(b),ownerB) as JSONObject).getString("text"))
                assertTrue(runCatching {connection.request("fixture/drop",params(b),ownerB)}.isFailure)
                assertEquals(0,failuresA);assertEquals(1,failuresB);assertEquals(1,failuresC)
                connection.change(ownerB,"new second",2);delay(1100)
                assertEquals("new second",(connection.request("fixture/fast",params(b),ownerB) as JSONObject).getString("text"))
                val final=metrics();assertEquals(2,final.getInt("connections"));assertFalse(final.getJSONObject("documents").has(a));assertEquals("new second",final.getJSONObject("documents").getString(b));assertEquals("third",final.getJSONObject("documents").getString(c))
                val ids=final.getJSONArray("ids");assertEquals(ids.length(),(0 until ids.length()).map {ids.getInt(it)}.toSet().size)
                val file=FileNode("outline.py","outline.py",false,emptyList(),null,null,"python");val buffer=DocumentBuffer(file,FileText("source","base"))
                val service=NativeLanguage(engine,buffer,this,connection);service.start()
                try {
                    val reading=async {service.readOutline("source")};delay(10);buffer.state.edit {selection=androidx.compose.ui.text.TextRange(3)}
                    assertEquals("background",reading.await().first().name);assertEquals(3,buffer.state.selection.start);assertTrue(service.locations.isEmpty())
                    service.navigate(SemanticQuery.Supertypes);assertEquals("supertype",service.locations.first().name)
                    service.navigate(SemanticQuery.Subtypes);assertEquals("subtype",service.locations.first().name)
                    val hierarchy=async{service.navigate(SemanticQuery.Supertypes)};delay(10);buffer.state.edit{selection=androidx.compose.ui.text.TextRange(4)};hierarchy.await()
                    assertTrue(service.locations.isEmpty());assertTrue(service.locationStatus.contains("changed"));buffer.state.edit{selection=androidx.compose.ui.text.TextRange(3)}
                    val stale=async {runCatching {service.readOutline("source")}};delay(10);buffer.state.edit {replace(0,length,"draft")}
                    assertTrue(stale.await().isFailure);assertEquals("draft",buffer.text)
                } finally {service.close()}
            } finally {connection.close()}
        } finally {process.destroy();process.waitFor(5,java.util.concurrent.TimeUnit.SECONDS)}
    }

    @Test fun workspaceAnalysisScopeCountsUnknownAndGeneratedSources() {
        // Contract: .hickory-source/frontends/contract.md
        val a=FileNode("a.py","src/a.py",false,emptyList(),null,null,"python")
        val scope=workspaceAnalysisScope(listOf(FileNode("src","src",true,listOf(a,FileNode("out.py","src/out.py",false,emptyList(),null,"owner","python")),null,null),a,FileNode("notes.md","notes.md",false,emptyList(),null,null,"markdown"),FileNode("unknown","unknown",false,emptyList(),null,null)))
        assertEquals(listOf("notes.md","src/a.py"),scope.files.map {it.path});assertEquals(1,scope.generated);assertEquals(1,scope.unclassified)
    }
    @Test fun backgroundSourceAcquisitionSharesReadAndRetainsAvailableDraft() = kotlinx.coroutines.runBlocking {
        // Contract: .hickory-source/frontends/contract.md
        val listener=com.sun.net.httpserver.HttpServer.create(java.net.InetSocketAddress("127.0.0.1",0),0)
        val count=java.util.concurrent.atomic.AtomicInteger();val arrived=kotlinx.coroutines.CompletableDeferred<Unit>();val gate=java.util.concurrent.CountDownLatch(1)
        listener.createContext("/api/file") {exchange ->
            count.incrementAndGet();arrived.complete(Unit);gate.await(5,java.util.concurrent.TimeUnit.SECONDS)
            val bytes="""{"path":"background.py","content":"loaded disk text","hash":"disk"}""".toByteArray();exchange.sendResponseHeaders(200,bytes.size.toLong());exchange.responseBody.use {it.write(bytes)}
        };listener.start()
        val workspace=Workspace(this);workspace.engine=Engine("http://127.0.0.1:${listener.address.port}")
        val file=FileNode("background.py","background.py",false,emptyList(),null,null,"python")
        try {
            val first=async {workspace.loadBuffer(file)};val second=async {workspace.loadBuffer(file)}
            kotlinx.coroutines.withTimeout(3000) {arrived.await()}
            val draft=DocumentBuffer(file,FileText("base","base"));draft.state.edit {replace(0,length,"retained draft")};workspace.buffers[file.path]=draft
            gate.countDown();assertSame(draft,first.await());assertSame(draft,second.await());assertEquals(1,count.get());assertEquals("retained draft",draft.text);assertTrue(draft.dirty);assertNull(workspace.active)
            val other=file.copy(name="other.py",path="other.py")
            val loaded=workspace.loadBuffer(other);assertEquals("loaded disk text",loaded.text);assertNotNull(loaded.language);assertNull(workspace.active)
            val third=workspace.loadBuffer(other.copy(name="third.py",path="third.py"));assertSame(loaded.language!!.connection,third.language!!.connection);assertNull(workspace.active)
        } finally {gate.countDown();workspace.buffers.values.forEach {it.language?.close()};listener.stop(0)}
    }

    @Test fun diagnosticEngineHashesOverrideWindowCounterCoincidence() {
        // Contract: .hickory-source/frontends/contract.md
        val source="value";val hash="sha256:"+java.security.MessageDigest.getInstance("SHA-256").digest(source.toByteArray()).joinToString(""){"%02x".format(it)}
        val range=JSONObject().put("start",JSONObject().put("line",0).put("character",0)).put("end",JSONObject().put("line",0).put("character",1))
        val item=JSONObject().put("range",range).put("message","report").put("data",JSONObject().put("hickory",JSONObject().put("source_hash",hash)))
        val params=JSONObject().put("uri","hick:///proof.py").put("version",99).put("diagnostics",org.json.JSONArray().put(item))
        assertNotNull(decodeNativeDiagnostics(params,source,1)[0].location)
        assertNull(decodeNativeDiagnostics(params,"changed",99)[0].location)
        item.put("data",JSONObject().put("hickory",JSONObject().put("source_hash",JSONObject.NULL)).put("provider",JSONObject().put("hickory",JSONObject().put("source_hash",hash))))
        params.put("version",1);val unknown=decodeNativeDiagnostics(params,source,1)[0];assertEquals("report",unknown.message);assertNull(unknown.location)
    }

    @Test fun workspaceChangeEditorBatchAdmissionAndUndo() {
        // Contract: .hickory-source/frontends/contract.md
        val workspace = Workspace()
        val a = DocumentBuffer(FileNode("a.py","a.py",false,emptyList(),null,null),FileText("one","base"))
        val b = DocumentBuffer(FileNode("b.py","b.py",false,emptyList(),null,null),FileText("one","base"))
        workspace.buffers["a.py"]=a;workspace.buffers["b.py"]=b
        fun edit() = JSONObject().put("range",JSONObject().put("start",JSONObject().put("line",0).put("character",0)).put("end",JSONObject().put("line",0).put("character",3))).put("newText","renamed")
        val raw=JSONObject().put("changes",JSONObject().put("hick:///a.py",org.json.JSONArray().put(edit())).put("hick:///b.py",org.json.JSONArray().put(edit())))
        val plan=planWorkspaceEdit("Rename",raw,workspace.sourceSnapshots())
        b.saving=true;assertTrue(runCatching {workspace.applyWorkspacePlan(plan)}.isFailure);assertEquals("one",a.text);b.saving=false
        workspace.applyWorkspacePlan(plan);assertEquals("renamed",a.text);assertEquals("renamed",b.text)
        a.state.edit {replace(0,length,"later local edit")}
        assertTrue(runCatching {workspace.applyWorkspacePlan(invertChangePlan(plan))}.isFailure);assertEquals("renamed",b.text)
        a.state.edit {replace(0,length,"renamed")};workspace.applyWorkspacePlan(invertChangePlan(plan))
        assertEquals("one",a.text);assertEquals("one",b.text);assertEquals("one",a.base);assertEquals("one",b.base)
    }

    @Test fun workspaceChangePlanSharedSafetyCasesAndInverse() {
        // Contract: .hickory-source/frontends/contract.md
        val fixture = JSONObject(javaClass.classLoader!!.getResource("change-plan-cases.json")!!.readText())
        val rawSources = fixture.getJSONObject("sources")
        val sources = rawSources.keys().asSequence().associateWith { path -> rawSources.getJSONObject(path).let { SourceSnapshot(it.getString("text"),it.getBoolean("writable")) } }
        val plan = planWorkspaceEdit("Rename",fixture.getJSONObject("edit"),sources)
        val expected = fixture.getJSONObject("after")
        assertEquals(expected.keys().asSequence().associateWith { expected.getString(it) },plan.patches.associate { it.path to it.after })
        assertNull(validateChangePlan(plan,sources))
        validateReferencedTargets(plan,listOf("a.py","b.py","b.py"))
        assertTrue(runCatching {validateReferencedTargets(plan,listOf("a.py","unopened.py"))}.isFailure)
        val after = sources.toMutableMap(); plan.patches.forEach { after[it.path] = SourceSnapshot(it.after,true) }
        assertNotNull(validateChangePlan(plan,after))
        val inverse = invertChangePlan(plan); assertNull(validateChangePlan(inverse,after))
        for (patch in inverse.patches) {
            val restored = StringBuilder(patch.before)
            for (edit in patch.edits.asReversed()) restored.replace(edit.from,edit.to,edit.text)
            assertEquals(sources.getValue(patch.path).text,restored.toString())
        }
        after["b.py"] = SourceSnapshot("later edit",true); assertNotNull(validateChangePlan(inverse,after))
        val invalid = fixture.getJSONArray("invalid")
        for (index in 0 until invalid.length()) {
            val item = invalid.getJSONObject(index); val captured = sources.toMutableMap()
            item.optString("readonly").takeIf { it.isNotEmpty() }?.let { path -> captured[path] = captured.getValue(path).copy(writable=false) }
            assertTrue(item.getString("name"),runCatching { planWorkspaceEdit(item.getString("name"),item.getJSONObject("edit"),captured) }.isFailure)
            assertEquals("é😀 one\nsecond",captured.getValue("a.py").text); assertEquals("call(one)",captured.getValue("b.py").text)
        }
    }

    @Test fun workspaceSearchUnicodeAndQueryInvalidation() {
        // Contract: .hickory-source/frontends/contract.md
        assertEquals(4, searchCharacter("é😀 target", 7)); assertNull(searchCharacter("é😀 target", 3))
        assertNull(searchCharacter("x", -1)); assertNull(searchCharacter("x", 9))
        val search = WorkspaceSearch(); val before = search.revision
        search.result = WorkspaceSearchResult(emptyList(), true); search.updateQuery("new")
        assertNotEquals(before, search.revision); assertNull(search.result)
        val queryRevision = search.revision; search.updateOptions(wholeWord = true); assertNotEquals(queryRevision, search.revision)
    }
    @Test fun workspaceSearchRefusesDirtySourceWithoutNavigation() = kotlinx.coroutines.runBlocking {
        // Contract: .hickory-source/frontends/contract.md
        val workspace = Workspace(); workspace.engine = Engine("http://127.0.0.1:1")
        val file = FileNode("search.txt", "search.txt", false, emptyList(), null, null)
        val buffer = DocumentBuffer(file, FileText("é😀 target", "base")); buffer.state.edit { append(" changed") }; workspace.buffers[file.path] = buffer
        val hash = java.security.MessageDigest.getInstance("SHA-256").digest("é😀 target".toByteArray()).joinToString("") { "%02x".format(it) }
        workspace.openSearchHit(WorkspaceSearchHit(file.path, 1, "é😀 target", 7, hash, null))
        assertTrue(workspace.search.notice.contains("Search source changed")); assertNull(workspace.active)
        assertEquals("é😀 target changed", buffer.text); assertTrue(buffer.dirty)
    }

    @org.junit.Test fun semanticProblemsRetirePositionsWhenSourceChanges() = kotlinx.coroutines.runBlocking {
        // Contract: .hickory-source/frontends/contract.md
        val file = FileNode("problem.py", "problem.py", false, emptyList(), null, null)
        val buffer = DocumentBuffer(file, FileText("value = 1", "base"))
        val service = NativeLanguage(Engine("http://127.0.0.1:1"), buffer, this); buffer.language = service
        val workspace = Workspace(); workspace.buffers[file.path] = buffer
        val range = JSONObject().put("start", JSONObject().put("line", 0).put("character", 1)).put("end", JSONObject().put("line", 0).put("character", 5))
        val params = JSONObject().put("uri", "hick:///problem.py").put("version", 1).put("diagnostics", org.json.JSONArray().put(JSONObject().put("message", "issue").put("severity", 2).put("range", range)))
        service.diagnostics = decodeNativeDiagnostics(params, buffer.text, 1); service.diagnosticSource = buffer.text
        assertEquals(1, workspace.problems.size); assertTrue(workspace.problems[0].current)
        assertEquals(1, nextWorkspaceProblem(workspace.problems, null, 1)?.location?.character)
        buffer.state.edit { append(" changed") }; assertEquals(1, workspace.problems.size); assertFalse(workspace.problems[0].current)
        assertNull(nextWorkspaceProblem(workspace.problems, null, 1))
        params.remove("version"); service.diagnostics = decodeNativeDiagnostics(params, buffer.text, 1); service.diagnosticSource = buffer.text
        assertFalse(workspace.problems[0].current); assertNull(workspace.problems[0].location); service.close()
    }

    @org.junit.Test fun semanticNavigationTrailBranchesAndRetainsCarets() {
        // Contract: .hickory-source/frontends/contract.md
        fun point(name: String, line: Int = 0) = SemanticLocation("hick:///$name.py", line, 0)
        var trail = NavigationTrail().record(point("a"), point("b")).record(point("b"), point("c"))
        var step = trail.step(-1, point("c")); trail = step.first; assertEquals("b.py", step.second?.path)
        step = trail.step(1, point("b", 7)); trail = step.first; assertEquals("c.py", step.second?.path)
        step = trail.step(-1, point("c")); trail = step.first; assertEquals(7, step.second?.line)
        trail = trail.record(point("b", 7), point("d")); assertFalse(trail.canForward)
        val count = trail.entries.size; trail = trail.record(point("d"), point("d")); assertEquals(count, trail.entries.size)
        for (index in 0 until 300) trail = trail.record(null, point(index.toString()))
        assertEquals(256, trail.entries.size); assertNull(trail.step(1, null).second)
    }

    @org.junit.Test fun semanticNavigationRetiresStaleSourceAndSelection() = kotlinx.coroutines.runBlocking {
        // Contract: .hickory-source/frontends/contract.md
        val buffer = DocumentBuffer(FileNode("main.py", "main.py", false, emptyList(), null, null), FileText("value = 1\nprint(value)", "base"))
        val language = NativeLanguage(Engine("http://127.0.0.1:1"), buffer, this)
        val location = SemanticLocation("hick:///other.py", 0, 0)
        var opened = 0; buffer.openSemanticLocation = { opened++ }
        language.locations = listOf(location); language.locationSource = buffer.text; language.locationSelection = buffer.state.selection
        buffer.state.edit { selection = androidx.compose.ui.text.TextRange(1) }; assertTrue(!language.open(location)); assertEquals(0, opened)
        buffer.state.edit { selection = language.locationSelection; append("\n# changed") }; assertTrue(!language.open(location)); assertEquals(0, opened)
        language.locationSource = buffer.text; language.locationSelection = buffer.state.selection; assertTrue(language.open(location)); assertEquals(1, opened)
        val local = SemanticLocation("hick:///main.py", 0, 2)
        language.locations = listOf(local); assertTrue(language.open(local)); assertEquals(androidx.compose.ui.text.TextRange(2), buffer.state.selection)
        assertTrue(buffer.dirty); assertEquals(1, opened)
        language.close(); assertTrue(!language.open(local)); assertEquals(1, opened)
    }

    @org.junit.Test fun semanticNavigationRevealsReservedSourceTabAndRetainsTheUtility() = kotlinx.coroutines.runBlocking {
        val source = FileNode("agent", "agent", false, emptyList(), null, null)
        val buffer = DocumentBuffer(source, FileText("prefix target\n", "base"))
        val workspace = Workspace(this)
        workspace.buffers[source.path] = buffer
        workspace.show("agent")
        var revealed = 0
        workspace.openSemanticLocation(SemanticLocation("hick:///agent", 0, 7), expectedSource = buffer.text) { revealed++ }
        assertSame(buffer, workspace.currentBuffer)
        assertEquals(sourceTab(source.path), workspace.active)
        assertEquals(androidx.compose.ui.text.TextRange(7), buffer.state.selection)
        assertEquals(1, revealed)
        assertTrue(workspace.layout.groups().any { "agent" in it.tabs })
        assertFalse(buffer.dirty)
        workspace.show("agent")
        assertNull(workspace.currentBuffer)
    }

    @org.junit.Test fun semanticSymbolsAndCallTreesRetainNamesAndPositions() {
        // Contract: .hickory-source/frontends/contract.md
        val range = JSONObject().put("start", JSONObject().put("line", 2).put("character", 4)).put("end", JSONObject().put("line", 2).put("character", 8))
        val inner = JSONObject().put("name", "Inner").put("range", JSONObject().put("start", JSONObject().put("line", 3).put("character", 6)))
        val outer = JSONObject().put("name", "Outer").put("selectionRange", range).put("children", org.json.JSONArray().put(inner))
        val other = JSONObject().put("name", "Other").put("location", JSONObject().put("uri", "hick:///other.py").put("range", range))
        val call = JSONObject().put("from", JSONObject().put("name", "Caller").put("uri", "hick:///call.py").put("selectionRange", range))
        val locations = semanticLocations(org.json.JSONArray().put(outer).put(other).put(call), "hick:///source.py")
        assertEquals(listOf("Outer", "Inner", "Other", "Caller"), locations.map { it.name })
        assertEquals(listOf(2, 3, 2, 2), locations.map { it.line }); assertEquals("other.py", locations[2].path)
        assertEquals("textDocument/documentSymbol", SemanticQuery.Outline.method); assertEquals("workspace/symbol", SemanticQuery.Symbols.method)
    }
    @org.junit.Test fun semanticLocationSafetyAndNormalization() {
        // Contract: .hickory-source/frontends/contract.md
        assertEquals("src/my file.py", SemanticLocation("hick:///src/my%20file.py", 2, 4).path)
        assertEquals("literal%20.py", SemanticLocation("hick:///literal%2520.py", 0, 0).path)
        assertEquals("name?#.py", SemanticLocation("hick:///name%3F%23.py", 0, 0).path)
        for (uri in listOf("hick:/main.py", "hick:///a?", "file:///etc/passwd", "hick-output:///main.py", "hick://host/main.py", "hick:///../secret", "hick:///%2e%2e/secret", "hick:///a%5Cb", "hick:///a%00b", "hick:///a?context=old", "hick:///a#old")) assertEquals(uri, null, SemanticLocation(uri, 0, 0).path)
        val range = JSONObject().put("start", JSONObject().put("line", 2).put("character", 4)).put("end", JSONObject().put("line", 2).put("character", 8))
        val direct = JSONObject().put("uri", "hick:///a.py").put("range", range)
        val link = JSONObject().put("targetUri", "hick:///a.py").put("targetSelectionRange", range)
        assertEquals(listOf(SemanticLocation("hick:///a.py", 2, 4)), semanticLocations(org.json.JSONArray().put(direct).put(link)))
        assertTrue(semanticLocations(JSONObject().put("uri", "hick:///a.py")).isEmpty())
        assertEquals(11, SemanticQuery.entries.size)
        assertEquals("textDocument/prepareTypeHierarchy",SemanticQuery.Supertypes.method);assertEquals("typeHierarchy/supertypes",SemanticQuery.Supertypes.hierarchyFollowup);assertEquals("typeHierarchy/subtypes",SemanticQuery.Subtypes.hierarchyFollowup)
    }

    @Test fun agentMessagesUseLiterateBuffersWithoutFileOwnership() {
        // Contract: .hickory-source/frontends/contract.md
        val session = AgentSession()
        session.prompt = "# Draft\n\n**Hello**"
        val draft = session.draft
        session.provider = "claude-subscription"; session.model = "sonnet"; session.effort = "high"
        assertTrue(session.draft === draft)
        assertEquals("# Draft\n\n**Hello**", session.prompt)
        assertTrue(!draft.readOnly); assertEquals(null, draft.file.docID)
        val message = session.document("old-user", "# Previous question")
        assertTrue(message.readOnly); assertEquals(null, message.file.docID)
        assertTrue(session.document("old-user", "# Previous question") === message)
        assertEquals("# Previous question", message.text)
    }

    @Test fun emptyCSVAndFormattedMarkdownRetainExactEditRanges() {
        // Contract: .hickory-source/frontends/contract.md
        val empty = inlineTables(computeInlineMarkup("", "empty.csv")).single()
        assertEquals(0, empty.from); assertEquals(0, empty.to)
        val formatted = "<" + "!-- hickory-table-formats: B2:USD --" + ">\n| A | B |\n| --- | --- |\n| x | 2 |"
        val table = inlineTables(computeInlineMarkup(formatted, "table.md")).single()
        assertEquals(formatted, formatted.substring(table.from, table.to))
    }
    @Test fun structuralTablePresentationAndDeclaredFormulaLanguage() {
        // Contract: .hickory-source/frontends/contract.md
        val source = "<h:table language=\"python\" header=\"false\">\n1,=A1+1\n</h:table>\nAfter"
        val parsed = computeInlineMarkup(source, "table.md")
        val table = inlineTables(parsed).single()
        assertEquals("python", inlineTableAttribute("language", table, source))
        assertEquals("false", inlineTableAttribute("header", table, source))
        val changed = source.replace("1,=A1+1", "1,=A1+1\n2,=A2+1")
        val grid = inlineTables(tablePresentation(parsed, changed)).single()
        assertEquals("1,=A1+1\n2,=A2+1", changed.substring(grid.from, grid.to))
        assertTrue(changed.endsWith("</h:table>\nAfter"))
        val csv = "1,2\n"
        assertEquals(null, inlineTableAttribute("language", inlineTables(computeInlineMarkup(csv, "data.csv")).single(), csv))
    }

    @org.junit.Test fun expectationsShareTerminalPreviewWithoutScanningLiteralExamples() {
        val source = "<h:exec>\necho hi\n<h:diagram>inert</h:diagram>\n<h:expect>\nhi\n</h:expect>\n</h:exec>\n<h:expect/>\n<h:file path=\"example.md\">\n<h:expect>inert</h:expect>\n</h:file>"
        val blocks = computeInlineMarkup(source, "book.md").diagrams
        assertEquals(listOf("terminal", "terminal"), blocks.map { it.renderer })
        assertEquals("hi\n", blocks.first().body)
        assertEquals("", blocks.last().body)
        assertEquals("<h:expect>\nhi\n</h:expect>", source.substring(blocks.first().from, blocks.first().to))
    }

    @org.junit.Test fun previewWhitespaceScalesWithoutChangingSourceOffsets() {
        // Contract: .hickory-source/frontends/contract.md
        val source = "print(1)\nnext()\n"
        val normal = editorReplacements(emptyList(), emptyList(), source, outputs = mapOf(0 to "1\n"), fontScale = 1f)
        val reduced = editorReplacements(emptyList(), emptyList(), source, outputs = mapOf(0 to "1\n"), fontScale = .85f)
        val next = source.indexOf("next")
        assertTrue(editorDisplayOffset(next, reduced) > editorDisplayOffset(next, normal))
        for (projection in listOf(normal, reduced)) {
            val displayed = editorDisplay(source, projection)
            val at = editorDisplayOffset(next, projection)
            assertTrue(displayed.substring(at).startsWith("next()"))
            assertEquals(next, editorSourceOffset(at, projection))
        }
    }

    @org.junit.Test fun containerShellSkipsProse() {
        val source = "<h:container name=\"dev\">\n# setup\necho \"ready\"\n</h:container>"
        val snapshot = computeInlineMarkup(source, "workbook.md")
        assertTrue(snapshot.marks.any { it.kind == "syntax-comment" })
        assertTrue(snapshot.marks.any { it.kind == "syntax-string" })
        assertTrue(snapshot.marks.none { it.kind == "heading" })
    }

    @org.junit.Test fun recordedOutputUsesByteSpanAfterUnicodeBlock() {
        // Contract: .hickory-source/frontends/contract.md
        val source = "<h:exec>\necho café\n</h:exec>\n"
        val buffer = DocumentBuffer(FileNode("workbook.md", "workbook.md", false, emptyList(), null, null), FileText(source, "base"))
        buffer.receiptSource = source
        buffer.receipts = listOf(org.json.JSONObject().put("span", org.json.JSONArray(listOf(0, source.toByteArray(Charsets.UTF_8).size - 1))).put("transcript", org.json.JSONArray().put(org.json.JSONObject().put("kind", "out").put("data", "café\n"))))
        assertEquals(mapOf(2 to "café\n"), buffer.recordedOutputPlacements())
        assertEquals(source, buffer.text)
        buffer.state.edit { append("changed") }
        assertTrue(buffer.recordedOutputPlacements().isEmpty())
    }

    @org.junit.Test fun patchAndPathFenceOfferOnlyOwnedCodeCandidates() {
        // Contract: .hickory-source/frontends/contract.md
        val source = "<hick:project/>\n<hick:file id=\"base\" path=\"x.py\">\nprint(1)\n</hick:file>\n```patch id=\"change\" after=\"base\"\n--- a/x.py\n+++ b/x.py\n@@ -1 +1 @@\n-print(1)\n+print(\"café 😀\")\n```\n```python path=\"other.py\"\nprint(3)\n```\n```diff\n--- a/fake.py\n+++ b/fake.py\n@@ -1 +1 @@\n-a\n+b\n```"
        val snapshot = computeInlineMarkup(source, "workbook.md")
        assertEquals("x.py", snapshot.debug.launch[6])
        assertEquals("x.py", snapshot.debug.code[9])
        assertNull(snapshot.debug.code[8])
        assertNull(snapshot.debug.code[7])
        assertEquals("other.py", snapshot.debug.launch[11])
        assertEquals("other.py", snapshot.debug.code[12])
        assertNull(snapshot.debug.launch[16])
        assertNull(snapshot.debug.code[19])
        assertTrue(snapshot.marks.any { it.kind == "diff-add" })
        assertTrue(snapshot.marks.any { it.kind == "syntax-string" && source.substring(it.from, it.to) == "\"café 😀\"" })
        val headerLookingAddition = "```patch\n+++ b/x.py\n@@ -0,0 +1,1 @@\n+++ b/fake.py\n```"
        assertEquals("x.py", inlineDebugContext(headerLookingAddition, "p.md").code[3])
        assertNull(inlineDebugContext(headerLookingAddition, "p.md").launch[3])
    }

    @org.junit.Test fun fencedCodeHasLocalSyntaxWithoutInventedFileContext() {
        // Contract: .hickory-source/frontends/contract.md
        val source = "def is prose here\n```python\ndef coupon():\n    return \"café 😀\" # note\n```\n```unknown\ndef literal(): pass\n```"
        val snapshot = computeInlineMarkup(source, "workbook.md")
        val syntax = snapshot.marks.filter { it.kind.startsWith("syntax-") }
        assertEquals(listOf("def", "return"), syntax.filter { it.kind == "syntax-keyword" }.map { source.substring(it.from, it.to) })
        assertTrue(syntax.any { it.kind == "syntax-string" && source.substring(it.from, it.to) == "\"café 😀\"" })
        assertTrue(syntax.any { it.kind == "syntax-comment" })
        assertTrue(snapshot.debug.code.isEmpty())
        assertTrue(snapshot.debug.launch.isEmpty())
    }

    @org.junit.Test fun diagramDiscoveryKeepsLiteralExamplesAndUnicodeSafe() {
        // Contract: .hickory-source/frontends/contract.md
        val source = "😀\n```xml\n<h:diagram>ignored</h:diagram>\n```\n<h:file path=\"x.xml\">\n<h:diagram>also ignored</h:diagram>\n</h:file>\n<h:diagram renderer=\"graph\">{\"topology\":{\"nodes\":[]}}</h:diagram>\n<h:diagram>graph TD; A-->B</h:diagram>"
        val snapshot = computeInlineMarkup(source, "diagram.md")
        assertEquals(listOf("graph", "mermaid"), snapshot.diagrams.map { it.renderer })
        assertEquals(listOf(7, 8), snapshot.diagrams.map { it.line })
        val projected = tablePresentation(snapshot, "prefix\n" + source)
        assertEquals(2, projected.diagrams.size)
        assertEquals(snapshot.diagrams[0].from + 7, projected.diagrams[0].from)
        val replacements = editorReplacements(emptyList(), emptyList(), source, snapshot.diagrams)
        assertEquals(2, replacements.size)
        assertTrue(!editorDisplay(source, replacements).contains("graph TD"))
    }

    @org.junit.Test fun markdownHeadingFoldsRespectLiteralBodies() {
        // Contract: .hickory-source/frontends/contract.md
        val source = "# A\nintro 😀\n## B\nbody\n```python\n# comment\n```\n<h:file path=\"x.py\">\n# literal\n</h:file>\n## C\nlast\nNext\n====\nend"
        val folds = simpleFoldRanges(source, true)
        assertEquals(listOf(0, 2, 10, 12), folds.map { it.line })
        assertEquals(source.indexOf("Next") - 1, folds[0].to)
        assertTrue(source.substring(folds[1].from, folds[1].to).contains("# literal"))
        assertTrue(simpleFoldRanges("# comment\nbody").isEmpty())
        assertTrue(simpleFoldRanges("```python\n# comment\nbody", true).isEmpty())
    }

    @org.junit.Test fun tablePresentationSurvivesPendingParse() {
        // Contract: .hickory-source/frontends/contract.md
        val source = "Before\n\n| A | B |\n| --- | --- |\n| café 😀 | 2 |\n\nAfter"
        val parsed = computeInlineMarkup(source, "table.md")
        val table = inlineTables(parsed).single()
        for (changed in listOf("\n" + source, source + "\n", source.replace("| 2 |", "| 300 |"), source.replace("| 2 |", "|  |"))) {
            val pending = tablePresentation(parsed, changed)
            val grid = inlineTables(pending).single()
            assertEquals(table.displayRows, grid.displayRows)
            assertEquals(table.rows.size, grid.rows.size)
            assertEquals("café 😀", grid.rows[1][0].value)
            assertTrue(pending.debug.code.isEmpty())
        }
    }

    @Test fun sharedSmartEnterFixtures() {
        // Contract: .hickory-source/frontends/contract.md
        val loader = javaClass.classLoader!!
        val rules = SimpleEnterRules(loader.getResourceAsStream("enter-rules.json")!!.bufferedReader().use { it.readText() })
        val cases = JSONObject(loader.getResourceAsStream("enter-cases.json")!!.bufferedReader().use { it.readText() }).getJSONArray("cases")
        for (index in 0 until cases.length()) {
            val item = cases.getJSONObject(index); val before = item.getString("before"); val from = before.indexOf('¦'); val source = before.replace("¦", "")
            val plan = smartEnter(source, from, from, item.getString("path"), item.getString("unit"), item.getString("mode"), rules)
            val updated = source.replaceRange(plan.from, plan.to, plan.text)
            assertEquals(item.getString("name"), item.getString("after"), updated.substring(0, plan.caret) + "¦" + updated.substring(plan.caret))
        }
        val long = "x\n".repeat(32769) + "if ok:"
        assertEquals("\n", smartEnter(long, long.length, long.length, "x.py", "    ", "smart", rules).text)
    }

    @Test fun identifierNavigationAndWhitespace() {
        // Contract: .hickory-source/frontends/contract.md
        val source = "THIS_IS_ONE_WORD HTTPServer42 café😀"
        assertEquals(16, simpleWordOffset(source, 0, true, false))
        assertEquals(4, simpleWordOffset(source, 0, true, true))
        assertEquals(7, simpleWordOffset(source, 4, true, true))
        assertEquals(5, simpleWordOffset(source, 7, false, true))
        assertEquals(21, simpleWordOffset(source, 17, true, true))
        assertEquals(27, simpleWordOffset(source, 21, true, true))
        assertEquals(29, simpleWordOffset(source, 27, true, true))
        assertEquals(34, simpleWordOffset(source, 30, true, true))
        assertEquals(7, simpleWordOffset("E\u0301clairCase", 0, true, true))
        assertEquals(0, simpleWordOffset("E\u0301clairCase", 7, false, true))
        assertEquals(0, simpleWordOffset("a\r\nb", 3, false, true))
        assertEquals(0, simpleWordOffset("", 0, false, true))
        assertEquals("\r\n  ", simpleNewlineText("root\r\n  café", 12))
        assertEquals("\n ", simpleNewlineText("  value\nnext", 1))
        assertEquals(19, simpleBackspaceStart("root\n    child\n       ", 22))
        assertNull(simpleBackspaceStart("word ", 5))
        assertNull(simpleBackspaceStart("\t", 1))
    }

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

    @Test fun tableVerticalGoalSurvivesClampingUntilExplicitReset() {
        // Contract: .hickory-source/frontends/contract.md
        val goal = GridVerticalCaretGoal()
        assertEquals(80f, goal.position(80f)); assertEquals(80f, goal.position(6f)); assertEquals(80f, goal.position(60f))
        goal.reset(); assertEquals(6f, goal.position(6f))
        goal.reset(); assertEquals(0f, goal.position(0f)); assertEquals(0f, goal.position(30f))
    }
    @Test fun tableHorizontalArrowsLeaveInteriorAndSelectionsNative() {
        // Contract: .hickory-source/frontends/contract.md
        assertFalse(gridHorizontalExit(androidx.compose.ui.text.TextRange(2), 4, -1))
        assertFalse(gridHorizontalExit(androidx.compose.ui.text.TextRange(2), 4, 1))
        assertFalse(gridHorizontalExit(androidx.compose.ui.text.TextRange(0, 4), 4, -1))
        assertFalse(gridHorizontalExit(androidx.compose.ui.text.TextRange(0, 4), 4, 1))
        assertTrue(gridHorizontalExit(androidx.compose.ui.text.TextRange(0), 4, -1))
        assertTrue(gridHorizontalExit(androidx.compose.ui.text.TextRange(4), 4, 1))
        assertFalse(gridHorizontalExit(androidx.compose.ui.text.TextRange(0), 4, 1))
        assertFalse(gridHorizontalExit(androidx.compose.ui.text.TextRange(4), 4, -1))
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
    private suspend fun threadDebugger(protocol:Any?=3,adopt:Boolean=true,notices:Any?=null,block:suspend(NativeDebugger,MutableList<JSONObject>)->Unit)=kotlinx.coroutines.coroutineScope{
        val listener=com.sun.net.httpserver.HttpServer.create(java.net.InetSocketAddress("127.0.0.1",0),0)
        listener.createContext("/api/launch-profiles"){exchange->val payload=JSONObject("""{"profiles":[{"id":"python","name":"Python","can_debug":true,"program":"main.py"}],"fingerprint":"current"}""");if(protocol!=null)payload.put("debug_protocol",protocol);if(notices!=null)payload.put("notices",notices);val bytes=payload.toString().toByteArray();exchange.sendResponseHeaders(200,bytes.size.toLong());exchange.responseBody.use{it.write(bytes)}};listener.start()
        val debug=NativeDebugger(Engine("http://127.0.0.1:${listener.address.port}"),this);val requests=mutableListOf<JSONObject>();debug.emitForTest={requests.add(it)}
        try{debug.prepare(DocumentBuffer(file(),FileText("value = 1\n","saved")));if(adopt)debug.receive(JSONObject("""{"event":"started","doc":"hick:///main.py","session":"ours","breakpoints":[],"capabilities":{"frame_locals":true}}""").put("request_id",requests.last().getString("request_id")).put("source_hash",requests.last().getString("source_hash")));block(debug,requests)}finally{debug.close();listener.stop(0)}
    }
    private fun threadStop(stop:Long,thread:Long,frame:Long)=JSONObject("""{"event":"stopped","session":"ours","line":0,"frames":[{"name":"frame","source":"main.py","source_line":0}],"variables":[{"name":"value","value":"retained"}]}""").put("stop",stop).put("thread",thread).apply{getJSONArray("frames").getJSONObject(0).put("id",frame)}
    private fun threadReply(request:String,stop:Long,thread:Long)=JSONObject("""{"event":"threads","session":"ours","threads":[{"id":10,"name":"Main"},{"id":20,"name":"Worker"}]}""").put("stop",stop).put("selected",thread).put("request_id",request)
    @OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
    @Test fun resourceLineageRejectsIdenticalByteOccupantsAndRetainsExecutionEditor()=kotlinx.coroutines.runBlocking{
        val background=kotlinx.coroutines.CoroutineScope(kotlinx.coroutines.SupervisorJob()+kotlinx.coroutines.Dispatchers.Unconfined)
        try{threadDebugger{debug,requests->
            val workspace=Workspace(background);workspace.engine=Engine("http://127.0.0.1:1");workspace.debugger=debug
            val primary=debug.sourceEditor("main.py")!!;val text=primary.text;val peerFile=FileNode("peer.py","peer.py",false,emptyList(),null,null);val peer=DocumentBuffer(peerFile,FileText(text,"saved"));workspace.buffers["main.py"]=primary;workspace.buffers["peer.py"]=peer;workspace.rootFiles=listOf(file(),peerFile);workspace.show("main.py")
            primary.state.edit{append("# prior edit\n")};primary.state.undoState.undo();primary.state.edit{selection=androidx.compose.ui.text.TextRange(2)};primary.sourceScroll.scrollTo(7);val state=primary.state;assertTrue(state.undoState.canRedo)
            debug.receive(threadStop(1,10,7));workspace.revealDebug("main.py",0);val held=primary.debugRevealCurrent;assertTrue(held());assertTrue(debug.sourceMatches(primary));assertFalse(debug.sourceMatches(peer))
            fun source(id:String)=ResourceSource(id,text)
            fun adopt(patches:List<ResourceEditorPatch>,restoring:Map<String,DocumentBuffer> = emptyMap()){
                val expected=patches.associate{patch->patch.path to (patch.after?.let{ResourceSaved("file",it.text,debugHash(it.text),false,false,null)} ?: ResourceSaved("missing"))}
                val files=patches.mapNotNull{patch->patch.after?.let{ResourcePublishedFile(FileNode(patch.path,patch.path,false,emptyList(),null,null),expected.getValue(patch.path),"sha256:"+debugHash(it.text))}}
                val receipt=ResourcePublication(2,"request","workspace","context",true,"record",patches.map{it.path},expected,files,adoptionCurrent=true)
                workspace.adoptResourceReceipt(patches,receipt,"request","workspace","context",expected,workspace.captureResourceEditors(patches,restoring),restoring)
            }
            adopt(listOf(ResourceEditorPatch("main.py",source("a"),source("b")),ResourceEditorPatch("peer.py",source("b"),source("a"))))
            assertSame(primary,workspace.buffers["peer.py"]);assertSame(peer,workspace.buffers["main.py"]);assertFalse(held());assertFalse(debug.sourceMatches(primary));assertFalse(debug.sourceMatches(peer));assertTrue(debug.breakpointRows(peer).isEmpty())
            workspace.revealDebug("main.py",0);assertSame(primary,workspace.currentBuffer);assertFalse(primary.debugRevealCurrent());assertTrue(workspace.status.contains("unbound"));val before=requests.size;debug.toggle(peer,0,"value == 1");debug.toggle(primary,0,"value == 1");assertEquals(before,requests.size)
            assertSame(state,primary.state);assertEquals(2,state.selection.start);assertEquals(7,primary.sourceScroll.value);assertTrue(state.undoState.canRedo);assertEquals("main.py",debug.owner)
            adopt(listOf(ResourceEditorPatch("main.py",source("b"),source("a")),ResourceEditorPatch("peer.py",source("a"),source("b"))));assertTrue(debug.sourceMatches(primary));assertFalse(debug.sourceMatches(peer))
            adopt(listOf(ResourceEditorPatch("main.py",source("a"),null)));assertTrue(primary.resourceDeleted);assertFalse(debug.sourceMatches(primary));assertTrue(debug.hasInspection)
            adopt(listOf(ResourceEditorPatch("main.py",null,source("c"))));val independent=workspace.buffers.getValue("main.py");assertNotSame(primary,independent);assertFalse(debug.sourceMatches(independent));debug.select(debug.frames.first());workspace.revealDebug("main.py",0);assertSame(primary,workspace.currentBuffer);assertFalse(primary.debugRevealCurrent());val current=requests.size;debug.toggle(independent,0,"value == 1");assertEquals(current,requests.size)
            adopt(listOf(ResourceEditorPatch("main.py",source("c"),null)));adopt(listOf(ResourceEditorPatch("main.py",null,source("restored"))),mapOf("restored" to primary));assertSame(primary,workspace.buffers["main.py"]);assertTrue(debug.sourceMatches(primary));assertSame(state,primary.state);assertTrue(state.undoState.canRedo)
            val helper=DocumentBuffer(FileNode("helper.py","helper.py",false,emptyList(),null,null),FileText(text,"saved"));debug.frames.first().put("source","helper.py").put("source_hash",debugHash(text));debug.rememberSource(helper);assertFalse(debug.sourceMatches(helper));assertTrue(debug.framesCurrent);assertEquals("paused",debug.status);debug.stop();assertEquals("stop",requests.last().getString("op"))
        }}finally{background.coroutineContext[kotlinx.coroutines.Job]?.cancel()}
    }
    @Test fun catalogInputNoticesSurviveAutomaticLaunch()=kotlinx.coroutines.runBlocking{
        threadDebugger(notices=org.json.JSONArray().put("Skipped unsupported physical input.")){debug,_->assertEquals("running",debug.status);assertEquals(listOf("Skipped unsupported physical input."),debug.catalogNotices)}
        assertTrue(nativeCatalogNotices(JSONObject()).isEmpty());assertTrue(nativeCatalogNotices(JSONObject().put("notices",true)).single().contains("could not be read"));assertEquals(65,nativeCatalogNotices(JSONObject().put("notices",org.json.JSONArray((0..64).map{"notice"}))).size)
    }
    @Test fun incompletePauseNeverGrantsEvaluationOrMovementAuthority()=kotlinx.coroutines.runBlocking{
        threadDebugger{debug,requests->
            debug.receive(threadStop(1,10,100).apply{remove("stop")});assertFalse(debug.framesCurrent);assertNull(debug.line("main.py"));val count=requests.size;debug.evaluate("value");debug.step("over");assertEquals(count,requests.size)
            debug.receive(JSONObject().put("event","value").put("session","ours").put("expression","value").put("value","unbound"));assertTrue(debug.values.isEmpty());debug.stop();assertEquals("stop",requests.last().getString("op"))
        }
    }
    @Test fun recursiveValuesRejectReusedReferencesAndClosedInspectorReplies(){
        val tree=NativeVariableTree();val scope=NativeDebugScope("session","5",-1,0);val token=tree.open()
        fun rows()=listOf(JSONObject().put("name","root").put("value","dict").put("variables_reference",1))
        fun reply(request:NativeVariableRequest,reference:Int=1,empty:Boolean=false)=scope.put(JSONObject().put("event","children").put("request_id",request.request).put("reference",reference).put("variables",org.json.JSONArray(if(empty)emptyList<JSONObject>() else listOf(JSONObject().put("name","nested").put("value","list").put("variables_reference",2)))))
        tree.replace(scope,"locals",rows());val old=tree.groups.getValue("locals").single();val held=tree.expand(old,token)!!;assertNotNull(tree.expansions[old.id]!!.pending)
        tree.replace(scope,"locals",rows());val current=tree.groups.getValue("locals").single();assertNotEquals(old.id,current.id);assertFalse(tree.admit(reply(held)));assertNull(tree.expand(old,token))
        val request=tree.expand(current,token)!!;assertFalse(tree.admit(reply(request,9)));assertFalse(tree.admit(reply(request).put("frame",1)));assertTrue(tree.admit(reply(request)));val nested=tree.expansions[current.id]!!.children.single();val child=tree.expand(nested,token)!!
        tree.collapse(current,token);assertFalse(tree.admit(reply(child,2)));assertNull(tree.expand(nested,token))
        val empty=tree.expand(current,token)!!;assertTrue(tree.admit(reply(empty,empty=true)));assertEquals("No reported children.",tree.expansions[current.id]!!.message)
        val failed=tree.expand(current,token)!!;assertTrue(tree.refuse(failed.request,"provider refused"));assertEquals("provider refused",tree.expansions[current.id]!!.message)
        val closing=tree.expand(current,token)!!;tree.close(token);assertFalse(tree.admit(reply(closing)));val reopened=tree.open();assertNull(tree.expand(current,token));assertNotNull(tree.expand(current,reopened))
        tree.replace(scope.copy(stop="6"),"locals",rows());assertNull(tree.expand(current,reopened));tree.clear();assertTrue(tree.groups.isEmpty())
    }
    @Test fun breakpointReceiptsRequireLatestSourceAndConditionIntent(){
        val path="literal %2F λ?#.py";val uri=languageURI(path);assertEquals(path,SemanticLocation(uri,0,0).path);assertTrue(uri.contains("%252F"));assertTrue(uri.contains("%3F%23"))
        val source=NativeSourceSnapshot("editor",path,"first\nsecond\n",true,true);val model=NativeBreakpointModel();val hash=debugHash(source.text)
        val initial=model.toggle(source,0,"value > 0");model.stamp(source.identity,initial.revision,hash);model.request(source,"session","old")
        val revised=model.toggle(source,0,"value > 1");model.stamp(source.identity,revised.revision,hash);model.request(source,"session","new")
        fun receipt(vararg rows:JSONObject)=JSONObject().put("breakpoints",org.json.JSONArray(rows.toList()))
        val bound=receipt(JSONObject().put("line",0).put("moved_to",1).put("state","bound"))
        assertFalse(model.admit(source,bound,"session","old",hash));assertFalse(model.admit(source.copy(text="edited"),bound,"session","new",hash));assertFalse(model.admit(source.copy(saved=false),bound,"session","new",hash));assertTrue(model.rows(source.copy(text="edited"),"session",hash).isEmpty())
        assertFalse(model.admit(source,receipt(JSONObject().put("line",1)),"session","new",hash));assertFalse(model.admit(source,receipt(JSONObject().put("line",0),JSONObject().put("line",0)),"session","new",hash));assertFalse(model.admit(source,receipt(JSONObject().put("line",0).put("moved_to",999)),"session","new",hash))
        assertTrue(model.admit(source,bound,"session","new",hash));assertEquals("value > 1",model.rows(source,"session",hash).single().getString("condition"));assertEquals(1,model.rows(source,"session",hash).single().getInt("line"));assertFalse(model.admit(source,bound,"session","new",hash))
        model.request(source,"session","refused");assertFalse(model.refuse("other","refused","wrong session"));assertFalse(model.refuse("session","old","held refusal"));assertTrue(model.refuse("session","refused","provider refused"));assertEquals("refused",model.rows(source,"session",hash).single().getString("state"));assertTrue(model.notice(source,"session",hash).contains("provider refused"));assertFalse(model.refuse("session","refused","duplicate"))
        model.readdress(source.copy(path="renamed.py"));assertTrue(model.rows(source,"session",hash).isEmpty());assertTrue(model.receipts.isEmpty())
    }
    @Test fun breakpointFailureRetiresAllBoundAndPendingAuthorityWithoutLosingPause()=kotlinx.coroutines.runBlocking{
        threadDebugger{debug,_->
            debug.receive(threadStop(5,10,100));val frames=debug.frames;val model=debug.breakpointModel
            val sources=(1..3).map{NativeSourceSnapshot("editor-$it","source-$it.py","value = 1\n",true,true)};val hash=debugHash(sources.first().text)
            fun bound()=JSONObject().put("breakpoints",org.json.JSONArray().put(JSONObject().put("line",0).put("state","bound")))
            for((index,source) in sources.withIndex()){val intent=model.toggle(source,0,"value == 1");model.stamp(source.identity,intent.revision,hash);model.request(source,"ours","bound-$index");assertTrue(model.admit(source,bound(),"ours","bound-$index",hash))}
            val first=sources[0];val changed=model.toggle(first,0,"value == 2");model.stamp(first.identity,changed.revision,hash);model.request(first,"ours","failed-update");model.request(sources[1],"ours","held-other-source");assertEquals("bound",model.rows(sources[2],"ours",hash).single().getString("state"))
            debug.receive(JSONObject().put("event","failed").put("session","ours").put("request_id","failed-update").put("message","Adapter rejected the replacement set"))
            assertEquals("paused",debug.status);assertEquals("ours",debug.sessionID);assertSame(frames,debug.frames);assertTrue(debug.framesCurrent);assertEquals(3,model.intents.size);assertEquals("value == 2",model.intents.getValue(first.identity).marks.single().getString("condition"))
            assertTrue(model.receipts.values.flatMap{it.marks}.none{it.optString("state")=="bound"});for(source in sources)assertTrue(model.notice(source,"ours",hash).contains("Adapter rejected"))
            assertFalse(model.admit(sources[1],bound(),"ours","held-other-source",hash));assertFalse(model.admit(sources[2],bound(),"ours","",hash));assertFalse(model.admit(sources[2],bound(),"ours","bound-2",hash));assertTrue(model.rows(sources[2],"ours",hash).all{it.optString("state")!="bound"})
            model.request(sources[2],"ours","explicit-fresh");assertTrue(model.admit(sources[2],bound(),"ours","explicit-fresh",hash));assertEquals("bound",model.rows(sources[2],"ours",hash).single().getString("state"));assertFalse(model.refuse("ours","failed-update","duplicate failure"))
        }
    }
    @Test fun movementFailureDistinguishesRefusalFromRetiredPause()=kotlinx.coroutines.runBlocking{
        threadDebugger{debug,requests->
            debug.receive(threadStop(5,10,100));val edited=DocumentBuffer(file(),FileText("value = 1\n","saved"));edited.state.edit{append("# draft\n")};val before=requests.size;debug.moveTo(edited,0);assertEquals(before,requests.size);assertEquals("paused",debug.status);debug.step("over");val first=requests.last();assertEquals(5,first.getInt("stop"));assertEquals(10,first.getInt("thread"));assertEquals(100,first.getInt("frame"))
            debug.receive(JSONObject().put("event","failed").put("session","ours").put("request_id",first.getString("request_id")).put("scope_retired",false).put("message","provider refused"));assertEquals("paused",debug.status);assertTrue(debug.framesCurrent)
            debug.step("over");val next=requests.last();debug.receive(JSONObject().put("event","failed").put("session","ours").put("request_id",first.getString("request_id")).put("message","held refusal"));assertEquals("running",debug.status)
            debug.receive(JSONObject().put("event","failed").put("session","ours").put("request_id",next.getString("request_id")).put("scope_retired",true).put("message","stop read failed"));assertEquals("running",debug.status);assertFalse(debug.framesCurrent);assertTrue(debug.frames.isNotEmpty());val count=requests.size;debug.evaluate("value");debug.step("over");assertEquals(count,requests.size);debug.stop();assertEquals("stop",requests.last().getString("op"))
        }
    }
    @Test fun unsupportedDebuggerCatalogNeverStartsOrChangesTheEditor()=kotlinx.coroutines.runBlocking{
        for(protocol in listOf<Any?>(null,1,2,true,"3",3.5))threadDebugger(protocol=protocol,adopt=false){debug,requests->
            val buffer=DocumentBuffer(file(),FileText("value = 1\n","saved"));buffer.state.edit{selection=androidx.compose.ui.text.TextRange(2)};buffer.sourceScroll.scrollTo(7);val state=buffer.state
            debug.prepare(buffer);assertTrue(requests.isEmpty());assertNull(debug.sessionID);assertFalse(debug.active);assertTrue(debug.message.contains("source-bound debugger protocol 3"));assertTrue(debug.hasInspection);assertSame(state,buffer.state);assertEquals("value = 1\n",buffer.text);assertEquals(2,buffer.state.selection.start);assertEquals(7,buffer.sourceScroll.value);assertFalse(buffer.dirty)
        }
    }
    @Test fun startupRepliesRequireExactLaunchAndRetiredCleanupNeverStopsForeignSessions()=kotlinx.coroutines.runBlocking{
        threadDebugger{debug,requests->
            val old=requests.first{it.optString("op")=="start"}.getString("request_id");val buffer=DocumentBuffer(file(),FileText("value = 1\n","saved"));debug.close();debug.prepare(buffer);val current=requests.last().getString("request_id");assertNotEquals(old,current)
            fun started(id:String,session:String)=JSONObject().put("event","started").put("doc","hick:///main.py").put("request_id",id).put("session",session).put("source_hash",debugHash("value = 1\n")).put("breakpoints",org.json.JSONArray())
            val count=requests.size;debug.receive(started("foreign","unrelated"));assertEquals(count,requests.size);assertNull(debug.sessionID)
            debug.receive(started(old,"retired"));assertEquals("stop",requests.last().getString("op"));assertEquals("retired",requests.last().getString("session"));assertEquals("starting",debug.status);assertNull(debug.sessionID);val cleaned=requests.size;debug.receive(started(old,"retired"));assertEquals(cleaned,requests.size)
            debug.receive(JSONObject().put("event","build").put("doc","hick:///main.py").put("request_id",old).put("events",org.json.JSONArray().put(JSONObject().put("data","old build"))));assertFalse(debug.message.contains("old build"))
            debug.receive(JSONObject().put("event","failed").put("doc","hick:///main.py").put("session",JSONObject.NULL).put("request_id",old).put("message","old failure"));assertEquals("starting",debug.status)
            debug.receive(JSONObject().put("event","build").put("doc","hick:///main.py").put("request_id",current).put("events",org.json.JSONArray().put(JSONObject().put("data","current build"))));assertTrue(debug.message.contains("current build"));debug.receive(started(current,"new"));assertEquals("new",debug.sessionID);assertEquals("running",debug.status)
            repeat(65){debug.close();debug.prepare(buffer)};val bounded=requests.size;debug.receive(started(current,"evicted"));assertEquals(bounded,requests.size);assertNull(debug.sessionID)
            val pending=requests.last().getString("request_id");debug.receive(JSONObject().put("event","failed").put("doc","hick:///main.py").put("session",JSONObject.NULL).put("request_id",pending).put("message","Current launch failed"));assertEquals("failed",debug.status);assertTrue(debug.hasInspection);debug.receive(started(pending,"failed-owned"));assertEquals("stop",requests.last().getString("op"));assertEquals("failed-owned",requests.last().getString("session"));assertEquals("failed",debug.status)
        }
    }
    @Test fun threadListsAndSelectionRetainFramesAndRejectObsoleteObservations()=kotlinx.coroutines.runBlocking{
        threadDebugger{debug,requests->
            debug.receive(threadStop(1,10,7));val frames=debug.frames;val variables=debug.variables;val first=requests.last().getString("request_id")
            debug.receive(threadReply("old-$first",1,10));assertNull(debug.threadReading)
            debug.receive(threadReply(first,0,10));assertNull(debug.threadReading);assertSame(frames,debug.frames);assertSame(variables,debug.variables)
            debug.readThreads();debug.receive(threadReply(requests.last().getString("request_id"),1,10));val observed=debug.threadReading!!
            debug.selectThread(observed,20);val selection=requests.last();assertEquals("select_thread",selection.getString("op"));assertEquals(1,selection.getLong("stop"));assertSame(frames,debug.frames);assertSame(variables,debug.variables)
            val selecting=requests.size;debug.step("over");debug.select(frames.first());debug.evaluate("value");debug.readThreads();assertEquals(selecting,requests.size);assertEquals("paused",debug.status);assertSame(frames,debug.frames)
            debug.receive(JSONObject().put("event","failed").put("session","ours").put("request_id",selection.getString("request_id")).put("message","Thread is running"));assertEquals("paused",debug.status);assertEquals("ours",debug.sessionID);assertSame(frames,debug.frames);assertSame(observed,debug.threadReading);assertFalse(debug.threadBusy)
            debug.readThreads();debug.receive(threadReply(requests.last().getString("request_id"),1,10));val fresh=debug.threadReading!!;val count=requests.size;debug.selectThread(observed,20);assertEquals(count,requests.size)
            debug.selectThread(fresh,20);val selectedRequest=requests.last().getString("request_id");debug.receive(threadStop(2,20,8));assertTrue(debug.threadBusy);assertNull(debug.threadReading);assertEquals(8,debug.frames.first().getLong("id"));debug.receive(threadReply(selectedRequest,2,20));assertEquals(20,debug.threadReading!!.selected);assertFalse(debug.threadBusy)
            val selectedFrames=debug.frames;val selectedReading=debug.threadReading;debug.readThreads();val refresh=requests.last().getString("request_id");debug.receive(JSONObject().put("event","failed").put("session","ours").put("request_id","foreign").put("message","Old failure"));assertTrue(debug.threadBusy)
            debug.receive(JSONObject().put("event","failed").put("session","ours").put("request_id",refresh).put("message","Read failed"));assertFalse(debug.threadBusy);assertSame(selectedFrames,debug.frames);assertSame(selectedReading,debug.threadReading)
            debug.step("over");debug.receive(threadReply(refresh,2,20));assertNull(debug.threadReading);debug.receive(threadStop(2,20,8));assertEquals("running",debug.status);debug.receive(threadStop(4,10,9));assertEquals("paused",debug.status);assertEquals(9,debug.frames.first().getLong("id"))
            val optional=requests.last().getString("request_id");val beforeStep=requests.size;assertTrue(debug.threadBusy);debug.step("over");assertEquals(beforeStep+1,requests.size);assertEquals("running",debug.status);debug.receive(threadReply(optional,4,10));assertNull(debug.threadReading)
        }
    }
    @Test fun evaluationRepliesRequireLatestRequestAndExactStopThreadFrame()=kotlinx.coroutines.runBlocking{
        threadDebugger{debug,requests->
            debug.receive(threadStop(1,10,7));debug.receive(threadReply(requests.last().getString("request_id"),1,10));val oldFrame=debug.frames.first();debug.evaluate("value");val old=requests.last().getString("request_id");debug.evaluate("value");val latest=requests.last().getString("request_id")
            fun value(request:String,stop:Long,thread:Long,frame:Long,text:String)=JSONObject().put("event","value").put("session","ours").put("request_id",request).put("stop",stop).put("thread",thread).put("frame",frame).put("expression","value").put("value",text)
            debug.receive(value(old,1,10,7,"old"));assertTrue(debug.values.isEmpty());debug.receive(value(latest,1,10,7,"current"));assertEquals("current",debug.values["value"])
            debug.step("over");debug.receive(threadStop(3,20,8));debug.receive(threadReply(requests.last().getString("request_id"),3,20));debug.evaluate("value");val current=requests.last();assertEquals(3,current.getLong("stop"));assertEquals(20,current.getLong("thread"));assertEquals(8,current.getLong("frame"))
            debug.receive(value(latest,1,10,7,"old pause"));debug.receive(value(current.getString("request_id"),3,10,8,"wrong thread"));debug.receive(value(current.getString("request_id"),3,20,7,"wrong frame"));assertTrue(debug.values.isEmpty())
            debug.receive(value(current.getString("request_id"),3,20,8,"new pause"));assertEquals("new pause",debug.values["value"]);val count=requests.size;debug.select(oldFrame);assertEquals(count,requests.size);assertEquals("paused",debug.status)
            val frames=debug.frames;val selector=debug.threadReading!!;debug.evaluate("unsupported");debug.receive(JSONObject().put("event","failed").put("session","ours").put("request_id",requests.last().getString("request_id")).put("message","Adapter cannot evaluate this suspended frame"));assertEquals("paused",debug.status);assertEquals("ours",debug.sessionID);assertSame(frames,debug.frames);assertSame(selector,debug.threadReading);debug.selectThread(selector,10);assertEquals("select_thread",requests.last().getString("op"));assertEquals(10,requests.last().getLong("thread"))
        }
    }
    @Test fun threadDecoderRejectsMalformedRowsAndPreservesUnsignedStopIdentity(){
        val stop="18446744073709551615";val event=threadReply("request",1,10).put("stop",java.math.BigInteger(stop));val result=nativeThreadReading(event,"ours",stop,10,"request");assertEquals(stop,result.stop);assertEquals(listOf(10L,20L),result.threads.map{it.id})
        fun refused(raw:JSONObject){assertTrue(runCatching{nativeThreadReading(raw,"ours",stop,10,"request")}.isFailure)}
        refused(JSONObject(event.toString()).put("stop",1.0));refused(JSONObject(event.toString()).put("request_id","older"));refused(JSONObject(event.toString()).put("selected",20))
        refused(JSONObject(event.toString()).apply{getJSONArray("threads").put(JSONObject().put("id",10).put("name","duplicate"))})
        refused(JSONObject(event.toString()).apply{getJSONArray("threads").getJSONObject(0).put("name","x".repeat(4097))})
        refused(JSONObject(event.toString()).put("threads",org.json.JSONArray()))
        for(id in listOf<Any>(true,1.5,2147483648L,-2147483649L))refused(JSONObject(event.toString()).apply{getJSONArray("threads").getJSONObject(0).put("id",id)})
    }
    @Test fun zeroAndNegativeThreadFrameIdsRemainScopedIdentities()=kotlinx.coroutines.runBlocking{
        threadDebugger{debug,requests->
            fun reply(request:String,stop:Long,selected:Long)=threadReply(request,stop,selected).put("threads",org.json.JSONArray().put(JSONObject().put("id",0).put("name","Zero")).put(JSONObject().put("id",-7).put("name","Negative")))
            debug.receive(threadStop(1,0,-1));debug.receive(reply(requests.last().getString("request_id"),1,0));val reading=debug.threadReading!!;assertEquals(0,reading.selected);debug.selectThread(reading,-7);val selected=requests.last();assertEquals(-7,selected.getLong("thread"));debug.receive(threadStop(2,-7,-2));debug.receive(reply(selected.getString("request_id"),2,-7));assertEquals(-7,debug.threadReading!!.selected)
            debug.evaluate("signed");val request=requests.last();assertEquals(-2,request.getLong("frame"));assertEquals(-7,request.getLong("thread"));debug.receive(JSONObject().put("event","value").put("session","ours").put("request_id",request.getString("request_id")).put("stop",2).put("thread",-7).put("frame",-2).put("expression","signed").put("value","valid"));assertEquals("valid",debug.values["signed"])
        }
    }
    @Test fun frameLocalsRequireLatestScopeAndStackRefusalsRetainReadableHistory()=kotlinx.coroutines.runBlocking{
        threadDebugger{debug,requests->
            val stopped=threadStop(1,10,7);stopped.getJSONArray("frames").put(JSONObject().put("id",8).put("name","caller").put("source","main.py").put("source_line",0));debug.receive(stopped)
            val caller=debug.frames[1];var invalidations=0;androidx.compose.runtime.snapshots.Snapshot.sendApplyNotifications();val observer=androidx.compose.runtime.snapshots.SnapshotStateObserver{it()};observer.start()
            try{observer.observeReads("selected-frame",{_:String->invalidations++}){debug.line("main.py")};debug.select(caller);androidx.compose.runtime.snapshots.Snapshot.sendApplyNotifications();assertTrue("Frame selection must invalidate the gutter before locals arrive",invalidations>0)}finally{observer.stop()}
            val request=requests.last();assertEquals("state",request.getString("op"));assertEquals(8,request.getLong("frame"));assertTrue(debug.variables.isEmpty())
            fun locals(id:String,stop:Long,thread:Long,frame:Long)=JSONObject().put("event","frame_variables").put("session","ours").put("request_id",id).put("stop",stop).put("thread",thread).put("frame",frame).put("variables",org.json.JSONArray().put(JSONObject().put("name","caller_value").put("value","41")))
            val id=request.getString("request_id");debug.receive(locals(id,1,10,7));debug.receive(locals("old-$id",1,10,8));assertTrue(debug.variables.isEmpty());debug.receive(locals(id,1,10,8));assertEquals("caller_value",debug.variables.single().getString("name"))
            val frames=debug.frames;val variables=debug.variables;debug.readFrameVariables();debug.receive(JSONObject().put("event","failed").put("session","ours").put("request_id",requests.last().getString("request_id")).put("message","Scopes unavailable"));assertSame(frames,debug.frames);assertSame(variables,debug.variables);assertEquals("paused",debug.status);assertTrue(debug.frameNotice.contains("Scopes unavailable"))
            debug.step("over");debug.receive(threadStop(3,20,9).put("stack_error","Adapter stack unavailable").put("variables_error","Scopes unavailable"));assertSame(frames,debug.frames);assertFalse(debug.framesCurrent);assertNull(debug.line("main.py"));assertTrue(debug.variables.isEmpty());assertTrue(debug.frameNotice.contains("Adapter stack unavailable"));val count=requests.size;debug.select(caller);debug.evaluate("caller_value");assertEquals(count,requests.size)
        }
    }
    @Test fun nativeDebugSessionAndPendingStop() = kotlinx.coroutines.runBlocking {
        val listener = com.sun.net.httpserver.HttpServer.create(java.net.InetSocketAddress("127.0.0.1", 0), 0)
        var captured = JSONObject()
        listener.createContext("/api/launch-profiles") { exchange ->
            captured = JSONObject(exchange.requestBody.bufferedReader().readText())
            val bytes = """{"debug_protocol":3,"profiles":[{"id":"other","name":"Other","can_debug":true,"program":"other.py"},{"id":"python","name":"Python","can_debug":true,"program":"main.py"}],"fingerprint":"current"}""".toByteArray()
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
            debug.toggle(buffer, 0, "value > 0")
            debug.prepare(buffer); assertEquals(buffer.text, captured.getString("expected_source"))
            assertEquals("starting", debug.status); assertTrue(debug.profiles.isEmpty());assertTrue(debug.hasInspection)
            assertEquals("python", requests.single().getJSONObject("profile").getString("id"))
            val launchRequest=requests.last().getString("request_id")
            debug.receive(JSONObject("""{"event":"started","doc":"hick:///other.py","session":"wrong"}""").put("request_id",launchRequest)); assertNull(debug.sessionID)
            debug.receive(JSONObject("""{"event":"started","doc":"hick:///main.py","session":"ours","breakpoints":[{"line":0,"moved_to":1,"state":"bound"}]}""").put("request_id",launchRequest).put("source_hash",debugHash(buffer.text)))
            assertEquals("value > 0", debug.breakpoints["main.py"]!!.first().getString("condition"))
            assertEquals(1, debug.breakpoints["main.py"]!!.first().getInt("line"))
            debug.receive(JSONObject("""{"event":"stopped","session":"old","line":0}""")); assertEquals("running", debug.status)
            var revealed = ""; debug.onLocation = { path, line -> revealed = "$path:$line" }
            debug.receive(JSONObject("""{"event":"stopped","session":"ours","stop":1,"thread":10,"line":null,"frames":[{"id":7,"name":"helper","source":"helper.py","source_line":2}],"variables":[{"name":"value","value":"1"}]}"""))
            assertTrue(debug.hasInspection)
            assertEquals("helper.py:2", revealed); assertEquals(2, debug.line("helper.py"))
            val helper = DocumentBuffer(FileNode("helper.py", "helper.py", false, emptyList(), null, null), FileText("before", "base"))
            debug.rememberSource(helper); assertFalse(debug.sourceMatches(helper))
            debug.frames.first().put("source_hash",java.security.MessageDigest.getInstance("SHA-256").digest(helper.text.toByteArray(Charsets.UTF_8)).joinToString(""){"%02x".format(it)})
            debug.rememberSource(helper);assertTrue(debug.sourceMatches(helper))
            helper.state.edit { replace(0, length, "saved replacement") }; helper.base = helper.text
            assertFalse(debug.sourceMatches(helper))
            debug.evaluate("value", true); assertEquals(7, requests.last().getInt("frame"))
            debug.step("over"); assertEquals("over", requests.last().getString("how")); assertTrue(debug.variables.isEmpty())
            debug.receive(JSONObject("""{"event":"value","session":"ours","expression":"value","value":"old"}""")); assertTrue(debug.values.isEmpty())
            debug.close(); debug.prepare(buffer);val pendingLaunch=requests.last().getString("request_id"); requests.clear(); debug.stop()
            assertEquals("stopping", debug.status); assertTrue(requests.isEmpty())
            debug.receive(JSONObject("""{"event":"started","doc":"hick:///main.py","session":"late","breakpoints":[]}""").put("request_id",pendingLaunch).put("source_hash",debugHash(buffer.text)))
            assertEquals("stop", requests.last().getString("op")); assertEquals("late", requests.last().getString("session"))
            buffer.state.edit { append("edited") }; debug.close(); debug.prepare(buffer)
            assertTrue(debug.message.contains("Save this file")); assertFalse(debug.sourceMatches(buffer));assertTrue(debug.hasInspection)
            val managed = DocumentBuffer(FileNode("source.md", "source.md", false, emptyList(), "managed", null), FileText("<h:file path=\"main.py\">print(1)</h:file>", "base"))
            debug.prepare(managed)
            assertEquals("main.py", captured.getString("program")); assertEquals(managed.text, captured.getString("expected_document")); assertFalse(captured.has("expected_source"))
        } finally { debug.close(); listener.stop(0) }
    }
    @Test fun debugRevealRetainsEditorsAndRequiresCapturedSourceAndPauseIdentity()=kotlinx.coroutines.runBlocking{
        threadDebugger{debug,_->
            val workspace=Workspace(this);workspace.engine=Engine("http://127.0.0.1:1");workspace.debugger=debug
            val primary=debug.sourceEditor("main.py")!!;val helperFile=FileNode("helper.py","helper.py",false,emptyList(),null,null);val helper=DocumentBuffer(helperFile,FileText("first\nsecond\nthird\n","saved"));workspace.buffers["main.py"]=primary;workspace.buffers["helper.py"]=helper;workspace.rootFiles=listOf(file(),helperFile);workspace.files=emptyList();workspace.directoryScope="elsewhere";workspace.show("main.py")
            debug.receive(threadStop(1,10,7));workspace.revealDebug("main.py",0);assertSame(primary,workspace.currentBuffer);assertNotNull(primary.debugReveal);val held=primary.debugRevealCurrent;assertTrue(held());debug.step("over");assertFalse(held())
            val stopped=threadStop(3,20,9);stopped.getJSONArray("frames").getJSONObject(0).put("source","helper.py").put("source_line",2);debug.receive(stopped);workspace.revealDebug("helper.py",2)
            assertSame(helper,workspace.currentBuffer);assertNull(helper.debugReveal);assertTrue(workspace.status.contains("unbound"));assertEquals("elsewhere",workspace.directoryScope)
            debug.frames.first().put("source_hash",java.security.MessageDigest.getInstance("SHA-256").digest(helper.text.toByteArray(Charsets.UTF_8)).joinToString(""){"%02x".format(it)});workspace.revealDebug("helper.py",2);assertNotNull(helper.debugReveal);assertTrue(helper.debugRevealCurrent())
            debug.frames.first().put("source_reference",1);assertTrue(debug.sourceMatches(helper));assertFalse(debug.positionSourceMatches(helper));workspace.revealDebug("helper.py",2);assertTrue(workspace.status.contains("Adapter source"));assertFalse(helper.debugRevealCurrent());debug.frames.first().put("source_reference",0)
            workspace.revealDebug("helper.py",2);helper.state.edit{selection=androidx.compose.ui.text.TextRange(1)};assertFalse(helper.debugRevealCurrent());workspace.revealDebug("helper.py",2);assertTrue(helper.debugRevealCurrent());helper.sourceScroll.scrollTo(12);assertFalse(helper.debugRevealCurrent());workspace.revealDebug("helper.py",2);helper.state.edit{replace(0,length,"independent draft")};assertFalse(helper.debugRevealCurrent());assertSame(helper,workspace.currentBuffer)
        }
    }
    @Test(timeout=150000) fun liveNativeThreadChannel()=kotlinx.coroutines.runBlocking{
        val origin=System.getenv("HICKORY_NATIVE_DEBUG_ORIGIN");org.junit.Assume.assumeNotNull(origin)
        val engine=Engine(origin!!);val file=FileNode("debug.py","debug.py",false,emptyList(),null,null);val loaded=engine.file(file);val buffer=DocumentBuffer(file,loaded);val state=buffer.state;val debug=NativeDebugger(engine,this)
        try{
            val line=loaded.content.lines().indexOfFirst{"# hickory-thread-breakpoint" in it};check(line>=0){"The live native thread fixture needs the shared breakpoint marker in debug.py."}
            debug.toggle(buffer,line);debug.prepare(buffer)
            suspend fun await(label:String,condition:()->Boolean){kotlinx.coroutines.withTimeout(45000){while(!condition()){check(debug.status!="failed"){"$label: ${debug.message}"};kotlinx.coroutines.delay(25)}}}
            await("Initial paused thread list"){debug.status=="paused"&&debug.threadReading!=null&&!debug.threadBusy}
            val first=debug.threadReading!!;val main=first.selected;val worker=first.threads.firstOrNull{it.name.contains("hickory-worker")} ?: error("The fixture did not report its named active worker: ${first.threads}")
            debug.selectThread(first,worker.id);await("Worker selection"){debug.threadReading?.selected==worker.id&&!debug.threadBusy&&debug.framesCurrent}
            val frame=debug.frames.firstOrNull{it.optString("name").contains("worker")} ?: debug.frames.first();debug.select(frame)
            await("Selected worker frame locals"){debug.variables.any{it.optString("name")=="worker_value"&&it.optString("value")=="41"}&&debug.frameNotice.contains("reported locals for this frame.")}
            debug.evaluate("worker_value");await("Worker value"){debug.values["worker_value"]=="41"}
            val inspection=debug.variableTree.open();val nested=debug.variableTree.groups["locals"].orEmpty().firstOrNull{it.name=="worker_nested"} ?: error("The v3 fixture must report worker_nested")
            debug.expandVariable(nested,inspection);await("Reported nested dictionary children"){debug.variableTree.expansions[nested.id]?.children?.isNotEmpty()==true}
            val items=debug.variableTree.expansions.getValue(nested.id).children.firstOrNull{it.name.trim('\'','"')=="items"} ?: error("Dictionary children: ${debug.variableTree.expansions.getValue(nested.id).children.map{it.name}}");debug.expandVariable(items,inspection);await("Reported nested list children"){debug.variableTree.expansions[items.id]?.children?.isNotEmpty()==true}
            val item=debug.variableTree.expansions.getValue(items.id).children.firstOrNull{it.name in listOf("0","[0]")} ?: error("List children: ${debug.variableTree.expansions.getValue(items.id).children.map{it.name}}");debug.expandVariable(item,inspection);await("Reported recursive dictionary value"){debug.variableTree.expansions[item.id]?.children?.any{it.name.trim('\'','"')=="value"&&it.value=="41"}==true};debug.variableTree.close(inspection)

            val frames=debug.frames;val selector=debug.threadReading!!;val previous=debug.message;debug.evaluate("__hickory_missing_value__")
            await("Explicit unavailable expression"){debug.message!=previous&&debug.message.contains("Choose another current frame/thread")};assertEquals("paused",debug.status);assertSame(frames,debug.frames);assertSame(selector,debug.threadReading)
            debug.selectThread(selector,main);await("Return to main thread"){debug.threadReading?.selected==main&&!debug.threadBusy&&debug.framesCurrent};debug.evaluate("main_value");await("Main value"){debug.values["main_value"]=="7"}
            assertSame(state,buffer.state);assertEquals(loaded.content,buffer.text);assertEquals(loaded.content,engine.file(file).content);assertFalse(buffer.dirty)
            debug.stop();await("Stopped session"){!debug.active}
        }finally{if(debug.active)debug.stop();debug.close()}
    }
    @OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
    @Test(timeout=180000) fun livePausedResourceLineageChannel()=kotlinx.coroutines.runBlocking{
        val origin=System.getenv("HICKORY_NATIVE_DEBUG_ORIGIN");org.junit.Assume.assumeNotNull(origin)
        val background=kotlinx.coroutines.CoroutineScope(kotlinx.coroutines.SupervisorJob()+kotlinx.coroutines.Dispatchers.Unconfined);val engine=Engine(origin!!);val workspace=Workspace(background);workspace.engine=engine;workspace.rootFiles=engine.files().second
        val debug=NativeDebugger(engine,this);workspace.debugger=debug
        val file=FileNode("debug.py","debug.py",false,emptyList(),null,null);val buffer=workspace.loadBuffer(file);buffer.language?.close();buffer.language=null;workspace.show("debug.py");val original=buffer.text;val state=buffer.state
        suspend fun await(label:String,ready:()->Boolean){kotlinx.coroutines.withTimeout(45000){while(!ready()){check(debug.status!="failed"){"$label: ${debug.message}"};kotlinx.coroutines.delay(25)}}}
        try{
            state.edit{append("# previous draft\n")};state.undoState.undo();state.edit{selection=androidx.compose.ui.text.TextRange(2)};buffer.sourceScroll.scrollTo(7);assertTrue(state.undoState.canRedo)
            val line=original.lines().indexOfFirst{"# hickory-thread-breakpoint" in it};check(line>=0);debug.toggle(buffer,line);debug.prepare(buffer);await("Paused source before resource move"){debug.status=="paused"&&debug.threadReading!=null};workspace.revealDebug("debug.py",line);val held=debug.locationGuard("debug.py",line);assertTrue(held());assertTrue(debug.sourceMatches(buffer))
            val edit=JSONObject().put("documentChanges",org.json.JSONArray().put(JSONObject().put("kind","rename").put("oldUri",languageURI("debug.py")).put("newUri",languageURI("lineage-moved.py"))))
            val resources=workspace.resourceChanges;resources.prepare(workspace,"Move paused source",edit);assertNotNull(resources.notice,resources.review);resources.apply(workspace);assertFalse(resources.notice,resources.hasPending);assertSame(buffer,workspace.buffers["lineage-moved.py"]);assertFalse(held());assertFalse(debug.sourceMatches(buffer));assertFalse(debug.positionSourceMatches(buffer));assertEquals("debug.py",debug.owner)
            assertEquals(original,engine.file(buffer.file).content);val revision=debug.breakpointModel.intents.getValue(buffer.debugIdentity).revision;debug.toggle(buffer,line,"main_value == 7");assertEquals(revision,debug.breakpointModel.intents.getValue(buffer.debugIdentity).revision);debug.select(debug.frames.first());workspace.revealDebug("debug.py",line);assertSame(buffer,workspace.currentBuffer);assertFalse(buffer.debugRevealCurrent());assertSame(state,buffer.state);assertEquals(2,state.selection.start);assertEquals(7,buffer.sourceScroll.value);assertTrue(state.undoState.canRedo)
            workspace.changes.open("undo",workspace);await("Guarded menu Undo resource review"){!resources.busy&&resources.review!=null};resources.apply(workspace);assertFalse(resources.notice,resources.hasPending);assertSame(buffer,workspace.buffers["debug.py"]);assertEquals(original,engine.file(file).content);assertTrue(debug.sourceMatches(buffer));assertFalse(buffer.resourceDeleted)
            assertTrue(debug.breakpointRows(buffer).all{it.optString("state")!="bound"});debug.toggle(buffer,line,"main_value == 7");await("Restored original editor live condition binding"){debug.breakpointModel.receipts[buffer.debugIdentity]?.marks?.any{it.optString("state")=="bound"&&it.optString("condition")=="main_value == 7"}==true}
            assertSame(state,buffer.state);assertEquals(original,buffer.text);assertTrue(state.undoState.canRedo);debug.stop();await("Stopped restored session"){!debug.active}
        }finally{if(debug.active)debug.stop();debug.close();background.coroutineContext[kotlinx.coroutines.Job]?.cancel()}
    }
    @Test(timeout=90000) fun liveLiteralPathBreakpointChannel()=kotlinx.coroutines.runBlocking{
        val origin=System.getenv("HICKORY_NATIVE_DEBUG_ORIGIN");org.junit.Assume.assumeNotNull(origin)
        val path="literal %2F λ?#.py";val engine=Engine(origin!!);val file=FileNode(path,path,false,emptyList(),null,null);val loaded=engine.file(file);val buffer=DocumentBuffer(file,loaded);val debug=NativeDebugger(engine,this)
        suspend fun await(label:String,ready:()->Boolean){repeat(600){if(ready())return;if(debug.status=="failed")error("$label: ${debug.message}");kotlinx.coroutines.delay(50)};error("$label: ${debug.status}: ${debug.message}")}
        try{
            debug.toggle(buffer,1);debug.prepare(buffer);await("Literal-path source breakpoint pause"){debug.status=="paused"};assertEquals(path,debug.owner);assertEquals(1,debug.line(path));assertEquals(loaded.content,buffer.text)
            debug.toggle(buffer,1,"value == 1");await("Literal-path live condition receipt"){debug.breakpointModel.receipts[buffer.debugIdentity]?.marks?.singleOrNull()?.let{it.optString("condition")=="value == 1"&&it.optString("state")=="bound"}==true}
            val revision=debug.breakpointModel.intents.getValue(buffer.debugIdentity).revision;buffer.state.edit{append("# unsaved\n")};debug.toggle(buffer,1,"value == 2");assertEquals(revision,debug.breakpointModel.intents.getValue(buffer.debugIdentity).revision);assertTrue(debug.breakpointRows(buffer).isEmpty());assertTrue(debug.message.contains("unbound",ignoreCase=true));assertEquals(loaded.content,engine.file(file).content);assertTrue(buffer.dirty)
            debug.stop();await("Literal-path session stopped"){!debug.active}
        }finally{if(debug.active)debug.stop();debug.close()}
    }
    @OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
    @Test(timeout=120000) fun liveSourceAdjacentPytestDebugPreservesEditorAndRetainsSelection()=kotlinx.coroutines.runBlocking {
        val origin=System.getenv("HICKORY_NATIVE_TEST_DEBUG_ORIGIN");org.junit.Assume.assumeNotNull(origin)
        val path=System.getenv("HICKORY_NATIVE_TEST_DEBUG_INPUT") ?: "test_choice.py"
        val engine=Engine(origin!!);val workspace=Workspace(this);workspace.engine=engine
        val tree=engine.files().second
        fun find(nodes:List<FileNode>):FileNode? {for(node in nodes){if(node.path==path)return node;find(node.children)?.let{return it}};return null}
        // Authored inputs must use their indexed document identity and read
        // endpoint, just as the editor does; the plain-file API refuses them.
        val file=find(tree) ?: error("Source test fixture input is not listed: $path")
        val loaded=engine.file(file);val buffer=DocumentBuffer(file,loaded);val state=buffer.state
        workspace.buffers[path]=buffer;workspace.rootFiles=tree;workspace.show(path);workspace.showTests=true
        val debug=NativeDebugger(engine,this);workspace.debugger=debug
        val index=SourceTestIndex(this,0){input,source->SourceTestScan.decode(engine.request("POST","/api/tests/source-candidates",JSONObject().put("path",input).put("source",source)))}
        workspace.sourceTestIndex=index;buffer.testIndex=index
        buffer.inspectTests={line->launch {workspace.inspectTestSource(buffer,line)}}
        suspend fun await(label:String,condition:()->Boolean){repeat(1000){if(condition())return;kotlinx.coroutines.delay(30)};error("$label: ${debug.status} ${debug.message} ${workspace.testSources.notice}")}
        try {
            assertFalse("The fixture input must be writable, not another construction's generated output",buffer.readOnly)
            state.edit{append("# earlier native edit\n")};assertTrue(buffer.text.endsWith("# earlier native edit\n"));state.undoState.undo();assertEquals(loaded.content,buffer.text);assertTrue(state.undoState.canRedo)
            buffer.readTestSources();await("indexed selected source-test gutter"){buffer.sourceTestRows().any{it.second.name=="test_selected"}}
            val row=buffer.sourceTestRows().single{it.second.name=="test_selected"};buffer.inspectTests!!.invoke(row.first)
            val model=workspace.testSources;await("source-test gutter opens floating observation"){model.observation!=null&&workspace.showTests}
            val facts=model.observation?.second ?: error(model.notice)
            val selected=facts.candidates.single{it.name=="test_selected"};assertEquals("TestChoice::test_selected",selected.nameFilter)
            val line=buffer.text.lines().indexOfFirst{"# source-test-breakpoint" in it};check(line>=0)
            buffer.state.edit{selection=androidx.compose.ui.text.TextRange(1,3)};val selection=buffer.state.selection;val scroll=buffer.sourceScroll.value
            debug.toggle(buffer,line);model.debug(selected,TestSourceSelection.NAME_FILTER,workspace)
            await("selected saved test pause"){debug.status in listOf("paused","failed")};assertEquals(debug.message,"paused",debug.status)
            assertEquals(1,model.debugRuns.size);assertEquals(facts.fingerprint,model.debugRuns.single().fingerprint);assertEquals(facts.sourceHashes,model.debugRuns.single().contributors)
            debug.evaluate("selected_value");await("selected test evaluation"){debug.values["selected_value"]!=null};assertEquals("41",debug.values["selected_value"])
            workspace.showTests=false
            debug.step("continue");await("selected test process completion"){!debug.active}
            assertEquals(debug.message,"finished",debug.status);assertEquals("finished",model.debugRuns.single().state)
            assertEquals(0,model.debugRuns.single().exit)
            // Check after actual completion, so this cannot pass merely because
            // a stopped test never reached its accidentally included sibling.
            val files=engine.request("GET","/api/files").toString();assertFalse(files.contains("sibling-ran.txt"))
            assertSame(buffer,workspace.currentBuffer);assertSame(state,buffer.state);assertEquals(selection,buffer.state.selection);assertEquals(scroll,buffer.sourceScroll.value);assertEquals(loaded.content,buffer.text);assertFalse(buffer.dirty);assertTrue(state.undoState.canRedo)
        }finally{index.stop();if(debug.active)debug.stop();debug.close()}
    }
    @Test fun liveNativeDebugChannel() = kotlinx.coroutines.runBlocking {
        val origin = System.getenv("HICKORY_NATIVE_DEBUG_ORIGIN")
        org.junit.Assume.assumeNotNull(origin)
        val engine = Engine(origin!!); val file = FileNode("debug.py", "debug.py", false, emptyList(), null, null)
        val loaded = engine.file(file); val buffer = DocumentBuffer(file, loaded); val debug = NativeDebugger(engine, this)
        try {
            debug.toggle(buffer, 1); debug.prepare(buffer)
            suspend fun paused() { repeat(200) { if (debug.status == "paused" || debug.status == "failed") return; kotlinx.coroutines.delay(50) } }
            paused(); assertEquals(debug.message, "paused", debug.status); debug.step("over"); paused()
            assertEquals(debug.message, "paused", debug.status); assertTrue(debug.variables.isNotEmpty())
            debug.evaluate("value", true); repeat(100) { if (debug.values["value"] == null) kotlinx.coroutines.delay(50) }
            assertNotNull(debug.values["value"]); assertEquals(loaded.content, buffer.text); assertFalse(buffer.dirty)
            debug.stop(); repeat(100) { if (debug.active) kotlinx.coroutines.delay(50) }; assertFalse(debug.message, debug.active)
        } finally { debug.close() }
    }
}
