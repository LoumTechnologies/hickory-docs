//! In-memory volume state management using tar archives.
//!
//! Each volume's contents are stored as a tar archive in memory. The
//! pipeline seeds input volumes from host directories, passes them to
//! containers via tar injection, and extracts updated state after exec.

use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

use anyhow::{Context, Result};

/// In-memory store for volume contents.
///
/// Each volume is represented as a tar archive (raw bytes). This allows
/// efficient injection into containers and extraction from them without
/// needing to mount actual filesystems.
pub struct VolumeStore {
    /// Volume name → tar archive bytes.
    archives: HashMap<String, Vec<u8>>,
}

impl Default for VolumeStore {
    fn default() -> Self {
        Self::new()
    }
}

impl VolumeStore {
    pub fn new() -> Self {
        Self {
            archives: HashMap::new(),
        }
    }

    /// Seed a volume from a host directory by creating a tar archive of its contents.
    pub fn seed_from_directory(&mut self, name: &str, dir: &Path) -> Result<()> {
        let mut builder = tar::Builder::new(Vec::new());
        builder
            .append_dir_all(".", dir)
            .with_context(|| format!("failed to tar directory: {}", dir.display()))?;
        let data = builder
            .into_inner()
            .context("failed to finalize tar archive")?;
        self.archives.insert(name.to_string(), data);
        Ok(())
    }

    /// Seed a volume with raw tar data.
    pub fn seed_tar(&mut self, name: &str, data: Vec<u8>) {
        self.archives.insert(name.to_string(), data);
    }

    /// Get the tar archive for a volume, if it exists.
    pub fn get(&self, name: &str) -> Option<&[u8]> {
        self.archives.get(name).map(|v| v.as_slice())
    }

    /// Update a volume with new tar data (replaces entirely).
    pub fn update(&mut self, name: &str, data: Vec<u8>) {
        self.archives.insert(name.to_string(), data);
    }

    /// Check if a volume has been seeded.
    pub fn contains(&self, name: &str) -> bool {
        self.archives.contains_key(name)
    }

    /// List all volume names.
    pub fn volumes(&self) -> Vec<&str> {
        self.archives.keys().map(|s| s.as_str()).collect()
    }

    /// Unpack a volume's tar archive into a HashMap of path → content.
    ///
    /// Only extracts regular files. Returns file contents as strings
    /// (lossy UTF-8 conversion for binary files).
    pub fn unpack_to_files(&self, name: &str) -> Result<HashMap<String, String>> {
        let data = self
            .archives
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("volume '{name}' not found"))?;

        let mut files = HashMap::new();
        let mut archive = tar::Archive::new(data.as_slice());

        for entry in archive.entries().context("failed to read tar entries")? {
            let mut entry = entry.context("failed to read tar entry")?;
            if entry.header().entry_type().is_file() {
                let path = entry
                    .path()
                    .context("failed to read tar entry path")?
                    .to_string_lossy()
                    .into_owned();

                // Strip leading "./" if present
                let path = path.strip_prefix("./").unwrap_or(&path).to_string();
                if path.is_empty() {
                    continue;
                }

                let mut content = String::new();
                entry
                    .read_to_string(&mut content)
                    .with_context(|| format!("failed to read tar entry: {path}"))?;
                files.insert(path, content);
            }
        }

        Ok(files)
    }
}

/// Create a tar archive from a set of files.
pub fn pack_files(files: &HashMap<String, String>) -> Result<Vec<u8>> {
    let mut builder = tar::Builder::new(Vec::new());

    let mut sorted_paths: Vec<&String> = files.keys().collect();
    sorted_paths.sort();

    for path in sorted_paths {
        let content = &files[path];
        let mut header = tar::Header::new_gnu();
        header.set_size(content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();

        builder
            .append_data(&mut header, path, content.as_bytes())
            .with_context(|| format!("failed to add {path} to tar"))?;
    }

    builder.into_inner().context("failed to finalize tar")
}

/// Read a tar archive into `(path, bytes)` pairs, in archive order.
///
/// Bytes, not strings: a volume can carry a binary, and the enforcement path
/// must not be the reason a document that worked with unrestricted volumes
/// stops working with restricted ones. Paths are normalized the same way
/// [`VolumeStore::unpack_to_files`] normalizes them (leading `./` stripped),
/// so a pattern in `<hick:allow read="src/**">` matches what the author sees.
pub fn read_tar_files(data: &[u8]) -> Result<Vec<(String, Vec<u8>)>> {
    let mut out = Vec::new();
    let mut archive = tar::Archive::new(data);
    for entry in archive.entries().context("failed to read tar entries")? {
        let mut entry = entry.context("failed to read tar entry")?;
        if !entry.header().entry_type().is_file() {
            continue;
        }
        let path = entry
            .path()
            .context("failed to read tar entry path")?
            .to_string_lossy()
            .into_owned();
        let path = path.strip_prefix("./").unwrap_or(&path).to_string();
        if path.is_empty() {
            continue;
        }
        let mut content = Vec::new();
        std::io::Read::read_to_end(&mut entry, &mut content)
            .with_context(|| format!("failed to read tar entry: {path}"))?;
        out.push((path, content));
    }
    Ok(out)
}

/// Pack `(path, bytes)` pairs into a tar archive, sorted by path so the same
/// contents always produce the same bytes (cache keys depend on it).
pub fn pack_tar_files(files: &[(String, Vec<u8>)]) -> Result<Vec<u8>> {
    let mut sorted: Vec<&(String, Vec<u8>)> = files.iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));

    let mut builder = tar::Builder::new(Vec::new());
    for (path, content) in sorted {
        let mut header = tar::Header::new_gnu();
        header.set_size(content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, path, content.as_slice())
            .with_context(|| format!("failed to add {path} to tar"))?;
    }
    builder.into_inner().context("failed to finalize tar")
}

/// Keep only the entries of `data` whose path `permits` accepts.
///
/// This is how a partial read grant is enforced: a container allowed
/// `read="src/**"` is handed a volume containing `src/**` and nothing else,
/// rather than the whole archive plus a promise not to look.
pub fn filter_tar(data: &[u8], permits: impl Fn(&str) -> bool) -> Result<Vec<u8>> {
    let kept: Vec<(String, Vec<u8>)> = read_tar_files(data)?
        .into_iter()
        .filter(|(path, _)| permits(path))
        .collect();
    pack_tar_files(&kept)
}

/// Merge a container's writes into a volume, accepting only the paths
/// `permits` accepts.
///
/// `base` is the volume as it stood before the container ran; `written` is
/// what came back out of it. Entries the container may not write keep their
/// base content, so a partial writer cannot smuggle a change into a path it
/// was not granted.
///
/// **Deletions do not propagate through a partial grant.** Absence in
/// `written` is indistinguishable from "never mounted it", and treating it as
/// a delete would let a restricted writer erase what it cannot overwrite.
pub fn merge_permitted_writes(
    base: Option<&[u8]>,
    written: &[u8],
    permits: impl Fn(&str) -> bool,
) -> Result<Vec<u8>> {
    let mut merged: Vec<(String, Vec<u8>)> = match base {
        Some(data) => read_tar_files(data)?,
        None => Vec::new(),
    };
    for (path, content) in read_tar_files(written)? {
        if !permits(&path) {
            continue;
        }
        match merged.iter_mut().find(|(p, _)| *p == path) {
            Some(existing) => existing.1 = content,
            None => merged.push((path, content)),
        }
    }
    pack_tar_files(&merged)
}

/// Encode bytes as base64.
pub fn encode_base64(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}

/// Decode base64 string to bytes.
///
/// Strips all ASCII whitespace before decoding so that line-wrapped output
/// from the container's `base64` command (which wraps at 76 columns) is
/// handled transparently.
pub fn decode_base64(encoded: &str) -> Result<Vec<u8>> {
    use base64::Engine;
    let clean: String = encoded
        .chars()
        .filter(|c| !c.is_ascii_whitespace())
        .collect();
    base64::engine::general_purpose::STANDARD
        .decode(&clean)
        .context("failed to decode base64")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn volume_store_seed_and_get() {
        let mut store = VolumeStore::new();

        // Create a tar manually
        let mut files = HashMap::new();
        files.insert("hello.txt".to_string(), "world".to_string());
        let tar_data = pack_files(&files).unwrap();

        store.seed_tar("test-vol", tar_data);
        assert!(store.contains("test-vol"));
        assert!(store.get("test-vol").is_some());
    }

    #[test]
    fn volume_store_unpack() {
        let mut store = VolumeStore::new();

        let mut files = HashMap::new();
        files.insert("a.txt".to_string(), "aaa".to_string());
        files.insert("sub/b.txt".to_string(), "bbb".to_string());
        let tar_data = pack_files(&files).unwrap();

        store.seed_tar("vol", tar_data);
        let unpacked = store.unpack_to_files("vol").unwrap();
        assert_eq!(unpacked.get("a.txt").unwrap(), "aaa");
        assert_eq!(unpacked.get("sub/b.txt").unwrap(), "bbb");
    }

    #[test]
    fn volume_store_update_replaces() {
        let mut store = VolumeStore::new();

        let mut files1 = HashMap::new();
        files1.insert("file.txt".to_string(), "v1".to_string());
        store.seed_tar("vol", pack_files(&files1).unwrap());

        let mut files2 = HashMap::new();
        files2.insert("file.txt".to_string(), "v2".to_string());
        store.update("vol", pack_files(&files2).unwrap());

        let unpacked = store.unpack_to_files("vol").unwrap();
        assert_eq!(unpacked.get("file.txt").unwrap(), "v2");
    }

    /// Protects docs/guarantees/execution/declared-capabilities-are-enforced.md
    #[test]
    fn filtering_and_merging_survive_binary_content() {
        // The enforcement path must not be the reason a volume carrying
        // something that is not UTF-8 stops working.
        let binary = vec![0xffu8, 0x00, 0xfe, 0x01];
        let base = pack_tar_files(&[
            ("keep.bin".to_string(), binary.clone()),
            ("Models.cs".to_string(), b"base".to_vec()),
        ])
        .unwrap();

        // A partial read grant hands over only its subtree.
        let scoped = filter_tar(&base, |p| p == "keep.bin").unwrap();
        let seen = read_tar_files(&scoped).unwrap();
        assert_eq!(seen, vec![("keep.bin".to_string(), binary.clone())]);

        // A partial write grant merges only its subtree back.
        let written = pack_tar_files(&[
            ("keep.bin".to_string(), b"rewritten".to_vec()),
            ("Models.cs".to_string(), b"hacked".to_vec()),
        ])
        .unwrap();
        let merged = merge_permitted_writes(Some(&base), &written, |p| p == "keep.bin").unwrap();
        let after: HashMap<String, Vec<u8>> =
            read_tar_files(&merged).unwrap().into_iter().collect();
        assert_eq!(after.get("keep.bin").unwrap(), b"rewritten");
        assert_eq!(after.get("Models.cs").unwrap(), b"base");
    }

    #[test]
    fn tar_roundtrip() {
        let mut files = HashMap::new();
        files.insert("README.md".to_string(), "# Hello".to_string());
        files.insert("src/main.rs".to_string(), "fn main() {}".to_string());
        files.insert(
            "data/config.json".to_string(),
            r#"{"key":"value"}"#.to_string(),
        );

        let tar_data = pack_files(&files).unwrap();
        assert!(!tar_data.is_empty());

        let mut store = VolumeStore::new();
        store.seed_tar("roundtrip", tar_data);
        let unpacked = store.unpack_to_files("roundtrip").unwrap();

        assert_eq!(unpacked.len(), 3);
        assert_eq!(unpacked["README.md"], "# Hello");
        assert_eq!(unpacked["src/main.rs"], "fn main() {}");
    }

    #[test]
    fn base64_roundtrip() {
        let data = b"hello world tar data";
        let encoded = encode_base64(data);
        let decoded = decode_base64(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn base64_decode_with_newlines() {
        // Simulates output from the container's `base64` command which wraps
        // at 76 columns.
        let data = b"hello world tar data with enough bytes to wrap";
        let encoded = encode_base64(data);
        // Insert newlines every 20 chars to simulate wrapping
        let wrapped: String = encoded
            .as_bytes()
            .chunks(20)
            .map(|c| std::str::from_utf8(c).unwrap())
            .collect::<Vec<_>>()
            .join("\n");
        let decoded = decode_base64(&wrapped).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn seed_from_directory() {
        let dir = std::env::temp_dir().join("hick-vol-test-seed");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("sub")).unwrap();

        let mut f = std::fs::File::create(dir.join("file.txt")).unwrap();
        f.write_all(b"content").unwrap();

        let mut f = std::fs::File::create(dir.join("sub/nested.txt")).unwrap();
        f.write_all(b"nested content").unwrap();

        let mut store = VolumeStore::new();
        store.seed_from_directory("dir-vol", &dir).unwrap();

        let unpacked = store.unpack_to_files("dir-vol").unwrap();
        assert_eq!(unpacked.get("file.txt").unwrap(), "content");
        assert_eq!(unpacked.get("sub/nested.txt").unwrap(), "nested content");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
