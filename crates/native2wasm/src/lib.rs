pub mod detect;
pub mod strategy;

use std::path::Path;

use anyhow::{Context, Result, bail};

use crate::detect::{BinaryFormat, DetectedBinary};
use emu_core::pipeline::{EmulationConfig, run_emulation};
use native2wasm_core::target::{CpuArch, OsTarget};
use native2wasm_core::pipeline::{PipelineConfig, run_pipeline};

/// Run the AOT transpilation pipeline for the detected binary.
///
/// Dispatches to the appropriate format-specific parser (elf2wasm, mach2wasm,
/// pe2wasm) and then runs the shared pipeline.
pub fn run_aot(
    input: &Path,
    detected: &DetectedBinary,
    output: &Path,
    opt_level: u8,
    emit_llvm: bool,
    semantics_dir: Option<&Path>,
    wasi_sysroot: Option<&Path>,
) -> Result<()> {
    match detected.format {
        BinaryFormat::Elf => {
            let binary = elf2wasm::elf::parse(input)
                .with_context(|| format!("failed to parse ELF: {}", input.display()))?;

            log::info!(
                "ELF: arch={}, entry=0x{:x}, {} code, {} data, {} syms",
                binary.arch,
                binary.entry_point,
                binary.code_sections.len(),
                binary.data_sections.len(),
                binary.symbols.len(),
            );

            let config = PipelineConfig {
                output: output.to_path_buf(),
                opt_level,
                emit_llvm,
                semantics_dir: semantics_dir.map(|p| p.to_path_buf()),
                wasi_sysroot: wasi_sysroot.map(|p| p.to_path_buf()),
                os_headers: elf2wasm::os_headers(),
                os_sources: elf2wasm::os_sources(),
            };
            run_pipeline(&binary, &config)
        }
        BinaryFormat::MachO => {
            let binary = mach2wasm::macho::parse(input)
                .with_context(|| format!("failed to parse Mach-O: {}", input.display()))?;

            log::info!(
                "Mach-O: arch={}, entry=0x{:x}, {} code, {} data, {} syms",
                binary.arch,
                binary.entry_point,
                binary.code_sections.len(),
                binary.data_sections.len(),
                binary.symbols.len(),
            );

            let config = PipelineConfig {
                output: output.to_path_buf(),
                opt_level,
                emit_llvm,
                semantics_dir: semantics_dir.map(|p| p.to_path_buf()),
                wasi_sysroot: wasi_sysroot.map(|p| p.to_path_buf()),
                os_headers: mach2wasm::os_headers(),
                os_sources: mach2wasm::os_sources(),
            };
            run_pipeline(&binary, &config)
        }
        BinaryFormat::Pe => {
            let binary = pe2wasm::pe::parse(input)
                .with_context(|| format!("failed to parse PE: {}", input.display()))?;

            log::info!(
                "PE: arch={}, entry=0x{:x}, {} code, {} data, {} syms",
                binary.arch,
                binary.entry_point,
                binary.code_sections.len(),
                binary.data_sections.len(),
                binary.symbols.len(),
            );

            let config = PipelineConfig {
                output: output.to_path_buf(),
                opt_level,
                emit_llvm,
                semantics_dir: semantics_dir.map(|p| p.to_path_buf()),
                wasi_sysroot: wasi_sysroot.map(|p| p.to_path_buf()),
                os_headers: pe2wasm::os_headers(),
                os_sources: pe2wasm::os_sources(),
            };
            run_pipeline(&binary, &config)
        }
    }
}

/// Run the EMU strategy: user-mode emulation.
///
/// For x86_64 binaries, selects the appropriate provider based on OS:
/// - Linux ELF → BlinkProvider (direct blink execution)
/// - macOS Mach-O → BlinkMachoProvider (blink + Mach-O loader + XNU shims)
/// - Windows PE → BlinkPeProvider (blink + PE loader + Win32 shims)
///
/// For ARM64 binaries:
/// - Linux ELF → QemuTciProvider (QEMU TCI user-mode emulation)
pub fn run_emu(
    input: &Path,
    detected: &DetectedBinary,
    output: &Path,
    emulator_path: Option<&Path>,
) -> Result<()> {
    let provider: Box<dyn emu_core::provider::EmulatorProvider> = match detected.cpu_arch {
        CpuArch::X86_64 => {
            match (detected.format, detected.os_target) {
                (BinaryFormat::Elf, OsTarget::Linux) => {
                    if let Some(path) = emulator_path {
                        Box::new(emu_x86::BlinkProvider::with_path(path.to_path_buf()))
                    } else {
                        Box::new(emu_x86::BlinkProvider::new())
                    }
                }
                (BinaryFormat::MachO, OsTarget::Darwin) => {
                    log::info!(
                        "macOS x86_64 EMU: blink + Mach-O loader shim \
                         (basic POSIX calls only; complex binaries may need --strategy vm)"
                    );
                    if let Some(path) = emulator_path {
                        Box::new(emu_x86::BlinkMachoProvider::with_path(path.to_path_buf()))
                    } else {
                        Box::new(emu_x86::BlinkMachoProvider::new())
                    }
                }
                (BinaryFormat::Pe, OsTarget::Windows) => {
                    log::info!(
                        "Windows x86_64 EMU: blink + PE loader shim \
                         (MSVC CRT console only; complex binaries may need --strategy vm)"
                    );
                    if let Some(path) = emulator_path {
                        Box::new(emu_x86::BlinkPeProvider::with_path(path.to_path_buf()))
                    } else {
                        Box::new(emu_x86::BlinkPeProvider::new())
                    }
                }
                _ => {
                    bail!(
                        "unsupported format/OS combination for x86_64 EMU: {} {}",
                        detected.format,
                        detected.os_target,
                    )
                }
            }
        }
        CpuArch::AArch64 => {
            match (detected.format, detected.os_target) {
                (BinaryFormat::Elf, OsTarget::Linux) => {
                    log::info!("ARM64 EMU: QEMU TCI user-mode emulation");
                    if let Some(path) = emulator_path {
                        Box::new(emu_arm64::QemuTciProvider::with_path(path.to_path_buf()))
                    } else {
                        Box::new(emu_arm64::QemuTciProvider::new())
                    }
                }
                _ => {
                    bail!(
                        "ARM64 EMU only supports Linux ELF binaries. \
                         Got {} {}. Use --strategy vm for other ARM64 binaries.",
                        detected.format,
                        detected.os_target,
                    )
                }
            }
        }
    };

    let config = EmulationConfig {
        input,
        output,
        emulator_path,
    };

    run_emulation(provider.as_ref(), &config)
}

/// Run the VM strategy: full Linux VM in WASM via c2w.
///
/// Builds a container image containing the target binary and any required
/// compatibility layer (Wine for Windows binaries, Darling for macOS), then
/// converts it to a WASM module using c2w's pipeline.
///
/// This is the universal fallback — slower than AOT or EMU but works for
/// any binary.
pub fn run_vm(
    input: &Path,
    detected: &DetectedBinary,
    output: &Path,
) -> Result<()> {
    use std::fs;
    use std::io::Write;

    // Verify Docker is available (required for VM strategy).
    which::which("docker").context(
        "Docker is required for the VM strategy.\n\
         Install Docker or use a different strategy:\n  \
         --strategy aot  (for static binaries)\n  \
         --strategy emu  (for dynamic x86_64/ARM64 Linux binaries)"
    )?;

    let tmpdir = tempfile::tempdir().context("failed to create temp dir")?;

    // Determine the c2w target architecture.
    let c2w_arch = match detected.cpu_arch {
        CpuArch::X86_64 => "amd64",
        CpuArch::AArch64 => "aarch64",
    };

    // Copy the input binary to the build context.
    let binary_name = input
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("binary");
    let ctx_binary = tmpdir.path().join(binary_name);
    fs::copy(input, &ctx_binary)
        .with_context(|| format!("failed to copy {} to build context", input.display()))?;

    // Generate a Dockerfile for the VM container.
    let dockerfile_content = generate_vm_dockerfile(detected, binary_name);
    let dockerfile_path = tmpdir.path().join("Dockerfile");
    let mut f = fs::File::create(&dockerfile_path)?;
    f.write_all(dockerfile_content.as_bytes())?;

    log::info!(
        "VM strategy: building container for {} {} {} binary",
        detected.format,
        detected.cpu_arch,
        detected.os_target,
    );

    // Build the container image.
    let image_tag = format!("native2wasm-vm-{}", std::process::id());
    let build_status = std::process::Command::new("docker")
        .arg("build")
        .arg("-t")
        .arg(&image_tag)
        .arg("-f")
        .arg(&dockerfile_path)
        .arg(tmpdir.path())
        .status()
        .context("failed to run docker build")?;

    if !build_status.success() {
        bail!("docker build failed for VM container");
    }

    log::info!("VM container built: {image_tag}");

    // Run c2w to convert the container to WASM.
    let c2w_cli = c2w::cli::Cli {
        assets: None,
        dockerfile: None,
        builder: "docker".to_string(),
        target_arch: c2w_arch.to_string(),
        build_arg: vec![],
        to_js: false,
        debug_image: false,
        show_dockerfile: false,
        legacy: false,
        external_bundle: false,
        extra_flag: vec![],
        target_stage: None,
        pack: None,
        wizer: false,
        wizer_only: false,
        wizer_map_dir: vec![],
        wizer_init_func: "wizer.initialize".to_string(),
        no_docker: false,
        template: None,
        template_version: None,
        cache_dir: None,
        registry_username: None,
        registry_password: None,
        image_name: Some(image_tag.clone()),
        output: Some(output.to_string_lossy().to_string()),
    };

    let result = c2w::run(c2w_cli);

    // Clean up the Docker image.
    let _ = std::process::Command::new("docker")
        .args(["rmi", &image_tag])
        .output();

    result.with_context(|| {
        format!(
            "c2w pipeline failed for {} {} binary",
            detected.format, detected.os_target,
        )
    })
}

/// Generate a Dockerfile for the VM container based on the detected binary.
fn generate_vm_dockerfile(detected: &DetectedBinary, binary_name: &str) -> String {
    match detected.os_target {
        OsTarget::Linux => {
            // Linux binaries run directly in the VM.
            format!(
                "FROM ubuntu:22.04\n\
                 COPY {binary_name} /usr/local/bin/{binary_name}\n\
                 RUN chmod +x /usr/local/bin/{binary_name}\n\
                 ENTRYPOINT [\"/usr/local/bin/{binary_name}\"]\n"
            )
        }
        OsTarget::Windows => {
            // Windows binaries run under Wine in the VM.
            log::info!(
                "Windows VM: will install Wine for Win32/Win64 compatibility. \
                 This may take a while on first build."
            );
            format!(
                "FROM ubuntu:22.04\n\
                 ENV DEBIAN_FRONTEND=noninteractive\n\
                 RUN dpkg --add-architecture i386 && \\\n\
                     apt-get update && \\\n\
                     apt-get install -y --no-install-recommends \\\n\
                       wine64 wine32 && \\\n\
                     apt-get clean && rm -rf /var/lib/apt/lists/*\n\
                 COPY {binary_name} /app/{binary_name}\n\
                 ENTRYPOINT [\"wine64\", \"/app/{binary_name}\"]\n"
            )
        }
        OsTarget::Darwin => {
            // macOS binaries run under Darling in the VM.
            // Darling is experimental and requires a custom kernel module,
            // so we provide a best-effort container setup.
            log::info!(
                "macOS VM: Darling-based macOS emulation is experimental. \
                 Consider using --strategy emu for simple x86_64 macOS binaries."
            );
            format!(
                "FROM ubuntu:22.04\n\
                 ENV DEBIAN_FRONTEND=noninteractive\n\
                 # Darling requires building from source.\n\
                 # This is a placeholder — full Darling support requires\n\
                 # a custom base image with Darling pre-installed.\n\
                 RUN apt-get update && \\\n\
                     apt-get install -y --no-install-recommends \\\n\
                       curl ca-certificates && \\\n\
                     apt-get clean && rm -rf /var/lib/apt/lists/*\n\
                 COPY {binary_name} /app/{binary_name}\n\
                 # To use Darling, replace this with:\n\
                 # ENTRYPOINT [\"darling\", \"shell\", \"/app/{binary_name}\"]\n\
                 ENTRYPOINT [\"/bin/sh\", \"-c\", \
                   \"echo 'macOS binary requires Darling. Install from https://darlinghq.org' && exit 1\"]\n"
            )
        }
    }
}
