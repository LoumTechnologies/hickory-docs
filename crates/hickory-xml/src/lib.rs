use std::collections::HashSet;
use std::sync::Arc;

// =============
// DOM contracts
// =============

/// Minimal node model needed by ConvertXmlToDynamicTree.
#[derive(Debug, Clone)]
pub enum XmlNode {
    Text(String),
    Element(Arc<dyn XmlElement>),
}

/// Minimal document model: your parser should produce something that implements this.
pub trait XmlDocument: Send + Sync {
    /// Top-level children (C# iterated `document.ChildNodes.OfType<IElement>()`).
    fn child_nodes(&self) -> Vec<XmlNode>;

    /// Used for indentation lookups.
    fn source_text(&self) -> Option<&str>;
}

/// Minimal element model: wraps what AngleSharp IElement gave you.
pub trait XmlElement: Send + Sync + std::fmt::Debug {
    fn tag_name(&self) -> &str;
    fn namespace_uri(&self) -> Option<&str>;

    fn attributes(&self) -> Vec<(String, String)>;
    fn get_attribute(&self, name: &str) -> Option<String>;
    fn has_attribute(&self, name: &str) -> bool {
        self.get_attribute(name).is_some()
    }

    fn child_nodes(&self) -> Vec<XmlNode>;

    /// For `#generated-files` selector support.
    fn id(&self) -> Option<String> {
        self.get_attribute("id")
    }

    /// 1-based line number where the element starts (AngleSharp SourceReference.Position.Line).
    fn source_line_1_based(&self) -> Option<usize>;

    /// Find the root element (needed for indentation in C#). If your DOM can't do this,
    /// you can return `None` and indentation will become an error.
    fn root_element(&self) -> Option<Arc<dyn XmlElement>>;

    /// Owner document (needed for indentation). If your DOM can't do this,
    /// you can return `None` and indentation will become an error.
    fn owner_document(&self) -> Option<Arc<dyn XmlDocument>>;

    /// Returns the exact original text slice that was parsed to produce this element, if available.
    /// Implementations may return `None` if they don't track original source.
    fn original_source(&self) -> Option<&str> {
        None
    }
}

// ========================
// roxmltree implementation
// ========================

struct RoxmlDocumentShared {
    _source: String,
    doc: roxmltree::Document<'static>,
}

unsafe impl Send for RoxmlDocumentShared {}
unsafe impl Sync for RoxmlDocumentShared {}

impl RoxmlDocumentShared {
    fn new(source: String) -> Result<Arc<Self>, roxmltree::Error> {
        let leaked: &'static str = Box::leak(source.clone().into_boxed_str());
        let doc = roxmltree::Document::parse(leaked)?;
        Ok(Arc::new(Self {
            _source: source,
            doc,
        }))
    }
}

#[derive(Clone)]
struct RoxmlDocument {
    shared: Arc<RoxmlDocumentShared>,
}

impl XmlDocument for RoxmlDocument {
    fn child_nodes(&self) -> Vec<XmlNode> {
        let mut nodes = Vec::new();
        for node in self.shared.doc.root().children() {
            if node.is_element() {
                nodes.push(XmlNode::Element(Arc::new(RoxmlElement {
                    doc: self.shared.clone(),
                    node_id: node.id(),
                })));
            }
        }
        nodes
    }

    fn source_text(&self) -> Option<&str> {
        Some(self.shared.doc.input_text())
    }
}

struct RoxmlElement {
    doc: Arc<RoxmlDocumentShared>,
    node_id: roxmltree::NodeId,
}

impl std::fmt::Debug for RoxmlElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RoxmlElement")
            .field("tag_name", &self.tag_name())
            .finish()
    }
}

impl RoxmlElement {
    fn node(&self) -> roxmltree::Node<'_, 'static> {
        self.doc.doc.get_node(self.node_id).unwrap()
    }
}

impl XmlElement for RoxmlElement {
    fn tag_name(&self) -> &str {
        self.node().tag_name().name()
    }

    fn namespace_uri(&self) -> Option<&str> {
        self.node().tag_name().namespace()
    }

    fn attributes(&self) -> Vec<(String, String)> {
        self.node()
            .attributes()
            .map(|a| (a.name().to_string(), a.value().to_string()))
            .collect()
    }

    fn get_attribute(&self, name: &str) -> Option<String> {
        self.node().attribute(name).map(|s| s.to_string())
    }

    fn child_nodes(&self) -> Vec<XmlNode> {
        let mut nodes = Vec::new();
        for node in self.node().children() {
            if node.is_text() {
                nodes.push(XmlNode::Text(node.text().unwrap_or("").to_string()));
            } else if node.is_element() {
                nodes.push(XmlNode::Element(Arc::new(RoxmlElement {
                    doc: self.doc.clone(),
                    node_id: node.id(),
                })));
            }
        }
        nodes
    }

    fn source_line_1_based(&self) -> Option<usize> {
        let pos = self.doc.doc.text_pos_at(self.node().range().start);
        Some(pos.row as usize)
    }

    fn root_element(&self) -> Option<Arc<dyn XmlElement>> {
        let mut curr = self.node();
        while let Some(parent) = curr.parent() {
            if parent.is_element() {
                curr = parent;
            } else {
                break;
            }
        }
        if curr.is_element() {
            Some(Arc::new(RoxmlElement {
                doc: self.doc.clone(),
                node_id: curr.id(),
            }))
        } else {
            None
        }
    }

    fn owner_document(&self) -> Option<Arc<dyn XmlDocument>> {
        Some(Arc::new(RoxmlDocument {
            shared: self.doc.clone(),
        }) as Arc<dyn XmlDocument>)
    }

    fn original_source(&self) -> Option<&str> {
        let range = self.node().range();
        Some(&self.doc.doc.input_text()[range.start..range.end])
    }
}

// ==================
// Errors / exceptions
// ==================

#[derive(Debug, thiserror::Error)]
pub enum HickoryDocumentFormatError {
    #[error("{message} (file={file:?}, line={line:?}, col={col:?})")]
    Format {
        file: Option<String>,
        line: Option<usize>,
        col: Option<usize>,
        message: String,
    },
}

impl HickoryDocumentFormatError {
    pub fn new(
        file: Option<String>,
        line: Option<usize>,
        col: Option<usize>,
        message: impl Into<String>,
    ) -> Self {
        Self::Format {
            file,
            line,
            col,
            message: message.into(),
        }
    }
}

// =======================
// Parsing entry point
// =======================

/// Parse an XML string into an `XmlDocument`.
/// Returns `None` if the input is not valid XML.
pub fn parse_xml(input: &str) -> Option<Arc<dyn XmlDocument>> {
    match RoxmlDocumentShared::new(input.to_string()) {
        Ok(shared) => Some(Arc::new(RoxmlDocument { shared }) as Arc<dyn XmlDocument>),
        Err(_) => None,
    }
}

/// Parse an XML string into an `XmlDocument`, returning the parse error on failure.
pub fn try_parse_xml(input: &str) -> Result<Arc<dyn XmlDocument>, roxmltree::Error> {
    let shared = RoxmlDocumentShared::new(input.to_string())?;
    Ok(Arc::new(RoxmlDocument { shared }) as Arc<dyn XmlDocument>)
}

// =======================
// Selector implementation
// =======================

pub fn select_elements_under(
    root: &Arc<dyn XmlElement>,
    selector: &str,
) -> Result<Vec<Arc<dyn XmlElement>>, HickoryDocumentFormatError> {
    if let Some(id) = selector.strip_prefix('#') {
        let mut out = Vec::new();
        walk_elements(root, &mut |e| {
            if e.id().as_deref() == Some(id) {
                out.push(e.clone());
            }
        });
        return Ok(out);
    }

    if let Some(xpath) = selector.strip_prefix("//") {
        // Supports:
        //   //tag
        //   //tag[@attr='value']
        let (tag, attr_filter) = parse_min_xpath(xpath)?;
        let mut out = Vec::new();

        walk_elements(root, &mut |e| {
            if e.tag_name() != tag {
                return;
            }
            if let Some((attr, expected)) = &attr_filter {
                if e.get_attribute(attr).as_deref() != Some(expected.as_str()) {
                    return;
                }
            }
            out.push(e.clone());
        });

        return Ok(out);
    }

    // Simple tag selector like "doc"
    let tag = selector.trim();
    let mut out = Vec::new();
    walk_elements(root, &mut |e| {
        if e.tag_name() == tag {
            out.push(e.clone());
        }
    });
    Ok(out)
}

type AttrFilter = Option<(String, String)>;

fn parse_min_xpath(s: &str) -> Result<(&str, AttrFilter), HickoryDocumentFormatError> {
    // "//file[@path='x']"
    if let Some(bracket) = s.find('[') {
        let tag = &s[..bracket];
        let rest = &s[bracket..];
        // Expect [@name='value']
        if !rest.starts_with("[@") || !rest.ends_with(']') {
            return Err(HickoryDocumentFormatError::new(
                None,
                None,
                None,
                "Unsupported XPath selector",
            ));
        }
        let inner = &rest[2..rest.len() - 1]; // drop [@ and ]
        let Some(eq) = inner.find('=') else {
            return Err(HickoryDocumentFormatError::new(
                None,
                None,
                None,
                "Unsupported XPath selector",
            ));
        };
        let attr = inner[..eq].to_string();
        let mut value = inner[eq + 1..].trim().to_string();
        // accept either '...' or "..."
        if (value.starts_with('\'') && value.ends_with('\''))
            || (value.starts_with('"') && value.ends_with('"'))
        {
            value = value[1..value.len() - 1].to_string();
        }
        Ok((tag, Some((attr, value))))
    } else {
        Ok((s, None))
    }
}

pub fn walk_elements(root: &Arc<dyn XmlElement>, f: &mut dyn FnMut(&Arc<dyn XmlElement>)) {
    f(root);
    for c in root.child_nodes() {
        if let XmlNode::Element(e) = c {
            walk_elements(&e, f);
        }
    }
}

pub fn find_first_element_by_tag_in_document(
    doc: &Arc<dyn XmlDocument>,
    tag: &str,
) -> Option<Arc<dyn XmlElement>> {
    for node in doc.child_nodes() {
        if let XmlNode::Element(el) = node {
            let mut found = None;
            walk_elements(&el, &mut |e| {
                if found.is_none() && e.tag_name() == tag {
                    found = Some(e.clone());
                }
            });
            if found.is_some() {
                return found;
            }
        }
    }
    None
}

/// Search across multiple documents for elements matching a selector.
/// Deduplicates by pointer identity.
pub fn find_elements_in_documents(
    documents: &[Arc<dyn XmlDocument>],
    selector: &str,
) -> Result<Vec<Arc<dyn XmlElement>>, HickoryDocumentFormatError> {
    let mut matches: HashSet<usize> = HashSet::new();
    let mut out: Vec<Arc<dyn XmlElement>> = Vec::new();

    for doc in documents {
        for node in doc.child_nodes() {
            if let XmlNode::Element(el) = node {
                let found = select_elements_under(&el, selector)?;
                for e in found {
                    let key = Arc::as_ptr(&e) as *const () as usize;
                    if matches.insert(key) {
                        out.push(e);
                    }
                }
            }
        }
    }

    Ok(out)
}

// =======================
// Unit tests
// =======================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_xml() {
        let xml = r#"<?xml version="1.0"?><root><child attr="val">text</child></root>"#;
        let doc = parse_xml(xml).expect("should parse valid XML");
        let nodes = doc.child_nodes();
        assert_eq!(nodes.len(), 1);
        if let XmlNode::Element(root) = &nodes[0] {
            assert_eq!(root.tag_name(), "root");
            let children = root.child_nodes();
            assert_eq!(children.len(), 1);
            if let XmlNode::Element(child) = &children[0] {
                assert_eq!(child.tag_name(), "child");
                assert_eq!(child.get_attribute("attr").as_deref(), Some("val"));
                let grandchildren = child.child_nodes();
                assert_eq!(grandchildren.len(), 1);
                if let XmlNode::Text(t) = &grandchildren[0] {
                    assert_eq!(t, "text");
                } else {
                    panic!("expected text node");
                }
            } else {
                panic!("expected element node");
            }
        } else {
            panic!("expected element node");
        }
    }

    #[test]
    fn parse_invalid_xml_returns_none() {
        assert!(parse_xml("not xml at all <>>").is_none());
    }

    #[test]
    fn try_parse_returns_error() {
        assert!(try_parse_xml("not xml").is_err());
    }

    #[test]
    fn selector_by_id() {
        let xml = r#"<root><a id="x"/><b id="y"/></root>"#;
        let doc = parse_xml(xml).unwrap();
        let root_el = match &doc.child_nodes()[0] {
            XmlNode::Element(e) => e.clone(),
            _ => panic!("expected element"),
        };
        let found = select_elements_under(&root_el, "#y").unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].tag_name(), "b");
    }

    #[test]
    fn selector_by_tag() {
        let xml = r#"<root><a/><b/><a/></root>"#;
        let doc = parse_xml(xml).unwrap();
        let root_el = match &doc.child_nodes()[0] {
            XmlNode::Element(e) => e.clone(),
            _ => panic!("expected element"),
        };
        let found = select_elements_under(&root_el, "a").unwrap();
        assert_eq!(found.len(), 2);
    }

    #[test]
    fn selector_xpath_with_attr() {
        let xml = r#"<root><file path="a.txt"/><file path="b.txt"/></root>"#;
        let doc = parse_xml(xml).unwrap();
        let root_el = match &doc.child_nodes()[0] {
            XmlNode::Element(e) => e.clone(),
            _ => panic!("expected element"),
        };
        let found = select_elements_under(&root_el, "//file[@path='b.txt']").unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].get_attribute("path").as_deref(), Some("b.txt"));
    }

    #[test]
    fn find_first_element_in_document() {
        let xml = r#"<root><a/><b><c/></b></root>"#;
        let doc = parse_xml(xml).unwrap();
        let found = find_first_element_by_tag_in_document(&doc, "c").unwrap();
        assert_eq!(found.tag_name(), "c");
    }

    #[test]
    fn namespace_preserved() {
        let xml = r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0"><hick:file path="out.txt"/></hick:doc>"#;
        let doc = parse_xml(xml).unwrap();
        let file_el = find_first_element_by_tag_in_document(&doc, "file").unwrap();
        assert_eq!(
            file_el.namespace_uri(),
            Some("http://www.hickorydocs.com/1.0")
        );
    }

    #[test]
    fn original_source_preserved() {
        let xml = r#"<root><child attr="v">inner</child></root>"#;
        let doc = parse_xml(xml).unwrap();
        let child = find_first_element_by_tag_in_document(&doc, "child").unwrap();
        let src = child.original_source().unwrap();
        assert_eq!(src, r#"<child attr="v">inner</child>"#);
    }

    #[test]
    fn source_line_numbers() {
        let xml = "<root>\n  <child/>\n</root>";
        let doc = parse_xml(xml).unwrap();
        let child = find_first_element_by_tag_in_document(&doc, "child").unwrap();
        assert_eq!(child.source_line_1_based(), Some(2));
    }

    #[test]
    fn find_elements_across_documents() {
        let xml1 = r#"<root><item id="a"/></root>"#;
        let xml2 = r#"<root><item id="b"/></root>"#;
        let doc1 = parse_xml(xml1).unwrap();
        let doc2 = parse_xml(xml2).unwrap();
        let found = find_elements_in_documents(&[doc1, doc2], "item").unwrap();
        assert_eq!(found.len(), 2);
    }
}
