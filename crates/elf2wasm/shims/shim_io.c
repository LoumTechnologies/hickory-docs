/*
 * I/O shims: read, write, open, close, lseek, readv, writev → WASI.
 *
 * These map Linux I/O syscalls to WASI preview1 imports.
 * At the WASI level, I/O is identical regardless of source OS.
 */

#include "wasi_helpers.h"
#include <string.h>

/* Linux open flags */
#define O_RDONLY    0x0000
#define O_WRONLY    0x0001
#define O_RDWR      0x0002
#define O_CREAT     0x0040
#define O_TRUNC     0x0200
#define O_APPEND    0x0400

int64_t shim_read(int fd, void *buf, uint64_t count) {
    wasi_iovec_t iov = {
        .buf = (uint8_t *)buf,
        .buf_len = (uint32_t)count,
    };
    uint32_t nread = 0;
    wasi_errno_t err = __wasi_fd_read((wasi_fd_t)fd, &iov, 1, &nread);
    if (err != WASI_ERRNO_SUCCESS) {
        return -(int64_t)err;
    }
    return (int64_t)nread;
}

int64_t shim_write(int fd, const void *buf, uint64_t count) {
    wasi_ciovec_t iov = {
        .buf = (const uint8_t *)buf,
        .buf_len = (uint32_t)count,
    };
    uint32_t nwritten = 0;
    wasi_errno_t err = __wasi_fd_write((wasi_fd_t)fd, &iov, 1, &nwritten);
    if (err != WASI_ERRNO_SUCCESS) {
        return -(int64_t)err;
    }
    return (int64_t)nwritten;
}

int shim_open(const char *path, int flags, int mode) {
    (void)mode;

    /*
     * WASI path_open requires a pre-opened directory FD as the base.
     * FD 3 is conventionally the first pre-opened dir in WASI.
     * We use it as the base for all path operations.
     */
    wasi_fd_t preopened_dir = 3;
    uint32_t path_len = (uint32_t)strlen(path);

    /* Map Linux open flags to WASI oflags. */
    uint32_t oflags = 0;
    if (flags & O_CREAT) {
        oflags |= 1; /* WASI_OFLAGS_CREAT */
    }
    if (flags & O_TRUNC) {
        oflags |= 8; /* WASI_OFLAGS_TRUNC */
    }

    /* Map Linux access mode to WASI fdflags. */
    uint32_t fdflags = 0;
    if (flags & O_APPEND) {
        fdflags |= 1; /* WASI_FDFLAGS_APPEND */
    }

    /* Request broad rights — the runtime will enforce sandboxing. */
    uint64_t rights = 0xFFFFFFFFFFFFFFFFULL;

    wasi_fd_t opened_fd = 0;
    wasi_errno_t err = __wasi_path_open(
        preopened_dir, 1 /* SYMLINK_FOLLOW */,
        path, path_len,
        oflags, rights, rights, fdflags, &opened_fd);

    if (err != WASI_ERRNO_SUCCESS) {
        return -(int)err;
    }
    return (int)opened_fd;
}

int shim_close(int fd) {
    wasi_errno_t err = __wasi_fd_close((wasi_fd_t)fd);
    if (err != WASI_ERRNO_SUCCESS) {
        return -(int)err;
    }
    return 0;
}

int64_t shim_lseek(int fd, int64_t offset, int whence) {
    wasi_whence_t wasi_whence;
    switch (whence) {
        case 0: wasi_whence = WASI_WHENCE_SET; break; /* SEEK_SET */
        case 1: wasi_whence = WASI_WHENCE_CUR; break; /* SEEK_CUR */
        case 2: wasi_whence = WASI_WHENCE_END; break; /* SEEK_END */
        default: return -WASI_ERRNO_INVAL;
    }

    wasi_filesize_t newoffset = 0;
    wasi_errno_t err = __wasi_fd_seek((wasi_fd_t)fd, offset, wasi_whence, &newoffset);
    if (err != WASI_ERRNO_SUCCESS) {
        return -(int64_t)err;
    }
    return (int64_t)newoffset;
}

/*
 * Linux iovec structure (matches kernel's struct iovec).
 * On x86_64: iov_base is 8 bytes, iov_len is 8 bytes.
 * In WASM32, pointers are 4 bytes, so we need to handle the layout difference.
 */
struct linux_iovec {
    void *iov_base;
    uint64_t iov_len;
};

int64_t shim_writev(int fd, const void *iov_ptr, int iovcnt) {
    if (iovcnt <= 0) return 0;

    /* Process each iovec individually via WASI fd_write. */
    const struct linux_iovec *iovs = (const struct linux_iovec *)iov_ptr;
    int64_t total = 0;

    for (int i = 0; i < iovcnt; i++) {
        if (iovs[i].iov_len == 0) continue;

        wasi_ciovec_t ciov = {
            .buf = (const uint8_t *)iovs[i].iov_base,
            .buf_len = (uint32_t)iovs[i].iov_len,
        };
        uint32_t nwritten = 0;
        wasi_errno_t err = __wasi_fd_write((wasi_fd_t)fd, &ciov, 1, &nwritten);
        if (err != WASI_ERRNO_SUCCESS) {
            return total > 0 ? total : -(int64_t)err;
        }
        total += nwritten;
    }

    return total;
}

int64_t shim_readv(int fd, const void *iov_ptr, int iovcnt) {
    if (iovcnt <= 0) return 0;

    const struct linux_iovec *iovs = (const struct linux_iovec *)iov_ptr;
    int64_t total = 0;

    for (int i = 0; i < iovcnt; i++) {
        if (iovs[i].iov_len == 0) continue;

        wasi_iovec_t iov = {
            .buf = (uint8_t *)iovs[i].iov_base,
            .buf_len = (uint32_t)iovs[i].iov_len,
        };
        uint32_t nread = 0;
        wasi_errno_t err = __wasi_fd_read((wasi_fd_t)fd, &iov, 1, &nread);
        if (err != WASI_ERRNO_SUCCESS) {
            return total > 0 ? total : -(int64_t)err;
        }
        total += nread;
        if (nread < iovs[i].iov_len) break; /* short read */
    }

    return total;
}
