use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::mpsc;
use yrs::types::Events;
use yrs::{
    DeepObservable, Doc, GetString, Transact, TransactionMut, WriteTxn, Xml, XmlElementRef,
    XmlFragment, XmlFragmentRef, XmlOut, XmlTextRef,
};

use crate::namespace::NamespaceUri;
use crate::plugin::ElementEvent;

/// An event produced by the Yrs bridge, tagged with its namespace.
#[derive(Clone, Debug)]
pub struct YrsChangeEvent {
    pub namespace_uri: NamespaceUri,
    pub event: ElementEvent,
}

/// Registers `observe_deep` on a Yrs document's root XmlFragment and pushes
/// namespace-resolved events into a tokio mpsc channel.
pub struct YrsBridge {
    _subscription: yrs::Subscription,
}

impl YrsBridge {
    /// Attach to a Yrs document. Returns the bridge (holds the subscription alive)
    /// and a receiver for change events.
    ///
    /// `namespace_map` maps prefix -> NamespaceUri for resolving `prefix:localname` tags.
    pub fn attach(
        doc: &Doc,
        namespace_map: HashMap<String, NamespaceUri>,
        tx: mpsc::UnboundedSender<YrsChangeEvent>,
    ) -> Self {
        let mut init_txn = doc.transact_mut();
        let root: XmlFragmentRef = init_txn.get_or_insert_xml_fragment("root");
        drop(init_txn);

        let ns_map = Arc::new(namespace_map);

        let sub = root.observe_deep(move |txn, events: &Events| {
            for event in events.iter() {
                let change_events = resolve_event(txn, event, &ns_map);
                for ce in change_events {
                    let _ = tx.send(ce);
                }
            }
        });

        Self { _subscription: sub }
    }
}

/// Resolve a single Yrs deep-observe event into zero or more `YrsChangeEvent`s.
fn resolve_event(
    txn: &TransactionMut<'_>,
    event: &yrs::types::Event,
    namespace_map: &HashMap<String, NamespaceUri>,
) -> Vec<YrsChangeEvent> {
    use yrs::types::Event;

    let mut results = Vec::new();

    match event {
        Event::XmlFragment(xml_event) => {
            // XmlEvent fires for both XmlFragmentRef and XmlElementRef targets.
            let target = xml_event.target();
            match target {
                XmlOut::Element(elem) => {
                    // Attribute changes on this element
                    let tag = elem.tag().to_string();
                    if let Some((ns_uri, local)) = resolve_tag(&tag, namespace_map) {
                        let keys = xml_event.keys(txn);
                        for (attr_name, change) in keys.iter() {
                            match change {
                                yrs::types::EntryChange::Inserted(new_val) => {
                                    results.push(YrsChangeEvent {
                                        namespace_uri: ns_uri.clone(),
                                        event: ElementEvent::AttributeChanged {
                                            local_name: local.clone(),
                                            attr_name: attr_name.to_string(),
                                            old: None,
                                            new: Some(out_to_string(new_val)),
                                        },
                                    });
                                }
                                yrs::types::EntryChange::Updated(old_val, new_val) => {
                                    results.push(YrsChangeEvent {
                                        namespace_uri: ns_uri.clone(),
                                        event: ElementEvent::AttributeChanged {
                                            local_name: local.clone(),
                                            attr_name: attr_name.to_string(),
                                            old: Some(out_to_string(old_val)),
                                            new: Some(out_to_string(new_val)),
                                        },
                                    });
                                }
                                yrs::types::EntryChange::Removed(old_val) => {
                                    results.push(YrsChangeEvent {
                                        namespace_uri: ns_uri.clone(),
                                        event: ElementEvent::AttributeChanged {
                                            local_name: local.clone(),
                                            attr_name: attr_name.to_string(),
                                            old: Some(out_to_string(old_val)),
                                            new: None,
                                        },
                                    });
                                }
                            }
                        }

                        // Child additions (new elements inserted under this element)
                        for delta in xml_event.delta(txn) {
                            if let yrs::types::Change::Added(values) = delta {
                                for value in values {
                                    process_added_out(txn, value, namespace_map, &mut results);
                                }
                            }
                        }
                    }
                }
                XmlOut::Fragment(_frag) => {
                    // Root fragment — child additions
                    for delta in xml_event.delta(txn) {
                        if let yrs::types::Change::Added(values) = delta {
                            for value in values {
                                process_added_out(txn, value, namespace_map, &mut results);
                            }
                        }
                    }
                }
                XmlOut::Text(_) => {}
            }
        }
        Event::XmlText(text_event) => {
            let target: &XmlTextRef = text_event.target();
            if let Some(parent) = Xml::parent(target)
                && let Some(xml_elem) = parent.into_xml_element()
            {
                let tag = xml_elem.tag().to_string();
                if let Some((ns_uri, local)) = resolve_tag(&tag, namespace_map) {
                    let new_text = target.get_string(txn);
                    results.push(YrsChangeEvent {
                        namespace_uri: ns_uri,
                        event: ElementEvent::TextChanged {
                            local_name: local,
                            new_text,
                        },
                    });
                }
            }
        }
        _ => {}
    }

    results
}

/// Process an added child output from a delta. Recurses into children to
/// find namespace-qualified elements at any depth.
fn process_added_out(
    txn: &TransactionMut<'_>,
    value: &yrs::Out,
    namespace_map: &HashMap<String, NamespaceUri>,
    results: &mut Vec<YrsChangeEvent>,
) {
    if let yrs::Out::YXmlElement(elem) = value {
        let tag = elem.tag().to_string();
        if let Some((ns_uri, local)) = resolve_tag(&tag, namespace_map) {
            let mut attrs = Vec::new();
            for (k, v) in elem.attributes(txn) {
                attrs.push((k.to_string(), v));
            }

            let text_content = collect_text(txn, elem);

            results.push(YrsChangeEvent {
                namespace_uri: ns_uri,
                event: ElementEvent::Created {
                    local_name: local,
                    attributes: attrs,
                    text_content,
                },
            });
        }

        // Recurse into children regardless of whether this element matched
        for i in 0..XmlFragment::len(elem, txn) {
            if let Some(XmlOut::Element(child)) = XmlFragment::get(elem, txn, i) {
                process_added_element(txn, &child, namespace_map, results);
            }
        }
    }
}

/// Process a nested element (already resolved to XmlElementRef).
fn process_added_element(
    txn: &TransactionMut<'_>,
    elem: &XmlElementRef,
    namespace_map: &HashMap<String, NamespaceUri>,
    results: &mut Vec<YrsChangeEvent>,
) {
    let tag = elem.tag().to_string();
    if let Some((ns_uri, local)) = resolve_tag(&tag, namespace_map) {
        let mut attrs = Vec::new();
        for (k, v) in elem.attributes(txn) {
            attrs.push((k.to_string(), v));
        }

        let text_content = collect_text(txn, elem);

        results.push(YrsChangeEvent {
            namespace_uri: ns_uri,
            event: ElementEvent::Created {
                local_name: local,
                attributes: attrs,
                text_content,
            },
        });
    }

    // Recurse into children
    for i in 0..XmlFragment::len(elem, txn) {
        if let Some(XmlOut::Element(child)) = XmlFragment::get(elem, txn, i) {
            process_added_element(txn, &child, namespace_map, results);
        }
    }
}

/// Collect all text content from an XmlElement (immediate text children).
fn collect_text(txn: &TransactionMut<'_>, elem: &XmlElementRef) -> String {
    let mut text = String::new();
    for i in 0..XmlFragment::len(elem, txn) {
        if let Some(XmlOut::Text(t)) = XmlFragment::get(elem, txn, i) {
            text.push_str(&t.get_string(txn));
        }
    }
    text
}

/// Resolve `prefix:localname` into `(NamespaceUri, localname)`.
fn resolve_tag(
    tag: &str,
    namespace_map: &HashMap<String, NamespaceUri>,
) -> Option<(NamespaceUri, String)> {
    if let Some((prefix, local)) = tag.split_once(':') {
        namespace_map
            .get(prefix)
            .map(|uri| (uri.clone(), local.to_string()))
    } else {
        namespace_map
            .get("")
            .map(|uri| (uri.clone(), tag.to_string()))
    }
}

fn out_to_string(val: &yrs::Out) -> String {
    match val {
        yrs::Out::Any(any) => any_to_string(any),
        _ => format!("{:?}", val),
    }
}

fn any_to_string(val: &yrs::Any) -> String {
    match val {
        yrs::Any::String(s) => s.to_string(),
        yrs::Any::Number(n) => n.to_string(),
        yrs::Any::BigInt(n) => n.to_string(),
        yrs::Any::Bool(b) => b.to_string(),
        yrs::Any::Null | yrs::Any::Undefined => String::new(),
        other => format!("{:?}", other),
    }
}
