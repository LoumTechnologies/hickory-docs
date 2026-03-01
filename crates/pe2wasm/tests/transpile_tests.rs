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

/// Test that a minimal PE header with wrong COFF machine type is rejected.
#[test]
fn reject_wrong_machine() {
    // Build a minimal PE with machine type = ARM64 (0xAA64).
    // This is enough structure for goblin to parse the COFF header.
    let pe_bytes = build_minimal_pe(0xAA64);
    let result = pe2wasm::pe::parse_bytes(pe_bytes, Path::new("arm64.exe"));
    assert!(result.is_err());
    let msg = format!("{}", result.unwrap_err());
    assert!(msg.contains("machine=0xaa64") || msg.contains("unsupported"));
}

/// Verify shim source files are properly embedded (compile-time check).
#[test]
fn shim_sources_embedded() {
    // The syscall_shim module includes shim sources via include_bytes!().
    // If any source file is missing, this test won't even compile.
    // We verify at runtime that the compile_shims function exists and is callable
    // (though it will fail without clang in PATH — we just test the plumbing).
    let tmp = tempfile::tempdir().unwrap();
    let result = pe2wasm::syscall_shim::compile_shims(tmp.path(), None);
    // This will fail if clang is not available, which is fine — we're testing
    // that the code paths exist and the sources are embedded.
    match result {
        Ok(objects) => {
            assert!(!objects.is_empty(), "should produce at least one object");
        }
        Err(e) => {
            let msg = format!("{e}");
            // Expected failures: clang not found, or compilation fails.
            assert!(
                msg.contains("not found") || msg.contains("compilation failed"),
                "unexpected error: {msg}"
            );
        }
    }
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

/// Test that error types have useful Display implementations.
#[test]
fn error_display() {
    use pe2wasm::error::Error;
    let err = Error::UnsupportedArch {
        arch: "arm64".to_string(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("arm64"));
    assert!(msg.contains("x86_64"));

    let err = Error::ToolNotFound {
        tool: "llc".to_string(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("llc"));

    let err = Error::UnsupportedImport {
        dll: "advapi32.dll".to_string(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("advapi32.dll"));
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
