import Foundation

struct SimpleEnterPlan { var range: NSRange; var text: String; var caret: Int }
private struct EnterDefinition: Decodable {
    var id: String; var names: [String]; var comments: [String]; var blocks: [[String]]
    var multiline: [String]; var quotes: String; var open: String; var close: String
    var middle: String; var next: String; var regexLiterals: Bool; var after: String; var literal: String; var tags: Bool
    var voidTags: [String]; var foldCase: Bool; var offside: Bool
}
private final class EnterProfile {
    let rule: EnterDefinition
    private var patterns: [String: NSRegularExpression] = [:]
    init(_ rule: EnterDefinition) { self.rule = rule }
    func matches(_ pattern: String, _ text: String) -> Bool {
        guard !pattern.isEmpty else { return false }
        if patterns[pattern] == nil { patterns[pattern] = try? NSRegularExpression(pattern: pattern, options: rule.foldCase ? .caseInsensitive : []) }
        return patterns[pattern]?.firstMatch(in: text, range: NSRange(location: 0, length: text.utf16.count)) != nil
    }
}
private final class EnterResourceMarker: NSObject {}
final class SimpleEnterRules {
    let maxContext: Int; let maxLine: Int
    private let profiles: [EnterProfile]
    init(data: Data) throws {
        struct Registry: Decodable { var maxContext: Int; var maxLine: Int; var profiles: [EnterDefinition] }
        let registry = try JSONDecoder().decode(Registry.self, from: data)
        guard registry.profiles.allSatisfy({ rule in rule.blocks.allSatisfy { $0.count == 2 && $0.allSatisfy { !$0.isEmpty } } && rule.comments.allSatisfy { !$0.isEmpty } && rule.multiline.allSatisfy { !$0.isEmpty } }) else {
            throw NSError(domain: "SmartEnterRules", code: 1, userInfo: [NSLocalizedDescriptionKey: "Invalid indentation delimiters; rebuild the bundled Enter rules."])
        }
        maxContext = registry.maxContext; maxLine = registry.maxLine; profiles = registry.profiles.map(EnterProfile.init)
    }
    fileprivate func profile(_ path: String) -> EnterProfile? {
        let name = (path.replacingOccurrences(of: "\\", with: "/") as NSString).lastPathComponent.lowercased()
        let ext = (name as NSString).pathExtension
        return profiles.first { $0.rule.names.contains(ext.isEmpty ? name : ext) }
    }
    static let shared: SimpleEnterRules? = {
        // Both SwiftPM and Xcode bundle layouts, with a readable-text fallback if absent.
        let roots = [Bundle.main.resourceURL, Bundle.main.bundleURL, Bundle(for: EnterResourceMarker.self).bundleURL.deletingLastPathComponent(), URL(fileURLWithPath: CommandLine.arguments[0]).deletingLastPathComponent()].compactMap { $0 }
        let bundles = [Bundle.main] + roots.compactMap { Bundle(url: $0.appendingPathComponent("HickorySimple_HickorySimple.bundle")) }
        for bundle in bundles {
            if let url = bundle.url(forResource: "enter-rules", withExtension: "json", subdirectory: "HickoryAssets/Resources") ?? bundle.url(forResource: "enter-rules", withExtension: "json", subdirectory: "Resources") ?? bundle.url(forResource: "enter-rules", withExtension: "json"),
               let data = try? Data(contentsOf: url), let rules = try? SimpleEnterRules(data: data) { return rules }
        }
        return nil
    }()
}
private func enterOutdent(_ prefix: String, _ unit: String) -> String {
    var result = prefix
    if result.hasSuffix("\t") { result.removeLast(); return result }
    var count = 0
    while count < unit.count && result.hasSuffix(" ") { result.removeLast(); count += 1 }
    return result
}
private let enterExpressionCache: NSCache<NSString, NSRegularExpression> = { let cache = NSCache<NSString, NSRegularExpression>(); cache.countLimit = 32; return cache }()
private func enterExpression(_ pattern: String) -> NSRegularExpression? {
    if let value = enterExpressionCache.object(forKey: pattern as NSString) { return value }
    guard let value = try? NSRegularExpression(pattern: pattern) else { return nil }
    enterExpressionCache.setObject(value, forKey: pattern as NSString); return value
}
private func enterCapture(_ pattern: String, _ text: String) -> [String]? {
    guard let regex = enterExpression(pattern), let match = regex.firstMatch(in: text, range: NSRange(location: 0, length: text.utf16.count)) else { return nil }
    let source = text as NSString
    return (0..<match.numberOfRanges).map { match.range(at: $0).location == NSNotFound ? "" : source.substring(with: match.range(at: $0)) }
}
private struct EnterOpening { var token: String; var indent: String; var line: Int }
private struct EnterResult { var indent: String; var split = ""; var literal = false }
private final class EnterScanner {
    var block = "", quoted = false, brackets: [EnterOpening] = [], keywords: [String] = []
    var logical = "", logicalIndent = "", literal: String?, singleIndent: String?
    func clean(_ row: String, _ profile: EnterProfile) -> String {
        let text = row as NSString, rule = profile.rule
        var code = "", index = 0; quoted = false
        func has(_ mark: String) -> Bool { let size = mark.utf16.count; return index + size <= text.length && text.substring(with: NSRange(location: index, length: size)) == mark }
        while index < text.length {
            if !block.isEmpty {
                if has(block) && (index == 0 || text.character(at: index - 1) != 92) { index += block.utf16.count; block = "" } else { index += 1 }
                code += " "; continue
            }
            let pair = rule.blocks.first { has($0[0]) }, multi = rule.multiline.first { has($0) }
            if let pair { block = pair[1]; index += pair[0].utf16.count; code += " "; continue }
            if let multi { block = multi; index += multi.utf16.count; code += " "; continue }
            if rule.comments.contains(where: has) { break }
            if rule.regexLiterals, text.character(at: index) == 47, enterCapture(#"(?:^|\b(?:return|throw|yield|case)|[=(:,!&|?;{}\[>])\s*$"#, code) != nil {
                index += 1; var characterClass = false, closed = false
                while index < text.length {
                    let next = text.character(at: index); index += 1
                    if next == 92 { index = min(text.length, index + 1); continue }
                    if next == 91 { characterClass = true }; if next == 93 { characterClass = false }
                    if next == 47 && !characterClass { closed = true; break }
                }
                if !closed { quoted = true; block = "/" }; code += "~"; continue
            }
            let char = text.substring(with: NSRange(location: index, length: 1)); index += 1
            if rule.quotes.contains(char) {
                if char == "'", rule.id == "rust", !text.substring(from: index).contains("'") { code += char; continue }
                var closed = false
                while index < text.length {
                    let next = text.substring(with: NSRange(location: index, length: 1)); index += 1
                    if next == "\\" { index = min(text.length, index + 1); continue }
                    if next == char { closed = true; break }
                }
                if !closed { quoted = true; block = char }; code += "~"; continue
            }
            code += char
        }
        return code.trimmingCharacters(in: .whitespaces)
    }
    func line(_ row: String, _ profile: EnterProfile, _ number: Int, _ unit: String) -> EnterResult {
        let prefix = String(row.prefix { $0 == " " || $0 == "\t" }), rule = profile.rule
        if let literal, row.trimmingCharacters(in: .whitespaces).isEmpty || prefix.count > literal.count { return EnterResult(indent: prefix, literal: true) }
        literal = nil
        let wasBlock = !block.isEmpty, code = clean(row, profile), startDepth = brackets.count
        if wasBlock || !block.isEmpty || quoted || code.isEmpty { return EnterResult(indent: prefix, literal: true) }
        if startDepth == 0 { logical = code; logicalIndent = prefix }
        var base = prefix
        if profile.matches(rule.close, code) { base = keywords.popLast() ?? enterOutdent(prefix, unit) }
        let middle = profile.matches(rule.middle, code)
        if middle, !rule.offside, let parent = keywords.last { base = parent }
        if rule.tags {
            let regex = enterExpression(#"<(/?)([A-Za-z][\w:.-]*)\b[^>]*>"#)!, text = code as NSString
            for match in regex.matches(in: code, range: NSRange(location: 0, length: text.length)) {
                let raw = text.substring(with: match.range), name = text.substring(with: match.range(at: 2))
                let tag = rule.foldCase ? name.lowercased() : name
                if match.range(at: 1).length > 0 { if brackets.last?.token == "</" + tag + ">" { base = brackets.removeLast().indent } }
                else if !raw.hasSuffix("/>") && !rule.voidTags.contains(tag) { brackets.append(EnterOpening(token: "</" + tag + ">", indent: base, line: number)) }
            }
        } else {
            for char in code {
                if "{[(".contains(char) { brackets.append(EnterOpening(token: String(char), indent: base, line: number)) }
                if let index = Array("}])").firstIndex(of: char), brackets.last?.token == String(Array("{[(")[index]) { base = brackets.removeLast().indent }
            }
        }
        if let top = brackets.last {
            let split = top.token.hasPrefix("</") ? top.token : String(Array("}])")[Array("{[(").firstIndex(of: Character(top.token))!])
            return EnterResult(indent: top.line == number || base.count <= top.indent.count ? top.indent + unit : base, split: split)
        }
        if profile.matches(rule.next, code) {
            if singleIndent == nil { singleIndent = base }; return EnterResult(indent: base + unit)
        }
        if profile.matches(rule.open, code) || middle {
            if !middle { keywords.append(base) }; return EnterResult(indent: base + unit)
        }
        if profile.matches(rule.literal, code) { literal = base; return EnterResult(indent: base + unit) }
        if profile.matches(rule.after, logical) { base = enterOutdent(logicalIndent, unit) }
        if let singleIndent { base = singleIndent; self.singleIndent = nil }
        return EnterResult(indent: base)
    }
}
func smartEnter(_ source: String, range: NSRange, path: String, unit: String, mode: String, rules: SimpleEnterRules? = .shared) -> SimpleEnterPlan {
    let text = source as NSString, currentLine = (source as NSString).lineRange(for: NSRange(location: range.location, length: 0))
    let row = text.substring(with: NSRange(location: currentLine.location, length: range.location - currentLine.location)), prefix = String(row.prefix { $0 == " " || $0 == "\t" })
    let eol = simpleNewlineText(source, at: range.location).hasPrefix("\r\n") ? "\r\n" : "\n"
    func fallback(_ indent: String) -> SimpleEnterPlan { SimpleEnterPlan(range: range, text: eol + indent, caret: range.location + eol.utf16.count + indent.utf16.count) }
    if mode == "none" { return fallback("") }
    guard mode != "copy", let rules, range.location <= rules.maxContext, row.utf16.count <= rules.maxLine else { return fallback(prefix) }
    let rows = text.substring(to: range.location).components(separatedBy: "\n"), markdown = ["md", "markdown"].contains((path as NSString).pathExtension.lowercased())
    var profile = rules.profile(path), scanner = EnterScanner(), fence = "", payload = "", result = EnterResult(indent: prefix, literal: true)
    if !markdown && profile == nil { return fallback(prefix) }
    for (line, raw) in rows.enumerated() {
        let row = raw.hasSuffix("\r") ? String(raw.dropLast()) : raw, trimmed = row.trimmingCharacters(in: .whitespaces)
        if row.utf16.count > rules.maxLine { return fallback(prefix) }
        if markdown {
            let marker = enterCapture(#"^(`{3,}|~{3,})([^ ]*)"#, trimmed)
            if !fence.isEmpty {
                if trimmed.hasPrefix(fence), enterCapture(#"^(`+|~+)\s*$"#, trimmed) != nil { fence = ""; profile = nil; scanner = EnterScanner(); continue }
            } else if payload.isEmpty, let marker { fence = marker[1]; profile = rules.profile(marker[2]); scanner = EnterScanner(); continue }
            else {
                if let open = enterCapture(#"^<[\w.-]+:(file|copy|cut|exec)\b[^>]*>"#, trimmed) {
                    payload = open[1]; let hint = enterCapture(#"\b(?:path|language|lang)\s*=\s*["']([^"']+)["']"#, open[0])
                    profile = hint.flatMap { rules.profile($0[1]) }; scanner = EnterScanner(); continue
                }
                if !payload.isEmpty, trimmed.hasPrefix("</"), trimmed.contains(":" + payload + ">") { payload = ""; profile = nil; scanner = EnterScanner(); continue }
            }
        }
        if let profile { result = scanner.line(row, profile, line, unit) }
    }
    guard profile != nil, !result.literal else { return fallback(prefix) }
    let suffixRange = text.lineRange(for: NSRange(location: NSMaxRange(range), length: 0))
    let suffix = text.substring(with: NSRange(location: NSMaxRange(range), length: NSMaxRange(suffixRange) - NSMaxRange(range))).trimmingCharacters(in: .newlines)
    let whitespace = suffix.prefix { $0 == " " || $0 == "\t" }.utf16.count
    if !result.split.isEmpty, (profile?.rule.foldCase == true ? String(suffix.dropFirst(whitespace)).lowercased() : String(suffix.dropFirst(whitespace))).hasPrefix(result.split), let top = scanner.brackets.last {
        let openingHere = top.line == rows.count - 1
        let inserted = openingHere ? eol + result.indent + eol + top.indent : eol + top.indent
        return SimpleEnterPlan(range: NSRange(location: range.location, length: range.length + whitespace), text: inserted, caret: range.location + eol.utf16.count + (openingHere ? result.indent.utf16.count : top.indent.utf16.count))
    }
    return fallback(result.indent)
}
