//! Explicit source-bound inputs for completion and reusable template variables.
use chrono::{DateTime, Datelike, FixedOffset};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
const MAX_VALUE: usize = 1_048_576;
const KNOWN: &[&str] = &[
    "CURSOR_INDEX",
    "CURSOR_NUMBER",
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
/// The optional grouped row provides a zero-based ordinal for variables.
/// It carries no source authority. The producing offer still owns the position.
#[derive(Debug)]
pub struct SnippetEditorContext {
    pub from: usize,
    pub to: usize,
    pub caret: Option<(usize, usize)>,
    pub snapshot: SnippetVariableSnapshot,
    pub transform_program_versions:Vec<u32>,
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
        let programs=if let Some(raw)=raw.get("transform_program_versions"){let values=raw.as_array().ok_or("Malformed snippet transform program versions")?;if values.len()>2{return Err("Excessive snippet transform program versions".into())};values.iter().map(|n|n.as_u64().and_then(|n|u32::try_from(n).ok()).ok_or_else(||"Malformed snippet transform program version".to_owned())).collect::<Result<Vec<_>,_>>()?}else{vec![2]};
        crate::snippet_transform::validate_program_versions(&programs)?;
        let mut allowed=vec!["version","expected","selection","clock","random_hex","clipboard"];
        if raw.get("transform_program_versions").is_some(){allowed.push("transform_program_versions")};
        if raw.get("carets").is_some(){allowed.push("carets")};
        keys(raw,&allowed)?;
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
        let caret = raw.get("carets").map(|row| {
            keys(row, &["index", "count"])?;
            let count = row["count"].as_u64().filter(|count| (2..=16).contains(count))
                .ok_or("Malformed snippet caret count")?;
            let index = row["index"].as_u64().filter(|index| *index < count)
                .ok_or("Malformed snippet caret index")?;
            Ok::<_, String>((index as usize, count as usize))
        }).transpose()?;
        let snapshot =
            SnippetVariableSnapshot::read(&raw["clock"], &raw["random_hex"], &raw["clipboard"])?;
        Ok(Self { from, to, caret, snapshot, transform_program_versions:programs })
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
/// Bind an ordered selection row without recapturing its shared host snapshot.
/// Validate before changing any value so an excessive row never partially binds.
pub fn bind_cursor_variables(
    values: &mut BTreeMap<String, Option<String>>,
    requested: &BTreeSet<String>,
    index: usize,
) -> Result<(), String> {
    if index >= 4096 {
        return Err("Snippet cursor index exceeds the selection limit".into());
    }
    for (name, number) in [("CURSOR_INDEX", index), ("CURSOR_NUMBER", index + 1)] {
        values.insert(
            name.into(),
            requested.contains(name).then(|| number.to_string()),
        );
    }
    Ok(())
}
#[cfg(test)]
mod cursor_variable_witnesses {
    use super::*;
    #[test]
    fn single_selection_defaults_and_ordered_rows_keep_the_same_snapshot() {
        let requested = ["CURSOR_INDEX", "CURSOR_NUMBER", "CLIPBOARD"]
            .into_iter()
            .map(String::from)
            .collect();
        let context = SnippetVariableContext {
            source: "old",
            from: 0,
            to: 3,
            path: Path::new("/work/main.py"),
            workspace: Some(Path::new("/work")),
            language: Some("python"),
        };
        let snapshot = SnippetVariableSnapshot {
            date: None,
            random: None,
            clipboard: Some(Some("captured once".into())),
        };
        let presence = variable_presence(&context, &snapshot, &requested).unwrap();
        assert_eq!(presence["CURSOR_INDEX"].as_deref(), Some("0"));
        assert_eq!(presence["CURSOR_NUMBER"].as_deref(), Some("1"));
        let mut values = variables(&context, &snapshot, &requested).unwrap();
        assert_eq!(values["CURSOR_INDEX"].as_deref(), Some("0"));
        assert_eq!(values["CURSOR_NUMBER"].as_deref(), Some("1"));
        for index in [1, 17, 4095] {
            bind_cursor_variables(&mut values, &requested, index).unwrap();
            assert_eq!(values["CURSOR_INDEX"], Some(index.to_string()));
            assert_eq!(values["CURSOR_NUMBER"], Some((index + 1).to_string()));
            assert_eq!(values["CLIPBOARD"].as_deref(), Some("captured once"));
        }
        let before = values.clone();
        for index in [4096, usize::MAX] {
            assert!(bind_cursor_variables(&mut values, &requested, index).is_err());
            assert_eq!(values, before)
        }
        bind_cursor_variables(&mut values, &BTreeSet::new(), 0).unwrap();
        assert_eq!(values["CURSOR_INDEX"], None);
        assert_eq!(values["CURSOR_NUMBER"], None);
        assert_eq!(values["CLIPBOARD"].as_deref(), Some("captured once"));
    }
}

pub fn variable_presence(
    context: &SnippetVariableContext<'_>,
    snapshot: &SnippetVariableSnapshot,
    requested: &BTreeSet<String>,
) -> Result<BTreeMap<String, Option<String>>, String> {
    read_variables(context, snapshot, requested, true, None)
}
pub fn variables(
    context: &SnippetVariableContext<'_>,
    snapshot: &SnippetVariableSnapshot,
    requested: &BTreeSet<String>,
) -> Result<BTreeMap<String, Option<String>>, String> {
    read_variables(context, snapshot, requested, false, None)
}
/// Share requested-value allocation and source scanning across the complete reading.
pub fn variable_presence_with_budget(
    context: &SnippetVariableContext<'_>,
    snapshot: &SnippetVariableSnapshot,
    requested: &BTreeSet<String>,
    budget: &mut crate::snippet::SnippetExpansionBudget,
) -> Result<BTreeMap<String, Option<String>>, String> {
    read_variables(context, snapshot, requested, true, Some(budget))
}
pub fn variables_with_budget(
    context: &SnippetVariableContext<'_>,
    snapshot: &SnippetVariableSnapshot,
    requested: &BTreeSet<String>,
    budget: &mut crate::snippet::SnippetExpansionBudget,
) -> Result<BTreeMap<String, Option<String>>, String> {
    read_variables(context, snapshot, requested, false, Some(budget))
}
fn read_variables(
    context: &SnippetVariableContext<'_>,
    snapshot: &SnippetVariableSnapshot,
    requested: &BTreeSet<String>,
    presence_only: bool,
    budget: Option<&mut crate::snippet::SnippetExpansionBudget>,
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
    let budget = std::cell::RefCell::new(budget);
    if let Some(budget) = budget.borrow_mut().as_deref_mut() {
        budget.charge_context_steps(KNOWN.len() + requested.len())?;
    }
    let scan = |bytes: usize| -> Result<(), String> {
        if let Some(budget) = budget.borrow_mut().as_deref_mut() {
            budget.charge_context_scan(bytes)?
        };
        Ok(())
    };
    let mut values: BTreeMap<String, Option<String>> =
        KNOWN.iter().map(|name| ((*name).into(), None)).collect();
    let mut insert = |name: &str, value: Option<&str>| -> Result<(), String> {
        if requested.contains(name) {
            if !presence_only && value.is_some_and(|value| value.len() > MAX_VALUE) {
                return Err(format!(
                    "Snippet context {name} exceeds the expansion limit"
                ));
            }
            if !presence_only {
                if let (Some(value), Some(budget)) = (value, budget.borrow_mut().as_deref_mut()) {
                    budget.charge_context_bytes(value.len())?;
                }
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
        if presence_only {
            insert("TM_CURRENT_LINE", Some(""))?;
        } else {
            let mut start = context.from;
            let mut end = context.from;
            for (at, ch) in source[..context.from].char_indices().rev() {
                scan(ch.len_utf8())?;
                if ch == '\r' || ch == '\n' {
                    start = at + ch.len_utf8();
                    break;
                };
                start = at;
                if context.from - start > MAX_VALUE {
                    return Err(
                        "Snippet context TM_CURRENT_LINE exceeds the expansion limit".into(),
                    );
                }
            }
            for (at, ch) in source[context.from..].char_indices() {
                scan(ch.len_utf8())?;
                if ch == '\r' || ch == '\n' {
                    break;
                };
                end = context.from + at + ch.len_utf8();
                if end - start > MAX_VALUE {
                    return Err(
                        "Snippet context TM_CURRENT_LINE exceeds the expansion limit".into(),
                    );
                }
            }
            insert("TM_CURRENT_LINE", Some(&source[start..end]))?;
        }
    }
    if requested.contains("TM_CURRENT_WORD") {
        if presence_only {
            insert("TM_CURRENT_WORD", Some(""))?;
        } else {
            let word = |ch: char| crate::snippet_case::contains("identifier", ch as u32);
            let mut start = context.from;
            let mut end = context.from;
            for (at, ch) in source[..context.from].char_indices().rev() {
                scan(ch.len_utf8())?;
                if !word(ch) {
                    break;
                };
                start = at;
                if context.from - start > MAX_VALUE {
                    return Err(
                        "Snippet context TM_CURRENT_WORD exceeds the expansion limit".into(),
                    );
                }
            }
            for (at, ch) in source[context.from..].char_indices() {
                scan(ch.len_utf8())?;
                if !word(ch) {
                    break;
                };
                end = context.from + at + ch.len_utf8();
                if end - start > MAX_VALUE {
                    return Err(
                        "Snippet context TM_CURRENT_WORD exceeds the expansion limit".into(),
                    );
                }
            }
            insert("TM_CURRENT_WORD", Some(&source[start..end]))?;
        }
    }
    if requested.contains("TM_LINE_INDEX") || requested.contains("TM_LINE_NUMBER") {
        if presence_only {
            insert("TM_LINE_INDEX", Some(""))?;
            insert("TM_LINE_NUMBER", Some(""))?;
        } else {
            scan(context.from)?;
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
        }
    }
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
    bind_cursor_variables(&mut values, requested, 0)?;
    Ok(values)
}
#[cfg(test)]
mod context_budget_witnesses {
    use super::*;
    use crate::snippet::SnippetExpansionBudget;
    fn requested(names: &[&str]) -> BTreeSet<String> {
        names.iter().map(|s| s.to_string()).collect()
    }
    fn snapshot() -> SnippetVariableSnapshot {
        SnippetVariableSnapshot {
            date: None,
            random: None,
            clipboard: None,
        }
    }
    #[test]
    fn unused_large_prefix_and_presence_do_not_scan_or_copy_values() {
        let source = format!("{}\nname", "x".repeat(8 * MAX_VALUE + 2));
        let context = SnippetVariableContext {
            source: &source,
            from: 8 * MAX_VALUE + 2,
            to: 8 * MAX_VALUE + 2,
            path: Path::new("/work/main.py"),
            workspace: Some(Path::new("/work")),
            language: Some("python"),
        };
        let mut budget = SnippetExpansionBudget::default();
        let names = requested(&["TM_FILENAME"]);
        assert_eq!(variables_with_budget(&context,&snapshot(),&names,&mut budget).unwrap()["TM_FILENAME"].as_deref(),Some("main.py"));
        assert!(!budget.exhausted());
        let names = requested(&["TM_CURRENT_WORD", "TM_CURRENT_LINE", "TM_LINE_INDEX"]);
        let values =
            variable_presence_with_budget(&context, &snapshot(), &names, &mut budget).unwrap();
        for name in &names {
            assert!(values[name].is_some())
        }
        assert!(!budget.exhausted());
        let mut line_budget = SnippetExpansionBudget::default();
        assert!(
            variables_with_budget(
                &context,
                &snapshot(),
                &requested(&["TM_LINE_INDEX"]),
                &mut line_budget
            )
            .is_err()
        );
        assert!(line_budget.exhausted());
        let mut word_budget = SnippetExpansionBudget::default();
        assert!(
            variables_with_budget(
                &context,
                &snapshot(),
                &requested(&["TM_CURRENT_WORD"]),
                &mut word_budget
            )
            .is_err()
        );
        assert!(!word_budget.exhausted());
    }
    #[test]
    fn requested_values_charge_before_copy_and_line_boundaries_are_preserved() {
        let source = "pre\r\nname\rnext\n";
        let mut context = SnippetVariableContext {
            source,
            from: 7,
            to: 8,
            path: Path::new("/work/main.py"),
            workspace: None,
            language: Some("python"),
        };
        let names = requested(&[
            "TM_CURRENT_LINE",
            "TM_CURRENT_WORD",
            "TM_LINE_INDEX",
            "TM_LINE_NUMBER",
        ]);
        let mut budget = SnippetExpansionBudget::default();
        let values = variables_with_budget(&context, &snapshot(), &names, &mut budget).unwrap();
        assert_eq!(values["TM_CURRENT_LINE"].as_deref(), Some("name"));
        assert_eq!(values["TM_CURRENT_WORD"].as_deref(), Some("name"));
        assert_eq!(values["TM_LINE_INDEX"].as_deref(), Some("1"));
        assert_eq!(values["TM_LINE_NUMBER"].as_deref(), Some("2"));
        context.from = 12;
        context.to = 12;
        let values = variables_with_budget(&context, &snapshot(), &names, &mut budget).unwrap();
        assert_eq!(values["TM_CURRENT_LINE"].as_deref(), Some("next"));
        assert_eq!(values["TM_LINE_INDEX"].as_deref(), Some("2"));
        let mut limited = SnippetExpansionBudget::default();
        limited.charge_context_bytes(MAX_VALUE - 1).unwrap();
        assert!(
            variables_with_budget(
                &context,
                &snapshot(),
                &requested(&["TM_CURRENT_WORD"]),
                &mut limited
            )
            .is_err()
        );
        assert!(limited.exhausted());
        assert_eq!(context.source, source)
    }
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
        assert_eq!(context.caret, None);
        let mut grouped = raw.clone();
        grouped["carets"] = serde_json::json!({"index":1,"count":2});
        assert_eq!(
            SnippetEditorContext::read(&grouped, source, "sha256:held", "held-revision")
                .unwrap().caret,
            Some((1, 2))
        );
        for invalid in [
            serde_json::json!({"index":2,"count":2}),
            serde_json::json!({"index":0,"count":1}),
            serde_json::json!({"index":0,"count":17}),
            serde_json::json!({"index":false,"count":2}),
        ] {
            grouped["carets"] = invalid;
            assert!(SnippetEditorContext::read(&grouped, source, "sha256:held", "held-revision").is_err());
        }
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
