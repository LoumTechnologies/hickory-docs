import SwiftUI
import WebKit

@MainActor extension DocumentBuffer {
    var canAuthorElements: Bool {
        !readOnly && !resourceDeleted && !saving &&
            (file.doc_id != nil || ["md", "markdown"].contains((file.path as NSString).pathExtension.lowercased()))
    }

    #if os(macOS)
    func elementFormContainsWheel(at point: NSPoint, in window: NSWindow?) -> Bool {
        guard elementForm != nil, let surface = elementFormSurface,
              let window, surface.window === window else { return false }
        return surface.bounds.contains(surface.convert(point, from: nil))
    }
    #endif

    func closeElementForm() {
        elementForm = nil
        focusSource?()
    }

    func openElementForm(line: Int? = nil) {
        guard canAuthorElements, let editor = sourceEditorState?(), editor.attached, editor.editable,
              !editor.composing, line != nil || editor.selections == 1 else {
            error = "Finish typing and choose a single selection in an editable Markdown document to insert an element."
            return
        }
        error = ""
        if elementLines != nil { elementFormError = "" }
        elementForm = NativeElementIntent(buffer: self, editor: editor.identity, range: editor.selection, line: line)
    }
}

struct NativeElementIntent {
    let id = UUID()
    let revision: UUID
    let editor: UUID
    let selection: UUID
    let path: String
    let source: String
    let range: NSRange
    let line: Int?

    @MainActor init(buffer: DocumentBuffer, editor: UUID, range: NSRange, line: Int?) {
        revision = buffer.sourceEditIdentity
        selection = buffer.selectionIdentity
        path = buffer.file.path
        source = buffer.text
        self.editor = editor
        self.range = range
        self.line = line
    }

    @MainActor func current(_ buffer: DocumentBuffer) -> Bool {
        guard buffer.canAuthorElements, buffer.sourceEditIdentity == revision, buffer.selectionIdentity == selection,
              exactUTF8(buffer.file.path, path), exactUTF8(buffer.text, source),
              let state = buffer.sourceEditorState?(), state.identity == editor, state.attached,
              state.editable, !state.composing, exactUTF8(state.source, source) else { return false }
        return line != nil || (state.selections == 1 && state.selection == range)
    }
}

@MainActor final class NativeElementCoordinator: NSObject, WKNavigationDelegate, WKScriptMessageHandler {
    let buffer: DocumentBuffer
    weak var view: WKWebView?
    var asset: URL?
    private var ready = false
    private var sentRevision: UUID?
    private var loadedRevision: UUID?
    private var sentIntent: UUID?
    private var readTask: Task<Void, Never>?

    init(_ buffer: DocumentBuffer) { self.buffer = buffer }

    func update() {
        guard ready, let view else { return }
        let revision = buffer.sourceEditIdentity
        if sentRevision != revision {
            sentRevision = revision
            readTask?.cancel()
            readTask = Task { [weak view, weak buffer] in
                try? await Task.sleep(for: .milliseconds(70))
                guard !Task.isCancelled, let view, let buffer, buffer.sourceEditIdentity == revision else { return }
                view.callAsyncJavaScript("await globalThis.hickoryElements.read(revision, source)",
                    arguments: ["revision": revision.uuidString, "source": buffer.text], in: nil, in: .page, completionHandler: nil)
            }
        }
        guard let intent = buffer.elementForm else {
            if sentIntent != nil { view.evaluateJavaScript("globalThis.hickoryElements.close()") }
            sentIntent = nil
            return
        }
        if !intent.current(buffer) {
            if sentIntent != nil { view.evaluateJavaScript("globalThis.hickoryElements.close()") }
            sentIntent = nil
            let message = "The editor changed. Close this form and open it again."
            if buffer.elementFormError != message { buffer.elementFormError = message }
            return
        }
        guard intent.revision == loadedRevision, sentIntent != intent.id else { return }
        sentIntent = intent.id
        var request: [String: Any] = ["id": intent.id.uuidString, "revision": revision.uuidString,
                                      "from": intent.range.location, "to": NSMaxRange(intent.range)]
        if let line = intent.line { request["line"] = line }
        view.callAsyncJavaScript("globalThis.hickoryElements.open(request)", arguments: ["request": request],
                                 in: nil, in: .page, completionHandler: nil)
    }

    func userContentController(_ userContentController: WKUserContentController, didReceive message: WKScriptMessage) {
        guard message.frameInfo.isMainFrame, view != nil, let value = message.body as? [String: Any],
              let kind = value["kind"] as? String else { return }
        switch kind {
        case "ready":
            ready = true
            buffer.elementFormError = ""
            update()
        case "lines":
            let revision = buffer.sourceEditIdentity
            guard value["revision"] as? String == revision.uuidString, let lines = value["lines"] as? [Int] else { return }
            loadedRevision = revision
            buffer.elementLines = (revision, Set(lines))
            update()
        case "error":
            buffer.elementFormError = value["message"] as? String ?? "Close the form and reopen the document to retry."
        case "cancel":
            guard value["id"] as? String == buffer.elementForm?.id.uuidString else { return }
            buffer.closeElementForm()
        case "apply":
            guard let intent = buffer.elementForm, value["id"] as? String == intent.id.uuidString,
                  let plan = value["plan"] as? [String: Any] else { return }
            if apply(plan, intent: intent) { buffer.closeElementForm() }
            else { buffer.elementFormError = "The edit could not be applied. Close the form and try again in the current source." }
        default: break
        }
    }

    private func apply(_ plan: [String: Any], intent: NativeElementIntent) -> Bool {
        guard intent.current(buffer), let from = plan["from"] as? Int, let to = plan["to"] as? Int,
              let text = plan["insert"] as? String, from >= 0, to >= from, to <= intent.source.utf16.count,
              let start = plan["selectFrom"] as? Int, let end = plan["selectTo"] as? Int,
              start >= from, end >= start, end <= from + text.utf16.count else { return false }
        let range = NSRange(location: from, length: to - from)
        guard intent.line != nil || range == intent.range else { return false }
        return buffer.applyElementDraft?(NativeSourceEdit(range: range, text: text), intent.source,
                                         NSRange(location: start, length: end - start)) == true
    }

    func webView(_ webView: WKWebView, decidePolicyFor action: WKNavigationAction,
                 decisionHandler: @escaping (WKNavigationActionPolicy) -> Void) {
        decisionHandler(action.targetFrame?.isMainFrame == true && action.request.url == asset ? .allow : .cancel)
    }

    func webViewWebContentProcessDidTerminate(_ webView: WKWebView) { unavailable() }
    func webView(_ webView: WKWebView, didFailProvisionalNavigation navigation: WKNavigation!, withError error: Error) { unavailable() }
    func unavailable() {
        readTask?.cancel()
        ready = false
        buffer.elementLines = nil
        buffer.elementFormError = "Element forms are unavailable. Reopen the document to retry."
    }

    func makeView() -> WKWebView {
        let configuration = WKWebViewConfiguration()
        configuration.websiteDataStore = .nonPersistent()
        configuration.userContentController.add(self, name: "elements")
        let web = WKWebView(frame: .zero, configuration: configuration)
        web.navigationDelegate = self
        view = web
        #if os(macOS)
        buffer.elementFormSurface = web
        #endif
        let owner = Bundle(for: MathCoordinator.self)
        let roots = [Bundle.main.resourceURL, Bundle.main.bundleURL, owner.resourceURL, owner.bundleURL,
                     URL(fileURLWithPath: CommandLine.arguments[0]).deletingLastPathComponent()].compactMap { $0 }
        if let folder = nativeMathResource(roots: roots)?.deletingLastPathComponent() {
            let file = folder.appendingPathComponent("element-authoring.html").standardizedFileURL.resolvingSymlinksInPath()
            if FileManager.default.isReadableFile(atPath: folder.appendingPathComponent("node_modules/element-authoring.js").path) {
                asset = file
                web.loadFileURL(file, allowingReadAccessTo: folder)
            }
        }
        if asset == nil { Task { unavailable() } }
        return web
    }

    func dismantle(_ view: WKWebView) {
        readTask?.cancel()
        self.view = nil
        #if os(macOS)
        if buffer.elementFormSurface === view { buffer.elementFormSurface = nil }
        #endif
        view.stopLoading()
        view.navigationDelegate = nil
        view.configuration.userContentController.removeScriptMessageHandler(forName: "elements")
        buffer.elementLines = nil
        buffer.elementForm = nil
    }
}

struct NativeElementForm: View {
    var buffer: DocumentBuffer
    @State private var activated = false

    var body: some View {
        GeometryReader { geometry in
            let open = buffer.elementForm != nil
            let width = min(360, max(1, geometry.size.width - 20))
            let height = min(490, max(1, geometry.size.height - 16))
            let row = buffer.elementForm?.line.flatMap { line in buffer.visibleDebugLines.first { $0.line == line }?.y } ?? 0
            Group {
                if activated {
                    NativeElementWebView(buffer: buffer, revision: buffer.sourceEditIdentity, intent: buffer.elementForm?.id)
                } else { Color.clear }
            }
                .frame(width: open ? width : 1, height: open ? height : 1)
                .background(.background)
                .overlay {
                    if open, !buffer.elementFormError.isEmpty || buffer.elementLines?.revision != buffer.sourceEditIdentity {
                        VStack(alignment: .leading, spacing: 8) {
                            Text(buffer.elementFormError.isEmpty ? "Reading document elements…" : buffer.elementFormError).textSelection(.enabled)
                            Button("Close") { buffer.closeElementForm() }
                        }.padding().frame(maxWidth: .infinity, maxHeight: .infinity).background(.background)
                    }
                }
                .clipShape(RoundedRectangle(cornerRadius: 8))
                .overlay(RoundedRectangle(cornerRadius: 8).strokeBorder(.secondary.opacity(0.35)).allowsHitTesting(false))
                .opacity(open ? 1 : 0).allowsHitTesting(open).accessibilityHidden(!open)
                .position(x: geometry.size.width - width / 2 - 10,
                          y: min(max(8, row), max(8, geometry.size.height - height - 8)) + height / 2)
        }.task {
            try? await Task.sleep(for: .milliseconds(70))
            if !Task.isCancelled { activated = true }
        }
    }
}

#if os(macOS)
private struct NativeElementWebView: NSViewRepresentable {
    var buffer: DocumentBuffer
    var revision: UUID
    var intent: UUID?
    func makeCoordinator() -> NativeElementCoordinator { NativeElementCoordinator(buffer) }
    func makeNSView(context: Context) -> WKWebView { context.coordinator.makeView() }
    func updateNSView(_ view: WKWebView, context: Context) { Task { context.coordinator.update() } }
    static func dismantleNSView(_ view: WKWebView, coordinator: NativeElementCoordinator) { coordinator.dismantle(view) }
}
#else
private struct NativeElementWebView: UIViewRepresentable {
    var buffer: DocumentBuffer
    var revision: UUID
    var intent: UUID?
    func makeCoordinator() -> NativeElementCoordinator { NativeElementCoordinator(buffer) }
    func makeUIView(context: Context) -> WKWebView { context.coordinator.makeView() }
    func updateUIView(_ view: WKWebView, context: Context) { Task { context.coordinator.update() } }
    static func dismantleUIView(_ view: WKWebView, coordinator: NativeElementCoordinator) { coordinator.dismantle(view) }
}
#endif
