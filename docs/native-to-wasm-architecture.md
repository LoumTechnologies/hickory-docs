# Native Binary → WASI WebAssembly AOT Transpiler Architecture

Reference architecture for ahead-of-time static binary translation from native executables to WASI WebAssembly modules. This document covers the shared approach used by `mach2wasm` (macOS Mach-O) and should be mirrored by `pe2wasm` (Windows PE) or any future platform transpiler.

## Why AOT Translation (Not Emulation)

The `crates/c2w/` crate converts OCI Linux containers to WASM by embedding a full CPU emulator (Bochs/TinyEMU) + Linux kernel — achieving ~1-3% of native performance. For macOS or Windows binaries, the equivalent approach would stack even more layers (Bochs + Linux + Darling/Wine), making it even slower.

AOT static binary translation eliminates all emulation layers:
- Parse the native binary format (Mach-O / PE)
- Lift x86_64 machine code → LLVM IR (via Remill)
- Compile LLVM IR → wasm32 (via llc)
- Link with OS syscall shims → standalone WASI module

This achieves 50-96% of native performance (per elfconv benchmarks for the analogous Linux/ELF case).

## Shared Pipeline

All platform transpilers follow the same 4-stage pipeline:

```
Native binary (Mach-O / PE / ELF)
       │
  ┌────┴─────────┐
  │ Binary Parser │  Parse format: segments, symbols, entry point, imports
  └────┬─────────┘
       │
  ┌────┴─────┐
  │  Remill  │  Lift x86_64 machine code → LLVM IR (C++ FFI)
  └────┬─────┘
       │
  ┌────┴─────┐
  │   llc    │  Compile LLVM IR → wasm32 object file
  └────┬─────┘
       │
  ┌────┴──────┐
  │  wasm-ld  │  Link lifted code + OS syscall shims → .wasm
  └────┬──────┘
       │
  standalone WASI module (runnable by wasmtime/wasmer/etc.)
```

## Crate Structure Pattern

Each transpiler crate follows identical layout:

```
crates/<name>/
├── Cargo.toml                    # Dual lib+bin, clap, goblin, anyhow, which, tempfile
├── build.rs                      # cmake bridge builder + remill stub fallback
├── cpp/
│   ├── CMakeLists.txt            # Build C++ bridge against pre-installed Remill
│   ├── remill_bridge.h           # C-compatible FFI header (SHARED across transpilers)
│   └── remill_bridge.cpp         # Wraps Remill TraceLifter API (SHARED)
├── shims/
│   ├── <os>_syscall.h            # OS syscall number constants
│   ├── wasi_helpers.h            # WASI import declarations (SHARED)
│   ├── syscall_dispatch.c        # switch(syscall_number) dispatcher
│   ├── shim_io.c                 # read/write/open/close/lseek → WASI
│   ├── shim_process.c            # exit/getpid stubs
│   ├── shim_memory.c             # mmap/VirtualAlloc → WASM linear memory
│   └── shim_misc.c               # OS-specific stubs
├── src/
│   ├── main.rs                   # CLI binary entrypoint
│   ├── lib.rs                    # Public API + pipeline orchestration
│   ├── cli.rs                    # clap derive CLI (same flags across transpilers)
│   ├── error.rs                  # thiserror typed errors
│   ├── <format>.rs               # Binary format parser (macho.rs / pe.rs)
│   ├── lifter.rs                 # Remill FFI bridge (SHARED Rust side)
│   ├── linker.rs                 # llc + wasm-ld invocation (SHARED)
│   └── syscall_shim.rs           # Shim compilation management
└── tests/
    ├── transpile_tests.rs
    └── fixtures/
```

## Shared vs Platform-Specific Modules

### Shared (identical or near-identical across transpilers)

| Module | Purpose |
|--------|---------|
| `cpp/remill_bridge.h` | C FFI header for Remill |
| `cpp/remill_bridge.cpp` | C++ wrapper around Remill TraceLifter |
| `cpp/CMakeLists.txt` | cmake build for bridge |
| `build.rs` | cmake bridge builder with stub fallback when Remill absent |
| `shims/wasi_helpers.h` | WASI preview1 import declarations |
| `src/lifter.rs` | Rust FFI bindings to Remill bridge |
| `src/linker.rs` | llc + wasm-ld invocation |
| `src/cli.rs` | CLI flags (input, output, -O, --emit-llvm, --verbose) |
| `src/lib.rs` | Pipeline orchestration (parse → lift → compile → link) |

### Platform-Specific

| Module | macOS (mach2wasm) | Windows (pe2wasm) |
|--------|-------------------|-------------------|
| Binary parser | `macho.rs` — goblin Mach-O | `pe.rs` — goblin PE |
| Syscall header | `darwin_syscall.h` — BSD syscall numbers | `nt_syscall.h` — NT syscall numbers |
| Syscall dispatch | BSD # dispatch (prefix `0x2000000`) | NT syscall dispatch or Win32 API thunks |
| I/O shims | `fd_read`/`fd_write` via WASI | Same WASI mapping |
| Memory shims | `mmap` → `memory.grow` | `VirtualAlloc` → `memory.grow` |
| Process shims | `exit`/`getpid` stubs | `ExitProcess`/`GetCurrentProcessId` stubs |
| Misc shims | `sysctl`/`ioctl` stubs | Registry/environment stubs |

## Binary Parser Output (Shared Type)

Both parsers produce the same intermediate representation:

```rust
pub struct ParsedBinary {
    pub bytes: Vec<u8>,
    pub arch: String,                       // "x86_64"
    pub entry_point: u64,
    pub code_sections: Vec<CodeSection>,     // .text or __TEXT,__text
    pub data_sections: Vec<DataSection>,     // .data/.bss or __DATA
    pub symbols: HashMap<u64, String>,
    pub imports: Vec<String>,               // dylib paths or DLL names
    pub is_static: bool,
}

pub struct CodeSection {
    pub name: String,
    pub segment: String,
    pub addr: u64,
    pub data: Vec<u8>,
}

pub struct DataSection {
    pub name: String,
    pub segment: String,
    pub addr: u64,
    pub data: Vec<u8>,
    pub writable: bool,
}
```

## Remill Integration (Shared)

Remill is **pre-installed on the system** (not vendored — it's 2GB+ with LLVM). The `build.rs` only compiles a thin C++ bridge.

**Prerequisites (same for all transpilers):**
- Remill installed (via its cmake superbuild)
- LLVM 17+ with WebAssembly target enabled
- `llc` and `wasm-ld` on PATH
- `clang` with `wasm32-wasi` target support
- `wasi-sysroot` (for libc)

**FFI contract** — the Rust side calls these C functions:
- `remill_context_create(semantics_dir)` → opaque context
- `remill_add_code_region(ctx, name, vaddr, bytes, size)` — register code
- `remill_add_data_section(ctx, name, vaddr, bytes, size, writable)` — register data
- `remill_set_entry_point(ctx, vaddr)` — mark `_start`
- `remill_lift(ctx)` → opaque LLVM module (or NULL + error)
- `remill_write_bc(module, path)` / `remill_write_ll(module, path)` — emit output

The bridge replaces `syscall` (macOS) / `syscall`/`int 0x2e` (Windows) instructions with calls to `__<name>_syscall()` (e.g., `__mach2wasm_syscall`, `__pe2wasm_syscall`).

## Syscall Shim Architecture

### Dispatch Pattern

The lifter replaces native syscall instructions with a call to a dispatch function:

```c
int64_t __<transpiler>_syscall(uint64_t syscall_number,
                               uint64_t arg0, uint64_t arg1, uint64_t arg2,
                               uint64_t arg3, uint64_t arg4, uint64_t arg5);
```

The dispatcher `switch`es on the syscall number and calls the appropriate shim function. Each shim maps native OS semantics to WASI preview1 imports.

### WASI Mapping Reference

| Operation | macOS (BSD syscall) | Windows (NT syscall / Win32) | WASI target |
|-----------|--------------------|-----------------------------|-------------|
| Write to fd/handle | `write(2)` / SYS 4 | `NtWriteFile` / `WriteFile` | `fd_write()` |
| Read from fd/handle | `read(2)` / SYS 3 | `NtReadFile` / `ReadFile` | `fd_read()` |
| Open file | `open(2)` / SYS 5 | `NtCreateFile` / `CreateFileW` | `path_open()` |
| Close fd/handle | `close(2)` / SYS 6 | `NtClose` / `CloseHandle` | `fd_close()` |
| Seek | `lseek(2)` / SYS 199 | `NtSetInformationFile` | `fd_seek()` |
| Exit | `exit(2)` / SYS 1 | `NtTerminateProcess` / `ExitProcess` | `proc_exit()` |
| Get PID | `getpid(2)` / SYS 20 | `GetCurrentProcessId` | stub (return 42) |
| Alloc memory | `mmap(2)` / SYS 197 | `NtAllocateVirtualMemory` / `VirtualAlloc` | `memory.grow` |
| Free memory | `munmap(2)` / SYS 73 | `NtFreeVirtualMemory` / `VirtualFree` | no-op |

### Shim Compilation

Shims are C source files compiled to `wasm32-wasi` object files via `clang --target=wasm32-wasi`. Sources are embedded in the Rust binary via `include_bytes!()` and written to a temp directory for compilation.

## Windows PE-Specific Considerations

### PE Parsing (goblin)

goblin's `PE` type provides:
- `header.coff_header.machine` — validate is `IMAGE_FILE_MACHINE_AMD64` (0x8664)
- `entry` — RVA of entry point (`AddressOfEntryPoint`)
- `sections` — iterate `.text`, `.data`, `.rdata`, `.bss`
- `imports` — DLL import directory (kernel32.dll, ntdll.dll, etc.)
- `exports` — export directory (for DLLs)

Key differences from Mach-O:
- PE uses RVA (Relative Virtual Address) from image base, not absolute virtual addresses
- Import Address Table (IAT) needs special handling — redirect to shim functions
- `.rdata` is read-only data (like `__DATA_CONST`)
- Thread Local Storage (TLS) directory may need stub support

### NT Syscall Layer

Windows has two syscall layers:
1. **NT syscalls** (`syscall` instruction, number in RAX) — low-level kernel interface
2. **Win32 API** (function calls to kernel32.dll, etc.) — high-level wrappers

For **statically-linked** binaries or binaries that use NT syscalls directly:
- Intercept `syscall` instruction, dispatch on NT syscall number
- NT numbers are **not stable across Windows versions** — use a version table or target a specific Windows version

For **dynamically-linked** binaries (the common case on Windows):
- Replace IAT entries with shim function pointers
- Implement Win32 API functions (WriteFile, ReadFile, etc.) that call WASI
- This is more practical than NT syscall interception for most PE binaries

### Recommended Win32 API Shims (Phase 0)

| Win32 API | Source DLL | WASI mapping |
|-----------|-----------|-------------|
| `WriteFile` | kernel32 | `fd_write()` |
| `ReadFile` | kernel32 | `fd_read()` |
| `CreateFileA/W` | kernel32 | `path_open()` |
| `CloseHandle` | kernel32 | `fd_close()` |
| `SetFilePointer` | kernel32 | `fd_seek()` |
| `ExitProcess` | kernel32 | `proc_exit()` |
| `GetCurrentProcessId` | kernel32 | stub |
| `VirtualAlloc` | kernel32 | `memory.grow` |
| `VirtualFree` | kernel32 | no-op |
| `GetStdHandle` | kernel32 | return 0/1/2 for stdin/stdout/stderr |
| `GetLastError` | kernel32 | thread-local errno stub |
| `SetLastError` | kernel32 | thread-local errno stub |

### Import Resolution Strategy

Instead of syscall interception, pe2wasm should:
1. Parse PE import directory to find all DLL imports
2. For each imported function, generate a WASM function that calls the appropriate shim
3. Patch the IAT in the lifted code to point to these generated functions
4. Unsupported imports return error codes or stub values

This is analogous to how Wine/ReactOS work, but at compile time rather than runtime.

## Phase Roadmap (Shared Pattern)

Each transpiler follows the same phasing:

### Phase 0: Static C binaries
- Parse binary, lift x86_64, minimal syscall shims, compile to WASM
- Target: simple statically-linked C CLI tools (hello world, cat, etc.)

### Phase 1: Dynamic libraries + core OS library
- **macOS:** Resolve `LC_LOAD_DYLIB`, compile Darling's `libSystem` to wasm32
- **Windows:** Resolve PE imports, implement kernel32/ntdll Win32 API shims

### Phase 2: Framework support
- **macOS:** CoreFoundation (CFString, CFArray, etc.)
- **Windows:** CRT (C Runtime), MSVCRT, possibly COM basics

### Phase 3: Advanced runtime
- **macOS:** Objective-C runtime (objc4), `objc_msgSend`
- **Windows:** C++ exception handling (SEH), possibly .NET P/Invoke stubs

## CLI Interface (Shared)

All transpilers use identical CLI flags:

```
<transpiler> <input> [-o output.wasm] [-O 0-3] [--emit-llvm] [--verbose]
                     [--semantics-dir <path>] [--wasi-sysroot <path>]
```

## Build System (Shared)

`build.rs` pattern:
1. Check if Remill is installed (`$REMILL_PREFIX/include/remill/Arch/Arch.h`)
2. If yes: build C++ bridge via cmake, link Remill + LLVM, set `cfg(has_remill)`
3. If no: compile C stub that returns NULL/errors, warn user
4. Either way, `cargo build` succeeds — Remill is only needed at runtime for lifting

## Testing Strategy (Shared)

1. **Unit tests** — parser rejects invalid input, CLI parses correctly, error types display
2. **Shim compilation** — verify shim C sources are embedded and compilable (needs clang)
3. **Integration tests** — end-to-end with fixture binaries (needs Remill + llc + wasm-ld)
4. **Workspace tests** — `cargo test --workspace -- --test-threads=1` must pass

## Reference Specifications

| Platform | Binary format spec | Syscall reference | Compatibility layer |
|----------|-------------------|-------------------|---------------------|
| macOS | Apple Mach-O Reference | XNU `bsd/kern/syscalls.master` | [Darling](https://darlinghq.org) |
| Windows | PE/COFF Specification (MS) | ReactOS syscall tables | [Wine](https://winehq.org) / [ReactOS](https://reactos.org) |
| Linux | ELF specification | Linux `arch/x86/entry/syscalls/syscall_64.tbl` | [elfconv](https://github.com/psvri/elfconv) |
