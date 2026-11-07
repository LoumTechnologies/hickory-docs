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
func attribute(_ element: AXUIElement, _ name: String) -> CFTypeRef? {
    var value: CFTypeRef?
    return AXUIElementCopyAttributeValue(element, name as CFString, &value) == .success ? value : nil
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
            if kind == "browser" && !title.contains("Hickory Simple") { continue }
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

if CommandLine.arguments.contains("--self-test") {
    let frames = grid(CGRect(x: 100, y: 20, width: 1024, height: 768), desktopTop: 900)
    precondition(frames.count == 4)
    precondition(frames[0].origin == CGPoint(x: 108, y: 120))
    precondition(frames[1].minX > frames[0].maxX && frames[2].minY > frames[0].maxY)
    precondition(frames[3].maxX <= 1124 && frames[3].maxY <= 880)
    let shifted = grid(CGRect(x: -1400, y: -100, width: 1400, height: 900), desktopTop: 900)
    precondition(shifted[0].minX == -1392 && shifted[0].minY == 108)
    print("Window layout geometry passed, including shifted displays and top-left conversion.")
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
