package com.hickorydocs.simple

import android.annotation.SuppressLint
import android.webkit.WebView
import android.webkit.WebViewClient
import android.webkit.WebResourceRequest
import android.webkit.RenderProcessGoneDetail
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.ui.Modifier
import androidx.compose.ui.viewinterop.AndroidView
import org.json.JSONObject

private fun mathPreviewScript(source: String, renderer: String?, revision: Int): String {
    val escaped = JSONObject.quote(source)
    return when (renderer) {
        "notebook" -> "globalThis.renderNotebookOutput($escaped)"
        "terminal" -> "globalThis.renderTerminal($escaped)"
        "source-output" -> "globalThis.renderSourceOutput($escaped,$revision)"
        null -> "globalThis.renderMath($escaped)"
        else -> "globalThis.renderDiagram($escaped,${JSONObject.quote(renderer)},$revision)"
    }
}

@SuppressLint("SetJavaScriptEnabled")
@Composable fun NativeMathView(source: String, modifier: Modifier = Modifier, diagramRenderer: String? = null, onReady: (() -> Unit)? = null, onLink: ((String) -> Unit)? = null) {
    val ready = rememberUpdatedState(onReady)
    val activateLink = rememberUpdatedState(onLink)
    val dark = rememberUpdatedState(isSystemInDarkTheme())
    AndroidView(modifier = modifier, factory = { context ->
        WebView(context).apply {
            settings.javaScriptEnabled = true; settings.allowFileAccess = false; settings.allowContentAccess = false; settings.blockNetworkLoads = true
            webViewClient = object : WebViewClient() {
                override fun onRenderProcessGone(view: WebView, detail: RenderProcessGoneDetail): Boolean { view.destroy(); return true }
                override fun shouldOverrideUrlLoading(view: WebView, request: WebResourceRequest): Boolean {
                    val url = request.url
                    if (request.isForMainFrame && url.scheme == "hickory-diagram" && url.host == "navigate" &&
                        url.getQueryParameter("revision") == view.getTag(android.R.id.custom)?.toString()) {
                        url.pathSegments.singleOrNull()?.let { activateLink.value?.invoke(it) }
                    }
                    return url.toString() != "file:///android_asset/math.html"
                }
                override fun onPageFinished(view: WebView, url: String) {
                    if (url == "file:///android_asset/math.html") {
                        val rendered = view.tag as? String ?: return
                        view.evaluateJavascript("globalThis.hickoryAndroidDiagram=true;globalThis.setPreviewAppearance(${dark.value});" + rendered + ";!!document.querySelector('.katex')") { if (it == "true" && view.tag == rendered) ready.value?.invoke() }
                    }
                }
            }
            setTag(android.R.id.custom, 1)
            tag = mathPreviewScript(source, diagramRenderer, 1)
            loadUrl("file:///android_asset/math.html")
        }
    }, update = { view ->
        view.evaluateJavascript("globalThis.setPreviewAppearance?.(${dark.value})", null)
        val current = view.getTag(android.R.id.custom) as? Int ?: 1
        if (view.tag != mathPreviewScript(source, diagramRenderer, current)) {
            val revision = current + 1
            view.setTag(android.R.id.custom, revision)
            val command = mathPreviewScript(source, diagramRenderer, revision)
            view.tag = command
            view.evaluateJavascript("if(globalThis.renderDiagram){" + command + "};!!document.querySelector('.katex')") { if (it == "true" && view.tag == command) ready.value?.invoke() }
        }
    }, onRelease = { it.stopLoading(); it.destroy() })
}
