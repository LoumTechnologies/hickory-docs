<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Keep the Compose terminal attached to its backend PTY

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

Compose hosts the same bundled xterm interactive binding as Apple. The local page has no network access: a restricted host bridge forwards xterm input and size events, while native OkHttp carries ordered PTY bytes. WebView callbacks post output onto the UI thread and reject retired renderers. Releasing the view closes its connection without closing the workspace session, so moving a pane reattaches to retained output. A failed connection is reported without resending uncertain input.

<ui:copy id="simple-8f29e9977d-0">
package com.hickorydocs.simple

import android.annotation.SuppressLint
import android.util.Base64
import android.webkit.*
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.sp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import okhttp3.*
import okio.ByteString
import org.json.JSONObject

class TerminalSession {
    var id by mutableStateOf<String?>(null)
    var opening by mutableStateOf(false)
    var error by mutableStateOf("")
    suspend fun open(engine: Engine) {
        if (id != null || opening) return
        opening = true; error = ""
        try { id = engine.openTerminal() }
        catch (cancelled: CancellationException) { throw cancelled }
        catch (failure: Exception) { error = "${failure.message} Check the terminal backend, then choose Terminal again." }
        finally { opening = false }
    }
}
</ui:copy>

Enable JavaScript only for the bundled local page, deny page navigation and keep network transport native. OkHttp queues input frames in order. Output crosses back onto the UI thread only while its renderer remains active; teardown removes the bridge and cancels the socket without closing the backend session.

<ui:copy id="simple-terminal-android-attachment">
private val terminalHTTP = OkHttpClient()
// The page has no network permissions. Its bridge is installed only for our
// bundled file, and is removed before destroying the detached renderer.
@SuppressLint("SetJavaScriptEnabled")
@Composable fun TerminalPane(workspace: Workspace) {
    val session = workspace.terminal; val engine = workspace.engine
    val scope = workspace.viewModelScope
    Column(Modifier.fillMaxSize()) {
        val id = session.id
        if (engine != null && id != null) key(engine, id) {
            AndroidView(modifier = Modifier.weight(1f).fillMaxWidth(), factory = { context ->
                WebView(context).apply {
                    val view = this
                    var active = true
                    var socket: WebSocket? = null
                    val dispose = { active = false; socket?.cancel(); removeJavascriptInterface("TerminalHost"); Unit }
                    tag = dispose
                    fun failure(message: String) { view.post { if (active) { session.error = "$message Reopen the Terminal tab to reconnect; input was not retried."; socket?.cancel(); socket = null } } }
                    settings.javaScriptEnabled = true
                    settings.allowFileAccess = true; settings.allowContentAccess = false
                    settings.allowFileAccessFromFileURLs = false; settings.allowUniversalAccessFromFileURLs = false
                    addJavascriptInterface(object {
                        @JavascriptInterface fun postMessage(json: String) {
                            view.post {
                                if (!active) return@post
                                val message = runCatching { JSONObject(json) }.getOrNull() ?: return@post
                                when (message.optString("kind")) {
                                    "input" -> if (socket?.send(message.optString("data")) != true) failure("Terminal input could not be sent.")
                                    "resize" -> {
                                        val rows = message.optInt("rows"); val cols = message.optInt("cols")
                                        if (rows in 1..1000 && cols in 1..1000) scope.launch {
                                            try { engine.terminalResize(id, rows, cols) }
                                            catch (cancelled: CancellationException) { throw cancelled }
                                            catch (error: Exception) { if (active) session.error = "${error.message} Resize the pane to retry its terminal size." }
                                        }
                                    }
                                }
                            }
                        }
                    }, "TerminalHost")
                    webViewClient = object : WebViewClient() {
                        override fun shouldOverrideUrlLoading(view: WebView, request: WebResourceRequest) = true
                        override fun onPageFinished(view: WebView, url: String) {
                            if (!active || url != "file:///android_asset/math.html" || socket != null) return
                            view.evaluateJavascript("try { globalThis.startInteractiveTerminal(); 'ready' } catch(error) { String(error) }") { result ->
                                if (!active) return@evaluateJavascript
                                if (result != "\"ready\"") { session.error = "Terminal renderer unavailable. Prepare bundled assets and rebuild the app."; return@evaluateJavascript }
                                val address = engine.debugURL.substringBefore("/api/") + "/api/terminals/ws?session=" + java.net.URLEncoder.encode(id, "UTF-8")
                                socket = terminalHTTP.newWebSocket(Request.Builder().url(address).build(), object : WebSocketListener() {
                                    override fun onOpen(webSocket: WebSocket, response: Response) { view.post { if (active) session.error = "" } }
                                    fun output(bytes: ByteArray) {
                                        val encoded = Base64.encodeToString(bytes, Base64.NO_WRAP)
                                        view.post { if (active) view.evaluateJavascript("globalThis.writeInteractiveTerminal('$encoded')", null) }
                                    }
                                    override fun onMessage(webSocket: WebSocket, bytes: ByteString) { output(bytes.toByteArray()) }
                                    override fun onMessage(webSocket: WebSocket, text: String) { output(text.toByteArray(Charsets.UTF_8)) }
                                    override fun onFailure(webSocket: WebSocket, error: Throwable, response: Response?) { failure(error.message ?: "Terminal disconnected.") }
                                    override fun onClosed(webSocket: WebSocket, code: Int, reason: String) { failure("Terminal disconnected.") }
                                })
                            }
                        }
                        override fun onRenderProcessGone(view: WebView, detail: RenderProcessGoneDetail): Boolean { session.error = "Terminal renderer stopped. Reopen the tab to reconnect."; dispose(); return true }
                    }
                    loadUrl("file:///android_asset/math.html")
                }
            }, onRelease = { view ->
                @Suppress("UNCHECKED_CAST") val dispose = view.tag as? (() -> Unit)
                dispose?.invoke(); view.stopLoading(); view.destroy()
            })
        } else Text(if (session.opening) "Opening Terminal…" else "Choose Terminal to connect.", modifier = Modifier.weight(1f))
        if (session.error.isNotEmpty()) Text(session.error, color = MaterialTheme.colorScheme.error, fontSize = 12.sp)
    }
}
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/TerminalPane.kt"><ui:paste select="#simple-8f29e9977d-0"/><ui:paste select="#simple-terminal-android-attachment"/></ui:file>

</ui:doc>
