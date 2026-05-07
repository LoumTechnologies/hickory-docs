//! Integration tests for native2wasm detection, strategy selection, and pipelines.

use std::path::Path;

use native2wasm::detect::{BinaryFormat, DetectedBinary};
use native2wasm::strategy::{Strategy, select_strategy};
use native2wasm_core::target::{CpuArch, OsTarget};

// ---------------------------------------------------------------------------
// Helpers: build minimal binary headers for detection tests
// ---------------------------------------------------------------------------

/// Build a minimal static x86_64 ELF.
fn build_x86_64_elf(is_static: bool) -> Vec<u8> {
    let mut buf = vec![0u8; 256];

    // ELF magic
    buf[0] = 0x7F;
    buf[1] = b'E';
    buf[2] = b'L';
    buf[3] = b'F';
    // ELFCLASS64
    buf[4] = 2;
    // ELFDATA2LSB (little-endian)
    buf[5] = 1;
    // EI_VERSION
    buf[6] = 1;
    // EI_OSABI = Linux
    buf[7] = 0;
    // e_type = ET_EXEC (2)
    buf[16] = 0x02;
    buf[17] = 0x00;
    // e_machine = EM_X86_64 (0x3E)
    buf[18] = 0x3E;
    buf[19] = 0x00;
    // e_version = 1
    buf[20] = 0x01;
    // e_ehsize = 64
    buf[52] = 64;
    buf[53] = 0;

    if !is_static {
        // Add a minimal program header for PT_DYNAMIC to make it look dynamic.
        // e_phoff = 64 (right after ELF header)
        buf[32] = 64;
        // e_phentsize = 56
        buf[54] = 56;
        buf[55] = 0;
        // e_phnum = 1
        buf[56] = 1;
        buf[57] = 0;

        // PT_INTERP (type = 3) at offset 64 indicates dynamic linking
        buf[64] = 3; // p_type = PT_INTERP
    }

    buf
}

/// Build a minimal ARM64 ELF.
fn build_arm64_elf() -> Vec<u8> {
    let mut buf = vec![0u8; 256];

    buf[0] = 0x7F;
    buf[1] = b'E';
    buf[2] = b'L';
    buf[3] = b'F';
    buf[4] = 2; // 64-bit
    buf[5] = 1; // little-endian
    buf[6] = 1;
    buf[16] = 0x02; // ET_EXEC
    // e_machine = EM_AARCH64 (0xB7)
    buf[18] = 0xB7;
    buf[19] = 0x00;
    buf[20] = 0x01;
    buf[52] = 64;

    buf
}

// ---------------------------------------------------------------------------
// Detection tests
// ---------------------------------------------------------------------------

#[test]
fn detect_x86_64_elf() {
    let data = build_x86_64_elf(true);
    let tmpdir = tempfile::tempdir().unwrap();
    let path = tmpdir.path().join("test.elf");
    std::fs::write(&path, &data).unwrap();

    let detected = native2wasm::detect::detect(&path).unwrap();
    assert_eq!(detected.format, BinaryFormat::Elf);
    assert_eq!(detected.cpu_arch, CpuArch::X86_64);
    assert_eq!(detected.os_target, OsTarget::Linux);
}

#[test]
fn detect_arm64_elf() {
    let data = build_arm64_elf();
    let tmpdir = tempfile::tempdir().unwrap();
    let path = tmpdir.path().join("test_arm64.elf");
    std::fs::write(&path, &data).unwrap();

    let detected = native2wasm::detect::detect(&path).unwrap();
    assert_eq!(detected.format, BinaryFormat::Elf);
    assert_eq!(detected.cpu_arch, CpuArch::AArch64);
    assert_eq!(detected.os_target, OsTarget::Linux);
}

// ---------------------------------------------------------------------------
// Strategy selection tests
// ---------------------------------------------------------------------------

#[test]
fn strategy_auto_static_selects_aot() {
    let detected = DetectedBinary {
        format: BinaryFormat::Elf,
        cpu_arch: CpuArch::X86_64,
        os_target: OsTarget::Linux,
        is_static: true,
    };
    let strategy = select_strategy(&detected, Strategy::Auto);
    assert_eq!(strategy, Strategy::Aot);
}

#[test]
fn strategy_auto_dynamic_x86_selects_emu() {
    let detected = DetectedBinary {
        format: BinaryFormat::Elf,
        cpu_arch: CpuArch::X86_64,
        os_target: OsTarget::Linux,
        is_static: false,
    };
    let strategy = select_strategy(&detected, Strategy::Auto);
    assert_eq!(strategy, Strategy::Emu);
}

#[test]
fn strategy_auto_dynamic_arm64_linux_selects_emu() {
    let detected = DetectedBinary {
        format: BinaryFormat::Elf,
        cpu_arch: CpuArch::AArch64,
        os_target: OsTarget::Linux,
        is_static: false,
    };
    let strategy = select_strategy(&detected, Strategy::Auto);
    assert_eq!(strategy, Strategy::Emu);
}

#[test]
fn strategy_auto_macho_x86_dynamic_selects_emu() {
    let detected = DetectedBinary {
        format: BinaryFormat::MachO,
        cpu_arch: CpuArch::X86_64,
        os_target: OsTarget::Darwin,
        is_static: false,
    };
    let strategy = select_strategy(&detected, Strategy::Auto);
    assert_eq!(strategy, Strategy::Emu);
}

#[test]
fn strategy_auto_pe_x86_dynamic_selects_emu() {
    let detected = DetectedBinary {
        format: BinaryFormat::Pe,
        cpu_arch: CpuArch::X86_64,
        os_target: OsTarget::Windows,
        is_static: false,
    };
    let strategy = select_strategy(&detected, Strategy::Auto);
    assert_eq!(strategy, Strategy::Emu);
}

#[test]
fn strategy_auto_macho_arm64_dynamic_selects_vm() {
    let detected = DetectedBinary {
        format: BinaryFormat::MachO,
        cpu_arch: CpuArch::AArch64,
        os_target: OsTarget::Darwin,
        is_static: false,
    };
    let strategy = select_strategy(&detected, Strategy::Auto);
    assert_eq!(strategy, Strategy::Vm);
}

#[test]
fn strategy_explicit_overrides_auto() {
    let detected = DetectedBinary {
        format: BinaryFormat::Elf,
        cpu_arch: CpuArch::X86_64,
        os_target: OsTarget::Linux,
        is_static: true,
    };
    // Even though it's static (would select AOT), explicit Emu overrides.
    let strategy = select_strategy(&detected, Strategy::Emu);
    assert_eq!(strategy, Strategy::Emu);
}

// ---------------------------------------------------------------------------
// End-to-end tests (require artifacts, run with --ignored)
// ---------------------------------------------------------------------------

#[test]
#[ignore]
fn emu_x86_64_end_to_end() {
    // Requires: blink.wasm in cache
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/hello_x86_64.elf");
    if !fixture.exists() {
        eprintln!("SKIP: fixture not found at {}", fixture.display());
        return;
    }

    let detected = native2wasm::detect::detect(&fixture).unwrap();
    assert_eq!(detected.format, BinaryFormat::Elf);
    assert_eq!(detected.cpu_arch, CpuArch::X86_64);

    let tmpdir = tempfile::tempdir().unwrap();
    let output = tmpdir.path().join("hello.wasm");

    let result = native2wasm::run_emu(&fixture, &detected, &output, None);
    assert!(result.is_ok(), "EMU pipeline failed: {:?}", result.err());
    assert!(output.exists(), "output WASM not produced");
}

#[test]
#[ignore]
fn aot_x86_64_end_to_end() {
    // Requires: Remill + clang + wasm-ld + llc
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/hello_x86_64.elf");
    if !fixture.exists() {
        eprintln!("SKIP: fixture not found at {}", fixture.display());
        return;
    }

    let detected = native2wasm::detect::detect(&fixture).unwrap();
    assert_eq!(detected.format, BinaryFormat::Elf);
    assert!(detected.is_static);

    let tmpdir = tempfile::tempdir().unwrap();
    let output = tmpdir.path().join("hello.wasm");

    let result = native2wasm::run_aot(&fixture, &detected, &output, 2, false, None, None);
    assert!(result.is_ok(), "AOT pipeline failed: {:?}", result.err());
    assert!(output.exists(), "output WASM not produced");
}
