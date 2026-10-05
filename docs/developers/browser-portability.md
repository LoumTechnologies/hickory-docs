# Browser portability inventory

This is the boundary established on 2026-10-04, not a claim that the full native
engine runs in WASM. The static fixture performs a real Rust operation beyond
parsing: it materializes unconditional literal files and maps their output bytes
back to the frozen document. The browser debugger consumes that result.

| Operation | Browser status | Shared implementation / next extraction |
| --- | --- | --- |
| Parse, source spans, structure, diagnostics | Working | `hick-lang` through `hick-lang-wasm` |
| Literal file bytes and source correspondence | Working, bounded subset | `hick-lang::literal_files`; native opening-break and dedent helpers |
| Editable document decorations | Working | Shared `EditingSurface`, instance-owned `EnvRegistry` |
| Local JS/TS debug | Working, ES5 execution subset | Worker interpreter, TypeScript emit/maps, existing debug client and gutter |
| Memory and IndexedDB files | Working | Revision-checked adapter, atomic snapshot, explicit save |
| Full non-executing weave | Requires extraction | `hick-literate::prepare_pipeline` and `run_pipeline_weave` still resolve paths/cache/run state |
| Render recorded results and never-run/stale cards | Requires extraction | `render::build_block_model` is registry-based; its fact types still depend on native crates |
| Transform fingerprints and standing | Requires extraction | Fingerprinting lives in `hick-lang`; handler context, resolution and rendering are not exposed by the browser facade |
| Full lineage and reverse edit planning | Requires extraction | `hickory-lineage::from_provenance_map` and `map_edits`; file application remains native |
| S3 workspace | Implemented against contract fixtures; no verified providers | Immutable objects/manifest, mandatory provider probe, conditional publication, persistent local outbox |
| Remote services / hybrid | Not implemented | Explicit auth, revision-bound requests, cancellation and reconnect before exposing native routes |
| Native shells, containers, git, terminals, agents, native LSP/DAP | Requires native host | No browser container or process runtime |

The literal operation refuses includes, upstream resolution, conditionals,
multiple declarations of a file, and nonliteral file children. It does not guess
at their output. Unrecognized source remains editable and round-trips unchanged;
preservation does not imply semantic support. Debugger results are ad hoc output,
not reusable verified cell evidence.

## Extraction tasks

1. Replace filesystem/include lookup in `prepare_pipeline` with a supplied
   document/asset snapshot. Carry named missing inputs and facts explicitly;
   preserve native selectors, prefix binding and conditional semantics.
2. Separate executor transcript/result structures from the process supervisors
   in `hickory-executor`. Supply frozen recordings, cache keys and missing/stale
   status to non-executing weave. Do not call an executor to display a recording.
3. Extract `BlockModelInput` and its facts without introducing a second registry.
   `hick-literate/src/render.rs` currently imports `hick_exec::FileContent`,
   executor transcripts, expectations and crate-local cell/status types.
4. Expose transform/check resolution and fingerprints through the same facade;
   supply referenced fragments, pin inputs and feature facts rather than reading
   a working tree or inventing TypeScript rules.
5. Separate provenance/edit planning from `apply_source_edits` filesystem writes.
   Return a revision-bound edit plan for the host to apply atomically.
6. Retire the landing demonstrations' TypeScript weave interpretation once the
   full shared operation is available. It is not used to derive debugger bytes.
   The existing intelligence demo's manual block lookup also needs replacement.
7. Expose completions and element actions as explicit host services; currently
   the portable surface uses syntax decorations and browser-debug extensions.
   The desktop `DocumentEditor` still owns its server and realtime wiring.
8. Expand the shared parity corpus from the current literal cases to copy/paste,
   includes, facts, transforms, sessions, cached/stale/never-run cells and reverse
   edits as each operation moves. Require native/browser byte comparisons.

A `cargo check -p hick-handlers --target wasm32-unknown-unknown` probe passed,
including `hick-flow` and `hick-exec`. This establishes that their handler/node
foundation compiles; it does not demonstrate browser execution of those paths.
No attempt to compile the full `hick-literate` dependency graph to WASM has been
made. The remaining native-layer blockers come from source inspection. The working
rows above have runtime/browser evidence, not compilation alone.

## Host boundary

`DocumentEmbed` imports editor code, parser assets and scoped styles. It does
not import `App`, `DocumentEditor`, workspace realtime, fleet or native routes.
The host supplies source, revision, optional asset resolution and extensions.
Selection positions are UTF-16; engine mappings are UTF-8 bytes. Instances own
their editor, environment registry and teardown. Parser initialization is shared
and idempotent. Optional debugger instances own independent disposable workers.

The external fixture and homepage tests block every `/api` request and reject
WebSocket construction. They exercise two editors, host replacement, read-only,
assets, unmount/remount, JS/TS stepping and watches, edited-source restart,
offline restart, exceptions, infinite-loop Stop, concurrent sessions, IndexedDB
reload/conflict, a cross-origin sandboxed iframe with keyboard editing, and
static assets below a nested deployment prefix with scoped CSP. S3 contract
flows test cross-origin CORS, conditional publication, offline queues and
explicit conflict resolution. The public entry is shared
source under `apps/web/src/embed`; there is no published package yet.

## Measurement baseline

Production assets measured on this checkout (Vite site build):

| Asset | Bytes | gzip bytes |
| --- | ---: | ---: |
| Parser/portable-operation WASM | 133,673 | 59,634 |
| Lazy debugger worker, including compiler | 3,937,572 | 1,080,045 |

The compiler dominates the debugger download. It loads on first Debug and
retains runner bytes for fresh offline workers. These are asset sizes, not the
whole homepage transfer. Host compression and cache policy determine transfer
size. There is no performance budget promised yet.

`embed/measure.ts` times initialization, structure and literal materialization
of a 73,782-byte Unicode document with 4,096 code lines. Each browser acceptance
run attaches its measurements to `test-results/browser-embedding.json` under
`portable-core-timings`. The measured run recorded:

| Browser | Parser load (ms) | Structure (ms) | Literal files (ms) |
| --- | ---: | ---: | ---: |
| Chromium | 24.5 | 5.4 | 2.4 |
| Firefox | 114 | 1 | 1 |
| WebKit | 71 | 11 | 4 |

Initialization can overlap other embeds; it is not an
isolated cold-load benchmark. The operations run on the UI thread today, so the
measurements guide a future asynchronous facade for larger documents.

IndexedDB durability is browser/origin policy, not a backup guarantee. Reload and
concurrent-tab conflicts are tested. Actual quota exhaustion, private-mode behavior,
crash recovery and real provider limits remain unmeasured. Complete workspace
import and injected quota errors have browser acceptance coverage.
