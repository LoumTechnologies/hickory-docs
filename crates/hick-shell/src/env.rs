use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Shell environment state: variables, exports, cwd, exit codes.
pub struct ShellEnvironment {
    pub variables: HashMap<String, String>,
    pub exports: HashSet<String>,
    pub cwd: PathBuf,
    pub last_exit_code: i32,
    pub positional_params: Vec<String>,
}

impl ShellEnvironment {
    pub fn new(cwd: PathBuf, env_vars: &HashMap<String, String>) -> Self {
        let mut variables = env_vars.clone();
        let mut exports = HashSet::new();
        for key in env_vars.keys() {
            exports.insert(key.clone());
        }
        // Ensure PATH is always set
        variables.entry("PATH".to_string()).or_insert_with(|| "/usr/bin:/bin".to_string());

        Self {
            variables,
            exports,
            cwd,
            last_exit_code: 0,
            positional_params: Vec::new(),
        }
    }

    /// Get a variable value.
    pub fn get_var(&self, name: &str) -> Option<&str> {
        match name {
            "?" => None, // handled specially by caller
            _ => self.variables.get(name).map(|s| s.as_str()),
        }
    }

    /// Set a variable.
    pub fn set_var(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.variables.insert(name.into(), value.into());
    }

    /// Get all exported variables as key=value pairs.
    pub fn exported_env(&self) -> Vec<(String, String)> {
        self.exports
            .iter()
            .filter_map(|name| {
                self.variables
                    .get(name)
                    .map(|val| (name.clone(), val.clone()))
            })
            .collect()
    }
}
