#ifndef NT_SYSCALL_H
#define NT_SYSCALL_H

/*
 * Windows NT syscall numbers (x86_64, Windows 10 20H2).
 *
 * On Windows x86_64, NT syscalls are invoked via the `syscall` instruction
 * with the syscall number in EAX. These numbers are version-specific;
 * we target Windows 10 20H2 as the most common baseline.
 *
 * Reference: https://j00ru.vexillium.org/syscalls/nt/64/
 */

/* NT kernel syscalls */
#define NT_NtReadFile               0x0006
#define NT_NtWriteFile              0x0008
#define NT_NtClose                  0x000F
#define NT_NtQueryInformationFile   0x0011
#define NT_NtSetInformationFile     0x0027
#define NT_NtTerminateProcess       0x002C
#define NT_NtCreateFile             0x0055
#define NT_NtAllocateVirtualMemory  0x0018
#define NT_NtFreeVirtualMemory      0x001E
#define NT_NtQueryVirtualMemory     0x0023
#define NT_NtQuerySystemInformation 0x0036
#define NT_NtQueryPerformanceCounter 0x0031

/*
 * Win32 API function IDs for IAT-redirected calls.
 *
 * When the PE binary calls imported Win32 functions via the IAT,
 * the lifter redirects them to __pe2wasm_syscall with these IDs
 * (offset by 0x10000 to distinguish from NT syscall numbers).
 */
#define WIN32_BASE                  0x10000

#define WIN32_WriteFile             (WIN32_BASE + 0x0001)
#define WIN32_ReadFile              (WIN32_BASE + 0x0002)
#define WIN32_CreateFileA           (WIN32_BASE + 0x0003)
#define WIN32_CreateFileW           (WIN32_BASE + 0x0004)
#define WIN32_CloseHandle           (WIN32_BASE + 0x0005)
#define WIN32_SetFilePointer        (WIN32_BASE + 0x0006)
#define WIN32_ExitProcess           (WIN32_BASE + 0x0007)
#define WIN32_GetCurrentProcessId   (WIN32_BASE + 0x0008)
#define WIN32_VirtualAlloc          (WIN32_BASE + 0x0009)
#define WIN32_VirtualFree           (WIN32_BASE + 0x000A)
#define WIN32_GetStdHandle          (WIN32_BASE + 0x000B)
#define WIN32_GetLastError          (WIN32_BASE + 0x000C)
#define WIN32_SetLastError          (WIN32_BASE + 0x000D)

#endif /* NT_SYSCALL_H */
