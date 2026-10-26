<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/editor/hickLang.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-1cd2ae06e1c5"></a>
## `apps/web/src/editor/hickLang.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The Editor Reads A Document With The Parser The Server Uses](../../../../../guarantees/language/the-editor-reads-with-the-parser-the-server-uses/contract.md), [Browser-local prediction practice uses the shared gate](../../../../../guarantees/verification/browser-predictions-use-the-shared-gate/contract.md).

<a id="f-1cd2ae06e1c5-RawTag"></a>
### `RawTag`

The hick parser, in the browser.
`hick-lang` — the parser `hick run` uses — compiled to WebAssembly
(`crates/hick-lang-wasm`, built into `generated/hick-lang` by
`just codegen`). The editor used to draw a document's structure from a
parser of its own, one regular expression, which disagreed with the real
one wherever the grammar is more than that: a rebound prefix
(`<h:doc xmlns:h="…">`), a verbatim element whose content quotes tags, a
document about hick's own syntax. Now there is one parser, and what the
editor draws is what the server reads.
The module must be loaded before anything parses: the parse itself is
synchronous (CodeMirror computes decorations synchronously), so the entry
points await `loadHickLang()` once before the first render, and the tests
load the bytes from disk in `test-setup.ts`.

<storycode:copy id="f-1cd2ae06e1c5-RawTag">
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

</storycode:copy>
<a id="f-1cd2ae06e1c5-RawBlock"></a>
### `RawBlock`

`hick_lang::StructureBlock` — byte offsets into the UTF-8 source.

<storycode:copy id="f-1cd2ae06e1c5-RawBlock">
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

</storycode:copy>
<a id="f-1cd2ae06e1c5-RawStructure"></a>
### `RawStructure`

<storycode:copy id="f-1cd2ae06e1c5-RawStructure">
/** `hick_lang::Structure`. */
export interface RawStructure {
  prefix: string;
  tags: RawTag[];
  blocks: RawBlock[];
  /** The first thing a strict parse would refuse, or null. */
  error: string | null;
}

</storycode:copy>
<a id="f-1cd2ae06e1c5-ready"></a>
### `ready`

<storycode:copy id="f-1cd2ae06e1c5-ready">
let ready = false;
</storycode:copy>
<a id="f-1cd2ae06e1c5-loading"></a>
### `loading`

<storycode:copy id="f-1cd2ae06e1c5-loading">
let loading: Promise<void> | null = null;

</storycode:copy>
<a id="f-1cd2ae06e1c5-loadHickLang"></a>
### `loadHickLang`

Load the parser. Idempotent; resolves at once after the first load.

<storycode:copy id="f-1cd2ae06e1c5-loadHickLang">
/** Load the parser. Idempotent; resolves at once after the first load. */
export async function loadHickLang(): Promise<void> {
  if (ready) return;
  loading ??= init({ module_or_path: wasmUrl }).then(() => { ready = true; }).catch((e) => { loading = null; throw e; });
  await loading;
}

</storycode:copy>
<a id="f-1cd2ae06e1c5-loadHickLangSync"></a>
### `loadHickLangSync`

Load the parser from bytes already in hand — the tests, which have a
filesystem and no fetch.

<storycode:copy id="f-1cd2ae06e1c5-loadHickLangSync">
/** Load the parser from bytes already in hand — the tests, which have a
 * filesystem and no fetch. */
export function loadHickLangSync(bytes: BufferSource): void {
  if (ready) return;
  initSync({ module: bytes });
  ready = true;
}

</storycode:copy>
<a id="f-1cd2ae06e1c5-hickLangReady"></a>
### `hickLangReady`

<storycode:copy id="f-1cd2ae06e1c5-hickLangReady">
export function hickLangReady(): boolean {
  return ready;
}

</storycode:copy>
<a id="f-1cd2ae06e1c5-rawStructure"></a>
### `rawStructure`

The structure of `text`, as `hick-lang` reads it. Offsets are BYTE
offsets; `hickDoc.ts` converts them to the UTF-16 indices JavaScript
strings and CodeMirror use.

<storycode:copy id="f-1cd2ae06e1c5-rawStructure">
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

</storycode:copy>
<a id="f-1cd2ae06e1c5-LiteralFile"></a>
### `LiteralFile`

<storycode:copy id="f-1cd2ae06e1c5-LiteralFile">
export interface LiteralFile {
  path: string;
  content: string;
  segments: { output: [number, number]; source: [number, number] }[];
}

</storycode:copy>
<a id="f-1cd2ae06e1c5-materializeLiteralFiles"></a>
### `materializeLiteralFiles`

<storycode:copy id="f-1cd2ae06e1c5-materializeLiteralFiles">
export function materializeLiteralFiles(source: string): LiteralFile[] {
  if (!ready) throw new Error("Load hick-lang before materializing files");
  const result = JSON.parse(literal_files(source)) as { version: number; files: LiteralFile[]; error: string | null };
  if (result.version !== 1) throw new Error("Unsupported portable document interface version");
  if (result.error) throw new Error(result.error);
  return result.files;
}


</storycode:copy>
<a id="f-1cd2ae06e1c5-ProjectEntry"></a>
### `ProjectEntry`

<storycode:copy id="f-1cd2ae06e1c5-ProjectEntry">
export interface ProjectEntry {
  path: string;
  bytes: number[];
  kind: "file" | "symlink";
  executable: boolean;
  origins?: { output: [number, number]; document: string; source: [number, number] }[];
}
</storycode:copy>
<a id="f-1cd2ae06e1c5-EvaluatedProject"></a>
### `EvaluatedProject`

<storycode:copy id="f-1cd2ae06e1c5-EvaluatedProject">
export interface EvaluatedProject {
  versions: ProjectEntry[][];
  old: ProjectEntry[];
  new: ProjectEntry[];
  inventory: { documents: string[]; exclude: string[] };
  local_states: Record<string, { old: ProjectEntry[]; new: ProjectEntry[] }>;
  scopes: Record<string, { old: ProjectEntry[]; new: ProjectEntry[] }>;
}
</storycode:copy>
<a id="f-1cd2ae06e1c5-materializeProject"></a>
### `materializeProject`

Pure source equivalence substrate; this never invokes native tools or cells.

<storycode:copy id="f-1cd2ae06e1c5-materializeProject">
/** Pure source equivalence substrate; this never invokes native tools or cells. */
export function materializeProject(entry: string, sources: Record<string, string>, input: ProjectEntry[] = []): EvaluatedProject {
  if (!ready) throw new Error("Load hick-lang before evaluating a project");
  const result = JSON.parse(evaluate_project(entry, JSON.stringify(sources), JSON.stringify(input))) as { version: number; project: EvaluatedProject | null; error: string | null };
  if (result.version !== 1) throw new Error("Unsupported portable project interface version");
  if (result.error || !result.project) throw new Error(result.error ?? "Project evaluation failed");
  return result.project;
}

</storycode:copy>
<a id="f-1cd2ae06e1c5-projectTransition"></a>
### `projectTransition`

<storycode:copy id="f-1cd2ae06e1c5-projectTransition">
export function projectTransition(old: ProjectEntry[], next: ProjectEntry[]): string {
  const result = JSON.parse(project_transition(JSON.stringify(old), JSON.stringify(next)));
  if (result.version !== 1 || result.error) throw new Error(result.error ?? "Unsupported project interface");
  return result.diff;
}
</storycode:copy>
<a id="f-1cd2ae06e1c5-regenerateProjectPatch"></a>
### `regenerateProjectPatch`

<storycode:copy id="f-1cd2ae06e1c5-regenerateProjectPatch">
export function regenerateProjectPatch(entry: string, sources: Record<string, string>, input: ProjectEntry[], id: string, next: ProjectEntry[]): Record<string, string> {
  const result = JSON.parse(regenerate_project_patch(entry, JSON.stringify(sources), JSON.stringify(input), id, JSON.stringify(next)));
  if (result.version !== 1 || result.error) throw new Error(result.error ?? "Unsupported project interface");
  return result.sources;
}

</storycode:copy>
<a id="f-1cd2ae06e1c5-reverseProjectEdit"></a>
### `reverseProjectEdit`

<storycode:copy id="f-1cd2ae06e1c5-reverseProjectEdit">
export function reverseProjectEdit(entry: string, sources: Record<string, string>, input: ProjectEntry[], endpoint: ProjectEntry[], edit: { path: string; range: [number, number]; replacement: string }): Record<string, string> {
  const result = JSON.parse(reverse_project_edit(entry, JSON.stringify(sources), JSON.stringify(input), JSON.stringify(endpoint), JSON.stringify(edit)));
  if (result.version !== 1 || result.error) throw new Error(result.error ?? "Unsupported project interface");
  return result.sources;
}

</storycode:copy>
<a id="f-1cd2ae06e1c5-projectPosition"></a>
### `projectPosition`

<storycode:copy id="f-1cd2ae06e1c5-projectPosition">
export function projectPosition(local: ProjectEntry[], endpoint: ProjectEntry[], path: string, byte: number): [string,number] | null {
  const result = JSON.parse(project_position(JSON.stringify(local),JSON.stringify(endpoint),path,byte));
  if (result.version !== 1 || result.error) throw new Error(result.error ?? "Unsupported project interface");
  return result.location;
}

</storycode:copy>
<a id="f-1cd2ae06e1c5-latestDebugProject"></a>
### `latestDebugProject`

Select a whole captured version; never patch the displayed file into another tree.

<storycode:copy id="f-1cd2ae06e1c5-latestDebugProject">
/** Select a whole captured version; never patch the displayed file into another tree. */
export function latestDebugProject(versions: ProjectEntry[][], displayed: ProjectEntry[], path: string): { project: ProjectEntry[]; path: string; latest: boolean } {
  const result = JSON.parse(latest_debug_project(JSON.stringify(versions), JSON.stringify(displayed), path));
  if (result.version !== 1 || result.error) throw new Error(result.error ?? "Unsupported project interface");
  const [index, target] = result.selection as [number, string];
  return { project: versions[index], path: target, latest: index === versions.length - 1 };
}

</storycode:copy>
<a id="f-1cd2ae06e1c5-browserPredictionTransition"></a>
### `browserPredictionTransition`

Private browser-host ledger transition; raw snapshots must never be UI readings.

<storycode:copy id="f-1cd2ae06e1c5-browserPredictionTransition">
/** Private browser-host ledger transition; raw snapshots must never be UI readings. */
export function browserPredictionTransition(snapshot: string, command: unknown): { snapshot: string | null; value: unknown; error: string | null } {
  if (!ready) throw Error("Load the shared prediction mechanism before use");
  return JSON.parse(browser_prediction(snapshot, JSON.stringify(command)));
}

</storycode:copy>
<a id="f-1cd2ae06e1c5-hickLangArtifactHash"></a>
### `hickLangArtifactHash`

Exact shipped parser/ledger WASM artifact, not a complete browser environment identity.

<storycode:copy id="f-1cd2ae06e1c5-hickLangArtifactHash">
/** Exact shipped parser/ledger WASM artifact, not a complete browser environment identity. */
export async function hickLangArtifactHash(): Promise<string> {
  const response = await fetch(wasmUrl);
  if (!response.ok) throw Error("Browser mechanism identity unavailable");
  return [...new Uint8Array(await crypto.subtle.digest("SHA-256", await response.arrayBuffer()))].map(byte => byte.toString(16).padStart(2, "0")).join("");
}
</storycode:copy>

### Assemble `apps/web/src/editor/hickLang.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/hickLang.ts"><storycode:paste select="#f-1cd2ae06e1c5-RawTag"/><storycode:paste select="#f-1cd2ae06e1c5-RawBlock"/><storycode:paste select="#f-1cd2ae06e1c5-RawStructure"/><storycode:paste select="#f-1cd2ae06e1c5-ready"/><storycode:paste select="#f-1cd2ae06e1c5-loading"/><storycode:paste select="#f-1cd2ae06e1c5-loadHickLang"/><storycode:paste select="#f-1cd2ae06e1c5-loadHickLangSync"/><storycode:paste select="#f-1cd2ae06e1c5-hickLangReady"/><storycode:paste select="#f-1cd2ae06e1c5-rawStructure"/><storycode:paste select="#f-1cd2ae06e1c5-LiteralFile"/><storycode:paste select="#f-1cd2ae06e1c5-materializeLiteralFiles"/><storycode:paste select="#f-1cd2ae06e1c5-ProjectEntry"/><storycode:paste select="#f-1cd2ae06e1c5-EvaluatedProject"/><storycode:paste select="#f-1cd2ae06e1c5-materializeProject"/><storycode:paste select="#f-1cd2ae06e1c5-projectTransition"/><storycode:paste select="#f-1cd2ae06e1c5-regenerateProjectPatch"/><storycode:paste select="#f-1cd2ae06e1c5-reverseProjectEdit"/><storycode:paste select="#f-1cd2ae06e1c5-projectPosition"/><storycode:paste select="#f-1cd2ae06e1c5-latestDebugProject"/><storycode:paste select="#f-1cd2ae06e1c5-browserPredictionTransition"/><storycode:paste select="#f-1cd2ae06e1c5-hickLangArtifactHash"/></storycode:file>

</storycode:doc>
