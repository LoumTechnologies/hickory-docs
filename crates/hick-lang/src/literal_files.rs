//! The portable, non-executing literal-file subset. Unsupported dependencies
//! are refused, never silently dropped. Coordinates remain source UTF-8 bytes.
use crate::{HickNode, SourceSpan, dedent, parse};

/// Remove the opening tag's line break, preserving the source coordinate.
pub fn strip_opening_break<'a>(
    text: &'a str,
    span: Option<&SourceSpan>,
) -> (&'a str, Option<SourceSpan>) {
    let stripped = text
        .strip_prefix("\r\n")
        .map(|s| (s, 2))
        .or_else(|| text.strip_prefix('\n').map(|s| (s, 1)));
    let Some((rest, taken)) = stripped else {
        return (text, span.copied());
    };
    (
        rest,
        span.map(|s| SourceSpan {
            start: s.start + taken,
            start_line: s.start_line + 1,
            start_col: 0,
            ..*s
        }),
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct LiteralSegment {
    pub output: (usize, usize),
    pub source: (usize, usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct LiteralFile {
    pub path: String,
    pub content: String,
    pub segments: Vec<LiteralSegment>,
}

/// Materialize unconditional top-level files whose children are literal text.
/// This uses the native file path's opening-break and dedent operations.
pub fn literal_files(source: &str) -> Result<Vec<LiteralFile>, String> {
    let doc = parse(source).map_err(|e| e.to_string())?;
    let shift = usize::from(source.starts_with('\u{feff}')) * 3;
    let mut files = Vec::new();
    for node in &doc.nodes {
        let HickNode::Tag(tag) = node else { continue };
        if matches!(tag.name.as_str(), "when" | "include" | "upstream") {
            return Err(format!(
                "<{}:{}> requires workspace/fact resolution",
                doc.prefix, tag.name
            ));
        }
        if tag.name != "file" {
            continue;
        }
        let path = tag
            .get_attribute("path")
            .ok_or("file requires path")?
            .to_string();
        if files.iter().any(|f: &LiteralFile| f.path == path) {
            return Err(format!(
                "multiple file declarations for {path} require full weave"
            ));
        }
        let mut file = LiteralFile {
            path,
            content: String::new(),
            segments: Vec::new(),
        };
        for (i, child) in tag.children.iter().enumerate() {
            let HickNode::Text(text, span) = child else {
                return Err(format!(
                    "{} has nonliteral children requiring full weave",
                    file.path
                ));
            };
            let (text, span) = if i == 0 {
                strip_opening_break(text, span.as_ref())
            } else {
                (text.as_str(), *span)
            };
            let mut at = span.ok_or("literal text has no source span")?.start + shift;
            if tag.source_column == 0 {
                let start = file.content.len();
                file.content.push_str(text);
                if !text.is_empty() {
                    file.segments.push(LiteralSegment {
                        output: (start, file.content.len()),
                        source: (at, at + text.len()),
                    });
                }
            } else {
                let woven = dedent(text, tag.source_column);
                let (text, skipped) = text.strip_prefix('\n').map_or((text, 0), |rest| (rest, 1));
                at += skipped;
                let start = file.content.len();
                let mut output = start;
                let mut previous_newline = None;
                for (i, line) in text.lines().enumerate() {
                    if i > 0 {
                        let newline = previous_newline.expect("a prior line has a newline");
                        file.segments.push(LiteralSegment {
                            output: (output, output + 1),
                            source: (newline, newline + 1),
                        });
                        output += 1;
                    }
                    let removed = line
                        .bytes()
                        .take_while(|b| *b == b' ')
                        .count()
                        .min(tag.source_column);
                    let length = line.len() - removed;
                    if length > 0 {
                        file.segments.push(LiteralSegment {
                            output: (output, output + length),
                            source: (at + removed, at + line.len()),
                        });
                    }
                    output += length;
                    let consumed = text[at - (span.unwrap().start + shift + skipped)..]
                        .find('\n')
                        .map(|n| n + 1);
                    if let Some(consumed) = consumed {
                        previous_newline = Some(at + consumed - 1);
                        at += consumed;
                    }
                }
                if output < start + woven.len() {
                    let newline =
                        previous_newline.expect("trailing output break has a source break");
                    file.segments.push(LiteralSegment {
                        output: (output, output + 1),
                        source: (newline, newline + 1),
                    });
                }
                file.content.push_str(&woven);
            }
        }
        files.push(file);
    }
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    // Guarantee: docs/guarantees/embedding/literal-files-use-native-bytes.md
    #[test]
    fn materializes_unicode_crlf_and_rebound_prefix_with_exact_lineage() {
        let source = "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\">\n🦀\n  <h:file path=\"a.js\">\r\n  var x = 'é';\r\n  </h:file>\n</h:doc>";
        let file = literal_files(source).unwrap().remove(0);
        assert_eq!(file.content, "var x = 'é';\n");
        for s in file.segments {
            assert_eq!(
                &file.content[s.output.0..s.output.1],
                &source[s.source.0..s.source.1]
            );
        }
    }
    #[test]
    fn dependent_files_are_refused() {
        assert!(
            literal_files("<hick:file path=\"a\"><hick:paste select=\"#x\"/></hick:file>").is_err()
        );
        assert!(
            literal_files("<hick:when test=\"x\"><hick:file path=\"a\">x</hick:file></hick:when>")
                .is_err()
        );
    }
}
