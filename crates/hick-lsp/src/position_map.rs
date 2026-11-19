//! Bidirectional position mapping between hick source and virtual file coordinates.

use crate::virtual_file::VirtualSegment;

/// A mapping entry from virtual file line to hick source line.
#[derive(Debug, Clone)]
pub struct LineMapping {
    /// 0-based line in the virtual file.
    pub virtual_line: u32,
    /// 0-based line in the .md source file.
    pub source_line: u32,
    /// Column offset to add when translating virtual -> source.
    pub column_offset: i32,
}

/// Position map for a single virtual file.
#[derive(Debug, Clone)]
pub struct PositionMap {
    pub mappings: Vec<LineMapping>,
    /// Set when the virtual file IS the source file, line for line.
    ///
    /// A plain file opened in the app is handed to its language server as
    /// itself, so every position maps to itself. Recorded as a flag rather
    /// than as N trivial mappings because every lookup below is a linear
    /// scan, and a semantic-token pass over a long file asks thousands of
    /// times.
    identity_lines: Option<u32>,
    line_count: u32,
    project: Option<crate::project_positions::ProjectProjection>,
    inspection: bool,
}

impl PositionMap {
    /// Inspection maps provide positions only, never an editable preimage.
    pub(crate) fn inspection(reading: &crate::fenced_source::FencedReading, sources: std::sync::Arc<std::collections::BTreeMap<String,String>>) -> Result<Self,String> {
        let project=crate::project_positions::ProjectProjection::literal(reading.content.clone(),reading.origins.clone(),sources,"source")?;
        Ok(Self{mappings:vec![],identity_lines:None,line_count:reading.content.split('\n').count() as u32,project:Some(project),inspection:true})
    }
    pub(crate) fn is_inspection(&self)->bool {self.inspection}
    pub fn for_project(
        entry: &hick_lang::ProjectEntry,
        sources: std::sync::Arc<std::collections::BTreeMap<String, String>>,
        primary: &str,
    ) -> Result<Self, String> {
        let projection = crate::project_positions::ProjectProjection::new(entry, sources, primary)?;
        let line_count = projection.output.split_inclusive('\n').count() as u32;
        let mut mappings = vec![];
        for origin in &projection.origins {
            if origin.document != primary {
                continue;
            }
            let source = &projection.sources[primary];
            let mut offset = 0;
            for line in projection.output[origin.output.0..origin.output.1].split_inclusive('\n') {
                let (virtual_line, virtual_column) = crate::project_positions::position(
                    &projection.output,
                    origin.output.0 + offset,
                )
                .ok_or("Invalid output provenance")?;
                let (source_line, source_column) =
                    crate::project_positions::position(source, origin.source.0 + offset)
                        .ok_or("Invalid source provenance")?;
                mappings.push(LineMapping {
                    virtual_line,
                    source_line,
                    column_offset: source_column as i32 - virtual_column as i32,
                });
                offset += line.len();
            }
        }
        Ok(Self {
            mappings,
            identity_lines: None,
            line_count,
            project: Some(projection),
            inspection: false,
        })
    }
    /// Source identity accompanies coordinates; foreign contributors never
    /// masquerade as positions in the primary document.
    pub fn is_project(&self) -> bool {
        self.project.is_some() && !self.inspection
    }
    pub fn source_document(&self, line: u32, column: u32) -> Option<(Option<String>, u32, u32)> {
        if self.inspection { return self.to_source(line,column).map(|(l,c)|(None,l,c)); }
        if let Some(project) = &self.project {
            return project
                .source(line, column)
                .map(|(doc, l, c)| (Some(doc), l, c));
        }
        self.to_source(line, column).map(|(l, c)| (None, l, c))
    }
    pub fn generated_line_in(&self, document: &str, line: u32) -> Option<u32> {
        self.project.as_ref()?.generated_line(document, line)
    }

    /// The map for a file that is its own virtual file: `lines` long, every
    /// line and column unchanged in both directions.
    pub fn identity(lines: u32) -> Self {
        Self {
            mappings: Vec::new(),
            identity_lines: Some(lines),
            line_count: lines,
            project: None,
            inspection: false,
        }
    }

    /// Whether this map is [`PositionMap::identity`].
    pub fn is_identity(&self) -> bool {
        self.identity_lines.is_some()
    }

    /// Build a position map from virtual file segments.
    pub fn build(segments: &[VirtualSegment]) -> Self {
        let mut mappings = Vec::new();
        let mut virtual_line: u32 = 0;

        for segment in segments {
            // source_line is 1-based, convert to 0-based
            let base_source_line = if let Some(span) = segment.source_span {
                (span.start_line as u32).saturating_sub(1)
            } else {
                (segment.source_line as u32).saturating_sub(1)
            };

            let column_offset = segment.source_column as i32;

            let lines: Vec<&str> = segment.text.split('\n').collect();
            // If the text ends with '\n', split produces a trailing empty element
            // which does not represent an actual line of content.
            let line_count = if segment.text.ends_with('\n') && lines.len() > 1 {
                lines.len() - 1
            } else {
                lines.len()
            };

            for i in 0..line_count {
                if segment.source_line != 0 || segment.source_span.is_some() {
                    mappings.push(LineMapping {
                        virtual_line,
                        source_line: base_source_line + i as u32,
                        column_offset,
                    });
                }
                virtual_line += 1;
            }
        }

        Self {
            mappings,
            identity_lines: None,
            line_count: virtual_line,
            project: None,
            inspection: false,
        }
    }

    /// The same map for a copy of the file with its **first line removed**.
    ///
    /// A virtual file's content begins with the newline that follows the
    /// `<hick:file>` tag, so line 0 belongs to the tag's own line. Weaving
    /// drops that newline. Anyone consuming the woven bytes also needs the
    /// shifted map so a first-line edit lands inside the authored body.
    pub fn without_first_line(mut self) -> Self {
        if let Some(lines) = self.identity_lines {
            self.identity_lines = Some(lines.saturating_sub(1));
            return self;
        }
        self.line_count = self.line_count.saturating_sub(1);
        self.mappings.retain(|mapping| mapping.virtual_line != 0);
        for mapping in &mut self.mappings {
            mapping.virtual_line = mapping.virtual_line.saturating_sub(1);
        }
        self
    }

    /// How many lines the virtual file has.
    ///
    /// Needed by the range-scoped requests: `textDocument/inlayHint` takes a
    /// range and takes it as REQUIRED, so a child asked without one answers
    /// nothing at all. The document's own range is meaningless to a child —
    /// its file is a few lines long and the document is not — so each is
    /// asked for the extent of its own file.
    pub fn virtual_lines(&self) -> u32 {
        self.identity_lines.unwrap_or(self.line_count)
    }

    /// Map a virtual file position to a .md source position.
    /// Returns (source_line_0based, source_col_0based).
    pub fn to_source(&self, virtual_line: u32, virtual_col: u32) -> Option<(u32, u32)> {
        if let Some(project) = &self.project {
            let (document, line, column) = project.source(virtual_line, virtual_col)?;
            return (document == project.primary).then_some((line, column));
        }
        if self.identity_lines.is_some() {
            return Some((virtual_line, virtual_col));
        }
        let mapping = self
            .mappings
            .iter()
            .find(|m| m.virtual_line == virtual_line)?;
        let source_col = if mapping.column_offset >= 0 {
            virtual_col + mapping.column_offset as u32
        } else {
            virtual_col.checked_sub((-mapping.column_offset) as u32)?
        };
        Some((mapping.source_line, source_col))
    }

    /// Map a .md source position to a virtual file position.
    /// Returns (virtual_line_0based, virtual_col_0based).
    pub fn to_virtual(&self, source_line: u32, source_col: u32) -> Option<(u32, u32)> {
        if let Some(project) = &self.project {
            return project.generated(&project.primary, source_line, source_col);
        }
        if self.identity_lines.is_some() {
            return Some((source_line, source_col));
        }
        let mapping = self
            .mappings
            .iter()
            .find(|m| m.source_line == source_line)?;
        let virtual_col = if mapping.column_offset >= 0 {
            source_col.checked_sub(mapping.column_offset as u32)?
        } else {
            source_col + (-mapping.column_offset) as u32
        };
        Some((mapping.virtual_line, virtual_col))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hick_lang::SourceSpan;

    fn seg(text: &str, start_line: usize, col: usize) -> VirtualSegment {
        VirtualSegment {
            text: text.to_string(),
            source_span: Some(SourceSpan::new(0, text.len(), start_line, col)),
            source_line: start_line,
            source_column: col,
        }
    }

    #[test]
    fn single_segment_single_line() {
        let segments = vec![seg("fn main() {}\n", 5, 4)];
        let map = PositionMap::build(&segments);

        assert_eq!(map.mappings.len(), 1);
        assert_eq!(map.mappings[0].virtual_line, 0);
        assert_eq!(map.mappings[0].source_line, 4); // 5 - 1 = 4 (0-based)
        assert_eq!(map.mappings[0].column_offset, 4);
    }

    #[test]
    fn single_segment_multi_line() {
        let segments = vec![seg("line1\nline2\nline3\n", 3, 2)];
        let map = PositionMap::build(&segments);

        assert_eq!(map.mappings.len(), 3);
        assert_eq!(map.mappings[0].virtual_line, 0);
        assert_eq!(map.mappings[0].source_line, 2);
        assert_eq!(map.mappings[1].virtual_line, 1);
        assert_eq!(map.mappings[1].source_line, 3);
        assert_eq!(map.mappings[2].virtual_line, 2);
        assert_eq!(map.mappings[2].source_line, 4);
    }

    #[test]
    fn multiple_segments() {
        let segments = vec![seg("aaa\n", 2, 0), seg("bbb\nccc\n", 5, 4)];
        let map = PositionMap::build(&segments);

        assert_eq!(map.mappings.len(), 3);
        // First segment
        assert_eq!(map.mappings[0].virtual_line, 0);
        assert_eq!(map.mappings[0].source_line, 1); // line 2, 0-based = 1
        // Second segment
        assert_eq!(map.mappings[1].virtual_line, 1);
        assert_eq!(map.mappings[1].source_line, 4); // line 5, 0-based = 4
        assert_eq!(map.mappings[2].virtual_line, 2);
        assert_eq!(map.mappings[2].source_line, 5);
    }

    #[test]
    fn to_source_basic() {
        let segments = vec![seg("hello\nworld\n", 3, 4)];
        let map = PositionMap::build(&segments);

        // virtual (0, 5) -> source (2, 9)
        assert_eq!(map.to_source(0, 5), Some((2, 9)));
        // virtual (1, 0) -> source (3, 4)
        assert_eq!(map.to_source(1, 0), Some((3, 4)));
    }

    #[test]
    fn to_source_unknown_line() {
        let segments = vec![seg("x\n", 1, 0)];
        let map = PositionMap::build(&segments);
        assert_eq!(map.to_source(99, 0), None);
    }

    #[test]
    fn to_virtual_basic() {
        let segments = vec![seg("hello\nworld\n", 3, 4)];
        let map = PositionMap::build(&segments);

        // source (2, 9) -> virtual (0, 5)
        assert_eq!(map.to_virtual(2, 9), Some((0, 5)));
        // source (3, 4) -> virtual (1, 0)
        assert_eq!(map.to_virtual(3, 4), Some((1, 0)));
    }

    #[test]
    fn to_virtual_col_less_than_offset_returns_none() {
        let segments = vec![seg("x\n", 3, 4)];
        let map = PositionMap::build(&segments);
        // source col 2 < offset 4 => None
        assert_eq!(map.to_virtual(2, 2), None);
    }

    #[test]
    fn to_virtual_unknown_line() {
        let segments = vec![seg("x\n", 1, 0)];
        let map = PositionMap::build(&segments);
        assert_eq!(map.to_virtual(99, 0), None);
    }

    #[test]
    fn round_trip() {
        let segments = vec![
            seg("fn main() {\n", 10, 8),
            seg("    println!(\"hi\");\n", 11, 8),
            seg("}\n", 12, 8),
        ];
        let map = PositionMap::build(&segments);

        // Round-trip: virtual -> source -> virtual
        for vl in 0..3u32 {
            let (sl, sc) = map.to_source(vl, 0).unwrap();
            let (vl2, vc2) = map.to_virtual(sl, sc).unwrap();
            assert_eq!(vl, vl2);
            assert_eq!(0, vc2);
        }
    }

    #[test]
    fn no_trailing_newline() {
        let segments = vec![seg("no newline", 1, 0)];
        let map = PositionMap::build(&segments);
        assert_eq!(map.mappings.len(), 1);
        assert_eq!(map.mappings[0].virtual_line, 0);
        assert_eq!(map.mappings[0].source_line, 0);
    }

    #[test]
    fn empty_segment() {
        let segments = vec![seg("", 1, 0)];
        let map = PositionMap::build(&segments);
        // Empty string split by \n gives [""], which is 1 line
        assert_eq!(map.mappings.len(), 1);
    }
}
impl PositionMap {
    pub fn source_edit_range(
        &self,
        range: tower_lsp::lsp_types::Range,
    ) -> Option<(Option<String>, tower_lsp::lsp_types::Range)> {
        use tower_lsp::lsp_types::{Position, Range};
        if self.inspection || range.end < range.start {
            return None;
        }
        if let Some(project) = &self.project {
            return project
                .source_edit_range(range)
                .map(|(path, range)| (Some(path), range));
        }
        if self.is_identity() {
            return Some((None, range));
        }
        let first = self
            .mappings
            .iter()
            .find(|m| m.virtual_line == range.start.line)?;
        let last = if range.end.character == 0 && range.end.line > range.start.line {
            range.end.line - 1
        } else {
            range.end.line
        };
        let count = last.checked_sub(range.start.line)?.checked_add(1)?;
        let mut lines: Vec<_> = self
            .mappings
            .iter()
            .filter(|m| m.virtual_line >= range.start.line && m.virtual_line <= last)
            .collect();
        lines.sort_by_key(|m| m.virtual_line);
        if lines.len() != count as usize
            || lines.iter().enumerate().any(|(i, m)| {
                m.virtual_line != range.start.line + i as u32
                    || m.source_line != first.source_line.checked_add(i as u32).unwrap_or(u32::MAX)
                    || m.column_offset != first.column_offset
            })
        {
            return None;
        }
        let (sl, sc) = self.to_source(range.start.line, range.start.character)?;
        let (el, ec) = if range.end.character == 0 && range.end.line > last {
            (lines.last()?.source_line.checked_add(1)?, 0)
        } else {
            self.to_source(range.end.line, range.end.character)?
        };
        Some((
            None,
            Range::new(Position::new(sl, sc), Position::new(el, ec)),
        ))
    }
}
impl PositionMap {
    pub fn project_source(&self, document: &str) -> Option<&str> {
        if self.inspection { return None; }
        self.project
            .as_ref()?
            .sources
            .get(document)
            .map(String::as_str)
    }
}
impl PositionMap {
    /// Actual byte-origin contributors, not every source loaded for the project.
    pub fn contributing_documents(&self) -> Vec<&str> {
        if self.inspection { return vec![]; }
        self.project
            .as_ref()
            .map(|project| {
                project
                    .origins
                    .iter()
                    .filter(|origin| origin.output.0 < origin.output.1)
                    .map(|origin| origin.document.as_str())
                    .collect::<std::collections::BTreeSet<_>>()
                    .into_iter()
                    .collect()
            })
            .unwrap_or_default()
    }
}
#[cfg(test)]
mod contributing_document_tests {
    use super::*;
    #[test]
    fn repeated_origins_keep_contributors_but_exclude_unrelated_loaded_sources() {
        let mut entry =
            hick_lang::ProjectEntry::file(std::sync::Arc::<[u8]>::from(&b"one\none\ntwo\n"[..]));
        entry.origins = vec![
            hick_lang::ByteOrigin {
                output: (0, 4),
                document: "a.md".into(),
                source: (0, 4),
            },
            hick_lang::ByteOrigin {
                output: (4, 8),
                document: "a.md".into(),
                source: (0, 4),
            },
            hick_lang::ByteOrigin {
                output: (8, 12),
                document: "b.md".into(),
                source: (0, 4),
            },
        ];
        let sources = std::sync::Arc::new(std::collections::BTreeMap::from([
            ("a.md".into(), "one\n".into()),
            ("b.md".into(), "two\n".into()),
            ("unrelated.md".into(), "not emitted\n".into()),
        ]));
        let map = PositionMap::for_project(&entry, sources, "a.md").unwrap();
        assert_eq!(
            map.generated_line_in("a.md", 0),
            None,
            "repeated copies have no single generated line"
        );
        assert_eq!(map.contributing_documents(), vec!["a.md", "b.md"]);
        assert!(PositionMap::identity(3).contributing_documents().is_empty());
    }
}
