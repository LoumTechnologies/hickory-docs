import { semanticPath } from "./Intelligence";
export interface SourceSnapshot { text: string; writable: boolean }
export interface SourceReplacement { from: number; to: number; text: string }
export interface SourcePatch { path: string; before: string; after: string; edits: SourceReplacement[] }
export interface ChangePlan { title: string; patches: SourcePatch[] }
export function sourceChangeOffset(source: string, position: unknown): number {
  const point = position as { line?: number; character?: number } | null;
  if (!Number.isSafeInteger(point?.line) || !Number.isSafeInteger(point?.character) || point!.line! < 0 || point!.character! < 0) throw new Error("Invalid UTF-16 source position");
  const rows = source.split("\n"), row = rows[point!.line!];
  if (row === undefined || point!.character! > row.length) throw new Error("Source edit lies outside its captured source");
  const result = rows.slice(0, point!.line!).reduce((sum, row) => sum + row.length + 1, 0) + point!.character!;
  if (result > 0 && result < source.length && /[\uD800-\uDBFF]/.test(source[result - 1]) && /[\uDC00-\uDFFF]/.test(source[result])) throw new Error("Source edit splits a Unicode scalar");
  return result;
}
interface WorkspaceEditTarget {path:string;items:unknown[]}
export function workspaceEditTargets(raw:unknown):WorkspaceEditTarget[] {
  if (!raw || typeof raw !== "object" || Array.isArray(raw)) throw new Error("No workspace edit to preview");
  const edit=raw as Record<string,unknown>,targets:WorkspaceEditTarget[]=[];
  const add=(uri:unknown,rawEdits:unknown,version?:unknown)=>{
    if(typeof uri!=="string"||!Array.isArray(rawEdits))throw new Error("Malformed workspace edit target");
    if(version!==undefined&&version!==null)throw new Error("Authored source version could not be established. Refresh the proposal.");
    const path=semanticPath(uri);if(!path)throw new Error(`Unsupported source identity: ${uri}`);
    targets.push({path,items:rawEdits});
  };
  if (edit.changes !== undefined) {
    if (!edit.changes || typeof edit.changes !== "object" || Array.isArray(edit.changes)) throw new Error("Malformed workspace changes");
    for (const [uri, items] of Object.entries(edit.changes)) add(uri, items);
  }
  if (edit.documentChanges !== undefined) {
    if (!Array.isArray(edit.documentChanges) || targets.length) throw new Error("Ambiguous workspace change representation");
    for (const value of edit.documentChanges) {
      const item = value as { kind?: unknown; textDocument?: { uri?: unknown; version?: unknown }; edits?: unknown } | null;
      if (!item || item.kind !== undefined) throw new Error("Resource create, move and delete require a filesystem proposal");
      add(item.textDocument?.uri, item.edits, item.textDocument?.version);
    }
  }
  return targets;
}
export function providerActionPaths(action:{data?:unknown;edit?:unknown}):string[] {
  const data=action.data as {hickory_acquire?:unknown}|undefined,raw=data?.hickory_acquire;
  if(raw===undefined)return workspaceEditTargets(action.edit).map(target=>target.path);
  if(!Array.isArray(raw)||!raw.length||raw.length>10000)throw new Error("Malformed provider source acquisition");
  return raw.map(item=>{const path=typeof item?.uri==="string"&&semanticPath(item.uri);if(!path)throw new Error("Unsupported provider acquisition source");return path;});
}
export function planWorkspaceEdit(title: string, raw: unknown, sources: ReadonlyMap<string, SourceSnapshot>,allowNoop=false): ChangePlan {
  const groups = new Map<string, SourceReplacement[]>();
  for(const {path,items:rawEdits} of workspaceEditTargets(raw)) {
    const source=sources.get(path);
    if(!source)throw new Error(`Acquire the affected source before preparing this proposal: ${path}`);
    if (!source.writable) throw new Error(`Source is generated, read-only or saving: ${path}`);
    const replacements = groups.get(path) ?? []; groups.set(path, replacements);
    for (const raw of rawEdits) {
      const item = raw as { range?: { start?: unknown; end?: unknown }; newText?: unknown } | null;
      if (!item?.range || typeof item.newText !== "string") throw new Error("Malformed source replacement");
      const replacement = { from: sourceChangeOffset(source.text, item.range.start), to: sourceChangeOffset(source.text, item.range.end), text: item.newText };
      if (replacement.to < replacement.from) throw new Error("Source replacement runs backwards");
      if (!replacements.some(other => other.from === replacement.from && other.to === replacement.to && other.text === replacement.text)) replacements.push(replacement);
      if (replacements.length > 10000) throw new Error("Proposal exceeds 10,000 replacements in one source");
    }
  }
  const patches = [...groups].sort(([a], [b]) => a < b ? -1 : a > b ? 1 : 0).flatMap(([path, edits]) => {
    edits.sort((a, b) => a.from - b.from || a.to - b.to);
    for (let i = 1; i < edits.length; i++) if (edits[i].from < edits[i - 1].to || edits[i].from === edits[i - 1].from) throw new Error(`Overlapping source edits: ${path}`);
    const before = sources.get(path)!.text;
    let after = before;
    for (const item of edits.slice().reverse()) after = after.slice(0, item.from) + item.text + after.slice(item.to);
    return before === after ? [] : [{ path, before, after, edits }];
  });
  if (!patches.length&&!allowNoop) throw new Error("The proposal contains no source changes");
  return { title, patches };
}
export function invertChangePlan(plan: ChangePlan): ChangePlan {
  return { title: `Undo ${plan.title}`, patches: plan.patches.map(patch => {
    let delta = 0;
    const edits = patch.edits.map(edit => { const from = edit.from + delta; delta += edit.text.length - (edit.to - edit.from); return { from, to: from + edit.text.length, text: patch.before.slice(edit.from, edit.to) }; });
    return { path: patch.path, before: patch.after, after: patch.before, edits };
  }) };
}
export function validateChangePlan(plan: ChangePlan, sources: ReadonlyMap<string, SourceSnapshot>): string | null {
  for (const patch of plan.patches) { const source = sources.get(patch.path); if (!source?.writable || source.text !== patch.before) return `Source changed, is unavailable, read-only or saving: ${patch.path}. Prepare a fresh proposal.`; }
  return null;
}
export function validateReferencedTargets(plan: ChangePlan, referencePaths: readonly string[]): void {
  const targets = new Set(plan.patches.map(patch => patch.path));
  const missing = [...new Set(referencePaths)].filter(path => !targets.has(path));
  if (missing.length) throw new Error(`Rename omitted reported reference source targets: ${missing.join(", ")}. Refresh project analysis and prepare again. No edits were applied.`);
}
