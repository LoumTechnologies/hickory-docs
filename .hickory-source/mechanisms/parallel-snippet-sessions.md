<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Prepare one snippet action for several carets

An expansion is local to the caret that requested it. For example, two selected
words can produce different initial field values even when they use the same
template. A peer retains its local field forest, source region and stable caret
identity. The primary caret survives sorting into source order. Moving between
fields advances each peer independently, and a peer at its final caret stays
there while the others navigate.

A batch has one authored source baseline, one transform work budget, and a
combined limit on fields, text and metadata. Each local edit must stay inside
the peer's current field, or inside its local region once the peer has reached
its final caret. The caller receives all edits and the resulting selections
together. A stale source, invalid scalar boundary, overlapping replacement or
failed transform leaves the previous state available for ordinary editing.
Coincident empty regions retain their order in the session even when a text
widget draws a single caret there.

## Prepare browser source coordinates

Use authored UTF-16 positions. The editor adapter converts to display positions
only when it reflects the complete proposal.

<ui:copy id="parallel-snippet-session-browser">
import {
  advanceSnippet, beginSnippet, editSnippet, snippetMetadataCost,
  type SnippetSession,
} from './SnippetSession';
import type {SnippetField} from './SnippetExpansion';

export interface SnippetRange {from: number; to: number}
export interface ParallelReplacement extends SnippetRange {text: string}
export interface ParallelExpansion extends ParallelReplacement {fields: readonly SnippetField[]}

interface Peer extends SnippetRange {
  id: number;
  text: string;
  selection: SnippetRange;
  session: SnippetSession | null;
}

export interface ParallelSnippet {
  source: string;
  peers: readonly Peer[];
  primary: number;
}

export interface ParallelSnippetChange {
  edits: readonly ParallelReplacement[];
  state: ParallelSnippet;
}

const maxCarets = 256;
const maxFields = 4096;
const maxBytes = 1024 * 1024;
const encoder = new TextEncoder();

function scalarText(text: string): boolean {
  for (let at = 0; at < text.length; at++) {
    const unit = text.charCodeAt(at);
    if (unit >= 0xd800 && unit <= 0xdbff) {
      const next = text.charCodeAt(++at);
      if (!(next >= 0xdc00 && next <= 0xdfff)) return false;
    } else if (unit >= 0xdc00 && unit <= 0xdfff) {
      return false;
    }
  }
  return true;
}

function boundary(text: string, at: number): boolean {
  if (!Number.isSafeInteger(at) || at < 0 || at > text.length) return false;
  if (at === 0 || at === text.length) return true;
  const left = text.charCodeAt(at - 1);
  const right = text.charCodeAt(at);
  return !(left === 13 && right === 10 ||
    left >= 0xd800 && left <= 0xdbff && right >= 0xdc00 && right <= 0xdfff);
}

function valid(text: string, range: SnippetRange): boolean {
  return range.from <= range.to && boundary(text, range.from) && boundary(text, range.to);
}

function sameRange(a: SnippetRange, b: SnippetRange): boolean {
  return a.from === b.from && a.to === b.to;
}

function replace(source: string, edits: readonly ParallelReplacement[]): string {
  const parts: string[] = [];
  let start = 0;
  for (const edit of edits) {
    parts.push(source.slice(start, edit.from), edit.text);
    start = edit.to;
  }
  parts.push(source.slice(start));
  return parts.join('');
}

function current(state: ParallelSnippet, source: string): boolean {
  return state.source === source && state.peers.every((peer, index) =>
    valid(source, peer) && source.slice(peer.from, peer.to) === peer.text &&
    valid(peer.text, peer.selection) && (!peer.session || peer.session.source === peer.text) &&
    (index === 0 || state.peers[index - 1].to <= peer.from)
  );
}

export function parallelSelections(state: ParallelSnippet): {
  ranges: SnippetRange[];
  primary: number;
} {
  return {
    ranges: state.peers.map(peer => ({
      from: peer.from + peer.selection.from, to: peer.from + peer.selection.to,
    })),
    primary: state.peers.findIndex(peer => peer.id === state.primary),
  };
}

export function beginParallelSnippet(
  source: string, expansions: readonly ParallelExpansion[], primary: number,
): ParallelSnippetChange | null {
  if (source.length > 8 * maxBytes || expansions.length < 2 || expansions.length > maxCarets ||
      !Number.isInteger(primary) || primary < 0 || primary >= expansions.length ||
      !scalarText(source) || encoder.encode(source).length > 8 * maxBytes) return null;

  const ordered = expansions.map((edit, id) => ({edit, id}))
    .sort((a, b) => a.edit.from - b.edit.from || a.edit.to - b.edit.to);
  const peers: Peer[] = [];
  const edits: ParallelReplacement[] = [];
  const budget = {work: 0};
  let bytes = 0;
  let metadata = 0;
  let fields = 0;
  let shift = 0;
  for (const {edit, id} of ordered) {
    const previous = edits.at(-1);
    if (!valid(source, edit) || previous && (previous.to > edit.from || sameRange(previous, edit)) ||
        edit.text.length > maxBytes - bytes || !scalarText(edit.text)) return null;
    bytes += encoder.encode(edit.text).length;
    fields += edit.fields.length;
    const cost = snippetMetadataCost(edit.fields);
    if (cost === null || bytes > maxBytes || fields > maxFields || cost > maxBytes - metadata) return null;
    metadata += cost;

    const session = beginSnippet(edit.text, edit.fields, budget);
    const exit = edit.fields.find(field => field.index === 0);
    if (!exit || !valid(edit.text, exit) || exit.from !== exit.to) return null;
    // An expansion without editable fields still keeps its own terminal caret.
    if (!session && (edit.fields.length !== 1 || exit.transform || exit.parent != null ||
        exit.id == null || !Number.isSafeInteger(exit.id) || exit.id < 0 || exit.id >= maxFields)) return null;
    const field = session?.fields.find(field => field.id === session.current) ?? exit;
    const from = edit.from + shift;
    peers.push({
      id, from, to: from + edit.text.length, text: edit.text, session,
      selection: {from: field.from, to: field.to},
    });
    edits.push({from: edit.from, to: edit.to, text: edit.text});
    shift += edit.text.length - (edit.to - edit.from);
  }

  const after = replace(source, edits);
  const state = {source: after, peers, primary};
  return current(state, after) ? {edits, state} : null;
}

function selectPeer(peer: Peer, absolute: SnippetRange): Peer | null {
  const selection = {from: absolute.from - peer.from, to: absolute.to - peer.from};
  if (!valid(peer.text, selection)) return null;
  if (!peer.session || peer.session.active === 0) return {...peer, selection};

  const session = peer.session;
  const match = session.fields.find(field => field.id === session.current &&
    field.from <= selection.from && selection.to <= field.to);
  if (!match || session.fields.some(field => field.transform && field.from < field.to &&
      field.from <= selection.from && selection.to <= field.to &&
      (selection.from < selection.to || field.from < selection.from && selection.from < field.to))) return null;
  return {...peer, selection};
}

export function selectParallelSnippet(
  state: ParallelSnippet, source: string, ranges: readonly SnippetRange[], primary: number,
): ParallelSnippet | null {
  if (!current(state, source) || ranges.length !== state.peers.length) return null;
  const main = state.peers[primary];
  if (!main) return null;
  const peers: Peer[] = [];
  for (const [index, peer] of state.peers.entries()) {
    const selected = selectPeer(peer, ranges[index]);
    if (!selected) return null;
    peers.push(selected);
  }
  return {...state, peers, primary: main.id};
}

// Ranges are in original source coordinates in peer order. An editor can supply
// different deletions or pasted fragments. Construct every peer before applying.
export function editParallelSnippet(
  state: ParallelSnippet, source: string, input: readonly ParallelReplacement[],
): ParallelSnippetChange | null {
  if (!current(state, source) || input.length !== state.peers.length) return null;

  const budget = {work: 0};
  const peers: Peer[] = [];
  const edits: ParallelReplacement[] = [];
  let shift = 0;
  let bytes = 0;
  for (const [index, peer] of state.peers.entries()) {
    const replacement = input[index];
    const selected = selectPeer(peer, replacement);
    if (!selected || replacement.text.length > maxBytes - bytes ||
        !scalarText(replacement.text)) return null;
    const {from, to} = selected.selection;
    let plan;
    if (peer.session?.active) {
      plan = editSnippet(peer.session, peer.text, from, to, replacement.text, budget);
      if (!plan) return null;
    }
    const text = plan?.session.source ?? replace(peer.text, [{from, to, text: replacement.text}]);
    const caret = from + replacement.text.length;
    const selection = plan?.selection ?? {from: caret, to: caret};
    const start = peer.from + shift;
    bytes += encoder.encode(text).length;
    if (bytes > maxBytes) return null;
    peers.push({id: peer.id, from: start, to: start + text.length, text, selection, session: plan?.session ?? null});
    shift += text.length - peer.text.length;

    for (const local of plan?.edits ?? [{from, to, text: replacement.text}]) {
      const edit = {from: peer.from + local.from, to: peer.from + local.to, text: local.text};
      const previous = edits.at(-1);
      if (previous && previous.to > edit.from) return null;
      if (previous && previous.to === edit.from) {
        // Adjacent and coincident empty contributions retain original peer order.
        previous.to = edit.to;
        previous.text += edit.text;
      } else {
        edits.push(edit);
      }
    }
  }

  const after = replace(source, edits);
  const updated = {...state, source: after, peers};
  return current(updated, after) ? {edits, state: updated} : null;
}

export function moveParallelSnippet(
  state: ParallelSnippet, source: string, direction: 1 | -1,
): ParallelSnippet | null {
  if (!current(state, source)) return null;
  const peers: Peer[] = [];
  for (const peer of state.peers) {
    const {session, text, selection} = peer;
    if (!session) {
      peers.push(peer);
      continue;
    }
    const next = advanceSnippet(session, text, selection.from, selection.to, direction);
    if (!next) {
      const first = session.scopes?.at(-1)?.order[0] ??
        Math.min(...session.fields.filter(field => !field.transform && field.index > 0).map(field => field.index));
      if (!(direction === 1 && session.active === 0 || direction === -1 && session.active === first)) return null;
      peers.push(peer);
      continue;
    }
    const exit = session.fields.find(field => field.index === 0)!;
    peers.push({
      ...peer, selection: next.selection,
      session: next.session ?? {...session, active: 0, current: exit.id!},
    });
  }
  return {...state, peers};
}
</ui:copy>

<ui:file path="apps/web/src/simple/ParallelSnippet.ts"><ui:paste select="#parallel-snippet-session-browser"/></ui:file>

## Reflect native Apple ranges in the same source model

Keep native UTF-16 ranges without letting NSString conversion or Swift's
canonical string equality weaken exact authored-byte comparisons.

<ui:copy id="parallel-snippet-session-apple">
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

    func editing(_ source: String, input: [NativeSourceEdit]) -> ParallelSnippetChange? {
        guard matches(source), input.count == peers.count else { return nil }
        var budget = SnippetTransformBudget()
        var edits: [NativeSourceEdit] = []
        var next = self
        var shift = 0
        var bytes = 0
        for (index, peer) in peers.enumerated() {
            let replacement = input[index]
            guard let range = localSelection(replacement.range, in: peer),
                  replacement.text.utf8.count <= Self.maxBytes - bytes else { return nil }
            let plan: SnippetInput?
            if let session = peer.session, session.active != 0 {
                plan = editSnippet(session, source: peer.text, from: range.location, to: NSMaxRange(range),
                                   text: replacement.text, budget: &budget)
                if plan == nil { return nil }
            } else {
                plan = nil
            }
            let local = plan?.edits ?? [NativeSourceEdit(range: range, text: replacement.text)]
            let text = plan?.session.source ?? Self.replace(peer.text, local)
            bytes += text.utf8.count
            guard bytes <= Self.maxBytes else { return nil }
            let caret = range.location + replacement.text.utf16.count
            let selected = plan.map { NSRange(location: $0.selection.from, length: $0.selection.to - $0.selection.from) }
            let updatedLength = text.utf16.count
            next.peers[index] = Peer(
                id: peer.id, range: NSRange(location: peer.range.location + shift, length: updatedLength),
                text: text, selection: selected ?? NSRange(location: caret, length: 0), session: plan?.session
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
</ui:copy>

<ui:file path="apps/apple-simple/Sources/ParallelSnippet.swift"><ui:paste select="#parallel-snippet-session-apple"/></ui:file>

## Prepare Android edit proposals before touching the text buffer

The same local ranges describe independent, whole replacements. Compose
receives a plan only after every peer has passed validation.

<ui:copy id="parallel-snippet-session-android">
package com.hickorydocs.simple

internal data class ParallelSnippetExpansion(val edit: NativeSourceEdit, val fields: List<SnippetField>)
internal data class ParallelSnippetChange(val edits: List<NativeSourceEdit>, val state: ParallelSnippet)

internal data class ParallelSnippet(
    val source: String,
    val peers: List<Peer>,
    val primary: Int,
) {
    data class Peer(
        val id: Int, val from: Int, val to: Int, val text: String,
        val selection: CompletionSelection, val session: SnippetSession?,
    )

    val selections: List<CompletionSelection>
        get() = peers.map { CompletionSelection(it.from + it.selection.from, it.from + it.selection.to) }
    val primaryIndex: Int get() = peers.indexOfFirst { it.id == primary }

    companion object {
        const val maxCarets = 256
        private const val maxFields = 4096
        private const val maxBytes = 1024 * 1024

        private fun utf8Size(text: String): Int = text.toByteArray(Charsets.UTF_8).size

        private fun scalarText(text: String): Boolean {
            var at = 0
            while (at < text.length) {
                val unit = text[at++]
                if (unit.isHighSurrogate()) {
                    if (at >= text.length || !text[at++].isLowSurrogate()) return false
                } else if (unit.isLowSurrogate()) return false
            }
            return true
        }

        private fun boundary(text: String, at: Int): Boolean {
            if (at !in 0..text.length) return false
            if (at == 0 || at == text.length) return true
            val left = text[at - 1]
            val right = text[at]
            return !(left == '\r' && right == '\n' || left.isHighSurrogate() && right.isLowSurrogate())
        }

        private fun valid(text: String, from: Int, to: Int): Boolean =
            from <= to && boundary(text, from) && boundary(text, to)

        private fun replace(source: String, edits: List<NativeSourceEdit>): String = buildString {
            var start = 0
            for (edit in edits) {
                append(source, start, edit.from)
                append(edit.text)
                start = edit.to
            }
            append(source, start, source.length)
        }

        fun begin(source: String, expansions: List<ParallelSnippetExpansion>, primary: Int): ParallelSnippetChange? {
            if (expansions.size !in 2..maxCarets || primary !in expansions.indices ||
                source.length > 8 * maxBytes || !scalarText(source) || utf8Size(source) > 8 * maxBytes) return null
            val ordered = expansions.withIndex().sortedWith(compareBy({ it.value.edit.from }, { it.value.edit.to }))
            val peers = mutableListOf<Peer>()
            val edits = mutableListOf<NativeSourceEdit>()
            val budget = SnippetTransformBudget()
            var shift = 0
            var bytes = 0
            var metadata = 0L
            var fields = 0
            for ((id, expansion) in ordered) {
                val edit = expansion.edit
                val previous = edits.lastOrNull()
                if (!valid(source, edit.from, edit.to) ||
                    previous != null && (previous.to > edit.from || previous.from == edit.from && previous.to == edit.to) ||
                    edit.text.length > maxBytes - bytes || !scalarText(edit.text)) return null
                bytes += utf8Size(edit.text)
                fields += expansion.fields.size
                if (bytes > maxBytes || fields > maxFields) return null
                metadata += snippetMetadataCost(expansion.fields, maxBytes - metadata) ?: return null

                val session = beginSnippet(edit.text, expansion.fields, budget)
                val exit = expansion.fields.firstOrNull { it.index == 0 } ?: return null
                if (exit.from != exit.to || !valid(edit.text, exit.from, exit.to)) return null
                if (session == null && (expansion.fields.size != 1 || exit.transform != null ||
                    exit.parent != null || exit.id == null || exit.id !in 0 until maxFields)) return null
                val field = session?.fields?.firstOrNull { it.id == session.current } ?: exit
                val start = edit.from + shift
                peers.add(Peer(id, start, start + edit.text.length, edit.text,
                    CompletionSelection(field.from, field.to), session))
                edits.add(edit)
                shift += edit.text.length - (edit.to - edit.from)
            }
            val after = replace(source, edits)
            val state = ParallelSnippet(after, peers, primary)
            return if (state.matches(after)) ParallelSnippetChange(edits, state) else null
        }
    }

    private fun matches(text: String): Boolean {
        if (text != source) return false
        var previous = 0
        for (peer in peers) {
            if (peer.from < previous || !valid(text, peer.from, peer.to) ||
                text.substring(peer.from, peer.to) != peer.text ||
                !valid(peer.text, peer.selection.from, peer.selection.to) ||
                peer.session != null && peer.session.source != peer.text) return false
            previous = peer.to
        }
        return true
    }

    private fun localSelection(peer: Peer, absolute: CompletionSelection): CompletionSelection? {
        if (absolute.from < peer.from || absolute.to > peer.to) return null
        val range = CompletionSelection(absolute.from - peer.from, absolute.to - peer.from)
        if (!valid(peer.text, range.from, range.to)) return null
        val session = peer.session
        if (session != null && session.active != 0) {
            if (session.fields.none { it.id == session.current && it.from <= range.from && range.to <= it.to }) return null
            if (session.fields.any { it.transform != null && it.from < it.to &&
                    it.from <= range.from && range.to <= it.to &&
                    (range.from < range.to || it.from < range.from && range.from < it.to) }) return null
        }
        return range
    }

    fun selecting(source: String, ranges: List<CompletionSelection>, primary: Int): ParallelSnippet? {
        if (!matches(source) || ranges.size != peers.size || primary !in peers.indices) return null
        val selected = peers.mapIndexed { index, peer ->
            peer.copy(selection = localSelection(peer, ranges[index]) ?: return null)
        }
        return copy(peers = selected, primary = peers[primary].id)
    }

    fun editing(source: String, input: List<NativeSourceEdit>): ParallelSnippetChange? {
        if (!matches(source) || input.size != peers.size) return null
        val budget = SnippetTransformBudget()
        val edits = mutableListOf<NativeSourceEdit>()
        val updated = mutableListOf<Peer>()
        var shift = 0
        var bytes = 0
        for ((index, peer) in peers.withIndex()) {
            val replacement = input[index]
            val range = localSelection(peer, CompletionSelection(replacement.from, replacement.to)) ?: return null
            if (replacement.text.length > maxBytes - bytes || !scalarText(replacement.text)) return null
            val session = peer.session
            val plan = if (session != null && session.active != 0) {
                editSnippet(session, peer.text, range.from, range.to, replacement.text, budget) ?: return null
            } else null
            val local = plan?.edits ?: listOf(NativeSourceEdit(range.from, range.to, replacement.text))
            val text = plan?.session?.source ?: replace(peer.text, local)
            bytes += utf8Size(text)
            if (bytes > maxBytes) return null
            val start = peer.from + shift
            val caret = range.from + replacement.text.length
            updated.add(Peer(peer.id, start, start + text.length, text,
                plan?.selection ?: CompletionSelection(caret, caret), plan?.session))
            shift += text.length - peer.text.length

            for (localEdit in local) {
                val edit = NativeSourceEdit(peer.from + localEdit.from, peer.from + localEdit.to, localEdit.text)
                val previous = edits.lastOrNull()
                if (previous != null && previous.to > edit.from) return null
                if (previous != null && previous.to == edit.from) {
                    edits[edits.lastIndex] = NativeSourceEdit(previous.from, edit.to, previous.text + edit.text)
                } else edits.add(edit)
            }
        }
        val after = replace(source, edits)
        val next = copy(source = after, peers = updated)
        return if (next.matches(after)) ParallelSnippetChange(edits, next) else null
    }

    fun moving(source: String, direction: Int): ParallelSnippet? {
        if (!matches(source) || direction != 1 && direction != -1) return null
        val updated = peers.map { peer ->
            val session = peer.session ?: return@map peer
            val advance = advanceSnippet(session, peer.text, peer.selection.from, peer.selection.to, direction)
            if (advance == null) {
                val first = session.scopes.lastOrNull()?.order?.firstOrNull() ?:
                    session.fields.filter { it.transform == null && it.index > 0 }.minOfOrNull { it.index }
                if (direction == 1 && session.active == 0 || direction == -1 && session.active == first) return@map peer
                return null
            }
            val next = advance.session ?: session.copy(active = 0, current = session.fields.first { it.index == 0 }.id!!)
            peer.copy(selection = advance.selection, session = next)
        }
        return copy(peers = updated)
    }
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/ParallelSnippet.kt"><ui:paste select="#parallel-snippet-session-android"/></ui:file>

</ui:doc>
