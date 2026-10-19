//! Context v1 validation only. Python owns selection and refresh policy.
//! This crate performs no capture, persistence, provider delivery or authorization.
mod types;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
pub use types::*;
use unicode_normalization::UnicodeNormalization;

pub const MAX_FRAME: usize = 1_048_576;
pub const MAX_PAGE: u64 = 256;
pub const SCHEMA: &str = include_str!("../../../protocols/context-v1/schema.json");
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    pub code: ErrorCode,
    pub message: String,
}
impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}
impl std::error::Error for Failure {}
pub type Result<T> = std::result::Result<T, Failure>;
fn require(ok: bool, code: ErrorCode, message: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(Failure {
            code,
            message: message.into(),
        })
    }
}
fn list(value: &Value) -> &[Value] {
    value.as_array().map(Vec::as_slice).unwrap_or(&[])
}
fn has(value: &Value, key: &str) -> bool {
    value.get(key).is_some()
}
fn num(value: &Value) -> u64 {
    value.as_u64().unwrap_or(0)
}
fn text(value: &Value) -> &str {
    value.as_str().unwrap_or("")
}
/// Validate any generated wire type by its schema definition name.
pub fn validate(value: &Value, name: &str) -> Result<()> {
    let mut schema: Value = serde_json::from_str(SCHEMA).expect("checked-in schema");
    require(
        schema["$defs"].get(name).is_some(),
        ErrorCode::InvalidSchema,
        "Unknown schema type",
    )?;
    schema["$ref"] = json!(format!("#/$defs/{name}"));
    let validator = jsonschema::validator_for(&schema).expect("valid checked-in schema");
    require(
        validator.is_valid(value) && types::deserializes(name, value),
        ErrorCode::InvalidSchema,
        "Schema validation failed",
    )
}
pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
/// Request objects have only integers, strings, arrays and records. serde_json's
/// ordered map encoding matches Python's sorted, compact, UTF-8 JSON encoding.
pub fn assembly_digest(request: &Value) -> String {
    digest(&serde_json::to_vec(request).expect("JSON value"))
}
fn span_bytes<'a>(s: &'a str, span: &Value) -> Result<&'a [u8]> {
    let (a, b) = (num(&span["start"]), num(&span["end"]));
    require(
        a <= b && b <= s.len() as u64,
        ErrorCode::InvalidOffset,
        "Invalid UTF-8 byte range",
    )?;
    require(
        s.is_char_boundary(a as usize) && s.is_char_boundary(b as usize),
        ErrorCode::InvalidOffset,
        "Split UTF-8 code point",
    )?;
    Ok(&s.as_bytes()[a as usize..b as usize])
}
pub fn utf16_to_utf8(s: &str, offset: u64) -> Result<usize> {
    let mut units = 0;
    for (byte, c) in s.char_indices() {
        if units == offset {
            return Ok(byte);
        }
        units += c.len_utf16() as u64;
    }
    require(
        units == offset,
        ErrorCode::InvalidOffset,
        "Split surrogate pair or out of range",
    )?;
    Ok(s.len())
}
pub fn negotiate(host: &Value, worker: &Value) -> Result<Negotiated> {
    validate(host, "Hello")?;
    validate(worker, "Hello")?;
    require(
        list(&host["versions"]).contains(&json!(1))
            && list(&worker["versions"]).contains(&json!(1)),
        ErrorCode::VersionSkew,
        "No context v1",
    )?;
    let mut capabilities: Vec<Value> = list(&host["capabilities"])
        .iter()
        .filter(|c| list(&worker["capabilities"]).contains(c))
        .cloned()
        .collect();
    capabilities.sort_by(|a, b| text(a).cmp(text(b)));
    capabilities.dedup();
    require(
        capabilities.contains(&json!("context.changes")),
        ErrorCode::CapabilityMissing,
        "Polling required",
    )?;
    let frame = num(&host["max_frame_bytes"])
        .min(num(&worker["max_frame_bytes"]))
        .min(MAX_FRAME as u64);
    let page = num(&host["max_page_items"])
        .min(num(&worker["max_page_items"]))
        .min(MAX_PAGE);
    require(
        frame > 0 && page > 0,
        ErrorCode::FrameLimit,
        "Zero negotiated limit",
    )?;
    Ok(serde_json::from_value(json!({"version":1,"capabilities":capabilities,"transport":"cursor_polling","max_frame_bytes":frame,"max_page_items":page})).expect("negotiated type"))
}
pub fn validate_frame(raw: &[u8], limit: usize, expected_id: Option<&Value>) -> Result<Value> {
    require(
        raw.len() <= limit.min(MAX_FRAME)
            && raw.ends_with(b"\n")
            && !raw[..raw.len().saturating_sub(1)].contains(&b'\n'),
        ErrorCode::FrameLimit,
        "One bounded newline-terminated JSON frame required",
    )?;
    let value: Value = serde_json::from_slice(raw).map_err(|_| Failure {
        code: ErrorCode::InvalidSchema,
        message: "Invalid JSON frame".into(),
    })?;
    require(
        value.is_object(),
        ErrorCode::InvalidSchema,
        "Frame must be object",
    )?;
    if let Some(id) = expected_id {
        require(
            value.get("id") == Some(id) && (has(&value, "result") != has(&value, "error")),
            ErrorCode::MismatchedResponse,
            "Unsolicited or mismatched response",
        )?;
    }
    Ok(value)
}
pub fn validate_page(page: &Value, epoch: &str, branch: &str) -> Result<ChangePage> {
    validate(page, "ChangePage")?;
    let (a, b) = (&page["from_cursor"], &page["next_cursor"]);
    require(
        text(&a["epoch"]) == epoch
            && text(&b["epoch"]) == epoch
            && text(&a["branch"]) == branch
            && text(&b["branch"]) == branch,
        ErrorCode::CursorExpired,
        "Reconcile after epoch/branch change",
    )?;
    let events = list(&page["events"]);
    require(
        num(&page["limit"]) > 0
            && num(&page["limit"]) <= MAX_PAGE
            && events.len() as u64 <= num(&page["limit"]),
        ErrorCode::InvalidPage,
        "Invalid page limit",
    )?;
    let mut sequence = num(&a["position"]);
    for event in events {
        require(
            num(&event["sequence"]) > sequence,
            ErrorCode::InvalidPage,
            "Events must advance in order",
        )?;
        sequence = num(&event["sequence"]);
    }
    require(
        num(&b["position"]) == sequence && (page["has_more"] != true || !events.is_empty()),
        ErrorCode::InvalidPage,
        "Cursor must describe last event",
    )?;
    Ok(serde_json::from_value(page.clone()).expect("validated page"))
}
pub fn validate_handle(handle: &Value) -> Result<()> {
    validate(handle, "Handle")?;
    require(
        num(&handle["span"]["start"]) <= num(&handle["span"]["end"]),
        ErrorCode::InvalidOffset,
        "Invalid handle span",
    )?;
    let required: &[&str] = match text(&handle["kind"]) {
        "stop" => &["run", "stop"],
        "frame" => &["run", "stop", "frame"],
        "invocation" => &["run", "invocation"],
        _ => &[],
    };
    require(
        required.iter().all(|k| has(handle, k)),
        ErrorCode::InvalidSchema,
        "Missing handle execution scope",
    )
}
pub fn resolve_handle(handle: &Value, request: &Value) -> Result<Handle> {
    validate_handle(handle)?;
    validate(request, "ResolveRequest")?;
    require(
        request["handle"] == handle["id"]
            && ["id", "revision"]
                .iter()
                .all(|k| request["expected_source"][k] == handle["source"][k])
            && ["run", "stop", "frame", "invocation"]
                .iter()
                .all(|k| request.get(k) == handle.get(k)),
        ErrorCode::StaleHandle,
        "Rust must resolve current source and execution scope",
    )?;
    Ok(serde_json::from_value(handle.clone()).expect("validated handle"))
}
#[derive(Default)]
pub struct Idempotency {
    entries: HashMap<String, String>,
}
impl Idempotency {
    /// True means new; false means replay. Persist with session receipts in #49/50.
    pub fn register(&mut self, key: &str, fingerprint: &str) -> Result<bool> {
        if let Some(old) = self.entries.get(key) {
            require(
                old == fingerprint,
                ErrorCode::IdempotencyConflict,
                "Key reused for different bytes",
            )?;
            return Ok(false);
        }
        self.entries.insert(key.into(), fingerprint.into());
        Ok(true)
    }
}
/// Validate a self-contained, topologically ordered root-to-request witness.
/// External artifact bytes are not fetched; only their immutable descriptors
/// are checked. This cannot establish authenticity or semantic support.
pub fn validate_bundle(value: &Value) -> Result<Bundle> {
    require(
        value["version"] == 1,
        ErrorCode::VersionSkew,
        "Unsupported context version",
    )?;
    validate(value, "Bundle")?;
    let mut nodes: HashMap<&str, &Value> = HashMap::new();
    for node in list(&value["nodes"]) {
        require(
            !nodes.contains_key(text(&node["id"])),
            ErrorCode::InvalidSchema,
            "Duplicate immutable reference",
        )?;
        require(
            has(node, "text") != has(node, "artifact"),
            ErrorCode::InvalidSchema,
            "Exactly one content representation",
        )?;
        for parent in list(&node["parents"]) {
            require(
                nodes.contains_key(text(parent)),
                ErrorCode::MissingReference,
                "Parents precede children; no cycles",
            )?;
        }
        if has(node, "text") {
            require(
                digest(text(&node["text"]).as_bytes()) == text(&node["digest"]),
                ErrorCode::DigestMismatch,
                "Modified node bytes",
            )?;
        } else {
            require(
                node["digest"] == node["artifact"]["digest"],
                ErrorCode::DigestMismatch,
                "Artifact digest mismatch",
            )?;
        }
        if has(node, "source_span") {
            let span = &node["source_span"];
            let length = if has(node, "text") {
                text(&node["text"]).len() as u64
            } else {
                num(&node["artifact"]["byte_length"])
            };
            require(
                has(node, "source")
                    && num(&span["end"]) >= num(&span["start"])
                    && num(&span["end"]) - num(&span["start"]) == length,
                ErrorCode::InvalidOffset,
                "Source excerpt length mismatch",
            )?;
        }
        let kind = text(&node["derivation"]);
        let exact = matches!(kind, "selection" | "normalization" | "concatenation");
        match kind {
            "root" => require(
                list(&node["parents"]).is_empty() && list(&node["mappings"]).is_empty(),
                ErrorCode::InvalidMapping,
                "Root has no derivation",
            )?,
            "opaque" => require(
                node["coverage"] == "opaque" && list(&node["mappings"]).is_empty(),
                ErrorCode::InvalidMapping,
                "Opaque coverage required",
            )?,
            _ => require(
                !list(&node["parents"]).is_empty(),
                ErrorCode::MissingReference,
                "Derivation needs genealogy",
            )?,
        }
        if matches!(kind, "summary" | "model_output" | "tool_output") {
            require(
                has(node, "receipt")
                    && node["coverage"] != "complete"
                    && list(&node["mappings"]).is_empty(),
                ErrorCode::InvalidMapping,
                "Genealogy is not exact or semantic support",
            )?;
        }
        if !exact {
            require(
                list(&node["mappings"]).is_empty(),
                ErrorCode::InvalidMapping,
                "Only verified transformations carry mappings",
            )?;
            if matches!(kind, "redaction" | "rendering") {
                require(
                    node["coverage"] != "complete",
                    ErrorCode::InvalidMapping,
                    "Unverified transformation",
                )?;
            }
        }
        let mut end = 0;
        for mapping in list(&node["mappings"]) {
            require(
                list(&node["parents"]).contains(&mapping["parent"]),
                ErrorCode::MissingReference,
                "Mapping needs declared parent",
            )?;
            let parent = nodes[text(&mapping["parent"])];
            require(
                has(parent, "text") && has(node, "text"),
                ErrorCode::InvalidMapping,
                "Text mapping required",
            )?;
            let mut left = span_bytes(text(&parent["text"]), &mapping["input"])?.to_vec();
            let right = span_bytes(text(&node["text"]), &mapping["output"])?;
            if kind == "normalization" {
                left = std::str::from_utf8(&left)
                    .expect("validated UTF-8")
                    .replace("\r\n", "\n")
                    .nfc()
                    .collect::<String>()
                    .into_bytes();
            }
            require(
                left == right && num(&mapping["output"]["start"]) == end,
                ErrorCode::InvalidMapping,
                "Transformation bytes or ordered coverage mismatch",
            )?;
            end = num(&mapping["output"]["end"]);
        }
        if exact {
            require(
                has(node, "text")
                    && end == text(&node["text"]).len() as u64
                    && node["coverage"] == "complete",
                ErrorCode::InvalidMapping,
                "Exact derivation covers every output byte",
            )?;
        }
        nodes.insert(text(&node["id"]), node);
    }
    let mut productions = HashMap::new();
    for receipt in list(&value["productions"]) {
        require(
            !productions.contains_key(text(&receipt["id"])),
            ErrorCode::InvalidSchema,
            "Duplicate production receipt",
        )?;
        require(
            nodes.contains_key(text(&receipt["output"]))
                && nodes.contains_key(text(&receipt["response"]))
                && list(&receipt["inputs"])
                    .iter()
                    .all(|p| nodes.contains_key(text(p))),
            ErrorCode::MissingReference,
            "Unknown production reference",
        )?;
        let output = nodes[text(&receipt["output"])];
        require(
            output["receipt"] == receipt["id"]
                && output["parents"] == receipt["inputs"]
                && receipt["kind"]
                    == if output["derivation"] == "tool_output" {
                        "tool"
                    } else {
                        "model"
                    },
            ErrorCode::InvalidMapping,
            "Producer and declared genealogy differ",
        )?;
        productions.insert(text(&receipt["id"]), receipt);
    }
    for node in nodes.values() {
        if matches!(
            text(&node["derivation"]),
            "summary" | "model_output" | "tool_output"
        ) {
            require(
                productions
                    .get(text(&node["receipt"]))
                    .is_some_and(|r| r["output"] == node["id"]),
                ErrorCode::MissingReference,
                "Missing typed production receipt",
            )?;
        }
    }
    let request = &value["request"];
    let mut slots = HashSet::new();
    for part in list(&request["parts"]) {
        require(
            slots.insert(text(&part["slot"])),
            ErrorCode::InvalidSchema,
            "Duplicate request slot",
        )?;
        require(
            has(part, "text") != has(part, "artifact"),
            ErrorCode::InvalidSchema,
            "Exactly one request content representation",
        )?;
        let mut end = 0;
        for segment in list(&part["segments"]) {
            let node = nodes.get(text(&segment["node"])).ok_or_else(|| Failure {
                code: ErrorCode::MissingReference,
                message: "Unknown request reference".into(),
            })?;
            require(
                num(&segment["span"]["start"]) == end,
                ErrorCode::UncoveredBytes,
                "Coverage must be contiguous and ordered",
            )?;
            if has(part, "text") && has(node, "text") {
                require(
                    span_bytes(text(&part["text"]), &segment["span"])?
                        == span_bytes(text(&node["text"]), &segment["node_span"])?,
                    ErrorCode::ModifiedBytes,
                    "Request bytes differ from reference",
                )?;
            } else {
                require(
                    has(part, "artifact")
                        && has(node, "artifact")
                        && part["artifact"] == node["artifact"],
                    ErrorCode::ModifiedBytes,
                    "Artifact descriptor differs",
                )?;
                for span in [&segment["span"], &segment["node_span"]] {
                    require(
                        num(&span["start"]) == 0 && span["end"] == node["artifact"]["byte_length"],
                        ErrorCode::InvalidOffset,
                        "Artifact reference covers entire artifact",
                    )?;
                }
            }
            end = num(&segment["span"]["end"]);
        }
        let length = if has(part, "text") {
            text(&part["text"]).len() as u64
        } else {
            num(&part["artifact"]["byte_length"])
        };
        require(
            end == length,
            ErrorCode::UncoveredBytes,
            "Every host-controlled byte needs lineage",
        )?;
    }
    let mut captures = HashSet::new();
    for receipt in list(&value["captures"]) {
        require(
            nodes.contains_key(text(&receipt["node"])),
            ErrorCode::MissingReference,
            "Unknown capture reference",
        )?;
        require(
            captures.insert(text(&receipt["id"])),
            ErrorCode::InvalidSchema,
            "Duplicate capture receipt",
        )?;
    }
    let mut states = HashMap::new();
    let mut last_receipts = HashMap::new();
    let mut keys = Idempotency::default();
    for receipt in list(&value["deliveries"]) {
        require(
            receipt["request"] == request["id"]
                && text(&receipt["assembly_digest"]) == assembly_digest(request),
            ErrorCode::ModifiedBytes,
            "Receipt must bind exact request assembly",
        )?;
        let key = text(&receipt["idempotency_key"]);
        keys.register(
            key,
            &format!(
                "{}:{}",
                text(&receipt["assembly_digest"]),
                text(&receipt["attempt"])
            ),
        )?;
        let old = states.get(key).copied();
        let state = text(&receipt["state"]);
        let allowed = match old {
            None => state == "prepared",
            Some("prepared") => matches!(
                state,
                "prepared" | "transport_started" | "failed" | "cancelled_uncertain"
            ),
            Some("transport_started") => matches!(
                state,
                "transport_started" | "provider_confirmed" | "failed" | "cancelled_uncertain"
            ),
            Some(previous) => state == previous,
        };
        require(
            allowed,
            ErrorCode::InvalidDelivery,
            "Invalid delivery transition",
        )?;
        require(
            old != Some(state) || last_receipts.get(key).copied() == Some(receipt),
            ErrorCode::InvalidDelivery,
            "Receipt replay must preserve every field",
        )?;
        require(
            has(receipt, "provider_receipt") == (state == "provider_confirmed"),
            ErrorCode::InvalidDelivery,
            "Provider confirmation needs observable receipt",
        )?;
        states.insert(key, state);
        last_receipts.insert(key, receipt);
    }
    let mut handles = HashMap::new();
    for handle in list(&value["handles"]) {
        require(
            !handles.contains_key(text(&handle["id"])),
            ErrorCode::InvalidSchema,
            "Duplicate handle",
        )?;
        validate_handle(handle)?;
        require(
            nodes.values().any(|n| {
                n["source"]["id"] == handle["source"]["id"]
                    && n["source"]["revision"] == handle["source"]["revision"]
            }),
            ErrorCode::MissingReference,
            "Handle source revision must be retained",
        )?;
        handles.insert(text(&handle["id"]), handle);
    }
    for annotation in list(&value["annotations"]) {
        require(
            nodes.contains_key(text(&annotation["evidence"]))
                && list(&annotation["parents"])
                    .iter()
                    .all(|p| nodes.contains_key(text(p))),
            ErrorCode::MissingReference,
            "Annotation evidence is immutable context",
        )?;
        if has(annotation, "location") {
            require(
                handles.contains_key(text(&annotation["location"])),
                ErrorCode::MissingReference,
                "Unknown typed location handle",
            )?;
            let bound = handles[text(&annotation["location"])];
            require(
                ["run", "stop", "frame", "invocation"]
                    .iter()
                    .all(|k| !has(bound, k) || annotation.get(k) == bound.get(k)),
                ErrorCode::StaleHandle,
                "Annotation and location execution scopes differ",
            )?;
        }
        if matches!(
            text(&annotation["kind"]),
            "paused" | "values" | "expression"
        ) {
            require(
                ["run", "stop", "frame", "invocation", "location"]
                    .iter()
                    .all(|k| has(annotation, k)),
                ErrorCode::InvalidSchema,
                "Debugger annotation needs captured execution scope",
            )?;
        }
        if annotation["kind"] == "profile" {
            require(
                has(annotation, "metric") && has(annotation, "run"),
                ErrorCode::InvalidSchema,
                "Profile needs measurement semantics",
            )?;
        }
    }
    Ok(serde_json::from_value(value.clone()).expect("validated bundle matches generated types"))
}
