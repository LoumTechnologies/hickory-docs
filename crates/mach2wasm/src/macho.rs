use std::collections::HashMap;
use std::path::Path;

use goblin::mach::{
    Mach, MachO, SingleArch,
    header::MH_EXECUTE,
    load_command::CommandVariant,
};

use crate::error::Error;

/// A parsed code section from the Mach-O binary.
#[derive(Debug, Clone)]
pub struct CodeSection {
    pub name: String,
    pub segment: String,
    pub addr: u64,
    pub data: Vec<u8>,
}

/// A parsed data section from the Mach-O binary.
#[derive(Debug, Clone)]
pub struct DataSection {
    pub name: String,
    pub segment: String,
    pub addr: u64,
    pub data: Vec<u8>,
    pub writable: bool,
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

/// Parse a Mach-O file from disk.
pub fn parse(path: &Path) -> Result<ParsedMachO, Error> {
    let bytes = std::fs::read(path).map_err(|e| {
        Error::Io(std::io::Error::new(e.kind(), format!("{}: {e}", path.display())))
    })?;
    parse_bytes(bytes, path)
}

/// Parse a Mach-O from raw bytes.
pub fn parse_bytes(bytes: Vec<u8>, path: &Path) -> Result<ParsedMachO, Error> {
    let mach = Mach::parse(&bytes).map_err(|_| Error::NotMachO {
        path: path.to_path_buf(),
    })?;

    match mach {
        Mach::Binary(macho) => parse_single(macho, bytes, path),
        Mach::Fat(fat) => {
            // Search for x86_64 slice in universal binary.
            for (idx, arch) in fat.iter_arches().enumerate() {
                let arch = arch.map_err(|_| Error::NotMachO {
                    path: path.to_path_buf(),
                })?;
                // CPU_TYPE_X86_64 = 0x01000007
                if arch.cputype == 0x0100_0007 {
                    let offset = arch.offset as usize;
                    let size = arch.size as usize;
                    let slice_bytes = bytes[offset..offset + size].to_vec();
                    let macho =
                        MachO::parse(&slice_bytes, 0).map_err(|_| Error::NotMachO {
                            path: path.to_path_buf(),
                        })?;
                    return parse_single(macho, slice_bytes, path);
                }
                // Also try by index with SingleArch
                let _ = idx;
            }
            Err(Error::NoX86Slice {
                path: path.to_path_buf(),
            })
        }
    }
}

fn parse_single(macho: MachO<'_>, bytes: Vec<u8>, path: &Path) -> Result<ParsedMachO, Error> {
    // Validate architecture: must be x86_64 (cputype 0x01000007).
    let cputype = macho.header.cputype;
    if cputype != 0x0100_0007 {
        return Err(Error::UnsupportedArch {
            arch: format!("cputype=0x{cputype:08x}"),
        });
    }

    // Validate file type.
    if macho.header.filetype != MH_EXECUTE {
        return Err(Error::NotMachO {
            path: path.to_path_buf(),
        });
    }

    // Extract entry point from LC_MAIN or LC_UNIXTHREAD.
    let entry_point = find_entry_point(&macho)?;

    // Extract code and data sections.
    let mut code_sections = Vec::new();
    let mut data_sections = Vec::new();

    for segment in &macho.segments {
        let seg_name = segment.name().unwrap_or("??");
        for (section, section_data) in segment.sections().into_iter().flatten() {
            let sect_name = section.name().unwrap_or("??");
            let addr = section.addr;
            let full_name = format!("{seg_name},{sect_name}");

            match seg_name {
                "__TEXT" => {
                    code_sections.push(CodeSection {
                        name: full_name,
                        segment: seg_name.to_string(),
                        addr,
                        data: section_data.to_vec(),
                    });
                }
                "__DATA" | "__DATA_CONST" => {
                    let writable = seg_name == "__DATA";
                    data_sections.push(DataSection {
                        name: full_name,
                        segment: seg_name.to_string(),
                        addr,
                        data: section_data.to_vec(),
                        writable,
                    });
                }
                "__BSS" => {
                    // BSS: zero-filled, writable
                    let size = section.size as usize;
                    data_sections.push(DataSection {
                        name: full_name,
                        segment: seg_name.to_string(),
                        addr,
                        data: vec![0u8; size],
                        writable: true,
                    });
                }
                _ => {}
            }
        }
    }

    if code_sections.is_empty() {
        return Err(Error::NoCodeSections);
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

    // Extract dylib paths from LC_LOAD_DYLIB.
    let mut dylib_paths = Vec::new();
    for cmd in &macho.load_commands {
        if let CommandVariant::LoadDylib(dylib)
        | CommandVariant::LoadWeakDylib(dylib)
        | CommandVariant::ReexportDylib(dylib) = cmd.command
        {
            let name_offset = dylib.dylib.name as usize;
            if let Some(name_bytes) = cmd.data.get(name_offset..) {
                if let Ok(name) = std::str::from_utf8(
                    &name_bytes[..name_bytes.iter().position(|&b| b == 0).unwrap_or(name_bytes.len())],
                ) {
                    dylib_paths.push(name.to_string());
                }
            }
        }
    }

    let is_static = dylib_paths.is_empty();

    Ok(ParsedMachO {
        bytes,
        arch: "x86_64".to_string(),
        entry_point,
        code_sections,
        data_sections,
        symbols,
        dylib_paths,
        is_static,
    })
}

fn find_entry_point(macho: &MachO<'_>) -> Result<u64, Error> {
    for cmd in &macho.load_commands {
        match cmd.command {
            // LC_MAIN (0x80000028)
            CommandVariant::Main(main_cmd) => {
                // entryoff is relative to __TEXT segment vmaddr.
                let text_vmaddr = macho
                    .segments
                    .iter()
                    .find(|s| s.name().unwrap_or("") == "__TEXT")
                    .map(|s| s.vmaddr)
                    .unwrap_or(0);
                return Ok(text_vmaddr + main_cmd.entryoff);
            }
            // LC_UNIXTHREAD — entry point in thread state registers.
            CommandVariant::Unixthread(thread_cmd) => {
                // For x86_64, the RIP is at offset 16 in the thread state
                // (after flavor + count fields = 8 bytes, then rax..rbp = 8 regs * 8 bytes before rip).
                // Thread state: flavor(4) + count(4) + rax(8) + rbx(8) + rcx(8) + rdx(8) +
                //               rdi(8) + rsi(8) + rbp(8) + rsp(8) + r8-r15(64) + rip(8)
                // rip is at offset 16*8 + 8 = 136 from start of thread state data.
                // But the actual layout for x86_64_thread_state: rip is register 16.
                let data = &cmd.data;
                // Skip past the cmd header (8 bytes for flavor + count).
                if data.len() >= 8 + 21 * 8 {
                    // rip is the 17th register (index 16) in the x86_64 thread state.
                    let rip_offset = 8 + 16 * 8;
                    let rip = u64::from_le_bytes(
                        data[rip_offset..rip_offset + 8].try_into().unwrap(),
                    );
                    return Ok(rip);
                }
                // Fallback: use the instruction_pointer from goblin if available.
                return Ok(thread_cmd.instruction_pointer);
            }
            _ => {}
        }
    }
    Err(Error::NoEntryPoint)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reject_non_macho() {
        let result = parse_bytes(b"not a macho file".to_vec(), Path::new("fake.bin"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, Error::NotMachO { .. }));
    }

    #[test]
    fn reject_empty() {
        let result = parse_bytes(vec![], Path::new("empty"));
        assert!(result.is_err());
    }

    // Valid Mach-O parsing is tested via integration tests with fixture binaries.
}
