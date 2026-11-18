//! Source code boundaries from the canonical Hickory/Markdown parser.
use crate::{HickNode, HickTag};
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
                    let conditional = dynamic
                        || matches!(
                            tag.name.as_str(),
                            "when" | "for" | "foreach" | "agent" | "patch" | "entry"
                        );
                    let kind = match tag.name.as_str() {
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
                                language: tag.get_attribute("language").map(str::to_owned),
                                path: tag.get_attribute("path").map(str::to_owned),
                                fragment: tag.get_attribute("id").map(str::to_owned),
                                dynamic: conditional,
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
