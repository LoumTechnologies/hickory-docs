#ifndef WASI_HELPERS_H
#define WASI_HELPERS_H

/*
 * WASI import declarations for use by NT syscall shims.
 *
 * These map to the WASI preview1 ABI which is imported by the WASM module
 * at runtime from the host (wasmtime, wasmer, etc.).
 *
 * We declare them as extern functions with the appropriate calling convention.
 * The actual imports are resolved by wasm-ld via --allow-undefined.
 */

#include <stdint.h>
#include <stddef.h>

/* WASI typedefs */
typedef uint32_t wasi_fd_t;
typedef uint16_t wasi_errno_t;
typedef uint64_t wasi_filesize_t;
typedef uint64_t wasi_filedelta_t;
typedef uint8_t  wasi_whence_t;

/* WASI iovec */
typedef struct {
    uint8_t *buf;
    uint32_t buf_len;
} wasi_iovec_t;

typedef struct {
    const uint8_t *buf;
    uint32_t buf_len;
} wasi_ciovec_t;

/* WASI error codes */
#define WASI_ERRNO_SUCCESS  0
#define WASI_ERRNO_BADF     8
#define WASI_ERRNO_INVAL   28
#define WASI_ERRNO_NOSYS   52
#define WASI_ERRNO_NOENT   44

/* WASI whence values */
#define WASI_WHENCE_SET  0
#define WASI_WHENCE_CUR  1
#define WASI_WHENCE_END  2

/*
 * WASI preview1 function imports.
 * These are provided by the WASI runtime.
 */

/* Process */
__attribute__((import_module("wasi_snapshot_preview1"), import_name("proc_exit")))
void __wasi_proc_exit(uint32_t rval) __attribute__((noreturn));

/* File I/O */
__attribute__((import_module("wasi_snapshot_preview1"), import_name("fd_read")))
wasi_errno_t __wasi_fd_read(wasi_fd_t fd, const wasi_iovec_t *iovs,
                            uint32_t iovs_len, uint32_t *nread);

__attribute__((import_module("wasi_snapshot_preview1"), import_name("fd_write")))
wasi_errno_t __wasi_fd_write(wasi_fd_t fd, const wasi_ciovec_t *iovs,
                             uint32_t iovs_len, uint32_t *nwritten);

__attribute__((import_module("wasi_snapshot_preview1"), import_name("fd_close")))
wasi_errno_t __wasi_fd_close(wasi_fd_t fd);

__attribute__((import_module("wasi_snapshot_preview1"), import_name("fd_seek")))
wasi_errno_t __wasi_fd_seek(wasi_fd_t fd, wasi_filedelta_t offset,
                            wasi_whence_t whence, wasi_filesize_t *newoffset);

__attribute__((import_module("wasi_snapshot_preview1"), import_name("path_open")))
wasi_errno_t __wasi_path_open(wasi_fd_t dirfd, uint32_t dirflags,
                              const char *path, uint32_t path_len,
                              uint32_t oflags, uint64_t fs_rights_base,
                              uint64_t fs_rights_inheriting,
                              uint32_t fdflags, wasi_fd_t *fd);

#endif /* WASI_HELPERS_H */
