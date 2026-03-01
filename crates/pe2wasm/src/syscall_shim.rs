use std::path::Path;
use std::process::Command;

use crate::error::Error;

/// C source files for NT syscall → WASI shims.
static SHIM_SOURCES: &[(&str, &[u8])] = &[
    (
        "syscall_dispatch.c",
        include_bytes!("../shims/syscall_dispatch.c"),
    ),
    ("shim_io.c", include_bytes!("../shims/shim_io.c")),
    ("shim_process.c", include_bytes!("../shims/shim_process.c")),
    ("shim_memory.c", include_bytes!("../shims/shim_memory.c")),
    ("shim_misc.c", include_bytes!("../shims/shim_misc.c")),
];

/// Compile all shim C sources to wasm32-wasi object files in the given output directory.
/// Returns the paths to the compiled .o files.
pub fn compile_shims(
    out_dir: &Path,
    wasi_sysroot: Option<&Path>,
) -> Result<Vec<std::path::PathBuf>, Error> {
    let clang = find_clang()?;

    let mut objects = Vec::new();

    for (name, source) in SHIM_SOURCES {
        let src_path = out_dir.join(name);
        std::fs::write(&src_path, source)?;

        let obj_name = name.replace(".c", ".o");
        let obj_path = out_dir.join(&obj_name);

        let mut cmd = Command::new(&clang);
        cmd.arg("--target=wasm32-wasi")
            .arg("-c")
            .arg("-O2")
            .arg("-Wall")
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
            return Err(Error::ShimCompilationFailed(format!("{name}: {stderr}")));
        }

        objects.push(obj_path);
    }

    // Also write out the header files so shims can #include them.
    std::fs::write(
        out_dir.join("nt_syscall.h"),
        include_bytes!("../shims/nt_syscall.h"),
    )?;
    std::fs::write(
        out_dir.join("wasi_helpers.h"),
        include_bytes!("../shims/wasi_helpers.h"),
    )?;

    Ok(objects)
}

fn find_clang() -> Result<std::path::PathBuf, Error> {
    // Try clang variants that might support wasm32.
    for name in &["clang", "clang-17", "clang-18", "clang-19"] {
        if let Ok(path) = which::which(name) {
            return Ok(path);
        }
    }
    Err(Error::ToolNotFound {
        tool: "clang (with wasm32-wasi support)".into(),
    })
}
