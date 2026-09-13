use super::*;
use crate::fence::{fence_at_or_after, parse_fence};

pub(crate) struct Parser<'a> {
    input: &'a str,
    pos: usize,
    line: usize,
    /// 0-based column position in current line.
    col: usize,
    /// The detected namespace prefix (e.g. `"hick"`, `"h"`).
    prefix: String,
    /// Opening tag marker, e.g. `"<hick:"`.
    open_marker: String,
    /// Closing tag marker, e.g. `"</hick:"`.
    close_marker: String,
    /// Recover from malformed input instead of failing. See [`parse_lenient`].
    lenient: bool,
    /// The first error a lenient parse recovered from, or `None`.
    pub(crate) recovered: Option<ParseError>,
}

/// A leading UTF-8 byte-order mark, which is an encoding signature rather than
/// content and must not be parsed as text.
///
/// `str::trim_start` does NOT remove it: `U+FEFF` is a format character, not
/// `White_Space`. So a document saved by Notepad, by PowerShell's
/// `Set-Content -Encoding UTF8`, or by any of the Windows editors that write
/// one, failed `opens_root` on its first byte and was parsed as a BARE
/// document — its `<hick:doc>` wrapper unrecognised, its cells never run, its
/// weave a single line of leftover XML declaration. `hick run` reported "1
/// file(s) written" and exited 0 while doing essentially nothing, which is the
/// worst way for this to fail: a Windows user gets no error to search for.
/// Found by running the real binary on a real Windows machine, 2026-08-20.
const BOM: &str = "\u{feff}";

/// Appended to a tag/attribute parse failure that a plausible-looking piece
/// of prose can trigger — e.g. a sentence or code span that quotes hick's own
/// syntax, `` `<hick:paste select="…" />` ``. The no-escaping invariant means
/// there is no such thing as "just talking about" a tag: any `<prefix:name`
/// found anywhere is parsed as markup. Without this, the error names only the
/// unexpected token, never why prose that reads as plainly not-code became a
/// syntax error.
const NO_ESCAPING_HINT: &str = "hick has no escaping — text that looks like <prefix:name> is \
    always parsed as a tag, even inside a sentence or a code span. To write about hick's own \
    syntax as prose, see the `h:`-prefix convention in docs/specs/freeform/bare-documents.md.";

impl<'a> Parser<'a> {
    pub(crate) fn new(input: &'a str) -> Self {
        let input = input.strip_prefix(BOM).unwrap_or(input);
        let prefix = resolve_prefix(input);
        let open_marker = format!("<{prefix}:");
        let close_marker = format!("</{prefix}:");
        Self {
            input,
            pos: 0,
            line: 1,
            col: 0,
            prefix,
            open_marker,
            close_marker,
            lenient: false,
            recovered: None,
        }
    }

    pub(crate) fn new_lenient(input: &'a str) -> Self {
        Self {
            lenient: true,
            ..Self::new(input)
        }
    }

    /// Remember the first error a lenient parse recovered from.
    fn recover(&mut self, error: ParseError) {
        if self.recovered.is_none() {
            self.recovered = Some(error);
        }
    }

    fn checkpoint(&self) -> (usize, usize, usize) {
        (self.pos, self.line, self.col)
    }

    fn restore(&mut self, at: (usize, usize, usize)) {
        (self.pos, self.line, self.col) = at;
    }

    fn remaining(&self) -> &'a str {
        &self.input[self.pos..]
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    /// Advance past the XML declaration if present.
    fn skip_xml_declaration(&mut self) {
        let rem = self.remaining().trim_start();
        let skipped_ws = self.remaining().len() - rem.len();
        if rem.starts_with("<?xml")
            && let Some(end) = rem.find("?>")
        {
            let total = skipped_ws + end + 2;
            self.advance(total);
        }
    }

    /// If the cursor is at `<!--`, skip past the matching `-->`.
    ///
    /// Returns `true` if a comment was skipped.
    fn skip_comment(&mut self) -> Result<bool, ParseError> {
        let rem = self.remaining();
        if !rem.starts_with("<!--") {
            return Ok(false);
        }
        let comment_line = self.line;
        match rem[4..].find("-->") {
            Some(end) => {
                self.advance(4 + end + 3);
                Ok(true)
            }
            None => Err(ParseError::UnclosedComment { line: comment_line }),
        }
    }

    /// Advance `n` bytes, tracking line and column numbers.
    fn advance(&mut self, n: usize) {
        let slice = &self.input[self.pos..self.pos + n];
        for ch in slice.chars() {
            if ch == '\n' {
                self.line += 1;
                self.col = 0;
            } else {
                self.col += 1;
            }
        }
        self.pos += n;
    }

    /// Parse the full document, wrapped or bare.
    pub(crate) fn parse_document(&mut self) -> Result<HickDocument, ParseError> {
        if !opens_root(self.input, &self.prefix, "doc") {
            return self.parse_bare_document();
        }

        self.skip_xml_declaration();

        // Find the root <PREFIX:doc ...> tag
        let doc_marker = format!("<{}:doc", self.prefix);
        let root_start = self
            .remaining()
            .find(&doc_marker)
            .ok_or(ParseError::MissingRoot)?;
        self.advance(root_start);

        // Parse the opening tag attributes and consume `>`
        let tag = match self.parse_open_tag() {
            Ok(tag) if tag.name == "doc" => tag,
            Ok(_) => return Err(ParseError::MissingRoot),
            // A root tag that does not parse is not a root: read the whole
            // file as a bare document, with the tag's bytes as text.
            Err(error) if self.lenient => {
                self.recover(error);
                self.restore((0, 1, 0));
                return self.parse_bare_document();
            }
            Err(error) => return Err(error),
        };

        // Extract weave path from the doc tag attributes
        let weave_path = tag.get_attribute("weave").map(|s| s.to_string());
        let volatile = tag.get_attribute("volatile") == Some("true");

        // Parse children until </PREFIX:doc>
        let (nodes, close_span) = self.parse_children(Some("doc"), tag.source_line)?;

        Ok(HickDocument {
            nodes,
            source: self.input.to_string(),
            prefix: self.prefix.clone(),
            weave_path,
            volatile,
            frontmatter: None,
            span_files: Vec::new(),
            root_tag: Some(HickTag { close_span, ..tag }),
        })
    }

    /// Parse a document with no root element: the whole file is the body.
    ///
    /// See `docs/specs/freeform/bare-documents.md`. There is no root tag to
    /// open and therefore none to close, so the child parse ends at EOF rather
    /// than at a closing marker. A stray closing tag is still an error — the
    /// ambiguity bare documents introduce is only about whether a root was
    /// ever opened.
    ///
    /// The frontmatter block is **not** consumed: its bytes stay in the node
    /// stream so that it weaves through verbatim and carries spans.
    fn parse_bare_document(&mut self) -> Result<HickDocument, ParseError> {
        let frontmatter = split_frontmatter(self.input).map(|(fm, _)| fm);

        let weave_path = frontmatter
            .as_ref()
            .and_then(|fm| fm.get("weave"))
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let volatile = frontmatter.as_ref().and_then(|fm| fm.get("volatile")) == Some("true");

        let (nodes, _) = self.parse_children(None, 1)?;

        Ok(HickDocument {
            nodes,
            source: self.input.to_string(),
            prefix: self.prefix.clone(),
            weave_path,
            volatile,
            frontmatter,
            span_files: Vec::new(),
            root_tag: None,
        })
    }

    /// Parse a session document whose root element is `<PREFIX:session>`.
    pub(crate) fn parse_session_document(&mut self) -> Result<SessionDocument, ParseError> {
        self.skip_xml_declaration();

        let session_marker = format!("<{}:session", self.prefix);
        let root_start = self
            .remaining()
            .find(&session_marker)
            .ok_or(ParseError::MissingRoot)?;
        self.advance(root_start);

        let tag = self.parse_open_tag()?;
        if tag.name != "session" {
            return Err(ParseError::MissingRoot);
        }

        let start_time = tag.get_attribute("start").map(|s| s.to_string());

        let (raw_nodes, _) = self.parse_children(Some("session"), tag.source_line)?;
        let nodes = extract_session_nodes(&raw_nodes);

        Ok(SessionDocument {
            nodes,
            source: self.input.to_string(),
            prefix: self.prefix.clone(),
            start_time,
        })
    }

    /// Parse children until the matching closing tag is found.
    /// Elements whose content is captured verbatim (raw text up to the next
    /// matching close marker) instead of being parsed for nested tags.
    ///
    /// These carry the agent session vocabulary's payloads: `hick:input`
    /// (multi-line tool-call payloads, e.g. replacement text that may quote
    /// unbalanced hick fragments) and `hick:tool-result` (tool observations
    /// that may excerpt arbitrary document slices). The only substring such
    /// content cannot contain is its own literal close tag.
    /// `close_name` is the element whose closing tag ends this run of
    /// children, or `None` for the body of a bare document, which ends at EOF.
    fn parse_children(
        &mut self,
        close_name: Option<&str>,
        open_line: usize,
    ) -> Result<(Vec<HickNode>, Option<SourceSpan>), ParseError> {
        let mut nodes = Vec::new();
        let mut text_start = self.pos;
        let mut text_start_line = self.line;
        let mut text_start_col = self.col;

        loop {
            // Look for the next opening marker, closing marker, or comment
            let rem = self.remaining();
            let next_open = rem.find(self.open_marker.as_str());
            let next_close = rem.find(self.close_marker.as_str());
            let next_comment = rem.find("<!--");
            let next_fence = fence_at_or_after(rem);

            // Determine the nearest marker
            let nearest = [next_open, next_close, next_comment, next_fence]
                .into_iter()
                .flatten()
                .min();

            let Some(offset) = nearest else {
                // No more markers. For a bare document's body that is the
                // end; for an element it means nobody closed it.
                if let Some(close_name) = close_name {
                    let error = ParseError::UnclosedTag {
                        prefix: self.prefix.clone(),
                        name: close_name.to_string(),
                        line: open_line,
                    };
                    if !self.lenient {
                        return Err(error);
                    }
                    self.recover(error);
                }
                let end = self.input.len();
                if end > text_start {
                    let span = SourceSpan::new(text_start, end, text_start_line, text_start_col);
                    nodes.push(HickNode::Text(
                        self.input[text_start..end].to_string(),
                        Some(span),
                    ));
                }
                self.advance(end - self.pos);
                return Ok((nodes, None));
            };

            let abs = self.pos + offset;
            let before_marker = self.checkpoint();

            // Markdown fences are a raw-text boundary.  In particular, a
            // code example containing `<hick:exec>` is code, not a document
            // element.  A fence opts into Hick only through one of the
            // ownership attributes in its info string; that small subset is
            // normalised into the same tag shape as the explicit spelling.
            if next_fence == Some(offset) {
                let Some(fence) = parse_fence(&self.input[abs..], self.line_at(abs)) else {
                    // Markdown treats an unclosed fence as code through EOF.
                    // Do likewise rather than letting a hick-looking string in
                    // that code escape its fence and become an element.
                    let end = self.input.len();
                    self.advance(end - self.pos);
                    nodes.push(HickNode::Text(
                        self.input[text_start..end].to_string(),
                        Some(SourceSpan::new(
                            text_start,
                            end,
                            text_start_line,
                            text_start_col,
                        )),
                    ));
                    return Ok((nodes, None));
                };
                if abs > text_start {
                    let span = SourceSpan::new(text_start, abs, text_start_line, text_start_col);
                    nodes.push(HickNode::Text(
                        self.input[text_start..abs].to_string(),
                        Some(span),
                    ));
                }
                // `fence.end` is relative to the fence opening while the
                // parser cursor is still before the prose leading up to it.
                // Advance over both ranges so a following explicit element is
                // parsed normally rather than becoming part of the fence.
                self.advance(offset + fence.end);
                match fence.tag {
                    Some(mut tag) => {
                        tag.source_column = fence.column;
                        tag.source_span = Some(SourceSpan::new(
                            abs,
                            abs + fence.open_end,
                            fence.line,
                            fence.column,
                        ));
                        tag.close_span = Some(SourceSpan::new(
                            abs + fence.close_start,
                            abs + fence.end,
                            fence.close_line,
                            fence.close_column,
                        ));
                        tag.children.push(HickNode::Text(
                            self.input[abs + fence.open_end..abs + fence.close_start].to_string(),
                            Some(SourceSpan::new(
                                abs + fence.open_end,
                                abs + fence.close_start,
                                fence.line + 1,
                                0,
                            )),
                        ));
                        nodes.push(HickNode::Tag(tag));
                    }
                    None => nodes.push(HickNode::Text(
                        self.input[abs..abs + fence.end].to_string(),
                        Some(SourceSpan::new(
                            abs,
                            abs + fence.end,
                            fence.line,
                            fence.column,
                        )),
                    )),
                }
                text_start = self.pos;
                text_start_line = self.line;
                text_start_col = self.col;
                continue;
            }

            // Is this a comment?
            if self.input[abs..].starts_with("<!--") {
                self.advance(offset);
                match self.skip_comment() {
                    Ok(_) => {}
                    // An unclosed comment is text from its `<` onward.
                    Err(error) if self.lenient => {
                        self.recover(error);
                        self.restore(before_marker);
                        self.advance(offset + 1);
                        continue;
                    }
                    Err(error) => return Err(error),
                }
                // Flush text before the comment
                if abs > text_start {
                    let span = SourceSpan::new(text_start, abs, text_start_line, text_start_col);
                    nodes.push(HickNode::Text(
                        self.input[text_start..abs].to_string(),
                        Some(span),
                    ));
                }
                text_start = self.pos;
                text_start_line = self.line;
                text_start_col = self.col;
                continue;
            }

            // Is this a closing tag?
            if self.input[abs..].starts_with(&self.close_marker) {
                self.advance(offset);
                let (name, line, span) = self.parse_close_tag();
                // In a bare document `close_name` is `None`, so every
                // closing tag is unexpected — there is no root that
                // could have opened it.
                if close_name == Some(name.as_str()) {
                    // Flush text before this closing tag
                    if abs > text_start {
                        let span =
                            SourceSpan::new(text_start, abs, text_start_line, text_start_col);
                        nodes.push(HickNode::Text(
                            self.input[text_start..abs].to_string(),
                            Some(span),
                        ));
                    }
                    return Ok((nodes, Some(span)));
                }
                let error = ParseError::UnexpectedClose {
                    prefix: self.prefix.clone(),
                    name,
                    line,
                };
                if !self.lenient {
                    return Err(error);
                }
                // A closer that matches nothing is text: the run of text
                // that was open before it simply continues through it.
                self.recover(error);
                continue;
            }

            // It's an opening tag. Parse it before flushing the text in
            // front of it, so a tag that does not parse can be left as text.
            self.advance(offset);
            let tag = match self.parse_open_tag() {
                Ok(tag) => tag,
                Err(error) if self.lenient => {
                    self.recover(error);
                    self.restore(before_marker);
                    self.advance(offset + 1);
                    continue;
                }
                Err(error) => return Err(error),
            };

            // Flush text before this tag
            if abs > text_start {
                let span = SourceSpan::new(text_start, abs, text_start_line, text_start_col);
                nodes.push(HickNode::Text(
                    self.input[text_start..abs].to_string(),
                    Some(span),
                ));
            }

            if tag.self_closing {
                nodes.push(HickNode::Tag(tag));
            } else if is_raw_content_tag(&tag.name) {
                // Verbatim-capture element (session vocabulary:
                // tool payloads and tool results): content is raw
                // text up to the next matching close marker, so
                // captured fragments containing unbalanced
                // hick-like markers cannot break the parse.
                let close = format!("{}{}>", self.close_marker, tag.name);
                let rem = self.remaining();
                let (end, close_span) = match rem.find(&close) {
                    Some(end) => {
                        let close_start = self.pos + end;
                        // The closer's line is the content's last line; it
                        // is recomputed below once the cursor is past it.
                        (end, Some((close_start, close_start + close.len())))
                    }
                    None => {
                        let error = ParseError::UnclosedTag {
                            prefix: self.prefix.clone(),
                            name: tag.name.clone(),
                            line: tag.source_line,
                        };
                        if !self.lenient {
                            return Err(error);
                        }
                        self.recover(error);
                        (rem.len(), None)
                    }
                };
                let raw = rem[..end].to_string();
                let span = SourceSpan::new(self.pos, self.pos + raw.len(), self.line, self.col);
                self.advance(end);
                let close_span = close_span.map(|(start, close_end)| {
                    let span = SourceSpan::new(start, close_end, self.line, self.col);
                    self.advance(close.len());
                    span
                });
                let mut children = Vec::new();
                if !raw.is_empty() {
                    children.push(HickNode::Text(raw, Some(span)));
                }
                nodes.push(HickNode::Tag(HickTag {
                    children,
                    close_span,
                    ..tag
                }));
            } else {
                // Parse nested children
                let tag_name = tag.name.clone();
                let tag_line = tag.source_line;
                let (children, close_span) = self.parse_children(Some(&tag_name), tag_line)?;
                nodes.push(HickNode::Tag(HickTag {
                    children,
                    close_span,
                    ..tag
                }));
            }

            text_start = self.pos;
            text_start_line = self.line;
            text_start_col = self.col;
        }
    }

    fn line_at(&self, offset: usize) -> usize {
        self.input[..offset]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count()
            + 1
    }

    /// Parse an opening tag. Cursor must be at the opening marker.
    fn parse_open_tag(&mut self) -> Result<HickTag, ParseError> {
        let tag_line = self.line;
        let tag_col = self.col;
        let tag_start_offset = self.pos;

        // Skip the opening marker (e.g. `<hick:` or `<h:`)
        self.advance(self.open_marker.len());

        // Read tag name (until whitespace, `>`, or `/`)
        let name_start = self.pos;
        while !self.is_eof() {
            let ch = self.input.as_bytes()[self.pos];
            if ch == b' ' || ch == b'\t' || ch == b'\n' || ch == b'\r' || ch == b'>' || ch == b'/' {
                break;
            }
            self.pos += 1;
            self.col += 1;
        }
        let name = self.input[name_start..self.pos].to_string();
        if name.is_empty() {
            return Err(ParseError::Syntax {
                line: tag_line,
                message: format!("empty tag name after <{}:", self.prefix),
            });
        }

        // Parse attributes
        let mut attributes = Vec::new();
        let mut self_closing = false;

        loop {
            self.skip_ws();
            if self.is_eof() {
                return Err(ParseError::Syntax {
                    line: tag_line,
                    message: format!("unexpected EOF in <{}:{name}>", self.prefix),
                });
            }

            let ch = self.input.as_bytes()[self.pos];

            if ch == b'>' {
                self.advance(1);
                break;
            }

            if ch == b'/' {
                // Check for />
                if self.pos + 1 < self.input.len() && self.input.as_bytes()[self.pos + 1] == b'>' {
                    self_closing = true;
                    self.advance(2);
                    break;
                }
                return Err(ParseError::Syntax {
                    line: tag_line,
                    message: "unexpected '/' not followed by '>'".to_string(),
                });
            }

            // Parse attribute: `name="value"`, or bare `name` for a flag.
            let attr_name = self.read_attr_name(tag_line)?;
            // A name that read as nothing means the byte here starts no
            // attribute at all. Without this the loop makes no progress and
            // a malformed tag hangs the parser instead of failing.
            if attr_name.is_empty() {
                return Err(ParseError::Syntax {
                    line: self.line,
                    message: format!(
                        "unexpected '{}' where an attribute name was expected in <{}:{name}>\n\n{NO_ESCAPING_HINT}",
                        self.input[self.pos..].chars().next().unwrap_or('?'),
                        self.prefix,
                    ),
                });
            }
            self.skip_ws();

            // A valueless attribute is a flag — `<hick:paste … distinct />`.
            // Additive: this spelling used to be a syntax error, so no
            // document that parsed before parses differently now. Flags read
            // as the empty string, which `has_flag` treats as "present".
            if self.is_eof() || self.input.as_bytes()[self.pos] != b'=' {
                attributes.push((attr_name, String::new()));
                continue;
            }
            self.advance(1); // skip =
            self.skip_ws();

            let attr_value = self.read_attr_value(tag_line)?;
            attributes.push((attr_name, attr_value));
        }

        let tag_end_offset = self.pos;
        Ok(HickTag {
            name,
            attributes,
            children: Vec::new(),
            self_closing,
            source_line: tag_line,
            source_column: tag_col,
            source_span: Some(SourceSpan::new(
                tag_start_offset,
                tag_end_offset,
                tag_line,
                tag_col,
            )),
            close_span: None,
        })
    }

    /// Parse a closing tag. Cursor must be at the closing marker.
    ///
    /// Returns the tag's name, the line it starts on, and its span. A closing
    /// tag cannot fail to parse: whatever follows the marker up to `>` (or
    /// whitespace) is its name.
    fn parse_close_tag(&mut self) -> (String, usize, SourceSpan) {
        let line = self.line;
        let start = self.pos;
        let col = self.col;
        // Skip the closing marker (e.g. `</hick:` or `</h:`)
        self.advance(self.close_marker.len());

        let name_start = self.pos;
        while !self.is_eof() {
            let ch = self.input.as_bytes()[self.pos];
            if ch == b'>' || ch == b' ' || ch == b'\t' || ch == b'\n' || ch == b'\r' {
                break;
            }
            self.pos += 1;
        }
        let name = self.input[name_start..self.pos].to_string();

        // Update line count
        let name_slice = &self.input[name_start..self.pos];
        self.line += name_slice.chars().filter(|&c| c == '\n').count();

        self.skip_ws();
        if !self.is_eof() && self.input.as_bytes()[self.pos] == b'>' {
            self.advance(1);
        }

        (name, line, SourceSpan::new(start, self.pos, line, col))
    }

    fn skip_ws(&mut self) {
        while !self.is_eof() {
            let ch = self.input.as_bytes()[self.pos];
            if ch == b' ' || ch == b'\t' || ch == b'\n' || ch == b'\r' {
                if ch == b'\n' {
                    self.line += 1;
                }
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn read_attr_name(&mut self, _context_line: usize) -> Result<String, ParseError> {
        let start = self.pos;
        while !self.is_eof() {
            let ch = self.input.as_bytes()[self.pos];
            if ch == b'='
                || ch == b' '
                || ch == b'\t'
                || ch == b'\n'
                || ch == b'\r'
                || ch == b'>'
                || ch == b'/'
            {
                break;
            }
            self.pos += 1;
        }
        let name = &self.input[start..self.pos];
        if name.is_empty() {
            return Err(ParseError::Syntax {
                line: self.line,
                message: format!("empty attribute name\n\n{NO_ESCAPING_HINT}"),
            });
        }
        Ok(name.to_string())
    }

    fn read_attr_value(&mut self, context_line: usize) -> Result<String, ParseError> {
        if self.is_eof() {
            return Err(ParseError::Syntax {
                line: context_line,
                message: "unexpected EOF reading attribute value".to_string(),
            });
        }

        let quote = self.input.as_bytes()[self.pos];
        if quote != b'"' && quote != b'\'' {
            return Err(ParseError::Syntax {
                line: self.line,
                message: format!("attribute value must be quoted\n\n{NO_ESCAPING_HINT}"),
            });
        }
        self.advance(1); // skip opening quote

        let start = self.pos;
        while !self.is_eof() && self.input.as_bytes()[self.pos] != quote {
            if self.input.as_bytes()[self.pos] == b'\n' {
                self.line += 1;
            }
            self.pos += 1;
        }

        if self.is_eof() {
            return Err(ParseError::Syntax {
                line: context_line,
                message: format!("unterminated attribute value\n\n{NO_ESCAPING_HINT}"),
            });
        }

        let value = self.input[start..self.pos].to_string();
        self.advance(1); // skip closing quote
        Ok(value)
    }
}

// ---------------------------------------------------------------------------
// Session node extraction
// ---------------------------------------------------------------------------

/// Is `name` a verbatim-capture element? See the note on `parse_children`.
fn is_raw_content_tag(name: &str) -> bool {
    // `transcript` joins the session vocabulary here for the same reason: its
    // content is bytes some other tool produced, and a meeting where somebody
    // said "the hick:copy tag" is not a parse error. See
    // `docs/specs/freeform/ingest.md` — the raw block is the source of truth,
    // and speaker turns are derived from it rather than stored beside it.
    // `context` is what a harness put in front of the model besides the
    // conversation — a compaction summary, a hook's output, an attached file.
    // Bytes another tool produced, like a tool result; see `hick ingest --from claude-code`.
    matches!(
        name,
        "input" | "tool-result" | "transcript" | "reasoning" | "context"
    )
}
