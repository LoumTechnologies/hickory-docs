/*
 * Miscellaneous syscall stubs: ioctl, uname.
 *
 * These return -ENOSYS or provide minimal stub data
 * to satisfy common queries without full implementation.
 */

#include <stdint.h>
#include <string.h>

/*
 * struct utsname layout (65 bytes per field on Linux).
 * Total: 5 * 65 = 325 bytes, plus domainname = 390 bytes.
 */
#define UTSNAME_FIELD_LEN 65

int shim_uname(void *buf) {
    if (!buf) return -14; /* -EFAULT */

    char *p = (char *)buf;
    /* Zero the whole structure first. */
    memset(p, 0, UTSNAME_FIELD_LEN * 6);

    /* sysname */
    strncpy(p, "Linux", UTSNAME_FIELD_LEN - 1);
    p += UTSNAME_FIELD_LEN;

    /* nodename */
    strncpy(p, "wasm", UTSNAME_FIELD_LEN - 1);
    p += UTSNAME_FIELD_LEN;

    /* release */
    strncpy(p, "6.1.0-wasi", UTSNAME_FIELD_LEN - 1);
    p += UTSNAME_FIELD_LEN;

    /* version */
    strncpy(p, "#1 elf2wasm", UTSNAME_FIELD_LEN - 1);
    p += UTSNAME_FIELD_LEN;

    /* machine */
    strncpy(p, "wasm32", UTSNAME_FIELD_LEN - 1);
    /* p += UTSNAME_FIELD_LEN; */

    /* domainname left as zeros */

    return 0;
}

int shim_ioctl(int fd, unsigned long request, void *arg) {
    (void)fd;
    (void)request;
    (void)arg;

    /* WASI has no ioctl equivalent. Return -ENOSYS. */
    return -38; /* -ENOSYS */
}
