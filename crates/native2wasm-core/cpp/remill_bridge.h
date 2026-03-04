#ifndef REMILL_BRIDGE_H
#define REMILL_BRIDGE_H

/*
 * C-compatible FFI header for the Remill TraceLifter bridge.
 *
 * This exposes a minimal API for lifting x86_64 machine code to LLVM IR
 * using Remill's semantic library. Called from Rust via FFI.
 *
 * Runtime-parameterized: os_name and syscall_symbol are passed at context
 * creation rather than being compiled in, allowing a single bridge to
 * support Darwin, Windows, and Linux binaries.
 */

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>

/* Opaque handles. */
typedef void RemillContext;
typedef void RemillModule;

/*
 * Create a new Remill lifting context.
 * semantics_dir: path to directory containing Remill's .bc semantics files.
 * os_name: Remill OS name ("macos", "windows", "linux").
 * arch_name: Remill architecture name ("amd64", "aarch64").
 * syscall_symbol: name of the external syscall dispatch function
 *                 (e.g., "__mach2wasm_syscall").
 * Returns NULL on failure.
 */
RemillContext *remill_context_create(const char *semantics_dir,
                                     const char *os_name,
                                     const char *arch_name,
                                     const char *syscall_symbol);

/* Destroy a lifting context. */
void remill_context_destroy(RemillContext *ctx);

/* Register a code region (e.g., __TEXT,__text) to be lifted. */
void remill_add_code_region(RemillContext *ctx,
                            const char *name,
                            uint64_t vaddr,
                            const uint8_t *bytes,
                            uint64_t size);

/* Register a data section to be included as global data. */
void remill_add_data_section(RemillContext *ctx,
                             const char *name,
                             uint64_t vaddr,
                             const uint8_t *bytes,
                             uint64_t size,
                             int writable);

/* Set the entry point address (_start). */
void remill_set_entry_point(RemillContext *ctx, uint64_t vaddr);

/*
 * Lift all registered code regions to LLVM IR.
 * Returns NULL on failure; call remill_get_error() for details.
 */
RemillModule *remill_lift(RemillContext *ctx);

/* Destroy a lifted module. */
void remill_module_destroy(RemillModule *module);

/* Write lifted module as LLVM bitcode (.bc). Returns 0 on success. */
int remill_write_bc(RemillModule *module, const char *path);

/* Write lifted module as LLVM IR text (.ll). Returns 0 on success. */
int remill_write_ll(RemillModule *module, const char *path);

/* Get the last error message (NULL if no error). */
const char *remill_get_error(RemillContext *ctx);

#ifdef __cplusplus
}
#endif

#endif /* REMILL_BRIDGE_H */
