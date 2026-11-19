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
export async function querySemanticLocations(client: LspClient, uri: string, position: { line: number; character: number }, kind: SemanticQuery, text = "", current = () => true): Promise<SemanticLocation[]> {
  const params = { textDocument: { uri }, position, ...(kind === "references" ? { context: { includeDeclaration: true } } : {}) };
  if (kind === "symbols") return semanticLocations(await client.request("workspace/symbol", { query: text }));
  if (kind === "outline") return semanticLocations(await client.request("textDocument/documentSymbol", { textDocument: { uri } }), uri);
  if (["callers","callees","supertypes","subtypes"].includes(kind)) {
    const followup=kind==="callers"?"callHierarchy/incomingCalls":kind==="callees"?"callHierarchy/outgoingCalls":`typeHierarchy/${kind}`;
    const items = await client.request(kind==="supertypes"||kind==="subtypes"?"textDocument/prepareTypeHierarchy":"textDocument/prepareCallHierarchy", params);
    if (!current() || !Array.isArray(items)) return [];
    if (items.length > 32) throw new Error("Too many hierarchy targets. Choose a more specific symbol.");
    const answers = await Promise.all(items.map(item => client.request(followup, { item })));
    return semanticLocations(answers);
  }
  return semanticLocations(await client.request(`textDocument/${kind}`, params));
}
export class NavigationTrail<T extends SemanticLocation = SemanticLocation> {
  entries: T[] = [];
  index = -1;
  get canBack() { return this.index > 0; }
  get canForward() { return this.index >= 0 && this.index + 1 < this.entries.length; }
  private same(a: SemanticLocation | undefined, b: SemanticLocation) { return !!a && a.uri === b.uri && a.range.start.line === b.range.start.line && a.range.start.character === b.range.start.character; }
  private point(value: T): T { const start = { ...value.range.start }; return { ...value, range: { start, end: { ...start } } }; }
  record(from: T | null, to: T) {
    this.entries = this.entries.slice(0, this.index + 1);
    for (const value of from ? [from, to] : [to]) {
      if (this.same(this.entries.at(-1), value)) this.entries[this.entries.length - 1] = this.point(value);
      else this.entries.push(this.point(value));
    }
    this.entries = this.entries.slice(-256); this.index = this.entries.length - 1;
  }
  prepareStep(direction: -1 | 1, departure: T | null) {
    const {entries, index} = this;
    const next = index + direction;
    if (index < 0 || next < 0 || next >= entries.length) return null;
    const current = () => this.entries === entries && this.index === index;
    return {
      location: entries[next],
      current,
      accept: () => {
        if (!current()) return false;
        this.entries = entries.slice();
        if (departure && entries[index].uri === departure.uri) this.entries[index] = this.point(departure);
        this.index = next;
        return true;
      },
    };
  }
  step(direction: -1 | 1, departure: T | null): T | null {
    const move = this.prepareStep(direction, departure);
    return move?.accept() ? move.location : null;
  }
}
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
export function searchCharacter(line: string, byteColumn: number): number | null {
  if (!Number.isSafeInteger(byteColumn) || byteColumn < 0) return null;
  let bytes = 0, units = 0;
  for (const scalar of line) {
    if (bytes === byteColumn) return units;
    bytes += new TextEncoder().encode(scalar).length; units += scalar.length;
  }
  return bytes === byteColumn ? units : null;
}
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
