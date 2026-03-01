use std::collections::HashMap;
use std::path::Path;

use goblin::pe::PE;
use goblin::pe::section_table::IMAGE_SCN_CNT_CODE;

use crate::error::Error;

/// IMAGE_FILE_MACHINE_AMD64
const MACHINE_AMD64: u16 = 0x8664;

/// IMAGE_SCN_CNT_UNINITIALIZED_DATA
const IMAGE_SCN_CNT_UNINITIALIZED_DATA: u32 = 0x00000080;

/// IMAGE_SCN_MEM_WRITE
const IMAGE_SCN_MEM_WRITE: u32 = 0x80000000;

/// Default image base for 64-bit PE executables.
const DEFAULT_IMAGE_BASE_64: u64 = 0x0000_0001_4000_0000;

/// A parsed code section from the PE binary.
#[derive(Debug, Clone)]
pub struct CodeSection {
    pub name: String,
    pub addr: u64,
    pub data: Vec<u8>,
}

/// A parsed data section from the PE binary.
#[derive(Debug, Clone)]
pub struct DataSection {
    pub name: String,
    pub addr: u64,
    pub data: Vec<u8>,
    pub writable: bool,
}

/// Parsed representation of a PE x86_64 binary.
#[derive(Debug)]
pub struct ParsedPE {
    pub bytes: Vec<u8>,
    pub arch: String,
    pub entry_point: u64,
    pub code_sections: Vec<CodeSection>,
    pub data_sections: Vec<DataSection>,
    pub symbols: HashMap<u64, String>,
    pub import_dlls: Vec<String>,
    pub is_static: bool,
}

/// Parse a PE file from disk.
pub fn parse(path: &Path) -> Result<ParsedPE, Error> {
    let bytes = std::fs::read(path).map_err(|e| {
        Error::Io(std::io::Error::new(
            e.kind(),
            format!("{}: {e}", path.display()),
        ))
    })?;
    parse_bytes(bytes, path)
}

/// Parse a PE from raw bytes.
pub fn parse_bytes(bytes: Vec<u8>, path: &Path) -> Result<ParsedPE, Error> {
    let pe = PE::parse(&bytes).map_err(|_| Error::NotPE {
        path: path.to_path_buf(),
    })?;

    // Validate MZ signature.
    if bytes.len() < 2 || bytes[0] != b'M' || bytes[1] != b'Z' {
        return Err(Error::NotPE {
            path: path.to_path_buf(),
        });
    }

    // Validate architecture: must be x86_64 (IMAGE_FILE_MACHINE_AMD64).
    let machine = pe.header.coff_header.machine;
    if machine != MACHINE_AMD64 {
        return Err(Error::UnsupportedArch {
            arch: format!("machine=0x{machine:04x}"),
        });
    }

    // Entry point: RVA + image base.
    let image_base = pe
        .header
        .optional_header
        .map(|oh| oh.windows_fields.image_base)
        .unwrap_or(DEFAULT_IMAGE_BASE_64);

    let entry_rva = pe
        .header
        .optional_header
        .map(|oh| oh.standard_fields.address_of_entry_point)
        .unwrap_or(0);

    if entry_rva == 0 {
        return Err(Error::NoEntryPoint);
    }

    let entry_point = image_base + entry_rva;

    // Extract code and data sections.
    let mut code_sections = Vec::new();
    let mut data_sections = Vec::new();

    for section in &pe.sections {
        let name = String::from_utf8_lossy(&section.name)
            .trim_end_matches('\0')
            .to_string();
        let vaddr = image_base + section.virtual_address as u64;
        let characteristics = section.characteristics;

        if characteristics & IMAGE_SCN_CNT_CODE != 0 {
            // Code section (e.g. .text).
            let data = extract_section_data(&bytes, section);
            code_sections.push(CodeSection {
                name,
                addr: vaddr,
                data,
            });
        } else if characteristics & IMAGE_SCN_CNT_UNINITIALIZED_DATA != 0 {
            // BSS-like section — zero-filled.
            let size = section.virtual_size as usize;
            data_sections.push(DataSection {
                name,
                addr: vaddr,
                data: vec![0u8; size],
                writable: true,
            });
        } else {
            // Data section (.data, .rdata, etc.).
            let writable = characteristics & IMAGE_SCN_MEM_WRITE != 0;
            let data = extract_section_data(&bytes, section);
            if !data.is_empty() {
                data_sections.push(DataSection {
                    name,
                    addr: vaddr,
                    data,
                    writable,
                });
            }
        }
    }

    if code_sections.is_empty() {
        return Err(Error::NoCodeSections);
    }

    // Extract symbols from the export table.
    let mut symbols = HashMap::new();
    for export in &pe.exports {
        if let Some(ref name) = export.name {
            let addr = image_base + export.rva as u64;
            symbols.insert(addr, name.to_string());
        }
    }

    // Collect imported DLL names.
    let import_dlls: Vec<String> = pe
        .imports
        .iter()
        .map(|imp| imp.dll.to_string())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    let is_static = import_dlls.is_empty();

    Ok(ParsedPE {
        bytes,
        arch: "x86_64".to_string(),
        entry_point,
        code_sections,
        data_sections,
        symbols,
        import_dlls,
        is_static,
    })
}

/// Extract raw section data from the PE file bytes.
fn extract_section_data(
    file_bytes: &[u8],
    section: &goblin::pe::section_table::SectionTable,
) -> Vec<u8> {
    let offset = section.pointer_to_raw_data as usize;
    let raw_size = section.size_of_raw_data as usize;
    let virtual_size = section.virtual_size as usize;

    if offset == 0 || raw_size == 0 {
        return vec![0u8; virtual_size];
    }

    let end = (offset + raw_size).min(file_bytes.len());
    if offset >= file_bytes.len() {
        return vec![0u8; virtual_size];
    }

    let mut data = file_bytes[offset..end].to_vec();
    // Pad to virtual size if needed.
    if data.len() < virtual_size {
        data.resize(virtual_size, 0);
    }
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reject_non_pe() {
        let result = parse_bytes(b"not a PE file".to_vec(), Path::new("fake.bin"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, Error::NotPE { .. }));
    }

    #[test]
    fn reject_empty() {
        let result = parse_bytes(vec![], Path::new("empty"));
        assert!(result.is_err());
    }

    #[test]
    fn reject_mz_only() {
        // Just an MZ signature with no valid PE header behind it.
        let mut data = vec![0u8; 64];
        data[0] = b'M';
        data[1] = b'Z';
        let result = parse_bytes(data, Path::new("mz_only.exe"));
        assert!(result.is_err());
    }
}
