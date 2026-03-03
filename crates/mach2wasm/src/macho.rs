use std::collections::HashMap;
use std::path::Path;

use goblin::mach::{Mach, MachO, header::MH_EXECUTE};

use native2wasm_core::binary::{
    CodeSection, DataSection, ParsedBinary,
};
use native2wasm_core::target::OsTarget;

/// OS-specific parse error type.
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("unsupported architecture: {arch} (only x86_64 is supported)")]
    UnsupportedArch { arch: String },

    #[error("not a Mach-O binary: {path}")]
    NotMachO { path: std::path::PathBuf },

    #[error("no x86_64 slice found in universal binary: {path}")]
    NoX86Slice { path: std::path::PathBuf },

    #[error("no entry point found in Mach-O binary")]
    NoEntryPoint,

    #[error("no code sections found in Mach-O binary")]
    NoCodeSections,

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Parsed representation of a Mach-O x86_64 binary.
#[derive(Debug)]
pub struct ParsedMachO {
    pub bytes: Vec<u8>,
    pub arch: String,
    pub entry_point: u64,
    pub code_sections: Vec<CodeSection>,
    pub data_sections: Vec<DataSection>,
    pub symbols: HashMap<u64, String>,
    pub dylib_paths: Vec<String>,
    pub is_static: bool,
}

impl ParsedBinary for ParsedMachO {
    fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    fn arch(&self) -> &str {
        &self.arch
    }

    fn entry_point(&self) -> u64 {
        self.entry_point
    }

    fn code_sections(&self) -> &[CodeSection] {
        &self.code_sections
    }

    fn data_sections(&self) -> &[DataSection] {
        &self.data_sections
    }

    fn symbols(&self) -> &HashMap<u64, String> {
        &self.symbols
    }

    fn is_static(&self) -> bool {
        self.is_static
    }

    fn dynamic_deps(&self) -> &[String] {
        &self.dylib_paths
    }

    fn os_target(&self) -> OsTarget {
        OsTarget::Darwin
    }
}

/// Parse a Mach-O file from disk.
pub fn parse(path: &Path) -> Result<ParsedMachO, ParseError> {
    let bytes = std::fs::read(path).map_err(|e| {
        ParseError::Io(std::io::Error::new(e.kind(), format!("{}: {e}", path.display())))
    })?;
    parse_bytes(bytes, path)
}

/// Parse a Mach-O from raw bytes.
pub fn parse_bytes(bytes: Vec<u8>, path: &Path) -> Result<ParsedMachO, ParseError> {
    // Determine if this is a fat binary and find the x86_64 slice offset/size.
    let slice_range = {
        let mach = Mach::parse(&bytes).map_err(|_| ParseError::NotMachO {
            path: path.to_path_buf(),
        })?;
        match mach {
            Mach::Binary(_) => None, // whole file is the binary
            Mach::Fat(fat) => {
                let mut found = None;
                for arch in fat.iter_arches() {
                    let arch = arch.map_err(|_| ParseError::NotMachO {
                        path: path.to_path_buf(),
                    })?;
                    // CPU_TYPE_X86_64 = 0x01000007
                    if arch.cputype == 0x0100_0007 {
                        found = Some((arch.offset as usize, arch.size as usize));
                        break;
                    }
                }
                match found {
                    Some(range) => Some(range),
                    None => {
                        return Err(ParseError::NoX86Slice {
                            path: path.to_path_buf(),
                        })
                    }
                }
            }
        }
    };
    // The `Mach` borrow is now dropped. We can move `bytes` freely.

    if let Some((offset, size)) = slice_range {
        let slice_bytes = bytes[offset..offset + size].to_vec();
        let macho = MachO::parse(&slice_bytes, 0).map_err(|_| ParseError::NotMachO {
            path: path.to_path_buf(),
        })?;
        let mut result = extract(&macho, path)?;
        result.bytes = slice_bytes;
        Ok(result)
    } else {
        let macho = MachO::parse(&bytes, 0).map_err(|_| ParseError::NotMachO {
            path: path.to_path_buf(),
        })?;
        let mut result = extract(&macho, path)?;
        result.bytes = bytes;
        Ok(result)
    }
}

/// Extract all relevant data from a parsed MachO into owned types.
/// The `bytes` field of the result is left empty; the caller fills it in.
fn extract(macho: &MachO<'_>, path: &Path) -> Result<ParsedMachO, ParseError> {
    // Validate architecture: must be x86_64 (cputype 0x01000007).
    let cputype = macho.header.cputype;
    if cputype != 0x0100_0007 {
        return Err(ParseError::UnsupportedArch {
            arch: format!("cputype=0x{cputype:08x}"),
        });
    }

    // Validate file type.
    if macho.header.filetype != MH_EXECUTE {
        return Err(ParseError::NotMachO {
            path: path.to_path_buf(),
        });
    }

    // Use goblin's pre-extracted entry point.
    let entry_point = if macho.entry != 0 {
        macho.entry
    } else {
        return Err(ParseError::NoEntryPoint);
    };

    // Extract code and data sections.
    let mut code_sections = Vec::new();
    let mut data_sections = Vec::new();

    for segment in &macho.segments {
        let seg_name = segment.name().unwrap_or("??");
        if let Ok(sections) = segment.sections() {
            for (section, section_data) in &sections {
                let sect_name = section.name().unwrap_or("??");
                let addr = section.addr;
                let full_name = format!("{seg_name},{sect_name}");

                match seg_name {
                    "__TEXT" => {
                        code_sections.push(CodeSection {
                            name: full_name,
                            addr,
                            data: section_data.to_vec(),
                        });
                    }
                    "__DATA" | "__DATA_CONST" => {
                        let writable = seg_name == "__DATA";
                        data_sections.push(DataSection {
                            name: full_name,
                            addr,
                            data: section_data.to_vec(),
                            writable,
                        });
                    }
                    _ => {
                        // Check for BSS-like sections (zero-filled).
                        let s_type = section.flags & 0xFF;
                        if s_type == 1 {
                            // S_ZEROFILL
                            let size = section.size as usize;
                            data_sections.push(DataSection {
                                name: full_name,
                                addr,
                                data: vec![0u8; size],
                                writable: true,
                            });
                        }
                    }
                }
            }
        }
    }

    if code_sections.is_empty() {
        return Err(ParseError::NoCodeSections);
    }

    // Extract symbols.
    let mut symbols = HashMap::new();
    if let Some(ref symtab) = macho.symbols {
        for sym in symtab.iter() {
            if let Ok((name, nlist)) = sym {
                if nlist.n_value != 0 {
                    symbols.insert(nlist.n_value, name.to_string());
                }
            }
        }
    }

    // Use goblin's pre-extracted dylib list.
    let dylib_paths: Vec<String> = macho.libs.iter().map(|s| s.to_string()).collect();
    let is_static = dylib_paths.is_empty();

    Ok(ParsedMachO {
        bytes: Vec::new(),
        arch: "x86_64".to_string(),
        entry_point,
        code_sections,
        data_sections,
        symbols,
        dylib_paths,
        is_static,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reject_non_macho() {
        let result = parse_bytes(b"not a macho file".to_vec(), Path::new("fake.bin"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ParseError::NotMachO { .. }));
    }

    #[test]
    fn reject_empty() {
        let result = parse_bytes(vec![], Path::new("empty"));
        assert!(result.is_err());
    }
}
