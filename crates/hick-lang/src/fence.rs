use crate::HickTag;

/// A complete Markdown fenced block, measured from its opening fence.
///
/// The parser deliberately understands only enough Markdown to establish the
/// raw boundary.  Markdown itself remains text; the `tag` is present only for
/// the three fence forms Hick owns.
pub(crate) struct Fence {
    pub(crate) open_end: usize,
    pub(crate) close_start: usize,
    pub(crate) end: usize,
    pub(crate) line: usize,
    pub(crate) column: usize,
    pub(crate) close_line: usize,
    pub(crate) close_column: usize,
    pub(crate) tag: Option<HickTag>,
}

/// Return the first line-start fence in `text`. Markdown permits up to three
/// spaces before a fence; keeping that rule means an indented code example is
/// never mistaken for a cell.
pub(crate) fn fence_at_or_after(text: &str) -> Option<usize> {
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        let indent = line.bytes().take_while(|byte| *byte == b' ').count();
        let rest = &line[indent..];
        if indent <= 3 && (rest.starts_with("```") || rest.starts_with("~~~")) {
            return Some(offset);
        }
        offset += line.len();
    }
    None
}

pub(crate) fn parse_fence(source: &str, line: usize) -> Option<Fence> {
    let first_end = source.find('\n').map(|n| n + 1).unwrap_or(source.len());
    let first = &source[..first_end];
    let indent = first.bytes().take_while(|byte| *byte == b' ').count();
    if indent > 3 {
        return None;
    }
    let rest = &first[indent..];
    let marker = rest.as_bytes().first().copied()?;
    if marker != b'`' && marker != b'~' {
        return None;
    }
    let width = rest.bytes().take_while(|byte| *byte == marker).count();
    if width < 3 {
        return None;
    }
    let info = rest[width..].trim();

    let mut close_start = first_end;
    let mut close_line = line + 1;
    let mut found_close = None;
    for candidate in source[first_end..].split_inclusive('\n') {
        let candidate_indent = candidate.bytes().take_while(|byte| *byte == b' ').count();
        let after_indent = &candidate[candidate_indent..];
        let candidate_width = after_indent
            .bytes()
            .take_while(|byte| *byte == marker)
            .count();
        if candidate_indent <= 3
            && candidate_width >= width
            && after_indent[candidate_width..].trim().is_empty()
        {
            found_close = Some(close_start + candidate.len());
            break;
        }
        close_start += candidate.len();
        close_line += 1;
    }
    let end = found_close?;
    let tag = fence_tag(info, line, indent);
    Some(Fence {
        open_end: first_end,
        close_start,
        end,
        line,
        column: indent,
        close_line,
        close_column: 0,
        tag,
    })
}

/// Parse the intentionally small, XML-shaped attribute tail in a fence info
/// string. The language token stays first for ordinary Markdown renderers;
/// Hick keeps it as `language=` on its normalised element.
fn fence_tag(info: &str, line: usize, column: usize) -> Option<HickTag> {
    let mut rest = info.trim();
    let language_end = rest.find(char::is_whitespace).unwrap_or(rest.len());
    let first = &rest[..language_end];
    let mut attributes = Vec::new();
    let language = if first.starts_with("output-for=") {
        rest = &rest[language_end..];
        String::new()
    } else {
        rest = &rest[language_end..];
        first.to_string()
    };
    while !rest.trim_start().is_empty() {
        rest = rest.trim_start();
        let name_end = rest.find(|c: char| c == '=' || c.is_whitespace())?;
        let name = &rest[..name_end];
        rest = &rest[name_end..];
        if !rest.starts_with('=') {
            return None;
        }
        rest = &rest[1..];
        let quote = rest.chars().next()?;
        if quote != '\"' && quote != '\'' {
            return None;
        }
        rest = &rest[quote.len_utf8()..];
        let value_end = rest.find(quote)?;
        attributes.push((name.to_string(), rest[..value_end].to_string()));
        rest = &rest[value_end + quote.len_utf8()..];
    }
    if first.starts_with("output-for=") {
        // The first attribute has no preceding language word.
        let value = first.strip_prefix("output-for=")?;
        let quote = value.chars().next()?;
        if (quote != '\"' && quote != '\'') || !value.ends_with(quote) {
            return None;
        }
        attributes.insert(
            0,
            (
                "output-for".to_string(),
                value[1..value.len() - 1].to_string(),
            ),
        );
    } else if !language.is_empty() {
        attributes.push(("language".to_string(), language));
    }
    let has = |name: &str| attributes.iter().any(|(key, _)| key == name);
    let name = if has("output-for") {
        "output"
    } else if has("path") {
        "file"
    } else if has("container") || has("image") || has("mount") || has("show") {
        "exec"
    } else {
        return None;
    };
    Some(HickTag {
        name: name.to_string(),
        attributes,
        children: Vec::new(),
        self_closing: false,
        source_line: line,
        source_column: column,
        source_span: None,
        close_span: None,
    })
}
