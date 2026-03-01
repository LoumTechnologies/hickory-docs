#ifndef LINUX_SYSCALL_H
#define LINUX_SYSCALL_H

/*
 * Linux x86_64 syscall numbers.
 *
 * On Linux x86_64, syscalls are invoked via the `syscall` instruction
 * with the syscall number in RAX. Arguments are passed in:
 *   rdi, rsi, rdx, r10, r8, r9
 *
 * These numbers are stable across kernel versions (part of the ABI).
 *
 * Reference: arch/x86/entry/syscalls/syscall_64.tbl
 */

#define SYS_LINUX_READ        0
#define SYS_LINUX_WRITE       1
#define SYS_LINUX_OPEN        2
#define SYS_LINUX_CLOSE       3
#define SYS_LINUX_STAT        4
#define SYS_LINUX_FSTAT       5
#define SYS_LINUX_LSTAT       6
#define SYS_LINUX_LSEEK       8
#define SYS_LINUX_MMAP        9
#define SYS_LINUX_MPROTECT   10
#define SYS_LINUX_MUNMAP     11
#define SYS_LINUX_BRK        12
#define SYS_LINUX_IOCTL      16
#define SYS_LINUX_ACCESS     21
#define SYS_LINUX_DUP        32
#define SYS_LINUX_DUP2       33
#define SYS_LINUX_GETPID     39
#define SYS_LINUX_FORK       57
#define SYS_LINUX_EXECVE     59
#define SYS_LINUX_EXIT       60
#define SYS_LINUX_UNAME      63
#define SYS_LINUX_FCNTL      72
#define SYS_LINUX_GETCWD     79
#define SYS_LINUX_CHDIR      80
#define SYS_LINUX_MKDIR      83
#define SYS_LINUX_RMDIR      84
#define SYS_LINUX_UNLINK     87
#define SYS_LINUX_CHMOD      90
#define SYS_LINUX_CHOWN      92
#define SYS_LINUX_GETUID     102
#define SYS_LINUX_GETGID     104
#define SYS_LINUX_GETEUID    107
#define SYS_LINUX_GETEGID    108
#define SYS_LINUX_GETPPID    110
#define SYS_LINUX_SYSINFO    99
#define SYS_LINUX_GETRUSAGE  98
#define SYS_LINUX_GETTIMEOFDAY 96
#define SYS_LINUX_WRITEV    20
#define SYS_LINUX_READV     19
#define SYS_LINUX_OPENAT   257
#define SYS_LINUX_EXIT_GROUP 231
#define SYS_LINUX_SET_TID_ADDRESS 218
#define SYS_LINUX_ARCH_PRCTL 158
#define SYS_LINUX_CLOCK_GETTIME 228

#endif /* LINUX_SYSCALL_H */
