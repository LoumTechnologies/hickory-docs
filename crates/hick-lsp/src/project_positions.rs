//! Byte provenance is converted to UTF-16 only at the language-tool boundary.
use hick_lang::{ByteOrigin, ProjectEntry};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Debug, Clone)]
pub(crate) struct ProjectProjection {
    pub primary: String,
    pub output: String,
    pub origins: Vec<ByteOrigin>,
    pub sources: Arc<BTreeMap<String, String>>,
}
impl ProjectProjection {
    pub fn new(
        entry: &ProjectEntry,
        sources: Arc<BTreeMap<String, String>>,
        primary: &str,
    ) -> Result<Self, String> {
        let output = std::str::from_utf8(&entry.bytes)
            .map_err(|_| "Binary entry has no language positions")?;
        for origin in &entry.origins {
            let source = sources
                .get(&origin.document)
                .ok_or_else(|| format!("Missing position source {}", origin.document))?;
            if source.get(origin.source.0..origin.source.1).is_none()
                || output.get(origin.output.0..origin.output.1).is_none()
            {
                return Err(format!(
                    "Invalid UTF-8 position origin in {}",
                    origin.document
                ));
            }
        }
        Ok(Self {
            primary: primary.into(),
            output: std::str::from_utf8(&entry.bytes)
                .map_err(|_| "Binary entry has no language positions")?
                .into(),
            origins: entry.origins.clone(),
            sources,
        })
    }
    pub fn source(&self, line: u32, column: u32) -> Option<(String, u32, u32)> {
        let byte = byte_position(&self.output, line, column)?;
        let mut origins: Vec<_> = self
            .origins
            .iter()
            .filter(|o| o.output.0 <= byte && byte < o.output.1)
            .collect();
        if origins.is_empty() {
            origins = self.origins.iter().filter(|o| o.output.1 == byte).collect();
        }
        let [origin] = origins.as_slice() else {
            return None;
        };
        let source = self.sources.get(&origin.document)?;
        let (line, column) = position(source, origin.source.0 + byte - origin.output.0)?;
        Some((origin.document.clone(), line, column))
    }
    pub fn generated(&self, document: &str, line: u32, column: u32) -> Option<(u32, u32)> {
        let source = self.sources.get(document)?;
        let byte = byte_position(source, line, column)?;
        let origins: Vec<_> = self
            .origins
            .iter()
            .filter(|o| o.document == document && o.source.0 <= byte && byte < o.source.1)
            .collect();
        let [origin] = origins.as_slice() else {
            return None;
        };
        position(&self.output, origin.output.0 + byte - origin.source.0)
    }
    pub fn generated_line(&self, document: &str, line: u32) -> Option<u32> {
        let mut rows = std::collections::BTreeSet::new();
        let source = self.sources.get(document)?;
        for origin in &self.origins {
            if origin.document != document {
                continue;
            }
            let source_range = source.get(origin.source.0..origin.source.1)?;
            let mut offset = 0;
            for chunk in source_range.split_inclusive('\n') {
                let (source_line, _) = position(source, origin.source.0 + offset)?;
                if source_line == line {
                    rows.insert(position(&self.output, origin.output.0 + offset)?.0);
                }
                offset += chunk.len();
            }
        }
        if rows.len() == 1 {
            rows.into_iter().next()
        } else {
            None
        }
    }
}
pub(crate) fn position(text: &str, byte: usize) -> Option<(u32, u32)> {
    let prefix = text.get(..byte)?;
    Some((
        prefix.bytes().filter(|b| *b == b'\n').count() as u32,
        prefix.rsplit('\n').next()?.encode_utf16().count() as u32,
    ))
}
pub(crate) fn byte_position(text: &str, line: u32, column: u32) -> Option<usize> {
    let mut start = 0;
    for (number, chunk) in text.split_inclusive('\n').enumerate() {
        if number as u32 == line {
            let mut utf16 = 0;
            for (offset, ch) in chunk.char_indices() {
                if utf16 == column {
                    return Some(start + offset);
                }
                if ch == '\n' {
                    return None;
                }
                utf16 += ch.len_utf16() as u32;
            }
            return (utf16 == column).then_some(start + chunk.len());
        }
        start += chunk.len();
    }
    (column == 0
        && line == text.bytes().filter(|b| *b == b'\n').count() as u32
        && (text.is_empty() || text.ends_with('\n')))
    .then_some(text.len())
}
