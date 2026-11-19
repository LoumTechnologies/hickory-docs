//! Source code boundaries from the canonical Hickory/Markdown parser.
use crate::{ByteOrigin, HickNode, HickTag};
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceCodeRegion {
    pub from: usize,
    pub to: usize,
    pub kind: &'static str,
    pub language: Option<String>,
    pub path: Option<String>,
    pub fragment: Option<String>,
    pub dynamic: bool,
}
impl SourceCodeRegion {
    /// Return a complete static ordinary fence's literal body and byte provenance.
    pub fn literal_fence(
        &self,
        source: &str,
        document: &str,
        max_origins: usize,
    ) -> Result<(String, Vec<ByteOrigin>), String> {
        if self.kind != "fence" || self.dynamic || self.path.is_some() {
            return Err("Choose a complete static code fence without an output path".into());
        }
        let prefix = source.get(..self.from).ok_or("Invalid canonical fence boundary")?;
        let opening = prefix.strip_suffix('\n').unwrap_or(prefix);
        let opening = opening.strip_suffix('\r').unwrap_or(opening);
        let opening = opening.rsplit('\n').next().unwrap_or("").trim_start_matches('\u{feff}');
        let indent = opening.bytes().take_while(|byte| *byte == b' ').count();
        if indent > 3 {
            return Err("Invalid canonical fence indentation".into());
        }
        let body = source.get(self.from..self.to).ok_or("Invalid canonical fence body")?;
        let mut content = String::with_capacity(body.len());
        let mut origins = Vec::new();
        let mut offset = self.from;
        for line in body.split_inclusive('\n') {
            let skipped = line.bytes().take(indent).take_while(|byte| *byte == b' ').count();
            let literal = &line[skipped..];
            if !literal.is_empty() {
                if origins.len() == max_origins {
                    return Err("Literal fence provenance exceeds its limit".into());
                }
                let start = content.len();
                content.push_str(literal);
                origins.push(ByteOrigin {
                    output: (start, content.len()),
                    document: document.into(),
                    source: (offset + skipped, offset + line.len()),
                });
            }
            offset += line.len();
        }
        Ok((content, origins))
    }
}
fn body(tag: &HickTag, shift: usize) -> Option<(usize, usize)> {
    Some((
        tag.source_span?.end.checked_add(shift)?,
        tag.close_span?.start.checked_add(shift)?,
    ))
}
/// Parse facts only. These spans do not grant output or edit ownership.
pub fn source_code_regions(source: &str) -> Result<Vec<SourceCodeRegion>, String> {
    let doc = crate::parse(source).map_err(|error| error.to_string())?;
    let shift = if source.starts_with('\u{feff}') { 3 } else { 0 };
    let mut regions = Vec::new();
    fn walk(
        nodes: &[HickNode],
        source: &str,
        shift: usize,
        dynamic: bool,
        inside_code: bool,
        regions: &mut Vec<SourceCodeRegion>,
    ) -> Result<(), String> {
        for node in nodes {
            match node {
                HickNode::Tag(tag) => {
                    let plain_patch = !inside_code && tag.name == "patch" && tag.attributes.is_empty()
                        && tag.source_span.and_then(|span| {
                            source.get(span.start.checked_add(shift)?..span.end.checked_add(shift)?)
                        }).is_some_and(|opening| {
                            let opening = opening.trim_start_matches(' ');
                            opening.starts_with("```") || opening.starts_with("~~~")
                        });
                    let conditional = dynamic
                        || matches!(
                            tag.name.as_str(),
                            "when" | "for" | "foreach" | "agent" | "patch" | "entry"
                        );
                    let kind = match tag.name.as_str() {
                        "patch" if plain_patch => Some("fence"),
                        "file" => Some("file"),
                        "exec" => Some("cell"),
                        "copy" => Some("fragment"),
                        _ => None,
                    };
                    if let (Some(kind), Some((from, to))) = (kind, body(tag, shift)) {
                        if from <= to && source.get(from..to).is_some() {
                            regions.push(SourceCodeRegion {
                                from,
                                to,
                                kind,
                                language: if plain_patch { Some("patch".into()) } else { tag.get_attribute("language").map(str::to_owned) },
                                path: tag.get_attribute("path").map(str::to_owned),
                                fragment: tag.get_attribute("id").map(str::to_owned),
                                dynamic: if plain_patch { dynamic } else { conditional },
                            });
                        }
                    }
                    walk(
                        &tag.children,
                        source,
                        shift,
                        conditional,
                        inside_code || kind.is_some(),
                        regions,
                    )?;
                }
                HickNode::Text(_, Some(span)) if !inside_code => {
                    if span.file_id.is_some() {
                        return Err(
                            "A spliced source span requires its captured project source identity"
                                .into(),
                        );
                    }
                    let start = span
                        .start
                        .checked_add(shift)
                        .ok_or("Source coordinate overflow")?;
                    let end = span
                        .end
                        .checked_add(shift)
                        .ok_or("Source coordinate overflow")?;
                    let text = source
                        .get(start..end)
                        .ok_or("Source text span is unavailable")?;
                    let mut offset = 0;
                    let mut line = span.start_line;
                    while let Some(relative) = crate::fence::fence_at_or_after(&text[offset..]) {
                        if regions.len() >= 10000 {
                            return Err("Too many embedded source regions".into());
                        }
                        line += text[offset..offset + relative]
                            .bytes()
                            .filter(|byte| *byte == b'\n')
                            .count();
                        offset += relative;
                        let opening = start + offset;
                        let tail = &text[offset..];
                        let Some(fence) = crate::fence::parse_fence(tail, line) else {
                            let content = tail.find('\n').map_or(tail.len(), |at| at + 1);
                            regions.push(SourceCodeRegion {
                                from: opening + content,
                                to: end,
                                kind: "incomplete-fence",
                                language: None,
                                path: None,
                                fragment: None,
                                dynamic: true,
                            });
                            break;
                        };
                        let first = tail[..fence.open_end].trim_start();
                        let marker = first.chars().next().ok_or("Missing fence marker")?;
                        let info = first.trim_start_matches(marker).trim();
                        let hint = info
                            .split_whitespace()
                            .next()
                            .filter(|value| !value.is_empty() && !value.contains('='));
                        let language = fence
                            .tag
                            .as_ref()
                            .and_then(|tag| tag.get_attribute("language"))
                            .or(hint)
                            .map(str::to_owned);
                        let path = fence
                            .tag
                            .as_ref()
                            .and_then(|tag| tag.get_attribute("path"))
                            .map(str::to_owned);
                        regions.push(SourceCodeRegion {
                            from: opening + fence.open_end,
                            to: opening + fence.close_start,
                            kind: "fence",
                            language,
                            path,
                            fragment: None,
                            dynamic,
                        });
                        line += tail[..fence.end]
                            .bytes()
                            .filter(|byte| *byte == b'\n')
                            .count();
                        offset += fence.end;
                        if offset >= text.len() {
                            break;
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
    walk(&doc.nodes, source, shift, false, false, &mut regions)?;
    regions.sort_by_key(|region| (region.from, region.to));
    if regions.len() > 10000 {
        return Err("Too many embedded source regions".into());
    }
    Ok(regions)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ordinary_markdown_patch_fences_keep_literal_authority_separate_from_constructors() {
        let source = "Prose 😀\r\n  ~~~patch\r\n  --- a/x\r\n  +++ b/x\r\n  ~~~\r\n```patch id=\"declared\"\nnot a literal review\n```\n";
        let regions = source_code_regions(source).unwrap();
        assert_eq!(regions.len(), 1);
        let fence = &regions[0];
        assert_eq!(fence.language.as_deref(), Some("patch"));
        let (diff, origins) = fence.literal_fence(source, "message", 2).unwrap();
        assert_eq!(diff, "--- a/x\r\n+++ b/x\r\n");
        assert_eq!(origins.len(), 2);
        let conditional = "\x3ch:doc xmlns:h=\"http://www.hickorydocs.com/1.0\">\x3ch:when test=\"false\">\n```patch\n--- a/x\n+++ b/x\n```\n\x3c/h:when>\x3c/h:doc>";
        let regions = source_code_regions(conditional).unwrap();
        assert!(regions[0].dynamic);
        assert!(regions[0].literal_fence(conditional, "message", 20).is_err());
    }
    #[test]
    fn literal_fences_and_nested_file_bodies_keep_exact_authored_bounds() {
        let source = "Prose\r\n```python\r\nold\r\n```\r\n\x3ch:doc xmlns:h=\"http://www.hickorydocs.com/1.0\">\x3ch:copy id=\"nested\">\x3ch:file path=\"a.ts\">value\x3c/h:file>\x3c/h:copy>\x3c/h:doc>";
        let regions = source_code_regions(source).unwrap();
        let fenced = regions.iter().find(|value| value.kind == "fence").unwrap();
        assert_eq!(&source[fenced.from..fenced.to], "old\r\n");
        assert_eq!(fenced.language.as_deref(), Some("python"));
        let file = regions.iter().find(|value| value.kind == "file").unwrap();
        assert_eq!(&source[file.from..file.to], "value");
        assert_eq!(file.path.as_deref(), Some("a.ts"));
    }
    #[test]
    fn empty_and_dynamic_bodies_and_incomplete_fences_remain_distinct() {
        let source = "\x3ch:doc xmlns:h=\"http://www.hickorydocs.com/1.0\">\x3ch:file path=\"empty.py\">\x3c/h:file>\x3ch:when test=\"fact\">\x3ch:file path=\"maybe.py\">old\x3c/h:file>\x3c/h:when>\x3c/h:doc>";
        let regions = source_code_regions(source).unwrap();
        let empty = regions
            .iter()
            .find(|value| value.path.as_deref() == Some("empty.py"))
            .unwrap();
        assert_eq!(empty.from, empty.to);
        assert!(!empty.dynamic);
        assert!(
            regions
                .iter()
                .find(|value| value.path.as_deref() == Some("maybe.py"))
                .unwrap()
                .dynamic
        );
        assert_eq!(
            source_code_regions("```python\nold").unwrap()[0].kind,
            "incomplete-fence"
        );
    }
}
