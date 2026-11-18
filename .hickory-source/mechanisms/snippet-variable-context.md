<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Capture variable values once from the intended source

Completion and reusable templates share one variable context. The source, scalar selection, authored path and workspace are explicit inputs; providers do not supply those values. A snapshot captures the requesting editor’s explicit clock and OS-backed UUID seed once for an expansion so mirrors retain identical values. A remote engine never substitutes its local timezone for the editor’s missing clock; unavailable time variables remain recognized but unset. Paths name the authored source and workspace in the engine filesystem, rather than pretending to be device-local paths. Clipboard remains a recognized unset variable unless the host explicitly supplies it. A missing seed leaves random variables unset and does not block other context readings. The host retains this immutable snapshot through preview and revalidation, so time and random values cannot change between otherwise identical plans.

Only requested source/clipboard variables are materialized. An excessive requested value refuses the entire expansion; an unused long source line cannot block a file or unrelated template. Byte selection endpoints must be scalar boundaries outside CRLF. Host leases and the engine’s source revision still authorize application separately.

## Bound the shared inputs before resolving values

The known-name catalogue and one-mebibyte value limit are common policy. Keep source/path inputs separate from host clock, seed and clipboard: recognizing a variable does not authorize a device read.

<ui:copy id="snippet-variable-context-engine">
//! Explicit source-bound inputs for completion and reusable template variables.
use chrono::{DateTime, Datelike, FixedOffset};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
const MAX_VALUE: usize = 1_048_576;
const KNOWN: &[&str] = &[
    "TM_SELECTED_TEXT",
    "TM_CURRENT_LINE",
    "TM_CURRENT_WORD",
    "TM_LINE_INDEX",
    "TM_LINE_NUMBER",
    "TM_FILENAME",
    "TM_FILENAME_BASE",
    "TM_DIRECTORY",
    "TM_FILEPATH",
    "RELATIVE_FILEPATH",
    "WORKSPACE_NAME",
    "WORKSPACE_FOLDER",
    "CLIPBOARD",
    "CURRENT_YEAR",
    "CURRENT_YEAR_SHORT",
    "CURRENT_MONTH",
    "CURRENT_MONTH_NAME",
    "CURRENT_MONTH_NAME_SHORT",
    "CURRENT_DATE",
    "CURRENT_DAY_NAME",
    "CURRENT_DAY_NAME_SHORT",
    "CURRENT_HOUR",
    "CURRENT_MINUTE",
    "CURRENT_SECOND",
    "CURRENT_SECONDS_UNIX",
    "CURRENT_TIMEZONE_OFFSET",
    "RANDOM",
    "RANDOM_HEX",
    "UUID",
    "BLOCK_COMMENT_START",
    "BLOCK_COMMENT_END",
    "LINE_COMMENT",
];
pub struct SnippetVariableContext<'a> {
    pub source: &'a str,
    pub from: usize,
    pub to: usize,
    pub path: &'a Path,
    pub workspace: Option<&'a Path>,
    pub language: Option<&'a str>,
}
#[derive(Clone, Debug)]
pub struct SnippetVariableSnapshot {
    pub date: Option<DateTime<FixedOffset>>,
    pub random: Option<[u8; 16]>,
    /// None means unattempted; Some(None) records an unavailable/denied read.
    pub clipboard: Option<Option<String>>,
}
</ui:copy>

## Admit one immutable host snapshot

Validate the supplied clock, seed and clipboard shapes once. Missing values remain recognized but unset, preserving defaults; the engine never discovers its own local clock or silently truncates a supplied value.

<ui:copy id="snippet-context-snapshot-admission">
impl SnippetVariableSnapshot {
    pub fn read(clock: &Value, random_hex: &Value, clipboard: &Value) -> Result<Self, String> {
        let date = if clock.is_null() {
            None
        } else {
            let object = clock.as_object().ok_or("Malformed editor snippet clock")?;
            if object.len() != 2
                || !object.contains_key("unix_millis")
                || !object.contains_key("utc_offset_minutes")
            {
                return Err("Unsupported editor clock fields".into());
            }
            let millis = clock["unix_millis"]
                .as_i64()
                .ok_or("Malformed editor snippet clock")?;
            let minutes = clock["utc_offset_minutes"]
                .as_i64()
                .filter(|v| (-840..=840).contains(v))
                .ok_or("Unsupported editor timezone offset")?;
            let offset =
                FixedOffset::east_opt(minutes as i32 * 60).ok_or("Invalid editor timezone")?;
            let date = DateTime::from_timestamp_millis(millis)
                .ok_or("Invalid editor clock")?
                .with_timezone(&offset);
            if !(1..=9999).contains(&date.year()) {
                return Err("Editor clock year outside supported range".into());
            }
            Some(date)
        };
        let random = if random_hex.is_null() {
            None
        } else {
            let text = random_hex
                .as_str()
                .filter(|text| text.len() == 32 && text.bytes().all(|c| c.is_ascii_hexdigit()))
                .ok_or("Malformed editor random snapshot")?;
            let mut bytes = [0; 16];
            for (index, slot) in bytes.iter_mut().enumerate() {
                *slot = u8::from_str_radix(&text[index * 2..index * 2 + 2], 16)
                    .map_err(|_| "Malformed editor random snapshot")?
            }
            Some(bytes)
        };
        let clipboard = if clipboard.is_null() {
            None
        } else {
            let object = clipboard.as_object().ok_or("Malformed clipboard context")?;
            if object.len() != 2
                || !object.contains_key("available")
                || !object.contains_key("text")
            {
                return Err("Unsupported clipboard context fields".into());
            }
            match (
                clipboard["available"].as_bool(),
                clipboard["text"].as_str(),
                clipboard["text"].is_null(),
            ) {
                (Some(true), Some(text), _) if text.len() <= MAX_VALUE => {
                    Some(Some(text.to_owned()))
                }
                (Some(false), None, true) => Some(None),
                _ => return Err("Malformed, inconsistent or excessive clipboard context".into()),
            }
        };
        Ok(Self {
            date,
            random,
            clipboard,
        })
    }
}
/// Editor context is data, never new source authority. Callers supply the held exact stamp.
#[derive(Debug)]
</ui:copy>

## Bind context to the held source and selection

A returned completion proof must describe the same source revision and exact authored selection. Validate scalar and CRLF boundaries before translating offsets. Reject foreign context instead of rebinding it to newer text.

<ui:copy id="snippet-context-source-proof">
pub struct SnippetEditorContext {
    pub from: usize,
    pub to: usize,
    pub snapshot: SnippetVariableSnapshot,
}
impl SnippetEditorContext {
    pub fn read(
        raw: &Value,
        source: &str,
        source_hash: &str,
        source_revision: &str,
    ) -> Result<Self, String> {
        fn keys(raw: &Value, allowed: &[&str]) -> Result<(), String> {
            let object = raw.as_object().ok_or("Malformed editor snippet context")?;
            if object.len() != allowed.len()
                || object.keys().any(|key| !allowed.contains(&key.as_str()))
            {
                return Err("Unsupported editor snippet context fields".into());
            }
            Ok(())
        }
        keys(
            raw,
            &[
                "version",
                "expected",
                "selection",
                "clock",
                "random_hex",
                "clipboard",
            ],
        )?;
        if raw["version"].as_u64() != Some(1) {
            return Err("Unsupported editor snippet context version".into());
        }
        keys(&raw["expected"], &["source_hash", "source_revision"])?;
        if raw["expected"]["source_hash"].as_str() != Some(source_hash)
            || raw["expected"]["source_revision"].as_str() != Some(source_revision)
        {
            return Err("Snippet source stamp changed. Request again.".into());
        }
        keys(&raw["selection"], &["from", "to"])?;
        fn byte(source: &str, raw: &Value) -> Result<usize, String> {
            let units = raw.as_u64().ok_or("Malformed snippet selection")?;
            let mut offset = 0u64;
            for (at, ch) in source.char_indices() {
                if offset == units {
                    if at > 0 && source.as_bytes()[at - 1] == b'\r' && ch == '\n' {
                        break;
                    }
                    return Ok(at);
                }
                offset += ch.len_utf16() as u64;
                if offset > units {
                    break;
                }
            }
            if offset == units && units == source.encode_utf16().count() as u64 {
                return Ok(source.len());
            }
            Err("Snippet selection splits a scalar/CRLF or is outside source".into())
        }
        let from = byte(source, &raw["selection"]["from"])?;
        let to = byte(source, &raw["selection"]["to"])?;
        if from > to {
            return Err("Backwards snippet selection".into());
        }
        let snapshot =
            SnippetVariableSnapshot::read(&raw["clock"], &raw["random_hex"], &raw["clipboard"])?;
        Ok(Self { from, to, snapshot })
    }
}
</ui:copy>

## Keep language comment values explicit

Comment delimiters depend on the producing language. Unsupported languages leave those values unset. Escape literal HTML-comment delimiters in Rust source strings so literate parsing preserves the intended runtime bytes.

<ui:copy id="snippet-context-comment-values">
fn comments(
    language: Option<&str>,
) -> (
    Option<&'static str>,
    Option<&'static str>,
    Option<&'static str>,
) {
    match language.unwrap_or("") {
        "c" | "cpp" | "rust" | "javascript" | "typescript" | "javascriptreact"
        | "typescriptreact" | "java" | "kotlin" | "swift" | "csharp" | "go" | "dart" | "php"
        | "jsonc" => (Some("//"), Some("/*"), Some("*/")),
        "python" | "ruby" | "shellscript" | "bash" | "sh" | "perl" | "yaml" | "toml" | "r"
        | "nix" => (Some("#"), None, None),
        "css" | "scss" | "less" => (None, Some("/*"), Some("*/")),
        "html" | "xml" | "markdown" => (None, Some("\x3c!--"), Some("--\x3e")),
        "lua" => (Some("--"), Some("--[["), Some("]]")),
        "sql" => (Some("--"), Some("/*"), Some("*/")),
        "haskell" => (Some("--"), Some("{-"), Some("-}")),
        "lisp" | "scheme" | "racket" => (Some(";"), Some("#|"), Some("|#")),
        "clojure" | "ini" => (Some(";"), None, None),
        _ => (None, None, None),
    }
}
/// Materialize only requested values while retaining every recognized unset name.
/// Query availability without cloning values from unused defaults.
</ui:copy>

## Inspect presence before materializing reachable values

The presence pass reports whether a value exists without allocating its full text. The parser uses it to follow only rendered defaults and field dependencies; only then materialize the reachable values and enforce byte/work limits. An unused oversized selection cannot prevent an unrelated filename expansion.

<ui:copy id="snippet-context-reachable-values">
pub fn variable_presence(
    context: &SnippetVariableContext<'_>,
    snapshot: &SnippetVariableSnapshot,
    requested: &BTreeSet<String>,
) -> Result<BTreeMap<String, Option<String>>, String> {
    read_variables(context, snapshot, requested, true)
}
pub fn variables(
    context: &SnippetVariableContext<'_>,
    snapshot: &SnippetVariableSnapshot,
    requested: &BTreeSet<String>,
) -> Result<BTreeMap<String, Option<String>>, String> {
    read_variables(context, snapshot, requested, false)
}
fn read_variables(
    context: &SnippetVariableContext<'_>,
    snapshot: &SnippetVariableSnapshot,
    requested: &BTreeSet<String>,
    presence_only: bool,
) -> Result<BTreeMap<String, Option<String>>, String> {
    let source = context.source;
    let scalar = |at: usize| {
        at <= source.len()
            && source.is_char_boundary(at)
            && !(at > 0
                && at < source.len()
                && source.as_bytes()[at - 1] == b'\r'
                && source.as_bytes()[at] == b'\n')
    };
    if context.from > context.to || !scalar(context.from) || !scalar(context.to) {
        return Err("Snippet context selection is stale or splits a source scalar/CRLF".into());
    }
    let mut values: BTreeMap<String, Option<String>> =
        KNOWN.iter().map(|name| ((*name).into(), None)).collect();
    let mut insert = |name: &str, value: Option<&str>| -> Result<(), String> {
        if requested.contains(name) {
            if !presence_only && value.is_some_and(|value| value.len() > MAX_VALUE) {
                return Err(format!(
                    "Snippet context {name} exceeds the expansion limit"
                ));
            }
            values.insert(
                name.into(),
                value.map(|value| {
                    if presence_only {
                        String::new()
                    } else {
                        value.to_owned()
                    }
                }),
            );
        }
        Ok(())
    };
    insert("TM_SELECTED_TEXT", Some(&source[context.from..context.to]))?;
    if requested.contains("TM_CURRENT_LINE") {
        let start = source[..context.from]
            .rfind(|ch| ch == '\n' || ch == '\r')
            .map_or(0, |at| at + 1);
        let end = source[context.from..]
            .find(|ch| ch == '\n' || ch == '\r')
            .map_or(source.len(), |at| context.from + at);
        insert(
            "TM_CURRENT_LINE",
            Some(source[start..end].trim_end_matches('\r')),
        )?;
    }
    if requested.contains("TM_CURRENT_WORD") {
        let word = |ch: char| crate::snippet_case::contains("identifier", ch as u32);
        let start = source[..context.from]
            .char_indices()
            .rev()
            .take_while(|(_, ch)| word(*ch))
            .last()
            .map_or(context.from, |(at, _)| at);
        let end = source[context.from..]
            .char_indices()
            .take_while(|(_, ch)| word(*ch))
            .last()
            .map_or(context.from, |(at, ch)| context.from + at + ch.len_utf8());
        insert("TM_CURRENT_WORD", Some(&source[start..end]))?;
    }
    let line = source[..context.from]
        .bytes()
        .enumerate()
        .filter(|(index, byte)| {
            *byte == b'\r'
                || *byte == b'\n' && (*index == 0 || source.as_bytes()[*index - 1] != b'\r')
        })
        .count();
    insert("TM_LINE_INDEX", Some(&line.to_string()))?;
    insert("TM_LINE_NUMBER", Some(&(line + 1).to_string()))?;
    for (name, path) in [
        (
            "TM_FILENAME",
            context.path.file_name().and_then(|s| s.to_str()),
        ),
        (
            "TM_FILENAME_BASE",
            context.path.file_stem().and_then(|s| s.to_str()),
        ),
        (
            "TM_DIRECTORY",
            context.path.parent().and_then(|p| p.to_str()),
        ),
        ("TM_FILEPATH", context.path.to_str()),
        (
            "RELATIVE_FILEPATH",
            context
                .workspace
                .and_then(|root| context.path.strip_prefix(root).ok())
                .and_then(|p| p.to_str()),
        ),
        (
            "WORKSPACE_NAME",
            context
                .workspace
                .and_then(Path::file_name)
                .and_then(|s| s.to_str()),
        ),
        ("WORKSPACE_FOLDER", context.workspace.and_then(Path::to_str)),
    ] {
        insert(name, path)?;
    }
    insert(
        "CLIPBOARD",
        snapshot
            .clipboard
            .as_ref()
            .and_then(|value| value.as_deref()),
    )?;
    for (name, format) in [
        ("CURRENT_YEAR", "%Y"),
        ("CURRENT_YEAR_SHORT", "%y"),
        ("CURRENT_MONTH", "%m"),
        ("CURRENT_MONTH_NAME", "%B"),
        ("CURRENT_MONTH_NAME_SHORT", "%b"),
        ("CURRENT_DATE", "%d"),
        ("CURRENT_DAY_NAME", "%A"),
        ("CURRENT_DAY_NAME_SHORT", "%a"),
        ("CURRENT_HOUR", "%H"),
        ("CURRENT_MINUTE", "%M"),
        ("CURRENT_SECOND", "%S"),
        ("CURRENT_TIMEZONE_OFFSET", "%:z"),
    ] {
        if requested.contains(name) {
            if let Some(date) = snapshot.date {
                insert(name, Some(&date.format(format).to_string()))?;
            }
        }
    }
    if let Some(date) = snapshot.date {
        insert("CURRENT_SECONDS_UNIX", Some(&date.timestamp().to_string()))?;
    }
    if let Some(mut random) = snapshot.random {
        let number = u32::from_be_bytes(random[..4].try_into().unwrap()) % 1_000_000;
        insert("RANDOM", Some(&format!("{number:06}")))?;
        let hex = random[..3]
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        insert("RANDOM_HEX", Some(&hex))?;
        random[6] = (random[6] & 0x0f) | 0x40;
        random[8] = (random[8] & 0x3f) | 0x80;
        let uuid = random
            .iter()
            .enumerate()
            .map(|(index, byte)| {
                format!(
                    "{}{byte:02x}",
                    if [4, 6, 8, 10].contains(&index) {
                        "-"
                    } else {
                        ""
                    }
                )
            })
            .collect::<String>();
        insert("UUID", Some(&uuid))?;
    }
    let (line, start, end) = comments(context.language);
    insert("LINE_COMMENT", line)?;
    insert("BLOCK_COMMENT_START", start)?;
    insert("BLOCK_COMMENT_END", end)?;
    Ok(values)
}

</ui:copy>

## Exercise values, source refusal and unused-value budgets

These focused cases compare fixed host clock/offset and seed values, refuse malformed source boundaries and excessive requested values, and prove that presence-only analysis does not allocate or reject unused data. They establish this mechanism separately from host permission and editor acceptance.

<ui:copy id="snippet-context-rust-witnesses">
#[cfg(test)]
mod tests {
    use super::*;
    fn snapshot() -> SnippetVariableSnapshot {
        SnippetVariableSnapshot {
            date: Some(DateTime::parse_from_rfc3339("2024-02-29T23:05:07+05:30").unwrap()),
            random: Some(core::array::from_fn(|index| index as u8)),
            clipboard: None,
        }
    }
    #[test]
    fn editor_context_is_source_bound_and_never_borrows_the_server_clock() {
        let source = "λ😀\r\nname";
        let raw = serde_json::json!({"version":1,"expected":{"source_hash":"sha256:held","source_revision":"held-revision"},"selection":{"from":1,"to":3},"clock":{"unix_millis":1709228107000i64,"utc_offset_minutes":330},"random_hex":"000102030405060708090a0b0c0d0e0f","clipboard":null});
        let context =
            SnippetEditorContext::read(&raw, source, "sha256:held", "held-revision").unwrap();
        assert_eq!(&source[context.from..context.to], "😀");
        assert_eq!(
            context.snapshot.date.unwrap().offset().local_minus_utc(),
            19800
        );
        for selection in [
            serde_json::json!({"from":2,"to":3}),
            serde_json::json!({"from":4,"to":4}),
            serde_json::json!({"from":false,"to":3}),
            serde_json::json!({"from":3,"to":1}),
        ] {
            let mut changed = raw.clone();
            changed["selection"] = selection;
            assert!(
                SnippetEditorContext::read(&changed, source, "sha256:held", "held-revision")
                    .is_err()
            )
        }
        assert!(
            SnippetEditorContext::read(&raw, source, "sha256:held", "new-equal-byte-revision")
                .is_err()
        );
        let mut no_clock = raw.clone();
        no_clock["clock"] = Value::Null;
        assert!(
            SnippetEditorContext::read(&no_clock, source, "sha256:held", "held-revision")
                .unwrap()
                .snapshot
                .date
                .is_none()
        );
        let mut wrong_clock = raw.clone();
        wrong_clock["clock"]["utc_offset_minutes"] = serde_json::json!(841);
        assert!(
            SnippetEditorContext::read(&wrong_clock, source, "sha256:held", "held-revision")
                .is_err()
        );
    }
    #[test]
    fn clipboard_reads_and_line_endings_keep_explicit_context_semantics() {
        let none = Value::Null;
        let available = serde_json::json!({"available":true,"text":"$1 literal"});
        let snapshot = SnippetVariableSnapshot::read(&none, &none, &available).unwrap();
        assert_eq!(snapshot.clipboard, Some(Some("$1 literal".into())));
        assert_eq!(
            SnippetVariableSnapshot::read(
                &none,
                &none,
                &serde_json::json!({"available":false,"text":null})
            )
            .unwrap()
            .clipboard,
            Some(None)
        );
        for value in [
            serde_json::json!({"available":true,"text":null}),
            serde_json::json!({"available":false,"text":"hidden"}),
            serde_json::json!({"available":1,"text":"x"}),
            serde_json::json!({"available":true,"text":"x","extra":false}),
        ] {
            assert!(SnippetVariableSnapshot::read(&none, &none, &value).is_err())
        }
        let source = "first\rsecond\r\nthird\nlast";
        let from = source.find("third").unwrap();
        let context = SnippetVariableContext {
            source,
            from,
            to: from,
            path: Path::new("a.py"),
            workspace: None,
            language: Some("python"),
        };
        let values = variables(
            &context,
            &snapshot,
            &BTreeSet::from([
                "TM_LINE_INDEX".into(),
                "TM_CURRENT_LINE".into(),
                "CLIPBOARD".into(),
            ]),
        )
        .unwrap();
        assert_eq!(values["TM_LINE_INDEX"].as_deref(), Some("2"));
        assert_eq!(values["TM_CURRENT_LINE"].as_deref(), Some("third"));
        assert_eq!(values["CLIPBOARD"].as_deref(), Some("$1 literal"));
    }
    #[test]
    fn authored_unicode_selection_identity_clock_random_and_comments_are_one_snapshot() {
        let source = "first\r\nλvalue = '😀'\r\n";
        let from = source.find("value").unwrap();
        let to = source.find("'\r").unwrap() + 1;
        let requested = KNOWN.iter().map(|name| (*name).into()).collect();
        let context = SnippetVariableContext {
            source,
            from,
            to,
            path: Path::new("/owned/src/main.py"),
            workspace: Some(Path::new("/owned")),
            language: Some("python"),
        };
        let values = variables(&context, &snapshot(), &requested).unwrap();
        for (name, expected) in [
            ("TM_SELECTED_TEXT", "value = '😀'"),
            ("TM_CURRENT_LINE", "λvalue = '😀'"),
            ("TM_CURRENT_WORD", "λvalue"),
            ("TM_LINE_INDEX", "1"),
            ("TM_LINE_NUMBER", "2"),
            ("TM_FILENAME", "main.py"),
            ("TM_FILENAME_BASE", "main"),
            ("RELATIVE_FILEPATH", "src/main.py"),
            ("WORKSPACE_NAME", "owned"),
            ("CURRENT_YEAR", "2024"),
            ("CURRENT_MONTH_NAME", "February"),
            ("CURRENT_DAY_NAME", "Thursday"),
            ("CURRENT_TIMEZONE_OFFSET", "+05:30"),
            ("RANDOM", "066051"),
            ("RANDOM_HEX", "000102"),
            ("UUID", "00010203-0405-4607-8809-0a0b0c0d0e0f"),
            ("LINE_COMMENT", "#"),
        ] {
            assert_eq!(values[name].as_deref(), Some(expected), "{name}")
        }
        assert_eq!(values["CLIPBOARD"], None);
        assert_eq!(values["BLOCK_COMMENT_START"], None);
    }
    #[test]
    fn unsupported_or_excessive_context_does_not_borrow_or_truncate_source() {
        let requested = BTreeSet::from(["TM_SELECTED_TEXT".into()]);
        let mut context = SnippetVariableContext {
            source: "😀\r\n",
            from: 1,
            to: 1,
            path: Path::new("a.json"),
            workspace: None,
            language: Some("json"),
        };
        assert!(variables(&context, &snapshot(), &requested).is_err());
        context.from = 5;
        context.to = 5;
        assert!(variables(&context, &snapshot(), &requested).is_err());
        let large = "x".repeat(MAX_VALUE + 1);
        context.source = &large;
        context.from = 0;
        context.to = large.len();
        assert!(variables(&context, &snapshot(), &requested).is_err());
        assert_eq!(
            variable_presence(&context, &snapshot(), &requested).unwrap()["TM_SELECTED_TEXT"],
            Some(String::new())
        );
        assert!(
            variables(
                &context,
                &snapshot(),
                &BTreeSet::from(["TM_FILENAME".into()])
            )
            .is_ok()
        );
    }
    #[test]
    fn unset_optional_sources_keep_default_semantics_and_comment_tokens_are_explicit() {
        let requested = KNOWN.iter().map(|name| (*name).into()).collect();
        let context = SnippetVariableContext {
            source: "word",
            from: 0,
            to: 0,
            path: Path::new("view.xml"),
            workspace: None,
            language: Some("xml"),
        };
        let mut snapshot = snapshot();
        snapshot.random = None;
        snapshot.date = None;
        snapshot.clipboard = Some(Some("$1 literal".into()));
        let values = variables(&context, &snapshot, &requested).unwrap();
        assert_eq!(values["RANDOM"], None);
        assert_eq!(values["CURRENT_YEAR"], None);
        assert_eq!(values["WORKSPACE_FOLDER"], None);
        assert_eq!(values["CLIPBOARD"].as_deref(), Some("$1 literal"));
        assert_eq!(values["BLOCK_COMMENT_START"].as_deref(), Some("\x3c!--"));
        assert_eq!(values["BLOCK_COMMENT_END"].as_deref(), Some("--\x3e"));
        assert_eq!(values["LINE_COMMENT"], None);
    }
}
</ui:copy>
<ui:file path="crates/hick-lsp/src/snippet_context.rs"><ui:paste select="#snippet-variable-context-engine"/><ui:paste select="#snippet-context-snapshot-admission"/><ui:paste select="#snippet-context-source-proof"/><ui:paste select="#snippet-context-comment-values"/><ui:paste select="#snippet-context-reachable-values"/><ui:paste select="#snippet-context-rust-witnesses"/></ui:file>

## Capture host values without moving source authority into UI callbacks

The adapters collect one editor-local clock and random seed, then retain plain immutable values. Completion binding checks the exact offered engine hash and opaque revision before creating the request envelope. The outer editor lease must still be current after any asynchronous hash or provider request. Template capture reuses the same snapshot across catalog, preview and revalidation; no keystroke performs a provider request. Missing browser crypto leaves random variables unavailable rather than inventing a weaker random source.

<ui:copy id="snippet-variable-context-browser">
import {sha256} from '../lib/reviews';
export interface SnippetClipboard {readonly available:boolean;readonly text:string|null}
export interface SnippetVariableSnapshot {readonly clipboard:Readonly<SnippetClipboard>|null;readonly clock:Readonly<{unix_millis:number;utc_offset_minutes:number}>|null;readonly random_hex:string|null}
export function captureSnippetVariables():SnippetVariableSnapshot {
 const now=new Date();let random_hex:string|null=null;try{const bytes=globalThis.crypto?.getRandomValues(new Uint8Array(16));if(bytes)random_hex=Array.from(bytes,b=>b.toString(16).padStart(2,'0')).join('');}catch{}
 const clock=Number.isFinite(now.getTime())?Object.freeze({unix_millis:now.getTime(),utc_offset_minutes:-now.getTimezoneOffset()}):null;return Object.freeze({clock,random_hex,clipboard:null});
}
function boundedClipboardText(value:string):boolean{if(value.length>1048576)return false;let bytes=0;for(let at=0;at<value.length;at++){const unit=value.charCodeAt(at);if(unit>=0xD800&&unit<=0xDBFF){const low=value.charCodeAt(++at);if(!(low>=0xDC00&&low<=0xDFFF))return false;bytes+=4;}else{if(unit>=0xDC00&&unit<=0xDFFF)return false;bytes+=unit<=0x7f?1:unit<=0x7ff?2:3;}if(bytes>1048576)return false;}return true;}
export async function captureSnippetClipboard():Promise<Readonly<SnippetClipboard>>{try{const value=await globalThis.navigator?.clipboard?.readText();if(typeof value==='string'&&boundedClipboardText(value))return Object.freeze({available:true,text:value});}catch{}return Object.freeze({available:false,text:null});}
export function validSnippetContextSelection(source:string,from:number,to:number):boolean {
 const boundary=(at:number)=>Number.isSafeInteger(at)&&at>=0&&at<=source.length&&!(at>0&&at<source.length&&(source.charCodeAt(at-1)>=0xD800&&source.charCodeAt(at-1)<=0xDBFF&&source.charCodeAt(at)>=0xDC00&&source.charCodeAt(at)<=0xDFFF||source[at-1]==='\r'&&source[at]==='\n'));
 return from<=to&&boundary(from)&&boundary(to)&&!/[\uD800-\uDFFF]/u.test(source);
}
export async function bindSnippetVariables(source:string,from:number,to:number,proof:unknown,snapshot:SnippetVariableSnapshot=captureSnippetVariables()):Promise<Record<string,unknown>> {
 if(!validSnippetContextSelection(source,from,to)||!proof||typeof proof!=='object'||Array.isArray(proof))throw Error('Snippet source context is unavailable. Request completions again.');const expected=proof as Record<string,unknown>;
 if(typeof expected.source_hash!=='string'||!/^sha256:[a-f0-9]{64}$/.test(expected.source_hash)||typeof expected.source_revision!=='string'||! /^[a-f0-9]{64}$/.test(expected.source_revision)||expected.source_hash!=='sha256:'+await sha256(source))throw Error('Snippet source context changed. Request completions again.');
 return Object.freeze({version:1,expected:Object.freeze({source_hash:expected.source_hash,source_revision:expected.source_revision}),selection:Object.freeze({from,to}),clock:snapshot.clock,random_hex:snapshot.random_hex,clipboard:snapshot.clipboard});
}

function snippetRecord(value:unknown):value is Record<string,unknown>{return !!value&&typeof value==='object'&&!Array.isArray(value);}
export async function resolveSnippetCompletion<T extends {label:string;data?:unknown}>(original:T,adapter:{source:string;from:number;to:number;current:()=>boolean;request:(item:T)=>Promise<T>;clipboard?:()=>Promise<Readonly<SnippetClipboard>>}):Promise<T>{
 const current=()=>{if(!adapter.current())throw Error('Source, cursor or provider changed. Request completions again.');};current();
 if(!snippetRecord(original.data)||!Object.prototype.hasOwnProperty.call(original.data,'hickory_context')){const result=await adapter.request(original);current();return result;}
 let snapshot=captureSnippetVariables(),item=original;
 for(let attempt=0;attempt<2;attempt++){
  current();if(!snippetRecord(item.data))throw Error('Completion context proof is unavailable. Request again.');const bound=await bindSnippetVariables(adapter.source,adapter.from,adapter.to,item.data.hickory_context,snapshot);current();
  const result=await adapter.request({...item,data:{...item.data,hickory_context:bound}} as T);current();const data=snippetRecord(result.data)?result.data:{};const expected=bound.expected as Record<string,unknown>,proof=data.hickory_context;if(!snippetRecord(proof)||proof.source_hash!==expected.source_hash||proof.source_revision!==expected.source_revision)throw Error('Completion source context changed. Request again.');
  if(!Object.prototype.hasOwnProperty.call(data,'hickory_context_request'))return result;
  const request=data.hickory_context_request;if(attempt!==0||!snippetRecord(request)||Object.keys(request).length!==2||request.version!==1||!Array.isArray(request.variables)||request.variables.length!==1||request.variables[0]!=='CLIPBOARD')throw Error('Unsupported or repeated completion context request. No edits were applied.');
  const clipboard=await(adapter.clipboard??captureSnippetClipboard)();current();snapshot=Object.freeze({clock:snapshot.clock,random_hex:snapshot.random_hex,clipboard:Object.freeze({available:clipboard.available,text:clipboard.text})});item=result;
 }
 throw Error('Completion context is unavailable. No edits were applied.');
}
export function snippetContextNotice(item:unknown):string{const data=snippetRecord(item)&&snippetRecord(item.data)?item.data:{};if(!Object.prototype.hasOwnProperty.call(data,'hickory_context_unavailable'))return '';const values=data.hickory_context_unavailable;if(!Array.isArray(values)||values.length>40||!values.every(value=>typeof value==='string'&&value.length<=64))throw Error('Malformed completion context availability');return values.length?values.every(value=>value==='CLIPBOARD')?'Clipboard unavailable; its fallback or empty text was used.':'Some requested context values were unavailable; their fallback or empty text was used.':'';}
</ui:copy>

<ui:copy id="snippet-variable-context-apple">
import Foundation
struct SnippetVariableClock:Codable,Equatable {let unix_millis:Int64;let utc_offset_minutes:Int}
struct SnippetClipboard:Codable,Equatable {let available:Bool;let text:String?
 static func ==(a:Self,b:Self)->Bool{a.available==b.available && ((a.text==nil&&b.text==nil)||(a.text != nil&&b.text != nil&&Array(a.text!.utf8)==Array(b.text!.utf8)))}
}
struct SnippetVariableSnapshot:Codable,Equatable {
 let clock:SnippetVariableClock?;let random_hex:String?;var clipboard:SnippetClipboard?=nil
 static func capture()->Self {let now=Date();return Self(clock:SnippetVariableClock(unix_millis:Int64((now.timeIntervalSince1970*1000).rounded(.towardZero)),utc_offset_minutes:TimeZone.current.secondsFromGMT(for:now)/60),random_hex:UUID().uuidString.replacingOccurrences(of:"-",with:""))}
 var body:[String:Any]{["clock":clock.map{["unix_millis":$0.unix_millis,"utc_offset_minutes":Int64($0.utc_offset_minutes)]} as Any? ?? NSNull(),"random_hex":random_hex as Any? ?? NSNull(),"clipboard":clipboard.map{["available":$0.available,"text":$0.text as Any? ?? NSNull()]} as Any? ?? NSNull()]}
}
#if os(macOS)
import AppKit
#elseif os(iOS)
import UIKit
#endif
@MainActor func captureSnippetClipboard(read:(()->String?)?=nil)->SnippetClipboard {
 let value:String?
 if let read{value=read()}else{
 #if os(macOS)
 value=NSPasteboard.general.string(forType:.string)
 #elseif os(iOS)
 value=UIPasteboard.general.string
 #else
 value=nil
 #endif
 }
 guard let value,value.utf16.prefix(1048577).count<=1048576,value.utf8.prefix(1048577).count<=1048576 else{return SnippetClipboard(available:false,text:nil)};return SnippetClipboard(available:true,text:value)
}
func validSnippetContextSelection(_ source:String,_ from:Int,_ to:Int)->Bool {
 let ns=source as NSString
 func boundary(_ at:Int)->Bool{guard at>=0,at<=ns.length else{return false};if at>0,at<ns.length{let before=ns.character(at:at-1),after=ns.character(at:at);if before>=0xD800&&before<=0xDBFF&&after>=0xDC00&&after<=0xDFFF||before==13&&after==10{return false}};return true}
 return from<=to&&boundary(from)&&boundary(to)
}
func bindSnippetVariables(_ source:String,from:Int,to:Int,proof:Any,snapshot:SnippetVariableSnapshot = .capture())throws->[String:Any] {
 guard validSnippetContextSelection(source,from,to),let expected=proof as? [String:Any],let hash=expected["source_hash"] as? String,let revision=expected["source_revision"] as? String,hash=="sha256:"+nativeSourceHash(source),revision.count==64,revision.utf8.allSatisfy({(48...57).contains($0)||(97...102).contains($0)}) else{throw EngineError(message:"Snippet source context changed. Request completions again.")}
 return snapshot.body.merging(["version":1,"expected":["source_hash":hash,"source_revision":revision],"selection":["from":from,"to":to]]){_,new in new}
}

@MainActor func resolveSnippetCompletion(_ original:[String:Any],source:String,from:Int,to:Int,current:()->Bool,request:([String:Any])async throws->[String:Any],clipboard:(()->String?)?=nil)async throws->[String:Any]{
 func check()throws{guard current() else{throw EngineError(message:"Source, cursor or provider changed. Request completions again.")}};try check()
 guard let initialData=original["data"] as? [String:Any],initialData.keys.contains("hickory_context") else{let result=try await request(original);try check();return result}
 var snapshot=SnippetVariableSnapshot.capture(),item=original
 for attempt in 0..<2{try check();guard let data=item["data"] as? [String:Any],let proof=data["hickory_context"] else{throw EngineError(message:"Completion context proof is unavailable. Request again.")};let bound=try bindSnippetVariables(source,from:from,to:to,proof:proof,snapshot:snapshot);try check();var params=item,wire=data;wire["hickory_context"]=bound;params["data"]=wire;let result=try await request(params);try check();let reply=result["data"] as? [String:Any] ?? [:]
  guard let expected=bound["expected"] as? [String:Any],let proof=reply["hickory_context"] as? [String:Any],let hash=proof["source_hash"] as? String,let revision=proof["source_revision"] as? String,hash==expected["source_hash"] as? String,revision==expected["source_revision"] as? String else{throw EngineError(message:"Completion source context changed. Request again.")}
  guard let raw=reply["hickory_context_request"] else{return result};guard attempt==0,let requested=raw as? [String:Any],requested.count==2,sourceChangeInteger(requested["version"])==1,let names=requested["variables"] as? [String],names==["CLIPBOARD"] else{throw EngineError(message:"Unsupported or repeated completion context request. No edits were applied.")};snapshot.clipboard=captureSnippetClipboard(read:clipboard);try check();item=result
 }
 throw EngineError(message:"Completion context is unavailable. No edits were applied.")
}
func snippetContextNotice(_ item:[String:Any])throws->String {guard let data=item["data"] as? [String:Any],let raw=data["hickory_context_unavailable"] else{return ""};guard let names=raw as? [String],names.count<=40,names.allSatisfy({$0.utf8.count<=64}) else{throw EngineError(message:"Malformed completion context availability")};guard !names.isEmpty else{return ""};return names.allSatisfy({$0=="CLIPBOARD"}) ? "Clipboard unavailable; its fallback or empty text was used.":"Some requested context values were unavailable; their fallback or empty text was used."}
</ui:copy>

<ui:copy id="snippet-variable-context-android">
package com.hickorydocs.simple
import org.json.JSONObject
import java.util.TimeZone
import java.util.UUID
data class SnippetClipboard(val available:Boolean,val text:String?)
data class SnippetVariableSnapshot(val unixMillis:Long?,val utcOffsetMinutes:Int?,val randomHex:String?,val clipboard:SnippetClipboard?=null) {
 companion object{fun capture():SnippetVariableSnapshot{val now=System.currentTimeMillis();return SnippetVariableSnapshot(now,TimeZone.getDefault().getOffset(now)/60000,UUID.randomUUID().toString().replace("-",""))}}
 fun body()=JSONObject().put("clock",if(unixMillis!=null&&utcOffsetMinutes!=null)JSONObject().put("unix_millis",unixMillis).put("utc_offset_minutes",utcOffsetMinutes) else JSONObject.NULL).put("random_hex",randomHex?:JSONObject.NULL).put("clipboard",clipboard?.let{JSONObject().put("available",it.available).put("text",it.text?:JSONObject.NULL)}?:JSONObject.NULL)
}
private fun boundedClipboardText(value:String):Boolean{if(value.length>1048576)return false;var bytes=0;var at=0;while(at<value.length){val unit=value[at++];if(unit.isHighSurrogate()){if(at==value.length||!value[at++].isLowSurrogate())return false;bytes+=4}else{if(unit.isLowSurrogate())return false;bytes+=if(unit.code<=0x7f)1 else if(unit.code<=0x7ff)2 else 3};if(bytes>1048576)return false};return true}
internal fun captureSnippetClipboard(read:(()->String?)?):SnippetClipboard=try{val value=read?.invoke();if(value!=null&&boundedClipboardText(value))SnippetClipboard(true,value) else SnippetClipboard(false,null)}catch(_:Exception){SnippetClipboard(false,null)}
internal fun validSnippetContextSelection(source:String,from:Int,to:Int):Boolean {
 fun boundary(at:Int)=at in 0..source.length&&!(at>0&&at<source.length&&(source[at-1].isHighSurrogate()&&source[at].isLowSurrogate()||source[at-1]=='\r'&&source[at]=='\n'))
 var at=0;while(at<source.length){val ch=source[at++];if(ch.isHighSurrogate()){if(at==source.length||!source[at++].isLowSurrogate())return false}else if(ch.isLowSurrogate())return false};return from<=to&&boundary(from)&&boundary(to)
}
internal fun bindSnippetVariables(source:String,from:Int,to:Int,proof:Any?,snapshot:SnippetVariableSnapshot=SnippetVariableSnapshot.capture()):JSONObject {
 check(validSnippetContextSelection(source,from,to)&&proof is JSONObject&&proof.opt("source_hash")=="sha256:"+debugHash(source)&&(proof.opt("source_revision") as? String)?.matches(Regex("[a-f0-9]{64}"))==true){"Snippet source context changed. Request completions again."}
 return snapshot.body().put("version",1).put("expected",JSONObject().put("source_hash",proof.getString("source_hash")).put("source_revision",proof.getString("source_revision"))).put("selection",JSONObject().put("from",from).put("to",to))
}

internal suspend fun resolveSnippetCompletion(original:JSONObject,source:String,from:Int,to:Int,current:()->Boolean,request:suspend (JSONObject)->JSONObject,clipboard:(()->String?)?=null):JSONObject {
 fun valid(){check(current()){"Source, cursor or provider changed. Request completions again."}};valid();if(original.optJSONObject("data")?.has("hickory_context")!=true){val result=request(original);valid();return result};var snapshot=SnippetVariableSnapshot.capture();var item=original
 for(attempt in 0 until 2){valid();val data=item.optJSONObject("data") ?: error("Completion context proof is unavailable. Request again.");val bound=bindSnippetVariables(source,from,to,data.opt("hickory_context"),snapshot);valid();val params=JSONObject(item.toString());params.put("data",JSONObject(data.toString()).put("hickory_context",bound));val result=request(params);valid();val reply=result.optJSONObject("data") ?: JSONObject();val expected=bound.getJSONObject("expected");val proof=reply.optJSONObject("hickory_context");check(proof!=null&&proof.opt("source_hash")==expected.opt("source_hash")&&proof.opt("source_revision")==expected.opt("source_revision")){"Completion source context changed. Request again."};if(!reply.has("hickory_context_request"))return result;val requested=reply.optJSONObject("hickory_context_request");val names=requested?.optJSONArray("variables");check(attempt==0&&requested!=null&&requested.length()==2&&(requested.opt("version") as? Number)?.toString()=="1"&&names?.length()==1&&names.opt(0)=="CLIPBOARD"){"Unsupported or repeated completion context request. No edits were applied."};snapshot=snapshot.copy(clipboard=captureSnippetClipboard(clipboard));valid();item=result}
 error("Completion context is unavailable. No edits were applied.")
}
internal fun snippetContextNotice(item:JSONObject):String {val data=item.optJSONObject("data")?:return "";if(!data.has("hickory_context_unavailable"))return "";val names=data.optJSONArray("hickory_context_unavailable")?:error("Malformed completion context availability");check(names.length()<=40&&(0 until names.length()).all{names.opt(it) is String&&names.getString(it).length<=64}){"Malformed completion context availability"};if(names.length()==0)return "";return if((0 until names.length()).all{names.getString(it)=="CLIPBOARD"})"Clipboard unavailable; its fallback or empty text was used." else "Some requested context values were unavailable; their fallback or empty text was used."}
</ui:copy>

## Keep the same context lease across all hosts

Each client captures its own clock, timezone offset and random seed once. It binds those values to the exact retained source proof and selection before a resolve. Clipboard validation rejects an excessive value before allocating an encoded copy, and scans at most the accepted byte budget. Clipboard access happens only after a valid demanded context response and never reaches the child language provider. A denial is an explicit availability value, so its fallback and notice are reproducible at Apply. Every asynchronous boundary rechecks the original editor lease; a foreign source proof refuses before clipboard access. Ordinary provider data remains opaque and unchanged.
<ui:file path="apps/web/src/simple/SnippetContext.ts"><ui:paste select="#snippet-variable-context-browser"/></ui:file>
<ui:file path="apps/apple-simple/Sources/SnippetContext.swift"><ui:paste select="#snippet-variable-context-apple"/></ui:file>
<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/SnippetContext.kt"><ui:paste select="#snippet-variable-context-android"/></ui:file>

## Exercise demand, denial and stale context without writing source

These focused host witnesses drive the reusable two-step handshake. They retain the same clock and seed, preserve the original opaque item, reject foreign source stamps before reading clipboard content, and retire a pending read when the source or selection lease changes. Actual provider and retained-editor flows are checked separately; these tests do not establish physical input or accessibility acceptance.
<ui:copy id="snippet-variable-context-browser-witness">
import {describe,it,expect,beforeAll,vi} from 'vitest';
import {resolveSnippetCompletion,captureSnippetClipboard} from './SnippetContext';
beforeAll(async()=>{const name='node:'+'crypto';const {webcrypto}=await import(name);Object.defineProperty(globalThis,'crypto',{value:webcrypto,configurable:true});});
const source='λ😀\r\nword';
const hash='sha256:b780b85cc61fcebbfa18870c45ac77760d59272f59d6b40db68250fb081f8452';
const proof={source_hash:hash,source_revision:'a'.repeat(64)};
const initial=():{label:string;data:Record<string,unknown>}=>({label:'field',data:{hickory_completion:'original-token',hickory_context:proof,opaque:{provider:['untouched']}}});
const requested=(token='resolved-token',p=proof)=>({label:'field',data:{hickory_completion:token,hickory_context:p,hickory_context_request:{version:1,variables:['CLIPBOARD']}}});
describe('source-bound completion variable handshake',()=>{
 it('bounds clipboard validation before encoding or walking an excessive value',async()=>{const original=globalThis.TextEncoder;try{vi.stubGlobal('TextEncoder',class{encode(){throw Error('Clipboard must not allocate an encoded copy');}});for(const value of['x'.repeat(1048577),'😀'.repeat(300000),'\uD800']){vi.stubGlobal('navigator',{clipboard:{readText:async()=>value}});expect(await captureSnippetClipboard()).toEqual({available:false,text:null});}vi.stubGlobal('navigator',{clipboard:{readText:async()=>'x'.repeat(1048576)}});expect((await captureSnippetClipboard()).available).toBe(true);}finally{vi.unstubAllGlobals();globalThis.TextEncoder=original;}});
 it('reads clipboard once on demand, keeps clock and seed, and leaves the original opaque item unchanged',async()=>{
  const original=initial(),before=JSON.stringify(original),calls:any[]=[];let reads=0;
  const final=await resolveSnippetCompletion(original,{source,from:5,to:9,current:()=>true,clipboard:async()=>{reads++;return {available:true,text:'clipboard text'};},request:async item=>{calls.push(item);return (calls.length===1?requested():{label:'done',data:{hickory_context:proof}}) as typeof item;}});
  expect(final.label).toBe('done');expect(reads).toBe(1);expect(calls).toHaveLength(2);expect(JSON.stringify(original)).toBe(before);
  const first=calls[0].data.hickory_context,second=calls[1].data.hickory_context;
  expect(first.expected).toEqual(proof);expect(first.selection).toEqual({from:5,to:9});expect(first.clipboard).toBe(null);expect(second.clipboard).toEqual({available:true,text:'clipboard text'});
  expect(second.clock).toEqual(first.clock);expect(second.random_hex).toBe(first.random_hex);expect(calls[1].data.hickory_completion).toBe('resolved-token');
 });
 it('does not request or read unavailable context when no clipboard variable was requested',async()=>{
  let reads=0,calls=0;await resolveSnippetCompletion(initial(),{source,from:5,to:9,current:()=>true,clipboard:async()=>{reads++;return {available:false,text:null};},request:async item=>{calls++;return {label:'done',data:{hickory_context:proof}} as typeof item;}});expect(calls).toBe(1);expect(reads).toBe(0);
 });
 it('preserves explicit clipboard denial for the second resolve',async()=>{
  const calls:any[]=[];await resolveSnippetCompletion(initial(),{source,from:5,to:9,current:()=>true,clipboard:async()=>({available:false,text:null}),request:async item=>{calls.push(item);return (calls.length===1?requested():{label:'done',data:{hickory_context:proof}}) as typeof item;}});expect(calls[1].data.hickory_context.clipboard).toEqual({available:false,text:null});
 });
 it('rejects foreign source proofs before privacy-sensitive clipboard access',async()=>{
  for(const p of [{...proof,source_hash:'sha256:'+'b'.repeat(64)},{...proof,source_revision:'b'.repeat(64)}]){let reads=0;await expect(resolveSnippetCompletion(initial(),{source,from:5,to:9,current:()=>true,clipboard:async()=>{reads++;return {available:true,text:'secret'};},request:async()=>requested('foreign',p) as any})).rejects.toThrow('source context changed');expect(reads).toBe(0);}
 });
 it('rejects malformed or repeated requests without another clipboard read',async()=>{
  for(const request of [{version:2,variables:['CLIPBOARD']},{version:1,variables:['PATH']},{version:1,variables:['CLIPBOARD'],extra:true}]){let reads=0;await expect(resolveSnippetCompletion(initial(),{source,from:5,to:9,current:()=>true,clipboard:async()=>{reads++;return {available:true,text:'secret'};},request:async()=>({...requested(),data:{...requested().data,hickory_context_request:request}}) as any})).rejects.toThrow('Unsupported');expect(reads).toBe(0);}
  let reads=0,calls=0;await expect(resolveSnippetCompletion(initial(),{source,from:5,to:9,current:()=>true,clipboard:async()=>{reads++;return {available:true,text:'secret'};},request:async()=>{calls++;return requested() as any;}})).rejects.toThrow('repeated');expect(reads).toBe(1);expect(calls).toBe(2);
 });
 it('retires before another request when source or cursor changes during clipboard access',async()=>{
  let current=true,calls=0;await expect(resolveSnippetCompletion(initial(),{source,from:5,to:9,current:()=>current,clipboard:async()=>{current=false;return {available:true,text:'secret'};},request:async()=>{calls++;return requested() as any;}})).rejects.toThrow('Source, cursor');expect(calls).toBe(1);
 });
 it('retires source changes while the provider is resolving before clipboard access',async()=>{
  let current=true,reads=0;await expect(resolveSnippetCompletion(initial(),{source,from:5,to:9,current:()=>current,clipboard:async()=>{reads++;return {available:true,text:'secret'};},request:async()=>{current=false;return requested() as any;}})).rejects.toThrow('Source, cursor');expect(reads).toBe(0);
 });
});
</ui:copy>
<ui:file path="apps/web/src/simple/SnippetContext.test.ts"><ui:paste select="#snippet-variable-context-browser-witness"/></ui:file>
<ui:copy id="snippet-variable-context-apple-witness">
import XCTest
@testable import HickorySimple

@MainActor final class SnippetContextHandshakeTests:XCTestCase {
 func testClipboardReadBoundsValidationWithoutEncodingAnExcessiveCopy(){for value in [String(repeating:"x",count:1048577),String(repeating:"😀",count:300000)]{let result=captureSnippetClipboard(read:{value});XCTAssertFalse(result.available);XCTAssertNil(result.text)};XCTAssertTrue(captureSnippetClipboard(read:{String(repeating:"x",count:1048576)}).available)}
 private let source="λ😀\r\nword"
 private var proof:[String:Any]{["source_hash":"sha256:"+nativeSourceHash(source),"source_revision":String(repeating:"a",count:64)]}
 private var original:[String:Any]{["label":"field","data":["hickory_completion":"original","hickory_context":proof,"opaque":["provider":["untouched"]]]]}
 private func requested(_ sourceProof:[String:Any]?=nil)->[String:Any]{["label":"field","data":["hickory_completion":"resolved","hickory_context":sourceProof ?? proof,"hickory_context_request":["version":1,"variables":["CLIPBOARD"]]]]}
 func testDemandedClipboardUsesOneCapturedSnapshotWithoutMutatingOriginal()async throws {
  let item=original,originalBytes=try JSONSerialization.data(withJSONObject:item,options:.sortedKeys);var calls=[[String:Any]](),reads=0
  let result=try await resolveSnippetCompletion(item,source:source,from:5,to:9,current:{true},request:{params in calls.append(params);return calls.count==1 ? self.requested():["label":"done","data":["hickory_context":self.proof]]},clipboard:{reads+=1;return "clipboard"})
  XCTAssertEqual(result["label"] as? String,"done");XCTAssertEqual(reads,1);XCTAssertEqual(calls.count,2);XCTAssertEqual(try JSONSerialization.data(withJSONObject:item,options:.sortedKeys),originalBytes)
  let first=(calls[0]["data"] as! [String:Any])["hickory_context"] as! [String:Any],second=(calls[1]["data"] as! [String:Any])["hickory_context"] as! [String:Any]
  XCTAssertEqual(first["random_hex"] as? String,second["random_hex"] as? String)
  XCTAssertEqual(try JSONSerialization.data(withJSONObject:first["clock"]!,options:.sortedKeys),try JSONSerialization.data(withJSONObject:second["clock"]!,options:.sortedKeys))
  XCTAssertTrue(first["clipboard"] is NSNull);XCTAssertEqual((second["clipboard"] as? [String:Any])?["text"] as? String,"clipboard")
 }
 func testForeignProofAndRetiredClipboardCannotCauseSecondProviderRequest()async {
  for foreign in [true,false]{var current=true,calls=0,reads=0
   do{_ = try await resolveSnippetCompletion(original,source:source,from:5,to:9,current:{current},request:{_ in calls+=1;var proof=self.proof;if foreign{proof["source_revision"]=String(repeating:"b",count:64)};return self.requested(proof)},clipboard:{reads+=1;current=false;return "secret"});XCTFail("Stale context was accepted") }catch{}
   XCTAssertEqual(calls,1);XCTAssertEqual(reads,foreign ? 0:1)
  }
 }
}
</ui:copy>
<ui:file path="apps/apple-simple/Tests/SnippetContextHandshakeTests.swift"><ui:paste select="#snippet-variable-context-apple-witness"/></ui:file>
<ui:copy id="snippet-variable-context-android-witness">
package com.hickorydocs.simple

import kotlinx.coroutines.runBlocking
import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Test

class SnippetContextHandshakeTest {
 @Test fun clipboardReadBoundsValidationBeforeAnExcessiveEncoding(){for(value in listOf("x".repeat(1048577),"😀".repeat(300000),"\uD800")){val result=captureSnippetClipboard{value};assertFalse(result.available);assertNull(result.text)};assertTrue(captureSnippetClipboard{"x".repeat(1048576)}.available)}
 private val source="λ😀\r\nword"
 private fun proof()=JSONObject().put("source_hash","sha256:"+debugHash(source)).put("source_revision","a".repeat(64))
 private fun original()=JSONObject().put("label","field").put("data",JSONObject().put("hickory_completion","original").put("hickory_context",proof()).put("opaque",JSONObject().put("provider","untouched")))
 private fun requested(p:JSONObject=proof())=JSONObject().put("label","field").put("data",JSONObject().put("hickory_completion","resolved").put("hickory_context",p).put("hickory_context_request",JSONObject().put("version",1).put("variables",org.json.JSONArray().put("CLIPBOARD"))))
 @Test fun demandedClipboardKeepsSnapshotAndOriginalOpaqueData()=runBlocking {
  val original=original();val before=original.toString();val calls=mutableListOf<JSONObject>();var reads=0
  val result=resolveSnippetCompletion(original,source,5,9,{true},{params->calls+=params;if(calls.size==1)requested() else JSONObject().put("label","done").put("data",JSONObject().put("hickory_context",proof()))},{reads++;"clipboard"})
  assertEquals("done",result.getString("label"));assertEquals(1,reads);assertEquals(2,calls.size);assertEquals(before,original.toString())
  val first=calls[0].getJSONObject("data").getJSONObject("hickory_context");val second=calls[1].getJSONObject("data").getJSONObject("hickory_context")
  assertEquals(first.opt("random_hex"),second.opt("random_hex"));assertEquals(first.getJSONObject("clock").toString(),second.getJSONObject("clock").toString());assertTrue(first.isNull("clipboard"));assertEquals("clipboard",second.getJSONObject("clipboard").getString("text"))
 }
 @Test fun foreignProofAndRetiredClipboardCannotCauseSecondProviderRequest()=runBlocking {
  for(foreign in listOf(true,false)){var current=true;var calls=0;var reads=0
   val result=runCatching{resolveSnippetCompletion(original(),source,5,9,{current},{calls++;requested(proof().also{if(foreign)it.put("source_revision","b".repeat(64))})},{reads++;current=false;"secret"})}
   assertTrue(result.isFailure);assertEquals(1,calls);assertEquals(if(foreign)0 else 1,reads)
  }
 }
}
</ui:copy>
<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/SnippetContextHandshakeTest.kt"><ui:paste select="#snippet-variable-context-android-witness"/></ui:file>
</ui:doc>
