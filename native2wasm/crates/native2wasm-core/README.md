# native2wasm-core

Shared AOT transpilation pipeline used by all format-specific transpilers.

## Key abstraction

`ParsedBinary` trait — covers code sections, data sections, symbols, entry
point, OS target, and architecture. Format-specific crates (`elf2wasm`,
`pe2wasm`, `mach2wasm`) implement this trait.

## Pipeline steps (via `run_pipeline`)

1. Remill lifting to LLVM IR
2. Optional `.ll` emission for debugging
3. OS shim compilation (via clang/wasm32-wasi)
4. Bitcode optimization
5. `llc` codegen to a wasm32 object file
6. `wasm-ld` link to a final `.wasm`

Shim sources (syscall headers and C files) are injected by each format-specific
caller via `PipelineConfig.os_headers` / `os_sources`.
