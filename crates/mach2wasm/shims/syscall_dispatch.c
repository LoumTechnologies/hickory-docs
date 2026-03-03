/*
 * Darwin BSD syscall dispatcher.
 *
 * The Remill lifter replaces x86_64 `syscall` instructions with calls to
 * `__mach2wasm_syscall(syscall_number, arg0..arg5)`. This function dispatches
 * to the appropriate WASI-backed shim.
 *
 * On macOS x86_64, the syscall number in RAX is OR'd with 0x2000000
 * (SYSCALL_CLASS_UNIX). We mask off the class prefix to get the BSD number.
 */

#include "darwin_syscall.h"
#include <stdint.h>

/* Shim functions implemented in other shim_*.c files. */
extern void     shim_exit(int status);
extern int64_t  shim_read(int fd, void *buf, uint64_t count);
extern int64_t  shim_write(int fd, const void *buf, uint64_t count);
extern int      shim_open(const char *path, int flags, int mode);
extern int      shim_close(int fd);
extern int      shim_getpid(void);
extern int64_t  shim_mmap(void *addr, uint64_t len, int prot, int flags, int fd, int64_t offset);
extern int      shim_munmap(void *addr, uint64_t len);
extern int64_t  shim_lseek(int fd, int64_t offset, int whence);
extern int64_t  shim_readv(int fd, const void *iov, int iovcnt);
extern int64_t  shim_writev(int fd, const void *iov, int iovcnt);
extern int64_t  shim_brk(void *addr);
extern int      shim_sysctl(const int *name, unsigned int namelen,
                            void *oldp, uint64_t *oldlenp,
                            const void *newp, uint64_t newlen);
extern int      shim_ioctl(int fd, unsigned long request, void *arg);

int64_t __mach2wasm_syscall(uint64_t syscall_number,
                            uint64_t arg0, uint64_t arg1, uint64_t arg2,
                            uint64_t arg3, uint64_t arg4, uint64_t arg5) {
    /* Strip the SYSCALL_CLASS_UNIX prefix if present. */
    uint32_t bsd_num = (uint32_t)(syscall_number & 0x00FFFFFF);

    switch (bsd_num) {
        case SYS_DARWIN_EXIT:
            shim_exit((int)arg0);
            return 0; /* unreachable */

        case SYS_DARWIN_READ:
            return shim_read((int)arg0, (void *)(uintptr_t)arg1, arg2);

        case SYS_DARWIN_WRITE:
            return shim_write((int)arg0, (const void *)(uintptr_t)arg1, arg2);

        case SYS_DARWIN_OPEN:
            return shim_open((const char *)(uintptr_t)arg0, (int)arg1, (int)arg2);

        case SYS_DARWIN_CLOSE:
            return shim_close((int)arg0);

        case SYS_DARWIN_GETPID:
            return shim_getpid();

        case SYS_DARWIN_MMAP:
            return shim_mmap((void *)(uintptr_t)arg0, arg1,
                             (int)arg2, (int)arg3, (int)arg4, (int64_t)arg5);

        case SYS_DARWIN_MUNMAP:
            return shim_munmap((void *)(uintptr_t)arg0, arg1);

        case SYS_DARWIN_LSEEK:
            return shim_lseek((int)arg0, (int64_t)arg1, (int)arg2);

        case SYS_DARWIN_READV:
            return shim_readv((int)arg0, (const void *)(uintptr_t)arg1, (int)arg2);

        case SYS_DARWIN_WRITEV:
            return shim_writev((int)arg0, (const void *)(uintptr_t)arg1, (int)arg2);

        case SYS_DARWIN_BRK:
            return shim_brk((void *)(uintptr_t)arg0);

        case SYS_DARWIN_SYSCTL:
            return shim_sysctl((const int *)(uintptr_t)arg0, (unsigned int)arg1,
                               (void *)(uintptr_t)arg2, (uint64_t *)(uintptr_t)arg3,
                               (const void *)(uintptr_t)arg4, arg5);

        case SYS_DARWIN_IOCTL:
            return shim_ioctl((int)arg0, (unsigned long)arg1,
                              (void *)(uintptr_t)arg2);

        default:
            /* Unsupported syscall — return -ENOSYS (38 on Darwin). */
            return -38;
    }
}
