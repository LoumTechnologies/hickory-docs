//! Frozen Unicode source-text mappings, independent of the host locale and OS.
use std::{collections::HashMap, sync::OnceLock};
pub const VERSION: &str = "hickory-ucd-16.0.0-v1";
struct Data {
    upper: Vec<(u32, String)>,
    lower: Vec<(u32, String)>,
    letter: Vec<[u32; 2]>,
    cased: Vec<[u32; 2]>,
    ignorable: Vec<[u32; 2]>,
    identifier: Vec<[u32; 2]>,
    fold: Vec<(u32, Vec<u32>)>,
}
struct CaseMap {
    data: Data,
    upper: HashMap<u32, String>,
    lower: HashMap<u32, String>,
}
fn table() -> &'static CaseMap {
    static TABLE: OnceLock<CaseMap> = OnceLock::new();
    TABLE.get_or_init(|| {
        let raw: serde_json::Value = serde_json::from_str(include_str!(
            "../../../apps/simple-contract/snippet-case-map.json"
        ))
        .expect("frozen case map");
        assert_eq!(raw["version"], VERSION);
        let ranges = |key: &str| {
            raw[key]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| [r[0].as_u64().unwrap() as u32, r[1].as_u64().unwrap() as u32])
                .collect()
        };
        let mappings = |key: &str| {
            raw[key]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| {
                    (
                        r[0].as_u64().unwrap() as u32,
                        r[1].as_str().unwrap().to_owned(),
                    )
                })
                .collect()
        };
        let data = Data {
            upper: mappings("upper"),
            lower: mappings("lower"),
            letter: ranges("letter"),
            cased: ranges("cased"),
            ignorable: ranges("ignorable"),
            identifier: ranges("identifier"),
            fold: raw["fold"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| {
                    (
                        r[0].as_u64().unwrap() as u32,
                        r[1].as_array()
                            .unwrap()
                            .iter()
                            .map(|v| v.as_u64().unwrap() as u32)
                            .collect(),
                    )
                })
                .collect(),
        };
        let upper = data.upper.iter().cloned().collect();
        let lower = data.lower.iter().cloned().collect();
        CaseMap { data, upper, lower }
    })
}
pub fn contains(kind: &str, scalar: u32) -> bool {
    let d = &table().data;
    let ranges = match kind {
        "letter" => &d.letter,
        "cased" => &d.cased,
        "ignorable" => &d.ignorable,
        "identifier" => &d.identifier,
        _ => return false,
    };
    ranges
        .binary_search_by(|r| {
            if scalar < r[0] {
                std::cmp::Ordering::Greater
            } else if scalar > r[1] {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Equal
            }
        })
        .is_ok()
}
pub fn convert(value: &str, upper: bool) -> String {
    let chars: Vec<char> = value.chars().collect();
    let map = if upper {
        &table().upper
    } else {
        &table().lower
    };
    let mut result = String::new();
    for (index, &ch) in chars.iter().enumerate() {
        if !upper && ch == 'Σ' {
            let before = chars[..index]
                .iter()
                .rev()
                .find(|c| !contains("ignorable", **c as u32));
            let after = chars[index + 1..]
                .iter()
                .find(|c| !contains("ignorable", **c as u32));
            if before.is_some_and(|c| contains("cased", *c as u32))
                && !after.is_some_and(|c| contains("cased", *c as u32))
            {
                result.push('ς');
                continue;
            }
        }
        if let Some(text) = map.get(&(ch as u32)) {
            result.push_str(text)
        } else {
            result.push(ch)
        }
    }
    result
}
pub fn words(value: &str) -> Vec<String> {
    let mut words = vec![];
    let mut current = String::new();
    for ch in value.chars() {
        if contains("letter", ch as u32) || ch.is_ascii_digit() {
            current.push(ch)
        } else if !current.is_empty() {
            words.push(std::mem::take(&mut current))
        }
    }
    if !current.is_empty() {
        words.push(current)
    }
    words
}
pub fn simple_folds() -> &'static [(u32, Vec<u32>)] {
    &table().data.fold
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frozen_unicode_cases() {
        let vectors: serde_json::Value = serde_json::from_str(include_str!(
            "../../../apps/simple-contract/snippet-case-cases.json"
        ))
        .unwrap();
        assert_eq!(vectors["case_map"], VERSION);
        for c in vectors["cases"].as_array().unwrap() {
            assert_eq!(
                convert(c["value"].as_str().unwrap(), c["mode"] == "upper"),
                c["expected"].as_str().unwrap()
            )
        }
        for c in vectors["words"].as_array().unwrap() {
            assert_eq!(
                words(c["value"].as_str().unwrap()),
                c["expected"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap())
                    .collect::<Vec<_>>()
            )
        }
    }
}
