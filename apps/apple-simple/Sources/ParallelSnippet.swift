import Foundation

struct ParallelSnippetExpansion {
    let edit: NativeSourceEdit
    let fields: [SnippetField]
}

struct ParallelSnippetChange {
    let edits: [NativeSourceEdit]
    let state: ParallelSnippet
}

struct ParallelSnippet {
    struct Peer {
        let id: Int
        var range: NSRange
        var text: String
        var selection: NSRange
        var session: SnippetSession?
    }

    var source: String
    var peers: [Peer]
    var primary: Int

    static let maxCarets = 256
    private static let maxFields = 4096
    private static let maxBytes = 1024 * 1024

    var selections: [NSRange] {
        peers.map { NSRange(location: $0.range.location + $0.selection.location, length: $0.selection.length) }
    }

    var primaryIndex: Int { peers.firstIndex { $0.id == primary }! }

    private static func valid(_ text: NSString, _ range: NSRange) -> Bool {
        guard range.location >= 0, range.length >= 0,
              range.location <= text.length, range.length <= text.length - range.location else { return false }
        return boundary(text, range.location) && boundary(text, NSMaxRange(range))
    }

    private static func boundary(_ text: NSString, _ at: Int) -> Bool {
        if at == 0 || at == text.length { return true }
        let left = text.character(at: at - 1)
        let right = text.character(at: at)
        return !(left == 13 && right == 10 ||
                 (0xd800...0xdbff).contains(left) && (0xdc00...0xdfff).contains(right))
    }

    private static func replace(_ source: String, _ edits: [NativeSourceEdit]) -> String {
        let result = NSMutableString(string: source)
        for edit in edits.reversed() { result.replaceCharacters(in: edit.range, with: edit.text) }
        return result as String
    }

    private func matches(_ text: String) -> Bool {
        guard exactUTF8(text, source) else { return false }
        let original = text as NSString
        var previous = 0
        for peer in peers {
            guard peer.range.location >= previous, Self.valid(original, peer.range),
                  exactUTF8(original.substring(with: peer.range), peer.text),
                  Self.valid(peer.text as NSString, peer.selection) else { return false }
            if let session = peer.session, !exactUTF8(session.source, peer.text) { return false }
            previous = NSMaxRange(peer.range)
        }
        return true
    }

    static func begin(_ source: String, expansions: [ParallelSnippetExpansion], primary: Int) -> ParallelSnippetChange? {
        guard (2...maxCarets).contains(expansions.count), expansions.indices.contains(primary),
              source.utf8.count <= 8 * maxBytes else { return nil }
        let original = source as NSString
        let ordered = expansions.enumerated().sorted {
            ($0.element.edit.range.location, $0.element.edit.range.length) <
                ($1.element.edit.range.location, $1.element.edit.range.length)
        }
        var edits: [NativeSourceEdit] = []
        var peers: [Peer] = []
        var shift = 0
        var bytes = 0
        var metadata = 0
        var fields = 0
        var budget = SnippetTransformBudget()
        for (id, expansion) in ordered {
            let edit = expansion.edit
            guard valid(original, edit.range) else { return nil }
            if let previous = edits.last,
               NSMaxRange(previous.range) > edit.range.location || previous.range == edit.range { return nil }
            bytes += edit.text.utf8.count
            fields += expansion.fields.count
            guard bytes <= maxBytes, fields <= maxFields,
                  let cost = snippetMetadataCost(expansion.fields), cost <= maxBytes - metadata else { return nil }
            metadata += cost
            let session = beginSnippet(edit.text, fields: expansion.fields, budget: &budget)
            guard let exit = expansion.fields.first(where: { $0.index == 0 }),
                  exit.from == exit.to, valid(edit.text as NSString, NSRange(location: exit.from, length: 0)) else { return nil }
            if session == nil {
                guard expansion.fields.count == 1, exit.parent == nil, exit.transform == nil,
                      let id = exit.id, (0..<maxFields).contains(id) else { return nil }
            }
            let field = session.flatMap { live in live.fields.first { $0.id == live.current } } ?? exit
            let length = edit.text.utf16.count
            let range = NSRange(location: edit.range.location + shift, length: length)
            let selection = NSRange(location: field.from, length: field.to - field.from)
            peers.append(Peer(id: id, range: range, text: edit.text, selection: selection, session: session))
            edits.append(edit)
            shift += length - edit.range.length
        }
        let after = replace(source, edits)
        let state = ParallelSnippet(source: after, peers: peers, primary: primary)
        return state.matches(after) ? ParallelSnippetChange(edits: edits, state: state) : nil
    }

    private func localSelection(_ absolute: NSRange, in peer: Peer) -> NSRange? {
        guard absolute.location >= peer.range.location else { return nil }
        let range = NSRange(location: absolute.location - peer.range.location, length: absolute.length)
        guard Self.valid(peer.text as NSString, range) else { return nil }
        if let session = peer.session, session.active != 0 {
            let end = NSMaxRange(range)
            guard session.fields.contains(where: {
                $0.id == session.current && $0.from <= range.location && end <= $0.to
            }), !session.fields.contains(where: {
                $0.transform != nil && $0.from < $0.to && $0.from <= range.location && end <= $0.to &&
                    (range.length > 0 || $0.from < range.location && range.location < $0.to)
            }) else { return nil }
        }
        return range
    }

    func selecting(_ source: String, ranges: [NSRange], primary: Int) -> ParallelSnippet? {
        guard matches(source), ranges.count == peers.count, peers.indices.contains(primary) else { return nil }
        var next = self
        for index in peers.indices {
            guard let selection = localSelection(ranges[index], in: peers[index]) else { return nil }
            next.peers[index].selection = selection
        }
        next.primary = peers[primary].id
        return next
    }

    static func plan(_ source: String, expansions: [ParallelSnippetExpansion], primary: Int,
                     continuing: ParallelSnippet? = nil) -> ParallelSnippetChange? {
        guard let continuing else { return begin(source, expansions: expansions, primary: primary) }
        let input = expansions.map(\.edit)
        guard continuing.primaryIndex == primary, input.map(\.range) == continuing.selections else { return nil }
        return continuing.preparingInput(source, input: input, expansions: expansions)
    }

    func editing(_ source: String, input: [NativeSourceEdit]) -> ParallelSnippetChange? {
        preparingInput(source, input: input, expansions: nil)
    }

    private func preparingInput(_ source: String, input: [NativeSourceEdit],
                                expansions: [ParallelSnippetExpansion]?) -> ParallelSnippetChange? {
        guard matches(source), input.count == peers.count else { return nil }
        var budget = SnippetTransformBudget()
        var edits: [NativeSourceEdit] = []
        var next = self
        var shift = 0
        var bytes = 0
        var metadata = 0
        var fields = 0
        for (index, peer) in peers.enumerated() {
            let replacement = input[index]
            guard let range = localSelection(replacement.range, in: peer),
                  replacement.text.utf8.count <= Self.maxBytes - bytes else { return nil }
            let inner = expansions?[index].fields
            if let inner {
                guard let exit = inner.first(where: { $0.index == 0 }), exit.from == exit.to,
                      let id = exit.id, (0..<Self.maxFields).contains(id), exit.transform == nil,
                      inner.count <= Self.maxFields, inner.allSatisfy({ field in
                          field.from >= 0 && field.from <= field.to && field.to <= replacement.text.utf16.count &&
                              Self.valid(replacement.text as NSString,
                                  NSRange(location: field.from, length: field.to - field.from))
                      }) else { return nil }
            }
            let plan: SnippetInput?
            if let session = peer.session, session.active != 0 {
                if let inner {
                    let primary = SourceReplacement(from: range.location, to: NSMaxRange(range), text: replacement.text)
                    plan = nestSnippet(session, source: peer.text, primary: primary, additional: [], inner: inner, budget: &budget)
                } else {
                    plan = editSnippet(session, source: peer.text, from: range.location, to: NSMaxRange(range),
                                       text: replacement.text, budget: &budget)
                }
                if plan == nil { return nil }
            } else {
                plan = nil
            }
            let local = plan?.edits ?? [NativeSourceEdit(range: range, text: replacement.text)]
            let text = plan?.session.source ?? Self.replace(peer.text, local)
            bytes += text.utf8.count
            guard bytes <= Self.maxBytes else { return nil }
            let caret = range.location + replacement.text.utf16.count
            var selected = plan.map { NSRange(location: $0.selection.from, length: $0.selection.to - $0.selection.from) }
            var session = plan?.session
            if plan == nil, let inner {
                let exit = inner.first { $0.index == 0 }!
                let shifted = inner.map { field -> SnippetField in
                    var copy = field
                    copy.from += range.location
                    copy.to += range.location
                    return copy
                }
                session = beginSnippet(text, fields: shifted, budget: &budget)
                if let live = session, let field = live.fields.first(where: { $0.id == live.current }) {
                    selected = NSRange(location: field.from, length: field.to - field.from)
                } else {
                    guard shifted.count == 1, exit.parent == nil else { return nil }
                    selected = NSRange(location: range.location + exit.from, length: 0)
                }
            }
            if inner != nil {
                let retained = session?.fields ?? []
                fields += retained.count
                guard fields <= Self.maxFields, let cost = snippetMetadataCost(retained),
                      cost <= Self.maxBytes - metadata else { return nil }
                metadata += cost
            }
            let updatedLength = text.utf16.count
            next.peers[index] = Peer(
                id: peer.id, range: NSRange(location: peer.range.location + shift, length: updatedLength),
                text: text, selection: selected ?? NSRange(location: caret, length: 0), session: session
            )
            shift += updatedLength - peer.range.length
            for replacement in local {
                var range = replacement.range
                range.location += peer.range.location
                if let previous = edits.last {
                    let end = NSMaxRange(previous.range)
                    if end > range.location { return nil }
                    if end == range.location {
                        let combined = NSRange(location: previous.range.location, length: previous.range.length + range.length)
                        edits[edits.count - 1] = NativeSourceEdit(range: combined, text: previous.text + replacement.text)
                        continue
                    }
                }
                edits.append(NativeSourceEdit(range: range, text: replacement.text))
            }
        }
        next.source = Self.replace(source, edits)
        return next.matches(next.source) ? ParallelSnippetChange(edits: edits, state: next) : nil
    }

    func moving(_ source: String, direction: Int) -> ParallelSnippet? {
        guard matches(source), direction == 1 || direction == -1 else { return nil }
        var next = self
        for (index, peer) in peers.enumerated() {
            guard var session = peer.session else { continue }
            guard let advance = advanceSnippet(session, source: peer.text, from: peer.selection.location,
                                               to: NSMaxRange(peer.selection), direction: direction) else {
                let first = session.scopes.last?.order.first ??
                    session.fields.filter { $0.transform == nil && $0.index > 0 }.map(\.index).min()
                if direction == 1 && session.active == 0 || direction == -1 && session.active == first { continue }
                return nil
            }
            if let live = advance.session {
                session = live
            } else {
                session.active = 0
                session.current = session.fields.first { $0.index == 0 }!.id!
            }
            next.peers[index].selection = NSRange(location: advance.selection.from, length: advance.selection.to - advance.selection.from)
            next.peers[index].session = session
        }
        return next
    }
}
