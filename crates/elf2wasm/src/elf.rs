use std::collections::HashMap;
use std::path::Path;

use goblin::elf::Elf;
use goblin::elf::header::{EM_X86_64, ET_EXEC, ET_DYN};
use goblin::elf::program_header::PT_LOAD;
use goblin::elf::section_header::{SHF_EXECINSTR, SHF_WRITE, SHT_NOBITS, SHT_PROGBITS};

use crate::error::Error;

/// A parsed code section from the ELF binary.
#[derive(Debug, Clone)]
pub struct CodeSection {
    pub name: String,
    pub addr: u64,
    pub data: Vec<u8>,
}

/// A parsed data section from the ELF binary.
#[derive(Debug, Clone)]
pub struct DataSection {
    pub name: String,
    pub addr: u64,
    pub data: Vec<u8>,
    pub writable: bool,
}

/// Parsed representation of an ELF x86_64 binary.
#[derive(Debug)]
pub struct ParsedELF {
    pub bytes: Vec<u8>,
    pub arch: String,
    pub entry_point: u64,
    pub code_sections: Vec<CodeSection>,
    pub data_sections: Vec<DataSection>,
    pub symbols: HashMap<u64, String>,
    pub shared_libs: Vec<String>,
    pub is_static: bool,
}

/// Parse an ELF file from disk.
pub fn parse(path: &Path) -> Result<ParsedELF, Error> {
    let bytes = std::fs::read(path).map_err(|e| {
        Error::Io(std::io::Error::new(
            e.kind(),
            format!("{}: {e}", path.display()),
        ))
    })?;
    parse_bytes(bytes, path)
}

/// Parse an ELF from raw bytes.
pub fn parse_bytes(bytes: Vec<u8>, path: &Path) -> Result<ParsedELF, Error> {
    // Validate ELF magic: 0x7F 'E' 'L' 'F'.
    if bytes.len() < 4 || bytes[0] != 0x7F || bytes[1] != b'E' || bytes[2] != b'L' || bytes[3] != b'F' {
        return Err(Error::NotELF {
            path: path.to_path_buf(),
        });
    }

    let elf = Elf::parse(&bytes).map_err(|_| Error::NotELF {
        path: path.to_path_buf(),
    })?;

    // Validate architecture: must be x86_64 (EM_X86_64 = 0x3E).
    if elf.header.e_machine != EM_X86_64 {
        return Err(Error::UnsupportedArch {
            arch: format!("e_machine=0x{:04x}", elf.header.e_machine),
        });
    }

    // Validate file type: must be executable or position-independent executable.
    if elf.header.e_type != ET_EXEC && elf.header.e_type != ET_DYN {
        return Err(Error::NotExecutable {
            path: path.to_path_buf(),
        });
    }

    // Entry point.
    let entry_point = elf.header.e_entry;
    if entry_point == 0 {
        return Err(Error::NoEntryPoint);
    }

    // Extract code and data sections.
    let mut code_sections = Vec::new();
    let mut data_sections = Vec::new();

    for sh in &elf.section_headers {
        let name = elf.shdr_strtab.get_at(sh.sh_name).unwrap_or("").to_string();

        // Skip non-allocated sections (not loaded into memory).
        if sh.sh_flags as u32 & 0x2 /* SHF_ALLOC */ == 0 {
            continue;
        }

        let flags = sh.sh_flags as u32;

        if flags & SHF_EXECINSTR as u32 != 0 && sh.sh_type == SHT_PROGBITS {
            // Code section (e.g. .text, .init, .fini).
            let data = extract_section_data(&bytes, sh);
            code_sections.push(CodeSection {
                name,
                addr: sh.sh_addr,
                data,
            });
        } else if sh.sh_type == SHT_NOBITS {
            // BSS section — zero-filled.
            let size = sh.sh_size as usize;
            data_sections.push(DataSection {
                name,
                addr: sh.sh_addr,
                data: vec![0u8; size],
                writable: true,
            });
        } else if sh.sh_type == SHT_PROGBITS {
            // Data section (.data, .rodata, etc.).
            let writable = flags & SHF_WRITE as u32 != 0;
            let data = extract_section_data(&bytes, sh);
            if !data.is_empty() {
                data_sections.push(DataSection {
                    name,
                    addr: sh.sh_addr,
                    data,
                    writable,
                });
            }
        }
    }

    if code_sections.is_empty() {
        // Fallback: try to extract code from PT_LOAD segments with execute permission.
        for phdr in &elf.program_headers {
            if phdr.p_type == PT_LOAD && phdr.p_flags & 0x1 /* PF_X */ != 0 {
                let offset = phdr.p_offset as usize;
                let filesz = phdr.p_filesz as usize;
                let end = (offset + filesz).min(bytes.len());
                if offset < bytes.len() && filesz > 0 {
                    code_sections.push(CodeSection {
                        name: format!("LOAD@0x{:x}", phdr.p_vaddr),
                        addr: phdr.p_vaddr,
                        data: bytes[offset..end].to_vec(),
                    });
                }
            }
        }
    }

    if code_sections.is_empty() {
        return Err(Error::NoCodeSections);
    }

    // Extract symbols from the symbol table.
    let mut symbols = HashMap::new();
    for sym in &elf.syms {
        if sym.st_value != 0 {
            if let Some(name) = elf.strtab.get_at(sym.st_name) {
                if !name.is_empty() {
                    symbols.insert(sym.st_value, name.to_string());
                }
            }
        }
    }
    // Also check dynamic symbols.
    for sym in &elf.dynsyms {
        if sym.st_value != 0 {
            if let Some(name) = elf.dynstrtab.get_at(sym.st_name) {
                if !name.is_empty() {
                    symbols.entry(sym.st_value).or_insert_with(|| name.to_string());
                }
            }
        }
    }

    // Collect shared library dependencies (DT_NEEDED entries).
    let shared_libs: Vec<String> = elf
        .libraries
        .iter()
        .map(|s| s.to_string())
        .collect();
    let is_static = shared_libs.is_empty();

    Ok(ParsedELF {
        bytes,
        arch: "x86_64".to_string(),
        entry_point,
        code_sections,
        data_sections,
        symbols,
        shared_libs,
        is_static,
    })
}

/// Extract raw section data from the ELF file bytes.
fn extract_section_data(
    file_bytes: &[u8],
    sh: &goblin::elf::SectionHeader,
) -> Vec<u8> {
    let offset = sh.sh_offset as usize;
    let size = sh.sh_size as usize;

    if offset == 0 || size == 0 {
        return Vec::new();
    }

    let end = (offset + size).min(file_bytes.len());
    if offset >= file_bytes.len() {
        return vec![0u8; size];
    }

    file_bytes[offset..end].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reject_non_elf() {
        let result = parse_bytes(b"not an ELF file".to_vec(), Path::new("fake.bin"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, Error::NotELF { .. }));
    }

    #[test]
    fn reject_empty() {
        let result = parse_bytes(vec![], Path::new("empty"));
        assert!(result.is_err());
    }

    #[test]
    fn reject_elf_magic_only() {
        // Just an ELF magic with no valid header behind it.
        let mut data = vec![0u8; 64];
        data[0] = 0x7F;
        data[1] = b'E';
        data[2] = b'L';
        data[3] = b'F';
        let result = parse_bytes(data, Path::new("magic_only"));
        assert!(result.is_err());
    }
}
