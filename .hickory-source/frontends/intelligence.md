<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Ask source questions independently of an editor

For maintainers extending semantic intelligence while keeping browser, Apple and Android presentations replaceable.

An IDE question has a typed operation, an input source position and provider results. The model normalizes locations, named symbols and call items; the connection adapter captures source, selection and provider generation; the workspace owns buffers and navigation; the UI only offers the question and displays the answer. Swapping a gutter menu or native text editor does not change these contracts.

## Name the questions

Use a closed vocabulary for declarations, definitions, types, implementations, usages, file outlines, workspace symbols and call relationships. Display names are independent of LSP method selection. A workspace symbol query has text rather than a fabricated source position.

<ui:copy id="simple-intelligence-browser-vocabulary">
import type { LspLocation, LspClient } from "../lsp/client";
export type SemanticLocation = LspLocation & { name?: string };
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
] as const;
export type SemanticQuery = typeof semanticQueries[number]["kind"];
</ui:copy>

## Normalize provider identities once

Nested document symbols inherit the queried file. Flat symbols, LocationLinks and call items carry explicit identities. Decode URI path segments once and reject traversal, external authorities and control bytes. Preserve symbol names for reading, while navigation identity is the URI and UTF-16 position. Bound nesting and result counts without claiming missing relationships.

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
export function semanticLocations(result: unknown, defaultUri?: string): SemanticLocation[] {
  const seen = new Set<string>(), locations: SemanticLocation[] = [];
  const visit = (value: unknown, depth: number) => {
    if (depth > 64) throw new Error("Provider symbol nesting is too deep. Narrow the query.");
    if (Array.isArray(value)) { for (const child of value) visit(child, depth + 1); return; }
    if (!value || typeof value !== "object") return;
    const item = value as Record<string, unknown>;
    if (item.from || item.to) { visit(item.from ?? item.to, depth + 1); return; }
    const target = (item.location ?? item) as Record<string, unknown>;
    const uri = target.uri ?? target.targetUri ?? (typeof item.name === "string" && !item.location ? defaultUri : undefined);
    const range = (target.selectionRange ?? target.range ?? target.targetSelectionRange ?? target.targetRange) as LspLocation["range"] | undefined;
    const position = range?.start;
    if (typeof uri === "string" && position && Number.isSafeInteger(position.line) && position.line >= 0 && Number.isSafeInteger(position.character) && position.character >= 0) {
      const key = JSON.stringify([uri, position.line, position.character]);
      if (!seen.has(key)) { seen.add(key); locations.push({ uri, range: range!, ...(typeof item.name === "string" ? { name: item.name } : {}) }); }
      if (locations.length > 10000) throw new Error("More than 10,000 locations. Narrow the symbol query.");
    }
    if (item.children) visit(item.children, depth + 1);
  };
  visit(result, 0); return locations;
}
</ui:copy>

## Follow the provider protocol

Calls are two-phase reads: prepare candidate items, then ask for incoming or outgoing calls for those exact items. A broad ambiguous preparation is refused instead of silently choosing one symbol. Outlines and workspace symbols use their own protocol shapes; source-position reads share one path. No query applies an edit or saves a file.

<ui:copy id="simple-intelligence-browser-query">
export async function querySemanticLocations(client: LspClient, uri: string, position: { line: number; character: number }, kind: SemanticQuery, text = ""): Promise<SemanticLocation[]> {
  const params = { textDocument: { uri }, position, ...(kind === "references" ? { context: { includeDeclaration: true } } : {}) };
  if (kind === "symbols") return semanticLocations(await client.request("workspace/symbol", { query: text }));
  if (kind === "outline") return semanticLocations(await client.request("textDocument/documentSymbol", { textDocument: { uri } }), uri);
  if (kind === "callers" || kind === "callees") {
    const items = await client.request("textDocument/prepareCallHierarchy", params);
    if (!Array.isArray(items)) return [];
    if (items.length > 32) throw new Error("Too many candidate call targets. Choose a more specific symbol.");
    const answers = await Promise.all(items.map(item => client.request(kind === "callers" ? "callHierarchy/incomingCalls" : "callHierarchy/outgoingCalls", { item })));
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
    case definition, references, typeDefinition, implementation, declaration, outline, symbols, callers, callees
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
        }
    }
    var method: String {
        switch self {
        case .outline: return "textDocument/documentSymbol"
        case .symbols: return "workspace/symbol"
        case .callers, .callees: return "textDocument/prepareCallHierarchy"
        default: return "textDocument/" + rawValue
        }
    }
}
struct SemanticLocation: Identifiable, Equatable {
    var uri: String; var line: Int; var character: Int; var name: String? = nil
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
            let location = SemanticLocation(uri: uri, line: line, character: character, name: item["name"] as? String)
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
    Callees("textDocument/prepareCallHierarchy", "Find callees")
}
data class SemanticLocation(val uri: String, val line: Int, val character: Int, val name: String? = null) {
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
            val location = SemanticLocation(uri, line, character, item.optString("name").takeIf { it.isNotEmpty() })
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

## Assemble the platform modules

Each platform imports the model through its ordinary module system. The maintained fragments stay source-backed; there is no second hand-maintained payload.

<ui:file path="apps/web/src/simple/Intelligence.ts"><ui:paste select="#simple-intelligence-browser-vocabulary"/><ui:paste select="#simple-intelligence-browser-locations"/><ui:paste select="#simple-intelligence-browser-query"/><ui:paste select="#simple-intelligence-browser-trail"/></ui:file>
<ui:file path="apps/apple-simple/Sources/Intelligence.swift"><ui:paste select="#simple-apple-semantic-locations"/><ui:paste select="#simple-intelligence-apple-trail"/></ui:file>
<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/Intelligence.kt"><ui:paste select="#simple-android-semantic-locations"/><ui:paste select="#simple-intelligence-android-trail"/></ui:file>

</ui:doc>
