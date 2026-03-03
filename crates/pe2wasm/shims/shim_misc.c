/*
 * Miscellaneous Win32 API stubs: GetStdHandle, GetLastError, SetLastError.
 */

#include <stdint.h>

/* Win32 standard handle constants */
#define STD_INPUT_HANDLE  ((int32_t)-10)
#define STD_OUTPUT_HANDLE ((int32_t)-11)
#define STD_ERROR_HANDLE  ((int32_t)-12)

/* Thread-local errno stub.
 * WASM is single-threaded, so a simple global suffices. */
static uint32_t g_last_error = 0;

int shim_get_std_handle(int32_t nStdHandle) {
    switch (nStdHandle) {
        case STD_INPUT_HANDLE:  return 0; /* WASI stdin */
        case STD_OUTPUT_HANDLE: return 1; /* WASI stdout */
        case STD_ERROR_HANDLE:  return 2; /* WASI stderr */
        default:
            return -1; /* INVALID_HANDLE_VALUE */
    }
}

uint32_t shim_get_last_error(void) {
    return g_last_error;
}

void shim_set_last_error(uint32_t code) {
    g_last_error = code;
}
