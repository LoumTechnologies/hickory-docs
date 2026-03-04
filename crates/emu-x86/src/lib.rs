//! x86_64 user-mode emulation via blink.
//!
//! blink (github.com/jart/blink) is a 116KB x86_64 Linux user-mode emulator
//! that handles 500+ instructions and 150+ syscalls. When compiled to WASM,
//! it runs as a pure interpreter (no JIT).
//!
//! This crate implements `EmulatorProvider` for x86_64 by:
//! 1. Locating a pre-compiled `blink.wasm` (from cache or explicit path)
//! 2. Preparing a guest filesystem with the target binary at `/guest/binary`
//! 3. The emulation pipeline (in emu-core) packs everything via wasi-vfs
//!
//! ## Building blink.wasm
//!
//! blink must be cross-compiled to wasm32-wasi with JIT disabled:
//!
//! ```bash
//! git clone https://github.com/jart/blink
//! cd blink
//! # Using wasi-sdk:
//! CC=/opt/wasi-sdk/bin/clang ./configure --target=wasm32-wasi \
//!     --disable-jit --disable-fork --disable-signals
//! make -j$(nproc)
//! # Output: blink → rename to blink.wasm
//! ```
//!
//! Then cache it:
//! ```bash
//! mkdir -p ~/.cache/native2wasm/emulators/blink
//! cp blink ~/.cache/native2wasm/emulators/blink/emulator.wasm
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use emu_core::cache;
use emu_core::loader;
use emu_core::provider::EmulatorProvider;
use native2wasm_core::target::{CpuArch, OsTarget};

/// Emulator provider name used for cache directory.
const PROVIDER_NAME: &str = "blink";

/// blink-based x86_64 Linux user-mode emulator provider.
pub struct BlinkProvider {
    /// Explicit path to blink.wasm (overrides cache lookup).
    explicit_path: Option<PathBuf>,
}

impl BlinkProvider {
    /// Create a new blink provider.
    pub fn new() -> Self {
        Self {
            explicit_path: None,
        }
    }

    /// Create a blink provider with an explicit emulator WASM path.
    pub fn with_path(path: PathBuf) -> Self {
        Self {
            explicit_path: Some(path),
        }
    }
}

impl Default for BlinkProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl EmulatorProvider for BlinkProvider {
    fn name(&self) -> &str {
        PROVIDER_NAME
    }

    fn supported_arch(&self) -> CpuArch {
        CpuArch::X86_64
    }

    fn supported_os(&self) -> OsTarget {
        OsTarget::Linux
    }

    fn emulator_wasm(&self) -> Result<PathBuf> {
        // 1. Explicit path
        if let Some(ref path) = self.explicit_path {
            if !path.exists() {
                bail!("blink.wasm not found at {}", path.display());
            }
            return Ok(path.clone());
        }

        // 2. Check cache
        if let Some(cached) = cache::cached_emulator(PROVIDER_NAME) {
            return Ok(cached);
        }

        // 3. Not found — provide instructions
        let cache_dir = cache::emulator_cache_dir(PROVIDER_NAME);
        bail!(
            "blink.wasm not found. To set up x86_64 emulation:\n\n\
             Option 1: Build from source\n\
               git clone https://github.com/jart/blink\n\
               cd blink\n\
               CC=/opt/wasi-sdk/bin/clang ./configure \\\n\
                 --target=wasm32-wasi --disable-jit\n\
               make -j$(nproc)\n\
               mkdir -p {cache_dir}\n\
               cp blink {cache_dir}/emulator.wasm\n\n\
             Option 2: Use --emulator-path to point to an existing blink.wasm\n\
               native2wasm input -o out.wasm --strategy emu \\\n\
                 --emulator-path /path/to/blink.wasm\n",
            cache_dir = cache_dir.display(),
        )
    }

    fn prepare_guest_fs(&self, binary: &Path, workdir: &Path) -> Result<PathBuf> {
        let guest_dir = workdir.join("guest");
        fs::create_dir_all(&guest_dir).context("failed to create guest directory")?;

        // Copy the target binary into the guest filesystem.
        let dest = guest_dir.join("binary");
        fs::copy(binary, &dest)
            .with_context(|| format!("failed to copy {} to guest filesystem", binary.display()))?;

        // Make the binary executable (on platforms that support it).
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o755);
            let _ = fs::set_permissions(&dest, perms);
        }

        log::info!(
            "packed guest binary: {} -> /guest/binary ({} bytes)",
            binary.display(),
            fs::metadata(&dest).map(|m| m.len()).unwrap_or(0),
        );

        Ok(guest_dir)
    }

    fn emulator_args(&self) -> Vec<String> {
        // blink executes: blink /guest/binary [args...]
        // The guest binary path is the first argument to blink.
        vec!["/guest/binary".to_string()]
    }
}

// ---------------------------------------------------------------------------
// macOS x86_64 emulation provider (blink + Mach-O loader shim)
// ---------------------------------------------------------------------------

/// Provider name for macOS x86_64 emulation.
const MACHO_PROVIDER_NAME: &str = "blink-macho";

/// blink-based x86_64 macOS user-mode emulator provider.
///
/// Uses the same blink emulator as Linux x86_64 but bundles:
/// - A Linux ELF loader wrapper that reads a Mach-O binary, maps its segments,
///   and translates XNU BSD syscalls to Linux equivalents
/// - Minimal libSystem.dylib stubs for basic POSIX-like calls
///
/// The loader wrapper runs inside blink as a normal Linux binary. It loads
/// the Mach-O and intercepts syscalls by stripping the `0x2000000` class
/// prefix and forwarding to Linux syscall equivalents.
///
/// Scope: handles binaries using basic POSIX-like calls (file I/O, exit,
/// getpid). Complex binaries should fall through to the VM layer.
pub struct BlinkMachoProvider {
    /// Explicit path to blink.wasm (overrides cache lookup).
    explicit_path: Option<PathBuf>,
}

impl BlinkMachoProvider {
    pub fn new() -> Self {
        Self {
            explicit_path: None,
        }
    }

    pub fn with_path(path: PathBuf) -> Self {
        Self {
            explicit_path: Some(path),
        }
    }
}

impl Default for BlinkMachoProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl EmulatorProvider for BlinkMachoProvider {
    fn name(&self) -> &str {
        MACHO_PROVIDER_NAME
    }

    fn supported_arch(&self) -> CpuArch {
        CpuArch::X86_64
    }

    fn supported_os(&self) -> OsTarget {
        OsTarget::Darwin
    }

    fn emulator_wasm(&self) -> Result<PathBuf> {
        // Uses the same blink.wasm as the Linux provider.
        if let Some(ref path) = self.explicit_path {
            if !path.exists() {
                bail!("blink.wasm not found at {}", path.display());
            }
            return Ok(path.clone());
        }

        // Check blink cache (shared with Linux provider).
        if let Some(cached) = cache::cached_emulator(PROVIDER_NAME) {
            return Ok(cached);
        }

        // Check macOS-specific cache.
        if let Some(cached) = cache::cached_emulator(MACHO_PROVIDER_NAME) {
            return Ok(cached);
        }

        let cache_dir = cache::emulator_cache_dir(PROVIDER_NAME);
        bail!(
            "blink.wasm not found. Required for macOS x86_64 emulation.\n\n\
             The macOS emulation provider uses the same blink emulator as Linux.\n\
             See: native2wasm --help for setup instructions.\n\n\
             Cache directory: {cache_dir}\n\n\
             Or use --emulator-path to point to an existing blink.wasm",
            cache_dir = cache_dir.display(),
        )
    }

    fn prepare_guest_fs(&self, binary: &Path, workdir: &Path) -> Result<PathBuf> {
        let guest_dir = workdir.join("guest");
        fs::create_dir_all(&guest_dir).context("failed to create guest directory")?;

        // Place the Mach-O binary for the loader to read.
        let dest = guest_dir.join("binary");
        fs::copy(binary, &dest)
            .with_context(|| format!("failed to copy {} to guest filesystem", binary.display()))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&dest, fs::Permissions::from_mode(0o755));
        }

        // Copy the Mach-O loader shim into the guest filesystem.
        if let Ok(loader_path) = loader::locate_loader("macho_loader") {
            let loader_dest = guest_dir.join("loader");
            fs::copy(&loader_path, &loader_dest)
                .context("failed to copy macho_loader to guest filesystem")?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(&loader_dest, fs::Permissions::from_mode(0o755));
            }
        } else {
            log::warn!("macho_loader not found; run scripts/build-loaders.sh");
        }

        // Create stub library directory.
        let lib_dir = guest_dir.join("lib");
        fs::create_dir_all(&lib_dir).context("failed to create guest lib directory")?;

        // Write the XNU syscall translation map for the loader.
        write_xnu_syscall_map(&guest_dir)?;

        log::info!(
            "packed macOS guest: {} -> /guest/binary ({} bytes)",
            binary.display(),
            fs::metadata(&dest).map(|m| m.len()).unwrap_or(0),
        );

        Ok(guest_dir)
    }

    fn emulator_args(&self) -> Vec<String> {
        // The loader wrapper reads /guest/binary (a Mach-O) and executes it.
        // blink runs the loader, which runs the Mach-O.
        vec!["/guest/loader".to_string(), "/guest/binary".to_string()]
    }
}

/// Write XNU BSD syscall translation metadata to the guest filesystem.
///
/// This is a simple text map that the loader wrapper reads to translate
/// XNU BSD syscalls (class 2, `0x2000000 | num`) to Linux equivalents.
///
/// Only covers basic POSIX-compatible syscalls. Complex or macOS-specific
/// syscalls (Mach ports, IOKit, Grand Central Dispatch) return -ENOSYS.
fn write_xnu_syscall_map(guest_dir: &Path) -> Result<()> {
    // Format: xnu_num linux_num name
    // XNU BSD syscalls have class=2 prefix stripped here; caller handles that.
    let map = "\
# XNU BSD → Linux syscall translation table
# Format: xnu_num linux_num name
# Only POSIX-compatible calls; complex syscalls return -ENOSYS
1 60 exit
2 57 fork
3 0 read
4 1 write
5 2 open
6 3 close
7 61 wait4
9 87 link
10 88 unlink
12 12 chdir
15 82 chmod
16 93 fchmod
20 39 getpid
23 104 setuid
24 102 getuid
25 105 setgid
27 62 kill
33 21 access
37 32 kill
39 80 getppid
41 22 pipe
46 92 sigaction
48 79 getcwd
54 16 ioctl
59 85 readlink
73 72 fcntl
74 0 read
75 1 write
78 56 clone
92 83 mkdir
97 4 stat
99 6 lstat
100 5 fstat
116 35 nanosleep
117 38 setitimer
120 39 getpid
121 1 write
153 231 fstatfs
190 8 lseek
194 89 readv
195 20 writev
196 78 getdents64
197 5 fstat
199 228 fstatfs
202 2 open
338 257 openat
339 258 mkdirat
340 263 unlinkat
343 262 newfstatat
344 261 futimesat
345 259 renameat
";
    let map_path = guest_dir.join("syscall_map.txt");
    fs::write(&map_path, map).context("failed to write XNU syscall map")?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Windows x86_64 emulation provider (blink + PE loader shim)
// ---------------------------------------------------------------------------

/// Provider name for Windows x86_64 emulation.
const PE_PROVIDER_NAME: &str = "blink-pe";

/// blink-based x86_64 Windows user-mode emulator provider.
///
/// Uses the same blink emulator as Linux x86_64 but bundles:
/// - A Linux ELF loader wrapper that reads a PE binary, resolves the IAT,
///   maps sections, and translates NT syscalls / Win32 API calls to Linux
/// - Minimal kernel32.dll / ntdll.dll stubs for console I/O
///
/// Scope: handles binaries using MSVC CRT console functions only
/// (WriteFile, ReadFile, GetStdHandle, ExitProcess). Complex Win32
/// binaries should fall through to the VM layer with Wine.
pub struct BlinkPeProvider {
    /// Explicit path to blink.wasm (overrides cache lookup).
    explicit_path: Option<PathBuf>,
}

impl BlinkPeProvider {
    pub fn new() -> Self {
        Self {
            explicit_path: None,
        }
    }

    pub fn with_path(path: PathBuf) -> Self {
        Self {
            explicit_path: Some(path),
        }
    }
}

impl Default for BlinkPeProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl EmulatorProvider for BlinkPeProvider {
    fn name(&self) -> &str {
        PE_PROVIDER_NAME
    }

    fn supported_arch(&self) -> CpuArch {
        CpuArch::X86_64
    }

    fn supported_os(&self) -> OsTarget {
        OsTarget::Windows
    }

    fn emulator_wasm(&self) -> Result<PathBuf> {
        if let Some(ref path) = self.explicit_path {
            if !path.exists() {
                bail!("blink.wasm not found at {}", path.display());
            }
            return Ok(path.clone());
        }

        if let Some(cached) = cache::cached_emulator(PROVIDER_NAME) {
            return Ok(cached);
        }

        if let Some(cached) = cache::cached_emulator(PE_PROVIDER_NAME) {
            return Ok(cached);
        }

        let cache_dir = cache::emulator_cache_dir(PROVIDER_NAME);
        bail!(
            "blink.wasm not found. Required for Windows x86_64 emulation.\n\n\
             The Windows emulation provider uses the same blink emulator as Linux.\n\
             See: native2wasm --help for setup instructions.\n\n\
             Cache directory: {cache_dir}\n\n\
             Or use --emulator-path to point to an existing blink.wasm",
            cache_dir = cache_dir.display(),
        )
    }

    fn prepare_guest_fs(&self, binary: &Path, workdir: &Path) -> Result<PathBuf> {
        let guest_dir = workdir.join("guest");
        fs::create_dir_all(&guest_dir).context("failed to create guest directory")?;

        // Place the PE binary for the loader to read.
        let dest = guest_dir.join("binary");
        fs::copy(binary, &dest)
            .with_context(|| format!("failed to copy {} to guest filesystem", binary.display()))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&dest, fs::Permissions::from_mode(0o755));
        }

        // Copy the PE loader shim into the guest filesystem.
        if let Ok(loader_path) = loader::locate_loader("pe_loader") {
            let loader_dest = guest_dir.join("loader");
            fs::copy(&loader_path, &loader_dest)
                .context("failed to copy pe_loader to guest filesystem")?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(&loader_dest, fs::Permissions::from_mode(0o755));
            }
        } else {
            log::warn!("pe_loader not found; run scripts/build-loaders.sh");
        }

        // Create stub DLL directory.
        let dll_dir = guest_dir.join("system32");
        fs::create_dir_all(&dll_dir).context("failed to create guest system32 directory")?;

        // Write the Win32 API translation map for the loader.
        write_win32_api_map(&guest_dir)?;

        log::info!(
            "packed Windows guest: {} -> /guest/binary ({} bytes)",
            binary.display(),
            fs::metadata(&dest).map(|m| m.len()).unwrap_or(0),
        );

        Ok(guest_dir)
    }

    fn emulator_args(&self) -> Vec<String> {
        vec!["/guest/loader".to_string(), "/guest/binary".to_string()]
    }
}

/// Write Win32 API translation metadata to the guest filesystem.
///
/// Maps the minimal set of Win32 API functions that console programs
/// typically use to their Linux syscall equivalents.
fn write_win32_api_map(guest_dir: &Path) -> Result<()> {
    let map = "\
# Win32 API → Linux syscall translation table
# Format: dll_name function_name linux_syscall_num description
# Only MSVC CRT console functions; complex Win32 returns error
kernel32.dll WriteFile 1 write
kernel32.dll ReadFile 0 read
kernel32.dll ExitProcess 60 exit
kernel32.dll GetStdHandle -1 return_handle
kernel32.dll CloseHandle 3 close
kernel32.dll CreateFileA 2 open
kernel32.dll CreateFileW 2 open
kernel32.dll GetFileSize 5 fstat
kernel32.dll SetFilePointer 8 lseek
kernel32.dll GetLastError -2 errno_translation
kernel32.dll SetLastError -3 set_errno
kernel32.dll GetCommandLineA -4 cmdline
kernel32.dll GetCommandLineW -4 cmdline
kernel32.dll GetCurrentProcess -5 getpid
kernel32.dll GetCurrentProcessId 39 getpid
kernel32.dll GetCurrentThreadId 186 gettid
kernel32.dll GetEnvironmentVariableA -6 getenv
kernel32.dll GetEnvironmentVariableW -6 getenv
kernel32.dll GetModuleHandleA -7 module_handle
kernel32.dll GetModuleHandleW -7 module_handle
kernel32.dll GetProcAddress -8 dlsym
kernel32.dll HeapAlloc -9 malloc
kernel32.dll HeapFree -10 free
kernel32.dll HeapCreate -11 heap_create
kernel32.dll VirtualAlloc 9 mmap
kernel32.dll VirtualFree 11 munmap
ntdll.dll RtlInitUnicodeString -12 init_string
ntdll.dll NtClose 3 close
ntdll.dll NtReadFile 0 read
ntdll.dll NtWriteFile 1 write
ntdll.dll NtTerminateProcess 60 exit
";
    let map_path = guest_dir.join("api_map.txt");
    fs::write(&map_path, map).context("failed to write Win32 API map")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_basics() {
        let provider = BlinkProvider::new();
        assert_eq!(provider.name(), "blink");
        assert_eq!(provider.supported_arch(), CpuArch::X86_64);
    }

    #[test]
    fn prepare_guest_fs_copies_binary() {
        let provider = BlinkProvider::new();
        let tmpdir = tempfile::tempdir().unwrap();

        // Create a fake binary.
        let fake_binary = tmpdir.path().join("test_binary");
        fs::write(&fake_binary, b"fake ELF content").unwrap();

        let workdir = tmpdir.path().join("work");
        fs::create_dir(&workdir).unwrap();

        let guest_dir = provider.prepare_guest_fs(&fake_binary, &workdir).unwrap();
        let packed = guest_dir.join("binary");
        assert!(packed.exists());
        assert_eq!(fs::read(&packed).unwrap(), b"fake ELF content");
    }

    #[test]
    fn macho_provider_basics() {
        let provider = BlinkMachoProvider::new();
        assert_eq!(provider.name(), "blink-macho");
        assert_eq!(provider.supported_arch(), CpuArch::X86_64);
        assert_eq!(provider.supported_os(), OsTarget::Darwin);
        assert_eq!(
            provider.emulator_args(),
            vec!["/guest/loader", "/guest/binary"]
        );
    }

    #[test]
    fn macho_prepare_guest_fs_creates_syscall_map() {
        let provider = BlinkMachoProvider::new();
        let tmpdir = tempfile::tempdir().unwrap();

        let fake_binary = tmpdir.path().join("test_macho");
        fs::write(&fake_binary, b"fake Mach-O content").unwrap();

        let workdir = tmpdir.path().join("work");
        fs::create_dir(&workdir).unwrap();

        let guest_dir = provider.prepare_guest_fs(&fake_binary, &workdir).unwrap();

        // Binary is packed.
        assert!(guest_dir.join("binary").exists());
        // Lib directory exists for stub libraries.
        assert!(guest_dir.join("lib").is_dir());
        // Syscall map exists.
        let map = guest_dir.join("syscall_map.txt");
        assert!(map.exists());
        let content = fs::read_to_string(&map).unwrap();
        assert!(content.contains("exit"));
        assert!(content.contains("write"));
    }

    #[test]
    fn pe_provider_basics() {
        let provider = BlinkPeProvider::new();
        assert_eq!(provider.name(), "blink-pe");
        assert_eq!(provider.supported_arch(), CpuArch::X86_64);
        assert_eq!(provider.supported_os(), OsTarget::Windows);
        assert_eq!(
            provider.emulator_args(),
            vec!["/guest/loader", "/guest/binary"]
        );
    }

    #[test]
    fn pe_prepare_guest_fs_creates_api_map() {
        let provider = BlinkPeProvider::new();
        let tmpdir = tempfile::tempdir().unwrap();

        let fake_binary = tmpdir.path().join("test.exe");
        fs::write(&fake_binary, b"fake PE content").unwrap();

        let workdir = tmpdir.path().join("work");
        fs::create_dir(&workdir).unwrap();

        let guest_dir = provider.prepare_guest_fs(&fake_binary, &workdir).unwrap();

        // Binary is packed.
        assert!(guest_dir.join("binary").exists());
        // system32 directory exists for DLL stubs.
        assert!(guest_dir.join("system32").is_dir());
        // API map exists.
        let map = guest_dir.join("api_map.txt");
        assert!(map.exists());
        let content = fs::read_to_string(&map).unwrap();
        assert!(content.contains("WriteFile"));
        assert!(content.contains("ExitProcess"));
        assert!(content.contains("kernel32.dll"));
        assert!(content.contains("ntdll.dll"));
    }
}
