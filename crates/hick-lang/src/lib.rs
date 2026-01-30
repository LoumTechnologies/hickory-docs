//! Parser and AST for `.hick` files.
//!
//! Only `<hick:*>` tags are parsed as structured elements. Content between them
//! is raw text that can contain `&`, `<`, `>` without escaping.

use std::fmt;

// ---------------------------------------------------------------------------
// AST
// ---------------------------------------------------------------------------

/// A node in the hick document tree.
#[derive(Debug, Clone, PartialEq)]
pub enum HickNode {
    Text(String),
    Tag(HickTag),
}

/// A `<hick:*>` element with attributes and children.
#[derive(Debug, Clone, PartialEq)]
pub struct HickTag {
    /// Local name after the `hick:` prefix, e.g. `"file"`, `"exec"`, `"container"`.
    pub name: String,
    /// Attribute pairs in document order.
    pub attributes: Vec<(String, String)>,
    /// Child nodes (text and nested tags).
    pub children: Vec<HickNode>,
    /// Whether this was a self-closing tag (`<hick:allow ... />`).
    pub self_closing: bool,
    /// 1-based line number where the opening tag starts.
    pub source_line: usize,
}

impl HickTag {
    /// Get the value of an attribute by name.
    pub fn get_attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }

    /// Iterate over child tags (skipping text nodes).
    pub fn child_tags(&self) -> impl Iterator<Item = &HickTag> {
        self.children.iter().filter_map(|n| match n {
            HickNode::Tag(t) => Some(t),
            HickNode::Text(_) => None,
        })
    }

    /// Collect all text content (direct and nested) into a single string.
    pub fn text_content(&self) -> String {
        let mut out = String::new();
        collect_text(&self.children, &mut out);
        out
    }
}

fn collect_text(nodes: &[HickNode], out: &mut String) {
    for node in nodes {
        match node {
            HickNode::Text(t) => out.push_str(t),
            HickNode::Tag(tag) => collect_text(&tag.children, out),
        }
    }
}

/// A parsed `.hick` document.
#[derive(Debug, Clone)]
pub struct HickDocument {
    /// Children of the root `<hick:doc>` element.
    pub nodes: Vec<HickNode>,
    /// Original source text.
    pub source: String,
}

impl HickDocument {
    /// Iterate over top-level tags.
    pub fn tags(&self) -> impl Iterator<Item = &HickTag> {
        self.nodes.iter().filter_map(|n| match n {
            HickNode::Tag(t) => Some(t),
            HickNode::Text(_) => None,
        })
    }

    /// Find all tags with a given local name (non-recursive).
    pub fn find_tags(&self, name: &str) -> Vec<&HickTag> {
        self.tags().filter(|t| t.name == name).collect()
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("line {line}: {message}")]
    Syntax { line: usize, message: String },

    #[error("missing root <hick:doc> element")]
    MissingRoot,

    #[error("unclosed tag <hick:{name}> opened at line {line}")]
    UnclosedTag { name: String, line: usize },

    #[error("line {line}: unexpected closing tag </hick:{name}>")]
    UnexpectedClose { name: String, line: usize },
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

/// Parse a `.hick` file from source text.
///
/// The parser is a custom scanner that only recognizes `<hick:*>` and
/// `</hick:*>` tags. Everything else is treated as raw text.
pub fn parse(source: &str) -> Result<HickDocument, ParseError> {
    let mut parser = Parser::new(source);
    parser.parse_document()
}

struct Parser<'a> {
    input: &'a str,
    pos: usize,
    line: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            pos: 0,
            line: 1,
        }
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

    /// Advance `n` bytes, tracking line numbers.
    fn advance(&mut self, n: usize) {
        let slice = &self.input[self.pos..self.pos + n];
        self.line += slice.chars().filter(|&c| c == '\n').count();
        self.pos += n;
    }

    /// Parse the full document.
    fn parse_document(&mut self) -> Result<HickDocument, ParseError> {
        self.skip_xml_declaration();

        // Find the root <hick:doc ...> tag
        let root_start = self
            .remaining()
            .find("<hick:doc")
            .ok_or(ParseError::MissingRoot)?;
        self.advance(root_start);

        // Parse the opening tag attributes and consume `>`
        let tag = self.parse_open_tag()?;
        if tag.name != "doc" {
            return Err(ParseError::MissingRoot);
        }

        // Parse children until </hick:doc>
        let nodes = self.parse_children("doc", tag.source_line)?;

        Ok(HickDocument {
            nodes,
            source: self.input.to_string(),
        })
    }

    /// Parse children until `</hick:{close_name}>` is found.
    fn parse_children(
        &mut self,
        close_name: &str,
        open_line: usize,
    ) -> Result<Vec<HickNode>, ParseError> {
        let mut nodes = Vec::new();
        let mut text_start = self.pos;

        loop {
            if self.is_eof() {
                return Err(ParseError::UnclosedTag {
                    name: close_name.to_string(),
                    line: open_line,
                });
            }

            // Look for the next `<hick:` or `</hick:` marker
            let rem = self.remaining();
            let next_open = rem.find("<hick:");
            let next_close = rem.find("</hick:");

            // Determine the nearest marker
            let nearest = match (next_open, next_close) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            };

            match nearest {
                None => {
                    // No more hick tags -- rest is text (will be caught as unclosed)
                    return Err(ParseError::UnclosedTag {
                        name: close_name.to_string(),
                        line: open_line,
                    });
                }
                Some(offset) => {
                    let abs = self.pos + offset;

                    // Is this a closing tag?
                    if self.input[abs..].starts_with("</hick:") {
                        // Flush text before this closing tag
                        if abs > text_start {
                            nodes.push(HickNode::Text(self.input[text_start..abs].to_string()));
                        }
                        self.advance(offset);

                        // Parse closing tag name
                        let (name, _) = self.parse_close_tag()?;
                        if name == close_name {
                            return Ok(nodes);
                        } else {
                            return Err(ParseError::UnexpectedClose {
                                name,
                                line: self.line,
                            });
                        }
                    }

                    // It's an opening tag
                    // Flush text before this tag
                    if abs > text_start {
                        nodes.push(HickNode::Text(self.input[text_start..abs].to_string()));
                    }
                    self.advance(offset);

                    let tag = self.parse_open_tag()?;

                    if tag.self_closing {
                        nodes.push(HickNode::Tag(tag));
                    } else {
                        // Parse nested children
                        let tag_name = tag.name.clone();
                        let tag_line = tag.source_line;
                        let children = self.parse_children(&tag_name, tag_line)?;
                        nodes.push(HickNode::Tag(HickTag { children, ..tag }));
                    }

                    text_start = self.pos;
                }
            }
        }
    }

    /// Parse `<hick:name attr="val" ...>` or `<hick:name ... />`.
    /// Cursor must be at `<hick:`.
    fn parse_open_tag(&mut self) -> Result<HickTag, ParseError> {
        let tag_line = self.line;

        // Skip `<hick:`
        self.advance(6);

        // Read tag name (until whitespace, `>`, or `/`)
        let name_start = self.pos;
        while !self.is_eof() {
            let ch = self.input.as_bytes()[self.pos];
            if ch == b' ' || ch == b'\t' || ch == b'\n' || ch == b'\r' || ch == b'>' || ch == b'/' {
                break;
            }
            self.pos += 1;
        }
        let name = self.input[name_start..self.pos].to_string();
        if name.is_empty() {
            return Err(ParseError::Syntax {
                line: tag_line,
                message: "empty tag name after <hick:".to_string(),
            });
        }

        // Update line count for name
        let name_slice = &self.input[name_start..self.pos];
        self.line += name_slice.chars().filter(|&c| c == '\n').count();

        // Parse attributes
        let mut attributes = Vec::new();
        let mut self_closing = false;

        loop {
            self.skip_ws();
            if self.is_eof() {
                return Err(ParseError::Syntax {
                    line: tag_line,
                    message: format!("unexpected EOF in <hick:{name}>"),
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

            // Parse attribute: name="value"
            let attr_name = self.read_attr_name(tag_line)?;
            self.skip_ws();

            if self.is_eof() || self.input.as_bytes()[self.pos] != b'=' {
                return Err(ParseError::Syntax {
                    line: self.line,
                    message: format!("expected '=' after attribute '{attr_name}'"),
                });
            }
            self.advance(1); // skip =
            self.skip_ws();

            let attr_value = self.read_attr_value(tag_line)?;
            attributes.push((attr_name, attr_value));
        }

        Ok(HickTag {
            name,
            attributes,
            children: Vec::new(),
            self_closing,
            source_line: tag_line,
        })
    }

    /// Parse `</hick:name>`. Cursor must be at `</hick:`.
    fn parse_close_tag(&mut self) -> Result<(String, usize), ParseError> {
        let line = self.line;
        // Skip `</hick:`
        self.advance(7);

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

        Ok((name, line))
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
                message: "empty attribute name".to_string(),
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
                message: "attribute value must be quoted".to_string(),
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
                message: "unterminated attribute value".to_string(),
            });
        }

        let value = self.input[start..self.pos].to_string();
        self.advance(1); // skip closing quote
        Ok(value)
    }
}

impl fmt::Display for HickTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<hick:{}", self.name)?;
        for (k, v) in &self.attributes {
            write!(f, " {k}=\"{v}\"")?;
        }
        if self.self_closing {
            write!(f, " />")?;
        } else {
            write!(f, ">")?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_document() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
</hick:doc>"#;
        let doc = parse(src).unwrap();
        assert!(
            doc.nodes.is_empty()
                || doc
                    .nodes
                    .iter()
                    .all(|n| matches!(n, HickNode::Text(t) if t.trim().is_empty()))
        );
    }

    #[test]
    fn parse_text_content_with_special_chars() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
This has & and < and > without escaping.
</hick:doc>"#;
        let doc = parse(src).unwrap();
        let text: String = doc
            .nodes
            .iter()
            .filter_map(|n| match n {
                HickNode::Text(t) => Some(t.as_str()),
                _ => None,
            })
            .collect();
        assert!(text.contains("&"));
        assert!(text.contains("<"));
        assert!(text.contains(">"));
    }

    #[test]
    fn parse_self_closing_tag() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:allow network="github.com:443" />
</hick:doc>"#;
        let doc = parse(src).unwrap();
        let tags: Vec<_> = doc.find_tags("allow");
        assert_eq!(tags.len(), 1);
        assert!(tags[0].self_closing);
        assert_eq!(tags[0].get_attribute("network"), Some("github.com:443"));
    }

    #[test]
    fn parse_nested_tags() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:container name="reporter" image="python:3.12">
  <hick:allow network="github.com:443" />
  <hick:deny network="*" />
</hick:container>
</hick:doc>"#;
        let doc = parse(src).unwrap();
        let containers = doc.find_tags("container");
        assert_eq!(containers.len(), 1);
        let c = containers[0];
        assert_eq!(c.get_attribute("name"), Some("reporter"));
        assert_eq!(c.get_attribute("image"), Some("python:3.12"));

        let child_tags: Vec<_> = c.child_tags().collect();
        assert_eq!(child_tags.len(), 2);
        assert_eq!(child_tags[0].name, "allow");
        assert_eq!(child_tags[1].name, "deny");
    }

    #[test]
    fn parse_exec_with_raw_content() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:exec container="demo" image="alpine">
cat /etc/alpine-release
</hick:exec>
</hick:doc>"#;
        let doc = parse(src).unwrap();
        let execs = doc.find_tags("exec");
        assert_eq!(execs.len(), 1);
        assert_eq!(execs[0].get_attribute("container"), Some("demo"));
        assert_eq!(execs[0].get_attribute("image"), Some("alpine"));

        let content = execs[0].text_content();
        assert!(content.contains("cat /etc/alpine-release"));
    }

    #[test]
    fn parse_file_with_nested_exec() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:file path="Getting Started.md">
# Getting Started

    <hick:exec container="demo" image="alpine">
    cat /etc/alpine-release
    </hick:exec>

Done!
</hick:file>
</hick:doc>"#;
        let doc = parse(src).unwrap();
        let files = doc.find_tags("file");
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].get_attribute("path"), Some("Getting Started.md"));

        let execs: Vec<_> = files[0].child_tags().collect();
        assert_eq!(execs.len(), 1);
        assert_eq!(execs[0].name, "exec");
    }

    #[test]
    fn parse_copy_paste_cut() {
        let src = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:exec container="analyzer">
<hick:copy id="version-info">
python -c "import sys; print(sys.version)"
</hick:copy>
</hick:exec>

<hick:file path="report.md">
Python: <hick:paste select="#version-info" />
</hick:file>

<hick:cut id="internal-only">
This is removed from output.
</hick:cut>
</hick:doc>"##;
        let doc = parse(src).unwrap();

        let copies = doc.tags().flat_map(|t| {
            let mut found = Vec::new();
            find_tags_recursive(t, "copy", &mut found);
            found
        });
        assert_eq!(copies.count(), 1);

        let pastes = doc.tags().flat_map(|t| {
            let mut found = Vec::new();
            find_tags_recursive(t, "paste", &mut found);
            found
        });
        let pastes: Vec<_> = pastes.collect();
        assert_eq!(pastes.len(), 1);
        assert_eq!(pastes[0].get_attribute("select"), Some(r#"#version-info"#));
        assert!(pastes[0].self_closing);

        let cuts = doc.find_tags("cut");
        assert_eq!(cuts.len(), 1);
        assert_eq!(cuts[0].get_attribute("id"), Some("internal-only"));
    }

    fn find_tags_recursive<'a>(tag: &'a HickTag, name: &str, out: &mut Vec<&'a HickTag>) {
        if tag.name == name {
            out.push(tag);
        }
        for child in tag.child_tags() {
            find_tags_recursive(child, name, out);
        }
    }

    #[test]
    fn parse_full_example_untrusted_code() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">

<hick:container name="reporter" image="python:3.12">
  <hick:allow network="github.com:443" />
  <hick:allow network="pypi.org:443" />
  <hick:deny network="*" />
  <hick:allow file-write="/output/*" />
</hick:container>

<hick:container name="mailer" image="alpine">
  <hick:allow network="smtp.gmail.com:587" />
  <hick:deny network="*" />
  <hick:allow file-read="/output/*" />
</hick:container>

<hick:volume name="shared-output" />

<hick:exec container="reporter" mount="shared-output:/output">
pip install untrusted-package
python -c "
import untrusted_package
with open('/output/report.html', 'w') as f:
    f.write(untrusted_package.generate())
"
</hick:exec>

<hick:exec container="mailer" mount="shared-output:/output">
apk add msmtp
cat /output/report.html | msmtp bob@example.com
</hick:exec>

</hick:doc>"#;
        let doc = parse(src).unwrap();

        // Containers
        let containers = doc.find_tags("container");
        assert_eq!(containers.len(), 2);
        assert_eq!(containers[0].get_attribute("name"), Some("reporter"));
        assert_eq!(containers[1].get_attribute("name"), Some("mailer"));

        // Reporter capabilities
        let reporter_rules: Vec<_> = containers[0].child_tags().collect();
        assert_eq!(reporter_rules.len(), 4);

        // Volumes
        let volumes = doc.find_tags("volume");
        assert_eq!(volumes.len(), 1);
        assert_eq!(volumes[0].get_attribute("name"), Some("shared-output"));

        // Execs
        let execs = doc.find_tags("exec");
        assert_eq!(execs.len(), 2);
        assert_eq!(execs[0].get_attribute("container"), Some("reporter"));
        assert_eq!(execs[1].get_attribute("container"), Some("mailer"));
        assert_eq!(
            execs[0].get_attribute("mount"),
            Some("shared-output:/output")
        );
    }

    #[test]
    fn parse_attenuate_and_fork() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:attenuate container="sandbox">
  <hick:deny network="*" />
</hick:attenuate>

<hick:fork from="base" to="analyzer-a">
  <hick:deny network="*" />
</hick:fork>

<hick:confirm message="Proceed?" />
</hick:doc>"#;
        let doc = parse(src).unwrap();

        let attenuates = doc.find_tags("attenuate");
        assert_eq!(attenuates.len(), 1);
        assert_eq!(attenuates[0].get_attribute("container"), Some("sandbox"));

        let forks = doc.find_tags("fork");
        assert_eq!(forks.len(), 1);
        assert_eq!(forks[0].get_attribute("from"), Some("base"));
        assert_eq!(forks[0].get_attribute("to"), Some("analyzer-a"));

        let confirms = doc.find_tags("confirm");
        assert_eq!(confirms.len(), 1);
        assert_eq!(confirms[0].get_attribute("message"), Some("Proceed?"));
    }

    #[test]
    fn parse_secret_tag() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:container name="deploy" image="alpine">
  <hick:secret name="PULUMI_TOKEN" from="pulumi-token" />
</hick:container>
</hick:doc>"#;
        let doc = parse(src).unwrap();
        let containers = doc.find_tags("container");
        let secrets: Vec<_> = containers[0]
            .child_tags()
            .filter(|t| t.name == "secret")
            .collect();
        assert_eq!(secrets.len(), 1);
        assert_eq!(secrets[0].get_attribute("name"), Some("PULUMI_TOKEN"));
        assert_eq!(secrets[0].get_attribute("from"), Some("pulumi-token"));
    }

    #[test]
    fn error_on_unclosed_tag() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:exec container="demo">
echo hello
</hick:doc>"#;
        let err = parse(src).unwrap_err();
        assert!(matches!(err, ParseError::UnexpectedClose { .. }));
    }

    #[test]
    fn error_on_missing_root() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<html><body>Hello</body></html>"#;
        let err = parse(src).unwrap_err();
        assert!(matches!(err, ParseError::MissingRoot));
    }

    #[test]
    fn line_numbers_tracked() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">

<hick:exec container="demo">
echo hello
</hick:exec>

</hick:doc>"#;
        let doc = parse(src).unwrap();
        let execs = doc.find_tags("exec");
        assert_eq!(execs[0].source_line, 4);
    }

    #[test]
    fn single_quoted_attributes() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:allow network='github.com:443' />
</hick:doc>"#;
        let doc = parse(src).unwrap();
        let allows = doc.find_tags("allow");
        assert_eq!(allows[0].get_attribute("network"), Some("github.com:443"));
    }

    #[test]
    fn multiple_execs_same_container() {
        let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:exec container="demo" image="alpine">
apk add curl
</hick:exec>
<hick:exec container="demo">
curl --version
</hick:exec>
</hick:doc>"#;
        let doc = parse(src).unwrap();
        let execs = doc.find_tags("exec");
        assert_eq!(execs.len(), 2);
        assert_eq!(execs[0].get_attribute("image"), Some("alpine"));
        assert!(execs[1].get_attribute("image").is_none());
    }
}
