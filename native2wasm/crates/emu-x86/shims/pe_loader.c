/*
 * PE loader shim for native2wasm.
 *
 * This is a minimal static Linux x86_64 ELF that runs inside blink.
 * It loads a PE binary from /guest/binary, maps its sections,
 * resolves the IAT with Linux syscall thunks, and jumps to the entry point.
 *
 * Compile: x86_64-linux-musl-gcc -static -Os -Wall -std=c11 -o pe_loader pe_loader.c
 */

#include <stdint.h>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/mman.h>
#include <fcntl.h>
#include <unistd.h>

/* ---------- PE structures ---------- */

#define PE_SIGNATURE  0x00004550u  /* "PE\0\0" */
#define IMAGE_FILE_MACHINE_AMD64  0x8664u

struct dos_header {
    uint16_t e_magic;     /* MZ */
    uint8_t  padding[58];
    uint32_t e_lfanew;    /* Offset to PE signature */
};

struct coff_header {
    uint16_t machine;
    uint16_t number_of_sections;
    uint32_t time_date_stamp;
    uint32_t pointer_to_symbol_table;
    uint32_t number_of_symbols;
    uint16_t size_of_optional_header;
    uint16_t characteristics;
};

struct optional_header_64 {
    uint16_t magic;                   /* 0x20B for PE32+ */
    uint8_t  major_linker_version;
    uint8_t  minor_linker_version;
    uint32_t size_of_code;
    uint32_t size_of_initialized_data;
    uint32_t size_of_uninitialized_data;
    uint32_t address_of_entry_point;
    uint32_t base_of_code;
    uint64_t image_base;
    uint32_t section_alignment;
    uint32_t file_alignment;
    uint16_t major_os_version;
    uint16_t minor_os_version;
    uint16_t major_image_version;
    uint16_t minor_image_version;
    uint16_t major_subsystem_version;
    uint16_t minor_subsystem_version;
    uint32_t win32_version_value;
    uint32_t size_of_image;
    uint32_t size_of_headers;
    uint32_t checksum;
    uint16_t subsystem;
    uint16_t dll_characteristics;
    uint64_t size_of_stack_reserve;
    uint64_t size_of_stack_commit;
    uint64_t size_of_heap_reserve;
    uint64_t size_of_heap_commit;
    uint32_t loader_flags;
    uint32_t number_of_rva_and_sizes;
    /* Data directories follow */
};

struct data_directory {
    uint32_t virtual_address;
    uint32_t size;
};

struct section_header {
    char     name[8];
    uint32_t virtual_size;
    uint32_t virtual_address;
    uint32_t size_of_raw_data;
    uint32_t pointer_to_raw_data;
    uint32_t pointer_to_relocations;
    uint32_t pointer_to_linenumbers;
    uint16_t number_of_relocations;
    uint16_t number_of_linenumbers;
    uint32_t characteristics;
};

/* Section characteristics flags */
#define IMAGE_SCN_MEM_EXECUTE  0x20000000u
#define IMAGE_SCN_MEM_READ     0x40000000u
#define IMAGE_SCN_MEM_WRITE    0x80000000u

struct import_directory_entry {
    uint32_t import_lookup_table_rva;
    uint32_t time_date_stamp;
    uint32_t forwarder_chain;
    uint32_t name_rva;
    uint32_t import_address_table_rva;
};

/* ---------- API translation ---------- */

#define MAX_API_MAP 256

struct api_entry {
    char     dll_name[64];
    char     func_name[64];
    int32_t  linux_syscall;
};

static struct api_entry g_api_map[MAX_API_MAP];
static int g_api_count = 0;

static void load_api_map(void) {
    FILE *f = fopen("/guest/api_map.txt", "r");
    if (!f) return;

    char line[256];
    while (fgets(line, sizeof(line), f) && g_api_count < MAX_API_MAP) {
        if (line[0] == '#' || line[0] == '\n') continue;

        char dll[64], func[64], desc[64];
        int32_t syscall_num;
        if (sscanf(line, "%63s %63s %d %63s", dll, func, &syscall_num, desc) >= 3) {
            strncpy(g_api_map[g_api_count].dll_name, dll, 63);
            strncpy(g_api_map[g_api_count].func_name, func, 63);
            g_api_map[g_api_count].linux_syscall = syscall_num;
            g_api_count++;
        }
    }
    fclose(f);
}

static int32_t find_linux_syscall(const char *dll_name, const char *func_name) {
    for (int i = 0; i < g_api_count; i++) {
        if (strcasecmp(g_api_map[i].dll_name, dll_name) == 0 &&
            strcmp(g_api_map[i].func_name, func_name) == 0) {
            return g_api_map[i].linux_syscall;
        }
    }
    return -999; /* Not found */
}

/* ---------- IAT thunk generation ---------- */

/*
 * For each imported function, we generate a small x86_64 thunk that:
 * 1. Shuffles Win64 ABI registers to Linux ABI:
 *      rcx -> rdi, rdx -> rsi, r8 -> rdx, r9 -> r10
 * 2. Puts the Linux syscall number in rax
 * 3. Executes syscall; ret
 *
 * Thunk layout (25 bytes):
 *   48 89 CF            mov rdi, rcx
 *   48 89 D6            mov rsi, rdx
 *   4C 89 C2            mov rdx, r8
 *   4D 89 CA            mov r10, r9
 *   48 C7 C0 xx xx xx xx  mov rax, <syscall_num>
 *   0F 05               syscall
 *   C3                  ret
 */

#define THUNK_SIZE 25
#define MAX_THUNKS 256

static uint8_t g_thunk_pool[MAX_THUNKS * THUNK_SIZE];
static int g_thunk_count = 0;

static uint8_t *allocate_thunk(int32_t linux_syscall) {
    if (g_thunk_count >= MAX_THUNKS) return NULL;

    uint8_t *thunk = &g_thunk_pool[g_thunk_count * THUNK_SIZE];
    g_thunk_count++;

    int i = 0;

    /* mov rdi, rcx */
    thunk[i++] = 0x48; thunk[i++] = 0x89; thunk[i++] = 0xCF;
    /* mov rsi, rdx */
    thunk[i++] = 0x48; thunk[i++] = 0x89; thunk[i++] = 0xD6;
    /* mov rdx, r8 */
    thunk[i++] = 0x4C; thunk[i++] = 0x89; thunk[i++] = 0xC2;
    /* mov r10, r9 */
    thunk[i++] = 0x4D; thunk[i++] = 0x89; thunk[i++] = 0xCA;

    /* mov rax, imm32 (sign-extended) */
    thunk[i++] = 0x48; thunk[i++] = 0xC7; thunk[i++] = 0xC0;
    thunk[i++] = (uint8_t)(linux_syscall & 0xFF);
    thunk[i++] = (uint8_t)((linux_syscall >> 8) & 0xFF);
    thunk[i++] = (uint8_t)((linux_syscall >> 16) & 0xFF);
    thunk[i++] = (uint8_t)((linux_syscall >> 24) & 0xFF);

    /* syscall */
    thunk[i++] = 0x0F; thunk[i++] = 0x05;
    /* ret */
    thunk[i++] = 0xC3;

    /* Pad remaining bytes with int3 (0xCC) */
    while (i < THUNK_SIZE) thunk[i++] = 0xCC;

    return thunk;
}

/* ---------- RVA helpers ---------- */

static void *rva_to_ptr(uint64_t image_base, uint32_t rva) {
    return (void *)(image_base + rva);
}

/* ---------- IAT resolution ---------- */

static void resolve_iat(uint8_t *file_data, uint64_t image_base,
                        struct data_directory *import_dir) {
    if (import_dir->virtual_address == 0 || import_dir->size == 0) return;

    struct import_directory_entry *imp =
        (struct import_directory_entry *)rva_to_ptr(image_base, import_dir->virtual_address);

    while (imp->name_rva != 0) {
        const char *dll_name = (const char *)rva_to_ptr(image_base, imp->name_rva);
        uint64_t *iat = (uint64_t *)rva_to_ptr(image_base, imp->import_address_table_rva);
        uint64_t *ilt = (uint64_t *)rva_to_ptr(image_base,
            imp->import_lookup_table_rva ? imp->import_lookup_table_rva
                                          : imp->import_address_table_rva);

        for (int j = 0; ilt[j] != 0; j++) {
            /* Check if import by name (bit 63 clear) */
            if (!(ilt[j] & (1ULL << 63))) {
                /* Hint/Name table entry: 2-byte hint + null-terminated name */
                uint32_t hnt_rva = (uint32_t)(ilt[j] & 0x7FFFFFFF);
                const char *func_name = (const char *)rva_to_ptr(image_base, hnt_rva + 2);

                int32_t linux_syscall = find_linux_syscall(dll_name, func_name);
                if (linux_syscall != -999) {
                    uint8_t *thunk = allocate_thunk(linux_syscall);
                    if (thunk) {
                        iat[j] = (uint64_t)(uintptr_t)thunk;
                    }
                }
            }
        }

        imp++;
    }
}

/* ---------- Main loader ---------- */

int main(int argc, char *argv[]) {
    const char *binary_path = "/guest/binary";
    if (argc > 1) binary_path = argv[1];

    /* Read the PE file */
    int fd = open(binary_path, O_RDONLY);
    if (fd < 0) {
        fprintf(stderr, "pe_loader: cannot open %s\n", binary_path);
        return 1;
    }

    off_t file_size = lseek(fd, 0, SEEK_END);
    lseek(fd, 0, SEEK_SET);

    uint8_t *file_data = mmap(NULL, (size_t)file_size, PROT_READ, MAP_PRIVATE, fd, 0);
    if (file_data == MAP_FAILED) {
        fprintf(stderr, "pe_loader: mmap failed\n");
        close(fd);
        return 1;
    }
    close(fd);

    /* Parse DOS header */
    struct dos_header *dos = (struct dos_header *)file_data;
    if (dos->e_magic != 0x5A4D) { /* "MZ" */
        fprintf(stderr, "pe_loader: not a PE file (no MZ header)\n");
        return 1;
    }

    /* Parse PE signature */
    uint32_t pe_sig = *(uint32_t *)(file_data + dos->e_lfanew);
    if (pe_sig != PE_SIGNATURE) {
        fprintf(stderr, "pe_loader: invalid PE signature\n");
        return 1;
    }

    /* Parse COFF header */
    struct coff_header *coff = (struct coff_header *)(file_data + dos->e_lfanew + 4);
    if (coff->machine != IMAGE_FILE_MACHINE_AMD64) {
        fprintf(stderr, "pe_loader: not x86_64 (machine=0x%04x)\n", coff->machine);
        return 1;
    }

    /* Parse Optional Header */
    struct optional_header_64 *opt =
        (struct optional_header_64 *)(file_data + dos->e_lfanew + 4 + sizeof(struct coff_header));

    if (opt->magic != 0x020B) { /* PE32+ */
        fprintf(stderr, "pe_loader: not PE32+ (magic=0x%04x)\n", opt->magic);
        return 1;
    }

    uint64_t image_base = opt->image_base;
    uint32_t entry_rva = opt->address_of_entry_point;
    uint32_t size_of_image = opt->size_of_image;

    /* Load the API translation table */
    load_api_map();

    /* Map the entire image region */
    void *image = mmap(
        (void *)image_base, size_of_image,
        PROT_READ | PROT_WRITE,
        MAP_PRIVATE | MAP_ANONYMOUS | MAP_FIXED,
        -1, 0
    );
    if (image == MAP_FAILED) {
        fprintf(stderr, "pe_loader: mmap image at 0x%lx failed\n", (unsigned long)image_base);
        return 1;
    }

    /* Copy headers */
    memcpy(image, file_data, opt->size_of_headers);

    /* Map sections */
    struct section_header *sections = (struct section_header *)(
        (uint8_t *)opt + coff->size_of_optional_header
    );

    for (uint16_t i = 0; i < coff->number_of_sections; i++) {
        struct section_header *sec = &sections[i];
        uint8_t *dest = (uint8_t *)image_base + sec->virtual_address;
        uint32_t copy_size = sec->size_of_raw_data < sec->virtual_size
                                 ? sec->size_of_raw_data
                                 : sec->virtual_size;

        if (sec->pointer_to_raw_data > 0 && copy_size > 0) {
            memcpy(dest, file_data + sec->pointer_to_raw_data, copy_size);
        }

        /* Zero-fill remainder */
        if (sec->virtual_size > copy_size) {
            memset(dest + copy_size, 0, sec->virtual_size - copy_size);
        }
    }

    /* Make thunk pool executable */
    size_t pool_size = (size_t)(MAX_THUNKS * THUNK_SIZE);
    void *pool_page = (void *)((uintptr_t)g_thunk_pool & ~0xFFFULL);
    size_t pool_mmap_size = pool_size + ((uintptr_t)g_thunk_pool - (uintptr_t)pool_page);
    mprotect(pool_page, pool_mmap_size, PROT_READ | PROT_WRITE | PROT_EXEC);

    /* Resolve IAT */
    struct data_directory *dirs = (struct data_directory *)(
        (uint8_t *)opt + offsetof(struct optional_header_64, number_of_rva_and_sizes) + 4
    );
    if (opt->number_of_rva_and_sizes > 1) {
        resolve_iat(file_data, image_base, &dirs[1]); /* Import table is directory[1] */
    }

    /* Set section protections */
    for (uint16_t i = 0; i < coff->number_of_sections; i++) {
        struct section_header *sec = &sections[i];
        uint8_t *dest = (uint8_t *)image_base + sec->virtual_address;
        int prot = 0;
        if (sec->characteristics & IMAGE_SCN_MEM_READ)    prot |= PROT_READ;
        if (sec->characteristics & IMAGE_SCN_MEM_WRITE)   prot |= PROT_WRITE;
        if (sec->characteristics & IMAGE_SCN_MEM_EXECUTE) prot |= PROT_EXEC;
        if (prot == 0) prot = PROT_READ;

        /* Align to page boundary */
        uintptr_t page_start = (uintptr_t)dest & ~0xFFFULL;
        size_t page_size = ((uintptr_t)dest + sec->virtual_size - page_start + 0xFFF) & ~0xFFFULL;
        mprotect((void *)page_start, page_size, prot);
    }

    /* Unmap the file */
    munmap(file_data, (size_t)file_size);

    /* Jump to entry point */
    uint64_t entry_addr = image_base + entry_rva;
    typedef int (*entry_fn)(void);
    entry_fn entry = (entry_fn)entry_addr;

    return entry();
}
