/*
 * Linux x86_64 syscall dispatcher.
 *
 * The Remill lifter replaces x86_64 `syscall` instructions with calls to
 * `__elf2wasm_syscall(syscall_number, arg0..arg5)`. This function dispatches
 * to the appropriate WASI-backed shim.
 *
 * On Linux x86_64, the syscall number comes directly from RAX.
 * Arguments are in rdi, rsi, rdx, r10, r8, r9 → mapped to arg0..arg5.
 */

#include "linux_syscall.h"
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
extern int64_t  shim_writev(int fd, const void *iov, int iovcnt);
extern int64_t  shim_readv(int fd, const void *iov, int iovcnt);
extern int      shim_ioctl(int fd, unsigned long request, void *arg);
extern int64_t  shim_brk(void *addr);
extern int      shim_uname(void *buf);

int64_t __elf2wasm_syscall(uint64_t syscall_number,
                            uint64_t arg0, uint64_t arg1, uint64_t arg2,
                            uint64_t arg3, uint64_t arg4, uint64_t arg5) {
    switch ((uint32_t)syscall_number) {
        case SYS_LINUX_EXIT:
        case SYS_LINUX_EXIT_GROUP:
            shim_exit((int)arg0);
            return 0; /* unreachable */

        case SYS_LINUX_READ:
            return shim_read((int)arg0, (void *)(uintptr_t)arg1, arg2);

        case SYS_LINUX_WRITE:
            return shim_write((int)arg0, (const void *)(uintptr_t)arg1, arg2);

        case SYS_LINUX_OPEN:
            return shim_open((const char *)(uintptr_t)arg0, (int)arg1, (int)arg2);

        case SYS_LINUX_OPENAT:
            /*
             * openat(dirfd, pathname, flags, mode)
             * If dirfd == AT_FDCWD (-100), behaves like open().
             * We simplify and always use the pathname directly.
             */
            return shim_open((const char *)(uintptr_t)arg1, (int)arg2, (int)arg3);

        case SYS_LINUX_CLOSE:
            return shim_close((int)arg0);

        case SYS_LINUX_LSEEK:
            return shim_lseek((int)arg0, (int64_t)arg1, (int)arg2);

        case SYS_LINUX_GETPID:
        case SYS_LINUX_GETPPID:
            return shim_getpid();

        case SYS_LINUX_MMAP:
            return shim_mmap((void *)(uintptr_t)arg0, arg1,
                             (int)arg2, (int)arg3, (int)arg4, (int64_t)arg5);

        case SYS_LINUX_MUNMAP:
            return shim_munmap((void *)(uintptr_t)arg0, arg1);

        case SYS_LINUX_BRK:
            return shim_brk((void *)(uintptr_t)arg0);

        case SYS_LINUX_WRITEV:
            return shim_writev((int)arg0, (const void *)(uintptr_t)arg1, (int)arg2);

        case SYS_LINUX_READV:
            return shim_readv((int)arg0, (const void *)(uintptr_t)arg1, (int)arg2);

        case SYS_LINUX_IOCTL:
            return shim_ioctl((int)arg0, (unsigned long)arg1,
                              (void *)(uintptr_t)arg2);

        case SYS_LINUX_UNAME:
            return shim_uname((void *)(uintptr_t)arg0);

        case SYS_LINUX_GETUID:
        case SYS_LINUX_GETEUID:
            return 0; /* root */

        case SYS_LINUX_GETGID:
        case SYS_LINUX_GETEGID:
            return 0; /* root */

        case SYS_LINUX_MPROTECT:
            /* WASM has no memory protection. No-op, return success. */
            return 0;

        case SYS_LINUX_SET_TID_ADDRESS:
            /* Single-threaded stub. Return a fake TID. */
            return 42;

        case SYS_LINUX_ARCH_PRCTL:
            /* x86_64 arch_prctl — used for TLS setup.
             * WASM is single-threaded; stub returns success. */
            return 0;

        case SYS_LINUX_CLOCK_GETTIME:
            /* Stub: zero out the timespec and return success. */
            if (arg1) {
                uint8_t *ts = (uint8_t *)(uintptr_t)arg1;
                for (int i = 0; i < 16; i++) ts[i] = 0;
            }
            return 0;

        default:
            /* Unsupported syscall — return -ENOSYS (38). */
            return -38;
    }
}
