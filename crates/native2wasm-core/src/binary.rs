use std::collections::HashMap;

use crate::target::OsTarget;

/// A parsed code section from a native binary.
#[derive(Debug, Clone)]
pub struct CodeSection {
    pub name: String,
    pub addr: u64,
    pub data: Vec<u8>,
}

/// A parsed data section from a native binary.
#[derive(Debug, Clone)]
pub struct DataSection {
    pub name: String,
    pub addr: u64,
    pub data: Vec<u8>,
    pub writable: bool,
}

/// Trait abstracting over parsed native binaries (Mach-O, PE, ELF).
pub trait ParsedBinary {
    /// Raw binary bytes.
    fn bytes(&self) -> &[u8];

    /// Architecture string (e.g., "x86_64").
    fn arch(&self) -> &str;

    /// Entry point virtual address.
    fn entry_point(&self) -> u64;

    /// Code sections loaded from the binary.
    fn code_sections(&self) -> &[CodeSection];

    /// Data sections loaded from the binary.
    fn data_sections(&self) -> &[DataSection];

    /// Symbol table: address → name.
    fn symbols(&self) -> &HashMap<u64, String>;

    /// Whether the binary is statically linked.
    fn is_static(&self) -> bool;

    /// Dynamic library/DLL dependencies.
    fn dynamic_deps(&self) -> &[String];

    /// Target operating system.
    fn os_target(&self) -> OsTarget;
}
