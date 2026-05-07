/*
 * Mach-O loader shim for native2wasm.
 *
 * This is a minimal static Linux x86_64 ELF that runs inside blink.
 * It loads a Mach-O binary from /guest/binary, maps its segments,
 * patches XNU syscalls to Linux equivalents, and jumps to the entry point.
 *
 * Compile: x86_64-linux-musl-gcc -static -Os -Wall -std=c11 -o macho_loader macho_loader.c
 */

#include <stdint.h>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/mman.h>
#include <fcntl.h>
#include <unistd.h>

/* ---------- Mach-O header structures (64-bit) ---------- */

#define MH_MAGIC_64  0xFEEDFACFu
#define CPU_TYPE_X86_64  0x01000007u

#define LC_SEGMENT_64  0x19u
#define LC_MAIN        0x80000028u

struct mach_header_64 {
    uint32_t magic;
    uint32_t cputype;
    uint32_t cpusubtype;
    uint32_t filetype;
    uint32_t ncmds;
    uint32_t sizeofcmds;
    uint32_t flags;
    uint32_t reserved;
};

struct load_command {
    uint32_t cmd;
    uint32_t cmdsize;
};

struct segment_command_64 {
    uint32_t cmd;
    uint32_t cmdsize;
    char     segname[16];
    uint64_t vmaddr;
    uint64_t vmsize;
    uint64_t fileoff;
    uint64_t filesize;
    uint32_t maxprot;
    uint32_t initprot;
    uint32_t nsects;
    uint32_t flags;
};

struct entry_point_command {
    uint32_t cmd;
    uint32_t cmdsize;
    uint64_t entryoff;
    uint64_t stacksize;
};

/* ---------- Syscall translation ---------- */

#define MAX_SYSCALL_MAP 512

struct syscall_entry {
    uint32_t xnu_num;
    uint32_t linux_num;
};

static struct syscall_entry g_syscall_map[MAX_SYSCALL_MAP];
static int g_syscall_count = 0;

static void load_syscall_map(void) {
    FILE *f = fopen("/guest/syscall_map.txt", "r");
    if (!f) return;

    char line[256];
    while (fgets(line, sizeof(line), f) && g_syscall_count < MAX_SYSCALL_MAP) {
        if (line[0] == '#' || line[0] == '\n') continue;

        uint32_t xnu, linux_num;
        char name[64];
        if (sscanf(line, "%u %u %63s", &xnu, &linux_num, name) >= 2) {
            g_syscall_map[g_syscall_count].xnu_num = xnu;
            g_syscall_map[g_syscall_count].linux_num = linux_num;
            g_syscall_count++;
        }
    }
    fclose(f);
}

static int find_linux_syscall(uint32_t xnu_num) {
    for (int i = 0; i < g_syscall_count; i++) {
        if (g_syscall_map[i].xnu_num == xnu_num)
            return (int)g_syscall_map[i].linux_num;
    }
    return -1; /* Not found */
}

/* ---------- Syscall patching ---------- */

/*
 * macOS x86_64 syscalls use: mov rax, 0x2000000|num; syscall
 * The mov rax,imm32 instruction is encoded as: B8 xx 00 00 02
 * (actually: B8 followed by a 4-byte little-endian immediate)
 *
 * We scan for this pattern and replace the immediate with the
 * Linux syscall number (which fits in the low bytes, high byte = 0).
 */
static void patch_syscalls(uint8_t *code, uint64_t size) {
    if (size < 5) return;

    for (uint64_t i = 0; i <= size - 5; i++) {
        /* Look for B8 xx 00 00 02 (mov eax, 0x020000xx) */
        if (code[i] == 0xB8 &&
            code[i + 2] == 0x00 &&
            code[i + 3] == 0x00 &&
            code[i + 4] == 0x02) {

            uint32_t xnu_num = code[i + 1]; /* Low byte is the BSD syscall number */
            int linux_num = find_linux_syscall(xnu_num);

            if (linux_num >= 0) {
                /* Rewrite to: mov eax, linux_num */
                code[i + 1] = (uint8_t)(linux_num & 0xFF);
                code[i + 2] = (uint8_t)((linux_num >> 8) & 0xFF);
                code[i + 3] = 0x00;
                code[i + 4] = 0x00;
            }
        }

        /* Also handle REX.W prefix: 48 C7 C0 xx 00 00 02 (mov rax, imm32 sign-extended) */
        if (i + 6 < size &&
            code[i] == 0x48 &&
            code[i + 1] == 0xC7 &&
            code[i + 2] == 0xC0 &&
            code[i + 4] == 0x00 &&
            code[i + 5] == 0x00 &&
            code[i + 6] == 0x02) {

            uint32_t xnu_num = code[i + 3];
            int linux_num = find_linux_syscall(xnu_num);

            if (linux_num >= 0) {
                code[i + 3] = (uint8_t)(linux_num & 0xFF);
                code[i + 4] = (uint8_t)((linux_num >> 8) & 0xFF);
                code[i + 5] = 0x00;
                code[i + 6] = 0x00;
            }
        }
    }
}

/* ---------- Main loader ---------- */

int main(int argc, char *argv[]) {
    const char *binary_path = "/guest/binary";
    if (argc > 1) binary_path = argv[1];

    /* Read the Mach-O file */
    int fd = open(binary_path, O_RDONLY);
    if (fd < 0) {
        fprintf(stderr, "macho_loader: cannot open %s\n", binary_path);
        return 1;
    }

    off_t file_size = lseek(fd, 0, SEEK_END);
    lseek(fd, 0, SEEK_SET);

    uint8_t *file_data = mmap(NULL, (size_t)file_size, PROT_READ, MAP_PRIVATE, fd, 0);
    if (file_data == MAP_FAILED) {
        fprintf(stderr, "macho_loader: mmap failed\n");
        close(fd);
        return 1;
    }
    close(fd);

    /* Parse Mach-O header */
    struct mach_header_64 *hdr = (struct mach_header_64 *)file_data;
    if (hdr->magic != MH_MAGIC_64) {
        fprintf(stderr, "macho_loader: not a 64-bit Mach-O (magic=0x%08x)\n", hdr->magic);
        return 1;
    }

    if (hdr->cputype != CPU_TYPE_X86_64) {
        fprintf(stderr, "macho_loader: not x86_64 (cputype=0x%08x)\n", hdr->cputype);
        return 1;
    }

    /* Load the syscall translation table */
    load_syscall_map();

    /* Walk load commands */
    uint64_t entry_offset = 0;
    uint64_t text_vmaddr = 0;
    uint8_t *cmd_ptr = file_data + sizeof(struct mach_header_64);

    for (uint32_t i = 0; i < hdr->ncmds; i++) {
        struct load_command *lc = (struct load_command *)cmd_ptr;

        if (lc->cmd == LC_SEGMENT_64) {
            struct segment_command_64 *seg = (struct segment_command_64 *)cmd_ptr;

            if (seg->vmsize == 0) {
                cmd_ptr += lc->cmdsize;
                continue;
            }

            /* Determine protection flags */
            int prot = PROT_READ;
            if (seg->initprot & 0x02) prot |= PROT_WRITE;
            if (seg->initprot & 0x04) prot |= PROT_EXEC;

            /* Map the segment */
            void *mapped = mmap(
                (void *)seg->vmaddr, seg->vmsize,
                PROT_READ | PROT_WRITE,
                MAP_PRIVATE | MAP_ANONYMOUS | MAP_FIXED,
                -1, 0
            );
            if (mapped == MAP_FAILED) {
                fprintf(stderr, "macho_loader: mmap segment %.16s at 0x%lx failed\n",
                        seg->segname, (unsigned long)seg->vmaddr);
                return 1;
            }

            /* Copy file data into segment */
            if (seg->filesize > 0) {
                memcpy(mapped, file_data + seg->fileoff, seg->filesize);
            }

            /* Track __TEXT for entry point calculation */
            if (memcmp(seg->segname, "__TEXT", 5) == 0 && seg->segname[5] == '\0') {
                text_vmaddr = seg->vmaddr;
            }

            /* Patch syscalls in executable segments */
            if (prot & PROT_EXEC) {
                patch_syscalls((uint8_t *)mapped, seg->vmsize);
            }

            /* Set final protection */
            if (prot != (PROT_READ | PROT_WRITE)) {
                mprotect(mapped, seg->vmsize, prot);
            }
        } else if (lc->cmd == LC_MAIN) {
            struct entry_point_command *ep = (struct entry_point_command *)cmd_ptr;
            entry_offset = ep->entryoff;
        }

        cmd_ptr += lc->cmdsize;
    }

    /* Unmap the file */
    munmap(file_data, (size_t)file_size);

    /* Jump to entry point */
    if (entry_offset == 0 && text_vmaddr == 0) {
        fprintf(stderr, "macho_loader: no entry point found\n");
        return 1;
    }

    uint64_t entry_addr = text_vmaddr + entry_offset;
    typedef int (*entry_fn)(int, char **);
    entry_fn entry = (entry_fn)entry_addr;

    return entry(argc, argv);
}
