//! Agent memory bank — persistent notes in `.agents/memory/*.md`.
//!
//! Memory files are project-scoped and committed to the repo. The agent reads
//! them at startup (for system prompt context) and can write or append to them
//! during a session. All writes are atomic: the new content is written to a
//! `.tmp` file and renamed into place, preventing partial-write corruption.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Persistent agent memory bank backed by `.agents/memory/*.md` files.
pub struct MemoryBank {
    memory_dir: PathBuf,
    /// In-memory cache: name (without extension) → content.
    memories: HashMap<String, String>,
}

impl MemoryBank {
    /// Load all `*.md` files from `<project_dir>/.agents/memory/`.
    ///
    /// Returns an empty bank if the directory does not exist.
    pub fn load(project_dir: &Path) -> Self {
        let memory_dir = project_dir.join(".agents").join("memory");
        if !memory_dir.is_dir() {
            return Self {
                memory_dir,
                memories: HashMap::new(),
            };
        }

        let mut entries: Vec<_> = std::fs::read_dir(&memory_dir)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
            .collect();
        entries.sort_by_key(|e| e.file_name());

        let mut memories = HashMap::new();
        for entry in entries {
            let path = entry.path();
            let name = path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            if name.is_empty() {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(&path) {
                memories.insert(name, content);
            }
        }

        Self {
            memory_dir,
            memories,
        }
    }

    /// Get the content of a named memory file (name without `.md` extension).
    pub fn get(&self, name: &str) -> Option<&str> {
        self.memories.get(name).map(|s| s.as_str())
    }

    /// All memory names (without extension), sorted alphabetically.
    pub fn names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.memories.keys().map(|s| s.as_str()).collect();
        names.sort();
        names
    }

    /// Format all memories for inclusion in the agent system prompt.
    ///
    /// Produces sections `# Memory: <name>\n\n<content>` separated by
    /// `\n\n---\n\n`. Returns an empty string when the bank is empty.
    pub fn to_context_string(&self) -> String {
        if self.memories.is_empty() {
            return String::new();
        }
        let mut names = self.names();
        names.sort();
        let parts: Vec<String> = names
            .iter()
            .map(|name| {
                let content = self.memories[*name].as_str();
                format!("# Memory: {name}\n\n{content}")
            })
            .collect();
        parts.join("\n\n---\n\n")
    }

    /// Atomically write a new or replacement memory file.
    ///
    /// Writes to `<name>.md.tmp` then renames to `<name>.md` to prevent
    /// partial writes on crash.
    pub fn write(&mut self, name: &str, content: &str) -> anyhow::Result<()> {
        self.ensure_dir()?;
        let final_path = self.memory_dir.join(format!("{name}.md"));
        let tmp_path = self.memory_dir.join(format!("{name}.md.tmp"));
        std::fs::write(&tmp_path, content)?;
        std::fs::rename(&tmp_path, &final_path)?;
        self.memories.insert(name.to_string(), content.to_string());
        Ok(())
    }

    /// Append `additional` content to an existing memory file, or create it.
    pub fn append(&mut self, name: &str, additional: &str) -> anyhow::Result<()> {
        let existing = self.memories.get(name).map(|s| s.as_str()).unwrap_or("");
        let new_content = if existing.is_empty() {
            additional.to_string()
        } else if existing.ends_with('\n') {
            format!("{existing}{additional}")
        } else {
            format!("{existing}\n{additional}")
        };
        self.write(name, &new_content)
    }

    fn ensure_dir(&self) -> anyhow::Result<()> {
        if !self.memory_dir.exists() {
            std::fs::create_dir_all(&self.memory_dir)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn missing_directory_no_error() {
        let dir = tempfile::tempdir().unwrap();
        let bank = MemoryBank::load(dir.path());
        assert!(bank.names().is_empty());
        assert!(bank.to_context_string().is_empty());
    }

    #[test]
    fn loads_memory_files() {
        let dir = tempfile::tempdir().unwrap();
        let memory_dir = dir.path().join(".agents").join("memory");
        fs::create_dir_all(&memory_dir).unwrap();
        fs::write(memory_dir.join("project.md"), "Tech stack: Rust.\n").unwrap();

        let bank = MemoryBank::load(dir.path());
        assert_eq!(bank.get("project"), Some("Tech stack: Rust.\n"));
    }

    #[test]
    fn write_creates_new_memory() {
        let dir = tempfile::tempdir().unwrap();
        let mut bank = MemoryBank::load(dir.path());
        bank.write("notes", "first note").unwrap();

        assert_eq!(bank.get("notes"), Some("first note"));
        // File exists on disk.
        let path = dir.path().join(".agents").join("memory").join("notes.md");
        assert!(path.exists());
        assert_eq!(fs::read_to_string(path).unwrap(), "first note");
    }

    #[test]
    fn write_is_atomic_no_tmp_left_behind() {
        let dir = tempfile::tempdir().unwrap();
        let mut bank = MemoryBank::load(dir.path());
        bank.write("data", "content").unwrap();

        let tmp = dir.path().join(".agents").join("memory").join("data.md.tmp");
        assert!(!tmp.exists(), "tmp file should be renamed away");
    }

    #[test]
    fn append_to_existing() {
        let dir = tempfile::tempdir().unwrap();
        let memory_dir = dir.path().join(".agents").join("memory");
        fs::create_dir_all(&memory_dir).unwrap();
        fs::write(memory_dir.join("log.md"), "entry one\n").unwrap();

        let mut bank = MemoryBank::load(dir.path());
        bank.append("log", "entry two\n").unwrap();

        let content = bank.get("log").unwrap();
        assert!(content.contains("entry one"));
        assert!(content.contains("entry two"));
    }

    #[test]
    fn append_creates_new_file() {
        let dir = tempfile::tempdir().unwrap();
        let mut bank = MemoryBank::load(dir.path());
        bank.append("new", "hello").unwrap();
        assert_eq!(bank.get("new"), Some("hello"));
    }

    #[test]
    fn to_context_string_includes_all() {
        let dir = tempfile::tempdir().unwrap();
        let memory_dir = dir.path().join(".agents").join("memory");
        fs::create_dir_all(&memory_dir).unwrap();
        fs::write(memory_dir.join("alpha.md"), "alpha content").unwrap();
        fs::write(memory_dir.join("beta.md"), "beta content").unwrap();

        let bank = MemoryBank::load(dir.path());
        let ctx = bank.to_context_string();
        assert!(ctx.contains("Memory: alpha"));
        assert!(ctx.contains("alpha content"));
        assert!(ctx.contains("Memory: beta"));
        assert!(ctx.contains("beta content"));
    }
}
