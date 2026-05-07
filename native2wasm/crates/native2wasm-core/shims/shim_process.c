/*
 * Process-related syscall shims: exit, getpid.
 */

#include "wasi_helpers.h"

void shim_exit(int status) {
    __wasi_proc_exit((uint32_t)status);
    /* unreachable — __wasi_proc_exit is noreturn */
    __builtin_unreachable();
}

int shim_getpid(void) {
    /* WASI has no process identity concept. Return a stub PID. */
    return 42;
}
