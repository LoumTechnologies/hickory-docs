/*
 * Minimal test fixture: "Hello, native2wasm!" via raw syscall.
 *
 * Uses no C runtime — just a raw write(2) syscall and exit(2).
 * This compiles to a tiny static ELF on both x86_64 and ARM64.
 *
 * Compile:
 *   x86_64:  x86_64-linux-musl-gcc -static -nostdlib -Os -o hello_x86_64.elf hello.c
 *   ARM64:   aarch64-linux-musl-gcc -static -nostdlib -Os -o hello_arm64.elf hello.c
 */

#ifdef __x86_64__

static long syscall3(long num, long a, long b, long c) {
    long ret;
    __asm__ volatile (
        "syscall"
        : "=a"(ret)
        : "a"(num), "D"(a), "S"(b), "d"(c)
        : "rcx", "r11", "memory"
    );
    return ret;
}

static void my_exit(int code) {
    __asm__ volatile (
        "syscall"
        :
        : "a"(60), "D"(code)
        :
    );
    __builtin_unreachable();
}

#elif defined(__aarch64__)

static long syscall3(long num, long a, long b, long c) {
    register long x8 __asm__("x8") = num;
    register long x0 __asm__("x0") = a;
    register long x1 __asm__("x1") = b;
    register long x2 __asm__("x2") = c;
    __asm__ volatile (
        "svc #0"
        : "+r"(x0)
        : "r"(x8), "r"(x1), "r"(x2)
        : "memory"
    );
    return x0;
}

static void my_exit(int code) {
    register long x8 __asm__("x8") = 93;
    register long x0 __asm__("x0") = code;
    __asm__ volatile (
        "svc #0"
        :
        : "r"(x8), "r"(x0)
        :
    );
    __builtin_unreachable();
}

#else
#error "Unsupported architecture"
#endif

void _start(void) {
    const char msg[] = "Hello, native2wasm!\n";
    /* write(1, msg, sizeof(msg) - 1) */
    syscall3(1, 1, (long)msg, sizeof(msg) - 1);
    my_exit(0);
}
