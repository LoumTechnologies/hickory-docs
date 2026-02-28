/*
 * Miscellaneous syscall stubs: sysctl, ioctl.
 *
 * These return -ENOSYS or provide minimal stub data
 * to satisfy common queries without full implementation.
 */

#include <stdint.h>
#include <string.h>

int shim_sysctl(const int *name, unsigned int namelen,
                void *oldp, uint64_t *oldlenp,
                const void *newp, uint64_t newlen) {
    (void)newp;
    (void)newlen;

    if (namelen < 2 || name == 0) {
        return -22; /* EINVAL */
    }

    /*
     * CTL_HW (6) / HW_NCPU (3): return 1 CPU.
     * This is commonly queried by runtime init code.
     */
    if (name[0] == 6 && name[1] == 3) {
        if (oldp && oldlenp && *oldlenp >= 4) {
            *(int *)oldp = 1;
            *oldlenp = 4;
            return 0;
        }
    }

    /*
     * CTL_HW (6) / HW_MEMSIZE (24): return 256 MiB.
     */
    if (name[0] == 6 && name[1] == 24) {
        if (oldp && oldlenp && *oldlenp >= 8) {
            *(uint64_t *)oldp = 256ULL * 1024 * 1024;
            *oldlenp = 8;
            return 0;
        }
    }

    /* Unknown sysctl — return ENOSYS rather than crashing. */
    return -38; /* ENOSYS */
}

int shim_ioctl(int fd, unsigned long request, void *arg) {
    (void)fd;
    (void)request;
    (void)arg;

    /* WASI has no ioctl equivalent. Stub returns ENOSYS. */
    return -38; /* ENOSYS */
}
