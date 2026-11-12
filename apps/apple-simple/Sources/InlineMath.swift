import SwiftUI
import WebKit
@MainActor final class MathCoordinator: NSObject, WKNavigationDelegate {
    var source = ""
    var diagramRenderer: String?
    private var displayed: String?
    var ready = false
    func display(_ view: WKWebView) {
        guard ready, let bytes = try? JSONSerialization.data(withJSONObject: source, options: .fragmentsAllowed), let json = String(data: bytes, encoding: .utf8) else { return }
        let key = (diagramRenderer.map { "diagram:" + $0 } ?? "math") + "\0" + source
        guard displayed != key else { return }
        displayed = key
        if diagramRenderer == "terminal" { view.evaluateJavaScript("globalThis.renderTerminal(" + json + ")", completionHandler: nil) }
        else if let renderer = diagramRenderer, let data = try? JSONSerialization.data(withJSONObject: renderer, options: .fragmentsAllowed), let engine = String(data: data, encoding: .utf8) {
            view.evaluateJavaScript("globalThis.renderDiagram(" + json + "," + engine + ")", completionHandler: nil)
        } else { view.evaluateJavaScript("globalThis.renderMath(" + json + ")", completionHandler: nil) }
    }
    func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) { ready = true; displayed = nil; display(webView) }
    func webView(_ webView: WKWebView, decidePolicyFor navigationAction: WKNavigationAction, decisionHandler: @escaping (WKNavigationActionPolicy) -> Void) {
        let url = navigationAction.request.url
        decisionHandler(url?.isFileURL == true || url?.scheme == "about" ? .allow : .cancel)
    }
}
// Inspect bundles without touching SwiftPM's fatal Bundle.module accessor.
func nativeMathResource(roots: [URL], bundles: [Bundle] = [.main, Bundle(for: MathCoordinator.self)]) -> URL? {
    // Resource-only SwiftPM bundles need not initialize as Foundation Bundles.
    // Inspect their installed directories too, including macOS Contents layout.
    let installed = roots.flatMap { root in
        [root, root.appendingPathComponent("HickorySimple_HickorySimple.bundle"), root.appendingPathComponent("HickorySimple_HickorySimple.resources")]
    } + bundles.flatMap { [$0.bundleURL] + [$0.resourceURL].compactMap { $0 } }
    for root in installed {
        for relative in ["HickoryAssets/Resources/math.html", "Resources/math.html", "Contents/Resources/Resources/math.html", "Contents/Resources/math.html", "math.html"] {
            let url = root.appendingPathComponent(relative)
            if FileManager.default.isReadableFile(atPath: url.path) { return url }
        }
    }
    return nil
}
@MainActor func makeMathView(_ coordinator: MathCoordinator) -> WKWebView {
    let configuration = WKWebViewConfiguration(); configuration.websiteDataStore = .nonPersistent(); configuration.preferences.javaScriptCanOpenWindowsAutomatically = false
    let view = WKWebView(frame: .zero, configuration: configuration); view.navigationDelegate = coordinator
    let owner = Bundle(for: MathCoordinator.self)
    let roots = [Bundle.main.resourceURL, Bundle.main.bundleURL, owner.resourceURL, owner.bundleURL,
        URL(fileURLWithPath: CommandLine.arguments[0]).deletingLastPathComponent()].compactMap { $0 }
    let url = nativeMathResource(roots: roots)
    if let url { view.loadFileURL(url, allowingReadAccessTo: url.deletingLastPathComponent()) }
    else { view.loadHTMLString("<body><p>Preview resources are missing. Rebuild the app with its bundled assets; source stays editable.</p><div id='math'></div><script>globalThis.renderMath=globalThis.renderDiagram=globalThis.renderTerminal=s=>document.getElementById('math').textContent=s</script></body>", baseURL: nil) }
    return view
}
#if os(macOS)
struct NativeMathView: NSViewRepresentable {
    var source: String
    var diagramRenderer: String? = nil
    func makeCoordinator() -> MathCoordinator { MathCoordinator() }
    func makeNSView(context: Context) -> WKWebView { context.coordinator.source = source; context.coordinator.diagramRenderer = diagramRenderer; return makeMathView(context.coordinator) }
    func updateNSView(_ view: WKWebView, context: Context) { context.coordinator.source = source; context.coordinator.diagramRenderer = diagramRenderer; context.coordinator.display(view) }
}
#else
struct NativeMathView: UIViewRepresentable {
    var source: String
    var diagramRenderer: String? = nil
    func makeCoordinator() -> MathCoordinator { MathCoordinator() }
    func makeUIView(context: Context) -> WKWebView { context.coordinator.source = source; context.coordinator.diagramRenderer = diagramRenderer; return makeMathView(context.coordinator) }
    func updateUIView(_ view: WKWebView, context: Context) { context.coordinator.source = source; context.coordinator.diagramRenderer = diagramRenderer; context.coordinator.display(view) }
}
#endif
// One read-only terminal surface for captured bytes and expect bodies.
struct NativeInlineTerminal: View {
    var text: String
    var editSource: (() -> Void)? = nil
    var body: some View {
        VStack(spacing: 0) {
            HStack { Image(systemName: "terminal"); Text("Terminal"); Spacer(); if let editSource { Button("Edit source text", action: editSource).buttonStyle(.plain) } }
                .font(.system(size: 11, design: .monospaced)).padding(6).background(Color(red: 0.11, green: 0.15, blue: 0.2))
            NativeMathView(source: text, diagramRenderer: "terminal")
        }.foregroundStyle(Color(red: 0.9, green: 0.93, blue: 0.95)).background(Color(red: 0.06, green: 0.08, blue: 0.11))
            .clipShape(RoundedRectangle(cornerRadius: 6)).overlay(RoundedRectangle(cornerRadius: 6).stroke(.gray.opacity(0.5))).accessibilityLabel("Output terminal")
    }
}
struct NativeDiagramPreview: View {
    var diagram: InlineDiagram
    var buffer: DocumentBuffer
    var body: some View {
        if diagram.renderer == "terminal" { NativeInlineTerminal(text: diagram.body, editSource: { buffer.sourceTable = diagram.id }) }
        else { VStack(spacing: 0) {
            HStack { Text("Diagram").font(.caption); Spacer(); Button("Edit source text") { buffer.sourceTable = diagram.id }.buttonStyle(.bordered) }.padding(4)
            NativeMathView(source: diagram.body, diagramRenderer: diagram.renderer)
        }.background(.background) }
    }
}
struct InlineEquationControl: View {
    var equation: InlineEquation
    var source: String
    var buffer: DocumentBuffer
    @State private var draft = ""
    @State private var notice = ""
    var body: some View {
        VStack(alignment: .leading) {
            NativeMathView(source: draft).frame(height: 120)
            TextEditor(text: $draft).font(.system(.caption, design: .monospaced)).frame(height: 64).disabled(buffer.readOnly)
            Button("Apply equation") {
                if buffer.applySourceEdits?([NativeSourceEdit(range: equation.range, text: draft)], source) != true { notice = "Equation changed. Reopen this line and try again." }
            }.disabled(buffer.readOnly || draft == equation.body)
            if !notice.isEmpty { Text(notice).font(.caption) }
        }.onAppear { draft = equation.body }
    }
}
