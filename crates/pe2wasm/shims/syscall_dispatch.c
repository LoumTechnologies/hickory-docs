/*
 * NT syscall and Win32 API dispatcher.
 *
 * The Remill lifter replaces x86_64 `syscall` instructions with calls to
 * `__pe2wasm_syscall(syscall_number, arg0..arg5)`. This function dispatches
 * to the appropriate WASI-backed shim.
 *
 * For NT kernel syscalls, the number comes directly from EAX.
 * For Win32 API calls redirected via the IAT, the number is offset
 * by WIN32_BASE (0x10000) to distinguish from raw NT syscall numbers.
 */

#include "nt_syscall.h"
#include <stdint.h>

/* Shim functions implemented in other shim_*.c files. */
extern void     shim_exit(int status);
extern int64_t  shim_read(int fd, void *buf, uint64_t count);
extern int64_t  shim_write(int fd, const void *buf, uint64_t count);
extern int      shim_open(const char *path, int flags, int mode);
extern int      shim_close(int fd);
extern int      shim_getpid(void);
extern int64_t  shim_virtual_alloc(void *addr, uint64_t size, uint32_t type, uint32_t protect);
extern int      shim_virtual_free(void *addr, uint64_t size, uint32_t type);
extern int64_t  shim_lseek(int fd, int64_t offset, int whence);
extern int      shim_get_std_handle(int32_t nStdHandle);
extern uint32_t shim_get_last_error(void);
extern void     shim_set_last_error(uint32_t code);

int64_t __pe2wasm_syscall(uint64_t syscall_number,
                           uint64_t arg0, uint64_t arg1, uint64_t arg2,
                           uint64_t arg3, uint64_t arg4, uint64_t arg5) {
    (void)arg5;

    switch ((uint32_t)syscall_number) {
        /* --- NT kernel syscalls --- */

        case NT_NtTerminateProcess:
            shim_exit((int)arg1);
            return 0; /* unreachable */

        case NT_NtReadFile:
            /* NtReadFile(Handle, Event, ApcRoutine, ApcContext, IoStatusBlock,
             *            Buffer, Length, ByteOffset, Key) */
            return shim_read((int)arg0, (void *)(uintptr_t)arg5, arg4);

        case NT_NtWriteFile:
            /* NtWriteFile(Handle, Event, ApcRoutine, ApcContext, IoStatusBlock,
             *             Buffer, Length, ByteOffset, Key) */
            return shim_write((int)arg0, (const void *)(uintptr_t)arg5, arg4);

        case NT_NtCreateFile:
            /* Simplified: map to open with defaults. */
            return shim_open((const char *)(uintptr_t)arg2, 0, 0);

        case NT_NtClose:
            return shim_close((int)arg0);

        case NT_NtAllocateVirtualMemory:
            return shim_virtual_alloc((void *)(uintptr_t)arg1, arg3,
                                      (uint32_t)arg4, (uint32_t)arg5);

        case NT_NtFreeVirtualMemory:
            return shim_virtual_free((void *)(uintptr_t)arg1, arg3, (uint32_t)arg4);

        /* --- Win32 API calls (IAT-redirected) --- */

        case WIN32_ExitProcess:
            shim_exit((int)arg0);
            return 0; /* unreachable */

        case WIN32_WriteFile:
            /* WriteFile(hFile, lpBuffer, nNumberOfBytesToWrite, ...) */
            return shim_write((int)arg0, (const void *)(uintptr_t)arg1, arg2);

        case WIN32_ReadFile:
            /* ReadFile(hFile, lpBuffer, nNumberOfBytesToRead, ...) */
            return shim_read((int)arg0, (void *)(uintptr_t)arg1, arg2);

        case WIN32_CreateFileA:
        case WIN32_CreateFileW:
            /* CreateFile(lpFileName, dwDesiredAccess, dwShareMode, ...) */
            return shim_open((const char *)(uintptr_t)arg0, (int)arg1, 0);

        case WIN32_CloseHandle:
            return shim_close((int)arg0);

        case WIN32_SetFilePointer:
            /* SetFilePointer(hFile, lDistanceToMove, lpDistanceToMoveHigh, dwMoveMethod) */
            return shim_lseek((int)arg0, (int64_t)arg1, (int)arg3);

        case WIN32_GetCurrentProcessId:
            return shim_getpid();

        case WIN32_VirtualAlloc:
            return shim_virtual_alloc((void *)(uintptr_t)arg0, arg1,
                                      (uint32_t)arg2, (uint32_t)arg3);

        case WIN32_VirtualFree:
            return shim_virtual_free((void *)(uintptr_t)arg0, arg1, (uint32_t)arg2);

        case WIN32_GetStdHandle:
            return shim_get_std_handle((int32_t)arg0);

        case WIN32_GetLastError:
            return (int64_t)shim_get_last_error();

        case WIN32_SetLastError:
            shim_set_last_error((uint32_t)arg0);
            return 0;

        default:
            /* Unsupported syscall — return STATUS_NOT_IMPLEMENTED (0xC0000002). */
            return -1;
    }
}
