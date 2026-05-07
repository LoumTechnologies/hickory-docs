//! Lightweight shell interpreter for `<hick:script>` elements.
//!
//! Parses shell scripts using `brush-parser` and dispatches commands to
//! individual WASI `.wasm` modules via `wasmtime`, replacing the heavyweight
//! container-based execution model of `hick:exec`.

pub mod builtins;
pub mod env;
pub mod error;
pub mod expand;
pub mod interpreter;
mod pipe;
mod redirect;
pub mod toolchain;
pub mod wasi_dispatch;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::env::ShellEnvironment;
use crate::error::ShellError;
use crate::interpreter::ShellInterpreter;
use crate::toolchain::Toolchain;

/// Captured output from a shell script execution.
#[derive(Debug, Clone)]
pub struct ScriptOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

/// Run a shell script to completion.
///
/// Parses `source` as a POSIX/bash script, then walks the AST dispatching
/// builtin commands to Rust implementations and external commands to WASI
/// modules found in `toolchain_dir`.
pub fn run_script(
    source: &str,
    toolchain_dir: &Path,
    preopened_dirs: &[(PathBuf, String)],
    env_vars: &HashMap<String, String>,
) -> Result<ScriptOutput, ShellError> {
    // Parse the script
    let tokens = brush_parser::tokenize_str(source)
        .map_err(|e| ShellError::Parse(format!("{e}")))?;

    let parser_opts = brush_parser::ParserOptions::default();
    let source_info = brush_parser::SourceInfo {
        source: String::from("<hick:script>"),
    };
    let program = brush_parser::parse_tokens(&tokens, &parser_opts, &source_info)
        .map_err(|e| ShellError::Parse(format!("{e}")))?;

    // Set up the environment
    let env = ShellEnvironment::new(
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
        env_vars,
    );

    // Set up the toolchain
    let engine = wasmtime::Engine::default();
    let mut toolchain = Toolchain::new(toolchain_dir, engine);
    toolchain.scan().map_err(ShellError::WasiExec)?;

    // Create and run the interpreter
    let mut interp = ShellInterpreter::new(env, toolchain, preopened_dirs.to_vec());
    interp.eval_program(&program)?;

    let (stdout, stderr, exit_code) = interp.into_output();
    Ok(ScriptOutput {
        stdout,
        stderr,
        exit_code,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn run(source: &str) -> ScriptOutput {
        run_script(
            source,
            &PathBuf::from("/nonexistent"),
            &[],
            &HashMap::new(),
        )
        .expect("script should succeed")
    }

    fn run_err(source: &str) -> ShellError {
        run_script(
            source,
            &PathBuf::from("/nonexistent"),
            &[],
            &HashMap::new(),
        )
        .expect_err("script should fail")
    }

    #[test]
    fn echo_hello() {
        let out = run("echo hello");
        assert_eq!(out.stdout, "hello\n");
        assert_eq!(out.exit_code, 0);
    }

    #[test]
    fn echo_multiple_args() {
        let out = run("echo hello world");
        assert_eq!(out.stdout, "hello world\n");
    }

    #[test]
    fn variable_assignment_and_echo() {
        let out = run("X=hello; echo $X");
        assert_eq!(out.stdout, "hello\n");
    }

    #[test]
    fn multiple_commands() {
        let out = run("echo first; echo second");
        assert_eq!(out.stdout, "first\nsecond\n");
    }

    #[test]
    fn exit_code_true() {
        let out = run("true");
        assert_eq!(out.exit_code, 0);
    }

    #[test]
    fn exit_code_false() {
        let out = run("false");
        assert_eq!(out.exit_code, 1);
    }

    #[test]
    fn and_list_success() {
        let out = run("true && echo yes");
        assert_eq!(out.stdout, "yes\n");
    }

    #[test]
    fn and_list_failure() {
        let out = run("false && echo yes");
        assert_eq!(out.stdout, "");
        assert_eq!(out.exit_code, 1);
    }

    #[test]
    fn or_list_success() {
        let out = run("true || echo no");
        assert_eq!(out.stdout, "");
        assert_eq!(out.exit_code, 0);
    }

    #[test]
    fn or_list_failure() {
        let out = run("false || echo fallback");
        assert_eq!(out.stdout, "fallback\n");
    }

    #[test]
    fn command_not_found() {
        let err = run_err("nonexistent-cmd");
        assert!(matches!(err, ShellError::CommandNotFound(_)));
    }

    #[test]
    fn env_vars_passed_in() {
        let mut vars = HashMap::new();
        vars.insert("GREETING".to_string(), "hi".to_string());
        let out = run_script(
            "echo $GREETING",
            &PathBuf::from("/nonexistent"),
            &[],
            &vars,
        )
        .unwrap();
        assert_eq!(out.stdout, "hi\n");
    }

    #[test]
    fn exit_code_variable() {
        let out = run("false; echo $?");
        assert_eq!(out.stdout, "1\n");
    }

    #[test]
    fn export_builtin() {
        let out = run("export FOO=bar; echo $FOO");
        assert_eq!(out.stdout, "bar\n");
    }

    #[test]
    fn empty_script() {
        let out = run("");
        assert_eq!(out.stdout, "");
        assert_eq!(out.exit_code, 0);
    }
}
