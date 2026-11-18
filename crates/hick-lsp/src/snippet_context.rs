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
