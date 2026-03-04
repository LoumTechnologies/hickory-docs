use std::path::Path;

use anyhow::{Context, Result};
use wasmtime::{Engine, Linker, Module, Store};
use wasi_common::WasiCtx;

use crate::env::ShellEnvironment;

/// Result of running a single WASM command.
pub struct WasiResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

/// Run a WASI module to completion with the given arguments and environment.
///
/// This is a blocking call on the current thread that executes the module's
/// `_start` function. Stdout and stderr are captured in memory.
pub fn run_wasi_module(
    engine: &Engine,
    module: &Module,
    args: &[String],
    env: &ShellEnvironment,
    preopened_dirs: &[(impl AsRef<Path>, impl AsRef<str>)],
    stdin_data: &[u8],
) -> Result<WasiResult> {
    let mut builder = wasi_common::sync::WasiCtxBuilder::new();

    // Set argv
    for arg in args {
        builder.arg(arg)?;
    }

    // Set env from exported shell variables
    for (key, value) in env.exported_env() {
        builder.env(&key, &value)?;
    }

    // Capture stdout and stderr in memory pipes
    let stdout_buf = wasi_common::pipe::WritePipe::new_in_memory();
    let stderr_buf = wasi_common::pipe::WritePipe::new_in_memory();

    builder.stdout(Box::new(stdout_buf.clone()));
    builder.stderr(Box::new(stderr_buf.clone()));

    // Provide stdin data
    let stdin_pipe = wasi_common::pipe::ReadPipe::from(stdin_data.to_vec());
    builder.stdin(Box::new(stdin_pipe));

    // Add preopened directories
    for (host_path, guest_path) in preopened_dirs {
        let dir = cap_std::fs::Dir::open_ambient_dir(
            host_path.as_ref(),
            cap_std::ambient_authority(),
        )
        .with_context(|| {
            format!(
                "opening preopened dir: {}",
                host_path.as_ref().display()
            )
        })?;
        builder.preopened_dir(dir, guest_path.as_ref())?;
    }

    let wasi = builder.build();
    let mut store = Store::new(engine, wasi);

    let mut linker = Linker::<WasiCtx>::new(engine);
    wasi_common::sync::add_to_linker(&mut linker, |cx| cx)?;

    linker.module(&mut store, "", module)?;

    let func = linker
        .get_default(&mut store, "")
        .context("no default export in WASI module")?;
    let typed = func.typed::<(), ()>(&store)?;

    let exit_code = match typed.call(&mut store, ()) {
        Ok(()) => 0,
        Err(e) => {
            // Check if this is a proc_exit trap (normal WASI exit)
            if let Some(exit) = e.downcast_ref::<wasi_common::I32Exit>() {
                exit.0
            } else {
                return Err(e.context("WASI module execution failed"));
            }
        }
    };

    // Extract captured output
    drop(store); // drop store to release references to the pipes

    let stdout_bytes = stdout_buf
        .try_into_inner()
        .map_err(|_| anyhow::anyhow!("failed to extract stdout"))?
        .into_inner();
    let stderr_bytes = stderr_buf
        .try_into_inner()
        .map_err(|_| anyhow::anyhow!("failed to extract stderr"))?
        .into_inner();

    Ok(WasiResult {
        stdout: String::from_utf8_lossy(&stdout_bytes).into_owned(),
        stderr: String::from_utf8_lossy(&stderr_bytes).into_owned(),
        exit_code,
    })
}
