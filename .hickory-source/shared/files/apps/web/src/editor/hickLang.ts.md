<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/editor/hickLang.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [The Editor Reads A Document With The Parser The Server Uses](../../../../../../guarantees/language/the-editor-reads-with-the-parser-the-server-uses/contract.md)
- [Browser-local prediction practice uses the shared gate](../../../../../../guarantees/verification/browser-predictions-use-the-shared-gate/contract.md)

Captured SHA-256: `2921e22374ee79aa59db038f3e87d77156634b632650f16b43b7a7755f4293c7`.

<book:entry path="apps/web/src/editor/hickLang.ts">
// The hick parser, in the browser.
//
// `hick-lang` — the parser `hick run` uses — compiled to WebAssembly
// (`crates/hick-lang-wasm`, built into `generated/hick-lang` by
// `just codegen`). The editor used to draw a document's structure from a
// parser of its own, one regular expression, which disagreed with the real
// one wherever the grammar is more than that: a rebound prefix
// (`<h:doc xmlns:h="…">`), a verbatim element whose content quotes tags, a
// document about hick's own syntax. Now there is one parser, and what the
// editor draws is what the server reads.
//
// The module must be loaded before anything parses: the parse itself is
// synchronous (CodeMirror computes decorations synchronously), so the entry
// points await `loadHickLang()` once before the first render, and the tests
// load the bytes from disk in `test-setup.ts`.

import init, { browser_prediction, initSync, structure as wasmStructure, literal_files, evaluate_project, latest_debug_project, project_position, project_transition, regenerate_project_patch, reverse_project_edit } from "./generated/hick-lang/hick_lang.js";
import wasmUrl from "./generated/hick-lang/hick_lang_bg.wasm?url";

/** `hick_lang::StructureTag` — byte offsets into the UTF-8 source. */
export interface RawTag {
  name: string;
  from: number;
  to: number;
  closing: boolean;
  self_closing: boolean;
  attrs: [string, string][];
}

/** `hick_lang::StructureBlock` — byte offsets into the UTF-8 source. */
export interface RawBlock {
  name: string;
  from: number;
  to: number;
  content_from: number;
  content_to: number;
  closed: boolean;
  attrs: [string, string][];
}

/** `hick_lang::Structure`. */
export interface RawStructure {
  prefix: string;
  tags: RawTag[];
  blocks: RawBlock[];
  /** The first thing a strict parse would refuse, or null. */
  error: string | null;
}

let ready = false;
let loading: Promise<void> | null = null;

/** Load the parser. Idempotent; resolves at once after the first load. */
export async function loadHickLang(): Promise<void> {
  if (ready) return;
  loading ??= init({ module_or_path: wasmUrl }).then(() => { ready = true; }).catch((e) => { loading = null; throw e; });
  await loading;
}

/** Load the parser from bytes already in hand — the tests, which have a
 * filesystem and no fetch. */
export function loadHickLangSync(bytes: BufferSource): void {
  if (ready) return;
  initSync({ module: bytes });
  ready = true;
}

export function hickLangReady(): boolean {
  return ready;
}

/**
 * The structure of `text`, as `hick-lang` reads it. Offsets are BYTE
 * offsets; `hickDoc.ts` converts them to the UTF-16 indices JavaScript
 * strings and CodeMirror use.
 */
export function rawStructure(text: string): RawStructure {
  if (!ready) {
    throw new Error(
      "hick-lang is not loaded: await loadHickLang() (or call loadHickLangSync) before parsing a document",
    );
  }
  return JSON.parse(wasmStructure(text)) as RawStructure;
}

export interface LiteralFile {
  path: string;
  content: string;
  segments: { output: [number, number]; source: [number, number] }[];
}

export function materializeLiteralFiles(source: string): LiteralFile[] {
  if (!ready) throw new Error("Load hick-lang before materializing files");
  const result = JSON.parse(literal_files(source)) as { version: number; files: LiteralFile[]; error: string | null };
  if (result.version !== 1) throw new Error("Unsupported portable document interface version");
  if (result.error) throw new Error(result.error);
  return result.files;
}


export interface ProjectEntry {
  path: string;
  bytes: number[];
  kind: "file" | "symlink";
  executable: boolean;
  origins?: { output: [number, number]; document: string; source: [number, number] }[];
}
export interface EvaluatedProject {
  versions: ProjectEntry[][];
  old: ProjectEntry[];
  new: ProjectEntry[];
  inventory: { documents: string[]; exclude: string[] };
  local_states: Record<string, { old: ProjectEntry[]; new: ProjectEntry[] }>;
  scopes: Record<string, { old: ProjectEntry[]; new: ProjectEntry[] }>;
}
/** Pure source equivalence substrate; this never invokes native tools or cells. */
export function materializeProject(entry: string, sources: Record<string, string>, input: ProjectEntry[] = []): EvaluatedProject {
  if (!ready) throw new Error("Load hick-lang before evaluating a project");
  const result = JSON.parse(evaluate_project(entry, JSON.stringify(sources), JSON.stringify(input))) as { version: number; project: EvaluatedProject | null; error: string | null };
  if (result.version !== 1) throw new Error("Unsupported portable project interface version");
  if (result.error || !result.project) throw new Error(result.error ?? "Project evaluation failed");
  return result.project;
}

export function projectTransition(old: ProjectEntry[], next: ProjectEntry[]): string {
  const result = JSON.parse(project_transition(JSON.stringify(old), JSON.stringify(next)));
  if (result.version !== 1 || result.error) throw new Error(result.error ?? "Unsupported project interface");
  return result.diff;
}
export function regenerateProjectPatch(entry: string, sources: Record<string, string>, input: ProjectEntry[], id: string, next: ProjectEntry[]): Record<string, string> {
  const result = JSON.parse(regenerate_project_patch(entry, JSON.stringify(sources), JSON.stringify(input), id, JSON.stringify(next)));
  if (result.version !== 1 || result.error) throw new Error(result.error ?? "Unsupported project interface");
  return result.sources;
}

export function reverseProjectEdit(entry: string, sources: Record<string, string>, input: ProjectEntry[], endpoint: ProjectEntry[], edit: { path: string; range: [number, number]; replacement: string }): Record<string, string> {
  const result = JSON.parse(reverse_project_edit(entry, JSON.stringify(sources), JSON.stringify(input), JSON.stringify(endpoint), JSON.stringify(edit)));
  if (result.version !== 1 || result.error) throw new Error(result.error ?? "Unsupported project interface");
  return result.sources;
}

export function projectPosition(local: ProjectEntry[], endpoint: ProjectEntry[], path: string, byte: number): [string,number] | null {
  const result = JSON.parse(project_position(JSON.stringify(local),JSON.stringify(endpoint),path,byte));
  if (result.version !== 1 || result.error) throw new Error(result.error ?? "Unsupported project interface");
  return result.location;
}

/** Select a whole captured version; never patch the displayed file into another tree. */
export function latestDebugProject(versions: ProjectEntry[][], displayed: ProjectEntry[], path: string): { project: ProjectEntry[]; path: string; latest: boolean } {
  const result = JSON.parse(latest_debug_project(JSON.stringify(versions), JSON.stringify(displayed), path));
  if (result.version !== 1 || result.error) throw new Error(result.error ?? "Unsupported project interface");
  const [index, target] = result.selection as [number, string];
  return { project: versions[index], path: target, latest: index === versions.length - 1 };
}

/** Private browser-host ledger transition; raw snapshots must never be UI readings. */
export function browserPredictionTransition(snapshot: string, command: unknown): { snapshot: string | null; value: unknown; error: string | null } {
  if (!ready) throw Error("Load the shared prediction mechanism before use");
  return JSON.parse(browser_prediction(snapshot, JSON.stringify(command)));
}

/** Exact shipped parser/ledger WASM artifact, not a complete browser environment identity. */
export async function hickLangArtifactHash(): Promise<string> {
  const response = await fetch(wasmUrl);
  if (!response.ok) throw Error("Browser mechanism identity unavailable");
  return [...new Uint8Array(await crypto.subtle.digest("SHA-256", await response.arrayBuffer()))].map(byte => byte.toString(16).padStart(2, "0")).join("");
}
</book:entry>

</book:doc>
