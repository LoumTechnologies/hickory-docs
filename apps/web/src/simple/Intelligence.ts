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
export interface WorkspaceProblem {
  id: string; path: string; message: string; severity: number;
  location: SemanticLocation | null; source: string | null; current: boolean;
}
export function reportedProblems(documents: readonly { uri: string; currentSource: string; source?: string; diagnostics: readonly { message: string; severity?: number; range: LspLocation["range"] }[] }[]): WorkspaceProblem[] {
  return documents.flatMap(document => {
    if (!document.diagnostics.length) return [];
    const current = document.source !== undefined && document.source === document.currentSource;
    const rows = current ? document.currentSource.split("\n") : [];
    return document.diagnostics.map((diagnostic, index) => {
    const path = semanticPath(document.uri) ?? document.uri, position = diagnostic.range?.start;
    const located = current && !!position && Number.isInteger(position.line) && position.line >= 0 && position.line < rows.length && Number.isInteger(position.character) && position.character >= 0 && position.character <= rows[position.line].length && semanticPath(document.uri) !== null;
    return { id: `${document.uri}:${index}`, path, message: diagnostic.message, severity: diagnostic.severity ?? 1,
      location: located ? { uri: document.uri, range: diagnostic.range } : null, source: document.source ?? null, current: located };
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
