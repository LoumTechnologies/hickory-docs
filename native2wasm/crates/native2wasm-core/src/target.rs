use std::fmt;

/// CPU architecture of the input binary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuArch {
    X86_64,
    AArch64,
}

impl CpuArch {
    /// Remill architecture name string (passed to `remill::Arch::Get`).
    pub fn remill_arch_name(&self) -> &'static str {
        match self {
            CpuArch::X86_64 => "amd64",
            CpuArch::AArch64 => "aarch64",
        }
    }

    /// Size of Remill's `State` struct in bytes for this architecture.
    ///
    /// x86_64: 3504 bytes (from Remill's `X86State` including GPRs, SSE, x87, segments).
    /// AArch64: 2728 bytes (from Remill's `AArch64State` including GPRs, NEON/FP, system regs).
    pub fn state_struct_size(&self) -> u32 {
        match self {
            CpuArch::X86_64 => 3504,
            CpuArch::AArch64 => 2728,
        }
    }

    /// Human-readable name for log messages.
    pub fn display_name(&self) -> &'static str {
        match self {
            CpuArch::X86_64 => "x86_64",
            CpuArch::AArch64 => "AArch64",
        }
    }
}

impl fmt::Display for CpuArch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.display_name())
    }
}

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
