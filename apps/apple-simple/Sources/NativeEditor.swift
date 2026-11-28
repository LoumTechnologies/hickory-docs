import SwiftUI
import CoreText
import WebKit
// The grid is hosted inside the text view at a source-backed layout anchor.
// Cell commits go through the editor's snapshot/permission guard and native undo.
struct NativeTableGrid: View {
    var table: InlineTable; var source: String; var buffer: DocumentBuffer
    @State private var caretTarget: Int?
    @State private var caretRequest = GridCaretRequest()
    @State private var preferredX: CGFloat?
    @State private var sharedReady = false
    @State private var legacyFocused = false
    private var showShared: Bool { sharedReady && !legacyFocused }
    var body: some View {
        ZStack {
            legacyBody.opacity(showShared ? 0 : 1).allowsHitTesting(!showShared).accessibilityHidden(showShared)
            VStack(spacing: 0) {
                NativeSpreadsheet(table: table, source: source, buffer: buffer, onReady: { sharedReady = true }, onUnavailable: { sharedReady = false })
            }.opacity(showShared ? 1 : 0).allowsHitTesting(showShared).accessibilityHidden(!showShared)
        }
    }
    private var legacyBody: some View {
        VStack(spacing: 0) {
            ScrollView([.horizontal, .vertical]) {
                LazyVStack(alignment: .leading, spacing: 0) {
                    HStack(spacing: 0) {
                        Text("#").frame(width: 32, height: 28)
                        ForEach(0..<(table.rows.map(\.count).max() ?? 0), id: \.self) { column in
                            Text(tableColumnName(column)).font(.system(size: 12, weight: .semibold)).frame(width: 140, height: 28).border(.secondary.opacity(0.3))
                        }
                    }
                    ForEach(table.rows.indices, id: \.self) { row in
                        HStack(spacing: 0) {
                            Text(String(row + 1)).font(.caption).frame(width: 32, height: 28)
                            ForEach(table.rows[row].indices, id: \.self) { column in
                                NativeGridCell(cell: table.rows[row][column], row: row, source: source, buffer: buffer, request: caretTarget == row * 10000 + column ? caretRequest : nil, onFocus: { if caretTarget != row * 10000 + column { caretTarget = nil; preferredX = nil } }, move: { dr, dc, x in
                                    let nextRow = min(max(0, row + dr), table.rows.count - 1)
                                    let nextColumn = min(max(0, column + dc), table.rows[nextRow].count - 1)
                                    guard nextRow != row || nextColumn != column else { return }
                                    let goal = dc == 0 ? preferredX ?? x : x
                                    preferredX = dc == 0 ? goal : nil
                                    caretRequest = GridCaretRequest(x: goal, edge: dc)
                                    caretTarget = nextRow * 10000 + nextColumn
                                }, onCaretActivity: { preferredX = nil }, onEditing: { legacyFocused = $0 }).frame(width: 140, height: 28).border(.secondary.opacity(0.3))
                            }
                        }
                    }
                }
            }
        }.background(.background).border(.secondary.opacity(0.4)).accessibilityElement(children: .contain).accessibilityLabel("Inline table grid")
    }
}
// The retained table bytes are a layout anchor, not a source insertion target.
// Include both boundaries: backspace at the first character would otherwise
// delete the preceding newline, and the final boundary still belongs to its row.
func tableCaretHidden(_ selection: NSRange, tables: [NSRange]) -> Bool {
    selection.length == 0 && tables.contains { selection.location >= $0.location && selection.location <= NSMaxRange($0) }
}
func tableSourceEditHitsGrid(_ range: NSRange, selection: NSRange, tables: [NSRange]) -> Bool {
    tableCaretHidden(selection, tables: tables) || tables.contains {
        NSIntersectionRange($0, range).length > 0 || range.length == 0 && range.location >= $0.location && range.location <= NSMaxRange($0)
    }
}
func selectionAfterSourceEdit(_ selection: NSRange, edit: NativeSourceEdit) -> NSRange {
    let delta = edit.text.utf16.count - edit.range.length
    func map(_ offset: Int) -> Int {
        if offset <= edit.range.location { return offset }
        if offset >= NSMaxRange(edit.range) { return offset + delta }
        return edit.range.location + edit.text.utf16.count
    }
    let start = map(selection.location), end = map(NSMaxRange(selection))
    return NSRange(location: start, length: max(0, end - start))
}
func tableColumnName(_ index: Int) -> String {
    var value = index + 1, name = ""
    while value > 0 { value -= 1; name = String(UnicodeScalar(65 + value % 26)!) + name; value /= 26 }
    return name
}
// Navigation carries a visual column, not a text selection. Match a grapheme
// boundary nearest that column, clamping short/empty cells without splitting emoji.
// Only a collapsed caret at the text edge belongs to table navigation. Let
// native text editing handle interior motion and collapse/extend selections.
func gridHorizontalExit(_ selection: NSRange, length: Int, direction: Int) -> Bool {
    selection.length == 0 && selection.location >= 0 && selection.location <= length &&
        (direction < 0 ? selection.location == 0 : direction > 0 && selection.location == length)
}
struct GridCaretRequest: Equatable { var id = UUID(); var x: CGFloat = 0; var edge = 0 }
func gridCaretOffset(_ text: String, request: GridCaretRequest, measure: (String) -> CGFloat) -> Int {
    let ns = text as NSString
    if request.edge < 0 { return ns.length }
    if request.edge > 0 { return 0 }
    var boundaries = [0], count = 0
    for character in text { count += String(character).utf16.count; boundaries.append(count) }
    var low = 0, high = boundaries.count - 1
    while low < high {
        let mid = (low + high) / 2
        if measure(ns.substring(to: boundaries[mid])) < request.x { low = mid + 1 } else { high = mid }
    }
    if low > 0 && abs(measure(ns.substring(to: boundaries[low - 1])) - request.x) <= abs(measure(ns.substring(to: boundaries[low])) - request.x) { low -= 1 }
    return boundaries[low]
}
private struct NativeGridCell: View {
    var cell: InlineCell; var row: Int; var source: String; var buffer: DocumentBuffer
    @State private var draft = ""
    @State private var notice = ""
    var request: GridCaretRequest?; var onFocus: () -> Void; var move: (Int, Int, CGFloat) -> Void
    var onCaretActivity: () -> Void = {}
    var onEditing: (Bool) -> Void = { _ in }
    @State private var committing = false
    private func commit() {
        guard !committing, !exactUTF8(draft,cell.value) else { return }
        committing = true; defer { committing = false }
        if !exactUTF8(buffer.inlineMarkup.source,source) || buffer.applyTableEdit?(NativeSourceEdit(range: cell.range, text: inlineCellText(draft, format: cell.format, delimiter: cell.delimiter)), source) != true { notice = "Table changed. Reopen the cell and try again." }
    }
    var body: some View {
        GridCellField(text: $draft, request: request, readOnly: buffer.readOnly, onFocus: { onEditing(true); onFocus() }, onCommit: { commit(); onEditing(false) },
            onArrow: { dr, dc, x in commit(); move(dr, dc, x) }, onCaretActivity: onCaretActivity)
            .padding(.horizontal, 5)
            .accessibilityLabel("Row \(row + 1), column \(tableColumnName(cell.column))")
            .onAppear { draft = cell.value }.onChange(of: cell.value) { _, value in draft = value }
            .onDisappear { commit(); onEditing(false) }.help(notice.isEmpty ? "Edit cell; press Return to apply" : notice)
    }
}
#if os(macOS)
import AppKit
struct GridCellField: NSViewRepresentable {
    @Binding var text: String
    var request: GridCaretRequest?; var readOnly: Bool; var onFocus: () -> Void; var onCommit: () -> Void; var onArrow: (Int, Int, CGFloat) -> Void
    var onCaretActivity: () -> Void = {}
    func makeCoordinator() -> Coordinator { Coordinator(self) }
    func makeNSView(context: Context) -> NSTextField {
        let field = NSTextField(); field.isBordered = false; field.drawsBackground = false
        field.font = .systemFont(ofSize: 13); field.focusRingType = .none; field.delegate = context.coordinator
        return field
    }
    func updateNSView(_ field: NSTextField, context: Context) {
        context.coordinator.parent = self
        if !exactUTF8(field.stringValue,text) { field.stringValue = text }
        field.isEditable = !readOnly; field.isSelectable = true
        guard let request, context.coordinator.applied != request.id else { return }
        DispatchQueue.main.async { [weak field, weak coordinator = context.coordinator] in
            guard let field, let coordinator, coordinator.parent.request?.id == request.id, let window = field.window else { return }
            coordinator.applyingRequest = true; defer { coordinator.applyingRequest = false }
            window.makeFirstResponder(field)
            guard let editor = field.currentEditor() as? NSTextView else { return }
            let font = field.font ?? .systemFont(ofSize: 13)
            let offset = gridCaretOffset(field.stringValue, request: request) { ($0 as NSString).size(withAttributes: [.font: font]).width }
            editor.setSelectedRange(NSRange(location: offset, length: 0)); coordinator.applied = request.id
        }
    }
    final class Coordinator: NSObject, NSTextFieldDelegate {
        var parent: GridCellField; var applied: UUID?; var applyingRequest = false
        init(_ parent: GridCellField) { self.parent = parent }
        private var mouseMonitor: Any?
        deinit { if let mouseMonitor { NSEvent.removeMonitor(mouseMonitor) } }
        func controlTextDidBeginEditing(_ notification: Notification) {
            if !applyingRequest { parent.onCaretActivity() }
            parent.onFocus()
            if let mouseMonitor { NSEvent.removeMonitor(mouseMonitor) }
            if let field = notification.object as? NSTextField, let editor = field.currentEditor() as? NSTextView {
                mouseMonitor = NSEvent.addLocalMonitorForEvents(matching: [.leftMouseDown, .leftMouseDragged]) { [weak self, weak editor] event in
                    if let editor, event.window === editor.window, editor.bounds.contains(editor.convert(event.locationInWindow, from: nil)) { self?.parent.onCaretActivity() }
                    return event
                }
            }
        }
        func controlTextDidChange(_ notification: Notification) {
            if !applyingRequest { parent.onCaretActivity() }
            if let field = notification.object as? NSTextField { parent.text = field.stringValue }
        }
        func controlTextDidEndEditing(_ notification: Notification) {
            if let mouseMonitor { NSEvent.removeMonitor(mouseMonitor); self.mouseMonitor = nil }
            parent.onCommit()
        }
        func control(_ control: NSControl, textView: NSTextView, doCommandBy command: Selector) -> Bool {
            guard !textView.hasMarkedText() else { return false }
            let name = NSStringFromSelector(command)
            if name != "moveUp:" && name != "moveDown:" { parent.onCaretActivity() }
            let delta: (Int, Int)
            switch name {
            case "moveUp:": delta = (-1, 0)
            case "moveDown:": delta = (1, 0)
            case "moveLeft:":
                guard gridHorizontalExit(textView.selectedRange(), length: (textView.string as NSString).length, direction: -1) else { return false }
                delta = (0, -1)
            case "moveRight:":
                guard gridHorizontalExit(textView.selectedRange(), length: (textView.string as NSString).length, direction: 1) else { return false }
                delta = (0, 1)
            case "insertNewline:": parent.onCommit(); return true
            default: return false
            }
            let offset = min((textView.string as NSString).length, NSMaxRange(textView.selectedRange()))
            let prefix = (textView.string as NSString).substring(to: offset)
            let x = (prefix as NSString).size(withAttributes: [.font: textView.font ?? NSFont.systemFont(ofSize: 13)]).width
            parent.onArrow(delta.0, delta.1, x); return true
        }
    }
}
#else
import UIKit
final class GridKeyboardField: UITextField {
    var onArrow: ((Int, Int) -> Void)?
    var onCaretActivity: (() -> Void)?
    override func touchesBegan(_ touches: Set<UITouch>, with event: UIEvent?) { onCaretActivity?(); super.touchesBegan(touches, with: event) }
    override var keyCommands: [UIKeyCommand]? {
        guard markedTextRange == nil else { return super.keyCommands }
        var inputs = [UIKeyCommand.inputUpArrow, UIKeyCommand.inputDownArrow]
        if let selected = selectedTextRange {
            let start = offset(from: beginningOfDocument, to: selected.start)
            let end = offset(from: beginningOfDocument, to: selected.end)
            let range = NSRange(location: start, length: end - start), length = (text ?? "").utf16.count
            if gridHorizontalExit(range, length: length, direction: -1) { inputs.append(UIKeyCommand.inputLeftArrow) }
            if gridHorizontalExit(range, length: length, direction: 1) { inputs.append(UIKeyCommand.inputRightArrow) }
        }
        return (inputs.map {
            let command = UIKeyCommand(input: $0, modifierFlags: [], action: #selector(arrow(_:))); command.wantsPriorityOverSystemBehavior = true; return command
        }) + (super.keyCommands ?? [])
    }
    @objc private func arrow(_ command: UIKeyCommand) {
        switch command.input { case UIKeyCommand.inputUpArrow: onArrow?(-1, 0); case UIKeyCommand.inputDownArrow: onArrow?(1, 0); case UIKeyCommand.inputLeftArrow: onArrow?(0, -1); default: onArrow?(0, 1) }
    }
}
struct GridCellField: UIViewRepresentable {
    @Binding var text: String
    var request: GridCaretRequest?; var readOnly: Bool; var onFocus: () -> Void; var onCommit: () -> Void; var onArrow: (Int, Int, CGFloat) -> Void
    var onCaretActivity: () -> Void = {}
    func makeCoordinator() -> Coordinator { Coordinator(self) }
    func makeUIView(context: Context) -> GridKeyboardField {
        let field = GridKeyboardField(); field.font = .systemFont(ofSize: 13); field.borderStyle = .none
        field.onCaretActivity = { [weak coordinator = context.coordinator] in coordinator?.parent.onCaretActivity() }
        field.delegate = context.coordinator; field.addTarget(context.coordinator, action: #selector(Coordinator.changed(_:)), for: .editingChanged)
        field.onArrow = { [weak field, weak coordinator = context.coordinator] dr, dc in
            guard let field, let coordinator else { return }
            let offset = field.selectedTextRange.map { field.offset(from: field.beginningOfDocument, to: $0.end) } ?? 0
            let prefix = ((field.text ?? "") as NSString).substring(to: offset)
            let x = (prefix as NSString).size(withAttributes: [.font: field.font ?? UIFont.systemFont(ofSize: 13)]).width
            coordinator.parent.onArrow(dr, dc, x)
        }
        return field
    }
    func updateUIView(_ field: GridKeyboardField, context: Context) {
        context.coordinator.parent = self
        if !exactUTF8(field.text,text) { context.coordinator.applyingRequest = true; field.text = text; context.coordinator.applyingRequest = false }
        field.isEnabled = !readOnly
        field.accessibilityHint = readOnly ? "This cell is read-only." : nil
        field.toolTip = field.accessibilityHint
        guard let request, context.coordinator.applied != request.id else { return }
        DispatchQueue.main.async { [weak field, weak coordinator = context.coordinator] in
            guard let field, let coordinator, coordinator.parent.request?.id == request.id, field.window != nil else { return }
            coordinator.applyingRequest = true; defer { coordinator.applyingRequest = false }
            field.becomeFirstResponder()
            let offset = gridCaretOffset(field.text ?? "", request: request) { ($0 as NSString).size(withAttributes: [.font: field.font ?? UIFont.systemFont(ofSize: 13)]).width }
            if let position = field.position(from: field.beginningOfDocument, offset: offset) { field.selectedTextRange = field.textRange(from: position, to: position); coordinator.lastSelection = NSRange(location: offset, length: 0) }
            coordinator.applied = request.id
        }
    }
    final class Coordinator: NSObject, UITextFieldDelegate {
        var parent: GridCellField; var applied: UUID?; var applyingRequest = false; var lastSelection: NSRange?
        init(_ parent: GridCellField) { self.parent = parent }
        @objc func changed(_ field: UITextField) { parent.onCaretActivity(); parent.text = field.text ?? "" }
        func textFieldDidChangeSelection(_ textField: UITextField) {
            let selection = textField.selectedTextRange.map { NSRange(location: textField.offset(from: textField.beginningOfDocument, to: $0.start), length: textField.offset(from: $0.start, to: $0.end)) }
            if !applyingRequest && textField.isFirstResponder && selection != nil && selection != lastSelection { parent.onCaretActivity() }
            lastSelection = selection
        }
        func textFieldDidBeginEditing(_ textField: UITextField) { if !applyingRequest { parent.onCaretActivity() }; parent.onFocus() }
        func textFieldDidEndEditing(_ textField: UITextField) { parent.onCommit() }
        func textFieldShouldReturn(_ textField: UITextField) -> Bool { parent.onCommit(); return false }
    }
}
#endif

// Local shared spreadsheet: only exact-source edits and formula RPC cross this boundary.
@MainActor final class NativeTableCoordinator: NSObject, WKNavigationDelegate, WKScriptMessageHandler {
    var table: InlineTable; var source: String; var buffer: DocumentBuffer
    weak var view: WKWebView?
    var onReady: () -> Void
    var revision = UUID().uuidString
    var ready = false
    var mounted = true
    var onUnavailable: () -> Void = {}
    var allowedURL: URL?
    private var identity: [ExactSourceText?] = []
    private var displayedRevision: String?
    private var displayedCalculationRevision: String?
    private var acceptedSource: String?
    private var acceptedRange: NSRange?
    private var writable: Bool { !buffer.readOnly && (exactUTF8(buffer.inlineMarkup.source,source) || exactUTF8(acceptedSource,source)) }
    func update(table: InlineTable, source: String) {
        self.table = table; self.source = source
        if exactUTF8(acceptedSource,source), let range = acceptedRange { self.table.range = range }
        display()
    }
    init(table: InlineTable, source: String, buffer: DocumentBuffer, onReady: @escaping () -> Void) { self.table = table; self.source = source; self.buffer = buffer; self.onReady = onReady }
    func display(force: Bool = false) {
        guard ready, NSMaxRange(table.range) <= (source as NSString).length else { return }
        let format = table.rows.first?.first?.format ?? "csv"
        let delimiter = table.rows.first?.first?.delimiter ?? ","
        let header = inlineTableAttribute("header", table: table, source: source) != "false"
        let language = inlineTableAttribute("language", table: table, source: source)
        let tableID = inlineTableAttribute("id", table: table, source: source)
        let document = buffer.file.doc_id
        let readOnly = !writable
        let hasFormulaEngine = buffer.receiptEngine != nil
        let documentPath = buffer.openDiagramFile == nil ? nil : buffer.file.path
        let key = [source, String(readOnly), String(hasFormulaEngine), NSStringFromRange(table.range), format, delimiter, documentPath,
                   String(header), language, tableID, document].map { $0.map(ExactSourceText.init) }
        if key != identity { identity = key; revision = UUID().uuidString }
        // Geometry refreshes share this same source snapshot. Replacing it
        // during a gesture would retire the grid's current selection handlers.
        let calculationRevision = buffer.receiptRevision
        guard force || displayedRevision != revision || displayedCalculationRevision != calculationRevision else { return }
        displayedRevision = revision
        displayedCalculationRevision = calculationRevision
        var snapshot: [String: Any] = [
            "revision": revision, "source": (source as NSString).substring(with: table.range),
            "format": format, "delimiter": delimiter, "readonly": readOnly, "header": header,
        ]
        snapshot["calculationRevision"] = calculationRevision
        snapshot["hasFormulaEngine"] = hasFormulaEngine
        snapshot["language"] = language
        snapshot["documentPath"] = documentPath
        if let tableID {
            var context: [String: Any] = ["source": source, "table": tableID]
            context["document"] = document
            snapshot["scope"] = context
        }
        send("update", snapshot)
    }
    private func send(_ method: String, _ value: Any) {
        guard let data = try? JSONSerialization.data(withJSONObject: value, options: [.fragmentsAllowed]), let json = String(data: data, encoding: .utf8) else { return }
        view?.evaluateJavaScript("globalThis.hickoryTable?." + method + "(" + json + ")", completionHandler: nil)
    }
    func userContentController(_ controller: WKUserContentController, didReceive message: WKScriptMessage) {
        guard mounted, view != nil, message.frameInfo.isMainFrame, message.frameInfo.request.url?.isFileURL == true,
              message.frameInfo.request.url == allowedURL, let event = message.body as? [String: Any], let kind = event["kind"] as? String else { return }
        if kind == "ready" { ready = true; display(force: true); onReady(); return }
        if kind == "unavailable" { ready = false; onUnavailable(); return }
        if kind == "formula" {
            guard let id = event["id"] as? Int, let operation = event["operation"] as? String, ["languages", "evaluate", "trace"].contains(operation) else { return }
            let engine = buffer.receiptEngine
            Task { [weak self] in
                do {
                    guard let engine else { throw EngineError(message: "Reconnect to the workspace to evaluate formulas. Table source is still editable.") }
                    var body: [String: Any]? = operation == "languages" ? nil : ["language": event["language"] as? String ?? "", "rows": event["rows"] as? [[String]] ?? []]
                    if let context = event["scope"] as? [String: Any] { body?["scope"] = context }
                    let result: NativeTableJSON = try await engine.request(operation == "languages" ? "GET" : "POST", "/api/formula/" + operation, body: body)
                    self?.reply(id, result.object, nil)
                } catch { self?.reply(id, NSNull(), error.localizedDescription) }
            }
            return
        }
        guard event["revision"] as? String == revision, exactUTF8(buffer.text,source) else { display(force: kind == "change"); return }
        if kind == "link", table.rows.first?.first?.format == "markdown", let href = event["href"] as? String {
            buffer.openDiagramFile?(href)
            return
        }
        if kind == "height", let height = event["height"] as? Double, height.isFinite {
            buffer.tableHeights[table.id] = min(1200, max(30, height)); buffer.renderMarkup?(); return
        }
        if kind == "change" {
            guard writable, let value = event["source"] as? String, value.utf16.count <= 1_000_000 else {
                display(force: true); return
            }
            let edit = NativeSourceEdit(range: table.range, text: value)
            acceptedSource = (source as NSString).replacingCharacters(in: table.range, with: value)
            acceptedRange = NSRange(location: table.range.location, length: value.utf16.count)
            if buffer.applyTableEdit?(edit, source) == true {
                table.range.length = value.utf16.count; source = buffer.text
            } else { acceptedSource = nil; acceptedRange = nil }
            display(force: true)
        }
    }
    private func reply(_ id: Int, _ value: Any, _ error: String?) {
        guard let data = try? JSONSerialization.data(withJSONObject: [id, value, error as Any? ?? NSNull()]), let json = String(data: data, encoding: .utf8) else { return }
        view?.evaluateJavaScript("globalThis.hickoryTable?.reply(..." + json + ")", completionHandler: nil)
    }
    func webViewWebContentProcessDidTerminate(_ webView: WKWebView) { ready = false; onUnavailable() }
    func webView(_ webView: WKWebView, didFailProvisionalNavigation navigation: WKNavigation!, withError error: Error) {
        ready = false; onUnavailable()
    }
    func webView(_ webView: WKWebView, decidePolicyFor navigationAction: WKNavigationAction, decisionHandler: @escaping (WKNavigationActionPolicy) -> Void) {
        decisionHandler(navigationAction.targetFrame?.isMainFrame != false && navigationAction.request.url == allowedURL ? .allow : .cancel)
    }
}
// Preserve arbitrary host JSON without evaluating or interpreting formula values locally.
indirect enum NativeTableJSON: Decodable {
    case value(Any), array([NativeTableJSON]), object([String: NativeTableJSON])
    init(from decoder: Decoder) throws {
        let c = try decoder.singleValueContainer()
        if c.decodeNil() { self = .value(NSNull()) }
        else if let v = try? c.decode(Bool.self) { self = .value(v) }
        else if let v = try? c.decode(String.self) { self = .value(v) }
        else if let v = try? c.decode(Double.self) { self = .value(v) }
        else if let v = try? c.decode([NativeTableJSON].self) { self = .array(v) }
        else { self = .object(try c.decode([String: NativeTableJSON].self)) }
    }
    var object: Any { switch self { case .value(let v): return v; case .array(let v): return v.map(\.object); case .object(let v): return v.mapValues(\.object) } }
}
@MainActor func makeNativeTableView(_ coordinator: NativeTableCoordinator, resource: URL? = nil) -> WKWebView {
    let configuration = WKWebViewConfiguration(); configuration.websiteDataStore = .nonPersistent()
    configuration.userContentController.add(coordinator, name: "table")
    configuration.userContentController.addUserScript(WKUserScript(source: "globalThis.hickoryTableMode = true", injectionTime: .atDocumentStart, forMainFrameOnly: true))
    let view = WKWebView(frame: .zero, configuration: configuration); coordinator.view = view; view.navigationDelegate = coordinator
    let owner = Bundle(for: MathCoordinator.self)
    let roots = [Bundle.main.resourceURL, Bundle.main.bundleURL, owner.resourceURL, owner.bundleURL, URL(fileURLWithPath: CommandLine.arguments[0]).deletingLastPathComponent()].compactMap { $0 }
    if let file = resource ?? nativeMathResource(roots: roots, requireSpreadsheet: true) {
        let local = file.standardizedFileURL.resolvingSymlinksInPath()
        coordinator.allowedURL = local
        view.loadFileURL(local, allowingReadAccessTo: local.deletingLastPathComponent())
    }
    return view
}
struct NativeSpreadsheet: View {
    var table: InlineTable; var source: String; var buffer: DocumentBuffer; var onReady: () -> Void; var onUnavailable: () -> Void = {}
    var body: some View { NativeSpreadsheetView(calculationRevision: buffer.receiptRevision, permissionsReadOnly: buffer.readOnly, document: buffer.file.doc_id, table: table, source: source, buffer: buffer, onReady: onReady, onUnavailable: onUnavailable) }
}
#if os(macOS)
struct NativeSpreadsheetView: NSViewRepresentable {
    var calculationRevision: String
    var permissionsReadOnly: Bool
    var document: String?
    var table: InlineTable; var source: String; var buffer: DocumentBuffer; var onReady: () -> Void; var onUnavailable: () -> Void = {}
    func makeCoordinator() -> NativeTableCoordinator { NativeTableCoordinator(table: table, source: source, buffer: buffer, onReady: onReady) }
    func makeNSView(context: Context) -> WKWebView { makeNativeTableView(context.coordinator) }
    func updateNSView(_ view: WKWebView, context: Context) { context.coordinator.onReady = onReady; context.coordinator.onUnavailable = onUnavailable; context.coordinator.update(table: table, source: source) }
    static func dismantleNSView(_ view: WKWebView, coordinator: NativeTableCoordinator) { coordinator.mounted = false; view.stopLoading(); view.configuration.userContentController.removeScriptMessageHandler(forName: "table"); view.navigationDelegate = nil }
}
#else
struct NativeSpreadsheetView: UIViewRepresentable {
    var calculationRevision: String
    var permissionsReadOnly: Bool
    var document: String?
    var table: InlineTable; var source: String; var buffer: DocumentBuffer; var onReady: () -> Void; var onUnavailable: () -> Void = {}
    func makeCoordinator() -> NativeTableCoordinator { NativeTableCoordinator(table: table, source: source, buffer: buffer, onReady: onReady) }
    func makeUIView(context: Context) -> WKWebView { makeNativeTableView(context.coordinator) }
    func updateUIView(_ view: WKWebView, context: Context) { context.coordinator.onReady = onReady; context.coordinator.onUnavailable = onUnavailable; context.coordinator.update(table: table, source: source) }
    static func dismantleUIView(_ view: WKWebView, coordinator: NativeTableCoordinator) { coordinator.mounted = false; view.stopLoading(); view.configuration.userContentController.removeScriptMessageHandler(forName: "table"); view.navigationDelegate = nil }
}
#endif

// Only inspect the current logical line. Identifier movement never waits for parsing.
private let simpleWholeWord = try! NSRegularExpression(pattern: #"[\p{L}\p{N}_][\p{L}\p{M}\p{N}_\u200C\u200D]*"#)
private let simpleWordPieces = try! NSRegularExpression(pattern: #"(?:\p{Lu}\p{M}*)+(?=\p{Lu}\p{M}*\p{Ll})|(?:\p{Lu}\p{M}*)?(?:\p{Ll}\p{M}*)+|(?:\p{Lu}\p{M}*)+|(?:\p{N}\p{M}*)+|\p{L}[\p{L}\p{M}\u200C\u200D]*"#)
func simpleWordOffset(_ source: String, at offset: Int, forward: Bool, pieces: Bool) -> Int {
    let text = source as NSString
    let probe = !forward && offset > 0 ? offset - 1 : offset
    let line = text.lineRange(for: NSRange(location: probe, length: 0))
    let matches = (pieces ? simpleWordPieces : simpleWholeWord).matches(in: text.substring(with: line), range: NSRange(location: 0, length: line.length))
    if forward { return matches.map { line.location + NSMaxRange($0.range) }.first { $0 > offset } ?? NSMaxRange(line) }
    return matches.map { line.location + $0.range.location }.last { $0 < offset } ?? line.location
}
struct EditorNavigationSettings: View {
    @AppStorage("editor.controlArrow") private var control = "native"
    @AppStorage("editor.altArrow") private var alt = "native"
    @AppStorage("editor.enterIndent") private var enter = "smart"
    var body: some View {
        Menu("Editor settings") {
            Picker("Enter indentation", selection: $enter) {
                Text("Smart (language rules)").tag("smart")
                Text("Match current line").tag("copy")
                Text("None").tag("none")
            }
            Picker("Ctrl + Left/Right", selection: $control) { modes }
            Picker("Alt/Option + Left/Right", selection: $alt) { modes }
        }
    }
    private var modes: some View {
        Group {
            Text("Platform default").tag("native")
            Text("Whole identifier").tag("word")
            Text("Word pieces").tag("pieces")
        }
    }
}
// Remember the anchor only while this exact selection continues a Shift gesture.
struct SimpleWordSelection {
    var previous: NSRange?; var anchor = 0; var head = 0
    mutating func move(_ source: String, selection: NSRange, forward: Bool, extend: Bool, pieces: Bool) -> NSRange {
        if !extend || previous != selection {
            anchor = forward ? selection.location : NSMaxRange(selection)
            head = forward ? NSMaxRange(selection) : selection.location
        }
        if extend || selection.length == 0 { head = simpleWordOffset(source, at: head, forward: forward, pieces: pieces) }
        if !extend { anchor = head }
        let result = NSRange(location: min(anchor, head), length: abs(head - anchor))
        previous = result; return result
    }
}

#if os(macOS)
import AppKit
final class SimpleMacTextView: NSTextView {
    var copiedQuote: ((NSRange) -> String?)?
    override func copy(_ sender: Any?) {
        guard selectedRanges.count == 1, let contents = copiedQuote?(selectedRange()) else { super.copy(sender); return }
        NSPasteboard.general.clearContents()
        NSPasteboard.general.setString(contents, forType: .string)
    }
    var submitMessage: (() -> Void)?
    private let sourceUndoManager=UndoManager()
    override var undoManager:UndoManager? {sourceUndoManager}
    // Intercept only explicitly configured arrow chords. All other editing stays in AppKit.
    private var wordSelection = SimpleWordSelection()
    override func keyDown(with event: NSEvent) {
        let modifiers = event.modifierFlags.intersection(.deviceIndependentFlagsMask).subtracting(.capsLock)
        if event.charactersIgnoringModifiers == "\r", modifiers == [.shift], !hasMarkedText(), let submitMessage {
            submitMessage(); return
        }
        if event.charactersIgnoringModifiers == "\r", modifiers == [.command, .shift],
           !hasMarkedText(), let link = linkAtCaret?(), openLink?(link) == true { return }
        let flags = event.modifierFlags.intersection(.deviceIndependentFlagsMask).subtracting([.capsLock, .numericPad, .function, .shift])
        let key = flags == .control ? "editor.controlArrow" : flags == .option ? "editor.altArrow" : ""
        let mode = UserDefaults.standard.string(forKey: key) ?? "native"
        guard ["word", "pieces"].contains(mode), [123, 124].contains(event.keyCode), !hasMarkedText() else {
            wordSelection.previous = nil; super.keyDown(with: event); return
        }
        let range = wordSelection.move(string, selection: selectedRange(), forward: event.keyCode == 124, extend: event.modifierFlags.contains(.shift), pieces: mode == "pieces")
        setSelectedRange(range); scrollRangeToVisible(range)
    }
    var toggleEquationAtPoint: ((CGPoint) -> Bool)?
    var followLinkAtPoint: ((CGPoint) -> Bool)?
    var linkAtCaret: (() -> InlineLink?)?
    var openLink: ((InlineLink) -> Bool)?
    override func menu(for event: NSEvent) -> NSMenu? {
        let menu = super.menu(for: event)
        guard !hasMarkedText(), let link = linkAtCaret?() else { return menu }
        let contextual = menu?.copy() as? NSMenu ?? NSMenu()
        let item = NSMenuItem(title: "Open Link", action: #selector(openContextLink(_:)), keyEquivalent: "\r")
        item.keyEquivalentModifierMask = [.command, .shift]
        item.target = self
        item.representedObject = link
        contextual.insertItem(item, at: 0)
        return contextual
    }
    @objc private func openContextLink(_ sender: NSMenuItem) {
        guard !hasMarkedText(), let link = sender.representedObject as? InlineLink else { return }
        _ = openLink?(link)
    }
    override func mouseDown(with event: NSEvent) {
        let point = convert(event.locationInWindow, from: nil)
        if !hasMarkedText() {
            if event.modifierFlags.contains(.command), followLinkAtPoint?(point) == true { return }
            if event.clickCount == 2, toggleEquationAtPoint?(point) == true { return }
        }
        super.mouseDown(with: event)
    }
    var tableRanges: [NSRange] = [] { didSet { needsDisplay = true } }
    var tableCaretIsHidden: Bool { tableCaretHidden(selectedRange(), tables: tableRanges) }
    override func drawInsertionPoint(in rect: NSRect, color: NSColor, turnedOn flag: Bool) {
        // Let the off phase erase an existing caret when a grid attaches.
        if !flag || !tableCaretIsHidden { super.drawInsertionPoint(in: rect, color: color, turnedOn: flag) }
    }

    override func accessibilityChildren() -> [Any]? {
        (super.accessibilityChildren() ?? []) + subviews.filter { $0 is NSHostingView<NativeTableGrid> }
    }
    var beginSnippetComposition: ((NSRange) -> Bool)?
    var reportSnippetComposition: ((_ finished: Bool) -> Void)?
    private(set) var ownsSnippetComposition = false
    private(set) var performingMarkedInput = false

    private func performMarkedInput(start: NSRange? = nil, _ apply: () -> Void) {
        if performingMarkedInput { apply(); return }
        if !ownsSnippetComposition, let start {
            ownsSnippetComposition = beginSnippetComposition?(start) == true
        }
        guard ownsSnippetComposition else { apply(); return }

        performingMarkedInput = true
        let history = undoManager
        history?.disableUndoRegistration()
        apply()
        history?.enableUndoRegistration()
        performingMarkedInput = false

        if !hasMarkedText() {
            ownsSnippetComposition = false
            reportSnippetComposition?(true)
        } else {
            reportSnippetComposition?(false)
        }
    }

    override func setMarkedText(_ value: Any, selectedRange: NSRange, replacementRange: NSRange) {
        performMarkedInput(start: replacementRange.location == NSNotFound ? self.selectedRange() : replacementRange) {
            super.setMarkedText(value, selectedRange: selectedRange, replacementRange: replacementRange)
        }
    }

    override func insertText(_ value: Any, replacementRange: NSRange) {
        performMarkedInput {
            super.insertText(value, replacementRange: replacementRange)
        }
    }

    override func unmarkText() {
        performMarkedInput { super.unmarkText() }
    }
}
#else
import UIKit
private final class SimpleTextView: UITextView {
    var copiedQuote: ((NSRange) -> String?)?
    override func copy(_ sender: Any?) {
        guard let contents = copiedQuote?(selectedRange) else { super.copy(sender); return }
        UIPasteboard.general.string = contents
    }
    var submitMessage: (() -> Void)?
    private let sourceUndoManager=UndoManager()
    override var undoManager:UndoManager? {sourceUndoManager}
    var snippetActive:(()->Bool)?
    var snippetCommand:((Int)->Bool)?
    var linkAtCaret: (() -> InlineLink?)?
    var openLink: ((InlineLink) -> Bool)?
    // UIKit retains native input and composition; hardware arrows only change selection.
    private var wordSelection = SimpleWordSelection()
    override var keyCommands: [UIKeyCommand]? {
        guard markedTextRange == nil else { return super.keyCommands }
        var commands = super.keyCommands ?? []
        if submitMessage != nil {
            let newline = UIKeyCommand(input: "\r", modifierFlags: [], action: #selector(insertComposerNewline))
            newline.wantsPriorityOverSystemBehavior = true
            commands.append(newline)
            let command = UIKeyCommand(input: "\r", modifierFlags: .shift, action: #selector(sendMessage))
            command.discoverabilityTitle = "Send message"
            command.wantsPriorityOverSystemBehavior = true
            commands.append(command)
        }
        if linkAtCaret?() != nil {
            let command = UIKeyCommand(input: "\r", modifierFlags: [.command, .shift], action: #selector(openCaretLink))
            command.discoverabilityTitle = "Open Link"
            commands.append(command)
        }
        if snippetActive?()==true {
            for (input,flags) in [("\t",UIKeyModifierFlags()),("\t",.shift),(UIKeyCommand.inputEscape,UIKeyModifierFlags())] {
                let command=UIKeyCommand(input:input,modifierFlags:flags,action:#selector(navigateTemplate(_:)));command.wantsPriorityOverSystemBehavior=true;commands.append(command)
            }
        }
        for (key, modifier) in [("editor.controlArrow", UIKeyModifierFlags.control), ("editor.altArrow", .alternate)] {
            guard ["word", "pieces"].contains(UserDefaults.standard.string(forKey: key) ?? "native") else { continue }
            for input in [UIKeyCommand.inputLeftArrow, UIKeyCommand.inputRightArrow] {
                for flags in [modifier, modifier.union(.shift)] {
                    let command = UIKeyCommand(input: input, modifierFlags: flags, action: #selector(moveIdentifier(_:)))
                    command.wantsPriorityOverSystemBehavior = true; commands.append(command)
                }
            }
        }
        return commands
    }
    @objc private func insertComposerNewline() {
        guard isEditable, markedTextRange == nil else { return }
        insertText("\n")
    }
    @objc private func sendMessage() { guard markedTextRange == nil else { return }; submitMessage?() }
    @objc private func openCaretLink() {
        guard markedTextRange == nil, let link = linkAtCaret?() else { return }
        _ = openLink?(link)
    }
    @objc private func navigateTemplate(_ command:UIKeyCommand){guard markedTextRange==nil else{return};_ = snippetCommand?(command.input==UIKeyCommand.inputEscape ? 0:command.modifierFlags.contains(.shift) ? -1:1)}
    @objc private func moveIdentifier(_ command: UIKeyCommand) {
        guard markedTextRange == nil else { return }
        let key = command.modifierFlags.contains(.control) ? "editor.controlArrow" : "editor.altArrow"
        let mode = UserDefaults.standard.string(forKey: key) ?? "native"
        guard ["word", "pieces"].contains(mode) else { return }
        selectedRange = wordSelection.move(text ?? "", selection: selectedRange, forward: command.input == UIKeyCommand.inputRightArrow, extend: command.modifierFlags.contains(.shift), pieces: mode == "pieces")
        scrollRangeToVisible(selectedRange)
    }
    var tableRanges: [NSRange] = [] { didSet { setNeedsDisplay() } }
    override func caretRect(for position: UITextPosition) -> CGRect {
        let selection = NSRange(location: offset(from: beginningOfDocument, to: position), length: 0)
        return tableCaretHidden(selection, tables: tableRanges) ? .zero : super.caretRect(for: position)
    }

    var beginSnippetComposition: ((NSRange) -> Bool)?
    var reportSnippetComposition: ((_ finished: Bool) -> Void)?
    private(set) var ownsSnippetComposition = false
    private(set) var performingMarkedInput = false

    private func performMarkedInput(start: NSRange? = nil, _ apply: () -> Void) {
        if performingMarkedInput { apply(); return }
        if !ownsSnippetComposition, let start {
            ownsSnippetComposition = beginSnippetComposition?(start) == true
        }
        guard ownsSnippetComposition else { apply(); return }

        performingMarkedInput = true
        let history = undoManager
        history?.disableUndoRegistration()
        apply()
        history?.enableUndoRegistration()
        performingMarkedInput = false

        if markedTextRange == nil {
            ownsSnippetComposition = false
            reportSnippetComposition?(true)
        } else {
            reportSnippetComposition?(false)
        }
    }

    override func setMarkedText(_ value: String?, selectedRange: NSRange) {
        guard isEditable else { return }
        performMarkedInput(start: self.selectedRange) {
            super.setMarkedText(value, selectedRange: selectedRange)
        }
    }

    override func insertText(_ value: String) {
        // UIKeyInput can be called directly without consulting the delegate.
        guard isEditable else { return }
        performMarkedInput { super.insertText(value) }
    }

    override func deleteBackward() {
        guard isEditable else { return }
        super.deleteBackward()
    }

    override func replace(_ range: UITextRange, withText text: String) {
        guard isEditable else { return }
        super.replace(range, withText: text)
    }

    override func unmarkText() {
        performMarkedInput { super.unmarkText() }
    }
}
#endif
func simpleIndent(_ source: String) -> String {
    let sample = String(source.prefix(16_384))
    let prefixes = sample.components(separatedBy: "\n").prefix(200).filter { !$0.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty }.map { String($0.prefix { $0 == " " || $0 == "\t" }) }
    if prefixes.filter({ $0.hasPrefix("\t") }).count > prefixes.filter({ $0.hasPrefix(" ") }).count { return "\t" }
    func gcd(_ a: Int, _ b: Int) -> Int { b == 0 ? a : gcd(b, a % b) }
    let width = prefixes.filter { !$0.isEmpty && $0.allSatisfy { $0 == " " } }.reduce(0) { gcd($0, $1.count) }
    return String(repeating: " ", count: width > 0 && width <= 8 ? width : 4)
}
func simpleNewlineText(_ source: String, at offset: Int) -> String {
    let text = source as NSString
    let line = text.lineRange(for: NSRange(location: offset, length: 0))
    let before = text.substring(with: NSRange(location: line.location, length: offset - line.location))
    let prefix = String(before.prefix { $0 == " " || $0 == "\t" })
    // Prefer this line's terminator, then the preceding one for the final line.
    let end = NSMaxRange(line)
    let hasTerminator = end > line.location && text.character(at: end - 1) == 10
    let crlf = hasTerminator
        ? end >= 2 && text.substring(with: NSRange(location: end - 2, length: 2)) == "\r\n"
        : line.location >= 2 && text.substring(with: NSRange(location: line.location - 2, length: 2)) == "\r\n"
    return (crlf ? "\r\n" : "\n") + prefix
}
func simpleBackspaceRange(_ source: String, at offset: Int) -> NSRange? {
    let text = source as NSString
    guard offset > 0 else { return nil }
    let line = text.lineRange(for: NSRange(location: offset, length: 0))
    let before = text.substring(with: NSRange(location: line.location, length: offset - line.location))
    guard !before.isEmpty, before.allSatisfy({ $0 == " " }) else { return nil }
    let unit = simpleIndent(source)
    guard unit != "\t" else { return nil }
    let count = (before.utf16.count - 1) % unit.count + 1
    return NSRange(location: offset - count, length: count)
}
func simpleTabText(_ source: String, at offset: Int) -> String {
    let unit = simpleIndent(source)
    if unit == "\t" { return unit }
    let text = source as NSString
    let start = text.lineRange(for: NSRange(location: offset, length: 0)).location
    let column = offset - start
    return String(repeating: " ", count: unit.count - column % unit.count)
}
func simpleTabEdits(_ source: String, range: NSRange, outdent: Bool = false) -> [NativeSourceEdit] {
    if range.length == 0 && !outdent { return [NativeSourceEdit(range: range, text: simpleTabText(source, at: range.location))] }
    let text = source as NSString, unit = simpleIndent(source)
    var offset = text.lineRange(for: NSRange(location: range.location, length: 0)).location
    let end = NSMaxRange(range)
    var edits: [NativeSourceEdit] = []
    repeat {
        if outdent {
            let tail = text.substring(from: offset)
            let count = tail.hasPrefix("\t") ? 1 : min(unit == "\t" ? 4 : unit.count, tail.prefix { $0 == " " }.count)
            if count > 0 { edits.append(NativeSourceEdit(range: NSRange(location: offset, length: count), text: "")) }
        } else { edits.append(NativeSourceEdit(range: NSRange(location: offset, length: 0), text: unit)) }
        let next = NSMaxRange(text.lineRange(for: NSRange(location: offset, length: 0)))
        if next <= offset { break }; offset = next
    } while offset < end
    return edits
}
struct SimpleFold: Equatable {
    var line: Int
    var range: NSRange
}
func simpleFoldRanges(_ source: String, markdown: Bool = false) -> [SimpleFold] {
    let rows = source.components(separatedBy: "\n")
    var stack: [(line: Int, indent: Int, end: Int)] = [], result: [SimpleFold] = []
    var offset = 0, last = -1
    for line in 0...rows.count {
        let row = line < rows.count ? rows[line] : nil
        if let row, row.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty { offset += row.utf16.count + 1; continue }
        let indent = row.map { $0.prefix { $0 == " " || $0 == "\t" }.reduce(0) { $0 + ($1 == "\t" ? 4 : 1) } } ?? -1
        while let header = stack.last, header.indent >= indent {
            stack.removeLast()
            if last > header.end { result.append(SimpleFold(line: header.line, range: NSRange(location: header.end, length: last - header.end))) }
        }
        if let row { last = offset + row.utf16.count; stack.append((line, indent, last)); offset = last + 1 }
    }
    if markdown {
        var headings: [(line: Int, level: Int, end: Int)] = [], stack: [(line: Int, level: Int, end: Int)] = []
        var starts: [Int] = []
        var position = 0, fence = "", payload = "", previous: (line: Int, end: Int)?
        var patch = false
        var patchRows = Set<Int>()
        var patchFolds: [SimpleFold] = []
        var hunk: (line: Int, start: Int, end: Int, old: Int, new: Int)?
        let headerPattern = try! NSRegularExpression(pattern: #"^@@ -[0-9]+(?:,([0-9]+))? \+[0-9]+(?:,([0-9]+))? @@"#)
        // A fold follows unified-diff line counts. The raw indentation pass
        // treats the context prefix as Python indentation and would hide a
        // different subset of the same hunk.
        func finishHunk() {
            if let hunk, hunk.old == 0, hunk.new == 0, hunk.end > hunk.start {
                patchFolds.append(SimpleFold(line: hunk.line, range: NSRange(location: hunk.start, length: hunk.end - hunk.start)))
            }
            hunk = nil
        }
        func patchRow(_ row: String, line: Int, end: Int) {
            let text = row as NSString
            if let match = headerPattern.firstMatch(in: row, range: NSRange(location: 0, length: text.length)) {
                finishHunk()
                func count(_ group: Int) -> Int? {
                    let range = match.range(at: group)
                    return range.location == NSNotFound ? 1 : Int(text.substring(with: range))
                }
                if let old = count(1), let new = count(2) {
                    hunk = (line, end, end, old, new)
                }
            } else if var current = hunk {
                switch row.first {
                case " " where current.old > 0 && current.new > 0: current.old -= 1; current.new -= 1
                case "-" where current.old > 0: current.old -= 1
                case "+" where current.new > 0: current.new -= 1
                case "\\" where row == #"\ No newline at end of file"#: break
                default: finishHunk(); return
                }
                current.end = end
                hunk = current
            }
        }
        let opening = try! NSRegularExpression(pattern: #"^ {0,3}<[\w.-]+:(file|copy|cut|exec|patch|entry|diagram|math|table)\b[^>]*>"#)
        for (line, row) in rows.enumerated() {
            starts.append(position)
            let rowEnd = position + row.utf16.count
            let end = row.utf16.last == 13 ? rowEnd - 1 : rowEnd
            position = rowEnd + 1
            let trimmed = row.trimmingCharacters(in: .whitespacesAndNewlines)
            if !payload.isEmpty {
                if trimmed.contains(":" + payload + ">"), trimmed.contains("</") {
                    if patch { finishHunk() }
                    payload = ""; patch = false
                } else if patch {
                    patchRows.insert(line)
                    patchRow(row.utf16.last == 13 ? String(row.dropLast()) : row, line: line, end: end)
                }
                previous = nil; continue
            }
            if !fence.isEmpty {
                if trimmed.allSatisfy({ $0 == fence.first! }), trimmed.count >= fence.count {
                    if patch { finishHunk() }
                    fence = ""; patch = false
                } else if patch {
                    patchRows.insert(line)
                    patchRow(row.utf16.last == 13 ? String(row.dropLast()) : row, line: line, end: end)
                }
                previous = nil; continue
            }
            if row.prefix(while: { $0 == " " }).count <= 3, trimmed.hasPrefix("```") || trimmed.hasPrefix("~~~") {
                fence = String(trimmed.prefix { $0 == trimmed.first! })
                let kind = trimmed.dropFirst(fence.count).split(whereSeparator: { $0.isWhitespace }).first
                patch = kind == "patch" || kind == "diff"
                if patch { patchRows.insert(line) }
                previous = nil; continue
            }
            if let match = opening.firstMatch(in: row, range: NSRange(location: 0, length: row.utf16.count)) {
                if !trimmed.contains("</"), !trimmed.hasSuffix("/>") {
                    payload = (row as NSString).substring(with: match.range(at: 1))
                    patch = payload == "patch"
                    if patch { patchRows.insert(line) }
                }
                previous = nil; continue
            }
            let indent = row.prefix { $0 == " " }.count
            if indent <= 3, let match = row.range(of: #"^ {0,3}(#{1,6})(?:[ \t]+|$)"#, options: .regularExpression) {
                let level = row[match].filter { $0 == "#" }.count
                headings.append((line, level, end)); previous = nil
            } else if indent <= 3, let title = previous, !trimmed.isEmpty,
                      trimmed.allSatisfy({ $0 == "=" }) || trimmed.allSatisfy({ $0 == "-" }) {
                headings.append((title.line, trimmed.first == "=" ? 1 : 2, title.end)); previous = nil
            } else { previous = !trimmed.isEmpty && indent <= 3 && !row.hasPrefix("\t") ? (line, end) : nil }
        }
        if patch { finishHunk() }
        result.removeAll { patchRows.contains($0.line) }
        result += patchFolds
        for header in headings + [(rows.count, 0, source.utf16.count)] {
            while let top = stack.last, top.level >= header.level {
                stack.removeLast()
                let end = header.line < rows.count ? starts[header.line] - 1 : source.utf16.count
                if end > top.end {
                    result.removeAll { $0.line == top.line }
                    result.append(SimpleFold(line: top.line, range: NSRange(location: top.end, length: end - top.end)))
                }
            }
            stack.append(header)
        }
    }
    return result.sorted { $0.range.location < $1.range.location }
}
#if os(macOS)
private typealias FoldFont = NSFont
private typealias EquationHost = NSHostingView<AnyView>
private typealias DiagramHost = NSHostingView<NativeDiagramPreview>
#else
private typealias FoldFont = UIFont
private typealias EquationHost = UIHostingController<AnyView>
private typealias DiagramHost = UIHostingController<NativeDiagramPreview>
#endif
private struct EquationLayout: Equatable {
    var range: NSRange
    var size: CGSize
}
@MainActor private final class FoldLayout: NSObject, @preconcurrency NSLayoutManagerDelegate {
    var ranges: [NSRange] = []
    var tables: [InlineTable] = []
    var diagrams: [InlineDiagram] = []
    var equations: [EquationLayout] = []
    private var markdownGlyphs: [Int: UInt16] = [:]
    private let checkboxFallback: [UInt16: UInt16] = [0x2610: 0x25A1, 0x2611: 0x2713]

    // Only changed marker positions need new glyphs when the caret moves.
    func presentMarkdown(_ glyphs: [Int: UInt16], in manager: NSLayoutManager) {
        guard glyphs != markdownGlyphs else { return }
        let length = manager.textStorage?.length ?? 0
        let changed = Set(glyphs.keys).union(markdownGlyphs.keys).filter { $0 < length && glyphs[$0] != markdownGlyphs[$0] }
        markdownGlyphs = glyphs
        for offset in changed {
            manager.invalidateGlyphs(forCharacterRange: NSRange(location: offset, length: 1), changeInLength: 0, actualCharacterRange: nil)
        }
    }

    func tableContains(_ offset: Int) -> Bool { tables.contains { offset > $0.range.location && NSLocationInRange(offset, $0.range) } }
    func contains(_ offset: Int) -> Bool { equations.contains { offset > $0.range.location && NSLocationInRange(offset, $0.range) } || tableContains(offset) || diagrams.contains { offset > $0.range.location && NSLocationInRange(offset, $0.range) } || ranges.contains { NSLocationInRange(offset, $0) } }
    func layoutManager(_ manager: NSLayoutManager, shouldGenerateGlyphs glyphs: UnsafePointer<CGGlyph>, properties: UnsafePointer<NSLayoutManager.GlyphProperty>, characterIndexes: UnsafePointer<Int>, font: FoldFont, forGlyphRange range: NSRange) -> Int {
        guard !ranges.isEmpty || !tables.isEmpty || !diagrams.isEmpty || !equations.isEmpty || !markdownGlyphs.isEmpty else { return 0 }
        var props = Array(UnsafeBufferPointer(start: properties, count: range.length))
        var drawn = Array(UnsafeBufferPointer(start: glyphs, count: range.length))
        var markerFont: CTFont?
        for i in 0..<range.length {
            let offset = characterIndexes[i]
            if equations.contains(where: { $0.range.location == offset }) { props[i] = .controlCharacter }
            else if contains(offset) { props[i] = .null }
            else if var marker = markdownGlyphs[offset] {
                if marker == 0 { props[i] = .null }
                else {
                    let fontForMarker = markerFont ?? CTFontCreateWithName(font.fontName as CFString, font.pointSize, nil)
                    markerFont = fontForMarker
                    var glyph: CGGlyph = 0
                    if CTFontGetGlyphsForCharacters(fontForMarker, &marker, &glyph, 1) {
                        drawn[i] = glyph
                    } else if var fallback = checkboxFallback[marker],
                              CTFontGetGlyphsForCharacters(fontForMarker, &fallback, &glyph, 1) {
                        drawn[i] = glyph
                    }
                }
            }
        }
        drawn.withUnsafeBufferPointer { drawn in
            props.withUnsafeBufferPointer {
                manager.setGlyphs(drawn.baseAddress!, properties: $0.baseAddress!, characterIndexes: characterIndexes, font: font, forGlyphRange: range)
            }
        }
        return range.length
    }
    func layoutManager(_ manager: NSLayoutManager, shouldUse action: NSLayoutManager.ControlCharacterAction, forControlCharacterAt index: Int) -> NSLayoutManager.ControlCharacterAction {
        if equations.contains(where: { $0.range.location == index }) { return .whitespace }
        return contains(index) ? .zeroAdvancement : action
    }
    func layoutManager(_ manager: NSLayoutManager, boundingBoxForControlGlyphAt glyphIndex: Int, for textContainer: NSTextContainer, proposedLineFragment proposedRect: CGRect, glyphPosition: CGPoint, characterIndex: Int) -> CGRect {
        guard let equation = equations.first(where: { $0.range.location == characterIndex }) else { return .zero }
        return CGRect(origin: .zero, size: equation.size)
    }
    func layoutManager(_ manager: NSLayoutManager, shouldBreakLineByWordBeforeCharacterAt index: Int) -> Bool {
        !equations.contains { index > $0.range.location && NSLocationInRange(index, $0.range) }
    }
    func layoutManager(_ manager: NSLayoutManager, shouldSetLineFragmentRect rect: UnsafeMutablePointer<CGRect>, lineFragmentUsedRect used: UnsafeMutablePointer<CGRect>, baselineOffset: UnsafeMutablePointer<CGFloat>, in textContainer: NSTextContainer, forGlyphRange glyphs: NSRange) -> Bool {
        let characters = manager.characterRange(forGlyphRange: glyphs, actualGlyphRange: nil)
        let height = equations.filter { NSLocationInRange($0.range.location, characters) }.map(\.size.height).max() ?? 0
        let extra = max(0, height - rect.pointee.height)
        guard extra > 0 else { return false }
        rect.pointee.size.height += extra; used.pointee.size.height += extra
        baselineOffset.pointee += extra / 2
        return true
    }
}
#if os(macOS)
@MainActor final class MacEditor: NSObject, NSTextViewDelegate {
    private let sourceEditorIdentity=UUID()
    let scroll = NSScrollView(frame: NSRect(x: 0, y: 0, width: 600, height: 400))
    let text = SimpleMacTextView(frame: NSRect(x: 0, y: 0, width: 600, height: 400))
    let buffer: DocumentBuffer
    private let folding = FoldLayout()
    private let sourceDraftHistory=SourceDraftHistory()
    private var markedSnippet:CapturedMarkedSnippet?
    private var tableViewport: CGPoint?
    private var diagramSource = ""
    private var diagramAnchor: Int?
    private var equationViews: [Int: EquationHost] = [:]
    private var equationSource = ""
    private var mathSource = ""
    private var mathViews: [Int: EquationHost] = [:]
    private var mathMeasurements: [Int: CGSize] = [:]
    private var mathRequests: [Int: UUID] = [:]
    private var diagramViews: [Int: DiagramHost] = [:]
    private var tableSnapshot = InlineSnapshot(source: "")
    private var tableSource = ""
    private var tableParseCurrent = false
    private var tableSourceVisible = false
    private var sourceTable: Int?
    #if os(macOS)
    private var tableViews: [Int: NSHostingView<NativeTableGrid>] = [:]
    #else
    private var tableViews: [Int: UIHostingController<NativeTableGrid>] = [:]
    #endif
    private var paintedInlineSource = ""
    private var hadInlineFonts = false
    private var wheelMonitor: Any?
    private weak var wheelOwner: NSScrollView?
    private var wheelPointer: NSPoint?
    private var observer: NSObjectProtocol?
    init(_ buffer: DocumentBuffer) {
        self.buffer = buffer
        super.init()
        text.isRichText = false; text.isAutomaticQuoteSubstitutionEnabled = false
        text.isAutomaticDashSubstitutionEnabled = false; text.isAutomaticTextReplacementEnabled = false
        text.isAutomaticSpellingCorrectionEnabled = false; text.isContinuousSpellCheckingEnabled = false
        text.font = .monospacedSystemFont(ofSize: 13, weight: .regular)
        let tabs = NSMutableParagraphStyle(); tabs.tabStops = []
        tabs.defaultTabInterval = ("    " as NSString).size(withAttributes: [.font: text.font ?? .monospacedSystemFont(ofSize: 13, weight: .regular)]).width
        text.typingAttributes[.paragraphStyle] = tabs
        text.isVerticallyResizable = true; text.isHorizontallyResizable = false
        text.autoresizingMask = [.width]; text.textContainer?.widthTracksTextView = true
        text.textContainerInset = NSSize(width: 8, height: 8)
        text.string = buffer.text; text.isEditable = !buffer.readOnly; text.allowsUndo = true
        text.setSelectedRange(buffer.selection)
        text.setAccessibilityLabel(buffer.file.path); text.delegate = self
        text.toggleEquationAtPoint = { [weak self] point in
            guard let self, let manager = self.text.layoutManager, let container = self.text.textContainer else { return false }
            let origin = self.text.textContainerOrigin
            return self.buffer.restoreEquationPreview(at: CGPoint(x: point.x - origin.x, y: point.y - origin.y),
                                                      layout: manager, container: container)
        }
        text.followLinkAtPoint = { [weak self] point in
            guard let self, let manager = self.text.layoutManager, let container = self.text.textContainer else { return false }
            let origin = self.text.textContainerOrigin
            return self.buffer.followInlineLink(at: CGPoint(x: point.x - origin.x, y: point.y - origin.y), layout: manager, container: container)
        }
        text.linkAtCaret = { [weak self] in
            guard let self else { return nil }
            return self.buffer.inlineLink(at: self.text.selectedRange(), in: self.text.string)
        }
        text.openLink = { [weak self] link in
            guard let self, self.text.linkAtCaret?() == link else { return false }
            return self.buffer.followInlineLink(link)
        }
        text.beginSnippetComposition = { [weak self] range in
            guard let self else{return false}
            self.unfoldAll()
            guard self.text.undoManager != nil,self.text.isEditable,self.text.window != nil,
                  let capture=CapturedMarkedSnippet(buffer:self.buffer,source:self.text.string,selection:self.text.selectedRange(),range:range) else{
                self.buffer.clearMulti()
                _ = self.buffer.snippets.finish()
                return false
            }
            self.markedSnippet=capture
            return true
        }
        text.reportSnippetComposition = { [weak self] finished in
            guard let self else{return}
            self.buffer.selection = self.text.selectedRange(); self.buffer.text = self.text.string
            guard finished,let lease=self.markedSnippet else{return}
            self.markedSnippet=nil
            let observed=self.text.string
            let selected=self.text.selectedRange()
            completeMarkedSnippet(
                lease, buffer:self.buffer, observed:observed, selected:selected,
                history:self.sourceDraftHistory, manager:self.text.undoManager,
                admitted:self.text.isEditable && self.text.window != nil,
                apply:{[weak self] edits,source,selection in self?.applySourceEdits(edits,source:source,resultSelection:selection,manageUndo:false) ?? false}
            )
        }
        scroll.hasVerticalScroller = true; scroll.documentView = text
        // Retain the wheel recipient across gestures until the pointer moves.
        // Tables and WebKit previews can move under a stationary mouse.
        wheelMonitor = NSEvent.addLocalMonitorForEvents(matching: [.scrollWheel, .mouseMoved, .leftMouseDragged, .rightMouseDragged, .otherMouseDragged, .leftMouseDown, .rightMouseDown]) { [weak self] event in
            guard let self else { return event }
            let pointer = NSEvent.mouseLocation
            // Observe movement itself: moving away and returning to the same
            // screen pixel must still release the previous wheel recipient.
            if self.wheelPointer != pointer || event.type != .scrollWheel { self.wheelOwner = nil; self.wheelPointer = pointer }
            guard event.type == .scrollWheel, event.window === self.scroll.window,
                  self.scroll.bounds.contains(self.scroll.convert(event.locationInWindow, from: nil)) else { return event }
            // The floating form overlaps TextKit's frame. Leave its wheel
            // events to WebKit even if the editor held the stationary pointer.
            if self.buffer.elementFormContainsWheel(at: event.locationInWindow, in: event.window) {
                self.wheelOwner = nil
                return event
            }
            let hit = self.scroll.hitTest(self.scroll.convert(event.locationInWindow, from: nil))
            let target = (hit as? NSScrollView) ?? hit?.enclosingScrollView ?? self.scroll
            if self.wheelOwner == nil { self.wheelOwner = target }
            if let owner = self.wheelOwner, owner === self.scroll || owner !== target {
                owner.scrollWheel(with: event); return nil
            }
            return event
        }
        buffer.navigate = { [weak self] fraction in
            guard let self else { return }
            self.unfoldAll()
            let source = self.text.string as NSString
            let offset = min(source.length, max(0, Int(Double(source.length) * fraction)))
            self.text.scrollRangeToVisible(source.lineRange(for: NSRange(location: offset, length: 0)))
        }
        buffer.renderLanguage = { [weak self] tokens, source in self?.paint(tokens, source: source) }
        text.copiedQuote = { [weak self] selection in
            guard let self, exactUTF8(self.buffer.inlineMarkup.source, self.text.string) else { return nil }
            return self.buffer.inlineMarkup.quoteContents(selection)
        }
        buffer.renderMarkup = { [weak self] in guard let self else { return }; self.paint(self.buffer.language?.tokens ?? [], source: self.buffer.text); self.measureDebugLines() }
        buffer.applyLanguageCompletion = { [weak self] range, insert, source, selection in
            guard let self, self.text.selectedRange() == selection else { return false }
            return self.applySourceEdits([NativeSourceEdit(range: range, text: insert)], source: source)
        }
        buffer.revealDebugLine = { [weak self] line in
            guard let self, let offset = languageOffset(self.buffer.text, ["line": line, "character": 0]) else { return false }
            self.unfoldAll(); self.text.scrollRangeToVisible(NSRange(location: offset, length: 0));return true
        }
        scroll.contentView.postsBoundsChangedNotifications = true
        observer = NotificationCenter.default.addObserver(forName: NSView.boundsDidChangeNotification, object: scroll.contentView, queue: .main) { [weak self] _ in Task { @MainActor in self?.measureDebugLines() } }
        text.layoutManager?.delegate = folding
        buffer.toggleFold = { [weak self] line in self?.toggleFold(line) }
        buffer.focusSource = { [weak self] in guard let self else { return }; self.text.window?.makeFirstResponder(self.text) }
        buffer.captureRecoveryViewport={ [weak self] in self?.scroll.contentView.bounds.origin ?? .zero }
        buffer.sourceEditorState={ [weak self] in guard let self else{return nil};return NativeSourceEditorState(identity:self.sourceEditorIdentity,source:self.text.string,selection:self.text.selectedRange(),composing:self.text.hasMarkedText(),editable:self.text.isEditable && !self.buffer.readOnly && !self.buffer.saving,attached:self.text.window != nil,selections:max(self.text.selectedRanges.count,self.buffer.multiRanges.count))}
        buffer.applyCompletionDraft = { [weak self] draft,initial in
            guard let self,self.text.selectedRange()==initial,self.text.selectedRanges.count==1,exactUTF8(draft.path,self.buffer.file.path) else{return false}
            let selection=NSRange(location:draft.selection.from,length:draft.selection.to-draft.selection.from)
            let after:SourceSnippetHistoryContext?
            if let batch=draft.caretBatch {
                guard exactUTF8(self.buffer.multiSource,draft.before),self.buffer.multiRanges==batch.before,
                      batch.before.first==initial,batch.after.first==selection else{return false}
                after=SourceSnippetHistoryContext(fields:[],active:0,current:0,parallel:batch.parallel,ranges:batch.after)
            }else{
                after=(draft.continuing ?? beginSnippet(draft.plan.patches.first?.after ?? draft.before,fields:draft.fields))
                    .map{SourceSnippetHistoryContext(fields:$0.fields,active:$0.active,current:$0.current,scopes:$0.scopes)}
            }
            return self.sourceDraftHistory.commit(draft.plan,path:draft.path,before:draft.before,selectionBefore:initial,selectionAfter:selection,contextBefore:self.buffer.snippets.historyContext,contextAfter:after,manager:self.text.undoManager,
                read:{[weak self] in self?.buffer.text ?? ""},apply:{[weak self] edits,source,selected in self?.applySourceEdits(edits,source:source,resultSelection:selected,manageUndo:false) ?? false},notice:{[weak self] text in self?.buffer.snippets.reportFailure(text)},restoreContext:{[weak self] value in self?.buffer.snippets.restoreHistory(value)},contextCost:snippetHistoryCost,discardContext:{[weak self] in _ = self?.buffer.snippets.finish()})
        }
        buffer.canUndoSource={ [weak self] in guard let self else{return false};return !self.buffer.readOnly && !self.text.hasMarkedText()&&self.text.undoManager?.canUndo==true }
        buffer.undoSource={ [weak self] in guard let self,!self.buffer.readOnly,!self.text.hasMarkedText(),let manager=self.text.undoManager,manager.canUndo else{return false};manager.undo();return true }
        buffer.canRedoSource={ [weak self] in guard let self else{return false};return !self.buffer.readOnly && !self.text.hasMarkedText()&&self.text.undoManager?.canRedo==true }
        buffer.redoSource={ [weak self] in guard let self,!self.buffer.readOnly,!self.text.hasMarkedText(),let manager=self.text.undoManager,manager.canRedo else{return false};manager.redo();return true }
        buffer.snippets.commit = { [weak self] input,before in
            guard let self,let plan=snippetChangePlan(path:self.buffer.file.path,before:before.source,input:input) else{return false}
            return self.sourceDraftHistory.commit(plan,path:self.buffer.file.path,before:before.source,selectionBefore:self.text.selectedRange(),selectionAfter:NSRange(location:input.selection.from,length:input.selection.to-input.selection.from),contextBefore:SourceSnippetHistoryContext(fields:before.fields,active:before.active,current:before.current,scopes:before.scopes),contextAfter:SourceSnippetHistoryContext(fields:input.session.fields,active:input.session.active,current:input.session.current,scopes:input.session.scopes),manager:self.text.undoManager,
                read:{[weak self] in self?.buffer.text ?? ""},apply:{[weak self] edits,source,selected in self?.applySourceEdits(edits,source:source,resultSelection:selected,manageUndo:false) ?? false},notice:{[weak self] text in self?.buffer.snippets.reportFailure(text)},restoreContext:{[weak self] value in self?.buffer.snippets.restoreHistory(value)},contextCost:snippetHistoryCost,discardContext:{[weak self] in _ = self?.buffer.snippets.finish()})
        }
        buffer.snippets.commitParallel = { [weak self] change,before in
            guard let self,!(self.text.hasMarkedText()),self.canReflectWorkspaceEdits(change.edits,source:before) else{return false}
            let next=change.state
            let step=ChangePlan(title:"Edit template fields",patches:[SourcePatch(path:self.buffer.file.path,before:before,after:next.source,edits:change.edits.map{
                SourceReplacement(from:$0.range.location,to:NSMaxRange($0.range),text:$0.text)
            })])
            return self.sourceDraftHistory.commit(
                step,path:self.buffer.file.path,before:before,selectionBefore:self.text.selectedRange(),
                selectionAfter:next.selections[next.primaryIndex],contextBefore:self.buffer.snippets.historyContext,
                contextAfter:SourceSnippetHistoryContext(fields:[],active:0,current:0,parallel:next),
                manager:self.text.undoManager,read:{[weak self] in self?.buffer.text ?? ""},
                apply:{[weak self] edits,source,selected in self?.applySourceEdits(edits,source:source,resultSelection:selected,manageUndo:false) ?? false},
                notice:{[weak self] value in self?.buffer.snippets.reportFailure(value)},
                restoreContext:{[weak self] context in self?.buffer.snippets.restoreHistory(context)},
                contextCost:snippetHistoryCost,discardContext:{[weak self] in _ = self?.buffer.snippets.finish()}
            )
        }
        buffer.applySourceEdits = { [weak self] edits, source in self?.applySourceEdits(edits, source: source) ?? false }
        buffer.applyElementDraft = { [weak self] edit, source, selection in
            self?.applySourceEdits([edit], source: source, resultSelection: selection) == true
        }
        buffer.canReflectWorkspaceEdits = { [weak self] edits, source in
            guard let self else { return false }
            return self.canReflectWorkspaceEdits(edits, source: source)
        }
        buffer.adoptSourceDraft = { [weak self] before, after, edit in
            guard let self, exactUTF8(self.buffer.text, before) else { return false }
            let selected = self.text.selectedRange()
            let plan = ChangePlan(title: "Adopt saved source", patches: [
                SourcePatch(path: self.buffer.file.path, before: before, after: after, edits: [edit]),
            ])
            return self.sourceDraftHistory.commit(plan,path:self.buffer.file.path,before:before,selectionBefore:selected,selectionAfter:edit.reviewedSelection(selected),contextBefore:self.buffer.snippets.historyContext,contextAfter:nil as SourceSnippetHistoryContext?,manager:self.text.undoManager,
                read:{[weak self] in self?.buffer.text ?? ""},apply:{[weak self] edits,source,selected in self?.applySourceEdits(edits,source:source,resultSelection:selected,manageUndo:false,reviewedResource:true) ?? false},notice:{[weak self] text in self?.buffer.snippets.reportFailure(text)},restoreContext:{[weak self] value in self?.buffer.snippets.restoreHistory(value)},contextCost:snippetHistoryCost,discardContext:{[weak self] in _ = self?.buffer.snippets.finish()})
        }
        buffer.reflectWorkspaceEdits = { [weak self] edits, source, after in self?.applySourceEdits(edits, source: source, expectedModelSource: after) ?? false }
        buffer.applyTableEdit = { [weak self] edit, source in self?.applyTableEdit(edit, source: source) ?? false }
        buffer.selectSourceRange = { [weak self] range, source in self?.selectSourceRange(range, source: source) ?? false }
        paint(buffer.language?.tokens ?? [], source: buffer.text)
        measureDebugLines()
    }
    deinit {
        if let wheelMonitor { NSEvent.removeMonitor(wheelMonitor) }
        if let observer { NotificationCenter.default.removeObserver(observer) }
    }
    func selectSourceRange(_ range: NSRange, source: String) -> Bool {
        guard exactUTF8(text.string,source), exactUTF8(buffer.text,source), !text.hasMarkedText(),
              validSourceEdits([NativeSourceEdit(range: range, text: "")], length: (source as NSString).length) else { return false }
        if !buffer.applyingMulti { buffer.clearMulti() }
        buffer.tableSourceVisible = true; unfoldAll(); text.setSelectedRange(range); buffer.selection = range; text.scrollRangeToVisible(range); return true
    }
    func applyTableEdit(_ edit: NativeSourceEdit, source: String) -> Bool {
        guard exactUTF8(buffer.text,source), !buffer.readOnly else { return false }
        let selection = text.selectedRange(), viewport = scroll.contentView.bounds.origin
        guard applySourceEdits([edit], source: source) else { return false }
        text.setSelectedRange(selectionAfterSourceEdit(selection, edit: edit))
        tableViewport = viewport
        scroll.contentView.scroll(to: viewport); scroll.reflectScrolledClipView(scroll.contentView)
        return true
    }
    func canReflectWorkspaceEdits(_ edits: [NativeSourceEdit], source: String) -> Bool {
        !buffer.readOnly && text.isEditable && exactUTF8(text.string,source) && exactUTF8(buffer.text,source) && !text.hasMarkedText() && validSourceEdits(edits, length: (source as NSString).length)
    }
    func applySourceEdits(_ edits: [NativeSourceEdit], source: String, expectedModelSource: String? = nil,resultSelection:NSRange?=nil,manageUndo:Bool=true,reviewedResource:Bool=false) -> Bool {
        guard !buffer.readOnly, (!buffer.saving || text.undoManager?.isUndoing==true || text.undoManager?.isRedoing==true), text.isEditable, exactUTF8(text.string,source), exactUTF8(buffer.text,expectedModelSource ?? source), !text.hasMarkedText(),
              (edits.isEmpty && resultSelection != nil || validSourceEdits(edits, length: (source as NSString).length)) else { return false }
        let resultLength=source.utf16.count+edits.reduce(0){$0+$1.text.utf16.count-$1.range.length}
        if let selected=resultSelection,!validSourceEdits([NativeSourceEdit(range:selected,text:"")],length:resultLength){return false}
        let wasMulti = buffer.applyingMulti; buffer.applyingMulti = true
        if !wasMulti { buffer.clearMulti() }
        defer { buffer.applyingMulti = wasMulti }
        let viewport=scroll.contentView.bounds.origin
        unfoldAll(); text.breakUndoCoalescing()
        let undo = manageUndo ? text.undoManager:nil; undo?.beginUndoGrouping()
        defer { undo?.endUndoGrouping(); text.breakUndoCoalescing() }
        for edit in edits.sorted(by: { $0.range.location > $1.range.location }) { text.insertText(edit.text, replacementRange: edit.range) }
        if let selected=resultSelection{text.setSelectedRange(selected);scroll.contentView.scroll(to:viewport);scroll.reflectScrolledClipView(scroll.contentView);if !reviewedResource {text.scrollRangeToVisible(selected)}}
        buffer.selection = text.selectedRange(); buffer.text = text.string; measureDebugLines(); return true
    }
    func toggleFold(_ line: Int) {
        guard let header = simpleFoldRanges(text.string, markdown: ["md", "markdown"].contains((buffer.file.path as NSString).pathExtension.lowercased())).first(where: { $0.line == line }) else { return }
        if let index = folding.ranges.firstIndex(of: header.range) { folding.ranges.remove(at: index); buffer.foldedLines.remove(line) }
        else {
            guard !text.hasMarkedText() else { return }
            let selection = text.selectedRange()
            if NSIntersectionRange(selection, header.range).length > 0 || NSLocationInRange(selection.location, header.range) { text.setSelectedRange(NSRange(location: header.range.location, length: 0)) }
            folding.ranges.append(header.range); buffer.foldedLines.insert(line)
        }
        invalidateFolds()
    }
    private func invalidateFolds() {
        let manager = text.layoutManager!
        manager.invalidateGlyphs(forCharacterRange: NSRange(location: 0, length: (text.string as NSString).length), changeInLength: 0, actualCharacterRange: nil)
        measureDebugLines()
    }
    private func unfoldAll() {
        guard !folding.ranges.isEmpty else { return }
        folding.ranges = []; buffer.foldedLines = []; invalidateFolds()
    }
    func textView(_ textView: NSTextView, shouldChangeTextIn affectedCharRange: NSRange, replacementString: String?) -> Bool {
        if buffer.applyingMulti { return true }
        if text.performingMarkedInput {return !buffer.readOnly && text.isEditable}
        if text.undoManager?.isUndoing == true || text.undoManager?.isRedoing == true { buffer.clearMulti(); unfoldAll(); return true }
        if exactUTF8(buffer.multiSource,buffer.text), buffer.multiRanges.count > 1, let replacementString, !text.hasMarkedText() {
            if buffer.snippets.parallel != nil {
                if buffer.snippets.inputParallel(source:text.string,primaryRange:affectedCharRange,text:replacementString){return false}
                _ = buffer.snippets.finish()
            }
            return !buffer.replaceMultiple(replacementString, affected: affectedCharRange)
        }
        if let equation = folding.equations.first(where: { NSIntersectionRange($0.range, affectedCharRange).length > 0 || affectedCharRange.length == 0 && affectedCharRange.location > $0.range.location && NSLocationInRange(affectedCharRange.location, $0.range) }) {
            buffer.sourceTable = buffer.inlineMarkup.equations.first { $0.renderedRange == equation.range }?.id
            measureDebugLines(); return false
        }
        if tableSourceEditHitsGrid(affectedCharRange, selection: text.selectedRange(), tables: text.tableRanges) { return false }
        let hidden = folding.ranges.contains { NSIntersectionRange($0, affectedCharRange).length > 0 || NSLocationInRange(affectedCharRange.location, $0) }
        if !hidden,!text.hasMarkedText(),text.selectedRanges.count==1,let replacementString,buffer.snippets.active != nil,buffer.snippets.input(source:text.string,range:affectedCharRange,text:replacementString) != nil{return false}
        unfoldAll(); return !hidden
    }
    func textDidChange(_ notification: Notification) { if !buffer.applyingMulti { buffer.clearMulti() }; unfoldAll(); buffer.selection = text.selectedRange(); buffer.text = text.string; presentMarkdown(); measureDebugLines() }
    func textViewDidChangeSelection(_ notification: Notification) {
        if let equation = folding.equations.first(where: { text.selectedRange().location > $0.range.location && NSLocationInRange(text.selectedRange().location, $0.range) }) {
            buffer.sourceTable = buffer.inlineMarkup.equations.first { $0.renderedRange == equation.range }?.id
            measureDebugLines()
        }
        if folding.ranges.contains(where: { text.selectedRange().location > $0.location && NSLocationInRange(text.selectedRange().location, $0) }) { unfoldAll() }
        buffer.snippets.selectionChanged(text.selectedRange())
        if !buffer.applyingMulti, let primary = buffer.multiRanges.first, text.selectedRange() != primary { buffer.clearMulti() }
        buffer.selection = text.selectedRange()
        presentMarkdown()
    }
    func presentMarkdown() {
        let snapshot = buffer.inlineMarkup
        let carets = text.selectedRanges.map(\.rangeValue).filter { $0.length == 0 }.map(\.location)
        let glyphs = exactUTF8(snapshot.source, text.string) ? snapshot.markdownGlyphs(carets: carets) : [:]
        if let manager = text.layoutManager { folding.presentMarkdown(glyphs, in: manager) }
    }
    func textView(_ textView: NSTextView, doCommandBy commandSelector: Selector) -> Bool {
        let snippetCommand=NSStringFromSelector(commandSelector)
        if !text.hasMarkedText(),buffer.snippets.isActive {
            if snippetCommand=="insertTab:",buffer.snippets.move(1){return true}
            if snippetCommand=="insertBacktab:",buffer.snippets.move(-1){return true}
            if snippetCommand=="cancelOperation:",buffer.snippets.finish(){return true}
        }
        if ["insertTab:", "insertBacktab:", "insertNewline:", "deleteBackward:"].contains(NSStringFromSelector(commandSelector)) {
            let selection = text.selectedRange()
            if tableSourceEditHitsGrid(selection, selection: selection, tables: text.tableRanges) { return true }
            if folding.ranges.contains(where: { NSIntersectionRange($0, selection).length > 0 || NSLocationInRange(selection.location, $0) }) { unfoldAll(); return true }
        }
        if NSStringFromSelector(commandSelector) == "insertNewline:", !buffer.readOnly, !text.hasMarkedText() {
            if buffer.multiRanges.count > 1 {
                text.insertText(simpleNewlineText(text.string, at: text.selectedRange().location), replacementRange: text.selectedRange()); return true
            }
            let plan = smartEnter(text.string, range: text.selectedRange(), path: buffer.file.path, unit: simpleIndent(text.string), mode: UserDefaults.standard.string(forKey: "editor.enterIndent") ?? "smart")
            guard !tableSourceEditHitsGrid(plan.range, selection: text.selectedRange(), tables: text.tableRanges) else { return true }
            if buffer.snippets.active != nil,buffer.snippets.input(source:text.string,range:plan.range,text:plan.text,caret:plan.caret) != nil{return true}
            if applySourceEdits([NativeSourceEdit(range: plan.range, text: plan.text)], source: text.string) {
                text.setSelectedRange(NSRange(location: plan.caret, length: 0)); text.scrollRangeToVisible(text.selectedRange())
            }
            return true
        }
        if NSStringFromSelector(commandSelector) == "deleteBackward:", !buffer.readOnly, !text.hasMarkedText(), buffer.multiRanges.count < 2,
           text.selectedRange().length == 0, let range = simpleBackspaceRange(text.string, at: text.selectedRange().location) {
            text.insertText("", replacementRange: range); return true
        }
        if NSStringFromSelector(commandSelector) == "insertBacktab:" || NSStringFromSelector(commandSelector) == "insertTab:" && text.selectedRange().length > 0 {
            guard !buffer.readOnly, !text.hasMarkedText() else { return false }
            _ = applySourceEdits(simpleTabEdits(text.string, range: text.selectedRange(), outdent: NSStringFromSelector(commandSelector) == "insertBacktab:"), source: text.string)
            return true
        }
        if NSStringFromSelector(commandSelector) == "insertTab:" {
            guard !buffer.readOnly, !text.hasMarkedText() else { return false }
            text.insertText(simpleTabText(text.string, at: text.selectedRange().location), replacementRange: text.selectedRange()); return true
        }
        guard NSStringFromSelector(commandSelector) == "complete:" else { return false }
        Task { await buffer.language?.complete() }; return true
    }
    func measureDebugLines() {
        presentTables()
        guard let manager = text.layoutManager, let container = text.textContainer else { return }
        let source = text.string as NSString, origin = text.textContainerOrigin
        let visible = text.visibleRect.offsetBy(dx: -origin.x, dy: -origin.y)
        manager.ensureLayout(forBoundingRect: visible, in: container)
        let glyphs = manager.glyphRange(forBoundingRect: visible, in: container)
        let visibleCharacters = manager.characterRange(forGlyphRange: glyphs, actualGlyphRange: nil)
        let starts = exactUTF8(buffer.inlineMarkup.source,source as String) ? buffer.inlineMarkup.lineStarts : nil
        var rows: [DebugLine] = []
        manager.enumerateLineFragments(forGlyphRange: glyphs) { rect, _, _, range, _ in
            let offset = manager.characterIndexForGlyph(at: range.location)
            guard !self.folding.contains(offset), offset == 0 || source.character(at: offset - 1) == 10 else { return }
            let line = starts.map { inlineLineAt(offset, starts: $0) } ?? source.substring(to: offset).filter { $0 == "\n" }.count
            rows.append(DebugLine(line: line, y: rect.minY + origin.y - self.scroll.contentView.bounds.minY, height: rect.height))
        }
        // TextKit exposes a separate fragment for an empty document or trailing newline.
        if source.length == 0 || source.character(at: source.length - 1) == 10 {
            let rect = manager.extraLineFragmentRect
            if rect.height > 0 {
                rows.append(DebugLine(line: starts.map { inlineLineAt(source.length, starts: $0) } ?? source.substring(to: source.length).filter { $0 == "\n" }.count,
                    y: rect.minY + origin.y - scroll.contentView.bounds.minY, height: rect.height))
            }
        }
        var selections: [CGRect] = []
        if exactUTF8(buffer.multiSource,text.string), let window = text.window {
            for range in buffer.multiRanges.dropFirst() where range.location <= NSMaxRange(visibleCharacters) && NSMaxRange(range) >= visibleCharacters.location {
                let screen = text.firstRect(forCharacterRange: range, actualRange: nil)
                let rect = text.convert(window.convertFromScreen(screen), from: nil).offsetBy(dx: -scroll.contentView.bounds.minX, dy: -scroll.contentView.bounds.minY)
                selections.append(CGRect(x: rect.minX, y: rect.minY, width: max(2, rect.width), height: rect.height))
            }
        }
        if selections != buffer.multiRects { buffer.multiRects = selections }
        if rows != buffer.visibleDebugLines { buffer.visibleDebugLines = rows }
    }

    // Until WebKit has measured a formula, keep its actual source readable.
    // Replace presentation glyphs only; the backing storage and undo never change.
    private var centeredMath = NSRange(location: 0, length: 0)
    private func presentMath() {
        let source = text.string, storage = text.textStorage!
        let style = text.typingAttributes[.paragraphStyle] as? NSParagraphStyle ?? .default
        buffer.centerEquationSource(storage, previous: &centeredMath, baseStyle: style)
        let manager = text.layoutManager!, container = text.textContainer!
        let previous = folding.equations
        if !exactUTF8(mathSource,source) {
            for host in mathViews.values { host.removeFromSuperview() }
            mathViews = [:]; mathMeasurements = [:]; mathRequests = [:]; mathSource = source
        }
        let equations = exactUTF8(buffer.inlineMarkup.source,source) ? buffer.inlineMarkup.equations.filter { equation in
            equation.id != buffer.sourceTable && NSMaxRange(equation.renderedRange) <= storage.length &&
                !folding.ranges.contains(where: { NSIntersectionRange($0, equation.renderedRange).length > 0 })
        } : []
        let live = Set(equations.map(\.id))
        for id in Array(mathViews.keys) where !live.contains(id) {
            mathMeasurements.removeValue(forKey: id); mathRequests.removeValue(forKey: id)
            if let host = mathViews.removeValue(forKey: id) { host.removeFromSuperview() }
        }
        for equation in equations where mathViews[equation.id] == nil {
            let request = UUID(); mathRequests[equation.id] = request
            let preview = AnyView(NativeMathView(source: equation.body, inline: !equation.displayBlock, onSize: { [weak self] size in
                guard let self, exactUTF8(self.mathSource,source), exactUTF8(self.buffer.text,source),
                      self.buffer.sourceTable != equation.id, self.mathRequests[equation.id] == request else { return }
                if self.mathMeasurements[equation.id] != size {
                    self.mathMeasurements[equation.id] = size
                    self.measureDebugLines()
                }
            }).overlay {
                Rectangle().fill(.clear).contentShape(Rectangle())
                    .onTapGesture(count: 2, coordinateSpace: .local) { [weak self] point in
                        guard let self, self.mathRequests[equation.id] == request else { return }
                        let width = self.mathViews[equation.id]?.frame.width ?? 1
                        guard self.buffer.toggleEquationSource(equation.id, in: source) else { return }
                        let offset = equationSourceCaret(equation, fraction: point.x / max(1, width))
                        _ = self.text.window?.makeFirstResponder(self.text)
                        self.text.setSelectedRange(NSRange(location: offset, length: 0))
                    }
                    .accessibilityLabel("Equation")
                    .accessibilityAddTraits(.isButton)
                    .accessibilityAction { [weak self] in
                        guard let self, self.mathRequests[equation.id] == request else { return }
                        _ = self.buffer.toggleEquationSource(equation.id, in: source)
                    }
            })
            let host = EquationHost(rootView: preview); host.identifier = NSUserInterfaceItemIdentifier("equation:\(equation.id)")
            host.frame = CGRect(x: -10000, y: -10000, width: 240, height: 80)
            host.alphaValue = 0; mathViews[equation.id] = host; text.addSubview(host)
        }
        let width = max(1, container.size.width - container.lineFragmentPadding * 2 - 1)
        let layouts = equations.compactMap { equation -> EquationLayout? in
            guard let natural = mathMeasurements[equation.id] else { return nil }
            let scale = min(1, width / natural.width, (equation.displayBlock ? 240 : 120) / natural.height)
            return EquationLayout(range: equation.renderedRange, size: CGSize(width: equation.displayBlock ? width : ceil(natural.width * scale), height: ceil(natural.height * scale)))
        }
        if previous != layouts {
            folding.equations = layouts
            for item in previous {
                let range = NSIntersectionRange(item.range, NSRange(location: 0, length: storage.length))
                if range.length > 0 { storage.addAttribute(.foregroundColor, value: NSColor.textColor, range: range) }
            }
            for item in previous + layouts {
                let range = NSIntersectionRange(item.range, NSRange(location: 0, length: storage.length))
                if range.length > 0 { manager.invalidateGlyphs(forCharacterRange: range, changeInLength: 0, actualCharacterRange: nil) }
            }
        }
        for layout in layouts { storage.addAttribute(.foregroundColor, value: NSColor.clear, range: layout.range) }
        let origin = text.textContainerOrigin
        for equation in equations {
            guard let host = mathViews[equation.id], let layout = layouts.first(where: { $0.range == equation.renderedRange }) else { continue }
            manager.ensureLayout(forCharacterRange: layout.range)
            let glyph = manager.glyphIndexForCharacter(at: layout.range.location)
            let line = manager.lineFragmentRect(forGlyphAt: glyph, effectiveRange: nil), position = manager.location(forGlyphAt: glyph)
            host.frame = CGRect(x: origin.x + line.minX + position.x, y: origin.y + line.minY + (line.height - layout.size.height) / 2, width: layout.size.width, height: layout.size.height)
            host.alphaValue = 1
        }
    }

    private func presentEquations() {
        let source = text.string, storage = text.textStorage!
        presentMath()
        var grouped: [Int: [InlineEquation]] = [:]
        let outputs = buffer.recordedOutputPlacements()
            .merging(buffer.terminalOutputRevision == buffer.sourceEditIdentity ? buffer.terminalOutput : [:]) { _, live in live }
            .merging(exactUTF8(buffer.runOutputSource,source) ? buffer.runOutputs : [:]) { _, live in live }
        var starts = [0]
        for (index, unit) in source.utf16.enumerated() where unit == 10 { starts.append(index + 1) }
        for (line, output) in outputs where starts.indices.contains(line) && starts[line] < storage.length {
            grouped[line] = [InlineEquation(line: line, range: NSRange(location: starts[line], length: 0), body: output)]
        }
        let signature = source + "\0" + outputs.keys.sorted().map { "\($0):\(outputs[$0] ?? "")" }.joined()
        if equationSource != signature {
            storage.enumerateAttribute(.paragraphStyle, in: NSRange(location: 0, length: storage.length)) { value, range, _ in
                if (value as? NSParagraphStyle)?.paragraphSpacing == 88 || (value as? NSParagraphStyle)?.paragraphSpacing == 188 { storage.removeAttribute(.paragraphStyle, range: range) }
            }
            for host in equationViews.values { host.removeFromSuperview() }
            equationViews = [:]; equationSource = signature
        }
        for line in Array(equationViews.keys) where grouped[line] == nil {
            if let host = equationViews.removeValue(forKey: line) { host.removeFromSuperview() }
        }
        for (line, values) in grouped {
            guard let first = values.first, first.range.location < storage.length else { continue }
            let paragraph = (source as NSString).paragraphRange(for: NSRange(location: first.range.location, length: 0))
            let height: CGFloat = outputs[line] == nil ? 80 : 180
            let style = NSMutableParagraphStyle(); style.paragraphSpacing = height + 8
            if (storage.attribute(.paragraphStyle, at: first.range.location, effectiveRange: nil) as? NSParagraphStyle)?.paragraphSpacing != height + 8 { storage.addAttribute(.paragraphStyle, value: style, range: paragraph) }
            let preview: AnyView
            if let output = outputs[line] {
                preview = AnyView(NativeInlineTerminal(text: output, sourceLinks: buffer.runOutputs[line] == output ? buffer.runOutputLinks : [:], onSource: { [weak buffer = self.buffer] offset in
                    guard let buffer, exactUTF8(buffer.text, source), exactUTF8(buffer.runOutputSource, source),
                          buffer.runOutputs[line] == output, starts.indices.contains(offset) else { return }
                    if buffer.selectSourceRange?(NSRange(location: starts[offset], length: 0), source) == true { buffer.focusSource?() }
                }))
            } else { preview = AnyView(NativeMathView(source: values.map(\.body).joined(separator: " \\qquad "))) }
            let host: EquationHost
            if let existing = equationViews[line] { existing.rootView = preview; host = existing }
            else { host = EquationHost(rootView: preview); equationViews[line] = host; text.addSubview(host) }
            let manager = text.layoutManager!, container = text.textContainer!
            let glyph = manager.glyphRange(forCharacterRange: NSRange(location: first.range.location, length: 1), actualCharacterRange: nil)
            let rect = manager.boundingRect(forGlyphRange: glyph, in: container), origin = text.textContainerOrigin
            host.frame = CGRect(x: origin.x, y: rect.maxY + origin.y + 2, width: max(100, text.bounds.width - 16), height: height)
        }
    }

    private func presentDiagrams() {
        presentEquations()
        let source = text.string
        let storage = text.textStorage!
        let diagrams = !buffer.tableSourceVisible ? tableSnapshot.diagrams.filter { $0.id != buffer.sourceTable } : []
        let changed = diagrams.map(\.range) != folding.diagrams.map(\.range)
        if !exactUTF8(diagramSource,source) || diagramAnchor != buffer.sourceTable || changed {
            diagramSource = source; diagramAnchor = buffer.sourceTable
            storage.beginEditing()
            let tabs = NSMutableParagraphStyle()
            for old in folding.diagrams {
                let range = NSIntersectionRange(old.range, NSRange(location: 0, length: storage.length))
                if range.length > 0 { storage.addAttribute(.paragraphStyle, value: tabs, range: range); storage.addAttribute(.foregroundColor, value: NSColor.textColor, range: range); text.layoutManager!.invalidateGlyphs(forCharacterRange: range, changeInLength: 0, actualCharacterRange: nil) }
            }
            folding.diagrams = diagrams
            let live = Set(diagrams.map(\.id))
            for id in Array(diagramViews.keys) where !live.contains(id) { diagramViews[id]?.removeFromSuperview(); diagramViews.removeValue(forKey: id) }
            for diagram in diagrams {
                let preview = NativeDiagramPreview(diagram: diagram, buffer: buffer)
                if let host = diagramViews[diagram.id] { host.rootView = preview }
                else { let host = DiagramHost(rootView: preview); diagramViews[diagram.id] = host; text.addSubview(host) }
            }
            storage.endEditing()
        }
        text.tableRanges = folding.tables.map(\.range) + diagrams.map(\.range)
        for diagram in diagrams {
            let natural = buffer.diagramSizes[diagram.id] ?? CGSize(width: 200, height: 80)
            let width = max(100, text.bounds.width - 16)
            let height: CGFloat = diagram.renderer == "terminal" ? 180 : max(24, ceil(natural.height * min(1, width / max(1, natural.width))))
            let style = NSMutableParagraphStyle(); style.minimumLineHeight = height; style.maximumLineHeight = height
            if (storage.attribute(.paragraphStyle, at: diagram.range.location, effectiveRange: nil) as? NSParagraphStyle)?.minimumLineHeight != height {
                storage.addAttribute(.paragraphStyle, value: style, range: NSRange(location: diagram.range.location, length: 1))
            }
            if storage.attribute(.foregroundColor, at: diagram.range.location, effectiveRange: nil) as? NSColor != .clear {
                storage.addAttribute(.foregroundColor, value: NSColor.clear, range: diagram.range)
            }
            if changed { text.layoutManager!.invalidateGlyphs(forCharacterRange: diagram.range, changeInLength: 0, actualCharacterRange: nil) }
            let manager = text.layoutManager!, container = text.textContainer!
            let glyph = manager.glyphRange(forCharacterRange: NSRange(location: diagram.range.location, length: 1), actualCharacterRange: nil)
            let rect = manager.boundingRect(forGlyphRange: glyph, in: container)
            diagramViews[diagram.id]?.frame = NSRect(x: text.textContainerOrigin.x, y: rect.minY + text.textContainerOrigin.y, width: max(100, text.bounds.width - 16), height: height)
        }
    }
    private func preserveViewport() -> () -> Void {
        guard let manager = text.layoutManager, let container = text.textContainer, manager.numberOfGlyphs > 0 else { return {} }
        let viewport = scroll.contentView.bounds.origin, origin = text.textContainerOrigin
        let glyph = manager.glyphIndex(for: NSPoint(x: viewport.x, y: viewport.y - origin.y), in: container)
        let character = manager.characterIndexForGlyph(at: min(glyph, manager.numberOfGlyphs - 1))
        let before = manager.lineFragmentRect(forGlyphAt: min(glyph, manager.numberOfGlyphs - 1), effectiveRange: nil).minY
        return { [self] in
            guard character < (text.string as NSString).length else { return }
            manager.ensureLayout(forCharacterRange: NSRange(location: character, length: 1))
            let after = manager.lineFragmentRect(forGlyphAt: manager.glyphIndexForCharacter(at: character), effectiveRange: nil).minY
            scroll.contentView.scroll(to: NSPoint(x: viewport.x, y: max(0, viewport.y + after - before)))
            scroll.reflectScrolledClipView(scroll.contentView)
        }
    }
    private func presentTables() {
        let restore = preserveViewport(); defer { restore() }
        let source = text.string
        let storage = text.textStorage!
        let parseCurrent = exactUTF8(buffer.inlineMarkup.source,source)
        if parseCurrent { tableSnapshot = buffer.inlineMarkup }
        else { tableSnapshot = tablePresentation(tableSnapshot, source: source) }
        let tables: [InlineTable] = !buffer.tableSourceVisible ? inlineTables(tableSnapshot).filter { $0.id != buffer.sourceTable }.map { value in var value = value; value.preferredHeight = buffer.tableHeights[value.id]; return value } : []
        let geometryChanged = folding.tables.map { [$0.range.location, $0.range.length, Int($0.height)] } != tables.map { [$0.range.location, $0.range.length, Int($0.height)] }
        if !exactUTF8(tableSource,source) || tableSourceVisible != buffer.tableSourceVisible || sourceTable != buffer.sourceTable || geometryChanged || tableParseCurrent != parseCurrent {
            tableParseCurrent = parseCurrent
            tableSource = source; tableSourceVisible = buffer.tableSourceVisible; sourceTable = buffer.sourceTable
            storage.beginEditing()
            for table in folding.tables {
                let range = NSIntersectionRange(table.range, NSRange(location: 0, length: storage.length))
                if range.length > 0 { storage.addAttribute(.foregroundColor, value: NSColor.textColor, range: range) }
            }
            let tabs = NSMutableParagraphStyle(); tabs.tabStops = []
            tabs.defaultTabInterval = ("    " as NSString).size(withAttributes: [.font: text.font ?? .monospacedSystemFont(ofSize: 13, weight: .regular)]).width
            if geometryChanged {
                storage.enumerateAttribute(.paragraphStyle, in: NSRange(location: 0, length: storage.length)) { value, range, _ in
                    if let style = value as? NSParagraphStyle, style.minimumLineHeight > 0 { storage.addAttribute(.paragraphStyle, value: tabs, range: range) }
                }
            }
            text.typingAttributes[.paragraphStyle] = tabs
            folding.tables = tables
            text.tableRanges = folding.tables.map(\.range)
            for table in folding.tables where table.range.length > 0 {
                let style = NSMutableParagraphStyle(); style.minimumLineHeight = table.height; style.maximumLineHeight = table.height
                storage.addAttribute(.paragraphStyle, value: style, range: NSRange(location: table.range.location, length: 1))
                storage.addAttribute(.foregroundColor, value: NSColor.clear, range: table.range)
            }
            storage.endEditing()
            if geometryChanged {
                for table in tables { text.layoutManager!.invalidateGlyphs(forCharacterRange: table.range, changeInLength: 0, actualCharacterRange: nil) }
            }
            let live = Set(folding.tables.map(\.id))
            for id in Array(tableViews.keys) where !live.contains(id) { tableViews[id]?.removeFromSuperview(); tableViews.removeValue(forKey: id) }
            for table in folding.tables {
                let grid = NativeTableGrid(table: table, source: source, buffer: buffer)
                if let host = tableViews[table.id] { host.rootView = grid } else { let host = NSHostingView(rootView: grid); tableViews[table.id] = host; text.addSubview(host) }
            }
        }
        let manager = text.layoutManager!, container = text.textContainer!
        for table in folding.tables {
            if table.range.length > 0 && storage.attribute(.foregroundColor, at: table.range.location, effectiveRange: nil) as? NSColor != .clear { storage.addAttribute(.foregroundColor, value: NSColor.clear, range: table.range) }
            let glyph = manager.glyphRange(forCharacterRange: NSRange(location: table.range.location, length: min(1, table.range.length)), actualCharacterRange: nil)
            let rect = table.range.length == 0 ? CGRect.zero : manager.boundingRect(forGlyphRange: glyph, in: container)
            tableViews[table.id]?.frame = NSRect(x: text.textContainerOrigin.x, y: rect.minY + text.textContainerOrigin.y, width: max(100, text.bounds.width - 16), height: table.height)
        }
        presentDiagrams()
        if let viewport = tableViewport, exactUTF8(buffer.inlineMarkup.source,source) {
            tableViewport = nil
            scroll.contentView.scroll(to: viewport); scroll.reflectScrolledClipView(scroll.contentView)
        }
    }
    private func paint(_ tokens: [NativeSemanticToken], source: String) {
        presentMarkdown()
        guard exactUTF8(text.string,source), !text.hasMarkedText(), let storage = text.textStorage else { return }
        let restore = preserveViewport(); defer { restore() }
        storage.beginEditing()
        storage.addAttribute(.foregroundColor, value: NSColor.textColor, range: NSRange(location: 0, length: storage.length))
        // Foreground colors reset on every paint, including provider-only refreshes.
        if exactUTF8(buffer.inlineMarkup.source,source), source.utf16.count <= 200_000 {
            for mark in buffer.inlineMarkup.marks where mark.kind.hasPrefix("syntax-") && NSMaxRange(mark.range) <= storage.length {
                let color: NSColor
                switch mark.kind {
                case "syntax-keyword": color = .systemPurple
                case "syntax-string": color = .systemGreen
                case "syntax-number": color = .systemOrange
                default: color = .secondaryLabelColor
                }
                storage.addAttribute(.foregroundColor, value: color, range: mark.range)
            }
        }
        for token in tokens where NSMaxRange(token.range) <= storage.length {
            let color: NSColor
            switch token.kind {
            case "keyword", "type", "class", "struct": color = .systemPurple
            case "function", "method": color = .systemBlue
            case "string": color = .systemGreen
            case "number": color = .systemOrange
            case "comment": color = .secondaryLabelColor
            default: color = .systemTeal
            }
            storage.addAttribute(.foregroundColor, value: color, range: token.range)
        }
        if exactUTF8(buffer.inlineMarkup.source,source) && !exactUTF8(paintedInlineSource,source) && (!buffer.inlineMarkup.marks.isEmpty || hadInlineFonts) && (source.utf16.count <= 200_000 || hadInlineFonts) {
            storage.addAttribute(.font, value: NSFont.monospacedSystemFont(ofSize: 13, weight: .regular), range: NSRange(location: 0, length: storage.length))
            storage.removeAttribute(.backgroundColor, range: NSRange(location: 0, length: storage.length))
            if source.utf16.count <= 200_000 {
                for mark in buffer.inlineMarkup.marks where NSMaxRange(mark.range) <= storage.length {
                    let font: NSFont
                    switch mark.kind {
                    case "heading": font = .systemFont(ofSize: CGFloat(max(14, 25 - mark.level * 2)), weight: .semibold)
                    case "strong": font = .boldSystemFont(ofSize: 13)
                    case "em", "math": font = NSFontManager.shared.convert(.systemFont(ofSize: 13), toHaveTrait: .italicFontMask)
                    default: font = .monospacedSystemFont(ofSize: 13, weight: .regular)
                    }
                    storage.addAttribute(.font, value: font, range: mark.range)
                    if mark.kind == "link" { storage.addAttribute(.foregroundColor, value: NSColor.systemBlue, range: mark.range) }
                    if mark.kind == "code" { storage.addAttribute(.backgroundColor, value: NSColor.quaternaryLabelColor, range: mark.range) }
                    if mark.kind == "diff-add" || mark.kind == "diff-remove" { storage.addAttribute(.backgroundColor, value: (mark.kind == "diff-add" ? NSColor.systemGreen : NSColor.systemRed).withAlphaComponent(0.15), range: mark.range) }
                }
            }
            paintedInlineSource = source; hadInlineFonts = !buffer.inlineMarkup.marks.isEmpty
        }
        storage.endEditing(); text.typingAttributes[.foregroundColor] = NSColor.textColor
        presentTables()
    }
}
@MainActor private enum EditorCache { static var views: [UUID: MacEditor] = [:] }
// SwiftUI owns a fresh host at each tree location. The cached editor moves
// between hosts, so teardown of an old representable cannot detach the new one.
private final class MacEditorHost: NSView {
    let content: NSView
    let measure: () -> Void
    init(_ content: NSView, measure: @escaping () -> Void) { self.content = content; self.measure = measure; super.init(frame: .zero) }
    required init?(coder: NSCoder) { fatalError("init(coder:) has not been implemented") }
    func attach() {
        guard window != nil else { return }
        window?.acceptsMouseMovedEvents = true
        if content.superview !== self { content.removeFromSuperview(); addSubview(content) }
        content.frame = bounds
    }
    override func viewDidMoveToWindow() { super.viewDidMoveToWindow(); attach() }
    override func layout() { super.layout(); attach(); Task { @MainActor in measure() } }
}
struct NativeEditor: NSViewRepresentable {
    let buffer: DocumentBuffer
    var readOnly: Bool? = nil
    var submitMessage: (() -> Void)? = nil
    func sizeThatFits(_ proposal: ProposedViewSize, nsView: NSView, context: Context) -> CGSize? {
        CGSize(width: proposal.width ?? 600, height: proposal.height ?? 400)
    }
    func makeNSView(context: Context) -> NSView {
        let editor = EditorCache.views[buffer.id] ?? MacEditor(buffer)
        EditorCache.views[buffer.id] = editor
        editor.text.submitMessage = submitMessage
        return MacEditorHost(editor.scroll) { [weak editor] in editor?.measureDebugLines() }
    }
    func updateNSView(_ view: NSView, context: Context) {
        if let editor = EditorCache.views[buffer.id] {
            editor.text.submitMessage = submitMessage
            if !exactUTF8(editor.text.string,buffer.text) {
                let viewport = editor.scroll.contentView.bounds.origin
                let length = buffer.text.utf16.count
                let start = min(buffer.selection.location, length)
                let selection = NSRange(location: start, length: min(buffer.selection.length, length - start))
                editor.text.string = buffer.text
                editor.text.setSelectedRange(selection)
                editor.presentMarkdown()
                editor.scroll.contentView.scroll(to: viewport)
                editor.scroll.reflectScrolledClipView(editor.scroll.contentView)
                editor.text.undoManager?.removeAllActions()
            }
            let editable = !(readOnly ?? buffer.readOnly)
            if editor.text.isEditable != editable {
                let selection = editor.text.selectedRanges, viewport = editor.scroll.contentView.bounds.origin
                editor.text.isEditable = editable
                if editor.text.selectedRanges != selection { editor.text.selectedRanges = selection }
                if editor.scroll.contentView.bounds.origin != viewport {
                    editor.scroll.contentView.scroll(to: viewport)
                    editor.scroll.reflectScrolledClipView(editor.scroll.contentView)
                }
            }
        }
        (view as? MacEditorHost)?.attach(); view.needsLayout = true
        if let pending=buffer.recoveryViewport,let editor=EditorCache.views[buffer.id]{let revision=buffer.sourceEditIdentity,selection=editor.text.selectedRange();Task { @MainActor in
            guard buffer.recoveryViewport != nil,buffer.sourceEditIdentity==revision,editor.text.selectedRange()==selection,editor.text.window != nil else{return}
            buffer.recoveryViewport=nil;let range=NSRange(location:min(pending.selection.anchor,pending.selection.head),length:abs(pending.selection.head-pending.selection.anchor));editor.text.setSelectedRange(range);buffer.selection=range;editor.scroll.layoutSubtreeIfNeeded();editor.scroll.contentView.scroll(to:CGPoint(x:pending.scroll_x,y:pending.scroll_y));editor.scroll.reflectScrolledClipView(editor.scroll.contentView)
        }}
        Task { @MainActor in EditorCache.views[buffer.id]?.measureDebugLines() }
    }
}
#else
import UIKit
@MainActor private final class MobileEditor: NSObject, UITextViewDelegate, UITextPasteDelegate, UIGestureRecognizerDelegate {
    private let sourceEditorIdentity=UUID()
    let text:SimpleTextView
    private let sourceTextStorage:NSTextStorage
    private let sourceLayoutManager:NSLayoutManager
    private let folding = FoldLayout()
    private let sourceDraftHistory=SourceDraftHistory()
    private var markedSnippet:CapturedMarkedSnippet?
    private var tableViewport: CGPoint?
    private var diagramSource = ""
    private var diagramAnchor: Int?
    private var equationViews: [Int: EquationHost] = [:]
    private var equationSource = ""
    private var mathSource = ""
    private var mathViews: [Int: EquationHost] = [:]
    private var mathMeasurements: [Int: CGSize] = [:]
    private var mathRequests: [Int: UUID] = [:]
    private var diagramViews: [Int: DiagramHost] = [:]
    private var tableSnapshot = InlineSnapshot(source: "")
    private var tableSource = ""
    private var tableParseCurrent = false
    private var tableSourceVisible = false
    private var sourceTable: Int?
    #if os(macOS)
    private var tableViews: [Int: NSHostingView<NativeTableGrid>] = [:]
    #else
    private var tableViews: [Int: UIHostingController<NativeTableGrid>] = [:]
    #endif
    private var paintedInlineSource = ""
    private var hadInlineFonts = false
    let buffer: DocumentBuffer
    init(_ buffer: DocumentBuffer) {
        self.buffer = buffer
        // The imported TextKit convenience initializer is a UIKit class factory.
        // The designated initializer runs Swift-owned input/history initialization.
        let storage=NSTextStorage()
        let manager=NSLayoutManager()
        let container=NSTextContainer(size:CGSize(width:0,height:CGFloat.greatestFiniteMagnitude))
        container.widthTracksTextView=true
        container.heightTracksTextView=false
        storage.addLayoutManager(manager)
        manager.addTextContainer(container)
        self.sourceTextStorage=storage
        self.sourceLayoutManager=manager
        self.text=SimpleTextView(frame:.zero,textContainer:container)
        super.init()
        text.font = .monospacedSystemFont(ofSize: 13, weight: .regular)
        let tabs = NSMutableParagraphStyle(); tabs.tabStops = []
        tabs.defaultTabInterval = ("    " as NSString).size(withAttributes: [.font: text.font ?? .monospacedSystemFont(ofSize: 13, weight: .regular)]).width
        text.typingAttributes[.paragraphStyle] = tabs
        text.autocorrectionType = .no; text.autocapitalizationType = .none
        text.smartQuotesType = .no; text.smartDashesType = .no; text.smartInsertDeleteType = .no
        text.text = buffer.text; text.isEditable = !buffer.readOnly; text.alwaysBounceVertical = true
        text.selectedRange = buffer.selection
        text.accessibilityLabel = buffer.file.path; text.delegate = self; text.pasteDelegate = self
        let equationTap = UITapGestureRecognizer(target: self, action: #selector(restoreEquationPreview(_:)))
        equationTap.numberOfTapsRequired = 2
        equationTap.delegate = self
        for existing in text.gestureRecognizers ?? [] where (existing as? UITapGestureRecognizer)?.numberOfTapsRequired == 2 {
            existing.require(toFail: equationTap)
        }
        text.addGestureRecognizer(equationTap)
        text.beginSnippetComposition = { [weak self] range in
            guard let self else{return false}
            self.unfoldAll()
            guard self.text.undoManager != nil,self.text.isEditable,self.text.window != nil,
                  let capture=CapturedMarkedSnippet(buffer:self.buffer,source:self.text.text ?? "",selection:self.text.selectedRange,range:range) else{
                self.buffer.clearMulti()
                _ = self.buffer.snippets.finish()
                return false
            }
            self.markedSnippet=capture
            return true
        }
        text.reportSnippetComposition = { [weak self] finished in
            guard let self else{return}
            self.buffer.selection = self.text.selectedRange; self.buffer.text = self.text.text
            guard finished,let lease=self.markedSnippet else{return}
            self.markedSnippet=nil
            let observed=self.text.text ?? ""
            let selected=self.text.selectedRange
            completeMarkedSnippet(
                lease, buffer:self.buffer, observed:observed, selected:selected,
                history:self.sourceDraftHistory, manager:self.text.undoManager,
                admitted:self.text.isEditable && self.text.window != nil,
                apply:{[weak self] edits,source,selection in self?.applySourceEdits(edits,source:source,resultSelection:selection,manageUndo:false) ?? false}
            )
        }
        text.linkAtCaret = { [weak self] in
            guard let self else { return nil }
            return self.buffer.inlineLink(at: self.text.selectedRange, in: self.text.text)
        }
        text.openLink = { [weak self] link in
            guard let self, self.text.linkAtCaret?() == link else { return false }
            return self.buffer.followInlineLink(link)
        }
        text.snippetActive={ [weak self] in self?.buffer.snippets.isActive == true }
        text.snippetCommand={ [weak self] direction in guard let self else{return false};return direction==0 ? self.buffer.snippets.finish():self.buffer.snippets.move(direction) }

        buffer.navigate = { [weak self] fraction in
            guard let self else { return }
            self.unfoldAll()
            let source = self.text.text as NSString
            let offset = min(source.length, max(0, Int(Double(source.length) * fraction)))
            self.text.scrollRangeToVisible(source.lineRange(for: NSRange(location: offset, length: 0)))
        }
        buffer.renderLanguage = { [weak self] tokens, source in self?.paint(tokens, source: source) }
        text.copiedQuote = { [weak self] selection in
            guard let self, exactUTF8(self.buffer.inlineMarkup.source, self.text.text ?? "") else { return nil }
            return self.buffer.inlineMarkup.quoteContents(selection)
        }
        buffer.renderMarkup = { [weak self] in guard let self else { return }; self.paint(self.buffer.language?.tokens ?? [], source: self.buffer.text); self.measureDebugLines() }
        buffer.applyLanguageCompletion = { [weak self] range, insert, source, selection in
            guard let self, self.text.selectedRange == selection else { return false }
            return self.applySourceEdits([NativeSourceEdit(range: range, text: insert)], source: source)
        }
        buffer.revealDebugLine = { [weak self] line in
            guard let self, let offset = languageOffset(self.buffer.text, ["line": line, "character": 0]) else { return false }
            self.unfoldAll(); self.text.scrollRangeToVisible(NSRange(location: offset, length: 0));return true
        }
        text.layoutManager.delegate = folding
        buffer.toggleFold = { [weak self] line in self?.toggleFold(line) }
        buffer.focusSource = { [weak self] in self?.text.becomeFirstResponder() }
        buffer.releaseSourceFocus = { [weak self] in
            guard let text = self?.text, text.isFirstResponder else { return }
            _ = text.resignFirstResponder()
        }
        buffer.captureRecoveryViewport={ [weak self] in self?.text.contentOffset ?? .zero }
        buffer.sourceEditorState={ [weak self] in guard let self else{return nil};return NativeSourceEditorState(identity:self.sourceEditorIdentity,source:self.text.text ?? "",selection:self.text.selectedRange,composing:self.text.markedTextRange != nil,editable:self.text.isEditable && !self.buffer.readOnly && !self.buffer.saving,attached:self.text.window != nil,selections:max(1,self.buffer.multiRanges.count))}
        buffer.applyCompletionDraft = { [weak self] draft,initial in
            guard let self,self.text.selectedRange==initial,exactUTF8(draft.path,self.buffer.file.path) else{return false}
            let selection=NSRange(location:draft.selection.from,length:draft.selection.to-draft.selection.from)
            let after:SourceSnippetHistoryContext?
            if let batch=draft.caretBatch {
                guard exactUTF8(self.buffer.multiSource,draft.before),self.buffer.multiRanges==batch.before,
                      batch.before.first==initial,batch.after.first==selection else{return false}
                after=SourceSnippetHistoryContext(fields:[],active:0,current:0,parallel:batch.parallel,ranges:batch.after)
            }else{
                after=(draft.continuing ?? beginSnippet(draft.plan.patches.first?.after ?? draft.before,fields:draft.fields))
                    .map{SourceSnippetHistoryContext(fields:$0.fields,active:$0.active,current:$0.current,scopes:$0.scopes)}
            }
            return self.sourceDraftHistory.commit(draft.plan,path:draft.path,before:draft.before,selectionBefore:initial,selectionAfter:selection,contextBefore:self.buffer.snippets.historyContext,contextAfter:after,manager:self.text.undoManager,
                read:{[weak self] in self?.buffer.text ?? ""},apply:{[weak self] edits,source,selected in self?.applySourceEdits(edits,source:source,resultSelection:selected,manageUndo:false) ?? false},notice:{[weak self] text in self?.buffer.snippets.reportFailure(text)},restoreContext:{[weak self] value in self?.buffer.snippets.restoreHistory(value)},contextCost:snippetHistoryCost,discardContext:{[weak self] in _ = self?.buffer.snippets.finish()})
        }
        buffer.canUndoSource={ [weak self] in guard let self else{return false};return !self.buffer.readOnly&&self.text.markedTextRange==nil&&self.text.undoManager?.canUndo==true }
        buffer.undoSource={ [weak self] in guard let self,!self.buffer.readOnly,self.text.markedTextRange==nil,let manager=self.text.undoManager,manager.canUndo else{return false};manager.undo();return true }
        buffer.canRedoSource={ [weak self] in guard let self else{return false};return !self.buffer.readOnly&&self.text.markedTextRange==nil&&self.text.undoManager?.canRedo==true }
        buffer.redoSource={ [weak self] in guard let self,!self.buffer.readOnly,self.text.markedTextRange==nil,let manager=self.text.undoManager,manager.canRedo else{return false};manager.redo();return true }
        buffer.snippets.commit = { [weak self] input,before in
            guard let self,let plan=snippetChangePlan(path:self.buffer.file.path,before:before.source,input:input) else{return false}
            return self.sourceDraftHistory.commit(plan,path:self.buffer.file.path,before:before.source,selectionBefore:self.text.selectedRange,selectionAfter:NSRange(location:input.selection.from,length:input.selection.to-input.selection.from),contextBefore:SourceSnippetHistoryContext(fields:before.fields,active:before.active,current:before.current,scopes:before.scopes),contextAfter:SourceSnippetHistoryContext(fields:input.session.fields,active:input.session.active,current:input.session.current,scopes:input.session.scopes),manager:self.text.undoManager,
                read:{[weak self] in self?.buffer.text ?? ""},apply:{[weak self] edits,source,selected in self?.applySourceEdits(edits,source:source,resultSelection:selected,manageUndo:false) ?? false},notice:{[weak self] text in self?.buffer.snippets.reportFailure(text)},restoreContext:{[weak self] value in self?.buffer.snippets.restoreHistory(value)},contextCost:snippetHistoryCost,discardContext:{[weak self] in _ = self?.buffer.snippets.finish()})
        }
        buffer.snippets.commitParallel = { [weak self] change,before in
            guard let self,!(self.text.markedTextRange != nil),self.canReflectWorkspaceEdits(change.edits,source:before) else{return false}
            let next=change.state
            let step=ChangePlan(title:"Edit template fields",patches:[SourcePatch(path:self.buffer.file.path,before:before,after:next.source,edits:change.edits.map{
                SourceReplacement(from:$0.range.location,to:NSMaxRange($0.range),text:$0.text)
            })])
            return self.sourceDraftHistory.commit(
                step,path:self.buffer.file.path,before:before,selectionBefore:self.text.selectedRange,
                selectionAfter:next.selections[next.primaryIndex],contextBefore:self.buffer.snippets.historyContext,
                contextAfter:SourceSnippetHistoryContext(fields:[],active:0,current:0,parallel:next),
                manager:self.text.undoManager,read:{[weak self] in self?.buffer.text ?? ""},
                apply:{[weak self] edits,source,selected in self?.applySourceEdits(edits,source:source,resultSelection:selected,manageUndo:false) ?? false},
                notice:{[weak self] value in self?.buffer.snippets.reportFailure(value)},
                restoreContext:{[weak self] context in self?.buffer.snippets.restoreHistory(context)},
                contextCost:snippetHistoryCost,discardContext:{[weak self] in _ = self?.buffer.snippets.finish()}
            )
        }
        buffer.applySourceEdits = { [weak self] edits, source in self?.applySourceEdits(edits, source: source) ?? false }
        buffer.applyElementDraft = { [weak self] edit, source, selection in
            self?.applySourceEdits([edit], source: source, resultSelection: selection) == true
        }
        buffer.canReflectWorkspaceEdits = { [weak self] edits, source in
            guard let self else { return false }
            return self.canReflectWorkspaceEdits(edits, source: source)
        }
        buffer.adoptSourceDraft = { [weak self] before, after, edit in
            guard let self, exactUTF8(self.buffer.text, before) else { return false }
            let selected = self.text.selectedRange
            let plan = ChangePlan(title: "Adopt saved source", patches: [
                SourcePatch(path: self.buffer.file.path, before: before, after: after, edits: [edit]),
            ])
            return self.sourceDraftHistory.commit(plan,path:self.buffer.file.path,before:before,selectionBefore:selected,selectionAfter:edit.reviewedSelection(selected),contextBefore:self.buffer.snippets.historyContext,contextAfter:nil as SourceSnippetHistoryContext?,manager:self.text.undoManager,
                read:{[weak self] in self?.buffer.text ?? ""},apply:{[weak self] edits,source,selected in self?.applySourceEdits(edits,source:source,resultSelection:selected,manageUndo:false,reviewedResource:true) ?? false},notice:{[weak self] text in self?.buffer.snippets.reportFailure(text)},restoreContext:{[weak self] value in self?.buffer.snippets.restoreHistory(value)},contextCost:snippetHistoryCost,discardContext:{[weak self] in _ = self?.buffer.snippets.finish()})
        }
        buffer.reflectWorkspaceEdits = { [weak self] edits, source, after in self?.applySourceEdits(edits, source: source, expectedModelSource: after) ?? false }
        buffer.applyTableEdit = { [weak self] edit, source in self?.applyTableEdit(edit, source: source) ?? false }
        buffer.selectSourceRange = { [weak self] range, source in self?.selectSourceRange(range, source: source) ?? false }
        paint(buffer.language?.tokens ?? [], source: buffer.text)
        measureDebugLines()
    }
    func selectSourceRange(_ range: NSRange, source: String) -> Bool {
        guard exactUTF8(text.text,source), exactUTF8(buffer.text,source), text.markedTextRange == nil,
              validSourceEdits([NativeSourceEdit(range: range, text: "")], length: (source as NSString).length) else { return false }
        if !buffer.applyingMulti { buffer.clearMulti() }
        buffer.tableSourceVisible = true; unfoldAll(); text.selectedRange = range; buffer.selection = range; text.scrollRangeToVisible(range); return true
    }
    func applyTableEdit(_ edit: NativeSourceEdit, source: String) -> Bool {
        guard exactUTF8(buffer.text,source), !buffer.readOnly else { return false }
        let selection = text.selectedRange, viewport = text.contentOffset
        guard applySourceEdits([edit], source: source) else { return false }
        text.selectedRange = selectionAfterSourceEdit(selection, edit: edit)
        tableViewport = viewport; text.setContentOffset(viewport, animated: false)
        return true
    }
    func canReflectWorkspaceEdits(_ edits: [NativeSourceEdit], source: String) -> Bool {
        !buffer.readOnly && text.isEditable && exactUTF8(text.text,source) && exactUTF8(buffer.text,source) && text.markedTextRange == nil && validSourceEdits(edits, length: (source as NSString).length)
    }
    func textPasteConfigurationSupporting(
        _ supporting: any UITextPasteConfigurationSupporting,
        performPasteOf value: NSAttributedString,
        to range: UITextRange
    ) -> UITextRange {
        guard supporting === text, text.window != nil, text.isEditable, !buffer.readOnly,
              !buffer.resourceDeleted, text.markedTextRange == nil,
              !value.string.isEmpty, exactUTF8(text.text, buffer.text) else { return range }
        let start = text.offset(from: text.beginningOfDocument, to: range.start)
        let end = text.offset(from: text.beginningOfDocument, to: range.end)
        guard start >= 0, end >= start else { return range }
        let target = NSRange(location: start, length: end - start)
        let selected = text.selectedRange
        let source = buffer.text
        let insertion = value.string
        let context = buffer.snippets.historyContext
        let multiple = target == selected && buffer.multiRanges.first == target && exactUTF8(buffer.multiSource, source)
        let ranges = multiple ? buffer.multiRanges.sorted { $0.location < $1.location } : [target]
        guard let primary = ranges.firstIndex(of: target) else { return range }
        let edits = ranges.map { NativeSourceEdit(range: $0, text: insertion) }
        guard canReflectWorkspaceEdits(edits, source: source),
              !ranges.contains(where: { tableSourceEditHitsGrid($0, selection: selected, tables: text.tableRanges) }) else { return range }
        if ranges.contains(where: { edit in
            folding.ranges.contains { NSIntersectionRange($0, edit).length > 0 || NSLocationInRange(edit.location, $0) }
        }) { unfoldAll(); return range }

        if ranges.count > 1, buffer.snippets.inputParallel(source: source, primaryRange: target, text: insertion) {
            return text.selectedTextRange ?? range
        }
        if ranges.count == 1, buffer.snippets.active != nil,
           buffer.snippets.input(source: source, range: target, text: insertion) != nil {
            return text.selectedTextRange ?? range
        }
        guard Set(ranges).count == ranges.count else { return range }
        let result = NSMutableString(string: source)
        for edit in edits.reversed() { result.replaceCharacters(in: edit.range, with: edit.text) }
        let count = insertion.utf16.count
        var delta = 0
        let carets = ranges.map { edit in
            let caret = NSRange(location: edit.location + delta + count, length: 0)
            delta += count - edit.length
            return caret
        }
        let plan = ChangePlan(title: "Paste", patches: [SourcePatch(
            path: buffer.file.path, before: source, after: result as String,
            edits: ranges.map { SourceReplacement(from: $0.location, to: NSMaxRange($0), text: insertion) }
        )])
        let selection = carets[primary]
        let logicalCarets = [selection] + carets.enumerated().compactMap { $0.offset == primary ? nil : $0.element }
        let contextAfter = carets.count > 1 ? SourceSnippetHistoryContext(fields: [], active: 0, current: 0, ranges: logicalCarets) : nil
        guard sourceDraftHistory.commit(
            plan, path: buffer.file.path, before: source, selectionBefore: selected, selectionAfter: selection,
            contextBefore: context, contextAfter: contextAfter, manager: text.undoManager,
            read: { [weak self] in self?.buffer.text ?? "" },
            apply: { [weak self] edits, before, selection in
                guard let self, !self.buffer.resourceDeleted else { return false }
                return self.applySourceEdits(edits, source: before, resultSelection: selection, manageUndo: false, userInput: true)
            },
            notice: { [weak self] in self?.buffer.snippets.reportFailure($0) },
            restoreContext: { [weak self] in self?.buffer.snippets.restoreHistory($0) },
            contextCost: snippetHistoryCost, discardContext: { [weak self] in _ = self?.buffer.snippets.finish() }
        ) else { return range }
        guard let first = text.position(from: text.beginningOfDocument, offset: selection.location - count),
              let last = text.position(from: first, offset: count) else { return range }
        return text.textRange(from: first, to: last) ?? range
    }
    func applySourceEdits(_ edits: [NativeSourceEdit], source: String, expectedModelSource: String? = nil,resultSelection:NSRange?=nil,manageUndo:Bool=true,userInput:Bool=false,reviewedResource:Bool=false) -> Bool {
        guard !buffer.readOnly, (!buffer.saving || userInput || text.undoManager?.isUndoing==true || text.undoManager?.isRedoing==true), text.isEditable, exactUTF8(text.text,source), exactUTF8(buffer.text,expectedModelSource ?? source), text.markedTextRange == nil,
              (edits.isEmpty && resultSelection != nil || validSourceEdits(edits, length: (source as NSString).length)) else { return false }
        let ordered = edits.sorted { $0.range.location > $1.range.location }
        var resultingSelection = resultSelection
        if expectedModelSource != nil {
            let initial = text.selectedRange
            guard validSourceEdits([NativeSourceEdit(range: initial, text: "")], length: source.utf16.count) else { return false }
            resultingSelection = ordered.reduce(initial) { selectionAfterSourceEdit($0, edit: $1) }
        }
        let resultLength=source.utf16.count+edits.reduce(0){$0+$1.text.utf16.count-$1.range.length}
        if let selected=resultingSelection,!validSourceEdits([NativeSourceEdit(range:selected,text:"")],length:resultLength){return false}
        let exactStorage = !manageUndo || userInput || expectedModelSource != nil || reviewedResource
        let replacements: [(UITextRange, String)]
        if exactStorage {
            func scalarBoundary(_ offset: Int) -> Bool {
                String.Index(utf16Offset: offset, in: source).samePosition(in: source.unicodeScalars) != nil
            }
            guard exactUTF8(sourceTextStorage.string, source), ordered.allSatisfy({
                scalarBoundary($0.range.location) && scalarBoundary(NSMaxRange($0.range))
            }) else { return false }
            if let expectedModelSource {
                let reconstructed = NSMutableString(string: source)
                for edit in ordered { reconstructed.replaceCharacters(in: edit.range, with: edit.text) }
                guard exactUTF8(reconstructed as String, expectedModelSource) else { return false }
            }
            replacements = []
        } else {
            replacements = ordered.compactMap { edit in
                guard let start = text.position(from: text.beginningOfDocument, offset: edit.range.location),
                      let end = text.position(from: start, offset: edit.range.length),
                      let target = text.textRange(from: start, to: end) else { return nil }
                return (target, edit.text)
            }
            guard replacements.count == edits.count else { return false }
        }
        let wasMulti = buffer.applyingMulti; buffer.applyingMulti = true
        if !wasMulti { buffer.clearMulti() }
        defer { buffer.applyingMulti = wasMulti }
        let viewport=text.contentOffset
        unfoldAll()
        let undo = manageUndo && expectedModelSource == nil ? text.undoManager:nil; undo?.beginUndoGrouping()
        defer { undo?.endUndoGrouping() }
        if exactStorage {
            sourceTextStorage.beginEditing()
            for edit in ordered { sourceTextStorage.replaceCharacters(in: edit.range, with: edit.text) }
            sourceTextStorage.endEditing()
        } else {
            for (target, replacement) in replacements { text.replace(target, withText: replacement) }
        }
        if let selected = resultingSelection {
            text.selectedRange = selected
            text.setContentOffset(viewport, animated: false)
            if expectedModelSource == nil && !reviewedResource { text.scrollRangeToVisible(selected) }
        }
        if let expectedModelSource, !exactUTF8(text.text, expectedModelSource) { return false }
        textViewDidChange(text); return true
    }
    func toggleFold(_ line: Int) {
        guard let header = simpleFoldRanges(text.text ?? "", markdown: ["md", "markdown"].contains((buffer.file.path as NSString).pathExtension.lowercased())).first(where: { $0.line == line }) else { return }
        if let index = folding.ranges.firstIndex(of: header.range) { folding.ranges.remove(at: index); buffer.foldedLines.remove(line) }
        else {
            guard text.markedTextRange == nil else { return }
            let selection = text.selectedRange
            if NSIntersectionRange(selection, header.range).length > 0 || NSLocationInRange(selection.location, header.range) { text.selectedRange = NSRange(location: header.range.location, length: 0) }
            folding.ranges.append(header.range); buffer.foldedLines.insert(line)
        }
        invalidateFolds()
    }
    private func invalidateFolds() {
        let manager = text.layoutManager
        manager.invalidateGlyphs(forCharacterRange: NSRange(location: 0, length: ((text.text ?? "") as NSString).length), changeInLength: 0, actualCharacterRange: nil)
        measureDebugLines()
    }
    private func unfoldAll() {
        guard !folding.ranges.isEmpty else { return }
        folding.ranges = []; buffer.foldedLines = []; invalidateFolds()
    }
    private func equationPoint(_ point: CGPoint) -> CGPoint {
        CGPoint(x: point.x - text.textContainerInset.left, y: point.y - text.textContainerInset.top)
    }
    func gestureRecognizer(_ gestureRecognizer: UIGestureRecognizer, shouldReceive touch: UITouch) -> Bool {
        text.markedTextRange == nil &&
            (buffer.exposedEquation(at: equationPoint(touch.location(in: text)), layout: sourceLayoutManager, container: text.textContainer) != nil ||
             buffer.inlineLink(at: equationPoint(touch.location(in: text)), layout: sourceLayoutManager, container: text.textContainer) != nil)
    }
    @objc private func restoreEquationPreview(_ gesture: UITapGestureRecognizer) {
        let point = equationPoint(gesture.location(in: text))
        if !buffer.restoreEquationPreview(at: point, layout: sourceLayoutManager, container: text.textContainer) {
            _ = buffer.followInlineLink(at: point, layout: sourceLayoutManager, container: text.textContainer)
        }
    }
    func textView(_ textView: UITextView, shouldChangeTextIn range: NSRange, replacementText text: String) -> Bool {
        // The host may refresh a recorded reading. Text input and its history
        // have no write authority over the retained bytes.
        guard textView.isEditable, !buffer.readOnly else { return false }
        if buffer.applyingMulti { return true }
        if self.text.performingMarkedInput { return true }
        if textView.undoManager?.isUndoing == true || textView.undoManager?.isRedoing == true { buffer.clearMulti(); unfoldAll(); return true }
        if textView.undoManager?.isUndoing != true && textView.undoManager?.isRedoing != true,
           let equation = folding.equations.first(where: { NSIntersectionRange($0.range, range).length > 0 || range.length == 0 && range.location > $0.range.location && NSLocationInRange(range.location, $0.range) }) {
            buffer.sourceTable = buffer.inlineMarkup.equations.first { $0.renderedRange == equation.range }?.id
            measureDebugLines(); return false
        }
        if text == "\n" || text == "\t" {
            if tableSourceEditHitsGrid(range, selection: textView.selectedRange, tables: self.text.tableRanges) { return false }
            if folding.ranges.contains(where: { NSIntersectionRange($0, range).length > 0 || NSLocationInRange(range.location, $0) }) { unfoldAll(); return false }
        }
        if text=="\t",textView.markedTextRange==nil,buffer.snippets.move(1){return false}
        if textView.markedTextRange==nil,buffer.snippets.parallel != nil,buffer.multiRanges.count>1 {
            if buffer.snippets.inputParallel(source:textView.text,primaryRange:range,text:text){return false}
            _ = buffer.snippets.finish()
        }
        if text != "\n",text != "\t",textView.markedTextRange==nil,buffer.snippets.active != nil,!tableSourceEditHitsGrid(range,selection:textView.selectedRange,tables:self.text.tableRanges),!folding.ranges.contains(where:{NSIntersectionRange($0,range).length>0||NSLocationInRange(range.location,$0)}),buffer.snippets.input(source:textView.text,range:range,text:text) != nil{return false}
        if text == "\n", textView.markedTextRange == nil, !buffer.readOnly, buffer.multiRanges.count < 2 {
            let plan = smartEnter(textView.text, range: range, path: buffer.file.path, unit: simpleIndent(textView.text), mode: UserDefaults.standard.string(forKey: "editor.enterIndent") ?? "smart")
            guard !tableSourceEditHitsGrid(plan.range, selection: textView.selectedRange, tables: self.text.tableRanges) else { return false }
            if buffer.snippets.active != nil,buffer.snippets.input(source:textView.text,range:plan.range,text:plan.text,caret:plan.caret) != nil{return false}
            // Let UIKit insert and record an ordinary Return in its native
            // history. Only indentation or range changes need a replacement.
            if plan.range == range, plan.text == text { unfoldAll(); return true }
            if applySourceEdits([NativeSourceEdit(range: plan.range, text: plan.text)], source: textView.text) {
                textView.selectedRange = NSRange(location: plan.caret, length: 0); buffer.selection = textView.selectedRange
            }
            return false
        }
        if text.isEmpty, range.length == 1, textView.selectedRange.length == 0, NSMaxRange(range) == textView.selectedRange.location,
           textView.markedTextRange == nil, !buffer.readOnly, buffer.multiRanges.count < 2,
           let deletion = simpleBackspaceRange(textView.text, at: NSMaxRange(range)), deletion.length > 1 {
            guard !tableSourceEditHitsGrid(deletion, selection: textView.selectedRange, tables: self.text.tableRanges) else { return false }
            if folding.ranges.contains(where: { NSIntersectionRange($0, deletion).length > 0 }) { unfoldAll(); return false }
            _ = applySourceEdits([NativeSourceEdit(range: deletion, text: "")], source: textView.text); return false
        }
        if text == "\t", textView.markedTextRange == nil, !buffer.readOnly {
            _ = applySourceEdits(simpleTabEdits(textView.text, range: range), source: textView.text)
            return false
        }
        if textView.undoManager?.isUndoing == true || textView.undoManager?.isRedoing == true { buffer.clearMulti(); unfoldAll(); return true }
        if exactUTF8(buffer.multiSource,buffer.text), buffer.multiRanges.count > 1, textView.markedTextRange == nil {
            _ = buffer.replaceMultiple(text, affected: range); return false
        }
        if tableSourceEditHitsGrid(range, selection: textView.selectedRange, tables: self.text.tableRanges) { return false }
        let hidden = folding.ranges.contains { NSIntersectionRange($0, range).length > 0 || NSLocationInRange(range.location, $0) }
        unfoldAll(); return !hidden
    }
    func textView(_ textView: UITextView, editMenuForTextIn range: NSRange, suggestedActions: [UIMenuElement]) -> UIMenu? {
        guard textView.markedTextRange == nil, let link = text.linkAtCaret?() else { return nil }
        let open = UIAction(title: "Open Link") { [weak self] _ in
            guard let self, self.text.markedTextRange == nil else { return }
            _ = self.text.openLink?(link)
        }
        return UIMenu(children: [open] + suggestedActions)
    }
    func textViewDidChange(_ textView: UITextView) { if !buffer.applyingMulti { buffer.clearMulti() }; unfoldAll(); buffer.selection = textView.selectedRange; buffer.text = textView.text; presentMarkdown(); measureDebugLines() }
    func textViewDidChangeSelection(_ textView: UITextView) {
        if let equation = folding.equations.first(where: { textView.selectedRange.location > $0.range.location && NSLocationInRange(textView.selectedRange.location, $0.range) }) {
            buffer.sourceTable = buffer.inlineMarkup.equations.first { $0.renderedRange == equation.range }?.id
            measureDebugLines()
        }
        if folding.ranges.contains(where: { textView.selectedRange.location > $0.location && NSLocationInRange(textView.selectedRange.location, $0) }) { unfoldAll() }
        buffer.snippets.selectionChanged(textView.selectedRange)
        if !buffer.applyingMulti, let primary = buffer.multiRanges.first, textView.selectedRange != primary { buffer.clearMulti() }
        buffer.selection = textView.selectedRange
        presentMarkdown()
    }
    func presentMarkdown() {
        let snapshot = buffer.inlineMarkup, selection = text.selectedRange
        let carets = selection.length == 0 ? [selection.location] : []
        let glyphs = exactUTF8(snapshot.source, text.text ?? "") ? snapshot.markdownGlyphs(carets: carets) : [:]
        folding.presentMarkdown(glyphs, in: sourceLayoutManager)
    }
    func scrollViewDidScroll(_ scrollView: UIScrollView) { measureDebugLines() }
    func measureDebugLines() {
        presentTables()
        let manager = text.layoutManager, container = text.textContainer
        let source = (text.text ?? "") as NSString, inset = text.textContainerInset
        let visible = CGRect(x: 0, y: text.contentOffset.y - inset.top, width: text.bounds.width, height: text.bounds.height)
        manager.ensureLayout(forBoundingRect: visible, in: container)
        let glyphs = manager.glyphRange(forBoundingRect: visible, in: container)
        let visibleCharacters = manager.characterRange(forGlyphRange: glyphs, actualGlyphRange: nil)
        let starts = exactUTF8(buffer.inlineMarkup.source,source as String) ? buffer.inlineMarkup.lineStarts : nil
        var rows: [DebugLine] = []
        manager.enumerateLineFragments(forGlyphRange: glyphs) { rect, _, _, range, _ in
            let offset = manager.characterIndexForGlyph(at: range.location)
            guard !self.folding.contains(offset), offset == 0 || source.character(at: offset - 1) == 10 else { return }
            let line = starts.map { inlineLineAt(offset, starts: $0) } ?? source.substring(to: offset).filter { $0 == "\n" }.count
            rows.append(DebugLine(line: line, y: rect.minY + inset.top - self.text.contentOffset.y, height: rect.height))
        }
        if source.length == 0 || source.character(at: source.length - 1) == 10 {
            let rect = manager.extraLineFragmentRect
            if rect.height > 0 {
                rows.append(DebugLine(line: starts.map { inlineLineAt(source.length, starts: $0) } ?? source.substring(to: source.length).filter { $0 == "\n" }.count,
                    y: rect.minY + inset.top - text.contentOffset.y, height: rect.height))
            }
        }
        var selections: [CGRect] = []
        if exactUTF8(buffer.multiSource,text.text) {
            for range in buffer.multiRanges.dropFirst() where range.location <= NSMaxRange(visibleCharacters) && NSMaxRange(range) >= visibleCharacters.location {
                guard let start = text.position(from: text.beginningOfDocument, offset: range.location), let end = text.position(from: start, offset: range.length), let target = text.textRange(from: start, to: end) else { continue }
                let rects = range.length == 0 ? [text.caretRect(for: start)] : text.selectionRects(for: target).map(\.rect)
                selections += rects.map { $0.offsetBy(dx: -text.contentOffset.x, dy: -text.contentOffset.y) }
            }
        }
        if selections != buffer.multiRects { buffer.multiRects = selections }
        if rows != buffer.visibleDebugLines { buffer.visibleDebugLines = rows }
    }

    // Until WebKit has measured a formula, keep its actual source readable.
    // Replace presentation glyphs only; the backing storage and undo never change.
    private var centeredMath = NSRange(location: 0, length: 0)
    private func presentMath() {
        let source = text.text ?? "", storage = text.textStorage
        let style = text.typingAttributes[.paragraphStyle] as? NSParagraphStyle ?? .default
        buffer.centerEquationSource(storage, previous: &centeredMath, baseStyle: style)
        let manager = text.layoutManager, container = text.textContainer
        let previous = folding.equations
        if !exactUTF8(mathSource,source) {
            for host in mathViews.values { host.view.removeFromSuperview() }
            mathViews = [:]; mathMeasurements = [:]; mathRequests = [:]; mathSource = source
        }
        let equations = exactUTF8(buffer.inlineMarkup.source,source) ? buffer.inlineMarkup.equations.filter { equation in
            equation.id != buffer.sourceTable && NSMaxRange(equation.renderedRange) <= storage.length &&
                !folding.ranges.contains(where: { NSIntersectionRange($0, equation.renderedRange).length > 0 })
        } : []
        let live = Set(equations.map(\.id))
        for id in Array(mathViews.keys) where !live.contains(id) {
            mathMeasurements.removeValue(forKey: id); mathRequests.removeValue(forKey: id)
            if let host = mathViews.removeValue(forKey: id) { host.view.removeFromSuperview() }
        }
        for equation in equations where mathViews[equation.id] == nil {
            let request = UUID(); mathRequests[equation.id] = request
            let preview = AnyView(NativeMathView(source: equation.body, inline: !equation.displayBlock, onSize: { [weak self] size in
                guard let self, exactUTF8(self.mathSource,source), exactUTF8(self.buffer.text,source),
                      self.buffer.sourceTable != equation.id, self.mathRequests[equation.id] == request else { return }
                if self.mathMeasurements[equation.id] != size {
                    self.mathMeasurements[equation.id] = size
                    self.measureDebugLines()
                }
            }).overlay {
                Rectangle().fill(.clear).contentShape(Rectangle())
                    .onTapGesture(count: 2, coordinateSpace: .local) { [weak self] point in
                        guard let self, self.mathRequests[equation.id] == request else { return }
                        let width = self.mathViews[equation.id]?.view.frame.width ?? 1
                        guard self.buffer.toggleEquationSource(equation.id, in: source) else { return }
                        let offset = equationSourceCaret(equation, fraction: point.x / max(1, width))
                        _ = self.text.becomeFirstResponder()
                        self.text.selectedRange = NSRange(location: offset, length: 0)
                    }
                    .accessibilityLabel("Equation")
                    .accessibilityAddTraits(.isButton)
                    .accessibilityAction { [weak self] in
                        guard let self, self.mathRequests[equation.id] == request else { return }
                        _ = self.buffer.toggleEquationSource(equation.id, in: source)
                    }
            })
            let host = EquationHost(rootView: preview); host.view.accessibilityIdentifier = "equation:\(equation.id)"
            host.view!.frame = CGRect(x: -10000, y: -10000, width: 240, height: 80)
            host.view.alpha = 0; mathViews[equation.id] = host; text.addSubview(host.view)
        }
        let width = max(1, container.size.width - container.lineFragmentPadding * 2 - 1)
        let layouts = equations.compactMap { equation -> EquationLayout? in
            guard let natural = mathMeasurements[equation.id] else { return nil }
            let scale = min(1, width / natural.width, (equation.displayBlock ? 240 : 120) / natural.height)
            return EquationLayout(range: equation.renderedRange, size: CGSize(width: equation.displayBlock ? width : ceil(natural.width * scale), height: ceil(natural.height * scale)))
        }
        if previous != layouts {
            folding.equations = layouts
            for item in previous {
                let range = NSIntersectionRange(item.range, NSRange(location: 0, length: storage.length))
                if range.length > 0 { storage.addAttribute(.foregroundColor, value: UIColor.label, range: range) }
            }
            for item in previous + layouts {
                let range = NSIntersectionRange(item.range, NSRange(location: 0, length: storage.length))
                if range.length > 0 { manager.invalidateGlyphs(forCharacterRange: range, changeInLength: 0, actualCharacterRange: nil) }
            }
        }
        for layout in layouts { storage.addAttribute(.foregroundColor, value: UIColor.clear, range: layout.range) }
        let origin = CGPoint(x: text.textContainerInset.left, y: text.textContainerInset.top)
        for equation in equations {
            guard let host = mathViews[equation.id], let layout = layouts.first(where: { $0.range == equation.renderedRange }) else { continue }
            manager.ensureLayout(forCharacterRange: layout.range)
            let glyph = manager.glyphIndexForCharacter(at: layout.range.location)
            let line = manager.lineFragmentRect(forGlyphAt: glyph, effectiveRange: nil), position = manager.location(forGlyphAt: glyph)
            host.view!.frame = CGRect(x: origin.x + line.minX + position.x, y: origin.y + line.minY + (line.height - layout.size.height) / 2, width: layout.size.width, height: layout.size.height)
            host.view.alpha = 1
        }
    }

    private func presentEquations() {
        let source = text.text ?? "", storage = text.textStorage
        presentMath()
        var grouped: [Int: [InlineEquation]] = [:]
        let outputs = buffer.recordedOutputPlacements()
            .merging(buffer.terminalOutputRevision == buffer.sourceEditIdentity ? buffer.terminalOutput : [:]) { _, live in live }
            .merging(exactUTF8(buffer.runOutputSource,source) ? buffer.runOutputs : [:]) { _, live in live }
        var starts = [0]
        for (index, unit) in source.utf16.enumerated() where unit == 10 { starts.append(index + 1) }
        for (line, output) in outputs where starts.indices.contains(line) && starts[line] < storage.length {
            grouped[line] = [InlineEquation(line: line, range: NSRange(location: starts[line], length: 0), body: output)]
        }
        let signature = source + "\0" + outputs.keys.sorted().map { "\($0):\(outputs[$0] ?? "")" }.joined()
        if equationSource != signature {
            storage.enumerateAttribute(.paragraphStyle, in: NSRange(location: 0, length: storage.length)) { value, range, _ in
                if (value as? NSParagraphStyle)?.paragraphSpacing == 88 || (value as? NSParagraphStyle)?.paragraphSpacing == 188 { storage.removeAttribute(.paragraphStyle, range: range) }
            }
            for host in equationViews.values { host.view.removeFromSuperview() }
            equationViews = [:]; equationSource = signature
        }
        for line in Array(equationViews.keys) where grouped[line] == nil {
            if let host = equationViews.removeValue(forKey: line) { host.view.removeFromSuperview() }
        }
        for (line, values) in grouped {
            guard let first = values.first, first.range.location < storage.length else { continue }
            let paragraph = (source as NSString).paragraphRange(for: NSRange(location: first.range.location, length: 0))
            let height: CGFloat = outputs[line] == nil ? 80 : 180
            let style = NSMutableParagraphStyle(); style.paragraphSpacing = height + 8
            if (storage.attribute(.paragraphStyle, at: first.range.location, effectiveRange: nil) as? NSParagraphStyle)?.paragraphSpacing != height + 8 { storage.addAttribute(.paragraphStyle, value: style, range: paragraph) }
            let preview: AnyView
            if let output = outputs[line] {
                preview = AnyView(NativeInlineTerminal(text: output, sourceLinks: buffer.runOutputs[line] == output ? buffer.runOutputLinks : [:], onSource: { [weak buffer = self.buffer] offset in
                    guard let buffer, exactUTF8(buffer.text, source), exactUTF8(buffer.runOutputSource, source),
                          buffer.runOutputs[line] == output, starts.indices.contains(offset) else { return }
                    if buffer.selectSourceRange?(NSRange(location: starts[offset], length: 0), source) == true { buffer.focusSource?() }
                }))
            } else { preview = AnyView(NativeMathView(source: values.map(\.body).joined(separator: " \\qquad "))) }
            let host: EquationHost
            if let existing = equationViews[line] { existing.rootView = preview; host = existing }
            else { host = EquationHost(rootView: preview); equationViews[line] = host; text.addSubview(host.view) }
            let manager = text.layoutManager, container = text.textContainer
            let glyph = manager.glyphRange(forCharacterRange: NSRange(location: first.range.location, length: 1), actualCharacterRange: nil)
            let rect = manager.boundingRect(forGlyphRange: glyph, in: container), origin = CGPoint(x: text.textContainerInset.left, y: text.textContainerInset.top)
            host.view.frame = CGRect(x: origin.x, y: rect.maxY + origin.y + 2, width: max(100, text.bounds.width - 16), height: height)
        }
    }

    private func presentDiagrams() {
        presentEquations()
        let source = text.text ?? ""
        let storage = text.textStorage
        let diagrams = !buffer.tableSourceVisible ? tableSnapshot.diagrams.filter { $0.id != buffer.sourceTable } : []
        let changed = diagrams.map(\.range) != folding.diagrams.map(\.range)
        if !exactUTF8(diagramSource,source) || diagramAnchor != buffer.sourceTable || changed {
            diagramSource = source; diagramAnchor = buffer.sourceTable
            storage.beginEditing()
            let tabs = NSMutableParagraphStyle()
            for old in folding.diagrams {
                let range = NSIntersectionRange(old.range, NSRange(location: 0, length: storage.length))
                if range.length > 0 { storage.addAttribute(.paragraphStyle, value: tabs, range: range); storage.addAttribute(.foregroundColor, value: UIColor.label, range: range); text.layoutManager.invalidateGlyphs(forCharacterRange: range, changeInLength: 0, actualCharacterRange: nil) }
            }
            folding.diagrams = diagrams
            let live = Set(diagrams.map(\.id))
            for id in Array(diagramViews.keys) where !live.contains(id) { diagramViews[id]?.view.removeFromSuperview(); diagramViews.removeValue(forKey: id) }
            for diagram in diagrams {
                let preview = NativeDiagramPreview(diagram: diagram, buffer: buffer)
                if let host = diagramViews[diagram.id] { host.rootView = preview }
                else { let host = DiagramHost(rootView: preview); diagramViews[diagram.id] = host; text.addSubview(host.view) }
            }
            storage.endEditing()
        }
        text.tableRanges = folding.tables.map(\.range) + diagrams.map(\.range)
        for diagram in diagrams {
            let natural = buffer.diagramSizes[diagram.id] ?? CGSize(width: 200, height: 80)
            let width = max(100, text.bounds.width - 16)
            let height: CGFloat = diagram.renderer == "terminal" ? 180 : max(24, ceil(natural.height * min(1, width / max(1, natural.width))))
            let style = NSMutableParagraphStyle(); style.minimumLineHeight = height; style.maximumLineHeight = height
            if (storage.attribute(.paragraphStyle, at: diagram.range.location, effectiveRange: nil) as? NSParagraphStyle)?.minimumLineHeight != height {
                storage.addAttribute(.paragraphStyle, value: style, range: NSRange(location: diagram.range.location, length: 1))
            }
            if storage.attribute(.foregroundColor, at: diagram.range.location, effectiveRange: nil) as? UIColor != .clear {
                storage.addAttribute(.foregroundColor, value: UIColor.clear, range: diagram.range)
            }
            if changed { text.layoutManager.invalidateGlyphs(forCharacterRange: diagram.range, changeInLength: 0, actualCharacterRange: nil) }
            let manager = text.layoutManager, container = text.textContainer
            let glyph = manager.glyphRange(forCharacterRange: NSRange(location: diagram.range.location, length: 1), actualCharacterRange: nil)
            let rect = manager.boundingRect(forGlyphRange: glyph, in: container)
            diagramViews[diagram.id]?.view.frame = CGRect(x: text.textContainerInset.left, y: rect.minY + text.textContainerInset.top, width: max(100, text.bounds.width - text.textContainerInset.left - text.textContainerInset.right), height: height)
        }
    }
    private func preserveViewport() -> () -> Void {
        let manager = text.layoutManager, container = text.textContainer
        guard manager.numberOfGlyphs > 0 else { return {} }
        let viewport = text.contentOffset
        let glyph = min(manager.glyphIndex(for: CGPoint(x: viewport.x, y: viewport.y - text.textContainerInset.top), in: container), manager.numberOfGlyphs - 1)
        let character = manager.characterIndexForGlyph(at: glyph)
        let before = manager.lineFragmentRect(forGlyphAt: glyph, effectiveRange: nil).minY
        return { [self] in
            guard character < (text.text as NSString).length else { return }
            manager.ensureLayout(forCharacterRange: NSRange(location: character, length: 1))
            let after = manager.lineFragmentRect(forGlyphAt: manager.glyphIndexForCharacter(at: character), effectiveRange: nil).minY
            text.setContentOffset(CGPoint(x: viewport.x, y: max(-text.adjustedContentInset.top, viewport.y + after - before)), animated: false)
        }
    }
    private func presentTables() {
        let restore = preserveViewport(); defer { restore() }
        let source = text.text ?? ""
        let storage = text.textStorage
        let parseCurrent = exactUTF8(buffer.inlineMarkup.source,source)
        if parseCurrent { tableSnapshot = buffer.inlineMarkup }
        else { tableSnapshot = tablePresentation(tableSnapshot, source: source) }
        let tables: [InlineTable] = !buffer.tableSourceVisible ? inlineTables(tableSnapshot).filter { $0.id != buffer.sourceTable }.map { value in var value = value; value.preferredHeight = buffer.tableHeights[value.id]; return value } : []
        let geometryChanged = folding.tables.map { [$0.range.location, $0.range.length, Int($0.height)] } != tables.map { [$0.range.location, $0.range.length, Int($0.height)] }
        if !exactUTF8(tableSource,source) || tableSourceVisible != buffer.tableSourceVisible || sourceTable != buffer.sourceTable || geometryChanged || tableParseCurrent != parseCurrent {
            tableParseCurrent = parseCurrent
            tableSource = source; tableSourceVisible = buffer.tableSourceVisible; sourceTable = buffer.sourceTable
            let tabs = NSMutableParagraphStyle(); tabs.tabStops = []
            tabs.defaultTabInterval = ("    " as NSString).size(withAttributes: [.font: text.font ?? .monospacedSystemFont(ofSize: 13, weight: .regular)]).width
            storage.beginEditing()
            for table in folding.tables {
                let range = NSIntersectionRange(table.range, NSRange(location: 0, length: storage.length))
                if range.length > 0 { storage.addAttribute(.foregroundColor, value: UIColor.label, range: range) }
            }
            if geometryChanged {
                storage.enumerateAttribute(.paragraphStyle, in: NSRange(location: 0, length: storage.length)) { value, range, _ in
                    if let style = value as? NSParagraphStyle, style.minimumLineHeight > 0 { storage.addAttribute(.paragraphStyle, value: tabs, range: range) }
                }
            }
            folding.tables = tables
            text.tableRanges = folding.tables.map(\.range)
            for table in folding.tables where table.range.length > 0 {
                let style = NSMutableParagraphStyle(); style.minimumLineHeight = table.height; style.maximumLineHeight = table.height
                storage.addAttribute(.paragraphStyle, value: style, range: NSRange(location: table.range.location, length: 1))
                storage.addAttribute(.foregroundColor, value: UIColor.clear, range: table.range)
            }
            storage.endEditing()
            // UIKit may lay out the insertion point when reading typing attributes.
            text.typingAttributes[.paragraphStyle] = tabs
            if geometryChanged {
                for table in tables { text.layoutManager.invalidateGlyphs(forCharacterRange: table.range, changeInLength: 0, actualCharacterRange: nil) }
            }
            let live = Set(folding.tables.map(\.id))
            for id in Array(tableViews.keys) where !live.contains(id) { tableViews[id]?.view.removeFromSuperview(); tableViews.removeValue(forKey: id) }
            for table in folding.tables {
                let grid = NativeTableGrid(table: table, source: source, buffer: buffer)
                if let host = tableViews[table.id] { host.rootView = grid } else { let host = UIHostingController(rootView: grid); host.view.backgroundColor = .systemBackground; tableViews[table.id] = host; text.addSubview(host.view) }
            }
        }
        let manager = text.layoutManager, container = text.textContainer
        for table in folding.tables {
            if table.range.length > 0 && storage.attribute(.foregroundColor, at: table.range.location, effectiveRange: nil) as? UIColor != .clear { storage.addAttribute(.foregroundColor, value: UIColor.clear, range: table.range) }
            let glyph = manager.glyphRange(forCharacterRange: NSRange(location: table.range.location, length: min(1, table.range.length)), actualCharacterRange: nil)
            let rect = table.range.length == 0 ? CGRect.zero : manager.boundingRect(forGlyphRange: glyph, in: container)
            tableViews[table.id]?.view.frame = CGRect(x: text.textContainerInset.left, y: rect.minY + text.textContainerInset.top, width: max(100, text.bounds.width - text.textContainerInset.left - text.textContainerInset.right), height: table.height)
        }
        presentDiagrams()
        if let viewport = tableViewport, exactUTF8(buffer.inlineMarkup.source,source) {
            tableViewport = nil; text.setContentOffset(viewport, animated: false)
        }
    }
    private func paint(_ tokens: [NativeSemanticToken], source: String) {
        presentMarkdown()
        guard exactUTF8(text.text,source), text.markedTextRange == nil else { return }
        let storage = text.textStorage
        let restore = preserveViewport(); defer { restore() }
        storage.beginEditing()
        storage.addAttribute(.foregroundColor, value: UIColor.label, range: NSRange(location: 0, length: storage.length))
        // Foreground colors reset on every paint, including provider-only refreshes.
        if exactUTF8(buffer.inlineMarkup.source,source), source.utf16.count <= 200_000 {
            for mark in buffer.inlineMarkup.marks where mark.kind.hasPrefix("syntax-") && NSMaxRange(mark.range) <= storage.length {
                let color: UIColor
                switch mark.kind {
                case "syntax-keyword": color = .systemPurple
                case "syntax-string": color = .systemGreen
                case "syntax-number": color = .systemOrange
                default: color = .secondaryLabel
                }
                storage.addAttribute(.foregroundColor, value: color, range: mark.range)
            }
        }
        for token in tokens where NSMaxRange(token.range) <= storage.length {
            let color: UIColor
            switch token.kind {
            case "keyword", "type", "class", "struct": color = .systemPurple
            case "function", "method": color = .systemBlue
            case "string": color = .systemGreen
            case "number": color = .systemOrange
            case "comment": color = .secondaryLabel
            default: color = .systemTeal
            }
            storage.addAttribute(.foregroundColor, value: color, range: token.range)
        }
        if exactUTF8(buffer.inlineMarkup.source,source) && !exactUTF8(paintedInlineSource,source) && (!buffer.inlineMarkup.marks.isEmpty || hadInlineFonts) && (source.utf16.count <= 200_000 || hadInlineFonts) {
            storage.addAttribute(.font, value: UIFont.monospacedSystemFont(ofSize: 13, weight: .regular), range: NSRange(location: 0, length: storage.length))
            storage.removeAttribute(.backgroundColor, range: NSRange(location: 0, length: storage.length))
            if source.utf16.count <= 200_000 {
                for mark in buffer.inlineMarkup.marks where NSMaxRange(mark.range) <= storage.length {
                    let font: UIFont
                    switch mark.kind {
                    case "heading": font = .systemFont(ofSize: CGFloat(max(14, 25 - mark.level * 2)), weight: .semibold)
                    case "strong": font = .boldSystemFont(ofSize: 13)
                    case "em", "math": font = .italicSystemFont(ofSize: 13)
                    default: font = .monospacedSystemFont(ofSize: 13, weight: .regular)
                    }
                    storage.addAttribute(.font, value: font, range: mark.range)
                    if mark.kind == "link" { storage.addAttribute(.foregroundColor, value: UIColor.systemBlue, range: mark.range) }
                    if mark.kind == "code" { storage.addAttribute(.backgroundColor, value: UIColor.secondarySystemBackground, range: mark.range) }
                    if mark.kind == "diff-add" || mark.kind == "diff-remove" { storage.addAttribute(.backgroundColor, value: (mark.kind == "diff-add" ? UIColor.systemGreen : UIColor.systemRed).withAlphaComponent(0.15), range: mark.range) }
                }
            }
            paintedInlineSource = source; hadInlineFonts = !buffer.inlineMarkup.marks.isEmpty
        }
        storage.endEditing(); text.typingAttributes[.foregroundColor] = UIColor.label
        presentTables()
    }
}
@MainActor private enum EditorCache { static var views: [UUID: MobileEditor] = [:] }
private final class MobileEditorHost: UIView {
    let content: UIView
    let measure: () -> Void
    init(_ content: UIView, measure: @escaping () -> Void) { self.content = content; self.measure = measure; super.init(frame: .zero) }
    required init?(coder: NSCoder) { fatalError("init(coder:) has not been implemented") }
    func attach() {
        guard window != nil else { return }
        if content.superview !== self { content.removeFromSuperview(); addSubview(content) }
        content.frame = bounds
    }
    override func didMoveToWindow() { super.didMoveToWindow(); attach() }
    override func layoutSubviews() { super.layoutSubviews(); attach(); Task { @MainActor in measure() } }
}
struct NativeEditor: UIViewRepresentable {
    let buffer: DocumentBuffer
    var readOnly: Bool? = nil
    var submitMessage: (() -> Void)? = nil
    func sizeThatFits(_ proposal: ProposedViewSize, uiView: UIView, context: Context) -> CGSize? {
        CGSize(width: proposal.width ?? 600, height: proposal.height ?? 400)
    }
    func makeUIView(context: Context) -> UIView {
        let editor = EditorCache.views[buffer.id] ?? MobileEditor(buffer)
        EditorCache.views[buffer.id] = editor
        editor.text.submitMessage = submitMessage
        return MobileEditorHost(editor.text) { [weak editor] in editor?.measureDebugLines() }
    }
    func updateUIView(_ view: UIView, context: Context) {
        if let editor = EditorCache.views[buffer.id] {
            editor.text.submitMessage = submitMessage
            if !exactUTF8(editor.text.accessibilityLabel, buffer.file.path) {
                editor.text.accessibilityLabel = buffer.file.path
            }
            if !exactUTF8(editor.text.text, buffer.text) {
                let viewport = editor.text.contentOffset
                let length = buffer.text.utf16.count
                let start = min(buffer.selection.location, length)
                let selection = NSRange(location: start, length: min(buffer.selection.length, length - start))
                editor.text.text = buffer.text
                editor.text.selectedRange = selection
                editor.presentMarkdown()
                editor.text.setContentOffset(viewport, animated: false)
                editor.text.undoManager?.removeAllActions()
            }
            let editable = !(readOnly ?? buffer.readOnly)
            if editor.text.isEditable != editable {
                let selection = editor.text.selectedRange, viewport = editor.text.contentOffset
                editor.text.isEditable = editable
                if editor.text.selectedRange != selection { editor.text.selectedRange = selection }
                if editor.text.contentOffset != viewport { editor.text.setContentOffset(viewport, animated: false) }
            }
        }
        (view as? MobileEditorHost)?.attach(); view.setNeedsLayout()
        if let pending=buffer.recoveryViewport,let editor=EditorCache.views[buffer.id]{let revision=buffer.sourceEditIdentity,selection=editor.text.selectedRange;Task { @MainActor in
            guard buffer.recoveryViewport != nil,buffer.sourceEditIdentity==revision,editor.text.selectedRange==selection,editor.text.window != nil else{return}
            buffer.recoveryViewport=nil;let range=NSRange(location:min(pending.selection.anchor,pending.selection.head),length:abs(pending.selection.head-pending.selection.anchor));editor.text.selectedRange=range;buffer.selection=range;editor.text.layoutIfNeeded();editor.text.setContentOffset(CGPoint(x:pending.scroll_x,y:pending.scroll_y),animated:false)
        }}
        Task { @MainActor in EditorCache.views[buffer.id]?.measureDebugLines() }
    }
}
#endif

struct NativeSourceEdit: Equatable, Sendable { var range: NSRange; var text: String }
struct NativeSearchRequest: Equatable, Sendable {
    var source: String; var query: String; var replacement: String; var caseSensitive: Bool; var regex: Bool
}
struct NativeSearchResult: Sendable {
    var edits: [NativeSourceEdit] = []; var error = ""
}
func nativeSearch(_ request: NativeSearchRequest) -> NativeSearchResult {
    guard !request.query.isEmpty else { return NativeSearchResult() }
    do {
        let pattern = request.regex ? request.query : NSRegularExpression.escapedPattern(for: request.query)
        let expression = try NSRegularExpression(pattern: pattern, options: request.caseSensitive ? [.anchorsMatchLines] : [.anchorsMatchLines, .caseInsensitive])
        let source = request.source as NSString
        var result = NativeSearchResult()
        expression.enumerateMatches(in: request.source, options: .reportProgress, range: NSRange(location: 0, length: source.length)) { match, _, stop in
            if Task.isCancelled { stop.pointee = true; return }
            guard let match, match.range.length > 0 else { return }
            guard result.edits.count < 10_000 else { result = NativeSearchResult(error: "Too many matches. Narrow your search before replacing."); stop.pointee = true; return }
            let groups = (0..<match.numberOfRanges).map { index -> String in
                let range = match.range(at: index); return range.location == NSNotFound ? "" : source.substring(with: range)
            }
            let replacement = request.regex ? nativeSearchReplacement(request.replacement, groups) : request.replacement
            result.edits.append(NativeSourceEdit(range: match.range, text: replacement))
        }
        return result
    } catch { return NativeSearchResult(error: "Invalid regular expression. Check the pattern and try again.") }
}
// Use the editor's $1 / $& / $$ replacement vocabulary rather than ICU templates.
func nativeSearchReplacement(_ template: String, _ groups: [String]) -> String {
    let expression = try! NSRegularExpression(pattern: #"\$([$&]|\d+)"#)
    let source = template as NSString
    var result = template
    for token in expression.matches(in: template, range: NSRange(location: 0, length: source.length)).reversed() {
        let key = source.substring(with: token.range(at: 1))
        var value = source.substring(with: token.range)
        if key == "$" { value = "$" }
        else if key == "&" { value = groups.first ?? "" }
        else {
            for length in stride(from: key.count, through: 1, by: -1) {
                if let number = Int(key.prefix(length)), number > 0, number < groups.count {
                    value = groups[number] + key.dropFirst(length); break
                }
            }
        }
        result = (result as NSString).replacingCharacters(in: token.range, with: value)
    }
    return result
}
func validSourceEdits(_ edits: [NativeSourceEdit], length: Int) -> Bool {
    var end = 0
    for edit in edits.sorted(by: { $0.range.location < $1.range.location }) {
        guard edit.range.location >= end, edit.range.location <= length, edit.range.length >= 0, edit.range.length <= length - edit.range.location else { return false }
        end = edit.range.location + edit.range.length
    }
    return !edits.isEmpty
}
struct NativeFindPanel: View {
    @Bindable var buffer: DocumentBuffer
    @State private var result = NativeSearchResult()
    @State private var measured: NativeSearchRequest?
    @State private var notice = ""
    @FocusState private var queryFocused: Bool
    private var request: NativeSearchRequest { NativeSearchRequest(source: buffer.text, query: buffer.findQuery, replacement: buffer.findReplacement, caseSensitive: buffer.findCase, regex: buffer.findRegex) }
    private var current: Bool { measured == request && result.error.isEmpty }
    private func move(_ backwards: Bool = false) {
        guard current, !result.edits.isEmpty else { return }
        let selection = buffer.selection
        let match = backwards ? (result.edits.last { NSMaxRange($0.range) <= selection.location } ?? result.edits.last!) : (result.edits.first { $0.range.location >= NSMaxRange(selection) } ?? result.edits.first!)
        if buffer.selectSourceRange?(match.range, request.source) != true { notice = "Text changed or input is composing. Finish typing and search again." }
    }
    private func replace(_ all: Bool) {
        guard current, !buffer.readOnly else { return }
        let edits = all ? result.edits : result.edits.filter { $0.range == buffer.selection }
        guard !edits.isEmpty else { move(); return }
        if buffer.applySourceEdits?(edits, request.source) != true { notice = "Replacement refused. Finish typing and search the current text again." }
    }
    var body: some View {
        VStack(alignment: .leading, spacing: 5) {
            HStack { Text("Find and replace").font(.caption); Spacer(); Button("×") { buffer.closeFind() }.accessibilityLabel("Close search") }
            TextField("Find", text: $buffer.findQuery).focused($queryFocused).onSubmit { move() }.accessibilityLabel("Find")
            TextField("Replace with", text: $buffer.findReplacement).accessibilityLabel("Replace with").disabled(reason: buffer.readOnly ? "This source is read-only." : nil)
            HStack { Toggle("Match case", isOn: $buffer.findCase); Toggle("Regex", isOn: $buffer.findRegex) }.toggleStyle(.switch)
            HStack { Button("Previous") { move(true) }; Button("Next") { move() }
                Button("Replace") { replace(false) }.disabled(reason: buffer.readOnly ? "This source is read-only." : nil)
                Button("Replace all") { replace(true) }.disabled(reason: buffer.readOnly ? "This source is read-only." : nil)
            }
            HStack {
                Button("Select all matches") {
                    if buffer.selectMultiple(result.edits.map(\.range), source: request.source) { buffer.closeFind() }
                    else { notice = "Select between 2 and 256 current matches." }
                }.disabled(reason: !current ? "Refresh the results against the current source." : result.edits.count < 2 ? "There is only one replacement to review." : nil)
                if buffer.multiRanges.count > 1 { Button("Clear selections") { buffer.clearMulti() } }
            }.disabled(reason: !current ? "Refresh the results against the current source." : result.edits.isEmpty ? "There are no matching replacements." : nil)
            Text(!notice.isEmpty ? notice : measured != request ? "Searching…" : !result.error.isEmpty ? result.error : "\(result.edits.count) matches").font(.caption).accessibilityLabel("Search status")
            if buffer.readOnly { Text("Read-only · searching is available").font(.caption) }
        }.padding(8).background(.regularMaterial).overlay(Rectangle().stroke(.secondary.opacity(0.4))).shadow(radius: 4)
            .onAppear { queryFocused = true }
            .onKeyPress(.escape) { buffer.closeFind(); return .handled }
            .task(id: request) {
                let snapshot = request; measured = nil; notice = ""
                try? await Task.sleep(for: .milliseconds(100))
                guard !Task.isCancelled else { return }
                let job = Task.detached(priority: .userInitiated) { nativeSearch(snapshot) }
                let found = await withTaskCancellationHandler(operation: { await job.value }, onCancel: { job.cancel() })
                guard !Task.isCancelled, request == snapshot else { return }
                result = found; measured = snapshot
            }
    }
}

// Source editing stays in TextKit. A double activation on the exposed
// equation returns to its preview only when it hits a glyph of that equation.
@MainActor extension DocumentBuffer {
    func centerEquationSource(_ storage: NSTextStorage, previous: inout NSRange, baseStyle: NSParagraphStyle) {
        if sourceTable != nil && !exactUTF8(inlineMarkup.source, text) { return }
        let full = NSRange(location: 0, length: storage.length)
        let old = NSIntersectionRange(previous, full)
        let equation = inlineMarkup.equations.first { $0.id == sourceTable && $0.displayBlock }
        let range: NSRange
        if let equation, exactUTF8(inlineMarkup.source, text), NSMaxRange(equation.renderedRange) <= storage.length {
            range = (storage.string as NSString).lineRange(for: equation.renderedRange)
        } else { range = NSRange(location: 0, length: 0) }
        if range != previous && old.length > 0 { storage.addAttribute(.paragraphStyle, value: baseStyle, range: old) }
        previous = range
        guard range.length > 0 else { return }
        let current = storage.attribute(.paragraphStyle, at: range.location, effectiveRange: nil) as? NSParagraphStyle
        if current?.alignment != .center {
            let centered = baseStyle.mutableCopy() as! NSMutableParagraphStyle
            centered.alignment = .center
            storage.addAttribute(.paragraphStyle, value: centered, range: range)
        }
    }

    func inlineLink(at point: CGPoint, layout: NSLayoutManager, container: NSTextContainer) -> InlineLink? {
        guard openDiagramFile != nil, exactUTF8(inlineMarkup.source, text), layout.numberOfGlyphs > 0 else { return nil }
        let glyph = layout.glyphIndex(for: point, in: container)
        guard glyph < layout.numberOfGlyphs,
              layout.boundingRect(forGlyphRange: NSRange(location: glyph, length: 1), in: container).contains(point) else { return nil }
        return inlineMarkup.links.first { NSLocationInRange(layout.characterIndexForGlyph(at: glyph), $0.range) }
    }

    func inlineLink(at selection: NSRange, in source: String) -> InlineLink? {
        guard openDiagramFile != nil, exactUTF8(source, text), exactUTF8(inlineMarkup.source, text) else { return nil }
        return inlineMarkup.links.first {
            NSLocationInRange(selection.location, $0.sourceRange) && NSMaxRange(selection) <= NSMaxRange($0.sourceRange)
        }
    }

    @discardableResult func followInlineLink(at point: CGPoint, layout: NSLayoutManager, container: NSTextContainer) -> Bool {
        guard let link = inlineLink(at: point, layout: layout, container: container) else { return false }
        return followInlineLink(link)
    }

    func followInlineLink(_ link: InlineLink) -> Bool {
        guard let openDiagramFile, exactUTF8(inlineMarkup.source, text) else { return false }
        openDiagramFile(link.href)
        return true
    }

    func exposedEquation(at point: CGPoint, layout: NSLayoutManager, container: NSTextContainer) -> InlineEquation? {
        guard let sourceTable, exactUTF8(inlineMarkup.source, text),
              let equation = inlineMarkup.equations.first(where: { $0.id == sourceTable }),
              layout.numberOfGlyphs > 0 else { return nil }
        let glyph = layout.glyphIndex(for: point, in: container)
        guard glyph < layout.numberOfGlyphs,
              NSLocationInRange(layout.characterIndexForGlyph(at: glyph), equation.renderedRange),
              layout.boundingRect(forGlyphRange: NSRange(location: glyph, length: 1), in: container).contains(point)
        else { return nil }
        return equation
    }

    @discardableResult func restoreEquationPreview(at point: CGPoint, layout: NSLayoutManager, container: NSTextContainer) -> Bool {
        guard let equation = exposedEquation(at: point, layout: layout, container: container) else { return false }
        return toggleEquationSource(equation.id, in: text)
    }

    @discardableResult func toggleEquationSource(_ id: Int, in source: String) -> Bool {
        guard exactUTF8(text, source), exactUTF8(inlineMarkup.source, source),
              inlineMarkup.equations.contains(where: { $0.id == id }) else { return false }
        sourceTable = sourceTable == id ? nil : id
        return true
    }
}

struct LanguageEditor: View {
    var showRunTerminal: ((String) -> Void)? = nil
    @State private var railWidth: CGFloat = 24
    @State private var resizeStart: CGFloat?
    var buffer: DocumentBuffer
    var debugger: NativeDebugger?
    var recordedDebug: NativeRecordedMessageSource? = nil
    var recordedRunEngine: Engine? = nil
    var recordedPatchReview: ((Int) -> Void)? = nil
    var presentRecordedInspection: ((NativeDebugInspector) -> Void)? = nil
    var submitMessage: (() -> Void)? = nil
    private var breakpointMarkup: InlineDebugContext? {
        guard debugger != nil, recordedDebug?.current() != false,
              exactUTF8(buffer.inlineMarkup.source, buffer.text) else { return nil }
        return buffer.inlineMarkup.debug
    }
    private func canToggleBreakpoint(on line: Int, in markup: InlineDebugContext?) -> Bool {
        guard let markup else { return false }
        if recordedDebug != nil { return markup.recorded.values.contains { $0.contains(line) } }
        return markup.code[line] != nil
    }
    private func lineNumber(_ line: Int, marker: String?) -> some View {
        ZStack {
            if let marker {
                Text(marker == "refused" ? "○" : marker == "bound" ? "●" : "◌")
                    .font(.system(size: 11)).foregroundStyle(.red)
                    .frame(maxWidth: .infinity, alignment: .leading).padding(.leading, 2)
            }
            Text(String(line + 1)).font(.system(size: 13, design: .monospaced)).foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .trailing).padding(.trailing, 6)
        }.frame(maxWidth: .infinity, maxHeight: .infinity)
            .overlay(alignment: .trailing) {
                if let change = buffer.gitChanges?[line] {
                    Rectangle().fill(change.removed.isEmpty ? Color.green : change.added == nil ? Color.red : Color.orange)
                        .frame(width: 3).allowsHitTesting(false).accessibilityHidden(true)
                }
            }
    }
    private var lineNumbers: some View {
        GeometryReader { geometry in
            let breakpoints = debugger?.breakpointMarkers(buffer) ?? []
            let markup = breakpointMarkup
            let rows = buffer.visibleDebugLines.filter { $0.y + $0.height > 0 && $0.y < geometry.size.height }
            ZStack(alignment: .topLeading) {
                ForEach(rows, id: \.line) { row in
                    let marker = breakpoints.first { $0["line"] as? Int == row.line }?["state"] as? String
                    Group {
                        if canToggleBreakpoint(on: row.line, in: markup) {
                            Button {
                                if canToggleBreakpoint(on: row.line, in: breakpointMarkup) { debugger?.toggle(buffer, line: row.line) }
                            } label: {
                                lineNumber(row.line, marker: marker)
                            }
                            .buttonStyle(SimpleChromeButtonStyle())
                            .accessibilityLabel("Toggle breakpoint on line \(row.line + 1)")
                            .accessibilityValue(marker ?? "No breakpoint")
                        } else {
                            lineNumber(row.line, marker: marker).accessibilityHidden(true).allowsHitTesting(false)
                        }
                    }
                    .frame(width: geometry.size.width, height: row.height)
                    .position(x: geometry.size.width / 2, y: row.y + row.height / 2)
                }
            }.clipped()
        }
        .frame(width: CGFloat(max(3, String(buffer.text.filter { $0 == "\n" }.count + 1).count)) * 8 + 12)
        .accessibilityElement(children: .contain)
        .accessibilityLabel("Line numbers; click code to toggle breakpoint")
    }
    var body: some View {
        HStack(spacing: 0) {
            GeometryReader { geometry in
                let headers = Set(simpleFoldRanges(buffer.text, markdown: ["md", "markdown"].contains((buffer.file.path as NSString).pathExtension.lowercased())).map(\.line))
                ZStack(alignment: .topLeading) {
                    ForEach(buffer.visibleDebugLines.filter { headers.contains($0.line) && $0.y >= 0 && $0.y < geometry.size.height }, id: \.line) { row in
                        Button(buffer.foldedLines.contains(row.line) ? "+" : "−") { buffer.toggleFold?(row.line) }
                            .buttonStyle(SimpleChromeButtonStyle()).font(.system(size: 13)).frame(width: 24, height: min(18, row.height))
                            .accessibilityLabel("\(buffer.foldedLines.contains(row.line) ? "Unfold" : "Fold") line \(row.line + 1)")
                            .position(x: 12, y: row.y + row.height / 2)
                    }
                }.clipped()
            }.frame(width: 24)
            lineNumbers
            NativeEditor(buffer: buffer, readOnly: buffer.readOnly, submitMessage: submitMessage).overlay {
                // Paint stopped execution independently of selection and text styling.
                // The source guard retires positional facts as soon as displayed bytes change.
                Canvas { context, size in
                    if let debugger, debugger.sourceMatches(buffer), let line = debugger.line(in: buffer.file.path),
                       let row = buffer.visibleDebugLines.first(where: { $0.line == line }) {
                        context.fill(Path(CGRect(x: 0, y: row.y, width: size.width, height: row.height)), with: .color(.yellow.opacity(0.18)))
                        // Watch answers belong to this selected stopped frame.
                        // Draw in trailing whitespace, without inserting source or
                        // moving later lines, and retire them on resume or edits.
                        let values = debugger.watches.compactMap { expression in debugger.values[expression].map { "\(expression) = \($0)" } }.joined(separator: "  ")
                        if !values.isEmpty {
                            let rows = buffer.text.components(separatedBy: "\n")
                            if rows.indices.contains(line) {
                                let source = context.resolve(Text(rows[line]).font(.system(size: 13, design: .monospaced)))
                                let width = source.measure(in: CGSize(width: size.width, height: .infinity)).width
                                context.draw(Text(String(values.prefix(240))).font(.system(size: 11, design: .monospaced)).foregroundColor(.secondary), at: CGPoint(x: width + 12, y: row.y + row.height / 2), anchor: .leading)
                            }
                        }
                    }
                    for rect in buffer.multiRects { context.fill(Path(rect), with: .color(.accentColor.opacity(rect.width <= 2 ? 0.8 : 0.22))) }
                }.allowsHitTesting(false).clipped()
            }
            GeometryReader { geometry in
                Canvas { context, size in
                    let lines = buffer.text.components(separatedBy: "\n")
                    let rows = min(lines.count, max(1, Int(size.height / 2)))
                    for row in 0..<rows {
                        let line = lines[row * lines.count / rows]
                        context.fill(Path(CGRect(x: 2, y: Double(row) * size.height / Double(rows), width: min(34, Double(line.count) * 0.65), height: 1)), with: .color(.secondary.opacity(0.5)))
                    }
                }.contentShape(Rectangle())
                    .gesture(DragGesture(minimumDistance: 0).onChanged { value in buffer.navigate?(min(1, max(0, value.location.y / max(1, geometry.size.height)))) })
                    .accessibilityLabel("Document minimap")
                    .accessibilityAdjustableAction { direction in buffer.navigate?(direction == .increment ? 1 : 0) }
            }.frame(width: 40)
            if let debugger {
                Rectangle().fill(.secondary.opacity(0.35)).frame(width: 4).contentShape(Rectangle())
                    .gesture(DragGesture(minimumDistance: 0).onChanged { value in
                        if resizeStart == nil { resizeStart = railWidth }
                        railWidth = min(220, max(24, (resizeStart ?? 24) - value.translation.width))
                    }.onEnded { _ in resizeStart = nil })
                    .accessibilityLabel("Resize right gutter").accessibilityAdjustableAction { direction in railWidth = min(220, max(24, railWidth + (direction == .increment ? 22 : -22))) }
                NativeDebugRail(buffer: buffer, debug: debugger, showRunTerminal: showRunTerminal, recordedDebug: recordedDebug, recordedRunEngine: recordedRunEngine, recordedPatchReview: recordedPatchReview, presentRecordedInspection: presentRecordedInspection).frame(width: railWidth)
            } else if buffer.startupObservation != nil {
                Color.clear.frame(width: railWidth + 4)
            }
        }.overlay {
            if recordedDebug == nil && buffer.canAuthorElements { NativeElementForm(buffer: buffer).id(buffer.id) }
        }.onChange(of:buffer.sourceRevision,initial:true){_,_ in debugger?.sourceChanged(buffer);buffer.readTestSources();buffer.readCoverageSource()}
        .onChange(of: debugger?.output ?? [], initial: true) { _, _ in
            buffer.runOutputSource = buffer.text
            buffer.runOutputLinks = debugger?.outputLinks(buffer) ?? [:]
            buffer.runOutputs = debugger?.outputPlacements(buffer) ?? [:]
        }.overlay(alignment: .topTrailing) {
            if recordedDebug == nil || buffer.language != nil {
            Button("⌁") {
                #if os(macOS)
                NSApp.activate()
                #endif
                Task { await buffer.language?.inspect() }
                Task { await buffer.language?.complete() }
            }.accessibilityLabel("Language tools")
                .help(buffer.language?.status ?? "Language services not connected")
                .padding(4).background(.regularMaterial).padding(.trailing, 42)
                .popover(isPresented: Binding(get: { buffer.showSnippetTools || (buffer.language?.showingSuggestions ?? false) }, set: { buffer.showSnippetTools=$0;buffer.language?.showingSuggestions = $0;if !$0{buffer.language?.cancelCompletion()} })) {
                    ScrollView { VStack(alignment: .leading, spacing: 8) {
                        Text("Language tools").font(.headline)
                        if buffer.snippets.isActive {
                            Button("Next template field"){_ = buffer.snippets.move(1)}
                            Button("Previous template field"){_ = buffer.snippets.move(-1)}
                            ForEach(Array(buffer.snippets.choices.enumerated()),id:\.offset){_,choice in Button(choice.isEmpty ? "Empty value":choice){_ = buffer.snippets.choose(choice)}}
                            Button("Finish template editing"){_ = buffer.snippets.finish()}
                        }
                        Text(buffer.snippets.isActive ? buffer.snippets.notice : buffer.language?.completionNotice.isEmpty==false ? buffer.language!.completionNotice : buffer.language?.status ?? "Language services not connected").font(.caption).textSelection(.enabled)
                        if let language = buffer.language {
                            TextField("Symbol search", text: Binding(get: { language.symbolQuery }, set: { language.symbolQuery = $0 })).textFieldStyle(.roundedBorder)
                            ForEach(SemanticQuery.allCases) { query in
                                Button(query.label) { Task { await language.navigate(query) } }
                            }
                            if !language.locationStatus.isEmpty { Text(language.locationStatus).font(.caption).textSelection(.enabled) }
                            ForEach(language.locations) { location in
                                Button(location.label) { language.open(location) }
                                    .disabled(reason: location.path == nil ? "The provider gave no path for this location." : !exactUTF8(buffer.text,language.locationSource) ? "Refresh the location against the current source." : buffer.selection != language.locationSelection ? "Request a new location from the current selection." : nil)
                            }
                            if !language.hover.isEmpty || !language.signature.isEmpty {
                                Text("Last received information; refresh for current position.").font(.caption).foregroundStyle(.secondary)
                                if !language.hover.isEmpty { Text(language.hover).textSelection(.enabled) }
                                if !language.signature.isEmpty { Text(language.signature).font(.system(.caption, design: .monospaced)).textSelection(.enabled) }
                            }
                            if !language.diagnostics.isEmpty {
                                if !exactUTF8(language.diagnosticSource,buffer.text) { Text("Cached diagnostics; refresh pending.").font(.caption) }
                                ForEach(Array(language.diagnostics.prefix(100))) { item in
                                    if item.range != nil { Button(item.message) { language.reveal(item) }.disabled(reason: !exactUTF8(language.diagnosticSource,buffer.text) ? "Refresh diagnostics against the current source." : nil) }
                                    else { Text(item.message + " (server message; source version unavailable)").font(.caption).textSelection(.enabled) }
                                }
                            }
                            if !language.inlayHints.isEmpty { Text("Last received inlay information").font(.caption); ForEach(Array(language.inlayHints.prefix(50).enumerated()), id: \.offset) { _, hint in Text(hint).font(.caption) } }
                            ScrollView {
                                VStack(alignment: .leading, spacing: 4) {
                                    ForEach(language.suggestions) { item in
                                        Button { Task {await language.accept(item)} } label: {
                                            VStack(alignment: .leading) { Text(item.label).font(.system(.body, design: .monospaced)); Text(item.detail).font(.caption).foregroundStyle(.secondary) }
                                        }.disabled(reason: !exactUTF8(item.source,buffer.text) ? "Request a new completion from the current source." : item.selection != buffer.selection ? "Request a new completion from the current selection." : buffer.readOnly ? "This source is read-only." : nil)
                                    }
                                }.frame(maxWidth: .infinity, alignment: .leading)
                            }.frame(height: 180)
                            Button("Retry language services") { language.start() }
                        }
                    }.padding() }.frame(width: 350, height: 420)
                }
            }
        }
        #if os(macOS)
        .overlay(alignment: .topLeading) {
            if exactUTF8(buffer.multiSource,buffer.text) && buffer.multiRanges.count > 1 {
                HStack {
                    Button("\(buffer.multiRanges.count) selections") { buffer.showMulti = true }
                    Button("×") { buffer.clearMulti() }.accessibilityLabel("Clear selections")
                }.font(.caption).padding(3).background(.regularMaterial)
                    .popover(isPresented: Binding(get: { buffer.showMulti }, set: { buffer.showMulti = $0 })) {
                        VStack { TextField("Insert at selections", text: Binding(get: { buffer.multiText }, set: { buffer.multiText = $0 }))
                            Button("Apply to selections") { _ = buffer.replaceMultiple(buffer.multiText); buffer.showMulti = false; buffer.focusSource?() }.disabled(reason: buffer.readOnly ? "This source is read-only." : nil)
                        }.padding().frame(width: 280)
                    }
            }
        }
        #endif
        .overlay(alignment: .topTrailing) {
            if buffer.showFind {
                GeometryReader { geometry in
                    ScrollView { NativeFindPanel(buffer: buffer) }
                        .modifier(SimpleReveal())
                        .frame(width: min(360, max(0, geometry.size.width - 8)), height: min(240, max(0, geometry.size.height - 8)))
                        .padding(4).frame(maxWidth: .infinity, alignment: .trailing)
                }
            }
        }
    }
}
struct InlineCellControl: View {
    var cell: InlineCell
    var source: String
    var buffer: DocumentBuffer
    @State private var draft = ""
    @State private var notice = ""
    var body: some View {
        HStack {
            TextField("Column \(cell.column + 1)", text: $draft).textFieldStyle(.roundedBorder)
            Button("Apply") {
                guard !exactUTF8(draft,cell.value) else { return }
                if buffer.applySourceEdits?([NativeSourceEdit(range: cell.range, text: inlineCellText(draft, format: cell.format, delimiter: cell.delimiter))], source) != true { notice = "Table changed. Reopen this row and try again." }
            }.disabled(reason: buffer.readOnly ? "This source is read-only." : nil)
            if !notice.isEmpty { Text(notice).font(.caption) }
        }.onAppear { draft = cell.value }
    }
}

@MainActor private func recordedDebugFences(_ buffer: DocumentBuffer, source: NativeRecordedMessageSource?) -> [Int: RecordedDebugFence] {
    guard source?.current() == true, exactUTF8(buffer.inlineMarkup.source, buffer.text) else { return [:] }
    return buffer.inlineMarkup.debug.recorded
}

struct NativeDebugRail: View {
    var buffer: DocumentBuffer
    var debug: NativeDebugger
    var showRunTerminal: ((String) -> Void)? = nil
    var recordedDebug: NativeRecordedMessageSource? = nil
    var recordedRunEngine: Engine? = nil
    var recordedPatchReview: ((Int) -> Void)? = nil
    var presentRecordedInspection: ((NativeDebugInspector) -> Void)? = nil
    @State private var recordedRun = NativeRecordedFenceRun()
    @State private var recordedConstruction = NativeRecordedConstruction()
    @State private var opened: Int?
    @State private var details: Int?
    @State private var gitLine: Int?
    @State private var commentLine: Int?
    @State private var pendingLineReading: LineReading?

    private struct LineReading {
        let action: String
        let line: Int
        let path: String
        let revision: UUID
    }

    private func showLineReading(_ reading: LineReading) {
        guard !buffer.resourceDeleted, reading.revision == buffer.sourceEditIdentity,
              exactUTF8(reading.path, buffer.file.path) else { return }
        if reading.action == "tests" {
            buffer.inspectTests?(reading.line)
        } else {
            buffer.inspectCoverage?(reading.line)
        }
    }

    private var paused: Int? { debug.sourceMatches(buffer) ? debug.line(in: buffer.file.path) : nil }
    private var showsSession: Bool { debug.hasSessionReading && (recordedDebug == nil || exactUTF8(debug.owner, buffer.file.path)) }
    private var recordedFences: [Int: RecordedDebugFence] { recordedDebugFences(buffer, source: recordedDebug) }
    private func inspect(_ line: Int) {
        if let presentRecordedInspection {
            presentRecordedInspection(NativeDebugInspector(buffer: buffer, debug: debug, recordedDebug: recordedDebug, line: line))
        } else {
            details = line
        }
    }
    private func canReviewRecordedPatch(_ line: Int) -> Bool {
        recordedPatchReview != nil && recordedDebug?.current() == true && exactUTF8(buffer.inlineMarkup.source, buffer.text)
            && buffer.inlineMarkup.debug.recordedPatches.contains(line)
    }
    private func canInspectRecordedConstruction(_ line: Int) -> Bool {
        recordedRunEngine != nil && recordedDebug?.workspaceID?.isEmpty == false && recordedDebug?.current() == true
            && exactUTF8(buffer.inlineMarkup.source, buffer.text) && buffer.inlineMarkup.debug.recordedConstructions.contains(line)
    }
    private func embeddedAnchors(_ line: Int) -> [Int] {
        guard exactUTF8(buffer.inlineMarkup.source,buffer.text) else { return [] }
        return buffer.inlineMarkup.diagrams.filter { $0.line == line }.map(\.id)
            + buffer.inlineMarkup.equations.filter { $0.line == line }.map(\.id)
            + inlineTables(buffer.inlineMarkup).filter { table in buffer.inlineMarkup.lineStarts.lastIndex(where: { $0 <= table.range.location }) == line }.map(\.id)
    }
    private func canCommentOnPatch(_ line: Int) -> Bool {
        recordedRunEngine != nil && recordedDebug?.side == .assistant && recordedDebug?.current() == true
            && buffer.inlineMarkup.debug.recordedPatchLines[line] != nil
            && exactUTF8(buffer.inlineMarkup.source, buffer.text)
    }

    private func patchComment(_ line: Int) -> NativePatchCommentAnchor? {
        guard canCommentOnPatch(line), let recordedDebug,
              let opening = buffer.inlineMarkup.debug.recordedPatchLines[line] else { return nil }
        let snapshot = buffer.text
        let rows = snapshot.components(separatedBy: "\n")
        guard rows.indices.contains(line) else { return nil }
        return NativePatchCommentAnchor(path: buffer.file.path, hash: nativeSourceHash(snapshot),
                                        line: line + 1, opening: opening + 1, turn: recordedDebug.turn, quote: rows[line]) {
            recordedDebug.current() && exactUTF8(buffer.text, snapshot)
        }
    }

    private func actions(_ line: Int,testLines:Set<Int>,coverageLines:[Int:CoverageProjectedLine]) -> [String] {
        var result: [String] = []
        if recordedDebug == nil, buffer.canAuthorElements, let reading = buffer.elementLines,
           reading.revision == buffer.sourceEditIdentity, reading.lines.contains(line) { result.append("edit-element") }
        if let row=coverageLines[line]{result.append("coverage:"+row.state)}
        if testLines.contains(line){result.append("tests")}
        if let comparison = buffer.gitComparison, exactUTF8(comparison.path, buffer.file.path),
           line == 0 || buffer.gitChanges?[line] != nil { result.append("git-change") }
        result += embeddedAnchors(line).map { "source:\($0)" }
        let fileLaunch = recordedDebug == nil && exactUTF8(buffer.inlineMarkup.source, buffer.text) && buffer.inlineMarkup.debug.launch[line] != nil
        if (showsSession && line == 0) || fileLaunch { result.append("debug") }
        if recordedFences[line] != nil {
            if recordedRunEngine != nil, recordedDebug?.workspace?.isEmpty == false { result.append("recorded-run") }
            if !debug.active { result.append("recorded-debug") }
        }
        if canReviewRecordedPatch(line) { result.append("recorded-review") }
        if canCommentOnPatch(line) { result.append("patch-comment") }
        if canInspectRecordedConstruction(line) { result.append("recorded-construction") }
        if exactUTF8(buffer.inlineMarkup.source,buffer.text) && buffer.inlineMarkup.equations.contains(where: { $0.line == line }) { result.append("equation") }
        if !buffer.receipts(on: line).isEmpty { result.append("output") }
        if paused == line { result += ["continue", "over", "in", "out", "stop", "inspect"] }
        return result
    }
    private func symbol(_ action: String) -> String {
        if action.hasPrefix("coverage:"){return action=="coverage:covered" ? "circle.fill":action=="coverage:uncovered" ? "circle":"circle.lefthalf.filled"}
        if action.hasPrefix("source:") { return "chevron.left.forwardslash.chevron.right" }
        if action == "edit-element" { return "pencil" }
        switch action { case "tests":return "checklist";case "git-change":return "plus.forwardslash.minus";case "patch-comment":return "text.bubble";case "debug", "recorded-debug", "continue": return "play"; case "recorded-run": return "terminal"; case "recorded-review", "recorded-construction": return "doc.text.magnifyingglass"; case "equation": return "function"; case "output": return "text.alignleft"; case "stop": return "stop.fill"; default: return "eye" }
    }
    private func actionLabel(_ action:String,line:Int)->String {
        if action.hasPrefix("coverage:"),let row=buffer.coverageRows().first(where:{$0.line==line}) {return "Coverage \(row.state) · \(row.count.map {"count \($0)"} ?? "count unavailable") · \(row.origins.count) measured location(s)"}
        if action == "git-change" { return "Inspect Git comparison" }
        if action == "edit-element" { return "Edit element" }
        if action == "patch-comment" { return "Comment on patch line" }
        if action == "over" { return "Step over" }
        if action == "in" { return "Step in" }
        if action == "out" { return "Step out" }
        if action=="debug",showsSession {return "Inspect debugger"}
        if action == "recorded-run" { return "Run recorded \(recordedFences[line]?.language ?? "code") fence" }
        if action == "recorded-debug" { return "Debug recorded \(recordedFences[line]?.language ?? "code") fence" }
        if action == "recorded-review" { return "Review recorded patch" }
        if action == "recorded-construction" { return "Inspect recorded construction patch" }
        return action == "tests" ? "Inspect source tests" : action.hasPrefix("source:") ? "Toggle source text" : action.capitalized
    }
    @ViewBuilder private func icon(_ action: String, line: Int, fromOverflow: Bool = false) -> some View {
        let cannotStartDebug = action == "debug" && !debug.hasSessionReading && (buffer.dirty || buffer.saving)
        Button {
            if action == "tests" || action.hasPrefix("coverage:") {
                let reading = LineReading(action: action, line: line, path: buffer.file.path,
                                          revision: buffer.sourceEditIdentity)
#if os(iOS)
                if fromOverflow {
                    pendingLineReading = reading
                    opened = nil
                    return
                }
#endif
                opened = nil
                showLineReading(reading)
                return
            }
            if action != "recorded-run" && action != "recorded-construction" { opened = nil }
            if action.hasPrefix("source:"), let anchor = Int(action.dropFirst(7)) { buffer.sourceTable = buffer.sourceTable == anchor ? nil : anchor; return }
            switch action {
            case "edit-element": buffer.openElementForm(line: line)
            case "git-change":
                gitLine = line
            case "patch-comment":
                if canCommentOnPatch(line) { commentLine = line }
            case "debug":
                if showsSession {inspect(line)}
                else { inspect(line) }
            case "recorded-run":
                guard let recordedRunEngine, let recordedDebug, let fence = recordedFences[line] else { return }
                recordedConstruction.close()
                Task { await recordedRun.start(buffer, source: recordedDebug, fence: fence, engine: recordedRunEngine) }
            case "recorded-debug":
                guard let recordedDebug, let fence = recordedFences[line] else { return }
                Task { await debug.prepareRecordedFence(buffer, source: recordedDebug, fence: fence); if !debug.active { inspect(line) } }
            case "recorded-review":
                if canReviewRecordedPatch(line) { recordedPatchReview?(line) }
            case "recorded-construction":
                guard canInspectRecordedConstruction(line), let recordedRunEngine, let recordedDebug else { return }
                recordedRun.close()
                Task { await recordedConstruction.inspect(buffer, source: recordedDebug, line: line, engine: recordedRunEngine) }
            case "continue", "over", "in", "out": debug.step(action)
            case "stop": debug.stop()
            default: inspect(line)
            }
        } label: {
            Group {
                switch action {
                case "over": Text("↓")
                case "in": Text("⇥")
                case "out": Text("↤")
                default: Image(systemName: symbol(action))
                }
            }
            .frame(width: 20, height: 18)
        }
        .foregroundStyle(action=="coverage:covered" ? Color.green:action=="coverage:uncovered" ? Color.red:action=="coverage:mixed" ? Color.orange:Color.primary)
        .buttonStyle(SimpleChromeButtonStyle())
        .disabled(reason: cannotStartDebug ? "Save the current source before starting the debugger." : nil,
                  fallbackHelp: actionLabel(action,line:line)).accessibilityLabel("\(actionLabel(action,line:line)) on line \(line + 1)")
    }
    private var runOutput: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack {
                Text("Recorded fence Run").font(.headline)
                Spacer()
                Button("Close") { recordedRun.close(); opened = nil }
            }
            if let terminal = recordedRun.terminal {
                Text(terminal.exit_code.map { "Exit code \($0)" } ?? terminal.state).font(.caption)
            }
            if !recordedRun.notice.isEmpty {
                Text(recordedRun.notice).font(.caption).textSelection(.enabled)
            }
            if !recordedRun.output.isEmpty {
                NativeInlineTerminal(text: recordedRun.output).frame(height: 150)
            }
        }.padding(10).frame(width: 300)
            .accessibilityElement(children: .contain).accessibilityLabel("Recorded fence terminal")
            .task(id: recordedRun.terminal?.id) {
                guard let recordedRunEngine else { return }
                while !Task.isCancelled, await recordedRun.refresh(recordedRunEngine) {
                    do { try await Task.sleep(for: .milliseconds(400)) } catch { return }
                }
            }
    }

    private var constructionReading: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack {
                Text("Recorded construction patch").font(.headline)
                Spacer()
                Button("Close") { recordedConstruction.close(); opened = nil }
            }
            if !recordedConstruction.notice.isEmpty {
                Text(recordedConstruction.notice).font(.caption).textSelection(.enabled)
            }
            if let receipt = recordedConstruction.receipt {
                Text("\(receipt.document) · \(receipt.operation_id)").font(.caption).textSelection(.enabled)
                ScrollView {
                    VStack(alignment: .leading, spacing: 8) {
                        ForEach(receipt.changes) { change in
                            DisclosureGroup(change.path) {
                                constructionEntry("Before", change.before)
                                constructionEntry("After", change.after)
                            }
                        }
                    }.frame(maxWidth: .infinity, alignment: .leading)
                }
                // AppKit's popover sizes to its content. Give the scroll view
                // a minimum height so asynchronously loaded rows stay visible.
                .frame(minHeight: 150, maxHeight: 320)
            }
        }.padding(10).frame(width: 330)
            .accessibilityElement(children: .contain).accessibilityLabel("Recorded construction reading")
    }

    private func constructionEntry(_ label: String, _ entry: NativeRecordedConstructionEntry?) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(label).font(.caption.bold())
            if let entry {
                Text("\(entry.kind) · \(entry.bytes) bytes\(entry.executable ? " · executable" : "")").font(.caption)
                if let text = entry.text {
                    Text(text).font(.system(.caption, design: .monospaced))
                } else {
                    Text(entry.text_status == "non_utf8" ? "Non-UTF-8 output" : "Output exceeds the preview limit").font(.caption)
                    Text(entry.sha256).font(.caption2)
                }
            } else {
                Text("Absent").font(.caption)
            }
        }.frame(maxWidth: .infinity, alignment: .leading).textSelection(.enabled)
    }

    var body: some View {
        GeometryReader { geometry in
            let coverageLines=Dictionary(uniqueKeysWithValues:buffer.coverageRows().map {($0.line,$0)})
            let testLines=Set(buffer.sourceTestRows().map(\.line))
            let rows = buffer.visibleDebugLines.filter { $0.y >= 0 && $0.y < geometry.size.height && !actions($0.line,testLines:testLines,coverageLines:coverageLines).isEmpty }
            let columns = 1
            ZStack(alignment: .topLeading) {
                ForEach(Array(rows.enumerated()), id: \.element.line) { index, row in
                    let items = actions(row.line,testLines:testLines,coverageLines:coverageLines)
                    let room = (index + 1 < rows.count ? rows[index + 1].y : geometry.size.height) - row.y
                    let capacity = max(1, Int(room / 20)) * columns
                    let visible = items.count <= capacity ? items.count : max(0, capacity - 1)
                    ForEach(0..<visible, id: \.self) { slot in
                        icon(items[slot], line: row.line).position(x: CGFloat(slot % columns) * 22 + 11, y: row.y + 9 + CGFloat(slot / columns) * 20)
                    }
                    let overflowSlot = visible < items.count ? visible : 0
                    if visible < items.count {
                        Button {
                            opened = opened == row.line ? nil : row.line
                        } label: {
                            Image(systemName: "ellipsis").font(.system(size: 12, weight: .semibold))
                                .foregroundStyle(Color.primary).frame(width: 22, height: 18)
                        }
                            .buttonStyle(SimpleChromeButtonStyle()).help("More line actions")
                            .accessibilityLabel("More gutter icons on line \(row.line + 1)")
                            .position(x: CGFloat(visible % columns) * 22 + 11, y: row.y + 9 + CGFloat(visible / columns) * 20)
                    }
                    Color.clear.frame(width: 22, height: 18).allowsHitTesting(false).accessibilityHidden(true)
                        .popover(isPresented: Binding(get: { opened == row.line || recordedRun.line == row.line || recordedConstruction.line == row.line || gitLine == row.line || commentLine == row.line }, set: { presented in
                            if !presented {
                                if opened == row.line { opened = nil }
                                if gitLine == row.line { gitLine = nil }
                                if commentLine == row.line { commentLine = nil }
                                if recordedRun.line == row.line { recordedRun.close() }
                                if recordedConstruction.line == row.line { recordedConstruction.close() }
                            }
                        })) {
                            if commentLine == row.line, let anchor = patchComment(row.line), let recordedRunEngine {
                                NativePatchCommentView(anchor: anchor, engine: recordedRunEngine)
                                    .id(anchor.hash + ":" + String(anchor.line))
                                    .presentationCompactAdaptation(.popover)
                            } else if gitLine == row.line, let comparison = buffer.gitComparison {
                                GitEditorComparisonLine(buffer: buffer, comparison: comparison, line: row.line)
                                    .presentationCompactAdaptation(.popover)
                            } else if recordedRun.line == row.line {
                                runOutput.presentationCompactAdaptation(.popover)
                            } else if recordedConstruction.line == row.line {
                                constructionReading.presentationCompactAdaptation(.popover)
                            } else {
                                LazyVGrid(columns: Array(repeating: GridItem(.fixed(22), spacing: 2), count: max(1, min(4, items.count - visible))), spacing: 2) {
                                    ForEach(Array(items.dropFirst(visible)), id: \.self) { action in
                                        icon(action, line: row.line, fromOverflow: true)
                                    }
                                }.padding(6).frame(width: CGFloat(min(4, items.count - visible)) * 24 + 12, height: CGFloat((items.count - visible + 3) / 4) * 20 + 12)
                                    .accessibilityElement(children: .contain).accessibilityLabel("More actions on line \(row.line + 1)")
                                    .presentationCompactAdaptation(.popover)
                                    .onDisappear {
                                        guard let reading = pendingLineReading, reading.line == row.line else { return }
                                        pendingLineReading = nil
                                        showLineReading(reading)
                                    }
                            }
                        }
                        .position(x: 11, y: row.y + 9 + CGFloat(overflowSlot) * 20)
                }
            }
        }.background(.quaternary.opacity(0.2))
        .popover(isPresented: Binding(get: { details != nil }, set: { if !$0 { details = nil } })) {
            if let details {
                NativeDebugInspector(buffer: buffer, debug: debug, recordedDebug: recordedDebug, line: details, showRunTerminal: showRunTerminal)
            }
        }.onChange(of:buffer.debugInspection){_,_ in if showsSession{inspect(0)}}
        .onChange(of: recordedDebug?.current() == true) { _, current in
            if !current { recordedRun.close(); recordedConstruction.close(); opened = nil; commentLine = nil }
        }
        .onChange(of: buffer.gitComparison?.id) { _, _ in gitLine = nil }
        .onDisappear { recordedRun.close(); recordedConstruction.close() }
        .task(id: buffer.base) { await buffer.refreshReceipts() }
    }
}

struct NativeDebugInspector: View, Identifiable {
    let id = UUID()
    let buffer: DocumentBuffer
    let debug: NativeDebugger
    let recordedDebug: NativeRecordedMessageSource?
    let line: Int
    var showRunTerminal: ((String) -> Void)? = nil
    @State private var adapterInspection:UUID?
    @State private var expression = ""
    @State private var condition = ""
    @State private var breakpointPage = 0

    var sourceCurrent: Bool { recordedDebug?.current() ?? true }
    private var paused: Int? { debug.sourceMatches(buffer) ? debug.line(in: buffer.file.path) : nil }
    private var showsSession: Bool { debug.hasSessionReading && (recordedDebug == nil || exactUTF8(debug.owner, buffer.file.path)) }
    private var recordedFences: [Int: RecordedDebugFence] { recordedDebugFences(buffer, source: recordedDebug) }

    var body: some View {
        ScrollView { controls.padding(10) }
            .frame(width: 330).frame(maxHeight: 460)
            .onAppear { adapterInspection = debug.adapterSources.open() }
            .onDisappear {
                if let token = adapterInspection { debug.adapterSources.close(token) }
                adapterInspection = nil
            }
    }

    @ViewBuilder private var threadControls:some View {
        let snapshot=debug.threadSnapshot
        VStack(alignment:.leading,spacing:4) {
            HStack{Button(debug.threadBusy ? "Reading threads…":"Read threads"){debug.readThreads()}.disabled(reason: !debug.canReadThreads ? "Pause at a current debugger stop to read its threads." : nil);Button("Refresh paused state"){debug.refreshPausedState()}.disabled(reason: debug.threadBusy ? "Wait for the debugger thread request to finish." : debug.refreshingPause ? "A debugger stop refresh is in progress." : nil)}
            if !debug.threadNotice.isEmpty {Text(debug.threadNotice).font(.caption).textSelection(.enabled)}
            if let selected=debug.selectedThread {Text("Reported thread \(selected)").font(.caption)}
            ForEach(debug.visibleThreads){thread in
                Button("\(thread.id==debug.selectedThread ? "●":"○") \(thread.name.isEmpty ? "Thread":thread.name) (\(thread.id))"){debug.selectThread(thread.id,snapshot:snapshot)}
                    .disabled(reason: !debug.canSelectThreads ? "Wait for the current debugger thread reading." : nil).accessibilityAddTraits(thread.id==debug.selectedThread ? .isSelected:[])
                    .accessibilityHint("Requests this reported thread’s stopped stack without changing source.")
            }
            if debug.threadPages>1 {HStack{Button("Previous threads"){debug.threadPage=max(0,debug.threadPage-1)}.disabled(reason: debug.threadPage==0 ? "You’re on the first page." : debug.threadBusy ? "Wait for the debugger thread request to finish." : nil);Text("Page \(debug.threadPage+1) of \(debug.threadPages)").font(.caption);Button("Next threads"){debug.threadPage=min(debug.threadPages-1,debug.threadPage+1)}.disabled(reason: debug.threadPage>=debug.threadPages-1 ? "You’re on the last page." : debug.threadBusy ? "Wait for the debugger thread request to finish." : nil)}}
        }
    }

    @ViewBuilder private var controls: some View {
        let ordinaryLaunch = recordedDebug == nil && inlineDebugPath(buffer.text, path: buffer.file.path, line: line, launch: true) != nil
        let code = recordedDebug == nil ? inlineDebugPath(buffer.text, path: buffer.file.path, line: line) != nil
            : recordedFences.values.contains { $0.contains(line) }
        VStack(alignment: .leading, spacing: 8) {
            Text("Source line \(line + 1) · \(debug.status)").font(.caption)
            if showsSession {Text("Runtime session: \(debug.owner)").font(.caption).textSelection(.enabled)}
            if showsSession, debug.status=="paused",paused != line {Text("The selected runtime frame has no current stop marker on this source line. Thread and frame controls still refer to the paused runtime.").font(.caption)}
            if exactUTF8(buffer.inlineMarkup.source,buffer.text) {
                ForEach(Array(buffer.inlineMarkup.equations.filter { $0.line == line }.prefix(4))) { equation in
                    InlineEquationControl(equation: equation, source: buffer.text, buffer: buffer).id("math:\(equation.id):\(buffer.text)")
                }
                if buffer.inlineMarkup.cells.filter({ $0.line == line }).count > 32 || buffer.inlineMarkup.equations.filter({ $0.line == line }).count > 4 { Text("More items remain editable in source.").font(.caption) }

            }
            ForEach(buffer.receipts(on: line)) { receipt in
                Text("Recorded output · " + (receipt.status ?? "unrecorded")).font(.caption)
                Text(receipt.command ?? "").font(.system(.caption, design: .monospaced)).textSelection(.enabled)
                let output = receipt.recorded_output ?? (receipt.transcript ?? []).filter { $0.kind == "out" || $0.kind == "err" }.compactMap(\.data).joined()
                NativeInlineTerminal(text: String(output.prefix(50_000))).frame(height: 180)
                if output.count > 50_000 { Text("Output truncated in this control.").font(.caption) }
            }
            if !buffer.receiptNotice.isEmpty { Text(buffer.receiptNotice).font(.caption) }
            if !buffer.receipts(on: line).isEmpty {
                Button("Refresh recorded output", systemImage: "arrow.clockwise") { Task { await buffer.refreshReceipts() } }
            }
            if showsSession || ordinaryLaunch || recordedFences[line] != nil || code {
            if buffer.dirty { Text("Save edited text before launching. Values belong to the source that ran.").font(.caption) }
            LazyVGrid(columns: [GridItem(.adaptive(minimum: 86))], spacing: 5) {
                if ordinaryLaunch {
                    Button("Debug",systemImage:"play"){Task{await debug.prepare(buffer,line:line)}}.disabled(reason: debug.active ? "Stop the current debugger session first." : buffer.dirty ? "Save the current source before launching." : buffer.saving ? "Wait for this source to finish saving." : nil)
                    Button("Launch profiles",systemImage:"list.bullet"){Task{await debug.prepare(buffer,line:line,chooseProfile:true)}}.disabled(reason: debug.active ? "Stop the current debugger session first." : buffer.dirty ? "Save the current source before launching." : buffer.saving ? "Wait for this source to finish saving." : nil)
                }
                if let recordedDebug, let fence = recordedFences[line] {
                    Button("Debug recorded fence", systemImage: "play") { Task { await debug.prepareRecordedFence(buffer, source: recordedDebug, fence: fence) } }.disabled(reason: debug.active ? "Stop the current debugger session first." : nil)
                }
                if code { Button("Breakpoint", systemImage: "circle.fill") { debug.toggle(buffer, line: line) } }
                if showsSession, debug.status == "paused" {
                    Button("Continue", systemImage: "play.fill") { debug.step("continue") }.disabled(reason: !debug.canResume ? "Wait for a confirmed paused debugger session." : nil); Button("Step over", systemImage: "arrow.turn.down.right") { debug.step("over") }.disabled(reason: !debug.canInspectFrame ? "Select a frame at the current debugger stop." : nil)
                    Button("Step in", systemImage: "arrow.down") { debug.step("in") }.disabled(reason: !debug.canInspectFrame ? "Select a frame at the current debugger stop." : nil); Button("Step out", systemImage: "arrow.up") { debug.step("out") }.disabled(reason: !debug.canInspectFrame ? "Select a frame at the current debugger stop." : nil)
                    if debug.capabilities["step_back"] as? Bool == true { Button("Step back", systemImage: "arrow.uturn.backward") { debug.step("back") }.disabled(reason: !debug.canInspectFrame ? "Select a frame at the current debugger stop." : nil) }
                }
                if showsSession, debug.active { Button("Stop", systemImage: "stop.fill") { debug.stop() } }
            }
            if exactUTF8(debug.owner,buffer.file.path) { ForEach(debug.targets) { target in Button("Debug file \(target.path)") { Task { await debug.prepare(buffer, program: target.path) } } } }
            ForEach(debug.profiles,id: \.exactID) { profile in
                Button {debug.launch(profile)} label:{VStack(alignment:.leading){Text(profile.actionLabel);if let notice=profile.attachNotice{Text(notice).font(.caption).fixedSize(horizontal:false,vertical:true)}}}.accessibilityLabel(profile.actionLabel+" · "+(profile.attachNotice ?? ""))
            }
            if let showRunTerminal {
                ForEach(debug.commandProfiles, id: \.exactID) { profile in
                    Button("Run \(profile.name)") { Task { await debug.runProfile(profile, buffer: buffer, terminal: showRunTerminal) } }
                        .disabled(reason: debug.runningProfile ? "A launch profile is already running." : debug.active ? "Stop the current debugger session first." : buffer.dirty ? "Save the current source before launching." : buffer.saving ? "Wait for this source to finish saving." : nil)
                }
            }
            if let notice=debug.attachReading{Text(notice).font(.caption).textSelection(.enabled)}
            if debug.offer != nil { Button("Install debugger", systemImage: "arrow.down.to.line") { Task { await debug.install() } } }
            if !debug.message.isEmpty { Text(debug.message).font(.caption).textSelection(.enabled) }
            ForEach(Array(debug.catalogNotices.enumerated()),id: \.offset){_,notice in Text("Launch catalog: "+notice).font(.caption).textSelection(.enabled)}
            if debug.pauseUnconfirmed {Text(debug.stackNotice).font(.caption).textSelection(.enabled);ScrollView{VStack(alignment:.leading){ForEach(Array(debug.frames.enumerated()),id: \.offset){_,frame in Text("Historical frame: \(frame["name"] as? String ?? "Frame")").font(.caption).textSelection(.enabled)}}}.frame(maxHeight:100)}
            if !debug.breakpointNotice.isEmpty {Text(debug.breakpointNotice).font(.caption).textSelection(.enabled)}
            let retained=debug.retainedBreakpointReading(buffer.file.path),readingPage=max(0,min(breakpointPage,max(0,(retained.count-1)/64)))
            if !retained.isEmpty {Text("Retained breakpoint intentions · \(retained.count)").font(.caption);Text("Readonly wishes for earlier source; no coordinates are applied. Latest 4096 historical wishes per path remain in memory.").font(.caption)
                if retained.count>64 {HStack{Button("Previous intentions"){breakpointPage=readingPage-1}.disabled(reason: readingPage==0 ? "You’re on the first page." : nil);Text("\(readingPage+1) / \((retained.count+63)/64)").font(.caption);Button("Next intentions"){breakpointPage=readingPage+1}.disabled(reason: (readingPage+1)*64>=retained.count ? "You’re on the last page." : nil)}}
                ForEach(Array(retained.dropFirst(readingPage*64).prefix(64).enumerated()),id: \.offset){_,reading in Text(reading).font(.caption).textSelection(.enabled)}
            }
            if showsSession, code, debug.capabilities["conditional_breakpoints"] as? Bool == true {
                TextField("Breakpoint condition", text: $condition)
                Button("Apply condition", systemImage: "checkmark") { debug.toggle(buffer, line: line, condition: condition) }
            }
            NativeAdapterSourceView(debug:debug,inspection:adapterInspection)
            if showsSession, debug.status == "paused" {
                threadControls
                HStack {Button("Run to this source line"){debug.moveToSource(buffer,line:line,how:"run_to")}.disabled(reason: !debug.canMoveToSource(buffer,how:"run_to") ? "Select a line in saved source at a current stop with Run to Line support." : nil);Button("Move instruction pointer here"){debug.moveToSource(buffer,line:line,how:"jump")}.disabled(reason: !debug.canMoveToSource(buffer,how:"jump") ? "Select a line in saved source at a current stop with instruction movement support." : nil)}
                Text("Position movement requires unchanged captured primary source; instruction-pointer movement also requires adapter goto-target support.").font(.caption)
                Text("Source positions compare captured launch/construction text; loaded runtime inputs are not frozen or proven by those captures.").font(.caption)
                let frameSnapshot=debug.frameSnapshot
                if !debug.stackNotice.isEmpty {Text(debug.stackNotice).font(.caption).textSelection(.enabled)}
                if !debug.sourceReferenceNotice.isEmpty {Text(debug.sourceReferenceNotice).font(.caption).textSelection(.enabled)}
                Button(debug.frameBusy ? "Reading frame variables…":"Read selected-frame variables"){debug.readFrameLocals()}.disabled(reason: !debug.canReadFrameLocals ? "Select a current paused frame to read its local variables." : nil)
                if !debug.frameNotice.isEmpty {Text(debug.frameNotice).font(.caption).textSelection(.enabled)}
                TextField("Expression", text: $expression)
                HStack { Button("Evaluate", systemImage: "equal") { debug.evaluate(expression) }.disabled(reason: !debug.canEvaluate ? "Select a frame at the current debugger stop." : expression.isEmpty ? "Enter an expression to evaluate." : nil); Button("Watch", systemImage: "eye") { debug.evaluate(expression, watch: true) }.disabled(reason: !debug.canEvaluate ? "Select a frame at the current debugger stop." : expression.isEmpty ? "Enter an expression to evaluate." : nil) }
                if !debug.evaluationNotice.isEmpty {Text(debug.evaluationNotice).font(.caption).textSelection(.enabled)}
                ScrollView {
                    VStack(alignment: .leading, spacing: 4) {
                        ForEach(Array(debug.frames.enumerated()), id: \.offset) { _, frame in Button(frame["name"] as? String ?? "Frame") { debug.select(frame,snapshot:frameSnapshot) }.disabled(reason: !debug.canInspectFrame ? "Select a frame at the current debugger stop." : nil) }
                        ForEach(debug.visibleVariableRows) { row in
                            VStack(alignment:.leading,spacing:2) {
                                HStack {if row.reference>0 {Button(row.expanded ? "−":"+"){debug.toggleVariable(row.id)}.accessibilityLabel("\(row.expanded ? "Collapse":"Expand") variable \(row.name)").disabled(reason: !debug.canInspectFrame ? "Select a frame at the current debugger stop." : nil)};Text("\(row.name) = \(row.value)").textSelection(.enabled)}
                                if row.pending {Text("Reading child variables…").font(.caption)}
                                if !row.notice.isEmpty {Text(row.notice).font(.caption);if row.retryable {Button("Retry expansion"){debug.readChildren(row.id)}.accessibilityLabel("Retry expansion of \(row.name)").disabled(reason: !debug.canInspectFrame ? "Select a frame at the current debugger stop." : nil)}}
                            }.padding(.leading,CGFloat(row.depth)*10)
                        }
                        ForEach(debug.values.readings) { reading in HStack { Text("\(reading.expression) = \(reading.value)"); if debug.isWatched(reading.expression) { Button("×") { debug.removeWatch(reading.expression) }.accessibilityLabel("Remove watch \(reading.expression)") } } }
                    }
                }.frame(maxHeight: 180)
            }
            }
        }
    }
}
