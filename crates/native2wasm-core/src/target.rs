use std::fmt;

/// Target operating system for the binary being transpiled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsTarget {
    Darwin,
    Windows,
    Linux,
}

impl OsTarget {
    /// Remill OS name string (passed to `remill::Arch::Get`).
    pub fn remill_os(&self) -> &'static str {
        match self {
            OsTarget::Darwin => "macos",
            OsTarget::Windows => "windows",
            OsTarget::Linux => "linux",
        }
    }

    /// Symbol name for the syscall dispatch function in shim code.
    pub fn syscall_symbol(&self) -> &'static str {
        match self {
            OsTarget::Darwin => "__mach2wasm_syscall",
            OsTarget::Windows => "__pe2wasm_syscall",
            OsTarget::Linux => "__elf2wasm_syscall",
        }
    }

    /// Human-readable name for log messages.
    pub fn display_name(&self) -> &'static str {
        match self {
            OsTarget::Darwin => "Darwin",
            OsTarget::Windows => "NT",
            OsTarget::Linux => "Linux",
        }
    }
}

impl fmt::Display for OsTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.display_name())
    }
}
