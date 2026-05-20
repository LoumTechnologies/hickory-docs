//! Agent skills — reusable prompt templates from `.agents/skills/*.md`.
//!
//! Skills are named Markdown files that the agent can invoke by name.
//! The `$ARGUMENTS` placeholder in the file body is replaced with any
//! inline arguments supplied at invocation time.

use std::collections::HashMap;
use std::path::Path;

/// A single skill loaded from `.agents/skills/<name>.md`.
pub struct Skill {
    /// Skill name — the filename stem (without `.md`).
    pub name: String,
    /// First non-empty line of the file, used as a short description.
    pub description: String,
    /// Full file content.
    pub content: String,
}

/// Loads and provides access to project skills from `.agents/skills/`.
pub struct SkillLoader {
    skills: HashMap<String, Skill>,
}

impl SkillLoader {
    /// Load all `*.md` files from `<project_dir>/.agents/skills/`.
    ///
    /// Returns an empty loader if the directory does not exist.
    pub fn load(project_dir: &Path) -> Self {
        let skills_dir = project_dir.join(".agents").join("skills");
        if !skills_dir.is_dir() {
            return Self { skills: HashMap::new() };
        }

        let mut entries: Vec<_> = std::fs::read_dir(&skills_dir)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
            .collect();
        entries.sort_by_key(|e| e.file_name());

        let mut skills = HashMap::new();
        for entry in entries {
            let path = entry.path();
            let name = path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            if name.is_empty() {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(&path) else {
                continue;
            };
            let description = content
                .lines()
                .find(|l| !l.trim().is_empty())
                .unwrap_or("")
                .trim()
                .to_string();
            skills.insert(
                name.clone(),
                Skill {
                    name,
                    description,
                    content,
                },
            );
        }
        Self { skills }
    }

    /// Look up a skill by name (filename stem without `.md`).
    pub fn get(&self, name: &str) -> Option<&Skill> {
        self.skills.get(name)
    }

    /// Invoke a skill by name, substituting `$ARGUMENTS` with `args`.
    ///
    /// When `args` is `None`, the `$ARGUMENTS` placeholder is removed.
    /// Returns `None` if no skill with the given name exists.
    pub fn invoke(&self, name: &str, args: Option<&str>) -> Option<String> {
        let skill = self.skills.get(name)?;
        let substituted = skill
            .content
            .replace("$ARGUMENTS", args.unwrap_or(""));
        Some(substituted)
    }

    /// List available skills as `(name, description)` pairs, sorted by name.
    pub fn list(&self) -> Vec<(&str, &str)> {
        let mut pairs: Vec<_> = self
            .skills
            .values()
            .map(|s| (s.name.as_str(), s.description.as_str()))
            .collect();
        pairs.sort_by_key(|(name, _)| *name);
        pairs
    }

    /// True when no skills are loaded.
    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn make_skills_dir(dir: &std::path::Path) -> std::path::PathBuf {
        let skills = dir.join(".agents").join("skills");
        fs::create_dir_all(&skills).unwrap();
        skills
    }

    #[test]
    fn empty_skills_dir_no_error() {
        let dir = tempfile::tempdir().unwrap();
        make_skills_dir(dir.path());
        let loader = SkillLoader::load(dir.path());
        assert!(loader.is_empty());
    }

    #[test]
    fn missing_directory_no_error() {
        let dir = tempfile::tempdir().unwrap();
        let loader = SkillLoader::load(dir.path());
        assert!(loader.is_empty());
    }

    #[test]
    fn discovers_skills() {
        let dir = tempfile::tempdir().unwrap();
        let skills = make_skills_dir(dir.path());
        fs::write(skills.join("greet.md"), "Say hello to the user.\n").unwrap();

        let loader = SkillLoader::load(dir.path());
        assert!(!loader.is_empty());
        assert!(loader.get("greet").is_some());
    }

    #[test]
    fn invoke_with_args() {
        let dir = tempfile::tempdir().unwrap();
        let skills = make_skills_dir(dir.path());
        fs::write(
            skills.join("summarise.md"),
            "Summarise the following:\n\n$ARGUMENTS\n",
        )
        .unwrap();

        let loader = SkillLoader::load(dir.path());
        let result = loader.invoke("summarise", Some("some text")).unwrap();
        assert!(result.contains("some text"));
        assert!(!result.contains("$ARGUMENTS"));
    }

    #[test]
    fn invoke_without_args() {
        let dir = tempfile::tempdir().unwrap();
        let skills = make_skills_dir(dir.path());
        fs::write(skills.join("hello.md"), "Greet: $ARGUMENTS\n").unwrap();

        let loader = SkillLoader::load(dir.path());
        let result = loader.invoke("hello", None).unwrap();
        assert!(!result.contains("$ARGUMENTS"));
        assert!(result.contains("Greet: "));
    }

    #[test]
    fn invoke_missing_skill_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        make_skills_dir(dir.path());
        let loader = SkillLoader::load(dir.path());
        assert!(loader.invoke("nonexistent", None).is_none());
    }

    #[test]
    fn list_shows_name_and_first_line() {
        let dir = tempfile::tempdir().unwrap();
        let skills = make_skills_dir(dir.path());
        fs::write(skills.join("alpha.md"), "First line description.\nMore content.\n").unwrap();
        fs::write(skills.join("beta.md"), "Beta description.\n").unwrap();

        let loader = SkillLoader::load(dir.path());
        let list = loader.list();
        assert_eq!(list.len(), 2);
        // Sorted by name: alpha, beta.
        assert_eq!(list[0].0, "alpha");
        assert_eq!(list[0].1, "First line description.");
        assert_eq!(list[1].0, "beta");
    }
}
