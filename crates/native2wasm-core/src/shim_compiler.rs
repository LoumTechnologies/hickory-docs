use std::path::Path;
use std::process::Command;

use crate::error::CoreError;

/// Shared shim source files (wasi_helpers.h, shim_process.c) that are
/// common across all OS targets.
static SHARED_HEADERS: &[(&str, &[u8])] = &[
    ("wasi_helpers.h", include_bytes!("../shims/wasi_helpers.h")),
];

static SHARED_SOURCES: &[(&str, &[u8])] = &[(
    "shim_process.c",
    include_bytes!("../shims/shim_process.c"),
)];

/// Compile all shim C sources to wasm32-wasi object files in the given output directory.
///
/// `os_headers` and `os_sources` are OS-specific shim files provided by each crate.
/// Returns the paths to the compiled .o files.
pub fn compile_shims(
    out_dir: &Path,
    wasi_sysroot: Option<&Path>,
    os_headers: &[(&str, &[u8])],
    os_sources: &[(&str, &[u8])],
) -> Result<Vec<std::path::PathBuf>, CoreError> {
    let clang = find_clang()?;

    // Write all headers first so #include directives resolve during compilation.
    for (name, content) in SHARED_HEADERS.iter().chain(os_headers.iter()) {
        std::fs::write(out_dir.join(name), content)?;
    }

    let mut objects = Vec::new();

    // Compile shared sources + OS-specific sources.
    let all_sources = SHARED_SOURCES.iter().chain(os_sources.iter());

    for (name, source) in all_sources {
        let src_path = out_dir.join(name);
        std::fs::write(&src_path, source)?;

        let obj_name = name.replace(".c", ".o");
        let obj_path = out_dir.join(&obj_name);

        let mut cmd = Command::new(&clang);
        cmd.arg("--target=wasm32-wasi")
            .arg("-c")
            .arg("-O2")
            .arg("-Wall")
            .arg("-ffreestanding")
            .arg("-I")
            .arg(out_dir);

        if let Some(sysroot) = wasi_sysroot {
            cmd.arg(format!("--sysroot={}", sysroot.display()));
        }

        cmd.arg("-o").arg(&obj_path).arg(&src_path);

        log::debug!("compiling shim: {:?}", cmd);

        let output = cmd.output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CoreError::ShimCompilationFailed(format!(
                "{name}: {stderr}"
            )));
        }

        objects.push(obj_path);
    }

    Ok(objects)
}

pub fn find_clang() -> Result<std::path::PathBuf, CoreError> {
    // Try clang variants that might support wasm32.
    for name in &["clang", "clang-17", "clang-18", "clang-19"] {
        if let Ok(path) = which::which(name) {
            return Ok(path);
        }
    }
    Err(CoreError::ToolNotFound {
        tool: "clang (with wasm32-wasi support)".into(),
    })
}
