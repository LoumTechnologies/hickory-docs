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

import init, { initSync, structure as wasmStructure, literal_files } from "./generated/hick-lang/hick_lang.js";
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
