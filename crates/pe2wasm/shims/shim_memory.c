/*
 * Memory-related shims: VirtualAlloc, VirtualFree.
 *
 * WASM linear memory does not support traditional virtual memory semantics.
 * We provide a simplified shim that uses __builtin_wasm_memory_grow
 * for allocation, and treats VirtualFree as a no-op.
 */

#include <stdint.h>

/* Win32 allocation type constants */
#define MEM_COMMIT      0x00001000
#define MEM_RESERVE     0x00002000
#define MEM_RELEASE     0x00008000

/* WASM page size is 64 KiB. */
#define WASM_PAGE_SIZE 65536

int64_t shim_virtual_alloc(void *addr, uint64_t size,
                            uint32_t alloc_type, uint32_t protect) {
    (void)addr;
    (void)protect;

    /* Only MEM_COMMIT and MEM_RESERVE are supported. */
    if (!(alloc_type & (MEM_COMMIT | MEM_RESERVE))) {
        return 0; /* NULL — allocation failed */
    }

    /* Grow linear memory. */
    uint32_t pages_needed = (uint32_t)((size + WASM_PAGE_SIZE - 1) / WASM_PAGE_SIZE);
    uint32_t old_pages = (uint32_t)__builtin_wasm_memory_grow(0, pages_needed);

    if (old_pages == (uint32_t)-1) {
        return 0; /* NULL — out of memory */
    }

    return (int64_t)(old_pages * WASM_PAGE_SIZE);
}

int shim_virtual_free(void *addr, uint64_t size, uint32_t free_type) {
    /* WASM linear memory cannot be freed. No-op. */
    (void)addr;
    (void)size;
    (void)free_type;
    return 1; /* Non-zero = success in Win32 convention */
}
