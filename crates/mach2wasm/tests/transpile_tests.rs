use std::path::Path;

/// Test that non-Mach-O files are rejected.
#[test]
fn reject_non_macho_file() {
    let result = mach2wasm::macho::parse_bytes(b"not a macho binary".to_vec(), Path::new("test"));
    assert!(result.is_err());
}

/// Test that empty input is rejected.
#[test]
fn reject_empty_input() {
    let result = mach2wasm::macho::parse_bytes(vec![], Path::new("empty"));
    assert!(result.is_err());
}

/// Test that a minimal valid-looking Mach-O header with wrong arch is rejected.
#[test]
fn reject_wrong_arch() {
    // Minimal Mach-O header: magic=MH_MAGIC_64, cputype=ARM64 (0x0100000C)
    let mut header = vec![0u8; 32];
    // MH_MAGIC_64 = 0xFEEDFACF
    header[0] = 0xCF;
    header[1] = 0xFA;
    header[2] = 0xED;
    header[3] = 0xFE;
    // cputype = ARM64 (0x0100000C) in little-endian
    header[4] = 0x0C;
    header[5] = 0x00;
    header[6] = 0x00;
    header[7] = 0x01;

    let result = mach2wasm::macho::parse_bytes(header, Path::new("arm64.bin"));
    assert!(result.is_err());
}

/// Verify shim source files are properly embedded (compile-time check).
#[test]
fn shim_sources_embedded() {
    // The syscall_shim module includes shim sources via include_bytes!().
    // If any source file is missing, this test won't even compile.
    // We verify at runtime that the compile_shims function exists and is callable
    // (though it will fail without clang in PATH — we just test the plumbing).
    let tmp = tempfile::tempdir().unwrap();
    let result = mach2wasm::syscall_shim::compile_shims(tmp.path(), None);
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
    let cli = mach2wasm::cli::Cli::parse_from(["mach2wasm", "input.bin"]);
    assert_eq!(cli.input.to_str().unwrap(), "input.bin");
    assert_eq!(cli.output.to_str().unwrap(), "output.wasm");
    assert_eq!(cli.opt_level, 2);
    assert!(!cli.emit_llvm);
    assert!(!cli.verbose);
}

#[test]
fn cli_parses_all_flags() {
    use clap::Parser;
    let cli = mach2wasm::cli::Cli::parse_from([
        "mach2wasm",
        "hello",
        "-o",
        "hello.wasm",
        "-O",
        "3",
        "--emit-llvm",
        "--verbose",
    ]);
    assert_eq!(cli.input.to_str().unwrap(), "hello");
    assert_eq!(cli.output.to_str().unwrap(), "hello.wasm");
    assert_eq!(cli.opt_level, 3);
    assert!(cli.emit_llvm);
    assert!(cli.verbose);
}

/// Test that error types have useful Display implementations.
#[test]
fn error_display() {
    use mach2wasm::error::Error;
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
}
