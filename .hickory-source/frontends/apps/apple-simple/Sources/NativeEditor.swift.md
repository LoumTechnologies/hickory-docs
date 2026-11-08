<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Move native text views without rebuilding their buffers

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

When an empty pane closes, the surviving editor can move to a new position in the SwiftUI tree. Keep its native editor cached for selection, scroll and undo, but give each representable a fresh host. Reparent the editor inside that host; reusing the representable root lets teardown of the old tree location interfere with its new owner.

SwiftUI owns the surrounding workspace; AppKit and UIKit own editing. Cache native views by buffer identity so changing a tab parent does not reconstruct text, undo history, selection or scroll. Delegates update only that buffer.

The minimap reserves forty units beside the text rather than overlaying readable lines. Its navigation changes scroll, never saved bytes. Language controls float over the editor and do not reflow it. Apple capabilities use the real backend LSP channel and retain their own snapshot guards. Native semantic styling does not replace text, preserving the platform undo and selection model.

Fold state belongs to the cached native editor. Discover indentation regions with a stack and UTF-16 ranges, matching TextKit's character indices. The layout delegate marks folded glyphs null and suppresses their control-character advancement, retaining the original text storage for saves, selection, language services and undo. Hidden rows are omitted from the measured rails. Before an edit, retire the folds; refuse the first edit if it would touch hidden bytes so a collapsed selection cannot silently erase unseen code. Reveals unfold first. UIKit explicitly uses TextKit 1 so the same delegate mechanism is available on both Apple platforms.

The line-action popover shares the debugger slot and offers Fold/Unfold without adding a new column. Nested ranges remain independent, and folding is allowed on read-only files because it writes no source. This is indentation-based folding, not a claim of native language-server fold support.

<ui:copy id="simple-7fad175521-0">
import SwiftUI
#if os(macOS)
import AppKit
#else
import UIKit
#endif
struct SimpleFold: Equatable {
    var line: Int
    var range: NSRange
}
func simpleFoldRanges(_ source: String) -> [SimpleFold] {
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
    return result.sorted { $0.range.location < $1.range.location }
}
#if os(macOS)
private typealias FoldFont = NSFont
#else
private typealias FoldFont = UIFont
#endif
@MainActor private final class FoldLayout: NSObject, @preconcurrency NSLayoutManagerDelegate {
    var ranges: [NSRange] = []
    func contains(_ offset: Int) -> Bool { ranges.contains { NSLocationInRange(offset, $0) } }
    func layoutManager(_ manager: NSLayoutManager, shouldGenerateGlyphs glyphs: UnsafePointer<CGGlyph>, properties: UnsafePointer<NSLayoutManager.GlyphProperty>, characterIndexes: UnsafePointer<Int>, font: FoldFont, forGlyphRange range: NSRange) -> Int {
        guard !ranges.isEmpty else { return 0 }
        var props = Array(UnsafeBufferPointer(start: properties, count: range.length))
        for i in 0..<range.length where contains(characterIndexes[i]) { props[i] = .null }
        props.withUnsafeBufferPointer { manager.setGlyphs(glyphs, properties: $0.baseAddress!, characterIndexes: characterIndexes, font: font, forGlyphRange: range) }
        return range.length
    }
    func layoutManager(_ manager: NSLayoutManager, shouldUse action: NSLayoutManager.ControlCharacterAction, forControlCharacterAt index: Int) -> NSLayoutManager.ControlCharacterAction {
        contains(index) ? .zeroAdvancement : action
    }
}
#if os(macOS)
@MainActor final class MacEditor: NSObject, NSTextViewDelegate {
    let scroll = NSScrollView(frame: NSRect(x: 0, y: 0, width: 600, height: 400))
    let text = NSTextView(frame: NSRect(x: 0, y: 0, width: 600, height: 400))
    let buffer: DocumentBuffer
    private let folding = FoldLayout()
    private var observer: NSObjectProtocol?
    init(_ buffer: DocumentBuffer) {
        self.buffer = buffer
        super.init()
        text.isRichText = false; text.isAutomaticQuoteSubstitutionEnabled = false
        text.isAutomaticDashSubstitutionEnabled = false; text.isAutomaticTextReplacementEnabled = false
        text.isAutomaticSpellingCorrectionEnabled = false; text.isContinuousSpellCheckingEnabled = false
        text.font = .monospacedSystemFont(ofSize: 13, weight: .regular)
        text.isVerticallyResizable = true; text.isHorizontallyResizable = false
        text.autoresizingMask = [.width]; text.textContainer?.widthTracksTextView = true
        text.textContainerInset = NSSize(width: 8, height: 8)
        text.string = buffer.text; text.isEditable = !buffer.readOnly; text.allowsUndo = true
        text.setAccessibilityLabel(buffer.file.path); text.delegate = self
        scroll.hasVerticalScroller = true; scroll.documentView = text
        buffer.navigate = { [weak self] fraction in
            guard let self else { return }
            self.unfoldAll()
            let source = self.text.string as NSString
            let offset = min(source.length, max(0, Int(Double(source.length) * fraction)))
            self.text.scrollRangeToVisible(source.lineRange(for: NSRange(location: offset, length: 0)))
        }
        buffer.renderLanguage = { [weak self] tokens, source in self?.paint(tokens, source: source) }
        buffer.applyLanguageCompletion = { [weak self] range, insert, source, selection in
            guard let self, !self.buffer.readOnly, !self.text.hasMarkedText(), self.text.string == source,
                  self.text.selectedRange() == selection, NSMaxRange(range) <= (source as NSString).length,
                  self.text.shouldChangeText(in: range, replacementString: insert) else { return false }
            self.text.textStorage?.replaceCharacters(in: range, with: insert)
            self.text.setSelectedRange(NSRange(location: range.location + (insert as NSString).length, length: 0))
            self.text.didChangeText(); return true
        }
        buffer.revealDebugLine = { [weak self] line in
            guard let self, let offset = languageOffset(self.buffer.text, ["line": line, "character": 0]) else { return }
            self.unfoldAll(); self.text.scrollRangeToVisible(NSRange(location: offset, length: 0))
        }
        scroll.contentView.postsBoundsChangedNotifications = true
        observer = NotificationCenter.default.addObserver(forName: NSView.boundsDidChangeNotification, object: scroll.contentView, queue: .main) { [weak self] _ in Task { @MainActor in self?.measureDebugLines() } }
        text.layoutManager?.delegate = folding
        buffer.toggleFold = { [weak self] line in self?.toggleFold(line) }
        paint(buffer.language?.tokens ?? [], source: buffer.text)
        measureDebugLines()
    }
    func toggleFold(_ line: Int) {
        guard let header = simpleFoldRanges(text.string).first(where: { $0.line == line }) else { return }
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
        let hidden = folding.ranges.contains { NSIntersectionRange($0, affectedCharRange).length > 0 || NSLocationInRange(affectedCharRange.location, $0) }
        unfoldAll(); return !hidden
    }
    func textDidChange(_ notification: Notification) { unfoldAll(); buffer.selection = text.selectedRange(); buffer.text = text.string; measureDebugLines() }
    func textViewDidChangeSelection(_ notification: Notification) {
        if folding.ranges.contains(where: { text.selectedRange().location > $0.location && NSLocationInRange(text.selectedRange().location, $0) }) { unfoldAll() }
        buffer.selection = text.selectedRange()
    }
    func textView(_ textView: NSTextView, doCommandBy commandSelector: Selector) -> Bool {
        guard NSStringFromSelector(commandSelector) == "complete:" else { return false }
        Task { await buffer.language?.complete() }; return true
    }
    func measureDebugLines() {
        guard let manager = text.layoutManager, let container = text.textContainer else { return }
        manager.ensureLayout(for: container)
        let source = text.string as NSString, origin = text.textContainerOrigin
        let visible = text.visibleRect.offsetBy(dx: -origin.x, dy: -origin.y)
        let glyphs = manager.glyphRange(forBoundingRect: visible, in: container)
        var rows: [DebugLine] = []
        manager.enumerateLineFragments(forGlyphRange: glyphs) { rect, _, _, range, _ in
            let offset = manager.characterIndexForGlyph(at: range.location)
            guard !self.folding.contains(offset), offset == 0 || source.character(at: offset - 1) == 10 else { return }
            let line = source.substring(to: offset).filter { $0 == "\n" }.count
            rows.append(DebugLine(line: line, y: rect.minY + origin.y - self.scroll.contentView.bounds.minY, height: rect.height))
        }
        // TextKit exposes a separate fragment for an empty document or trailing newline.
        if source.length == 0 || source.character(at: source.length - 1) == 10 {
            let rect = manager.extraLineFragmentRect
            if rect.height > 0 {
                rows.append(DebugLine(line: source.substring(to: source.length).filter { $0 == "\n" }.count,
                    y: rect.minY + origin.y - scroll.contentView.bounds.minY, height: rect.height))
            }
        }
        if rows != buffer.visibleDebugLines { buffer.visibleDebugLines = rows }
    }
    private func paint(_ tokens: [NativeSemanticToken], source: String) {
        guard text.string == source, !text.hasMarkedText(), let storage = text.textStorage else { return }
        storage.beginEditing()
        storage.addAttribute(.foregroundColor, value: NSColor.textColor, range: NSRange(location: 0, length: storage.length))
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
        storage.endEditing(); text.typingAttributes[.foregroundColor] = NSColor.textColor
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
        if content.superview !== self { content.removeFromSuperview(); addSubview(content) }
        content.frame = bounds
    }
    override func viewDidMoveToWindow() { super.viewDidMoveToWindow(); attach() }
    override func layout() { super.layout(); attach(); Task { @MainActor in measure() } }
}
struct NativeEditor: NSViewRepresentable {
    let buffer: DocumentBuffer
    func sizeThatFits(_ proposal: ProposedViewSize, nsView: NSView, context: Context) -> CGSize? {
        CGSize(width: proposal.width ?? 600, height: proposal.height ?? 400)
    }
    func makeNSView(context: Context) -> NSView {
        let editor = EditorCache.views[buffer.id] ?? MacEditor(buffer)
        EditorCache.views[buffer.id] = editor
        return MacEditorHost(editor.scroll) { [weak editor] in editor?.measureDebugLines() }
    }
    func updateNSView(_ view: NSView, context: Context) {
        (view as? MacEditorHost)?.attach(); view.needsLayout = true
        Task { @MainActor in EditorCache.views[buffer.id]?.measureDebugLines() }
    }
}
#else
import UIKit
@MainActor private final class MobileEditor: NSObject, UITextViewDelegate {
    let text = UITextView(usingTextLayoutManager: false)
    private let folding = FoldLayout()
    let buffer: DocumentBuffer
    init(_ buffer: DocumentBuffer) {
        self.buffer = buffer
        super.init()
        text.font = .monospacedSystemFont(ofSize: 13, weight: .regular)
        text.autocorrectionType = .no; text.autocapitalizationType = .none
        text.smartQuotesType = .no; text.smartDashesType = .no; text.smartInsertDeleteType = .no
        text.text = buffer.text; text.isEditable = !buffer.readOnly; text.alwaysBounceVertical = true
        text.accessibilityLabel = buffer.file.path; text.delegate = self
        buffer.navigate = { [weak self] fraction in
            guard let self else { return }
            self.unfoldAll()
            let source = self.text.text as NSString
            let offset = min(source.length, max(0, Int(Double(source.length) * fraction)))
            self.text.scrollRangeToVisible(source.lineRange(for: NSRange(location: offset, length: 0)))
        }
        buffer.renderLanguage = { [weak self] tokens, source in self?.paint(tokens, source: source) }
        buffer.applyLanguageCompletion = { [weak self] range, insert, source, selection in
            guard let self, !self.buffer.readOnly, self.text.markedTextRange == nil, self.text.text == source,
                  self.text.selectedRange == selection,
                  let start = self.text.position(from: self.text.beginningOfDocument, offset: range.location),
                  let end = self.text.position(from: start, offset: range.length), let target = self.text.textRange(from: start, to: end) else { return false }
            self.text.replace(target, withText: insert)
            self.buffer.selection = self.text.selectedRange; self.buffer.text = self.text.text; return true
        }
        buffer.revealDebugLine = { [weak self] line in
            guard let self, let offset = languageOffset(self.buffer.text, ["line": line, "character": 0]) else { return }
            self.unfoldAll(); self.text.scrollRangeToVisible(NSRange(location: offset, length: 0))
        }
        text.layoutManager.delegate = folding
        buffer.toggleFold = { [weak self] line in self?.toggleFold(line) }
        paint(buffer.language?.tokens ?? [], source: buffer.text)
        measureDebugLines()
    }
    func toggleFold(_ line: Int) {
        guard let header = simpleFoldRanges(text.text ?? "").first(where: { $0.line == line }) else { return }
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
    func textView(_ textView: UITextView, shouldChangeTextIn range: NSRange, replacementText text: String) -> Bool {
        let hidden = folding.ranges.contains { NSIntersectionRange($0, range).length > 0 || NSLocationInRange(range.location, $0) }
        unfoldAll(); return !hidden
    }
    func textViewDidChange(_ textView: UITextView) { unfoldAll(); buffer.selection = textView.selectedRange; buffer.text = textView.text; measureDebugLines() }
    func textViewDidChangeSelection(_ textView: UITextView) {
        if folding.ranges.contains(where: { textView.selectedRange.location > $0.location && NSLocationInRange(textView.selectedRange.location, $0) }) { unfoldAll() }
        buffer.selection = textView.selectedRange
    }
    func scrollViewDidScroll(_ scrollView: UIScrollView) { measureDebugLines() }
    func measureDebugLines() {
        let manager = text.layoutManager, container = text.textContainer
        manager.ensureLayout(for: container)
        let source = (text.text ?? "") as NSString, inset = text.textContainerInset
        let visible = CGRect(x: 0, y: text.contentOffset.y - inset.top, width: text.bounds.width, height: text.bounds.height)
        let glyphs = manager.glyphRange(forBoundingRect: visible, in: container)
        var rows: [DebugLine] = []
        manager.enumerateLineFragments(forGlyphRange: glyphs) { rect, _, _, range, _ in
            let offset = manager.characterIndexForGlyph(at: range.location)
            guard !self.folding.contains(offset), offset == 0 || source.character(at: offset - 1) == 10 else { return }
            let line = source.substring(to: offset).filter { $0 == "\n" }.count
            rows.append(DebugLine(line: line, y: rect.minY + inset.top - self.text.contentOffset.y, height: rect.height))
        }
        if source.length == 0 || source.character(at: source.length - 1) == 10 {
            let rect = manager.extraLineFragmentRect
            if rect.height > 0 {
                rows.append(DebugLine(line: source.substring(to: source.length).filter { $0 == "\n" }.count,
                    y: rect.minY + inset.top - text.contentOffset.y, height: rect.height))
            }
        }
        if rows != buffer.visibleDebugLines { buffer.visibleDebugLines = rows }
    }
    private func paint(_ tokens: [NativeSemanticToken], source: String) {
        guard text.text == source, text.markedTextRange == nil else { return }
        let storage = text.textStorage
        storage.beginEditing()
        storage.addAttribute(.foregroundColor, value: UIColor.label, range: NSRange(location: 0, length: storage.length))
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
        storage.endEditing(); text.typingAttributes[.foregroundColor] = UIColor.label
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
    func sizeThatFits(_ proposal: ProposedViewSize, uiView: UIView, context: Context) -> CGSize? {
        CGSize(width: proposal.width ?? 600, height: proposal.height ?? 400)
    }
    func makeUIView(context: Context) -> UIView {
        let editor = EditorCache.views[buffer.id] ?? MobileEditor(buffer)
        EditorCache.views[buffer.id] = editor
        return MobileEditorHost(editor.text) { [weak editor] in editor?.measureDebugLines() }
    }
    func updateUIView(_ view: UIView, context: Context) {
        (view as? MobileEditorHost)?.attach(); view.setNeedsLayout()
        Task { @MainActor in EditorCache.views[buffer.id]?.measureDebugLines() }
    }
}
#endif

</ui:copy>

## Show real language results beside the retained editor

AppKit and UIKit keep their own text views above. The language session supplies semantic ranges only for the unchanged source snapshot. The shell reserves a narrow minimap and anchors a server completion popover beside it. Insertion calls back into the cached native view, checking source, selection, read-only state and marked-text composition before its normal edit API. This preserves native undo instead of replacing the entire buffer from SwiftUI.

The left number gutter consumes the same TextKit line rectangles as the right debugger rail. Only the first visual fragment of each logical line receives a number; wrapped continuations stay blank. TextKit's extra fragment accounts for an empty document and the line after a final newline. Drawing numbers beside the cached native view leaves selection, undo and scrolling in the editor. The gutter reserves at least three digits, expanding only when the document needs more.

<ui:copy id="simple-apple-language-preview">
struct LanguageEditor: View {
    var buffer: DocumentBuffer
    var debugger: NativeDebugger?
    var body: some View {
        HStack(spacing: 0) {
            GeometryReader { geometry in
                Canvas { context, size in
                    for row in buffer.visibleDebugLines where row.y + row.height > 0 && row.y < size.height {
                        context.draw(Text(String(row.line + 1)).font(.system(size: 13, design: .monospaced)).foregroundColor(.secondary),
                            at: CGPoint(x: size.width - 6, y: row.y + row.height / 2), anchor: .trailing)
                    }
                }.clipped().allowsHitTesting(false).accessibilityHidden(true)
            }.frame(width: CGFloat(max(3, String(buffer.text.filter { $0 == "\n" }.count + 1).count)) * 8 + 12)
            NativeEditor(buffer: buffer)
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
            if let debugger { NativeDebugRail(buffer: buffer, debug: debugger).frame(width: 24) }
        }.overlay(alignment: .topTrailing) {
            Button("⌁") {
                #if os(macOS)
                NSApp.activate()
                #endif
                Task { await buffer.language?.complete() }
            }.accessibilityLabel("Language completions")
                .help(buffer.language?.status ?? "Language services not connected")
                .padding(4).background(.regularMaterial).padding(.trailing, 42)
                .popover(isPresented: Binding(get: { buffer.language?.showingSuggestions ?? false }, set: { buffer.language?.showingSuggestions = $0 })) {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Language server completions").font(.headline)
                        Text(buffer.language?.status ?? "Language services not connected").font(.caption).textSelection(.enabled)
                        if let language = buffer.language {
                            ScrollView {
                                VStack(alignment: .leading, spacing: 4) {
                                    ForEach(language.suggestions) { item in
                                        Button { language.accept(item) } label: {
                                            VStack(alignment: .leading) { Text(item.label).font(.system(.body, design: .monospaced)); Text(item.detail).font(.caption).foregroundStyle(.secondary) }
                                        }.disabled(buffer.readOnly)
                                    }
                                }.frame(maxWidth: .infinity, alignment: .leading)
                            }.frame(height: 180)
                            Button("Retry language services") { language.start() }
                            Text("Python needs basedpyright installed in this workspace. Snippets and multi-file completion edits are not applied here.").font(.caption)
                        }
                    }.padding().frame(width: 330)
                }
        }
    }
}
</ui:copy>

## Anchor debug actions to the native text layout

The fixed right rail measures actual visible logical lines, including wrapped text and scroll offset. Each line owns one button; the paused line uses a ⋯ popover with an adaptive grid of controls, so a tall action stack cannot move the next line's icon. Values appear as an overlay beside the stopped line and never enter the editable string. The same SwiftUI rail serves macOS and iOS. Native editor instances, selection and undo are untouched when debug events arrive.

<ui:copy id="simple-apple-debug-rail">
struct NativeDebugRail: View {
    var buffer: DocumentBuffer
    var debug: NativeDebugger
    @State private var opened: Int?
    @State private var expression = ""
    @State private var condition = ""
    private var paused: Int? { debug.sourceMatches(buffer) ? debug.line(in: buffer.file.path) : nil }
    var body: some View {
        let foldHeaders = Set(simpleFoldRanges(buffer.text).map(\.line))
        GeometryReader { geometry in
            ZStack(alignment: .topLeading) {
                ForEach(buffer.visibleDebugLines.filter { $0.y >= 0 && $0.y < geometry.size.height }, id: \.line) { row in
                    let held = debug.breakpoints[buffer.file.path]?.first(where: { $0["line"] as? Int == row.line })
                    let glyph = foldHeaders.contains(row.line) ? "⋯" : row.line == paused ? "⋯" : row.line == 0 ? "▷" : held == nil ? "·" : (held?["state"] as? String == "refused" ? "○" : held?["state"] as? String == "bound" ? "●" : "◌")
                    Button(glyph) { opened = row.line }
                        .buttonStyle(.plain).font(.system(size: 13)).frame(width: 24, height: min(18, row.height))
                        .accessibilityLabel("Debug actions on line \(row.line + 1)")
                        .position(x: 12, y: row.y + min(18, row.height) / 2)
                        .popover(isPresented: Binding(get: { opened == row.line }, set: { if !$0 { opened = nil } })) {
                            controls(row.line).padding(10).frame(width: 310)
                        }
                    if row.line == paused, !debug.variables.isEmpty {
                        Text(debug.variables.prefix(3).map { "\($0["name"] as? String ?? "") = \($0["value"] as? String ?? "")" }.joined(separator: "  "))
                            .font(.system(size: 11, design: .monospaced)).foregroundStyle(.secondary).lineLimit(1)
                            .padding(3).background(.regularMaterial).frame(width: 220, alignment: .trailing)
                            .offset(x: -224, y: row.y).allowsHitTesting(false).accessibilityLabel("Paused values")
                    }
                }
            }
        }.background(.quaternary.opacity(0.2))
    }
    @ViewBuilder private func controls(_ line: Int) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Line \(line + 1) · \(debug.status)").font(.caption)
            if !debug.sourceMatches(buffer) { Text("Save edited text before launching. Values belong to the source that ran.").font(.caption) }
            LazyVGrid(columns: [GridItem(.adaptive(minimum: 86))], spacing: 5) {
                if simpleFoldRanges(buffer.text).contains(where: { $0.line == line }) {
                    Button(buffer.foldedLines.contains(line) ? "Unfold" : "Fold") { buffer.toggleFold?(line); opened = nil }
                }
                if !buffer.foldedLines.isEmpty { Button("Unfold all") { for header in Array(buffer.foldedLines) { buffer.toggleFold?(header) }; opened = nil } }
                Button("Debug") { Task { await debug.prepare(buffer, line: line) } }.disabled(debug.active || buffer.dirty || buffer.saving)
                Button("Breakpoint") { debug.toggle(buffer.file.path, line: line) }
                if debug.status == "paused" {
                    Button("Continue") { debug.step("continue") }; Button("Step over") { debug.step("over") }
                    Button("Step in") { debug.step("in") }; Button("Step out") { debug.step("out") }
                    if debug.capabilities["step_back"] as? Bool == true { Button("Step back") { debug.step("back") } }
                }
                if debug.active { Button("Stop") { debug.stop() } }
            }
            if debug.owner == buffer.file.path { ForEach(debug.targets) { target in Button("Debug file \(target.path)") { Task { await debug.prepare(buffer, program: target.path) } } } }
            ForEach(debug.profiles) { profile in Button("Debug \(profile.name)") { debug.launch(profile); opened = nil } }
            if debug.offer != nil { Button("Install debugger") { Task { await debug.install() } } }
            if !debug.message.isEmpty { Text(debug.message).font(.caption).textSelection(.enabled) }
            if debug.capabilities["conditional_breakpoints"] as? Bool == true {
                TextField("Breakpoint condition", text: $condition)
                Button("Apply condition") { debug.toggle(buffer.file.path, line: line, condition: condition) }
            }
            if debug.status == "paused" {
                TextField("Expression", text: $expression)
                HStack { Button("Evaluate") { debug.evaluate(expression) }; Button("Watch") { debug.evaluate(expression, watch: true) } }
                ScrollView {
                    VStack(alignment: .leading, spacing: 4) {
                        ForEach(Array(debug.frames.enumerated()), id: \.offset) { _, frame in Button(frame["name"] as? String ?? "Frame") { debug.select(frame) } }
                        ForEach(Array(debug.variables.enumerated()), id: \.offset) { _, variable in Text("\(variable["name"] as? String ?? "") = \(variable["value"] as? String ?? "")").textSelection(.enabled) }
                        ForEach(debug.values.keys.sorted(), id: \.self) { key in HStack { Text("\(key) = \(debug.values[key] ?? "…")"); if debug.watches.contains(key) { Button("×") { debug.watches.removeAll { $0 == key }; debug.values.removeValue(forKey: key) }.accessibilityLabel("Remove watch \(key)") } } }
                    }
                }.frame(maxHeight: 180)
            }
        }
    }
}
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/apple-simple/Sources/NativeEditor.swift"><ui:paste select="#simple-7fad175521-0"/><ui:paste select="#simple-apple-language-preview"/><ui:paste select="#simple-apple-debug-rail"/></ui:file>

</ui:doc>
