//! Bidirectional position mapping between hick source and virtual file coordinates.

use crate::virtual_file::VirtualSegment;

/// A mapping entry from virtual file line to hick source line.
#[derive(Debug, Clone)]
pub struct LineMapping {
    /// 0-based line in the virtual file.
    pub virtual_line: u32,
    /// 0-based line in the .hick source file.
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
}

impl PositionMap {
    /// The map for a file that is its own virtual file: `lines` long, every
    /// line and column unchanged in both directions.
    pub fn identity(lines: u32) -> Self {
        Self {
            mappings: Vec::new(),
            identity_lines: Some(lines),
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
                mappings.push(LineMapping {
                    virtual_line,
                    source_line: base_source_line + i as u32,
                    column_offset,
                });
                virtual_line += 1;
            }
        }

        Self {
            mappings,
            identity_lines: None,
        }
    }

    /// How many lines the virtual file has.
    ///
    /// Needed by the range-scoped requests: `textDocument/inlayHint` takes a
    /// range and takes it as REQUIRED, so a child asked without one answers
    /// nothing at all. The document's own range is meaningless to a child —
    /// its file is a few lines long and the document is not — so each is
    /// asked for the extent of its own file.
    pub fn virtual_lines(&self) -> u32 {
        self.identity_lines.unwrap_or(self.mappings.len() as u32)
    }

    /// Map a virtual file position to a .hick source position.
    /// Returns (source_line_0based, source_col_0based).
    pub fn to_source(&self, virtual_line: u32, virtual_col: u32) -> Option<(u32, u32)> {
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

    /// Map a .hick source position to a virtual file position.
    /// Returns (virtual_line_0based, virtual_col_0based).
    pub fn to_virtual(&self, source_line: u32, source_col: u32) -> Option<(u32, u32)> {
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
