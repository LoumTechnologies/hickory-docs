package com.hickorydocs.simple

import android.annotation.SuppressLint
import android.webkit.WebView
import android.webkit.WebViewClient
import android.webkit.WebResourceRequest
import android.webkit.RenderProcessGoneDetail
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.viewinterop.AndroidView
import org.json.JSONObject

@SuppressLint("SetJavaScriptEnabled")
@Composable fun NativeMathView(source: String, modifier: Modifier = Modifier, diagramRenderer: String? = null) {
    AndroidView(modifier = modifier, factory = { context ->
        WebView(context).apply {
            settings.javaScriptEnabled = true; settings.allowFileAccess = false; settings.allowContentAccess = false; settings.blockNetworkLoads = true
            webViewClient = object : WebViewClient() {
                override fun onRenderProcessGone(view: WebView, detail: RenderProcessGoneDetail): Boolean { view.destroy(); return true }
                override fun shouldOverrideUrlLoading(view: WebView, request: WebResourceRequest): Boolean = request.url.toString() != "file:///android_asset/math.html"
                override fun onPageFinished(view: WebView, url: String) {
                    if (url == "file:///android_asset/math.html") view.evaluateJavascript(view.tag as? String ?: "", null)
                }
            }
            tag = if (diagramRenderer == "terminal") "globalThis.renderTerminal(" + JSONObject.quote(source) + ")" else if (diagramRenderer == null) "globalThis.renderMath(" + JSONObject.quote(source) + ")" else "globalThis.renderDiagram(" + JSONObject.quote(source) + "," + JSONObject.quote(diagramRenderer) + ")"; loadUrl("file:///android_asset/math.html")
        }
    }, update = { view ->
        val script = if (diagramRenderer == "terminal") "globalThis.renderTerminal(" + JSONObject.quote(source) + ")" else if (diagramRenderer == null) "globalThis.renderMath(" + JSONObject.quote(source) + ")" else "globalThis.renderDiagram(" + JSONObject.quote(source) + "," + JSONObject.quote(diagramRenderer) + ")"
        if (view.tag != script) {
            view.tag = script
            view.evaluateJavascript("if(globalThis.renderDiagram){" + script + "}", null)
        }
    }, onRelease = { it.stopLoading(); it.destroy() })
}
