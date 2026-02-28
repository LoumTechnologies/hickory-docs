use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::Error;

/// Compile LLVM bitcode to a wasm32 object file using `llc`.
pub fn compile_bc_to_obj(bc_path: &Path, obj_path: &Path, opt_level: u8) -> Result<(), Error> {
    let llc = find_tool("llc")?;

    let opt = format!("-O{}", opt_level.min(3));
    let mut cmd = Command::new(&llc);
    cmd.arg("-march=wasm32")
        .arg("-filetype=obj")
        .arg(&opt)
        .arg(bc_path)
        .arg("-o")
        .arg(obj_path);

    log::debug!("llc: {:?}", cmd);

    let output = cmd.output().map_err(Error::Io)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::LlcFailed(stderr.into_owned()));
    }

    Ok(())
}

/// Link lifted object + shim objects into a final WASI .wasm module using `wasm-ld`.
pub fn link_wasm(
    lifted_obj: &Path,
    shim_objs: &[PathBuf],
    output: &Path,
    wasi_sysroot: Option<&Path>,
) -> Result<(), Error> {
    let wasm_ld = find_tool("wasm-ld")?;

    let mut cmd = Command::new(&wasm_ld);
    cmd.arg("--export=_start").arg("--import-memory");

    // Add the lifted code object.
    cmd.arg(lifted_obj);

    // Add all shim objects.
    for obj in shim_objs {
        cmd.arg(obj);
    }

    // Link against wasi-sysroot libc if available.
    if let Some(sysroot) = wasi_sysroot {
        let lib_dir = sysroot.join("lib").join("wasm32-wasi");
        if lib_dir.exists() {
            cmd.arg(format!("-L{}", lib_dir.display()));
            cmd.arg("-lc");
        }
    }

    cmd.arg("-o").arg(output);

    // Allow undefined symbols for WASI imports.
    cmd.arg("--allow-undefined");

    log::debug!("wasm-ld: {:?}", cmd);

    let output_result = cmd.output().map_err(Error::Io)?;
    if !output_result.status.success() {
        let stderr = String::from_utf8_lossy(&output_result.stderr);
        return Err(Error::LinkFailed(stderr.into_owned()));
    }

    Ok(())
}

fn find_tool(name: &str) -> Result<PathBuf, Error> {
    which::which(name).map_err(|_| Error::ToolNotFound {
        tool: name.to_string(),
    })
}
