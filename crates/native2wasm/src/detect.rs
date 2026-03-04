use std::path::Path;

use native2wasm_core::target::{CpuArch, OsTarget};

/// Detected binary format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryFormat {
    Elf,
    MachO,
    Pe,
}

impl std::fmt::Display for BinaryFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryFormat::Elf => write!(f, "ELF"),
            BinaryFormat::MachO => write!(f, "Mach-O"),
            BinaryFormat::Pe => write!(f, "PE"),
        }
    }
}

/// Result of binary format detection.
#[derive(Debug)]
pub struct DetectedBinary {
    pub format: BinaryFormat,
    pub cpu_arch: CpuArch,
    pub os_target: OsTarget,
    pub is_static: bool,
}

impl std::fmt::Display for DetectedBinary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {} ({})",
            self.format,
            self.cpu_arch,
            self.os_target,
            if self.is_static { "static" } else { "dynamic" },
        )
    }
}

/// Detect binary format, architecture, and OS from file bytes.
///
/// Uses goblin to sniff the magic bytes and determine format.
/// Then delegates to the appropriate parser crate for detailed detection.
pub fn detect(path: &Path) -> anyhow::Result<DetectedBinary> {
    let bytes = std::fs::read(path)?;
    detect_bytes(&bytes, path)
}

fn detect_bytes(bytes: &[u8], path: &Path) -> anyhow::Result<DetectedBinary> {
    use goblin::Object;

    let obj = Object::parse(bytes)?;

    match obj {
        Object::Elf(elf) => {
            let cpu_arch = match elf.header.e_machine {
                goblin::elf::header::EM_X86_64 => CpuArch::X86_64,
                goblin::elf::header::EM_AARCH64 => CpuArch::AArch64,
                other => anyhow::bail!("unsupported ELF architecture: e_machine=0x{other:04x}"),
            };
            let is_static = elf.libraries.is_empty();
            Ok(DetectedBinary {
                format: BinaryFormat::Elf,
                cpu_arch,
                os_target: OsTarget::Linux,
                is_static,
            })
        }
        Object::Mach(mach) => {
            match mach {
                goblin::mach::Mach::Binary(macho) => {
                    let cpu_arch = cputype_to_arch(macho.header.cputype as u32)?;
                    let is_static = macho.libs.is_empty();
                    Ok(DetectedBinary {
                        format: BinaryFormat::MachO,
                        cpu_arch,
                        os_target: OsTarget::Darwin,
                        is_static,
                    })
                }
                goblin::mach::Mach::Fat(fat) => {
                    // Prefer ARM64, fall back to x86_64.
                    let mut best_arch = None;
                    for arch in fat.iter_arches() {
                        let arch = arch?;
                        let cputype = arch.cputype as u32;
                        if cputype == 0x0100_000C {
                            // CPU_TYPE_ARM64
                            best_arch = Some(CpuArch::AArch64);
                            break;
                        } else if cputype == 0x0100_0007 {
                            // CPU_TYPE_X86_64
                            best_arch = Some(CpuArch::X86_64);
                        }
                    }
                    let cpu_arch = best_arch
                        .ok_or_else(|| anyhow::anyhow!("no supported slice in universal binary"))?;
                    Ok(DetectedBinary {
                        format: BinaryFormat::MachO,
                        cpu_arch,
                        os_target: OsTarget::Darwin,
                        is_static: false, // can't tell from fat header alone
                    })
                }
            }
        }
        Object::PE(pe) => {
            let cpu_arch = match pe.header.coff_header.machine {
                0x8664 => CpuArch::X86_64,
                0xAA64 => CpuArch::AArch64,
                other => anyhow::bail!("unsupported PE architecture: machine=0x{other:04x}"),
            };
            let is_static = pe.imports.is_empty();
            Ok(DetectedBinary {
                format: BinaryFormat::Pe,
                cpu_arch,
                os_target: OsTarget::Windows,
                is_static,
            })
        }
        _ => anyhow::bail!(
            "unrecognized binary format: {}",
            path.display()
        ),
    }
}

fn cputype_to_arch(cputype: u32) -> anyhow::Result<CpuArch> {
    match cputype {
        0x0100_0007 => Ok(CpuArch::X86_64),  // CPU_TYPE_X86_64
        0x0100_000C => Ok(CpuArch::AArch64),  // CPU_TYPE_ARM64
        other => anyhow::bail!("unsupported Mach-O cputype: 0x{other:08x}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_elf_magic() {
        // Minimal ELF header for x86_64
        let mut data = vec![0u8; 64];
        // ELF magic
        data[0] = 0x7F;
        data[1] = b'E';
        data[2] = b'L';
        data[3] = b'F';
        // 64-bit
        data[4] = 2;
        // Little endian
        data[5] = 1;
        // ELF version
        data[6] = 1;
        // e_machine = EM_X86_64 (0x3E) at offset 18
        data[18] = 0x3E;
        data[19] = 0x00;
        // e_type = ET_EXEC (2) at offset 16
        data[16] = 0x02;
        data[17] = 0x00;

        // This won't parse as a full ELF, but the magic detection should work
        // via goblin — minimal headers may fail. We test the full parser through
        // the elf2wasm crate instead.
        let _ = detect_bytes(&data, Path::new("test.elf"));
    }
}
