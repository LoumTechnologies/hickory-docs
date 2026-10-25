<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/editor/generated/hick-lang/hick_lang.d.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Browser-local prediction practice uses the shared gate](../../../../../../../contract.md)

Captured SHA-256: `afd96435fdcf71c7238398b8f606c55c077f3aa6032ae183d16abff3514733df`.

<book:entry path="apps/web/src/editor/generated/hick-lang/hick_lang.d.ts">
/* tslint:disable */
/* eslint-disable */

/**
 * Private browser-host transition. Actor and actual result arguments must be
 * supplied by trusted adapters, never forwarded from model invocation fields.
 * Storage is user-controlled, not a native private directory or authenticated
 * evidence store. Snapshot persistence must succeed before admitted dispatch.
 */
export function browser_prediction(snapshot: string, command: string): string;

/**
 * Pure complete-project evaluation. Input/output entries are byte arrays,
 * preserving binary assets and metadata; no browser filesystem is consulted.
 */
export function evaluate_project(entry: string, sources_json: string, input_json: string): string;

/**
 * The same exact-source selection used by native document launches.
 */
export function latest_debug_project(versions_json: string, displayed_json: string, path: string): string;

/**
 * Versioned non-executing materialization of literal files, including lineage.
 * Missing semantics are a diagnostic, never a successful partial program.
 */
export function literal_files(source: string): string;

export function project_position(local_json: string, endpoint_json: string, path: string, byte: number): string;

/**
 * One deterministic transition renderer for all comparison surfaces.
 */
export function project_transition(old_json: string, new_json: string): string;

/**
 * Exact shared patch regeneration; every downstream act is checked before returning source.
 */
export function regenerate_project_patch(entry: string, sources_json: string, input_json: string, id: string, new_json: string): string;

export function reverse_project_edit(entry: string, sources_json: string, input_json: string, endpoint_json: string, edit_json: string): string;

/**
 * The structure of a document — `hick_lang::Structure` as JSON.
 *
 * Offsets are byte offsets into `source`'s UTF-8; the caller converts them
 * to whatever its strings index by.
 */
export function structure(source: string): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly browser_prediction: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly evaluate_project: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly latest_debug_project: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly literal_files: (a: number, b: number, c: number) => void;
    readonly project_position: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
    readonly project_transition: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly regenerate_project_patch: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number) => void;
    readonly reverse_project_edit: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number) => void;
    readonly structure: (a: number, b: number, c: number) => void;
    readonly __wbindgen_export: (a: number) => void;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
    readonly __wbindgen_export2: (a: number, b: number) => number;
    readonly __wbindgen_export3: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_export4: (a: number, b: number, c: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
</book:entry>

</book:doc>
