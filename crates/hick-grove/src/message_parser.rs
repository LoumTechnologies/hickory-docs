use std::collections::HashMap;

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::namespace::NamespaceUri;

/// A parsed multi-namespace XML message.
#[derive(Clone, Debug)]
pub struct ParsedMessage {
    pub namespaces: HashMap<String, NamespaceUri>,
    pub nodes: Vec<MessageNode>,
}

/// A node in the parsed message tree.
#[derive(Clone, Debug)]
pub enum MessageNode {
    Text(String),
    Element {
        namespace_uri: NamespaceUri,
        local_name: String,
        attributes: Vec<(String, String)>,
        children: Vec<MessageNode>,
    },
}

/// Parse a structured XML message, resolving namespace prefixes to URIs.
pub fn parse_message(xml: &str) -> anyhow::Result<ParsedMessage> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    let mut ns_stack: Vec<HashMap<String, NamespaceUri>> = vec![HashMap::new()];
    let mut node_stack: Vec<Vec<MessageNode>> = vec![vec![]];
    // Store (namespace_uri, local_name, attributes) so we don't lose attrs at End
    type TagEntry = (NamespaceUri, String, Vec<(String, String)>);
    let mut tag_stack: Vec<TagEntry> = vec![];
    let mut top_level_namespaces = HashMap::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let mut local_ns = HashMap::new();

                for attr in e.attributes().filter_map(|a| a.ok()) {
                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                    let val = String::from_utf8_lossy(&attr.value).to_string();

                    if key == "xmlns" {
                        local_ns.insert(String::new(), NamespaceUri::new(&val));
                    } else if let Some(prefix) = key.strip_prefix("xmlns:") {
                        local_ns.insert(prefix.to_string(), NamespaceUri::new(&val));
                    }
                }

                let mut scope = ns_stack.last().cloned().unwrap_or_default();
                for (k, v) in &local_ns {
                    scope.insert(k.clone(), v.clone());
                }

                if ns_stack.len() == 1 {
                    top_level_namespaces = scope.clone();
                }

                ns_stack.push(scope);

                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let (ns_uri, local_name) = resolve_tag_name(&tag_name, ns_stack.last().unwrap());

                let mut attrs = Vec::new();
                for attr in e.attributes().filter_map(|a| a.ok()) {
                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                    if key == "xmlns" || key.starts_with("xmlns:") {
                        continue;
                    }
                    let val = String::from_utf8_lossy(&attr.value).to_string();
                    attrs.push((key, val));
                }

                tag_stack.push((ns_uri, local_name, attrs));
                node_stack.push(vec![]);
            }
            Ok(Event::End(_)) => {
                let children = node_stack.pop().unwrap_or_default();
                let (ns_uri, local_name, attributes) = tag_stack.pop().unwrap();
                ns_stack.pop();

                let node = MessageNode::Element {
                    namespace_uri: ns_uri,
                    local_name,
                    attributes,
                    children,
                };

                if let Some(parent) = node_stack.last_mut() {
                    parent.push(node);
                }
            }
            Ok(Event::Empty(e)) => {
                let mut local_ns = HashMap::new();
                for attr in e.attributes().filter_map(|a| a.ok()) {
                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                    let val = String::from_utf8_lossy(&attr.value).to_string();
                    if key == "xmlns" {
                        local_ns.insert(String::new(), NamespaceUri::new(&val));
                    } else if let Some(prefix) = key.strip_prefix("xmlns:") {
                        local_ns.insert(prefix.to_string(), NamespaceUri::new(&val));
                    }
                }

                let mut scope = ns_stack.last().cloned().unwrap_or_default();
                for (k, v) in &local_ns {
                    scope.insert(k.clone(), v.clone());
                }

                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let (ns_uri, local_name) = resolve_tag_name(&tag_name, &scope);

                let mut attrs = Vec::new();
                for attr in e.attributes().filter_map(|a| a.ok()) {
                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                    if key == "xmlns" || key.starts_with("xmlns:") {
                        continue;
                    }
                    let val = String::from_utf8_lossy(&attr.value).to_string();
                    attrs.push((key, val));
                }

                let node = MessageNode::Element {
                    namespace_uri: ns_uri,
                    local_name,
                    attributes: attrs,
                    children: vec![],
                };

                if let Some(parent) = node_stack.last_mut() {
                    parent.push(node);
                }
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                if !text.is_empty()
                    && let Some(parent) = node_stack.last_mut()
                {
                    parent.push(MessageNode::Text(text));
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow::anyhow!("XML parse error: {}", e)),
            _ => {}
        }
    }

    Ok(ParsedMessage {
        namespaces: top_level_namespaces,
        nodes: node_stack.into_iter().next().unwrap_or_default(),
    })
}

fn resolve_tag_name(tag: &str, ns_scope: &HashMap<String, NamespaceUri>) -> (NamespaceUri, String) {
    if let Some((prefix, local)) = tag.split_once(':') {
        let uri = ns_scope
            .get(prefix)
            .cloned()
            .unwrap_or_else(|| NamespaceUri::new(format!("unknown:{}", prefix)));
        (uri, local.to_string())
    } else {
        let uri = ns_scope
            .get("")
            .cloned()
            .unwrap_or_else(|| NamespaceUri::new(""));
        (uri, tag.to_string())
    }
}
