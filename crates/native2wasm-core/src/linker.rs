use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::CoreError;

/// Optimize lifted bitcode for wasm32 lowering.
///
/// Remill's lifted bitcode targets x86_64, so we need to:
///   1. Disassemble `.bc` to `.ll` text
///   2. Retarget the module from x86_64 to wasm32-wasi (triple + data layout)
///   3. Replace bodies of functions that use unsupported types (x86_fp80) or
///      x86 inline asm with a stub that returns the first pointer argument
///      (matching Remill's `Memory*` return convention)
///   4. Reassemble back to `.bc`
pub fn optimize_bc(input: &Path, output: &Path, entry_point: u64) -> Result<(), CoreError> {
    let llvm_dis = find_tool("llvm-dis")?;
    let llvm_as = find_tool("llvm-as")?;

    // 1. Disassemble .bc -> .ll
    let ll_path = input.with_extension("ll");
    let mut cmd = Command::new(&llvm_dis);
    cmd.env_remove("DYLD_LIBRARY_PATH");
    cmd.arg(input).arg("-o").arg(&ll_path);
    log::debug!("llvm-dis: {:?}", cmd);
    let result = cmd.output().map_err(CoreError::Io)?;
    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        return Err(CoreError::OptFailed(format!("llvm-dis failed: {stderr}")));
    }

    // 2. Read and patch the .ll text
    let ll_text = std::fs::read_to_string(&ll_path)?;
    let retargeted = retarget_to_wasm32(&ll_text);
    let stubbed = stub_unsupported_float80_functions(&retargeted);
    let patched = inject_start_wrapper(&stubbed, entry_point);
    std::fs::write(&ll_path, patched)?;

    // 3. Reassemble .ll -> .bc
    let mut cmd = Command::new(&llvm_as);
    cmd.env_remove("DYLD_LIBRARY_PATH");
    cmd.arg(&ll_path).arg("-o").arg(output);
    log::debug!("llvm-as: {:?}", cmd);
    let result = cmd.output().map_err(CoreError::Io)?;
    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        return Err(CoreError::OptFailed(format!("llvm-as failed: {stderr}")));
    }

    // Clean up intermediate .ll
    let _ = std::fs::remove_file(&ll_path);

    Ok(())
}

/// Wasm32 data layout string for LLVM.
///
/// `p:32:32` makes pointers 32-bit (wasm32's linear memory address space).
/// `p10:8:8` and `p20:8:8` are wasm-specific address spaces (externref, funcref).
/// `ni:1:10:20` marks those as non-integral (can't be ptrtoint'd).
const WASM32_DATALAYOUT: &str =
    "e-m:e-p:32:32-p10:8:8-p20:8:8-i64:64-n32:64-S128-ni:1:10:20";
const WASM32_TRIPLE: &str = "wasm32-unknown-wasi";

/// Retarget the LLVM IR module from x86_64 to wasm32-wasi.
///
/// This patches the `target datalayout` and `target triple` lines at the top
/// of the `.ll` file. The key change is pointers shrink from 64-bit to 32-bit,
/// which LLVM handles via the data layout string. Opaque `ptr` types adapt
/// automatically; `i64` values used as addresses will be truncated at `inttoptr`.
fn retarget_to_wasm32(ll_text: &str) -> String {
    let mut result = String::with_capacity(ll_text.len());

    for line in ll_text.lines() {
        if line.starts_with("target datalayout = ") {
            log::info!("Retargeting data layout: {} -> wasm32", line);
            result.push_str(&format!("target datalayout = \"{WASM32_DATALAYOUT}\""));
        } else if line.starts_with("target triple = ") {
            log::info!("Retargeting triple: {} -> {WASM32_TRIPLE}", line);
            result.push_str(&format!("target triple = \"{WASM32_TRIPLE}\""));
        } else {
            result.push_str(line);
        }
        result.push('\n');
    }

    result
}

/// Inject a WASI `_start` entry point that calls the lifted trace function
/// for the binary's original entry address.
///
/// Remill lifts code into trace functions with the signature:
///   `ptr @sub_HEXADDR(ptr %state, i64 %pc, ptr %memory)`
///
/// We generate a `_start` function that:
///   1. Allocates a zero-initialized State struct (3504 bytes for x86_64)
///   2. Calls the entry trace with (state, entry_pc, null)
///   3. Returns void (WASI _start convention)
fn inject_start_wrapper(ll_text: &str, entry_point: u64) -> String {
    // Find the entry trace function name. Remill names them `sub_HEXADDR`
    // but may append `.N` suffixes for deduplication.
    let entry_name_prefix = format!("@sub_{:x}", entry_point);
    let mut entry_func_name = None;

    for line in ll_text.lines() {
        if line.starts_with("define ") {
            if let Some(at_pos) = line.find('@') {
                if let Some(paren_pos) = line[at_pos..].find('(') {
                    let name = &line[at_pos..at_pos + paren_pos];
                    // Match exact or with .N suffix
                    if name == entry_name_prefix
                        || (name.starts_with(&entry_name_prefix)
                            && name[entry_name_prefix.len()..].starts_with('.'))
                    {
                        entry_func_name = Some(name.to_string());
                        break;
                    }
                }
            }
        }
    }

    let entry_func = match entry_func_name {
        Some(name) => name,
        None => {
            log::warn!(
                "Entry trace function {} not found in lifted IR, _start will not be generated",
                entry_name_prefix
            );
            return ll_text.to_string();
        }
    };

    log::info!("Injecting _start wrapper calling {}", entry_func);

    // Remill's x86_64 State struct is 3504 bytes.
    // _start allocates it on the stack, zeroes it, and calls the entry trace.
    let start_wrapper = format!(
        r#"
; _start: WASI entry point generated by native2wasm-core.
; Allocates a zero-initialized x86_64 State struct and calls the entry trace.
define void @_start() {{
  %state = alloca [3504 x i8], align 16
  call void @llvm.memset.p0.i32(ptr %state, i8 0, i32 3504, i1 false)
  %mem = call ptr {entry_func}(ptr %state, i64 {entry_pc}, ptr null)
  ret void
}}

; Ensure llvm.memset intrinsic is declared.
declare void @llvm.memset.p0.i32(ptr nocapture writeonly, i8, i32, i1 immarg)
"#,
        entry_func = entry_func,
        entry_pc = entry_point,
    );

    let mut result = ll_text.to_string();
    result.push_str(&start_wrapper);
    result
}

/// Replace bodies of functions that use `x86_fp80` with a minimal stub
/// (`ret ptr %0`). These are x87 FPU semantic functions from Remill that
/// operate on 80-bit extended precision floats, which WebAssembly cannot lower
/// (no 80-bit float support or software libcall available).
///
/// Strategy: two-pass approach.
/// Pass 1: Collect bodies of all function definitions and check for `x86_fp80`.
/// Pass 2: Re-emit the IR, replacing bodies of identified functions with stubs.
fn stub_unsupported_float80_functions(ll_text: &str) -> String {
    // Pass 1: identify functions whose bodies use features unsupported on wasm32:
    //   - x86_fp80 type (80-bit extended precision, no wasm equivalent)
    //   - x86 inline asm with register constraints ({ax}, {cx}, etc.)
    let mut unsupported_functions = std::collections::HashSet::new();
    let mut current_func: Option<String> = None;
    let mut in_body = false;

    for line in ll_text.lines() {
        if line.starts_with("define ") && line.ends_with('{') {
            // Extract the function name: @name(
            if let Some(at_pos) = line.find('@') {
                if let Some(paren_pos) = line[at_pos..].find('(') {
                    let name = &line[at_pos..at_pos + paren_pos];
                    current_func = Some(name.to_string());
                    in_body = true;
                    // Also check the signature line itself
                    if line.contains("x86_fp80") {
                        unsupported_functions.insert(name.to_string());
                    }
                    continue;
                }
            }
        }

        if in_body {
            if line.trim() == "}" {
                current_func = None;
                in_body = false;
            } else if let Some(ref name) = current_func {
                // Check for x86_fp80 type
                if line.contains("x86_fp80") {
                    unsupported_functions.insert(name.clone());
                }
                // Check for x86 inline asm with register constraints
                if line.contains("asm ")
                    && (line.contains("{ax}")
                        || line.contains("{bx}")
                        || line.contains("{cx}")
                        || line.contains("{dx}")
                        || line.contains("{si}")
                        || line.contains("{di}")
                        || line.contains("{sp}")
                        || line.contains("{bp}")
                        || line.contains("{flags}")
                        || line.contains("{dirflag}")
                        || line.contains("{fpsr}")
                        || line.contains("{fpcr}"))
                {
                    unsupported_functions.insert(name.clone());
                }
            }
        }
    }

    log::info!(
        "Found {} functions using x86-only features (x86_fp80, inline asm) unsupported on wasm32, stubbing them",
        unsupported_functions.len()
    );

    // Pass 2: re-emit, replacing identified functions with stubs.
    let mut result = String::with_capacity(ll_text.len());
    let mut lines = ll_text.lines().peekable();

    while let Some(line) = lines.next() {
        if line.starts_with("define ") && line.ends_with('{') {
            // Check if this function needs stubbing
            let needs_stub = if let Some(at_pos) = line.find('@') {
                if let Some(paren_pos) = line[at_pos..].find('(') {
                    let name = &line[at_pos..at_pos + paren_pos];
                    unsupported_functions.contains(name)
                } else {
                    false
                }
            } else {
                false
            };

            if needs_stub {
                log::debug!("Stubbing: {}", &line[..line.len().min(100)]);
                // Emit function header
                result.push_str(line);
                result.push('\n');
                // Emit stub body
                result.push_str("  ret ptr %0\n");
                result.push_str("}\n");
                // Skip original body
                let mut brace_depth: i32 = 1;
                while let Some(body_line) = lines.next() {
                    for ch in body_line.chars() {
                        if ch == '{' {
                            brace_depth += 1;
                        } else if ch == '}' {
                            brace_depth -= 1;
                        }
                    }
                    if brace_depth <= 0 {
                        break;
                    }
                }
            } else {
                result.push_str(line);
                result.push('\n');
            }
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }

    result
}

/// Compile LLVM bitcode to a wasm32 object file using `llc`.
pub fn compile_bc_to_obj(bc_path: &Path, obj_path: &Path, opt_level: u8) -> Result<(), CoreError> {
    let llc = find_tool("llc")?;

    let opt = format!("-O{}", opt_level.min(3));
    let mut cmd = Command::new(&llc);
    // Clear DYLD_LIBRARY_PATH so the tool uses its own LLVM version,
    // not the one we set for the main process (which may differ).
    cmd.env_remove("DYLD_LIBRARY_PATH");
    cmd.arg("-march=wasm32")
        .arg("-filetype=obj")
        .arg(&opt)
        .arg(bc_path)
        .arg("-o")
        .arg(obj_path);

    log::debug!("llc: {:?}", cmd);

    let output = cmd.output().map_err(CoreError::Io)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CoreError::LlcFailed(stderr.into_owned()));
    }

    Ok(())
}

/// Link lifted object + shim objects into a final WASI .wasm module using `wasm-ld`.
pub fn link_wasm(
    lifted_obj: &Path,
    shim_objs: &[PathBuf],
    output: &Path,
    wasi_sysroot: Option<&Path>,
) -> Result<(), CoreError> {
    let wasm_ld = find_tool("wasm-ld")?;

    let mut cmd = Command::new(&wasm_ld);
    cmd.env_remove("DYLD_LIBRARY_PATH");
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

    let output_result = cmd.output().map_err(CoreError::Io)?;
    if !output_result.status.success() {
        let stderr = String::from_utf8_lossy(&output_result.stderr);
        return Err(CoreError::LinkFailed(stderr.into_owned()));
    }

    Ok(())
}

pub fn find_tool(name: &str) -> Result<PathBuf, CoreError> {
    which::which(name).map_err(|_| CoreError::ToolNotFound {
        tool: name.to_string(),
    })
}
