use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use wasmtime::{Engine, Module};

/// Manages resolution and caching of WASM command modules.
pub struct Toolchain {
    base_dir: PathBuf,
    commands: HashMap<String, PathBuf>,
    engine: Engine,
    module_cache: HashMap<PathBuf, Module>,
}

impl Toolchain {
    /// Create a new toolchain rooted at `base_dir`.
    pub fn new(base_dir: &Path, engine: Engine) -> Self {
        Self {
            base_dir: base_dir.to_path_buf(),
            commands: HashMap::new(),
            engine,
            module_cache: HashMap::new(),
        }
    }

    /// Scan the base directory for `*.wasm` files and populate the command map.
    pub fn scan(&mut self) -> Result<()> {
        self.commands.clear();

        if !self.base_dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(&self.base_dir)
            .with_context(|| format!("scanning toolchain dir: {}", self.base_dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("wasm") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    self.commands.insert(stem.to_string(), path);
                }
            }
        }

        log::debug!(
            "Toolchain scanned {}: {} commands found",
            self.base_dir.display(),
            self.commands.len()
        );
        Ok(())
    }

    /// Resolve a command name to its `.wasm` file path.
    pub fn resolve(&self, name: &str) -> Option<&Path> {
        self.commands.get(name).map(|p| p.as_path())
    }

    /// Load (compile or retrieve from cache) a WASM module by command name.
    pub fn load_module(&mut self, name: &str) -> Result<Module> {
        let path = self
            .commands
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("command not in toolchain: {name}"))?
            .clone();

        if let Some(module) = self.module_cache.get(&path) {
            return Ok(module.clone());
        }

        log::debug!("Compiling WASM module: {}", path.display());
        let module = Module::from_file(&self.engine, &path)
            .with_context(|| format!("compiling WASM module: {}", path.display()))?;

        self.module_cache.insert(path, module.clone());
        Ok(module)
    }

    /// Get a reference to the wasmtime engine.
    pub fn engine(&self) -> &Engine {
        &self.engine
    }
}
