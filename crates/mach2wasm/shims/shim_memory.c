/*
 * Memory-related syscall shims: mmap, munmap.
 *
 * WASM linear memory does not support traditional mmap semantics.
 * We provide a simplified shim that uses __builtin_wasm_memory_grow
 * for anonymous MAP_ANON mappings, and returns -ENOSYS for file-backed mappings.
 */

#include <stdint.h>

/* Darwin mmap flags */
#define MAP_ANON  0x1000
#define MAP_FIXED 0x0010

/* WASM page size is 64 KiB. */
#define WASM_PAGE_SIZE 65536

int64_t shim_mmap(void *addr, uint64_t len, int prot, int flags,
                  int fd, int64_t offset) {
    (void)addr;
    (void)prot;
    (void)offset;

    /* Only anonymous mappings are supported. */
    if (!(flags & MAP_ANON) || fd != -1) {
        return -38; /* ENOSYS */
    }

    /* MAP_FIXED not supported in WASM. */
    if (flags & MAP_FIXED) {
        return -22; /* EINVAL */
    }

    /* Grow linear memory. */
    uint32_t pages_needed = (uint32_t)((len + WASM_PAGE_SIZE - 1) / WASM_PAGE_SIZE);
    uint32_t old_pages = (uint32_t)__builtin_wasm_memory_grow(0, pages_needed);

    if (old_pages == (uint32_t)-1) {
        return -12; /* ENOMEM */
    }

    return (int64_t)(old_pages * WASM_PAGE_SIZE);
}

int shim_munmap(void *addr, uint64_t len) {
    /* WASM linear memory cannot be freed. No-op. */
    (void)addr;
    (void)len;
    return 0;
}
