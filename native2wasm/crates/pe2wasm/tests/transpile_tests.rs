use std::path::Path;

/// Test that non-PE files are rejected.
#[test]
fn reject_non_pe_file() {
    let result = pe2wasm::pe::parse_bytes(b"not a PE binary".to_vec(), Path::new("test"));
    assert!(result.is_err());
}

/// Test that empty input is rejected.
#[test]
fn reject_empty_input() {
    let result = pe2wasm::pe::parse_bytes(vec![], Path::new("empty"));
    assert!(result.is_err());
}

/// Test that a bare MZ header without valid PE signature is rejected.
#[test]
fn reject_mz_only() {
    let mut data = vec![0u8; 128];
    data[0] = b'M';
    data[1] = b'Z';
    let result = pe2wasm::pe::parse_bytes(data, Path::new("mz_only.exe"));
    assert!(result.is_err());
}

/// Test that a minimal PE header with unsupported COFF machine type is rejected.
#[test]
fn reject_wrong_machine() {
    // Build a minimal PE with machine type = i386 (0x014C), unsupported 32-bit.
    let pe_bytes = build_minimal_pe(0x014C);
    let result = pe2wasm::pe::parse_bytes(pe_bytes, Path::new("i386.exe"));
    assert!(result.is_err());
    let msg = format!("{}", result.unwrap_err());
    assert!(msg.contains("machine=0x014c") || msg.contains("unsupported"));
}

/// Verify CLI parsing works.
#[test]
fn cli_parses_defaults() {
    use clap::Parser;
    let cli = pe2wasm::cli::Cli::parse_from(["pe2wasm", "input.exe"]);
    assert_eq!(cli.input.to_str().unwrap(), "input.exe");
    assert_eq!(cli.output.to_str().unwrap(), "output.wasm");
    assert_eq!(cli.opt_level, 2);
    assert!(!cli.emit_llvm);
    assert!(!cli.verbose);
}

#[test]
fn cli_parses_all_flags() {
    use clap::Parser;
    let cli = pe2wasm::cli::Cli::parse_from([
        "pe2wasm",
        "hello.exe",
        "-o",
        "hello.wasm",
        "-O",
        "3",
        "--emit-llvm",
        "--verbose",
    ]);
    assert_eq!(cli.input.to_str().unwrap(), "hello.exe");
    assert_eq!(cli.output.to_str().unwrap(), "hello.wasm");
    assert_eq!(cli.opt_level, 3);
    assert!(cli.emit_llvm);
    assert!(cli.verbose);
}

/// Test that parse error types have useful Display implementations.
#[test]
fn error_display() {
    use pe2wasm::pe::ParseError;
    let err = ParseError::UnsupportedArch {
        arch: "arm64".to_string(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("arm64"));
    assert!(msg.contains("supported"));

    let err = ParseError::NoEntryPoint;
    let msg = format!("{err}");
    assert!(msg.contains("entry point"));
}

/// Build a minimal PE binary for testing.
/// Creates enough structure for goblin to parse the COFF header.
fn build_minimal_pe(machine: u16) -> Vec<u8> {
    let mut buf = vec![0u8; 512];

    // DOS header
    buf[0] = b'M';
    buf[1] = b'Z';
    // e_lfanew: offset to PE signature (at offset 0x3C)
    buf[0x3C] = 0x80;

    // PE signature at 0x80: "PE\0\0"
    buf[0x80] = b'P';
    buf[0x81] = b'E';
    buf[0x82] = 0;
    buf[0x83] = 0;

    // COFF header starts at 0x84
    // Machine type (2 bytes LE)
    buf[0x84] = (machine & 0xFF) as u8;
    buf[0x85] = ((machine >> 8) & 0xFF) as u8;
    // NumberOfSections = 0
    buf[0x86] = 0;
    buf[0x87] = 0;

    buf
}
