<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-grove/src/yrs_bridge.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-fac771a6dc39"></a>
## `crates/hick-grove/src/yrs_bridge.rs`

<a id="f-fac771a6dc39-YrsChangeEvent"></a>
### `YrsChangeEvent`

<storycode:copy id="f-fac771a6dc39-YrsChangeEvent">
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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

</storycode:copy>
<a id="f-fac771a6dc39-CachedElement"></a>
### `CachedElement`

Cached metadata for an element, populated on `Created` events.

<storycode:copy id="f-fac771a6dc39-CachedElement">
/// Cached metadata for an element, populated on `Created` events.
#[derive(Clone, Debug)]
struct CachedElement {
    namespace_uri: NamespaceUri,
    local_name: String,
    attributes: Vec<(String, String)>,
    element_id: Option<String>,
}

</storycode:copy>
<a id="f-fac771a6dc39-cache_key"></a>
### `cache_key`

Cache key: "qualified_tag:id_attr" (or just "qualified_tag:" if no id).

<storycode:copy id="f-fac771a6dc39-cache_key">
/// Cache key: "qualified_tag:id_attr" (or just "qualified_tag:" if no id).
fn cache_key(qualified_tag: &str, id: Option<&str>) -> String {
    format!("{}:{}", qualified_tag, id.unwrap_or(""))
}

</storycode:copy>
<a id="f-fac771a6dc39-ElementCache"></a>
### `ElementCache`

Shared element metadata cache used for removal detection.
Persists across `YrsBridge` instances so that removals in a later
transaction can reference metadata from earlier creations.

<storycode:copy id="f-fac771a6dc39-ElementCache">
/// Shared element metadata cache used for removal detection.
///
/// Persists across `YrsBridge` instances so that removals in a later
/// transaction can reference metadata from earlier creations.
#[derive(Clone)]
pub struct ElementCache(Arc<Mutex<HashMap<String, CachedElement>>>);

</storycode:copy>
<a id="f-fac771a6dc39-impl-ElementCache"></a>
### `impl-ElementCache`

<storycode:copy id="f-fac771a6dc39-impl-ElementCache">
impl ElementCache {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(HashMap::new())))
    }
}

</storycode:copy>
<a id="f-fac771a6dc39-impl-Default-for-ElementCache"></a>
### `impl-Default for ElementCache`

<storycode:copy id="f-fac771a6dc39-impl-Default-for-ElementCache">
impl Default for ElementCache {
    fn default() -> Self {
        Self::new()
    }
}

</storycode:copy>
<a id="f-fac771a6dc39-YrsBridge"></a>
### `YrsBridge`

Registers `observe_deep` on a Yrs document's root XmlFragment and pushes
namespace-resolved events into a tokio mpsc channel.

<storycode:copy id="f-fac771a6dc39-YrsBridge">
/// Registers `observe_deep` on a Yrs document's root XmlFragment and pushes
/// namespace-resolved events into a tokio mpsc channel.
pub struct YrsBridge {
    _subscription: yrs::Subscription,
}

</storycode:copy>
<a id="f-fac771a6dc39-impl-YrsBridge"></a>
### `impl-YrsBridge`

<storycode:copy id="f-fac771a6dc39-impl-YrsBridge">
impl YrsBridge {
    /// Attach to a Yrs document. Returns the bridge (holds the subscription alive)
    /// and a receiver for change events.
    ///
    /// `namespace_map` maps prefix -> NamespaceUri for resolving `prefix:localname` tags.
    pub fn attach(
        doc: &Doc,
        namespace_map: HashMap<String, NamespaceUri>,
        tx: mpsc::UnboundedSender<YrsChangeEvent>,
        element_cache: ElementCache,
    ) -> Self {
        let mut init_txn = doc.transact_mut();
        let root: XmlFragmentRef = init_txn.get_or_insert_xml_fragment("root");
        drop(init_txn);

        let ns_map = Arc::new(namespace_map);

        let sub = root.observe_deep(move |txn, events: &Events| {
            for event in events.iter() {
                let change_events = resolve_event(txn, event, &ns_map, &element_cache);
                for ce in change_events {
                    let _ = tx.send(ce);
                }
            }
        });

        Self { _subscription: sub }
    }
}

</storycode:copy>
<a id="f-fac771a6dc39-resolve_event"></a>
### `resolve_event`

Resolve a single Yrs deep-observe event into zero or more `YrsChangeEvent`s.

<storycode:copy id="f-fac771a6dc39-resolve_event">
/// Resolve a single Yrs deep-observe event into zero or more `YrsChangeEvent`s.
fn resolve_event(
    txn: &TransactionMut<'_>,
    event: &yrs::types::Event,
    namespace_map: &HashMap<String, NamespaceUri>,
    element_cache: &ElementCache,
) -> Vec<YrsChangeEvent> {
    use yrs::types::Event;

    let mut results = Vec::new();

    match event {
        Event::XmlFragment(xml_event) => {
            let target = xml_event.target();
            match target {
                XmlOut::Element(elem) => {
                    // Attribute changes on this element
                    let tag = elem.tag().to_string();
                    if let Some((ns_uri, local)) = resolve_tag(&tag, namespace_map) {
                        let element_id = read_id_attr(txn, elem);

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
                                            element_id: element_id.clone(),
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
                                            element_id: element_id.clone(),
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
                                            element_id: element_id.clone(),
                                        },
                                    });
                                }
                            }
                        }

                        // Child additions and removals
                        let mut has_removals = false;
                        for delta in xml_event.delta(txn) {
                            match delta {
                                yrs::types::Change::Added(values) => {
                                    for value in values {
                                        process_added_out(
                                            txn,
                                            value,
                                            namespace_map,
                                            element_cache,
                                            &mut results,
                                        );
                                    }
                                }
                                yrs::types::Change::Removed(_) => {
                                    has_removals = true;
                                }
                                _ => {}
                            }
                        }

                        if has_removals {
                            detect_removals(
                                txn,
                                ChildSource::Element(elem),
                                namespace_map,
                                element_cache,
                                &mut results,
                            );
                        }
                    }
                }
                XmlOut::Fragment(frag) => {
                    // Root fragment — child additions and removals
                    let mut has_removals = false;
                    for delta in xml_event.delta(txn) {
                        match delta {
                            yrs::types::Change::Added(values) => {
                                for value in values {
                                    process_added_out(
                                        txn,
                                        value,
                                        namespace_map,
                                        element_cache,
                                        &mut results,
                                    );
                                }
                            }
                            yrs::types::Change::Removed(_) => {
                                has_removals = true;
                            }
                            _ => {}
                        }
                    }

                    if has_removals {
                        detect_removals(
                            txn,
                            ChildSource::Fragment(frag),
                            namespace_map,
                            element_cache,
                            &mut results,
                        );
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
                    let element_id = read_id_attr(txn, &xml_elem);
                    let new_text = target.get_string(txn);
                    results.push(YrsChangeEvent {
                        namespace_uri: ns_uri,
                        event: ElementEvent::TextChanged {
                            local_name: local,
                            new_text,
                            element_id,
                        },
                    });
                }
            }
        }
        _ => {}
    }

    results
}

</storycode:copy>
<a id="f-fac771a6dc39-ChildSource"></a>
### `ChildSource`

Abstracts over fragment vs element for child enumeration.

<storycode:copy id="f-fac771a6dc39-ChildSource">
/// Abstracts over fragment vs element for child enumeration.
enum ChildSource<'a> {
    Fragment(&'a XmlFragmentRef),
    Element(&'a XmlElementRef),
}

</storycode:copy>
<a id="f-fac771a6dc39-detect_removals"></a>
### `detect_removals`

Detect removed children by diffing the element cache against current children.

<storycode:copy id="f-fac771a6dc39-detect_removals">
/// Detect removed children by diffing the element cache against current children.
fn detect_removals(
    txn: &TransactionMut<'_>,
    parent: ChildSource<'_>,
    namespace_map: &HashMap<String, NamespaceUri>,
    element_cache: &ElementCache,
    results: &mut Vec<YrsChangeEvent>,
) {
    // Build set of current child keys
    let mut current_keys = std::collections::HashSet::new();
    let child_count = match &parent {
        ChildSource::Fragment(frag) => XmlFragment::len(*frag, txn),
        ChildSource::Element(elem) => XmlFragment::len(*elem, txn),
    };

    for i in 0..child_count {
        let child = match &parent {
            ChildSource::Fragment(frag) => XmlFragment::get(*frag, txn, i),
            ChildSource::Element(elem) => XmlFragment::get(*elem, txn, i),
        };
        if let Some(XmlOut::Element(child_elem)) = child {
            let tag = child_elem.tag().to_string();
            let id = read_id_attr(txn, &child_elem);
            let key = cache_key(&tag, id.as_deref());
            current_keys.insert(key);
        }
    }

    // Diff against cache — anything NOT in current children was removed
    let mut cache = element_cache.0.lock().unwrap();
    let removed_keys: Vec<String> = cache
        .keys()
        .filter(|k| !current_keys.contains(k.as_str()))
        .cloned()
        .collect();

    for key in removed_keys {
        if let Some(cached) = cache.remove(&key) {
            // Verify this element's namespace is one we know about
            if namespace_map
                .values()
                .any(|uri| *uri == cached.namespace_uri)
            {
                results.push(YrsChangeEvent {
                    namespace_uri: cached.namespace_uri,
                    event: ElementEvent::Removed {
                        local_name: cached.local_name,
                        attributes: cached.attributes,
                        element_id: cached.element_id,
                    },
                });
            }
        }
    }
}

</storycode:copy>
<a id="f-fac771a6dc39-process_added_out"></a>
### `process_added_out`

Process an added child output from a delta. Recurses into children to
find namespace-qualified elements at any depth.

<storycode:copy id="f-fac771a6dc39-process_added_out">
/// Process an added child output from a delta. Recurses into children to
/// find namespace-qualified elements at any depth.
fn process_added_out(
    txn: &TransactionMut<'_>,
    value: &yrs::Out,
    namespace_map: &HashMap<String, NamespaceUri>,
    element_cache: &ElementCache,
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
            let element_id = attr_value(&attrs, "id");

            // Insert into cache
            {
                let key = cache_key(&tag, element_id.as_deref());
                let mut cache = element_cache.0.lock().unwrap();
                cache.insert(
                    key,
                    CachedElement {
                        namespace_uri: ns_uri.clone(),
                        local_name: local.clone(),
                        attributes: attrs.clone(),
                        element_id: element_id.clone(),
                    },
                );
            }

            results.push(YrsChangeEvent {
                namespace_uri: ns_uri,
                event: ElementEvent::Created {
                    local_name: local,
                    attributes: attrs,
                    text_content,
                    element_id,
                },
            });
        }

        // Recurse into children regardless of whether this element matched
        for i in 0..XmlFragment::len(elem, txn) {
            if let Some(XmlOut::Element(child)) = XmlFragment::get(elem, txn, i) {
                process_added_element(txn, &child, namespace_map, element_cache, results);
            }
        }
    }
}

</storycode:copy>
<a id="f-fac771a6dc39-process_added_element"></a>
### `process_added_element`

Process a nested element (already resolved to XmlElementRef).

<storycode:copy id="f-fac771a6dc39-process_added_element">
/// Process a nested element (already resolved to XmlElementRef).
fn process_added_element(
    txn: &TransactionMut<'_>,
    elem: &XmlElementRef,
    namespace_map: &HashMap<String, NamespaceUri>,
    element_cache: &ElementCache,
    results: &mut Vec<YrsChangeEvent>,
) {
    let tag = elem.tag().to_string();
    if let Some((ns_uri, local)) = resolve_tag(&tag, namespace_map) {
        let mut attrs = Vec::new();
        for (k, v) in elem.attributes(txn) {
            attrs.push((k.to_string(), v));
        }

        let text_content = collect_text(txn, elem);
        let element_id = attr_value(&attrs, "id");

        // Insert into cache
        {
            let key = cache_key(&tag, element_id.as_deref());
            let mut cache = element_cache.0.lock().unwrap();
            cache.insert(
                key,
                CachedElement {
                    namespace_uri: ns_uri.clone(),
                    local_name: local.clone(),
                    attributes: attrs.clone(),
                    element_id: element_id.clone(),
                },
            );
        }

        results.push(YrsChangeEvent {
            namespace_uri: ns_uri,
            event: ElementEvent::Created {
                local_name: local,
                attributes: attrs,
                text_content,
                element_id,
            },
        });
    }

    // Recurse into children
    for i in 0..XmlFragment::len(elem, txn) {
        if let Some(XmlOut::Element(child)) = XmlFragment::get(elem, txn, i) {
            process_added_element(txn, &child, namespace_map, element_cache, results);
        }
    }
}

</storycode:copy>
<a id="f-fac771a6dc39-read_id_attr"></a>
### `read_id_attr`

<storycode:copy id="f-fac771a6dc39-read_id_attr">
/// Read the `id` attribute from a Yrs element.
fn read_id_attr(txn: &TransactionMut<'_>, elem: &XmlElementRef) -> Option<String> {
    elem.get_attribute(txn, "id")
}

</storycode:copy>
<a id="f-fac771a6dc39-attr_value"></a>
### `attr_value`

Get an attribute value from a list of (key, value) pairs.

<storycode:copy id="f-fac771a6dc39-attr_value">
/// Get an attribute value from a list of (key, value) pairs.
fn attr_value(attrs: &[(String, String)], key: &str) -> Option<String> {
    attrs.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
}

</storycode:copy>
<a id="f-fac771a6dc39-collect_text"></a>
### `collect_text`

Collect all text content from an XmlElement (immediate text children).

<storycode:copy id="f-fac771a6dc39-collect_text">
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

</storycode:copy>
<a id="f-fac771a6dc39-resolve_tag"></a>
### `resolve_tag`

Resolve `prefix:localname` into `(NamespaceUri, localname)`.

<storycode:copy id="f-fac771a6dc39-resolve_tag">
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

</storycode:copy>
<a id="f-fac771a6dc39-out_to_string"></a>
### `out_to_string`

<storycode:copy id="f-fac771a6dc39-out_to_string">
fn out_to_string(val: &yrs::Out) -> String {
    match val {
        yrs::Out::Any(any) => any_to_string(any),
        _ => format!("{:?}", val),
    }
}

</storycode:copy>
<a id="f-fac771a6dc39-any_to_string"></a>
### `any_to_string`

<storycode:copy id="f-fac771a6dc39-any_to_string">
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
</storycode:copy>

### Assemble `crates/hick-grove/src/yrs_bridge.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-grove/src/yrs_bridge.rs"><storycode:paste select="#f-fac771a6dc39-YrsChangeEvent"/><storycode:paste select="#f-fac771a6dc39-CachedElement"/><storycode:paste select="#f-fac771a6dc39-cache_key"/><storycode:paste select="#f-fac771a6dc39-ElementCache"/><storycode:paste select="#f-fac771a6dc39-impl-ElementCache"/><storycode:paste select="#f-fac771a6dc39-impl-Default-for-ElementCache"/><storycode:paste select="#f-fac771a6dc39-YrsBridge"/><storycode:paste select="#f-fac771a6dc39-impl-YrsBridge"/><storycode:paste select="#f-fac771a6dc39-resolve_event"/><storycode:paste select="#f-fac771a6dc39-ChildSource"/><storycode:paste select="#f-fac771a6dc39-detect_removals"/><storycode:paste select="#f-fac771a6dc39-process_added_out"/><storycode:paste select="#f-fac771a6dc39-process_added_element"/><storycode:paste select="#f-fac771a6dc39-read_id_attr"/><storycode:paste select="#f-fac771a6dc39-attr_value"/><storycode:paste select="#f-fac771a6dc39-collect_text"/><storycode:paste select="#f-fac771a6dc39-resolve_tag"/><storycode:paste select="#f-fac771a6dc39-out_to_string"/><storycode:paste select="#f-fac771a6dc39-any_to_string"/></storycode:file>

</storycode:doc>
