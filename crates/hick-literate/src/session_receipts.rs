//! Receipts for editor snapshots and paragraph-scoped answer citations.
use crate::session_elements::{SessionFacts, mentions};
use hick_blocks::{Family, Link, LinkTarget, span_of};
use hick_lang::{HickDocument, HickNode, HickTag};

pub(crate) fn citations(tag: &HickTag, facts: &SessionFacts) -> Vec<Link> {
    let mut links = Vec::new();
    for node in &tag.children {
        match node {
            HickNode::Text(text, Some(span)) => {
                let mut offset = span.start;
                for paragraph in text.split("\n\n") {
                    let body = paragraph.trim();
                    let start = offset + paragraph.len() - paragraph.trim_start().len();
                    for (path, lines) in mentions(body) {
                        links.push(Link {
                            family: Family::Declared,
                            span: (start, start + body.len()),
                            title: format!("Declared — this paragraph cites {}{}. The model's assertion, not a verified derivation.", facts.relative(&path), lines.map(|(a,b)| format!(" lines {a}–{b}")).unwrap_or_default()),
                            to: LinkTarget { path: facts.relative(&path), lines },
                        });
                    }
                    offset += paragraph.len() + 2;
                }
            }
            HickNode::Tag(child)
                if child.name == "input" && child.get_attribute("name").is_none() =>
            {
                links.extend(citations(child, facts))
            }
            _ => {}
        }
    }
    links
}

pub(crate) fn editor_context(tag: &HickTag, facts: &SessionFacts) -> Vec<Link> {
    // Native recordings store a JSON string; ACP stores {turn, context}.
    // Both contain the same describe() prose followed by a JSON buffer snapshot.
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&tag.text_content()) else {
        return vec![];
    };
    let Some(text) = value.as_str().or_else(|| value["context"].as_str()) else {
        return vec![];
    };
    // ACP may append tool-use instructions after describe()'s snapshot JSON.
    let Some(snapshot) = text.lines().rev().find_map(|line| {
        serde_json::from_str::<serde_json::Value>(line)
            .ok()
            .filter(|value| value["buffers"].is_array())
    }) else {
        return vec![];
    };
    snapshot["buffers"].as_array().into_iter().flatten().filter_map(|buffer| {
        let path = buffer["path"].as_str().filter(|path| !path.is_empty())
            .map(str::to_string).or_else(|| buffer["provenance_uri"].as_str()
                .filter(|uri| uri.starts_with("hickory-buffer://")).map(str::to_string))
            .or_else(|| buffer["id"].as_str()
                .filter(|id| !id.is_empty()).map(|id| format!("hickory-buffer://{id}")))?;
        let content = buffer["content"].as_str()?;
        let lines = Some((1, content.lines().count().max(1)));
        Some(Link {
            family: Family::Context,
            span: span_of(tag),
            title: format!("Context — the editor snapshot of {} was supplied for this turn, including unsaved text. This records model input, not support for a claim; the current buffer may have changed.", buffer["name"].as_str().unwrap_or(&path)),
            to: LinkTarget { path: facts.relative(&path), lines },
        })
    }).collect()
}

/// Show supplied editor context beside the answer even when its record is folded.
/// These links explain available input, independently of model-authored citations.
pub(crate) fn answer_context(doc: &HickDocument, facts: &SessionFacts) -> Vec<Link> {
    fn prose_spans(tag: &HickTag) -> Vec<(usize, usize)> {
        let mut spans = Vec::new();
        for node in &tag.children {
            match node {
                HickNode::Text(text, Some(span)) => {
                    let mut offset = span.start;
                    for paragraph in text.split("\n\n") {
                        let body = paragraph.trim();
                        let start = offset + paragraph.len() - paragraph.trim_start().len();
                        if !body.is_empty() {
                            spans.push((start, start + body.len()));
                        }
                        offset += paragraph.len() + 2;
                    }
                }
                HickNode::Tag(child)
                    if child.name == "input" && child.get_attribute("name").is_none() =>
                {
                    spans.extend(prose_spans(child))
                }
                _ => {}
            }
        }
        spans
    }
    let mut context = Vec::new();
    let mut links = Vec::new();
    for tag in doc.all_tags() {
        match tag.name.as_str() {
            "user" => context.clear(),
            "context" if tag.get_attribute("kind") == Some("editor-buffers") => {
                context = editor_context(tag, facts)
            }
            "assistant" => {
                for span in prose_spans(tag) {
                    for input in &context {
                        let mut link = input.clone();
                        link.span = span;
                        links.push(link);
                    }
                }
            }
            _ => {}
        }
    }
    links
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session_elements::session_links;
    use std::path::Path;

    // Guarantee: docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md
    #[test]
    fn citations_anchor_to_each_paragraph_and_keep_unicode_byte_offsets() {
        let source = "<hick:session><hick:assistant><hick:input>\nFirst 📝 claim [source](note.md#L3-L5).\n\nSecond claim note.md:9-12.\n</hick:input></hick:assistant></hick:session>";
        let links = session_links(source, None);
        assert_eq!(links.len(), 2);
        assert_eq!(
            &source[links[0].link.span.0..links[0].link.span.1],
            "First 📝 claim [source](note.md#L3-L5)."
        );
        assert_eq!(
            &source[links[1].link.span.0..links[1].link.span.1],
            "Second claim note.md:9-12."
        );
        assert_eq!(links[0].link.to.lines, Some((3, 5)));
        assert_eq!(links[1].link.to.lines, Some((9, 12)));
        assert!(links.iter().all(|l| l.link.family == Family::Declared));
    }

    // Guarantee: docs/guarantees/agent/the-agent-sees-the-open-editors.md
    #[test]
    fn both_recorders_link_supplied_buffers_using_explicit_untitled_identities() {
        let text = format!(
            "Editor snapshot\n{}\nUse read_buffer for requested edits.",
            serde_json::json!({"buffers":[{"path":"/work/note.md","content":"unsaved\ntext\n"},{"id":"u1","provenance_uri":"hickory-buffer://window/u1","path":null,"name":"Untitled","content":"draft"}]})
        );
        for value in [
            serde_json::json!(text),
            serde_json::json!({"turn":"a","context":text}),
        ] {
            let source = format!(
                "<hick:session><hick:context kind=\"editor-buffers\">{value}</hick:context></hick:session>"
            );
            let links = session_links(&source, Some(Path::new("/work")));
            assert_eq!(links.len(), 2);
            assert_eq!(links[0].link.family, Family::Context);
            assert_eq!(links[0].link.to.path, "note.md");
            assert_eq!(links[0].link.to.lines, Some((1, 2)));
            assert!(links[0].link.title.contains("unsaved"));
            assert_eq!(links[1].link.to.path, "hickory-buffer://window/u1");
        }
    }
    // Guarantee: docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md
    #[test]
    fn an_uncited_untitled_answer_keeps_input_context_and_cited_paragraphs_resolve() {
        let snapshot = serde_json::json!({"turn":"a","context":format!("Editor snapshot\n{}\nUse read_buffer for requested edits.",
            serde_json::json!({"buffers":[{"id":"u1","name":"Untitled","path":null,"content":"Short.\n\nA longer paragraph. 📝"}]}))});
        let source = format!(
            "<hick:session><hick:user turn=\"a\">Which paragraph is longest?</hick:user>\n<hick:context kind=\"editor-buffers\">{snapshot}</hick:context>\n<hick:assistant>The second paragraph has the most characters.\n\nIt is [here](hickory-buffer://u1#L3-L3).</hick:assistant>\n<hick:user turn=\"b\">Unrelated</hick:user><hick:assistant>No inherited input.</hick:assistant></hick:session>"
        );
        let links = session_links(&source, None);
        let answer_context: Vec<_> = links
            .iter()
            .filter(|l| {
                l.link.family == Family::Context
                    && source[l.link.span.0..l.link.span.1].starts_with("The second paragraph")
            })
            .collect();
        assert_eq!(answer_context.len(), 1);
        assert_eq!(answer_context[0].link.to.path, "hickory-buffer://u1");
        assert_eq!(answer_context[0].link.to.lines, Some((1, 3)));
        let cited = links
            .iter()
            .find(|l| l.link.family == Family::Declared)
            .unwrap();
        assert_eq!(cited.link.to.path, "hickory-buffer://u1");
        assert_eq!(cited.link.to.lines, Some((3, 3)));
        assert!(
            links
                .iter()
                .all(|l| !source[l.link.span.0..l.link.span.1].starts_with("No inherited input"))
        );
    }
}
