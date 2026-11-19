<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Ask source questions independently of an editor

For maintainers extending semantic intelligence while keeping browser, Apple and Android presentations replaceable.

An IDE question has a typed operation, an input source position and provider results. The model normalizes locations, named symbols and call items; the connection adapter captures source, selection and provider generation; the workspace owns buffers and navigation; the UI only offers the question and displays the answer. Swapping a gutter menu or native text editor does not change these contracts.

## Name the questions

Use a closed vocabulary for declarations, definitions, types, implementations, usages, file outlines, workspace symbols and call relationships. Display names are independent of LSP method selection. A workspace symbol query has text rather than a fabricated source position.

<ui:copy id="simple-intelligence-browser-vocabulary">
import type { LspLocation, LspClient } from "../lsp/client";
export type SemanticLocation = LspLocation & { name?: string; kind?: number };
export const semanticQueries = [
  { kind: "definition", label: "Go to definition" },
  { kind: "references", label: "Find usages" },
  { kind: "typeDefinition", label: "Go to type definition" },
  { kind: "implementation", label: "Find implementations" },
  { kind: "declaration", label: "Go to declaration" },
  { kind: "outline", label: "File outline" },
  { kind: "symbols", label: "Search workspace symbols" },
  { kind: "callers", label: "Find callers" },
  { kind: "callees", label: "Find callees" },
  { kind: "supertypes", label: "Find supertypes" },
  { kind: "subtypes", label: "Find subtypes" },
] as const;
export type SemanticQuery = typeof semanticQueries[number]["kind"];
</ui:copy>

## Normalize provider identities once

Nested document symbols inherit the queried file. Flat symbols, LocationLinks and call items carry explicit identities. Decode URI path segments once and reject traversal, external authorities and control bytes. Preserve symbol names and valid LSP symbol kinds for reading and type/member filtering, while navigation identity is the URI and UTF-16 position. Main Structure also requests qualified labels so someone can distinguish methods in different containers. Invalid or absent kinds remain unknown rather than being guessed from names. Bound nesting and result counts without claiming missing relationships.

<ui:copy id="simple-intelligence-browser-locations">
export function semanticPath(uri: string): string | null {
  try {
    const value = new URL(uri);
    if (!uri.startsWith("hick:///") || /[?#]/.test(uri) || value.host || value.search || value.hash) return null;
    // URL canonicalization erases dot segments; refuse them in the original URI.
    const path = decodeURIComponent(uri.slice("hick:///".length));
    if (!path || /[:\\\u0000-\u001f]/.test(path) || path.split("/").some(part => !part || part === "." || part === "..")) return null;
    return path;
  } catch { return null; }
}
export function semanticLocations(result: unknown, defaultUri?: string, qualifyNames = false): SemanticLocation[] {
  const seen = new Set<string>(), locations: SemanticLocation[] = [];
  const visit = (value: unknown, depth: number, parent?: string) => {
    if (depth > 64) throw new Error("Provider symbol nesting is too deep. Narrow the query.");
    if (Array.isArray(value)) { for (const child of value) visit(child, depth + 1, parent); return; }
    if (!value || typeof value !== "object") return;
    const item = value as Record<string, unknown>;
    if (item.from || item.to) { visit(item.from ?? item.to, depth + 1); return; }
    const container = parent ?? (typeof item.containerName === "string" ? item.containerName : undefined);
    const name = typeof item.name === "string" ? (qualifyNames && container ? `${container}.${item.name}` : item.name) : undefined;
    const target = (item.location ?? item) as Record<string, unknown>;
    const uri = target.uri ?? target.targetUri ?? (typeof item.name === "string" && !item.location ? defaultUri : undefined);
    const range = (target.selectionRange ?? target.range ?? target.targetSelectionRange ?? target.targetRange) as LspLocation["range"] | undefined;
    const position = range?.start;
    if (typeof uri === "string" && position && Number.isSafeInteger(position.line) && position.line >= 0 && Number.isSafeInteger(position.character) && position.character >= 0) {
      const key = JSON.stringify([uri, position.line, position.character]);
      if (!seen.has(key)) { seen.add(key); locations.push({ uri, range: range!, ...(name === undefined ? {} : { name }), ...(Number.isInteger(item.kind) && Number(item.kind) >= 1 && Number(item.kind) <= 26 ? { kind: Number(item.kind) } : {}) }); }
      if (locations.length > 10000) throw new Error("More than 10,000 locations. Narrow the symbol query.");
    }
    if (item.children) visit(item.children, depth + 1, name ?? parent);
  };
  visit(result, 0); return locations;
}
</ui:copy>

## Follow the provider protocol

Call and type relationships share a prepare/follow-up model. Preserve opaque items until their matching follow-up; normalize only the returned source locations. Supertypes and subtypes use standard type-hierarchy methods through the shared server registration. Missing support remains an unavailable query or empty report, never an inferred relationship.

Calls are two-phase reads: prepare candidate items, then ask for incoming or outgoing calls for those exact items. A broad ambiguous preparation is refused instead of silently choosing one symbol. Outlines and workspace symbols use their own protocol shapes; source-position reads share one path. No query applies an edit or saves a file.

<ui:copy id="simple-intelligence-browser-query">
export async function querySemanticLocations(client: LspClient, uri: string, position: { line: number; character: number }, kind: SemanticQuery, text = ""): Promise<SemanticLocation[]> {
  const params = { textDocument: { uri }, position, ...(kind === "references" ? { context: { includeDeclaration: true } } : {}) };
  if (kind === "symbols") return semanticLocations(await client.request("workspace/symbol", { query: text }));
  if (kind === "outline") return semanticLocations(await client.request("textDocument/documentSymbol", { textDocument: { uri } }), uri);
  if (["callers","callees","supertypes","subtypes"].includes(kind)) {
    const followup=kind==="callers"?"callHierarchy/incomingCalls":kind==="callees"?"callHierarchy/outgoingCalls":`typeHierarchy/${kind}`;
    const items = await client.request(kind==="supertypes"||kind==="subtypes"?"textDocument/prepareTypeHierarchy":"textDocument/prepareCallHierarchy", params);
    if (!Array.isArray(items)) return [];
    if (items.length > 32) throw new Error("Too many candidate call targets. Choose a more specific symbol.");
    const answers = await Promise.all(items.map(item => client.request(followup, { item })));
    return semanticLocations(answers);
  }
  return semanticLocations(await client.request(`textDocument/${kind}`, params));
}
</ui:copy>

## Project the same model on Apple

The Swift model has no SwiftUI, editor or network dependency. It retains the same vocabulary and normalizes provider trees and call results into names and authored positions. The session owns request ordering and the workspace owns file opening. Traversal is bounded to 64 levels and 10,000 locations; the adapter labels bounded results.

<ui:copy id="simple-apple-semantic-locations">
import Foundation

enum SemanticQuery: String, CaseIterable, Identifiable {
    case definition, references, typeDefinition, implementation, declaration, outline, symbols, callers, callees, supertypes, subtypes
    var id: String { rawValue }
    var label: String {
        switch self {
        case .definition: return "Go to definition"
        case .references: return "Find usages"
        case .typeDefinition: return "Go to type definition"
        case .implementation: return "Find implementations"
        case .declaration: return "Go to declaration"
        case .outline: return "File outline"
        case .symbols: return "Search workspace symbols"
        case .callers: return "Find callers"
        case .callees: return "Find callees"
        case .supertypes: return "Find supertypes"
        case .subtypes: return "Find subtypes"
        }
    }
    var hierarchyFollowup:String? {
        switch self {
        case .callers:return "callHierarchy/incomingCalls"
        case .callees:return "callHierarchy/outgoingCalls"
        case .supertypes:return "typeHierarchy/supertypes"
        case .subtypes:return "typeHierarchy/subtypes"
        default:return nil
        }
    }
    var method: String {
        switch self {
        case .outline: return "textDocument/documentSymbol"
        case .symbols: return "workspace/symbol"
        case .callers, .callees: return "textDocument/prepareCallHierarchy"
        case .supertypes, .subtypes: return "textDocument/prepareTypeHierarchy"
        default: return "textDocument/" + rawValue
        }
    }
}
struct SemanticLocation: Identifiable, Equatable {
    var uri: String; var line: Int; var character: Int; var name: String? = nil; var kind: Int? = nil
    var id: String { "\(uri):\(line):\(character)" }
    var path: String? {
        guard uri.hasPrefix("hick:///"), let value = URLComponents(string: uri), value.scheme == "hick", value.host?.isEmpty != false,
              value.query == nil, value.fragment == nil,
              let decoded = value.percentEncodedPath.removingPercentEncoding, decoded.hasPrefix("/") else { return nil }
        let path = String(decoded.dropFirst()), parts = path.split(separator: "/", omittingEmptySubsequences: false)
        guard !path.isEmpty, !path.contains(":"), !path.contains("\\"), !path.unicodeScalars.contains(where: { $0.value < 32 }),
              parts.allSatisfy({ !$0.isEmpty && $0 != "." && $0 != ".." }) else { return nil }
        return path
    }
    var label: String { (name.map { $0 + " · " } ?? "") + "\(path ?? uri):\(line + 1):\(character + 1)" }
}
func semanticLocations(_ result: Any, defaultURI: String? = nil) -> [SemanticLocation] {
    var seen = Set<String>(), locations: [SemanticLocation] = []
    func visit(_ value: Any, _ depth: Int) {
        guard depth <= 64, locations.count < 10000 else { return }
        if let items = value as? [Any] { for item in items { visit(item, depth + 1) }; return }
        guard let item = value as? [String: Any] else { return }
        if let call = item["from"] ?? item["to"] { visit(call, depth + 1); return }
        let target = item["location"] as? [String: Any] ?? item
        let uri = (target["uri"] ?? target["targetUri"]) as? String ?? (item["name"] is String && item["location"] == nil ? defaultURI : nil)
        if let uri, let range = (target["selectionRange"] ?? target["range"] ?? target["targetSelectionRange"] ?? target["targetRange"]) as? [String: Any],
           let start = range["start"] as? [String: Any], let line = start["line"] as? Int, line >= 0,
           let character = start["character"] as? Int, character >= 0 {
            let location = SemanticLocation(uri: uri, line: line, character: character, name: item["name"] as? String, kind: (item["kind"] as? Int).flatMap {(1...26).contains($0) ? $0:nil})
            if seen.insert(location.id).inserted { locations.append(location) }
        }
        if let children = item["children"] { visit(children, depth + 1) }
    }
    visit(result, 0); return locations
}
</ui:copy>

## Project the same model on Android

The Kotlin model depends on JSON and URI decoding, not Compose or a text widget. It uses the same named-location identity, one-time path decoding, symbol-tree traversal and call-item normalization. Native sessions supply snapshots and workspaces supply navigation. This keeps renderer replacement independent of language-provider behavior.

<ui:copy id="simple-android-semantic-locations">
package com.hickorydocs.simple

import org.json.JSONObject
import org.json.JSONArray
import java.net.URI

enum class SemanticQuery(val method: String, val label: String) {
    Definition("textDocument/definition", "Go to definition"),
    References("textDocument/references", "Find usages"),
    TypeDefinition("textDocument/typeDefinition", "Go to type definition"),
    Implementation("textDocument/implementation", "Find implementations"),
    Declaration("textDocument/declaration", "Go to declaration"),
    Outline("textDocument/documentSymbol", "File outline"),
    Symbols("workspace/symbol", "Search workspace symbols"),
    Callers("textDocument/prepareCallHierarchy", "Find callers"),
    Callees("textDocument/prepareCallHierarchy", "Find callees"),
    Supertypes("textDocument/prepareTypeHierarchy", "Find supertypes"),
    Subtypes("textDocument/prepareTypeHierarchy", "Find subtypes");
    val hierarchyFollowup:String? get()=when(this) {
        Callers->"callHierarchy/incomingCalls"
        Callees->"callHierarchy/outgoingCalls"
        Supertypes->"typeHierarchy/supertypes"
        Subtypes->"typeHierarchy/subtypes"
        else->null
    }
}
data class SemanticLocation(val uri: String, val line: Int, val character: Int, val name: String? = null, val kind: Int? = null) {
    val path: String? get() = runCatching {
        val value = URI(uri)
        if (!uri.startsWith("hick:///") || value.scheme != "hick" || !value.authority.isNullOrEmpty() || value.query != null || value.fragment != null) return null
        val decoded = value.path ?: return null
        if (!decoded.startsWith("/")) return null
        val path = decoded.drop(1)
        if (path.isEmpty() || path.any { it == ':' || it == '\\' || it.code < 32 } || path.split('/').any { it.isEmpty() || it == "." || it == ".." }) null else path
    }.getOrNull()
    val label: String get() = (name?.let { "$it · " } ?: "") + "${path ?: uri}:${line + 1}:${character + 1}"
}
fun semanticLocations(result: Any, defaultURI: String? = null): List<SemanticLocation> {
    val locations = linkedMapOf<String, SemanticLocation>()
    fun visit(value: Any?, depth: Int) {
        if (depth > 64 || locations.size >= 10000) return
        if (value is JSONArray) { for (index in 0 until value.length()) visit(value.opt(index), depth + 1); return }
        val item = value as? JSONObject ?: return
        if (item.has("from") || item.has("to")) { visit(item.opt("from") ?: item.opt("to"), depth + 1); return }
        val target = item.optJSONObject("location") ?: item
        val uri = target.optString("uri").ifEmpty { target.optString("targetUri") }.ifEmpty { if (item.has("name") && !item.has("location")) defaultURI.orEmpty() else "" }
        val range = target.optJSONObject("selectionRange") ?: target.optJSONObject("range") ?: target.optJSONObject("targetSelectionRange") ?: target.optJSONObject("targetRange")
        val start = range?.optJSONObject("start")
        val line = start?.optInt("line", -1) ?: -1; val character = start?.optInt("character", -1) ?: -1
        if (uri.isNotEmpty() && line >= 0 && character >= 0) {
            val location = SemanticLocation(uri, line, character, item.optString("name").takeIf { it.isNotEmpty() },item.optInt("kind",-1).takeIf {it in 1..26})
            locations.putIfAbsent("$uri:$line:$character", location)
        }
        visit(item.opt("children"), depth + 1)
    }
    visit(result, 0); return locations.values.toList()
}
</ui:copy>

### `simple-intelligence-browser-trail`

Retain a bounded trail of actual source positions. A visit records departure and arrival, deduplicates identical positions, and discards the forward branch after a new visit. Back/forward updates the departure caret without changing document bytes. The workspace resolves each position against its current buffer; the trail neither opens files nor grants write authority.

<ui:copy id="simple-intelligence-browser-trail">
export class NavigationTrail {
  entries: SemanticLocation[] = [];
  index = -1;
  get canBack() { return this.index > 0; }
  get canForward() { return this.index >= 0 && this.index + 1 < this.entries.length; }
  private same(a: SemanticLocation | undefined, b: SemanticLocation) { return !!a && a.uri === b.uri && a.range.start.line === b.range.start.line && a.range.start.character === b.range.start.character; }
  private point(value: SemanticLocation): SemanticLocation { const start = { ...value.range.start }; return { uri: value.uri, range: { start, end: { ...start } } }; }
  record(from: SemanticLocation | null, to: SemanticLocation) {
    this.entries = this.entries.slice(0, this.index + 1);
    if (from && !this.same(this.entries.at(-1), from)) this.entries.push(this.point(from));
    if (!this.same(this.entries.at(-1), to)) this.entries.push(this.point(to));
    this.entries = this.entries.slice(-256); this.index = this.entries.length - 1;
  }
  step(direction: -1 | 1, current: SemanticLocation | null): SemanticLocation | null {
    const next = this.index + direction;
    if (next < 0 || next >= this.entries.length) return null;
    if (current && this.entries[this.index]?.uri === current.uri) this.entries[this.index] = this.point(current);
    this.index = next; return this.entries[next];
  }
}
</ui:copy>

### `simple-intelligence-apple-trail`

Retain a bounded trail of actual source positions. A visit records departure and arrival, deduplicates identical positions, and discards the forward branch after a new visit. Back/forward updates the departure caret without changing document bytes. The workspace resolves each position against its current buffer; the trail neither opens files nor grants write authority.

<ui:copy id="simple-intelligence-apple-trail">
struct NavigationTrail {
    private(set) var entries: [SemanticLocation] = []
    private(set) var index = -1
    var canBack: Bool { index > 0 }
    var canForward: Bool { index >= 0 && index + 1 < entries.count }
    mutating func record(from: SemanticLocation?, to: SemanticLocation) {
        entries = Array(entries.prefix(index + 1))
        if let from, entries.last?.id != from.id { entries.append(from) }
        if entries.last?.id != to.id { entries.append(to) }
        entries = Array(entries.suffix(256)); index = entries.count - 1
    }
    mutating func step(_ direction: Int, current: SemanticLocation?) -> SemanticLocation? {
        let next = index + direction
        guard (direction == -1 || direction == 1), entries.indices.contains(next) else { return nil }
        if let current, entries.indices.contains(index), entries[index].uri == current.uri { entries[index] = current }
        index = next; return entries[next]
    }
}
</ui:copy>

### `simple-intelligence-android-trail`

Retain a bounded trail of actual source positions. A visit records departure and arrival, deduplicates identical positions, and discards the forward branch after a new visit. Back/forward updates the departure caret without changing document bytes. The workspace resolves each position against its current buffer; the trail neither opens files nor grants write authority.

<ui:copy id="simple-intelligence-android-trail">
data class NavigationTrail(val entries: List<SemanticLocation> = emptyList(), val index: Int = -1) {
    val canBack get() = index > 0
    val canForward get() = index >= 0 && index + 1 < entries.size
    private fun same(a: SemanticLocation?, b: SemanticLocation) = a?.uri == b.uri && a.line == b.line && a.character == b.character
    fun record(from: SemanticLocation?, to: SemanticLocation): NavigationTrail {
        val values = entries.take(index + 1).toMutableList()
        if (from != null && !same(values.lastOrNull(), from)) values.add(from)
        if (!same(values.lastOrNull(), to)) values.add(to)
        val retained = values.takeLast(256); return NavigationTrail(retained, retained.lastIndex)
    }
    fun step(direction: Int, current: SemanticLocation?): Pair<NavigationTrail, SemanticLocation?> {
        val next = index + direction
        if (direction !in listOf(-1, 1) || next !in entries.indices) return this to null
        val values = entries.toMutableList()
        if (current != null && index in entries.indices && entries[index].uri == current.uri) values[index] = current
        return NavigationTrail(values, next) to values[next]
    }
}
</ui:copy>

### simple-intelligence-browser-problems

A reported problem is information from a language provider, not a verification verdict. Keep its message even when its source is stale or unversioned. Only a matching source snapshot and valid position contribute to next/previous navigation. Sorting and wraparound operate on source locations, independently of any popover or status bar.

<ui:copy id="simple-intelligence-browser-problems">
export interface WorkspaceProblem {
  id: string; path: string; message: string; severity: number;
  location: SemanticLocation | null; source: string | null; current: boolean;
}
export function reportedProblems(documents: readonly { uri: string; currentSource: string; source?: string; diagnostics: readonly { sourceSnapshot?: string; message: string; severity?: number; range: LspLocation["range"] }[] }[]): WorkspaceProblem[] {
  return documents.flatMap(document => {
    if (!document.diagnostics.length) return [];
    let rows: string[] | undefined;
    return document.diagnostics.map((diagnostic, index) => {
    const path = semanticPath(document.uri) ?? document.uri, position = diagnostic.range?.start;
    const source = diagnostic.sourceSnapshot ?? document.source, current = source !== undefined && source === document.currentSource;
    if (current) rows ??= document.currentSource.split("\n");
    const located = current && !!position && Number.isInteger(position.line) && position.line >= 0 && position.line < rows!.length && Number.isInteger(position.character) && position.character >= 0 && position.character <= rows![position.line].length && semanticPath(document.uri) !== null;
    return { id: `${document.uri}:${index}`, path, message: diagnostic.message, severity: diagnostic.severity ?? 1,
      location: located ? { uri: document.uri, range: diagnostic.range } : null, source: source ?? null, current: located };
    });
  });
}
export function nextWorkspaceProblem(problems: readonly WorkspaceProblem[], current: SemanticLocation | null, direction: -1 | 1): WorkspaceProblem | null {
  const compare = (a: SemanticLocation, b: SemanticLocation) => a.uri === b.uri ? a.range.start.line - b.range.start.line || a.range.start.character - b.range.start.character : a.uri < b.uri ? -1 : 1;
  const located = problems.filter(problem => problem.current && problem.location).sort((a, b) => compare(a.location!, b.location!));
  if (!located.length) return null;
  if (direction === 1) return (current ? located.find(problem => compare(problem.location!, current) > 0) : null) ?? located[0];
  return (current ? located.slice().reverse().find(problem => compare(problem.location!, current) < 0) : null) ?? located.at(-1)!;
}
</ui:copy>

### simple-intelligence-apple-problems

A reported problem is information from a language provider, not a verification verdict. Keep its message even when its source is stale or unversioned. Only a matching source snapshot and valid position contribute to next/previous navigation. Sorting and wraparound operate on source locations, independently of any popover or status bar.

<ui:copy id="simple-intelligence-apple-problems">
struct WorkspaceProblem: Identifiable {
    var id: String; var path: String; var message: String; var severity: Int
    var location: SemanticLocation?; var source: String?; var current: Bool
}
func nextWorkspaceProblem(_ problems: [WorkspaceProblem], current: SemanticLocation?, direction: Int) -> WorkspaceProblem? {
    guard direction == -1 || direction == 1 else { return nil }
    let located = problems.filter { $0.current && $0.location != nil }.sorted {
        let a = $0.location!, b = $1.location!; return (a.uri, a.line, a.character) < (b.uri, b.line, b.character)
    }
    guard !located.isEmpty else { return nil }
    if direction == 1 {
        return current.flatMap { point in located.first { let at = $0.location!; return (at.uri, at.line, at.character) > (point.uri, point.line, point.character) } } ?? located.first
    }
    return current.flatMap { point in located.last { let at = $0.location!; return (at.uri, at.line, at.character) < (point.uri, point.line, point.character) } } ?? located.last
}
</ui:copy>

### simple-intelligence-android-problems

A reported problem is information from a language provider, not a verification verdict. Keep its message even when its source is stale or unversioned. Only a matching source snapshot and valid position contribute to next/previous navigation. Sorting and wraparound operate on source locations, independently of any popover or status bar.

<ui:copy id="simple-intelligence-android-problems">
data class WorkspaceProblem(val id: String, val path: String, val message: String, val severity: Int, val location: SemanticLocation?, val source: String?, val current: Boolean)
fun nextWorkspaceProblem(problems: List<WorkspaceProblem>, current: SemanticLocation?, direction: Int): WorkspaceProblem? {
    if (direction !in listOf(-1, 1)) return null
    val order = compareBy<SemanticLocation>({ it.uri }, { it.line }, { it.character })
    val located = problems.filter { it.current && it.location != null }.sortedWith { a, b -> order.compare(a.location!!, b.location!!) }
    if (located.isEmpty()) return null
    if (direction == 1) return current?.let { point -> located.firstOrNull { order.compare(it.location!!, point) > 0 } } ?: located.first()
    return current?.let { point -> located.lastOrNull { order.compare(it.location!!, point) < 0 } } ?: located.last()
}
</ui:copy>

### simple-intelligence-browser-search

Search results are snapshots of saved workspace text. Keep every occurrence and its byte column, including generated-file ownership. Convert a byte column only at a Unicode scalar boundary; editors use UTF-16, so a character before the match may occupy two units. A result may reveal a precise position only after the open buffer matches the reported source hash. No search edits or saves source.

<ui:copy id="simple-intelligence-browser-search">
export function searchCharacter(line: string, byteColumn: number): number | null {
  if (!Number.isSafeInteger(byteColumn) || byteColumn < 0) return null;
  let bytes = 0, units = 0;
  for (const scalar of line) {
    if (bytes === byteColumn) return units;
    bytes += new TextEncoder().encode(scalar).length; units += scalar.length;
  }
  return bytes === byteColumn ? units : null;
}
</ui:copy>

### simple-intelligence-apple-search

Search results are snapshots of saved workspace text. Keep every occurrence and its byte column, including generated-file ownership. Convert a byte column only at a Unicode scalar boundary; editors use UTF-16, so a character before the match may occupy two units. A result may reveal a precise position only after the open buffer matches the reported source hash. No search edits or saves source.

<ui:copy id="simple-intelligence-apple-search">
struct WorkspaceSearchHit: Identifiable {
    var path: String; var line: Int; var text: String; var column: Int; var sourceHash: String?; var generatedBy: String?
    var id: String { "\(path):\(line):\(column)" }
    var character: Int? { searchCharacter(text, byteColumn: column) }
}
struct WorkspaceSearchResult { var hits: [WorkspaceSearchHit]; var truncated: Bool }
func searchCharacter(_ line: String, byteColumn: Int) -> Int? {
    guard byteColumn >= 0 else { return nil }
    var bytes = 0, units = 0
    for scalar in line.unicodeScalars {
        if bytes == byteColumn { return units }
        bytes += String(scalar).utf8.count; units += scalar.value > 0xffff ? 2 : 1
    }
    return bytes == byteColumn ? units : nil
}
</ui:copy>

### simple-intelligence-android-search

Search results are snapshots of saved workspace text. Keep every occurrence and its byte column, including generated-file ownership. Convert a byte column only at a Unicode scalar boundary; editors use UTF-16, so a character before the match may occupy two units. A result may reveal a precise position only after the open buffer matches the reported source hash. No search edits or saves source.

<ui:copy id="simple-intelligence-android-search">
data class WorkspaceSearchHit(val path: String, val line: Int, val text: String, val column: Int, val sourceHash: String?, val generatedBy: String?) {
    val character: Int? get() = searchCharacter(text, column)
}
data class WorkspaceSearchResult(val hits: List<WorkspaceSearchHit>, val truncated: Boolean)
fun searchCharacter(line: String, byteColumn: Int): Int? {
    if (byteColumn < 0) return null
    var bytes = 0; var units = 0
    while (units < line.length) {
        if (bytes == byteColumn) return units
        val scalar = line.codePointAt(units)
        bytes += when { scalar <= 0x7f -> 1; scalar <= 0x7ff -> 2; scalar <= 0xffff -> 3; else -> 4 }
        units += Character.charCount(scalar)
    }
    return if (bytes == byteColumn) units else null
}
</ui:copy>

### simple-intelligence-browser-analysis-scope

Background source discovery uses routing facts from the engine, not another frontend extension table. Enumerate every unique known-language authored file. Count generated outputs separately because their owners produce them; count unclassified files explicitly instead of treating them as analyzed. This is an input scope for language work, not evidence that a server has finished analysis. The visible Files subtree never narrows the retained workspace root.

<ui:copy id="simple-intelligence-browser-analysis-scope">
export function workspaceAnalysisScope(nodes: readonly import("../api/types").FileNode[]) {
  const files: import("../api/types").FileNode[] = [], seen = new Set<string>();
  let generated = 0, unclassified = 0;
  const visit = (nodes: readonly import("../api/types").FileNode[]) => { for (const file of nodes) {
    if (file.dir) { visit(file.children ?? []); continue; }
    if (seen.has(file.path)) continue; seen.add(file.path);
    if (file.generated_by) generated++; else if (file.language) files.push(file); else unclassified++;
  } };
  visit(nodes); files.sort((a,b) => a.path < b.path ? -1 : a.path > b.path ? 1 : 0);
  return { files, generated, unclassified };
}
</ui:copy>

### simple-intelligence-apple-analysis-scope

Background source discovery uses routing facts from the engine, not another frontend extension table. Enumerate every unique known-language authored file. Count generated outputs separately because their owners produce them; count unclassified files explicitly instead of treating them as analyzed. This is an input scope for language work, not evidence that a server has finished analysis. The visible Files subtree never narrows the retained workspace root.

<ui:copy id="simple-intelligence-apple-analysis-scope">
struct WorkspaceAnalysisScope { var files: [FileNode]; var generated: Int; var unclassified: Int }
func workspaceAnalysisScope(_ nodes: [FileNode]) -> WorkspaceAnalysisScope {
    var files: [FileNode] = [], seen = Set<String>(), generated = 0, unclassified = 0
    func visit(_ nodes: [FileNode]) { for file in nodes {
        if file.dir { visit(file.children ?? []); continue }
        guard seen.insert(file.path).inserted else { continue }
        if file.generated_by != nil { generated += 1 } else if let language = file.language, !language.isEmpty { files.append(file) } else { unclassified += 1 }
    } }
    visit(nodes); files.sort { $0.path < $1.path }
    return WorkspaceAnalysisScope(files:files,generated:generated,unclassified:unclassified)
}
</ui:copy>

### simple-intelligence-android-analysis-scope

Background source discovery uses routing facts from the engine, not another frontend extension table. Enumerate every unique known-language authored file. Count generated outputs separately because their owners produce them; count unclassified files explicitly instead of treating them as analyzed. This is an input scope for language work, not evidence that a server has finished analysis. The visible Files subtree never narrows the retained workspace root.

<ui:copy id="simple-intelligence-android-analysis-scope">
data class WorkspaceAnalysisScope(val files: List<FileNode>, val generated: Int, val unclassified: Int)
fun workspaceAnalysisScope(nodes: List<FileNode>): WorkspaceAnalysisScope {
    val files = mutableListOf<FileNode>(); val seen = mutableSetOf<String>(); var generated = 0; var unclassified = 0
    fun visit(nodes: List<FileNode>) { for(file in nodes) {
        if(file.dir) {visit(file.children);continue}
        if(!seen.add(file.path)) continue
        if(file.generatedBy != null) generated++ else if(!file.language.isNullOrEmpty()) files.add(file) else unclassified++
    } }
    visit(nodes);return WorkspaceAnalysisScope(files.sortedBy {it.path},generated,unclassified)
}
</ui:copy>

## Assemble the platform modules

Each platform imports the model through its ordinary module system. The maintained fragments stay source-backed; there is no second hand-maintained payload.

<ui:file path="apps/web/src/simple/Intelligence.ts"><ui:paste select="#simple-intelligence-browser-vocabulary"/><ui:paste select="#simple-intelligence-browser-locations"/><ui:paste select="#simple-intelligence-browser-query"/><ui:paste select="#simple-intelligence-browser-trail"/><ui:paste select="#simple-intelligence-browser-problems"/><ui:paste select="#simple-intelligence-browser-search"/><ui:paste select="#simple-intelligence-browser-analysis-scope"/></ui:file>
<ui:file path="apps/apple-simple/Sources/Intelligence.swift"><ui:paste select="#simple-apple-semantic-locations"/><ui:paste select="#simple-intelligence-apple-trail"/><ui:paste select="#simple-intelligence-apple-problems"/><ui:paste select="#simple-intelligence-apple-search"/><ui:paste select="#simple-intelligence-apple-analysis-scope"/></ui:file>
<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/Intelligence.kt"><ui:paste select="#simple-android-semantic-locations"/><ui:paste select="#simple-intelligence-android-trail"/><ui:paste select="#simple-intelligence-android-problems"/><ui:paste select="#simple-intelligence-android-search"/><ui:paste select="#simple-intelligence-android-analysis-scope"/></ui:file>

</ui:doc>
