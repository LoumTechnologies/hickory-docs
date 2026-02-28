#ifndef DARWIN_SYSCALL_H
#define DARWIN_SYSCALL_H

/*
 * Darwin/XNU BSD syscall numbers for x86_64.
 *
 * On macOS x86_64, BSD syscalls are invoked via `syscall` with
 * the syscall number OR'd with 0x2000000 (SYSCALL_CLASS_UNIX << 24).
 * The numbers below are the raw BSD syscall numbers (without the class prefix).
 *
 * Reference: xnu/bsd/kern/syscalls.master
 */

#define SYS_DARWIN_EXIT       1
#define SYS_DARWIN_FORK       2
#define SYS_DARWIN_READ       3
#define SYS_DARWIN_WRITE      4
#define SYS_DARWIN_OPEN       5
#define SYS_DARWIN_CLOSE      6
#define SYS_DARWIN_WAIT4      7
#define SYS_DARWIN_LINK       9
#define SYS_DARWIN_UNLINK    10
#define SYS_DARWIN_CHDIR     12
#define SYS_DARWIN_FCHDIR    13
#define SYS_DARWIN_MKNOD     14
#define SYS_DARWIN_CHMOD     15
#define SYS_DARWIN_CHOWN     16
#define SYS_DARWIN_GETPID    20
#define SYS_DARWIN_SETUID    23
#define SYS_DARWIN_GETUID    24
#define SYS_DARWIN_GETEUID   25
#define SYS_DARWIN_ACCESS    33
#define SYS_DARWIN_SYNC      36
#define SYS_DARWIN_KILL      37
#define SYS_DARWIN_GETPPID   39
#define SYS_DARWIN_DUP       41
#define SYS_DARWIN_PIPE      42
#define SYS_DARWIN_GETEGID   43
#define SYS_DARWIN_GETGID    47
#define SYS_DARWIN_IOCTL     54
#define SYS_DARWIN_SYMLINK   57
#define SYS_DARWIN_READLINK  58
#define SYS_DARWIN_EXECVE    59
#define SYS_DARWIN_UMASK     60
#define SYS_DARWIN_MUNMAP    73
#define SYS_DARWIN_FCNTL     92
#define SYS_DARWIN_FSYNC     95
#define SYS_DARWIN_GETDTABLESIZE 89
#define SYS_DARWIN_DUP2     90
#define SYS_DARWIN_SOCKET    97
#define SYS_DARWIN_CONNECT   98
#define SYS_DARWIN_BIND     104
#define SYS_DARWIN_SETSOCKOPT 105
#define SYS_DARWIN_LISTEN   106
#define SYS_DARWIN_GETTIMEOFDAY 116
#define SYS_DARWIN_GETRUSAGE 117
#define SYS_DARWIN_FSTAT    189
#define SYS_DARWIN_LSEEK    199
#define SYS_DARWIN_MMAP     197
#define SYS_DARWIN_STAT     338
#define SYS_DARWIN_SYSCTL   202
#define SYS_DARWIN_ISSETUGID 327

/* Syscall class prefix for BSD syscalls on x86_64. */
#define SYSCALL_CLASS_UNIX    0x2000000

#endif /* DARWIN_SYSCALL_H */
