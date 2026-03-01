use std::path::PathBuf;

use brush_parser::ast;

use crate::builtins;
use crate::env::ShellEnvironment;
use crate::error::ShellError;
use crate::expand::expand_word;
use crate::toolchain::Toolchain;
use crate::wasi_dispatch;

/// Shell interpreter state.
pub struct ShellInterpreter {
    pub env: ShellEnvironment,
    pub toolchain: Toolchain,
    pub preopened_dirs: Vec<(PathBuf, String)>,
    stdout_buf: String,
    stderr_buf: String,
}

impl ShellInterpreter {
    pub fn new(
        env: ShellEnvironment,
        toolchain: Toolchain,
        preopened_dirs: Vec<(PathBuf, String)>,
    ) -> Self {
        Self {
            env,
            toolchain,
            preopened_dirs,
            stdout_buf: String::new(),
            stderr_buf: String::new(),
        }
    }

    /// Consume the interpreter and return captured output.
    pub fn into_output(self) -> (String, String, i32) {
        (self.stdout_buf, self.stderr_buf, self.env.last_exit_code)
    }

    /// Evaluate a parsed program (top-level AST node).
    pub fn eval_program(&mut self, program: &ast::Program) -> Result<(), ShellError> {
        for complete_command in &program.complete_commands {
            self.eval_complete_command(complete_command)?;
        }
        Ok(())
    }

    fn eval_complete_command(
        &mut self,
        cc: &ast::CompleteCommand,
    ) -> Result<(), ShellError> {
        // CompleteCommand is CompoundList(Vec<CompoundListItem>)
        self.eval_compound_list(cc)
    }

    fn eval_compound_list(
        &mut self,
        list: &ast::CompoundList,
    ) -> Result<(), ShellError> {
        for item in &list.0 {
            self.eval_compound_list_item(item)?;
        }
        Ok(())
    }

    fn eval_compound_list_item(
        &mut self,
        item: &ast::CompoundListItem,
    ) -> Result<(), ShellError> {
        let ast::CompoundListItem(and_or_list, _separator) = item;
        self.eval_and_or_list(and_or_list)
    }

    fn eval_and_or_list(&mut self, list: &ast::AndOrList) -> Result<(), ShellError> {
        self.eval_pipeline(&list.first)?;

        for and_or in &list.additional {
            match and_or {
                ast::AndOr::And(pipeline) => {
                    if self.env.last_exit_code == 0 {
                        self.eval_pipeline(pipeline)?;
                    }
                }
                ast::AndOr::Or(pipeline) => {
                    if self.env.last_exit_code != 0 {
                        self.eval_pipeline(pipeline)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn eval_pipeline(&mut self, pipeline: &ast::Pipeline) -> Result<(), ShellError> {
        if pipeline.seq.len() == 1 {
            self.eval_command(&pipeline.seq[0])?;
        } else {
            // Simplified sequential pipeline for Phase 0
            let mut pipe_data = Vec::new();
            for (i, cmd) in pipeline.seq.iter().enumerate() {
                let is_last = i == pipeline.seq.len() - 1;
                let result = self.eval_command_with_stdin(cmd, &pipe_data, !is_last)?;
                pipe_data = result.into_bytes();
            }
        }

        if pipeline.bang {
            self.env.last_exit_code = if self.env.last_exit_code == 0 { 1 } else { 0 };
        }

        Ok(())
    }

    fn eval_command(&mut self, command: &ast::Command) -> Result<(), ShellError> {
        match command {
            ast::Command::Simple(simple) => self.eval_simple_command(simple),
            ast::Command::Compound(compound, _redirects) => {
                self.eval_compound_command(compound)
            }
            ast::Command::Function(func_def) => {
                log::debug!(
                    "Function definition '{}' (not yet implemented)",
                    func_def.fname.value
                );
                Ok(())
            }
            ast::Command::ExtendedTest(_) => {
                log::debug!("Extended test not yet implemented");
                self.env.last_exit_code = 0;
                Ok(())
            }
        }
    }

    fn eval_command_with_stdin(
        &mut self,
        command: &ast::Command,
        stdin_data: &[u8],
        capture_stdout: bool,
    ) -> Result<String, ShellError> {
        match command {
            ast::Command::Simple(simple) => {
                self.eval_simple_command_with_io(simple, stdin_data, capture_stdout)
            }
            _ => {
                self.eval_command(command)?;
                Ok(String::new())
            }
        }
    }

    fn eval_simple_command(&mut self, cmd: &ast::SimpleCommand) -> Result<(), ShellError> {
        self.eval_simple_command_with_io(cmd, &[], false)?;
        Ok(())
    }

    fn eval_simple_command_with_io(
        &mut self,
        cmd: &ast::SimpleCommand,
        stdin_data: &[u8],
        capture_stdout: bool,
    ) -> Result<String, ShellError> {
        // Process prefix assignments
        let mut assignments = Vec::new();
        if let Some(prefix) = &cmd.prefix {
            for item in &prefix.0 {
                if let ast::CommandPrefixOrSuffixItem::AssignmentWord(assignment, _word) = item {
                    let name = match &assignment.name {
                        ast::AssignmentName::VariableName(n) => n.clone(),
                        ast::AssignmentName::ArrayElementName(n, _) => n.clone(),
                    };
                    let value = match &assignment.value {
                        ast::AssignmentValue::Scalar(w) => expand_word(&w.value, &self.env),
                        ast::AssignmentValue::Array(_) => String::new(),
                    };
                    assignments.push((name, value));
                }
            }
        }

        // Build the command words (command name + arguments)
        let mut words = Vec::new();

        if let Some(word_or_name) = &cmd.word_or_name {
            words.push(expand_word(&word_or_name.value, &self.env));
        }

        // Track suffix assignments for builtins like export
        let mut suffix_assignments = Vec::new();
        if let Some(suffix) = &cmd.suffix {
            for item in &suffix.0 {
                match item {
                    ast::CommandPrefixOrSuffixItem::Word(w) => {
                        words.push(expand_word(&w.value, &self.env));
                    }
                    ast::CommandPrefixOrSuffixItem::AssignmentWord(assignment, _word) => {
                        let name = match &assignment.name {
                            ast::AssignmentName::VariableName(n) => n.clone(),
                            ast::AssignmentName::ArrayElementName(n, _) => n.clone(),
                        };
                        let value = match &assignment.value {
                            ast::AssignmentValue::Scalar(w) => {
                                expand_word(&w.value, &self.env)
                            }
                            ast::AssignmentValue::Array(_) => String::new(),
                        };
                        suffix_assignments.push((name, value));
                    }
                    _ => {}
                }
            }
        }

        // If no command word, just apply assignments to the environment
        if words.is_empty() {
            for (name, value) in assignments {
                self.env.set_var(name, value);
            }
            self.env.last_exit_code = 0;
            return Ok(String::new());
        }

        let cmd_name = &words[0];
        let args = &words[1..];

        if builtins::is_builtin(cmd_name) {
            // Handle suffix assignments for builtins like `export FOO=bar`
            if cmd_name == "export" && !suffix_assignments.is_empty() {
                for (name, value) in &suffix_assignments {
                    self.env.set_var(name.clone(), value.clone());
                    self.env.exports.insert(name.clone());
                }
            }
            let result = builtins::run_builtin(cmd_name, args, &mut self.env);
            self.env.last_exit_code = result.exit_code;
            if capture_stdout {
                return Ok(result.stdout);
            }
            self.stdout_buf.push_str(&result.stdout);
            return Ok(String::new());
        }

        // Try external WASM command
        if self.toolchain.resolve(cmd_name).is_some() {
            let module = self
                .toolchain
                .load_module(cmd_name)
                .map_err(ShellError::WasiExec)?;
            let result = wasi_dispatch::run_wasi_module(
                self.toolchain.engine(),
                &module,
                &words,
                &self.env,
                &self.preopened_dirs,
                stdin_data,
            )
            .map_err(ShellError::WasiExec)?;

            self.env.last_exit_code = result.exit_code;
            if capture_stdout {
                return Ok(result.stdout);
            }
            self.stdout_buf.push_str(&result.stdout);
            self.stderr_buf.push_str(&result.stderr);
            return Ok(String::new());
        }

        // Command not found
        self.env.last_exit_code = 127;
        Err(ShellError::CommandNotFound(cmd_name.clone()))
    }

    fn eval_compound_command(
        &mut self,
        compound: &ast::CompoundCommand,
    ) -> Result<(), ShellError> {
        match compound {
            ast::CompoundCommand::BraceGroup(bg) => {
                self.eval_compound_list(&bg.list)?;
            }
            ast::CompoundCommand::Subshell(sub) => {
                self.eval_compound_list(&sub.list)?;
            }
            _ => {
                log::debug!("Compound command not yet implemented in Phase 0");
            }
        }
        Ok(())
    }
}
