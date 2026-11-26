<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Arrange the four comparison windows without touching unrelated work

For maintainers iterating on the equivalent Apple and browser editor frontends.

Use visible display points, leaving room for the menu bar and Dock. Convert AppKit’s bottom-left screen coordinates to Accessibility’s top-left coordinates relative to the primary display. The four slots are browser, macOS, iOS and Android; these are desired bounds, because simulators may impose aspect ratios or minimum dimensions.

<ui:copy id="simple-1805c24098-0">
import AppKit
import ApplicationServices
import Darwin

func grid(_ visible: CGRect, desktopTop: CGFloat) -> [CGRect] {
    let gap: CGFloat = 8
    let width = max(1, (visible.width - gap * 3) / 2)
    let height = max(1, (visible.height - gap * 3) / 2)
    let left = visible.minX + gap, top = desktopTop - visible.maxY + gap
    return (0..<4).map { index in
        CGRect(x: left + CGFloat(index % 2) * (width + gap), y: top + CGFloat(index / 2) * (height + gap), width: width, height: height)
    }
}
</ui:copy>

For browsers, confirm the loopback editor URL through Accessibility because the
Hickory Docs site shares the editor's title. Browsers can expose the URL on the
window or on its web area. Skip windows whose URL is unavailable and refuse to
guess when several candidates match. Request position and size through macOS
Accessibility and report failures without claiming exact placement.

<ui:copy id="simple-1805c24098-1">
func attribute(_ element: AXUIElement, _ name: String) -> CFTypeRef? {
    var value: CFTypeRef?
    return AXUIElementCopyAttributeValue(element, name as CFString, &value) == .success ? value : nil
}
func isComparisonEditorURL(_ url: URL) -> Bool {
    url.scheme == "http" && url.host == "127.0.0.1" && url.path == "/simple.html"
}
func showsComparisonEditor(_ element: AXUIElement) -> Bool {
    if let document = attribute(element, kAXDocumentAttribute) as? String,
       let url = URL(string: document), isComparisonEditorURL(url) {
        return true
    }
    if attribute(element, kAXRoleAttribute) as? String == "AXWebArea" {
        guard let url = attribute(element, kAXURLAttribute) as? URL else { return false }
        return isComparisonEditorURL(url)
    }
    let children = attribute(element, kAXChildrenAttribute) as? [AXUIElement] ?? []
    return children.contains(where: showsComparisonEditor)
}
func matchingWindows(_ kind: String) -> [AXUIElement] {
    var result: [AXUIElement] = []
    for app in NSWorkspace.shared.runningApplications {
        let id = app.bundleIdentifier ?? "", name = app.localizedName ?? ""
        let binary = app.executableURL?.lastPathComponent ?? ""
        let matches: Bool
        switch kind {
        case "macOS": matches = id == "com.hickorydocs.simple.mac"
        case "iOS": matches = id == "com.apple.iphonesimulator"
        case "Android": matches = id == "com.google.android.emulator" || binary.hasPrefix("qemu-system") || name.contains("Android Emulator")
        default: matches = ["com.google.Chrome", "com.brave.Browser", "org.mozilla.firefox", "com.apple.Safari", "com.microsoft.edgemac"].contains(id)
        }
        guard matches else { continue }
        let application = AXUIElementCreateApplication(app.processIdentifier)
        AXUIElementSetMessagingTimeout(application, 1)
        for window in attribute(application, kAXWindowsAttribute) as? [AXUIElement] ?? [] {
            let title = attribute(window, kAXTitleAttribute) as? String ?? ""
            if kind == "browser" && !(title.contains("Hickory Docs") && showsComparisonEditor(window)) { continue }
            if kind == "iOS" && !(title.contains("iPhone") || title.contains("iPad")) { continue }
            if kind == "Android" && (title.isEmpty || title.contains("Extended Controls")) { continue }
            result.append(window)
        }
    }
    return result
}
func position(_ window: AXUIElement, in rectangle: CGRect, kind: String) {
    var origin = rectangle.origin, size = rectangle.size
    let sizeValue = AXValueCreate(.cgSize, &size)!, originValue = AXValueCreate(.cgPoint, &origin)!
    _ = AXUIElementSetAttributeValue(window, kAXMinimizedAttribute as CFString, kCFBooleanFalse)
    let resized = AXUIElementSetAttributeValue(window, kAXSizeAttribute as CFString, sizeValue)
    let moved = AXUIElementSetAttributeValue(window, kAXPositionAttribute as CFString, originValue)
    _ = AXUIElementPerformAction(window, kAXRaiseAction as CFString)
    if resized != .success || moved != .success {
        print("Layout: \(kind) requested \(rectangle); resize=\(resized.rawValue), move=\(moved.rawValue). The window may enforce minimum size or aspect ratio.")
    } else { print("Layout: placed \(kind).") }
}

</ui:copy>

The self-test checks display geometry and accepts the loopback editor URL while
rejecting the loopback home page and external pages. It runs without reading or
moving windows. Runtime layout requires existing Accessibility authorization.
Without it, print instructions and leave the frontends running. Poll for
compilation and device startup for up to five minutes and stop when the parent
comparison launcher exits. Screen selection is zero-based through
HICKORY_COMPARISON_SCREEN. The helper arranges each target once, allowing the
user to change the layout afterward.

<ui:copy id="simple-1805c24098-2">
if CommandLine.arguments.contains("--self-test") {
    precondition(isComparisonEditorURL(URL(string: "http://127.0.0.1:4321/simple.html?path=project.md")!))
    precondition(!isComparisonEditorURL(URL(string: "http://127.0.0.1:4321/")!))
    precondition(!isComparisonEditorURL(URL(string: "https://example.com/simple.html")!))
    let frames = grid(CGRect(x: 100, y: 20, width: 1024, height: 768), desktopTop: 900)
    precondition(frames.count == 4)
    precondition(frames[0].origin == CGPoint(x: 108, y: 120))
    precondition(frames[1].minX > frames[0].maxX && frames[2].minY > frames[0].maxY)
    precondition(frames[3].maxX <= 1124 && frames[3].maxY <= 880)
    let shifted = grid(CGRect(x: -1400, y: -100, width: 1400, height: 900), desktopTop: 900)
    precondition(shifted[0].minX == -1392 && shifted[0].minY == 108)
    print("Window layout geometry and comparison URL checks passed.")
    exit(0)
}
guard let hostText = CommandLine.arguments.dropFirst().first, let host = Int32(hostText), host > 1 else {
    fputs("Usage: simple-window-layout HOST_PID (or --self-test)\n", stderr); exit(2)
}
guard AXIsProcessTrusted() else {
    print("Layout skipped: macOS Accessibility access is needed to move comparison windows. Enable the terminal running just in System Settings > Privacy & Security > Accessibility, then rerun. No permissions were changed.")
    exit(0)
}
let screens = NSScreen.screens
let index = Int(ProcessInfo.processInfo.environment["HICKORY_COMPARISON_SCREEN"] ?? "0") ?? -1
guard screens.indices.contains(index), let primary = screens.first else {
    print("Layout skipped: choose HICKORY_COMPARISON_SCREEN from 0..<\(screens.count).")
    exit(0)
}
let frames = grid(screens[index].visibleFrame, desktopTop: primary.frame.maxY)
let kinds = ["browser", "macOS", "iOS", "Android"]
var pending = Set(kinds)
let deadline = Date().addingTimeInterval(300)
while !pending.isEmpty && Date() < deadline && (Darwin.kill(host, 0) == 0 || errno == EPERM) {
    for (slot, kind) in kinds.enumerated() where pending.contains(kind) {
        let windows = matchingWindows(kind)
        // Never guess which of several comparison windows the user means.
        if windows.count == 1 { position(windows[0], in: frames[slot], kind: kind); pending.remove(kind) }
    }
    Thread.sleep(forTimeInterval: 1)
}
if !pending.isEmpty { print("Layout: no unique window found for \(pending.sorted().joined(separator: ", ")). Close duplicate comparison windows or arrange these manually.") }
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="scripts/simple-window-layout.swift"><ui:paste select="#simple-1805c24098-0"/><ui:paste select="#simple-1805c24098-1"/><ui:paste select="#simple-1805c24098-2"/></ui:file>

</ui:doc>
