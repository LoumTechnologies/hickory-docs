<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Preserve ECMAScript transform semantics with bounded data programs

This reading is for the maintainer changing snippet regex behavior. Version 3 closes the legacy portable subset: lookaround, named and numeric references, scoped modifiers, Unicode property and string sets, and standard pattern flags compile into data. Matching uses ordered explicit backtracking rather than the host regex library. Lookbehind compiles in reverse order; assertion subprograms are atomic. Repeated groups clear captures before each new iteration, including failed alternatives.

This is implementation infrastructure until the supported-program handshake is wired and real provider/template paths pass. Existing version 2 replay remains available. No initialization capability or menu constitutes acceptance of a version 3 source action.

## Bound syntax before invoking the parser

Oxc supplies the ECMAScript grammar; it never evaluates code. The byte/nesting preflight runs first, then capture, AST and emitted-instruction limits. The small legacy class-control normalization preserves Annex B behavior where the pinned parser differs; Unicode mode still rejects those legacy escapes. These parser quirks have independent runtime oracle cases. Escaped named identifiers are decoded and validated against frozen Unicode17 ID_Start/ID_Continue, then mapped injectively to private ASCII names. This works around the parser comparing escape spellings and preserves duplicate-name/reference semantics without allowing an escape to become pattern syntax. UTF-16 escape pairs and Unicode17 identifier additions have focused positive and negative witnesses.
<ui:copy id="snippet-regex-v3-lowering">
//! Structural ECMAScript syntax lowering; never executes provider code.
use oxc_allocator::Allocator;
use oxc_regular_expression::{LiteralParser, Options, ast::*};
use serde_json::{Value, json};
use std::collections::BTreeMap;
const MAX_SYNTAX: usize = 8192;
const MAX_NODES: usize = 512;
const MAX_CAPTURES: usize = 32;

pub struct LoweredPattern {
    pub tree: Value,
    pub groups: usize,
    pub names: BTreeMap<String, Vec<u32>>,
}
fn preflight(pattern: &str, flags: &str) -> Result<(), String> {
    if pattern.len() > MAX_SYNTAX || flags.len() > 8 {
        return Err("Regex syntax byte budget exceeded".into());
    }
    let sets = flags.contains('v');
    let mut escaped = false;
    let mut classes = 0usize;
    let mut groups = 0usize;
    for ch in pattern.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '[' && (classes == 0 || sets) {
            classes += 1;
            if classes > 32 {
                return Err("Regex class nesting budget exceeded".into());
            }
            continue;
        }
        if ch == ']' && classes > 0 {
            classes -= 1;
            continue;
        }
        if classes > 0 {
            continue;
        }
        if ch == '(' {
            groups += 1;
            if groups > 32 {
                return Err("Regex group nesting budget exceeded".into());
            }
        } else if ch == ')' {
            groups = groups.saturating_sub(1)
        }
    }
    Ok(())
}
struct Lowerer {
    indices: BTreeMap<u32, u32>,
    names: BTreeMap<String, Vec<u32>>,
    nodes: usize,
}
impl Lowerer {
    fn inventory(&mut self, body: &Disjunction<'_>) -> Result<(), String> {
        for alternative in &body.body {
            for term in &alternative.body {
                self.inventory_term(term)?
            }
        }
        Ok(())
    }
    fn inventory_term(&mut self, term: &Term<'_>) -> Result<(), String> {
        match term {
            Term::CapturingGroup(group) => {
                let index = self.indices.len() as u32 + 1;
                if index as usize > MAX_CAPTURES {
                    return Err("Regex capture budget exceeded".into());
                }
                self.indices.insert(group.span.start, index);
                if let Some(name) = &group.name {
                    self.names.entry(name.to_string()).or_default().push(index)
                }
                self.inventory(&group.body)?
            }
            Term::IgnoreGroup(group) => self.inventory(&group.body)?,
            Term::LookAroundAssertion(group) => self.inventory(&group.body)?,
            Term::Quantifier(repeat) => self.inventory_term(&repeat.body)?,
            _ => {}
        }
        Ok(())
    }
    fn node(&mut self, value: Value) -> Result<Value, String> {
        self.nodes += 1;
        if self.nodes > MAX_NODES {
            return Err("Regex AST budget exceeded".into());
        }
        Ok(value)
    }
    fn disjunction(&mut self, body: &Disjunction<'_>) -> Result<Value, String> {
        let alternatives = body
            .body
            .iter()
            .map(|alternative| {
                let terms = alternative
                    .body
                    .iter()
                    .map(|term| self.term(term))
                    .collect::<Result<Vec<_>, _>>()?;
                self.node(json!(["seq", terms]))
            })
            .collect::<Result<Vec<_>, String>>()?;
        self.node(json!(["alt", alternatives]))
    }
    fn term(&mut self, term: &Term<'_>) -> Result<Value, String> {
        let value = match term {
            Term::BoundaryAssertion(value) => json!([
                "assert",
                match value.kind {
                    BoundaryAssertionKind::Start => "start",
                    BoundaryAssertionKind::End => "end",
                    BoundaryAssertionKind::Boundary => "word",
                    BoundaryAssertionKind::NegativeBoundary => "nonword",
                }
            ]),
            Term::LookAroundAssertion(value) => json!([
                "look",
                self.disjunction(&value.body)?,
                matches!(
                    value.kind,
                    LookAroundAssertionKind::Lookahead | LookAroundAssertionKind::Lookbehind
                ),
                matches!(
                    value.kind,
                    LookAroundAssertionKind::Lookbehind
                        | LookAroundAssertionKind::NegativeLookbehind
                )
            ]),
            Term::Quantifier(value) => {
                if value.min > 9_007_199_254_740_991
                    || value.max.is_some_and(|n| n > 9_007_199_254_740_991)
                {
                    return Err("Regex repetition integer budget exceeded".into());
                }
                json!([
                    "repeat",
                    self.term(&value.body)?,
                    value.min,
                    value.max,
                    !value.greedy
                ])
            }
            Term::Character(value) => json!(["char", value.value]),
            Term::Dot(_) => json!(["dot"]),
            Term::CharacterClassEscape(value) => Self::escape(value.kind),
            Term::UnicodePropertyEscape(value) => Self::property(value),
            Term::CharacterClass(value) => self.class(value)?,
            Term::CapturingGroup(value) => json!([
                "capture",
                self.indices[&value.span.start],
                self.disjunction(&value.body)?
            ]),
            Term::IgnoreGroup(value) => {
                let body = self.disjunction(&value.body)?;
                if let Some(modifiers) = &value.modifiers {
                    let flags = |value: Modifier| {
                        let mut result = String::new();
                        for (bit, ch) in
                            [(Modifier::I, 'i'), (Modifier::M, 'm'), (Modifier::S, 's')]
                        {
                            if value.contains(bit) {
                                result.push(ch)
                            }
                        }
                        result
                    };
                    json!([
                        "scope",
                        body,
                        flags(modifiers.enabling),
                        flags(modifiers.disabling)
                    ])
                } else {
                    body
                }
            }
            Term::IndexedReference(value) => json!(["ref", [value.index]]),
            Term::NamedReference(value) => json!([
                "ref",
                self.names
                    .get(value.name.as_str())
                    .ok_or("Unresolved named regex reference")?
            ]),
        };
        self.node(value)
    }
    fn escape(kind: CharacterClassEscapeKind) -> Value {
        json!([
            "escape",
            match kind {
                CharacterClassEscapeKind::D => "d",
                CharacterClassEscapeKind::NegativeD => "D",
                CharacterClassEscapeKind::S => "s",
                CharacterClassEscapeKind::NegativeS => "S",
                CharacterClassEscapeKind::W => "w",
                CharacterClassEscapeKind::NegativeW => "W",
            }
        ])
    }
    fn property(value: &UnicodePropertyEscape<'_>) -> Value {
        json!([
            "property",
            value.name.as_str(),
            value.value.as_ref().map(|s| s.as_str()),
            value.negative,
            value.strings
        ])
    }
    fn class(&mut self, value: &CharacterClass<'_>) -> Result<Value, String> {
        let mut items = vec![];
        for item in &value.body {
            items.push(match item {
                CharacterClassContents::CharacterClassRange(value) => {
                    json!(["range", value.min.value, value.max.value])
                }
                CharacterClassContents::Character(value) => {
                    json!(["range", value.value, value.value])
                }
                CharacterClassContents::CharacterClassEscape(value) => Self::escape(value.kind),
                CharacterClassContents::UnicodePropertyEscape(value) => Self::property(value),
                CharacterClassContents::NestedCharacterClass(value) => self.class(value)?,
                CharacterClassContents::ClassStringDisjunction(value) => json!([
                    "strings",
                    value
                        .body
                        .iter()
                        .map(|text| text.body.iter().map(|ch| ch.value).collect::<Vec<_>>())
                        .collect::<Vec<_>>()
                ]),
            });
            self.nodes += 1;
            if self.nodes > MAX_NODES {
                return Err("Regex class AST budget exceeded".into());
            }
        }
        self.node(json!([
            "class_set",
            match value.kind {
                CharacterClassContentsKind::Union => "union",
                CharacterClassContentsKind::Intersection => "intersection",
                CharacterClassContentsKind::Subtraction => "subtraction",
            },
            value.negative,
            items
        ]))
    }
}
// Annex B ClassControlLetter includes decimal digits and underscore. The pinned
// parser currently preserves these as literal characters; normalize only this
// exact legacy class production before feeding its otherwise complete AST.
fn legacy_class_controls(pattern: &str, flags: &str) -> Result<String, String> {
    let unicode = flags.contains('u') || flags.contains('v');
    let chars = pattern.chars().collect::<Vec<_>>();
    let mut output = String::new();
    let mut at = 0;
    let mut class = false;
    while at < chars.len() {
        let ch = chars[at];
        if ch == '\\' && at + 1 < chars.len() {
            if class
                && chars[at + 1] == 'c'
                && at + 2 < chars.len()
                && (chars[at + 2].is_ascii_digit() || chars[at + 2] == '_')
            {
                if unicode {
                    return Err("Invalid ECMAScript Unicode class-control escape".into());
                }
                output.push_str(&format!("\\x{:02X}", (chars[at + 2] as u32) % 32));
                at += 3;
                continue;
            }
            output.push(ch);
            output.push(chars[at + 1]);
            at += 2;
            continue;
        }
        if ch == '[' && !class {
            class = true
        } else if ch == ']' && class {
            class = false
        }
        output.push(ch);
        at += 1;
    }
    Ok(output)
}
pub fn lower_pattern(
    pattern: &str,
    flags: &str,
    data: &super::sets::UnicodeData,
) -> Result<LoweredPattern, String> {
    preflight(pattern, flags)?;
    let normalized = legacy_class_controls(pattern, flags)?;
    let normalized = normalize_named_identifiers(&normalized, flags, data)?;
    let arena = Allocator::default();
    let ast = LiteralParser::new(&arena, &normalized, Some(flags), Options::default())
        .parse()
        .map_err(|error| format!("Invalid ECMAScript regex: {error}"))?;
    let mut lower = Lowerer {
        indices: BTreeMap::new(),
        names: BTreeMap::new(),
        nodes: 0,
    };
    lower.inventory(&ast.body)?;
    let tree = lower.disjunction(&ast.body)?;
    Ok(LoweredPattern {
        tree,
        groups: lower.indices.len(),
        names: lower.names,
    })
}

/// Oxc retains escape spellings in names. Canonicalize validated Unicode17 names
/// to private ASCII identifiers so references and duplicate-name rules agree.
fn normalize_named_identifiers(
    pattern: &str,
    flags: &str,
    data: &super::sets::UnicodeData,
) -> Result<String, String> {
    fn scan_name(pattern: &str, start: usize) -> Result<(String, usize), String> {
        let end = pattern[start..]
            .find('>')
            .map(|n| start + n)
            .ok_or("Unclosed regex named identifier")?;
        let raw = &pattern[start..end];
        let bytes = raw.as_bytes();
        let mut at = 0;
        let mut result = String::new();
        fn hex4(bytes: &[u8], at: usize) -> Result<u32, String> {
            let value = bytes
                .get(at..at + 4)
                .ok_or("Incomplete regex identifier escape")?;
            let mut point = 0;
            for b in value {
                point = point * 16
                    + (*b as char)
                        .to_digit(16)
                        .filter(|_| b.is_ascii())
                        .ok_or("Invalid regex identifier escape")?
            }
            Ok(point)
        }
        while at < bytes.len() {
            if bytes[at] != b'\\' {
                let ch = raw[at..].chars().next().unwrap();
                result.push(ch);
                at += ch.len_utf8();
                continue;
            };
            if bytes.get(at + 1) != Some(&b'u') {
                return Err("Invalid regex identifier escape".into());
            };
            at += 2;
            let point = if bytes.get(at) == Some(&b'{') {
                at += 1;
                let begin = at;
                let mut point = 0u32;
                while bytes.get(at).is_some_and(|b| *b != b'}') {
                    let b = bytes[at];
                    let digit = (b as char)
                        .to_digit(16)
                        .filter(|_| b.is_ascii())
                        .ok_or("Invalid regex identifier escape")?;
                    point = point
                        .checked_mul(16)
                        .and_then(|p| p.checked_add(digit))
                        .filter(|p| *p <= 0x10ffff)
                        .ok_or("Invalid regex identifier code point")?;
                    at += 1
                }
                if at == begin || bytes.get(at) != Some(&b'}') {
                    return Err("Invalid regex identifier escape".into());
                };
                at += 1;
                point
            } else {
                let first = hex4(bytes, at)?;
                at += 4;
                if (0xd800..=0xdbff).contains(&first) {
                    if bytes.get(at..at + 2) != Some(&b"\\u"[..]) {
                        return Err("Unpaired regex identifier surrogate".into());
                    };
                    let second = hex4(bytes, at + 2)?;
                    if !(0xdc00..=0xdfff).contains(&second) {
                        return Err("Unpaired regex identifier surrogate".into());
                    };
                    at += 6;
                    0x10000 + ((first - 0xd800) << 10) + (second - 0xdc00)
                } else {
                    first
                }
            };
            result.push(char::from_u32(point).ok_or("Invalid regex identifier scalar")?)
        }
        Ok((result, end))
    }
    let bytes = pattern.as_bytes();
    let sets = flags.contains('v');
    let mut at = 0;
    let mut classes = 0usize;
    let mut named = false;
    while at < bytes.len() {
        let b = bytes[at];
        if b == b'\\' {
            at += 1;
            if at < bytes.len() {
                at += pattern[at..].chars().next().unwrap().len_utf8()
            };
            continue;
        };
        if b == b'[' && (classes == 0 || sets) {
            classes += 1
        } else if b == b']' && classes > 0 {
            classes -= 1
        } else if classes == 0
            && bytes.get(at..at + 3) == Some(&b"(?<"[..])
            && !matches!(bytes.get(at + 3), Some(b'=' | b'!'))
        {
            named = true;
            break;
        };
        at += pattern[at..].chars().next().unwrap().len_utf8()
    }
    if !named {
        return Ok(pattern.to_owned());
    };
    let mut names = BTreeMap::<String, String>::new();
    let mut result = String::new();
    at = 0;
    classes = 0;
    let mut copied = 0;
    while at < bytes.len() {
        let b = bytes[at];
        let capture = classes == 0
            && bytes.get(at..at + 3) == Some(&b"(?<"[..])
            && !matches!(bytes.get(at + 3), Some(b'=' | b'!'));
        let reference = classes == 0 && bytes.get(at..at + 3) == Some(&b"\\k<"[..]);
        if capture || reference {
            let start = at + 3;
            let (name, end) = scan_name(pattern, start)?;
            if !data.identifier_name(&name) {
                return Err("Invalid regex named identifier".into());
            };
            let next = names.len();
            let replacement = names.entry(name).or_insert_with(|| format!("__h{next}"));
            result.push_str(&pattern[copied..start]);
            result.push_str(replacement);
            copied = end;
            at = end + 1;
            if result.len() > 4 * MAX_SYNTAX {
                return Err("Normalized regex syntax budget exceeded".into());
            };
            continue;
        };
        if b == b'\\' {
            at += 1;
            if at < bytes.len() {
                at += pattern[at..].chars().next().unwrap().len_utf8()
            };
            continue;
        };
        if b == b'[' && (classes == 0 || sets) {
            classes += 1
        } else if b == b']' && classes > 0 {
            classes -= 1
        };
        at += pattern[at..].chars().next().unwrap().len_utf8()
    }
    result.push_str(&pattern[copied..]);
    if result.len() > 4 * MAX_SYNTAX {
        return Err("Normalized regex syntax budget exceeded".into());
    };
    Ok(result)
}
</ui:copy>

## Materialize Unicode sets once

Property aliases and v-mode intersection/subtraction operate on the frozen engine catalog. The engine computes simple-fold closure in the correct order for u versus v complement. Class strings retain longest-first priority. Hosts receive normalized ranges and literal point sequences.

<ui:copy id="snippet-regex-v3-sets">
//! Engine-side Unicode17 materialization. Hosts receive normalized ranges/strings.
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
type Range = (u32, u32);
#[derive(Clone, Default)]
struct Set {
    ranges: Vec<Range>,
    strings: BTreeSet<Vec<u32>>,
}
fn normalized(mut ranges: Vec<Range>) -> Vec<Range> {
    ranges.sort_unstable();
    let mut result: Vec<Range> = vec![];
    for (low, high) in ranges {
        if low > high {
            continue;
        }
        if let Some(last) = result.last_mut() {
            if low <= last.1 + 1 {
                last.1 = last.1.max(high);
                continue;
            }
        }
        result.push((low, high));
    }
    result
}
fn has(ranges: &[Range], point: u32) -> bool {
    let index = ranges.partition_point(|(low, _)| *low <= point);
    index > 0 && point <= ranges[index - 1].1
}
fn intersect(a: &[Range], b: &[Range]) -> Vec<Range> {
    let (mut x, mut y) = (0, 0);
    let mut result = vec![];
    while x < a.len() && y < b.len() {
        let low = a[x].0.max(b[y].0);
        let high = a[x].1.min(b[y].1);
        if low <= high {
            result.push((low, high))
        }
        if a[x].1 < b[y].1 { x += 1 } else { y += 1 }
    }
    result
}
fn subtract(a: &[Range], b: &[Range]) -> Vec<Range> {
    let mut result = vec![];
    let mut index = 0;
    for &(low, high) in a {
        let mut cursor = low;
        while index < b.len() && b[index].1 < cursor {
            index += 1
        }
        let mut scan = index;
        while scan < b.len() && b[scan].0 <= high {
            if b[scan].0 > cursor {
                result.push((cursor, b[scan].0 - 1))
            }
            cursor = cursor.max(b[scan].1 + 1);
            if cursor > high {
                break;
            }
            scan += 1
        }
        if cursor <= high {
            result.push((cursor, high))
        }
    }
    result
}
impl Set {
    fn clean(mut self) -> Self {
        let singles = self
            .strings
            .iter()
            .filter(|value| value.len() == 1)
            .map(|value| (value[0], value[0]))
            .collect::<Vec<_>>();
        self.ranges.extend(singles);
        self.strings.retain(|value| value.len() != 1);
        self.ranges = normalized(self.ranges);
        self
    }
    fn union(self, other: Self) -> Self {
        Self {
            ranges: normalized([self.ranges, other.ranges].concat()),
            strings: self.strings.union(&other.strings).cloned().collect(),
        }
    }
    fn intersection(self, other: Self) -> Self {
        Self {
            ranges: intersect(&self.ranges, &other.ranges),
            strings: self.strings.intersection(&other.strings).cloned().collect(),
        }
    }
    fn subtract(self, other: Self) -> Self {
        Self {
            ranges: subtract(&self.ranges, &other.ranges),
            strings: self.strings.difference(&other.strings).cloned().collect(),
        }
    }
    fn complement(self, maximum: u32) -> Result<Self, String> {
        if !self.strings.is_empty() {
            return Err("Cannot complement a string-valued regex set".into());
        }
        Ok(Self {
            ranges: subtract(&[(0, maximum)], &self.ranges),
            strings: BTreeSet::new(),
        })
    }
}
pub struct UnicodeData {
    properties: Value,
    identifier_start: Vec<Range>,
    identifier_continue: Vec<Range>,
    fold_unicode: BTreeMap<u32, u32>,
    fold_legacy: BTreeMap<u32, u32>,
    unicode_groups: Vec<Vec<u32>>,
    legacy_groups: Vec<Vec<u32>>,
}
impl UnicodeData {
    pub fn new(properties: Value, case: &Value) -> Result<Self, String> {
        if properties["unicode"] != "17.0.0" || case["unicode"] != "17.0.0" {
            return Err("Wrong regex Unicode data version".into());
        }
        let mut fold_unicode = BTreeMap::new();
        let mut unique = BTreeSet::new();
        for row in case["fold"]
            .as_array()
            .ok_or("Missing regex folding data")?
        {
            let cp = row[0].as_u64().unwrap() as u32;
            let mut group = row[1]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_u64().unwrap() as u32)
                .collect::<Vec<_>>();
            group.push(cp);
            group.sort_unstable();
            group.dedup();
            for point in &group {
                fold_unicode.insert(*point, group[0]);
            }
            unique.insert(group);
        }
        let unicode_groups = unique.into_iter().collect();
        let mut fold_legacy = BTreeMap::new();
        let mut groups: BTreeMap<u32, BTreeSet<u32>> = BTreeMap::new();
        for row in case["upper"]
            .as_array()
            .ok_or("Missing regex uppercase data")?
        {
            let cp = row[0].as_u64().unwrap() as u32;
            if cp > 65535 {
                continue;
            }
            let text = row[1].as_str().unwrap();
            let units = text.encode_utf16().collect::<Vec<_>>();
            if units.len() != 1 {
                continue;
            }
            let canonical = units[0] as u32;
            if cp >= 128 && canonical < 128 {
                continue;
            }
            fold_legacy.insert(cp, canonical);
            let group = groups.entry(canonical).or_default();
            group.insert(cp);
            group.insert(canonical);
        }
        let legacy_groups = groups
            .into_values()
            .map(|group| group.into_iter().collect())
            .collect();
        let identifiers = |name: &str| {
            case[name]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| (r[0].as_u64().unwrap() as u32, r[1].as_u64().unwrap() as u32))
                .collect()
        };
        Ok(Self {
            identifier_start: identifiers("identifier_start"),
            identifier_continue: identifiers("identifier_continue"),
            properties,
            fold_unicode,
            fold_legacy,
            unicode_groups,
            legacy_groups,
        })
    }
    pub fn identifier_name(&self, name: &str) -> bool {
        let mut chars = name.chars();
        let Some(first) = chars.next() else {
            return false;
        };
        (first == '$' || first == '_' || has(&self.identifier_start, first as u32))
            && chars.all(|ch| {
                ch == '$'
                    || ch == '_'
                    || ch == '\u{200c}'
                    || ch == '\u{200d}'
                    || has(&self.identifier_continue, ch as u32)
            })
    }
    pub fn canonical(&self, cp: u32, unicode: bool) -> u32 {
        if unicode {
            *self.fold_unicode.get(&cp).unwrap_or(&cp)
        } else {
            *self.fold_legacy.get(&cp).unwrap_or(&cp)
        }
    }
    fn folded(&self, mut set: Set, unicode: bool) -> Set {
        let groups = if unicode {
            &self.unicode_groups
        } else {
            &self.legacy_groups
        };
        let original = &set.ranges;
        let mut added = vec![];
        for group in groups {
            if group.iter().any(|point| has(original, *point)) {
                added.extend(group.iter().map(|point| (*point, *point)));
            }
        }
        set.ranges.extend(added);
        set.ranges = normalized(set.ranges);
        set.strings = set
            .strings
            .into_iter()
            .map(|text| {
                text.into_iter()
                    .map(|point| self.canonical(point, unicode))
                    .collect()
            })
            .collect();
        set
    }
    fn property(&self, name: &str, value: Option<&str>) -> Result<Set, String> {
        let raw = value.map_or_else(|| name.to_owned(), |value| format!("{name}={value}"));
        if let Some(data) = self.properties["strings"].get(&raw) {
            return Ok(Set {
                ranges: Self::ranges(&data["ranges"])?,
                strings: data["strings"]
                    .as_array()
                    .ok_or("Bad Unicode string data")?
                    .iter()
                    .map(|text| {
                        text.as_array()
                            .unwrap()
                            .iter()
                            .map(|point| point.as_u64().unwrap() as u32)
                            .collect()
                    })
                    .collect(),
            }
            .clean());
        }
        let canonical = self.properties["aliases"]
            .get(&raw)
            .and_then(Value::as_str)
            .ok_or_else(|| format!("Unknown exact Unicode property alias {raw}"))?;
        Ok(Set {
            ranges: Self::ranges(&self.properties["properties"][canonical])?,
            strings: BTreeSet::new(),
        })
    }
    fn ranges(raw: &Value) -> Result<Vec<Range>, String> {
        Ok(raw
            .as_array()
            .ok_or("Unicode property needs standards/oracle review")?
            .iter()
            .map(|row| {
                (
                    row[0].as_u64().unwrap() as u32,
                    row[1].as_u64().unwrap() as u32,
                )
            })
            .collect())
    }
    fn raw_set(&self, node: &Value, flags: &str) -> Result<Set, String> {
        let unicode = flags.contains('u') || flags.contains('v');
        let insensitive = flags.contains('i');
        let sets = flags.contains('v');
        let maximum = if unicode { 0x10ffff } else { 65535 };
        let fold = |set| {
            if insensitive {
                self.folded(set, unicode)
            } else {
                set
            }
        };
        let result = match node[0].as_str().ok_or("Malformed regex set IR")? {
            "char" => {
                let point = node[1].as_u64().ok_or("Bad regex character")? as u32;
                fold(Set {
                    ranges: vec![(point, point)],
                    strings: BTreeSet::new(),
                })
            }
            "range" => fold(Set {
                ranges: vec![(
                    node[1].as_u64().unwrap() as u32,
                    node[2].as_u64().unwrap() as u32,
                )],
                strings: BTreeSet::new(),
            }),
            "strings" => fold(
                Set {
                    ranges: vec![],
                    strings: node[1]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|text| {
                            text.as_array()
                                .unwrap()
                                .iter()
                                .map(|cp| cp.as_u64().unwrap() as u32)
                                .collect()
                        })
                        .collect(),
                }
                .clean(),
            ),
            "dot" => {
                let excluded = if flags.contains('s') {
                    vec![]
                } else {
                    vec![(10, 10), (13, 13), (0x2028, 0x2029)]
                };
                Set {
                    ranges: subtract(&[(0, maximum)], &excluded),
                    strings: BTreeSet::new(),
                }
            }
            "escape" => {
                let kind = node[1].as_str().unwrap();
                let mut base = match kind.to_ascii_lowercase().as_str() {
                    "d" => Set {
                        ranges: vec![(48, 57)],
                        strings: BTreeSet::new(),
                    },
                    "w" => Set {
                        ranges: vec![(48, 57), (65, 90), (95, 95), (97, 122)],
                        strings: BTreeSet::new(),
                    },
                    "s" => {
                        let mut value =
                            self.property("General_Category", Some("Space_Separator"))?;
                        value
                            .ranges
                            .extend([(9, 13), (0xfeff, 0xfeff), (0x2028, 0x2029)]);
                        value.ranges = normalized(value.ranges);
                        value
                    }
                    _ => return Err("Unknown regex shorthand".into()),
                };
                if kind.eq_ignore_ascii_case("w") && insensitive && unicode {
                    base = self.folded(base, true)
                }
                if kind.chars().next().unwrap().is_ascii_uppercase() {
                    base.complement(maximum)?
                } else {
                    base
                }
            }
            "property" => {
                let base = self.property(node[1].as_str().unwrap(), node[2].as_str())?;
                if node[3].as_bool().unwrap() {
                    if sets {
                        fold(base).complement(maximum)?
                    } else {
                        fold(base.complement(maximum)?)
                    }
                } else {
                    fold(base)
                }
            }
            "class_set" => {
                let operation = node[1].as_str().unwrap();
                let mut values = node[3].as_array().unwrap().iter();
                let mut result = values
                    .next()
                    .map(|value| self.raw_set(value, flags))
                    .transpose()?
                    .unwrap_or_default();
                for value in values {
                    let next = self.raw_set(value, flags)?;
                    result = match operation {
                        "union" => result.union(next),
                        "intersection" => result.intersection(next),
                        "subtraction" => result.subtract(next),
                        _ => return Err("Unknown Unicode set operation".into()),
                    };
                }
                if node[2].as_bool().unwrap() {
                    result.complement(maximum)?
                } else {
                    result
                }
            }
            _ => return Err("Unknown regex set materialization".into()),
        };
        Ok(result.clean())
    }
    pub fn instruction(&self, node: &Value, flags: &str) -> Result<Value, String> {
        let set = self.raw_set(node, flags)?;
        if set.ranges.len() + set.strings.len() > 65536 {
            return Err("Regex set materialization budget exceeded".into());
        }
        let mut strings = set.strings.into_iter().collect::<Vec<_>>();
        strings.sort_by(|a, b| b.len().cmp(&a.len()).then(a.cmp(b)));
        Ok(json!([
            "set",
            set.ranges,
            strings,
            flags.contains('i'),
            flags.contains('u') || flags.contains('v')
        ]))
    }
}
</ui:copy>

## Compile matching order rather than a bag of alternatives

Splits preserve ECMAScript leftmost priority. Repetitions retain bounded frames and clear descendant captures. Lookaround references a separate directional program, so lookbehind searches captures from the right while reported source spans remain increasing.

<ui:copy id="snippet-regex-v3-bytecode">
//! Ordered, data-only control flow. Set materialization remains engine-owned.
use serde_json::{Value, json};
use std::collections::BTreeSet;
pub struct CodeBuilder {
    pub code: Vec<Value>,
    pub subprograms: Vec<Value>,
    repeat: usize,
    nodes: usize,
}
impl CodeBuilder {
    pub fn new() -> Self {
        Self {
            code: vec![],
            subprograms: vec![],
            repeat: 0,
            nodes: 0,
        }
    }
    fn push(&mut self, instruction: Value) -> Result<usize, String> {
        self.nodes += 1;
        if self.nodes > 4096 {
            return Err("Regex instruction budget exceeded".into());
        }
        let at = self.code.len();
        self.code.push(instruction);
        Ok(at)
    }
    fn captures(node: &Value, result: &mut BTreeSet<u32>) {
        let Some(v) = node.as_array() else { return };
        match v[0].as_str() {
            Some("seq" | "alt") => {
                for child in v[1].as_array().unwrap() {
                    Self::captures(child, result)
                }
            }
            Some("capture") => {
                result.insert(v[1].as_u64().unwrap() as u32);
                Self::captures(&v[2], result)
            }
            Some("repeat" | "look" | "scope") => Self::captures(&v[1], result),
            _ => {}
        }
    }
    fn descendants(node: &Value) -> Vec<u32> {
        let mut values = BTreeSet::new();
        Self::captures(node, &mut values);
        values.into_iter().collect()
    }
    pub fn lower(
        &mut self,
        node: &Value,
        flags: &str,
        direction: i32,
        sets: &dyn Fn(&Value, &str) -> Result<Value, String>,
    ) -> Result<(), String> {
        let op = node[0].as_str().ok_or("Malformed regex IR")?;
        match op {
            "seq" => {
                let children = node[1].as_array().ok_or("Malformed regex sequence")?;
                if direction == 1 {
                    for child in children {
                        self.lower(child, flags, direction, sets)?
                    }
                } else {
                    for child in children.iter().rev() {
                        self.lower(child, flags, direction, sets)?
                    }
                }
            }
            "alt" => {
                let children = node[1].as_array().ok_or("Malformed regex alternatives")?;
                let mut jumps = vec![];
                for (index, child) in children.iter().enumerate() {
                    if index + 1 < children.len() {
                        let split = self.push(json!(["split", 0, 0]))?;
                        let first = self.code.len();
                        self.lower(child, flags, direction, sets)?;
                        jumps.push(self.push(json!(["jump", 0]))?);
                        self.code[split] = json!(["split", first, self.code.len()]);
                    } else {
                        self.lower(child, flags, direction, sets)?
                    }
                }
                let end = self.code.len();
                for jump in jumps {
                    self.code[jump] = json!(["jump", end]);
                }
            }
            "capture" => {
                self.push(json!(["mark", node[1]]))?;
                self.lower(&node[2], flags, direction, sets)?;
                self.push(json!(["capture", node[1]]))?;
            }
            "repeat" => {
                let id = self.repeat;
                self.repeat += 1;
                let start = self.push(json!(["repeat"]))?;
                let body = self.code.len();
                self.lower(&node[1], flags, direction, sets)?;
                self.push(json!(["repeat_end", id, start]))?;
                let end = self.code.len();
                self.code[start] = json!([
                    "repeat",
                    id,
                    node[2],
                    node[3],
                    node[4],
                    body,
                    end,
                    Self::descendants(&node[1])
                ]);
            }
            "scope" => {
                let mut active = flags.chars().collect::<BTreeSet<_>>();
                for flag in node[3]
                    .as_str()
                    .ok_or("Malformed regex flag scope")?
                    .chars()
                {
                    active.remove(&flag);
                }
                active.extend(
                    node[2]
                        .as_str()
                        .ok_or("Malformed regex flag scope")?
                        .chars(),
                );
                let changed = active.into_iter().collect::<String>();
                self.lower(&node[1], &changed, direction, sets)?;
            }
            "look" => {
                let before = std::mem::take(&mut self.code);
                let behind = node[3].as_bool().ok_or("Malformed regex look direction")?;
                let subdirection = if behind { -1 } else { 1 };
                self.lower(&node[1], flags, subdirection, sets)?;
                self.push(json!(["accept"]))?;
                let subcode = std::mem::replace(&mut self.code, before);
                let index = self.subprograms.len();
                self.subprograms
                    .push(json!({"direction":subdirection,"code":subcode}));
                self.push(json!(["look", index, node[2], Self::descendants(&node[1])]))?;
            }
            "ref" => {
                self.push(json!([
                    "ref",
                    node[1],
                    flags.contains('i'),
                    flags.contains('u') || flags.contains('v')
                ]))?;
            }
            "assert" => {
                self.push(json!([
                    "assert",
                    node[1],
                    flags.contains('m'),
                    flags.contains('i') && (flags.contains('u') || flags.contains('v'))
                ]))?;
            }
            "char" | "dot" | "escape" | "property" | "class_set" => {
                self.push(sets(node, flags)?)?;
            }
            _ => return Err("Unknown regex IR operation".into()),
        }
        Ok(())
    }
    pub fn finish(mut self) -> Result<(Vec<Value>, Vec<Value>), String> {
        self.push(json!(["accept"]))?;
        Ok((self.code, self.subprograms))
    }
}
</ui:copy>

## Parse the actual TextMate replacement language

Capture replacements, case operations and conditional bodies follow the existing VSCode snippet parser examples. Conditional bodies are literal text rather than recursively interpreted snippets. Malformed constructs retain the same literal fallback, and unknown case operators leave the capture unchanged.

<ui:copy id="snippet-regex-v3-format">
//! TextMate transform format parsing, including literal fallback behavior.
use serde_json::{Value, json};
struct Parser<'a> {
    text: &'a str,
    at: usize,
    groups: usize,
}
impl Parser<'_> {
    fn peek(&self) -> Option<char> {
        self.text[self.at..].chars().next()
    }
    fn take(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.at += ch.len_utf8();
        Some(ch)
    }
    fn accept(&mut self, ch: char) -> bool {
        if self.peek() == Some(ch) {
            self.take();
            true
        } else {
            false
        }
    }
    fn index(&mut self) -> Option<usize> {
        let start = self.at;
        let mut value = 0usize;
        while let Some(ch @ '0'..='9') = self.peek() {
            self.take();
            value = value
                .saturating_mul(10)
                .saturating_add(ch as usize - '0' as usize)
                .min(self.groups + 1);
        }
        if self.at == start { None } else { Some(value) }
    }
    fn until(&mut self, end: char) -> Option<String> {
        let mut result = String::new();
        while let Some(ch) = self.take() {
            if ch == end {
                return Some(result);
            }
            if ch == '\\' {
                let escaped = self.take()?;
                if !matches!(escaped, '$' | '}' | '\\') {
                    return None;
                }
                result.push(escaped)
            } else {
                result.push(ch)
            }
        }
        None
    }
    fn marker(&mut self) -> Option<Value> {
        let complex = self.accept('{');
        let index = self.index()?;
        if !complex || self.accept('}') {
            return Some(json!(["capture", index, ""]));
        }
        if !self.accept(':') {
            return None;
        }
        if self.accept('/') {
            let start = self.at;
            if !self
                .peek()
                .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_')
            {
                return None;
            }
            while self
                .peek()
                .is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_')
            {
                self.take();
            }
            let mode = self.text[start..self.at].to_owned();
            if !self.accept('}') {
                return None;
            }
            let mode = if [
                "upcase",
                "downcase",
                "capitalize",
                "camelcase",
                "pascalcase",
                "kebabcase",
                "snakecase",
            ]
            .contains(&mode.as_str())
            {
                mode.as_str()
            } else {
                ""
            };
            return Some(json!(["capture", index, mode]));
        }
        let (yes, no) = if self.accept('+') {
            let yes = self.until('}')?;
            if yes.is_empty() {
                return None;
            }
            (vec![json!(yes)], vec![])
        } else if self.accept('-') {
            let no = self.until('}')?;
            if no.is_empty() {
                return None;
            }
            (vec![json!(["capture", index, ""])], vec![json!(no)])
        } else if self.accept('?') {
            let yes = self.until(':')?;
            if yes.is_empty() {
                return None;
            }
            let no = self.until('}')?;
            if no.is_empty() {
                return None;
            }
            (vec![json!(yes)], vec![json!(no)])
        } else {
            let no = self.until('}')?;
            if no.is_empty() {
                return None;
            }
            (vec![json!(["capture", index, ""])], vec![json!(no)])
        };
        Some(json!(["if", index, yes, no]))
    }
}
pub fn compile_format(text: &str, groups: usize) -> Result<Value, String> {
    if text.len() > 8192 {
        return Err("Transform format syntax budget exceeded".into());
    }
    let mut parser = Parser {
        text,
        at: 0,
        groups,
    };
    let mut output: Vec<Value> = vec![];
    let mut literal = String::new();
    while let Some(ch) = parser.take() {
        if ch == '$' {
            let after = parser.at;
            if let Some(value) = parser.marker() {
                if !literal.is_empty() {
                    output.push(json!(std::mem::take(&mut literal)))
                }
                output.push(value);
                if output.len() > 512 {
                    return Err("Transform format node budget exceeded".into());
                }
                continue;
            }
            parser.at = after;
            literal.push('$');
        } else if ch == '\\' && matches!(parser.peek(), Some('\\' | '/')) {
            literal.push(parser.take().unwrap());
        } else {
            literal.push(ch)
        }
    }
    if !literal.is_empty() {
        output.push(json!(literal))
    }
    Ok(json!(output))
}
</ui:copy>

## Admit a bounded, exact wire schema

Every opcode, arity, program counter, capture number, flag and case-policy identity is checked before evaluation. Mathematical integers are normalized consistently across JSON implementations. Instruction, range, string, format-depth and metadata budgets prevent a hostile provider from expanding a small request into unbounded host state. Unknown effects and operations refuse. Check array counts before copying or visiting rows. Count canonical JSON bytes only after admission with a bounded streaming meter; never serialize a hostile raw value just to discover that it is too large. This also gives integral JSON numbers such as `3.0` the same normalized size across hosts. The focused witnesses poison excessive arrays and whole-value encoding, and exercise near-limit numeric metadata.

<ui:copy id="snippet-regex-v3-reader">
//! Strict typed v3 bytecode admission. No unknown operation can execute.
use serde_json::Value;
#[derive(Clone)]
pub enum Op {
    Accept,
    Jump(usize),
    Split(usize, usize),
    Mark(usize),
    Capture(usize),
    Assert(String, bool, bool),
    Reference(Vec<usize>, bool, bool),
    Set(Vec<(u32, u32)>, Vec<Vec<u32>>, bool, bool),
    Look(usize, bool, Vec<usize>),
    Repeat {
        id: usize,
        min: u64,
        max: Option<u64>,
        lazy: bool,
        body: usize,
        end: usize,
        clear: Vec<usize>,
    },
    RepeatEnd(usize, usize),
}
pub struct Code {
    pub direction: i32,
    pub ops: Vec<Op>,
}
pub struct Program {
    pub codes: Vec<Code>,
    pub groups: usize,
    pub flags: String,
    pub format: Value,
}
fn integer(raw: &Value, maximum: u64) -> Result<u64, String> {
    let value = raw
        .as_f64()
        .filter(|n| n.is_finite() && *n >= 0.0 && n.fract() == 0.0 && *n <= maximum as f64)
        .ok_or("Malformed regex integer")?;
    Ok(value as u64)
}
fn boolean(raw: &Value) -> Result<bool, String> {
    raw.as_bool()
        .ok_or_else(|| "Malformed regex boolean".into())
}
fn array(raw: &Value) -> Result<&Vec<Value>, String> {
    raw.as_array().ok_or_else(|| "Malformed regex array".into())
}
struct Meter {
    nodes: usize,
    items: usize,
}
impl Meter {
    fn tick(&mut self, n: usize) -> Result<(), String> {
        self.nodes = self.nodes.checked_add(n).ok_or("Regex metadata overflow")?;
        if self.nodes > 262144 {
            return Err("Regex metadata budget exceeded".into());
        }
        Ok(())
    }
    fn list(&mut self, raw: &Value, max: usize) -> Result<(), String> {
        if array(raw)?.len() > max {
            return Err("Regex array budget exceeded".into());
        }
        self.tick(array(raw)?.len())
    }
}
fn indices(raw: &Value, groups: usize) -> Result<Vec<usize>, String> {
    let rows = array(raw)?;
    if rows.len() > groups {
        return Err("Regex capture-list budget exceeded".into());
    }
    let mut result = vec![];
    for item in rows {
        let index = integer(item, groups as u64)? as usize;
        if index == 0 || result.contains(&index) {
            return Err("Malformed regex capture list".into());
        }
        result.push(index)
    }
    Ok(result)
}
fn formats(raw: &Value, groups: usize, depth: usize, meter: &mut Meter) -> Result<(), String> {
    if depth > 32 {
        return Err("Regex format depth exceeded".into());
    }
    meter.list(raw, 512)?;
    for item in array(raw)? {
        if let Some(text) = item.as_str() {
            if text.len() > 16384 {
                return Err("Regex format text budget exceeded".into());
            }
            meter.tick(text.len())?;
            continue;
        }
        let value = array(item)?;
        match value.first().and_then(Value::as_str) {
            Some("capture") if value.len() == 3 => {
                integer(&value[1], (groups + 1) as u64)?;
                if ![
                    "",
                    "upcase",
                    "downcase",
                    "capitalize",
                    "camelcase",
                    "pascalcase",
                    "kebabcase",
                    "snakecase",
                ]
                .contains(&value[2].as_str().ok_or("Malformed regex format mode")?)
                {
                    return Err("Unknown regex format mode".into());
                }
            }
            Some("if") if value.len() == 4 => {
                integer(&value[1], (groups + 1) as u64)?;
                formats(&value[2], groups, depth + 1, meter)?;
                formats(&value[3], groups, depth + 1, meter)?
            }
            _ => return Err("Unknown regex format operation".into()),
        }
    }
    Ok(())
}
fn code(
    raw: &Value,
    direction: i32,
    groups: usize,
    subs: usize,
    unicode: bool,
    meter: &mut Meter,
) -> Result<Code, String> {
    meter.list(raw, 4096)?;
    let rows = array(raw)?;
    meter.items = meter
        .items
        .checked_add(rows.len())
        .ok_or("Regex instruction budget overflow")?;
    if meter.items > 4096 {
        return Err("Regex total instruction budget exceeded".into());
    }
    if rows.is_empty() {
        return Err("Empty regex bytecode".into());
    }
    let pc = |raw| integer(raw, (rows.len() - 1) as u64).map(|n| n as usize);
    let group = |raw: &Value| -> Result<usize, String> {
        let value = integer(raw, groups as u64)? as usize;
        if value == 0 {
            Err("Regex group zero cannot be written".into())
        } else {
            Ok(value)
        }
    };
    let mut ops = vec![];
    for row in rows {
        let values = array(row)?;
        meter.tick(values.len())?;
        let op = values
            .first()
            .and_then(Value::as_str)
            .ok_or("Missing regex instruction")?;
        ops.push(match op {
            "accept" if values.len() == 1 => Op::Accept,
            "jump" if values.len() == 2 => Op::Jump(pc(&values[1])?),
            "split" if values.len() == 3 => Op::Split(pc(&values[1])?, pc(&values[2])?),
            "mark" if values.len() == 2 => Op::Mark(group(&values[1])?),
            "capture" if values.len() == 2 => Op::Capture(group(&values[1])?),
            "assert" if values.len() == 4 => {
                let kind = values[1].as_str().ok_or("Malformed regex assertion")?;
                if !["start", "end", "word", "nonword"].contains(&kind) {
                    return Err("Unknown regex assertion".into());
                }
                let insensitive = boolean(&values[3])?;
                if insensitive && !unicode {
                    return Err("Regex assertion mode mismatch".into());
                }
                Op::Assert(kind.to_owned(), boolean(&values[2])?, insensitive)
            }
            "ref" if values.len() == 4 => {
                if boolean(&values[3])? != unicode {
                    return Err("Regex reference mode mismatch".into());
                }
                Op::Reference(indices(&values[1], groups)?, boolean(&values[2])?, unicode)
            }
            "look" if values.len() == 4 => {
                if subs == 0 {
                    return Err("Regex assertion program missing".into());
                }
                Op::Look(
                    integer(&values[1], (subs - 1) as u64)? as usize + 1,
                    boolean(&values[2])?,
                    indices(&values[3], groups)?,
                )
            }
            "repeat" if values.len() == 8 => {
                let id = integer(&values[1], 4095)? as usize;
                let min = integer(&values[2], 9_007_199_254_740_991)?;
                let max = if values[3].is_null() {
                    None
                } else {
                    Some(integer(&values[3], 9_007_199_254_740_991)?)
                };
                if max.is_some_and(|n| n < min) {
                    return Err("Backwards regex repetition".into());
                }
                Op::Repeat {
                    id,
                    min,
                    max,
                    lazy: boolean(&values[4])?,
                    body: pc(&values[5])?,
                    end: pc(&values[6])?,
                    clear: indices(&values[7], groups)?,
                }
            }
            "repeat_end" if values.len() == 3 => {
                Op::RepeatEnd(integer(&values[1], 4095)? as usize, pc(&values[2])?)
            }
            "set" if values.len() == 5 => {
                if boolean(&values[4])? != unicode {
                    return Err("Regex character mode mismatch".into());
                }
                meter.list(&values[1], 65536)?;
                meter.list(&values[2], 65536)?;
                let maximum = if unicode { 0x10ffff } else { 65535 };
                let mut ranges: Vec<(u32, u32)> = vec![];
                for raw in array(&values[1])? {
                    let pair = array(raw)?;
                    if pair.len() != 2 {
                        return Err("Malformed regex range".into());
                    }
                    let low = integer(&pair[0], maximum)? as u32;
                    let high = integer(&pair[1], maximum)? as u32;
                    if low > high || ranges.last().is_some_and(|r| r.1 + 1 >= low) {
                        return Err("Unnormalized regex ranges".into());
                    }
                    ranges.push((low, high));
                }
                let mut strings: Vec<Vec<u32>> = vec![];
                for raw in array(&values[2])? {
                    meter.list(raw, 8192)?;
                    let value = array(raw)?
                        .iter()
                        .map(|v| integer(v, maximum).map(|cp| cp as u32))
                        .collect::<Result<Vec<_>, _>>()?;
                    if value.len() == 1
                        || strings.last().is_some_and(|prior| {
                            prior.len() < value.len()
                                || prior.len() == value.len() && prior >= &value
                        })
                    {
                        return Err("Unnormalized regex strings".into());
                    }
                    strings.push(value);
                }
                Op::Set(ranges, strings, boolean(&values[3])?, unicode)
            }
            _ => return Err("Unknown or malformed regex instruction".into()),
        });
    }
    for op in &ops {
        if let Op::RepeatEnd(id, start) = op {
            if !matches!(&ops[*start],Op::Repeat{id:other,..}if id==other) {
                return Err("Regex repeat edge mismatch".into());
            }
        }
    }
    Ok(Code { direction, ops })
}
fn normalized_format(raw: &Value) -> Value {
    serde_json::json!(
        raw.as_array()
            .unwrap()
            .iter()
            .map(|op| {
                if op.is_string() {
                    return op.clone();
                }
                let index = op[1].as_f64().unwrap() as u64;
                if op[0] == "capture" {
                    serde_json::json!(["capture", index, op[2]])
                } else {
                    serde_json::json!([
                        "if",
                        index,
                        normalized_format(&op[2]),
                        normalized_format(&op[3])
                    ])
                }
            })
            .collect::<Vec<_>>()
    )
}
pub fn read(raw: &Value) -> Result<Program, String> {
    let object = raw.as_object().ok_or("Malformed regex program")?;
    let required = [
        "version",
        "case_map",
        "unicode_map",
        "groups",
        "flags",
        "code",
        "subprograms",
        "format",
    ];
    if object.len() != required.len() || required.iter().any(|key| !object.contains_key(*key)) {
        return Err("Unknown or missing regex program fields".into());
    }
    if integer(&raw["version"], 3)? != 3
        || raw["case_map"] != "hickory-ucd-17.0.0-v1"
        || raw["unicode_map"] != "hickory-regex-ucd-17.0.0-v1"
    {
        return Err("Unsupported regex program/data version".into());
    }
    let groups = integer(&raw["groups"], 32)? as usize;
    let flags = raw["flags"]
        .as_str()
        .filter(|s| s.len() <= 8)
        .ok_or("Malformed regex flags")?
        .to_owned();
    let mut seen = vec![];
    for flag in flags.chars() {
        if !"dgimsuvy".contains(flag) || seen.contains(&flag) {
            return Err("Unknown or duplicate regex flags".into());
        }
        seen.push(flag)
    }
    if flags.contains('u') && flags.contains('v') {
        return Err("Incompatible regex Unicode flags".into());
    }
    let unicode = flags.contains('u') || flags.contains('v');
    let subs = array(&raw["subprograms"])?;
    if subs.len() > 128 {
        return Err("Regex assertion count budget exceeded".into());
    }
    let mut meter = Meter { nodes: 0, items: 0 };
    formats(&raw["format"], groups, 0, &mut meter)?;
    let mut codes = vec![code(
        &raw["code"],
        1,
        groups,
        subs.len(),
        unicode,
        &mut meter,
    )?];
    for sub in subs {
        let object = sub.as_object().ok_or("Malformed assertion program")?;
        if object.len() != 2 || !object.contains_key("direction") || !object.contains_key("code") {
            return Err("Malformed assertion fields".into());
        }
        let direction = sub["direction"]
            .as_f64()
            .filter(|n| *n == 1.0 || *n == -1.0)
            .ok_or("Malformed assertion direction")? as i32;
        codes.push(code(
            &sub["code"],
            direction,
            groups,
            subs.len(),
            unicode,
            &mut meter,
        )?);
    }
    metadata_bytes(raw)?;
    Ok(Program {
        codes,
        groups,
        flags,
        format: normalized_format(&raw["format"]),
    })
}

/// Count canonical JSON bytes after schema validation, without allocating a second wire value.
/// Integers have the same representation across all hosts, including input written as `3.0`.
fn metadata_bytes(raw: &Value) -> Result<usize, String> {
    fn add(total: &mut usize, n: usize) -> Result<(), String> {
        *total = total
            .checked_add(n)
            .filter(|n| *n <= 1_048_576)
            .ok_or("Regex metadata byte budget exceeded")?;
        Ok(())
    }
    fn string(text: &str, total: &mut usize) -> Result<(), String> {
        add(total, 2)?;
        for ch in text.chars() {
            add(
                total,
                match ch {
                    '"' | '\\' | '\u{8}' | '\t' | '\n' | '\u{c}' | '\r' => 2,
                    ch if (ch as u32) < 32 => 6,
                    ch => ch.len_utf8(),
                },
            )?
        }
        Ok(())
    }
    fn visit(raw: &Value, total: &mut usize, depth: usize) -> Result<(), String> {
        if depth > 128 {
            return Err("Regex metadata nesting exceeded".into());
        };
        match raw {
            Value::Null => add(total, 4),
            Value::Bool(value) => add(total, if *value { 4 } else { 5 }),
            Value::Number(number) => {
                let n = number.as_f64().ok_or("Malformed regex number")?;
                if !n.is_finite() || n.fract() != 0.0 || n.abs() > 9_007_199_254_740_991.0 {
                    return Err("Malformed regex integer".into());
                };
                add(total, (n as i64).to_string().len())
            }
            Value::String(text) => string(text, total),
            Value::Array(values) => {
                add(total, 2 + values.len().saturating_sub(1))?;
                for value in values {
                    visit(value, total, depth + 1)?
                }
                Ok(())
            }
            Value::Object(values) => {
                add(total, 2 + values.len().saturating_sub(1))?;
                for (key, value) in values {
                    string(key, total)?;
                    add(total, 1)?;
                    visit(value, total, depth + 1)?
                }
                Ok(())
            }
        }
    }
    let mut total = 0;
    visit(raw, &mut total, 0)?;
    Ok(total)
}
#[cfg(test)]
mod admission_witnesses {
    use super::*;
    #[test]
    fn canonical_integer_meter_is_bounded_without_serializing_raw_json() {
        let ranges = (0..55_000)
            .map(|i| serde_json::json!([(i * 16) as f64, (i * 16) as f64]))
            .collect::<Vec<_>>();
        let raw = serde_json::json!({"version":3.0,"case_map":"hickory-ucd-17.0.0-v1","unicode_map":"hickory-regex-ucd-17.0.0-v1","groups":0.0,"flags":"u","code":[["set",ranges,[],false,true],["accept"]],"subprograms":[],"format":[["capture",0.0,""]]});
        assert!(serde_json::to_vec(&raw).unwrap().len() > 1_048_576);
        assert!(metadata_bytes(&raw).unwrap() < 1_048_576);
        assert!(read(&raw).is_ok());
        let mut too_large = raw.clone();
        too_large["code"] = serde_json::json!([raw["code"][0], raw["code"][0], ["accept"]]);
        assert!(read(&too_large).is_err());
        let mut too_many = raw;
        too_many["code"] = serde_json::json!(vec![serde_json::json!(["accept"]); 4097]);
        assert!(read(&too_many).is_err());
    }
}
</ui:copy>

## Execute with explicit work and retained-state limits

The VM counts state copying, captures, comparisons and control flow against the shared action budget. A retained-backtrack cap bounds memory independently of work. Assertion nesting has its own cap. Non-Unicode patterns use UTF-16 units; Unicode modes use code points. Matching never executes provider commands or native regex text.

<ui:copy id="snippet-regex-v3-matcher">
//! Bounded ordered backtracking with atomic assertion subprograms.
use super::{
    reader::{Op, Program},
    sets::UnicodeData,
};
use std::collections::BTreeMap;
#[derive(Clone)]
struct Repeat {
    count: u64,
    at: usize,
}
#[derive(Clone)]
struct State {
    pc: usize,
    at: usize,
    captures: Vec<Option<(usize, usize)>>,
    marks: Vec<Option<usize>>,
    repeats: BTreeMap<usize, Repeat>,
}
pub struct Found {
    pub from: usize,
    pub to: usize,
    pub captures: Vec<Option<(usize, usize)>>,
}
pub struct Machine<'a> {
    pub text: Vec<u32>,
    pub offsets: Vec<usize>,
    pub unicode: bool,
    policy: &'a UnicodeData,
    work: &'a mut usize,
    retained: usize,
}
impl<'a> Machine<'a> {
    pub fn new(
        input: &str,
        unicode: bool,
        policy: &'a UnicodeData,
        work: &'a mut usize,
    ) -> Result<Self, String> {
        if input.len() > 65536 {
            return Err("Regex input byte budget exceeded".into());
        }
        let text = if unicode {
            input.chars().map(|ch| ch as u32).collect::<Vec<_>>()
        } else {
            input.encode_utf16().map(|unit| unit as u32).collect()
        };
        let mut offsets = vec![0];
        for point in &text {
            offsets.push(offsets.last().unwrap() + if unicode && *point > 65535 { 2 } else { 1 });
        }
        Ok(Self {
            text,
            offsets,
            unicode,
            policy,
            work,
            retained: 0,
        })
    }
    fn tick(&mut self, n: usize) -> Result<(), String> {
        if *self.work > 100000 || n > 100000 - *self.work {
            return Err("Regex work budget exceeded".into());
        }
        *self.work += n;
        Ok(())
    }
    fn copy(&mut self, state: &State) -> Result<State, String> {
        self.tick(state.captures.len() + state.marks.len() + state.repeats.len())?;
        Ok(state.clone())
    }
    fn push(&mut self, stack: &mut Vec<State>, state: State) -> Result<(), String> {
        if self.retained >= 1024 {
            return Err("Regex backtrack memory budget exceeded".into());
        }
        self.retained += 1;
        stack.push(state);
        Ok(())
    }
    fn clear(&mut self, state: &mut State, indices: &[usize]) -> Result<(), String> {
        for index in indices {
            self.tick(1)?;
            state.captures[*index] = None;
            state.marks[*index] = None;
        }
        Ok(())
    }
    fn advance(&self, at: usize, count: usize, direction: i32) -> Option<usize> {
        if direction == 1 {
            at.checked_add(count).filter(|at| *at <= self.text.len())
        } else {
            at.checked_sub(count)
        }
    }
    fn member(&mut self, ranges: &[(u32, u32)], point: u32) -> Result<bool, String> {
        let (mut low, mut high) = (0, ranges.len());
        while low < high {
            self.tick(1)?;
            let mid = (low + high) / 2;
            if ranges[mid].0 <= point {
                low = mid + 1
            } else {
                high = mid
            }
        }
        Ok(low > 0 && point <= ranges[low - 1].1)
    }
    fn word(&self, point: Option<u32>, insensitive: bool) -> bool {
        let Some(mut point) = point else { return false };
        if insensitive {
            point = self.policy.canonical(point, true)
        }
        (65..=90).contains(&point)
            || (97..=122).contains(&point)
            || (48..=57).contains(&point)
            || point == 95
    }
    fn newline(point: Option<u32>) -> bool {
        point.is_some_and(|point| [10, 13, 0x2028, 0x2029].contains(&point))
    }
    fn run(
        &mut self,
        program: &Program,
        which: usize,
        start: usize,
        captures: &[Option<(usize, usize)>],
        marks: &[Option<usize>],
        depth: usize,
    ) -> Result<Option<State>, String> {
        if depth > 32 {
            return Err("Regex assertion nesting budget exceeded".into());
        }
        self.tick(captures.len() + marks.len())?;
        let code = &program.codes[which];
        let direction = code.direction;
        let mut state = State {
            pc: 0,
            at: start,
            captures: captures.to_vec(),
            marks: marks.to_vec(),
            repeats: BTreeMap::new(),
        };
        let mut stack = vec![];
        let result = (|| {
            loop {
                self.tick(1)?;
                let op = code
                    .ops
                    .get(state.pc)
                    .ok_or("Regex instruction outside program")?;
                let mut failed = false;
                match op {
                    Op::Accept => return Ok(Some(state)),
                    Op::Jump(pc) => {
                        state.pc = *pc;
                        continue;
                    }
                    Op::Split(first, second) => {
                        let mut alternative = self.copy(&state)?;
                        alternative.pc = *second;
                        self.push(&mut stack, alternative)?;
                        state.pc = *first;
                        continue;
                    }
                    Op::Mark(index) => {
                        state = self.copy(&state)?;
                        state.marks[*index] = Some(state.at);
                        state.captures[*index] = None;
                    }
                    Op::Capture(index) => {
                        let start = state.marks[*index].ok_or("Regex capture has no start")?;
                        state = self.copy(&state)?;
                        state.captures[*index] = Some((start.min(state.at), start.max(state.at)));
                    }
                    Op::Assert(kind, multiline, insensitive) => {
                        let before = state
                            .at
                            .checked_sub(1)
                            .and_then(|at| self.text.get(at))
                            .copied();
                        let next = self.text.get(state.at).copied();
                        let boundary =
                            self.word(before, *insensitive) != self.word(next, *insensitive);
                        let yes = match kind.as_str() {
                            "start" => state.at == 0 || *multiline && Self::newline(before),
                            "end" => {
                                state.at == self.text.len() || *multiline && Self::newline(next)
                            }
                            "word" => boundary,
                            _ => !boundary,
                        };
                        failed = !yes;
                    }
                    Op::Reference(indices, insensitive, unicode) => {
                        let captured = indices
                            .iter()
                            .filter_map(|index| state.captures[*index])
                            .collect::<Vec<_>>();
                        if captured.len() > 1 {
                            return Err("Ambiguous named regex reference".into());
                        }
                        if let Some((from, to)) = captured.first().copied() {
                            if let Some(end) = self.advance(state.at, to - from, direction) {
                                let begin = state.at.min(end);
                                for offset in 0..to - from {
                                    self.tick(1)?;
                                    let a = self.text[from + offset];
                                    let b = self.text[begin + offset];
                                    if if *insensitive {
                                        self.policy.canonical(a, *unicode)
                                            != self.policy.canonical(b, *unicode)
                                    } else {
                                        a != b
                                    } {
                                        failed = true;
                                        break;
                                    }
                                }
                                if !failed {
                                    state.at = end
                                }
                            } else {
                                failed = true
                            }
                        }
                    }
                    Op::Set(ranges, strings, insensitive, unicode) => {
                        let mut candidates = vec![];
                        for value in strings {
                            self.tick(1)?;
                            let Some(end) = self.advance(state.at, value.len(), direction) else {
                                continue;
                            };
                            let begin = state.at.min(end);
                            let mut yes = true;
                            for (index, point) in value.iter().enumerate() {
                                self.tick(1)?;
                                let actual = self.text[begin + index];
                                if if *insensitive {
                                    self.policy.canonical(actual, *unicode) != *point
                                } else {
                                    actual != *point
                                } {
                                    yes = false;
                                    break;
                                }
                            }
                            if yes {
                                candidates.push(value.len())
                            }
                        }
                        let at = if direction == 1 {
                            Some(state.at)
                        } else {
                            state.at.checked_sub(1)
                        };
                        if let Some(point) = at.and_then(|at| self.text.get(at)).copied() {
                            if self.member(ranges, point)? {
                                candidates.push(1)
                            }
                        }
                        candidates.sort_unstable_by(|a, b| b.cmp(a));
                        if candidates.is_empty() {
                            failed = true
                        } else {
                            for count in candidates.iter().skip(1).rev() {
                                let mut alternative = self.copy(&state)?;
                                alternative.at = self.advance(state.at, *count, direction).unwrap();
                                alternative.pc += 1;
                                self.push(&mut stack, alternative)?;
                            }
                            state.at = self.advance(state.at, candidates[0], direction).unwrap();
                        }
                    }
                    Op::Look(sub, positive, indices) => {
                        state = self.copy(&state)?;
                        self.clear(&mut state, indices)?;
                        let found = self.run(
                            program,
                            *sub,
                            state.at,
                            &state.captures,
                            &state.marks,
                            depth + 1,
                        )?;
                        if *positive {
                            if let Some(found) = found {
                                state.captures = found.captures;
                                state.marks = found.marks
                            } else {
                                failed = true
                            }
                        } else if found.is_some() {
                            failed = true
                        }
                    }
                    Op::Repeat {
                        id,
                        min,
                        max,
                        lazy,
                        body,
                        end,
                        clear,
                    } => {
                        let frame = state.repeats.get(id).cloned().unwrap_or(Repeat {
                            count: 0,
                            at: state.at,
                        });
                        let can_exit = frame.count >= *min;
                        let can_repeat = max.is_none_or(|max| frame.count < max);
                        let exit = if can_exit {
                            let mut value = self.copy(&state)?;
                            value.repeats.remove(id);
                            value.pc = *end;
                            Some(value)
                        } else {
                            None
                        };
                        let more = if can_repeat {
                            let mut value = self.copy(&state)?;
                            value.repeats.insert(
                                *id,
                                Repeat {
                                    count: frame.count,
                                    at: state.at,
                                },
                            );
                            self.clear(&mut value, clear)?;
                            value.pc = *body;
                            Some(value)
                        } else {
                            None
                        };
                        match (exit, more) {
                            (Some(exit), Some(more)) => {
                                if *lazy {
                                    self.push(&mut stack, more)?;
                                    state = exit
                                } else {
                                    self.push(&mut stack, exit)?;
                                    state = more
                                }
                            }
                            (Some(exit), None) => state = exit,
                            (None, Some(more)) => state = more,
                            (None, None) => failed = true,
                        };
                        if !failed {
                            continue;
                        }
                    }
                    Op::RepeatEnd(id, start) => {
                        let entry = state
                            .repeats
                            .get(id)
                            .cloned()
                            .ok_or("Regex repeat frame missing")?;
                        let Op::Repeat { min, .. } = &code.ops[*start] else {
                            return Err("Regex repeat edge mismatch".into());
                        };
                        if entry.at == state.at && entry.count >= *min {
                            failed = true
                        } else {
                            state = self.copy(&state)?;
                            state.repeats.insert(
                                *id,
                                Repeat {
                                    count: entry.count + 1,
                                    at: entry.at,
                                },
                            );
                            state.pc = *start;
                            continue;
                        }
                    }
                }
                if failed {
                    let Some(previous) = stack.pop() else {
                        return Ok(None);
                    };
                    self.retained -= 1;
                    state = previous
                } else {
                    state.pc += 1
                }
            }
        })();
        self.retained -= stack.len();
        result
    }
    pub fn find(&mut self, program: &Program) -> Result<Vec<Found>, String> {
        let mut output = vec![];
        let mut search = 0;
        while search <= self.text.len() {
            let mut found = None;
            for start in search..=self.text.len() {
                if let Some(mut result) = self.run(
                    program,
                    0,
                    start,
                    &vec![None; program.groups + 1],
                    &vec![None; program.groups + 1],
                    0,
                )? {
                    result.captures[0] = Some((start, result.at));
                    found = Some(Found {
                        from: start,
                        to: result.at,
                        captures: result.captures,
                    });
                    break;
                }
                if program.flags.contains('y') {
                    break;
                }
            }
            let Some(value) = found else { break };
            search = if value.from == value.to {
                value.to + 1
            } else {
                value.to
            };
            output.push(value);
            if !program.flags.contains('g') {
                break;
            }
        }
        Ok(output)
    }
    pub fn units(&self, from: usize, to: usize) -> Vec<u16> {
        let mut result = vec![];
        for point in &self.text[from..to] {
            if self.unicode && *point > 65535 {
                let cp = *point - 65536;
                result.push(0xd800 + (cp >> 10) as u16);
                result.push(0xdc00 + (cp & 1023) as u16)
            } else {
                result.push(*point as u16)
            }
        }
        result
    }
}
</ui:copy>

## Refuse the entire output when Unicode or size is unsafe

Captures remain UTF-16 units until the complete result is written. This permits a valid surrogate pair to rejoin across chunks while refusing a final unpaired surrogate. The writer bounds UTF-8 output bytes. Casing uses frozen Unicode 17 data, including final sigma and the documented UTF-16 first-unit behavior of TextMate case modes.

<ui:copy id="snippet-regex-v3-formatter">
//! Frozen Unicode17 replacement modes; captures remain UTF16 until final output.
use serde_json::Value;
use std::collections::BTreeMap;
type Text = Vec<u16>;
pub struct CaseData {
    upper: BTreeMap<u32, Text>,
    lower: BTreeMap<u32, Text>,
    ranges: BTreeMap<&'static str, Vec<(u32, u32)>>,
}
fn tick(work: &mut usize, n: usize) -> Result<(), String> {
    if *work > 100000 || n > 100000 - *work {
        return Err("Regex format work budget exceeded".into());
    }
    *work += n;
    Ok(())
}
fn points(text: &[u16]) -> Vec<u32> {
    let mut output = vec![];
    let mut at = 0;
    while at < text.len() {
        let unit = text[at] as u32;
        at += 1;
        if (0xd800..=0xdbff).contains(&unit)
            && at < text.len()
            && (0xdc00..=0xdfff).contains(&text[at])
        {
            output.push(0x10000 + ((unit - 0xd800) << 10) + (text[at] as u32 - 0xdc00));
            at += 1
        } else {
            output.push(unit)
        }
    }
    output
}
fn encoded(points: &[u32]) -> Text {
    let mut output = vec![];
    for point in points {
        if *point > 65535 {
            let cp = *point - 65536;
            output.push(0xd800 + (cp >> 10) as u16);
            output.push(0xdc00 + (cp & 1023) as u16)
        } else {
            output.push(*point as u16)
        }
    }
    output
}
impl CaseData {
    pub fn new(data: &Value) -> Self {
        let map = |name: &str| {
            data[name]
                .as_array()
                .unwrap()
                .iter()
                .map(|row| {
                    (
                        row[0].as_u64().unwrap() as u32,
                        row[1].as_str().unwrap().encode_utf16().collect(),
                    )
                })
                .collect()
        };
        let mut ranges = BTreeMap::new();
        for kind in [
            "letter",
            "cased",
            "ignorable",
            "space",
            "upper_letter",
            "lower_letter",
        ] {
            ranges.insert(
                kind,
                data[kind]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|row| {
                        (
                            row[0].as_u64().unwrap() as u32,
                            row[1].as_u64().unwrap() as u32,
                        )
                    })
                    .collect(),
            );
        }
        Self {
            upper: map("upper"),
            lower: map("lower"),
            ranges,
        }
    }
    fn has(&self, kind: &str, point: Option<u32>, work: &mut usize) -> Result<bool, String> {
        let Some(point) = point else { return Ok(false) };
        let ranges = &self.ranges[kind];
        let (mut low, mut high) = (0, ranges.len());
        while low < high {
            tick(work, 1)?;
            let mid = (low + high) / 2;
            if ranges[mid].0 <= point {
                low = mid + 1
            } else {
                high = mid
            }
        }
        Ok(low > 0 && point <= ranges[low - 1].1)
    }
    fn convert(&self, value: &[u16], uppercase: bool, work: &mut usize) -> Result<Text, String> {
        let chars = points(value);
        let mapping = if uppercase { &self.upper } else { &self.lower };
        let mut result = vec![];
        for (index, point) in chars.iter().enumerate() {
            tick(work, 1)?;
            if !uppercase && *point == 0x3a3 {
                let mut before = index.checked_sub(1);
                let mut after = index + 1;
                while let Some(at) = before {
                    if !self.has("ignorable", Some(chars[at]), work)? {
                        break;
                    }
                    tick(work, 1)?;
                    before = at.checked_sub(1);
                }
                while after < chars.len() && self.has("ignorable", Some(chars[after]), work)? {
                    tick(work, 1)?;
                    after += 1;
                }
                if let Some(before) = before {
                    if self.has("cased", Some(chars[before]), work)?
                        && (after == chars.len() || !self.has("cased", Some(chars[after]), work)?)
                    {
                        result.push(0x3c2);
                        continue;
                    }
                }
            }
            if let Some(text) = mapping.get(point) {
                result.extend(text)
            } else {
                result.extend(encoded(&[*point]))
            }
        }
        Ok(result)
    }
    fn digit(point: Option<u32>) -> bool {
        point.is_some_and(|cp| (48..=57).contains(&cp))
    }
    fn white(&self, point: Option<u32>, work: &mut usize) -> Result<bool, String> {
        Ok(
            point.is_some_and(|cp| [9, 10, 11, 12, 13, 0xfeff, 0x2028, 0x2029].contains(&cp))
                || self.has("space", point, work)?,
        )
    }
    fn delimiter(&self, point: Option<u32>, work: &mut usize) -> Result<bool, String> {
        Ok(matches!(point, Some(45 | 95)) || self.white(point, work)?)
    }
    fn words(&self, value: &[u16], work: &mut usize) -> Result<Vec<Text>, String> {
        let mut output = vec![];
        let mut current = vec![];
        for point in points(value) {
            tick(work, 1)?;
            if self.has("letter", Some(point), work)? || Self::digit(Some(point)) {
                current.extend(encoded(&[point]))
            } else if !current.is_empty() {
                output.push(std::mem::take(&mut current));
            }
        }
        if !current.is_empty() {
            output.push(current)
        }
        Ok(output)
    }
    fn joined(parts: &[Text], separator: u16) -> Text {
        let mut result = vec![];
        for (index, part) in parts.iter().enumerate() {
            if index > 0 {
                result.push(separator)
            }
            result.extend(part)
        }
        result
    }
    fn kebab(&self, value: &[u16], work: &mut usize) -> Result<Text, String> {
        let chars = points(value);
        let mut any = false;
        for point in &chars {
            tick(work, 1)?;
            if self.has("letter", Some(*point), work)? || Self::digit(Some(*point)) {
                any = true;
                break;
            }
        }
        if !any {
            return Ok(value.to_vec());
        }
        let (mut first, mut last) = (0, chars.len());
        while first < last && self.white(Some(chars[first]), work)? {
            first += 1
        }
        while last > first && self.white(Some(chars[last - 1]), work)? {
            last -= 1
        }
        while first < last && chars[first] == 95 {
            first += 1
        }
        while last > first && chars[last - 1] == 95 {
            last -= 1
        }
        let cleaned = &chars[first..last];
        let mut parts = vec![];
        let mut at = 0;
        while at < cleaned.len() {
            tick(work, 1)?;
            let mut end = at;
            let mut run = at;
            while run < cleaned.len() && self.has("upper_letter", Some(cleaned[run]), work)? {
                tick(work, 1)?;
                run += 1
            }
            for stop in (at + 2..=run).rev() {
                tick(work, 1)?;
                let point = cleaned.get(stop).copied();
                if stop == cleaned.len()
                    || self.delimiter(point, work)?
                    || self.has("upper_letter", point, work)?
                        && self.has("lower_letter", cleaned.get(stop + 1).copied(), work)?
                {
                    end = stop;
                    break;
                }
            }
            if end == at {
                let mut stop = at;
                if self.has("upper_letter", cleaned.get(stop).copied(), work)? {
                    stop += 1
                }
                let begin = stop;
                while stop < cleaned.len() && self.has("lower_letter", Some(cleaned[stop]), work)? {
                    tick(work, 1)?;
                    stop += 1
                }
                if stop > begin {
                    while Self::digit(cleaned.get(stop).copied()) {
                        tick(work, 1)?;
                        stop += 1
                    }
                    end = stop
                }
            }
            if end == at
                && self.has("upper_letter", Some(cleaned[at]), work)?
                && self.has("upper_letter", cleaned.get(at + 1).copied(), work)?
                && self.has("lower_letter", cleaned.get(at + 2).copied(), work)?
            {
                end = at + 1
            }
            if end == at
                && self.has("upper_letter", Some(cleaned[at]), work)?
                && (at + 1 == cleaned.len()
                    || self.delimiter(cleaned.get(at + 1).copied(), work)?)
            {
                end = at + 1
            }
            if end == at && Self::digit(Some(cleaned[at])) {
                end = at + 1;
                while Self::digit(cleaned.get(end).copied()) {
                    tick(work, 1)?;
                    end += 1
                }
            }
            if end > at {
                parts.push(self.convert(&encoded(&cleaned[at..end]), false, work)?);
                at = end
            } else {
                at += 1
            }
        }
        if !parts.is_empty() {
            return Ok(Self::joined(&parts, 45));
        }
        let mut fallback = vec![];
        let mut current = vec![];
        for point in cleaned {
            tick(work, 1)?;
            if self.delimiter(Some(*point), work)? {
                if !current.is_empty() {
                    fallback.push(self.convert(&encoded(&current), false, work)?);
                    current.clear();
                }
            } else {
                current.push(*point)
            }
        }
        if !current.is_empty() {
            fallback.push(self.convert(&encoded(&current), false, work)?)
        }
        Ok(Self::joined(&fallback, 45))
    }
    fn snake(&self, value: &[u16], work: &mut usize) -> Result<Text, String> {
        let chars = points(value);
        let mut result = vec![];
        let mut separator = false;
        for (index, point) in chars.iter().enumerate() {
            tick(work, 1)?;
            if *point == 45 || self.white(Some(*point), work)? {
                if !separator {
                    result.push(95)
                }
                separator = true;
                continue;
            }
            separator = false;
            result.extend(encoded(&[*point]));
            if self.has("lower_letter", Some(*point), work)?
                && self.has("upper_letter", chars.get(index + 1).copied(), work)?
            {
                result.push(95)
            }
        }
        self.convert(&result, false, work)
    }
    pub fn mode(&self, value: &[u16], mode: &str, work: &mut usize) -> Result<Text, String> {
        match mode {
            "upcase" => self.convert(value, true, work),
            "downcase" => self.convert(value, false, work),
            "capitalize" => {
                let mut output = self.convert(&value[..value.len().min(1)], true, work)?;
                output.extend(&value[value.len().min(1)..]);
                Ok(output)
            }
            "camelcase" | "pascalcase" => {
                let words = self.words(value, work)?;
                if words.is_empty() {
                    return Ok(value.to_vec());
                }
                let mut output = vec![];
                for (index, word) in words.iter().enumerate() {
                    output.extend(self.convert(
                        &word[..word.len().min(1)],
                        index > 0 || mode == "pascalcase",
                        work,
                    )?);
                    output.extend(&word[word.len().min(1)..]);
                }
                Ok(output)
            }
            "kebabcase" => self.kebab(value, work),
            "snakecase" => self.snake(value, work),
            _ => Ok(value.to_vec()),
        }
    }
}
pub struct Writer {
    text: Text,
    bytes: usize,
    pending: bool,
}
impl Writer {
    pub fn new() -> Self {
        Self {
            text: vec![],
            bytes: 0,
            pending: false,
        }
    }
    pub fn append(&mut self, value: &[u16]) -> Result<(), String> {
        for unit in value {
            if self.pending {
                if !(0xdc00..=0xdfff).contains(unit) {
                    return Err("Regex result is not valid source Unicode".into());
                }
                self.pending = false;
                self.bytes += 4;
            } else if (0xd800..=0xdbff).contains(unit) {
                self.pending = true
            } else if (0xdc00..=0xdfff).contains(unit) {
                return Err("Regex result is not valid source Unicode".into());
            } else {
                self.bytes += if *unit <= 127 {
                    1
                } else if *unit <= 2047 {
                    2
                } else {
                    3
                }
            }
            if self.bytes > 1_048_576 {
                return Err("Regex output byte budget exceeded".into());
            }
            self.text.push(*unit);
        }
        Ok(())
    }
    pub fn finish(self) -> Result<String, String> {
        if self.pending {
            return Err("Regex result is not valid source Unicode".into());
        }
        String::from_utf16(&self.text)
            .map_err(|_| "Regex result is not valid source Unicode".into())
    }
}
pub fn emit(
    ops: &Value,
    captures: &[Option<Text>],
    policy: &CaseData,
    writer: &mut Writer,
    work: &mut usize,
    depth: usize,
) -> Result<(), String> {
    if depth > 32 {
        return Err("Regex format nesting budget exceeded".into());
    }
    for op in ops.as_array().ok_or("Malformed regex format")? {
        tick(work, 1)?;
        if let Some(value) = op.as_str() {
            writer.append(&value.encode_utf16().collect::<Vec<_>>())?;
            continue;
        }
        let index = op[1].as_u64().ok_or("Malformed format index")? as usize;
        let value = captures
            .get(index)
            .and_then(Option::as_ref)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        if op[0] == "if" {
            emit(
                &op[if value.is_empty() { 3 } else { 2 }],
                captures,
                policy,
                writer,
                work,
                depth + 1,
            )?
        } else {
            writer.append(&policy.mode(
                value,
                op[2].as_str().ok_or("Malformed format mode")?,
                work,
            )?)?
        }
    }
    Ok(())
}
</ui:copy>

## Cache immutable tables and keep the public edit API unchanged

Only explicitly negotiated version 3 compilation reaches this module. Table parsing is cached independently from program admission; a provider cannot change the frozen policy. Evaluation receives the same shared work counter as legacy derived occurrences, and returns one complete string or an error before any editor action is submitted.
<ui:copy id="snippet-regex-v3-engine-api">
//! Versioned ECMAScript data programs; legacy v2 remains in the owning module.
use serde_json::{Value, json};
use std::sync::OnceLock;
fn unicode() -> &'static sets::UnicodeData {
    static DATA: OnceLock<sets::UnicodeData> = OnceLock::new();
    DATA.get_or_init(|| {
        sets::UnicodeData::new(
            serde_json::from_str(include_str!(
                "../../../apps/simple-contract/snippet-regex-unicode17.json"
            ))
            .expect("Frozen Unicode17 properties"),
            &case_json(),
        )
        .expect("Frozen Unicode17 property schema")
    })
}
fn case_json() -> Value {
    serde_json::from_str(include_str!(
        "../../../apps/simple-contract/snippet-regex-case17.json"
    ))
    .expect("Frozen Unicode17 casing")
}
fn case() -> &'static formatter::CaseData {
    static DATA: OnceLock<formatter::CaseData> = OnceLock::new();
    DATA.get_or_init(|| formatter::CaseData::new(&case_json()))
}
pub(super) fn compile(pattern: &str, format: &str, flags: &str) -> Result<Value, String> {
    if pattern.len() > 8192
        || format.len() > 8192
        || pattern.len() + format.len() > 8192
        || flags.len() > 8
    {
        return Err("Regex syntax byte budget exceeded".into());
    };
    let lowered = ast_lowering::lower_pattern(pattern, flags, unicode())?;
    let mut code = bytecode::CodeBuilder::new();
    code.lower(&lowered.tree, flags, 1, &|node, flags| {
        unicode().instruction(node, flags)
    })?;
    let (code, subprograms) = code.finish()?;
    let format = format::compile_format(format, lowered.groups)?;
    let value = json!({"version":3,"case_map":"hickory-ucd-17.0.0-v1","unicode_map":"hickory-regex-ucd-17.0.0-v1","groups":lowered.groups,"flags":flags,"code":code,"subprograms":subprograms,"format":format});
    reader::read(&value)?;
    Ok(value)
}
pub(super) fn validate(raw: &Value) -> Result<(), String> {
    reader::read(raw).map(|_| ())
}
pub(super) fn apply(raw: &Value, input: &str, work: &mut usize) -> Result<String, String> {
    let program = reader::read(raw)?;
    let found = {
        let unicode_mode = program.flags.contains('u') || program.flags.contains('v');
        let mut machine = matcher::Machine::new(input, unicode_mode, unicode(), work)?;
        machine
            .find(&program)?
            .into_iter()
            .map(|found| {
                (
                    machine.offsets[found.from],
                    machine.offsets[found.to],
                    found
                        .captures
                        .into_iter()
                        .map(|range| range.map(|(a, b)| machine.units(a, b)))
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>()
    };
    let source = input.encode_utf16().collect::<Vec<_>>();
    let mut writer = formatter::Writer::new();
    if found.is_empty()
        && program
            .format
            .as_array()
            .unwrap()
            .iter()
            .any(|op| op[0] == "if" && op[3].as_array().is_some_and(|values| !values.is_empty()))
    {
        formatter::emit(&program.format, &[], case(), &mut writer, work, 0)?;
        return writer.finish();
    };
    let mut cursor = 0;
    for (from, to, captures) in found {
        writer.append(&source[cursor..from])?;
        formatter::emit(&program.format, &captures, case(), &mut writer, work, 0)?;
        cursor = to
    }
    writer.append(&source[cursor..])?;
    writer.finish()
}
</ui:copy>
<ui:copy id="snippet-regex-v3-engine-tests">
#[cfg(test)]
mod tests {
    use super::*;
    fn patterns() -> Value {
        serde_json::from_str(include_str!(
            "../../../apps/simple-contract/snippet-regex-pattern-cases.json"
        ))
        .unwrap()
    }
    #[test]
    fn ecmascript_pattern_oracle() {
        let rows = patterns();
        let mut checked = 0;
        for row in rows.as_array().unwrap() {
            let test = &row["case"];
            let compiled = compile(
                test["pattern"].as_str().unwrap(),
                test["format"].as_str().unwrap_or("$0"),
                test["flags"].as_str().unwrap(),
            );
            if test["error"] == true {
                assert!(compiled.is_err(), "Invalid syntax accepted: {test}");
                checked += 1;
                continue;
            };
            let program = reader::read(&compiled.unwrap()).unwrap();
            let mut work = 0;
            let unicode_mode = program.flags.contains('u') || program.flags.contains('v');
            let mut machine = matcher::Machine::new(
                test["input"].as_str().unwrap(),
                unicode_mode,
                unicode(),
                &mut work,
            )
            .unwrap();
            let actual=machine.find(&program).unwrap().into_iter().map(|found|json!({"from":machine.offsets[found.from],"to":machine.offsets[found.to],"indices":found.captures.iter().map(|r|r.map(|(a,b)|[machine.offsets[a],machine.offsets[b]])).collect::<Vec<_>>(),"capture_units":found.captures.iter().map(|r|r.map(|(a,b)|machine.units(a,b))).collect::<Vec<_>>() })).collect::<Vec<_>>();
            let mut expected = test["matches"].clone();
            for row in expected.as_array_mut().unwrap() {
                row.as_object_mut().unwrap().remove("name_units");
            }
            assert_eq!(json!(actual), expected, "{}", test["name"]);
            checked += 1
        }
        assert_eq!(checked, 858)
    }
    #[test]
    fn textmate_format_oracle() {
        let rows: Value = serde_json::from_str(include_str!(
            "../../../apps/simple-contract/snippet-regex-format-cases.json"
        ))
        .unwrap();
        for row in rows.as_array().unwrap() {
            let test = &row["case"];
            let program = compile(
                test["pattern"].as_str().unwrap(),
                test["format"].as_str().unwrap(),
                test["flags"].as_str().unwrap(),
            )
            .unwrap();
            assert_eq!(
                apply(&program, test["input"].as_str().unwrap(), &mut 0).unwrap(),
                test["output"].as_str().unwrap(),
                "{}",
                test["source"]
            )
        }
        assert_eq!(rows.as_array().unwrap().len(), 23)
    }
    #[test]
    fn bounded_wire_and_evaluation_refusals() {
        let rows: Value = serde_json::from_str(include_str!(
            "../../../apps/simple-contract/snippet-regex-budget-cases.json"
        ))
        .unwrap();
        for row in rows["cases"].as_array().unwrap() {
            let result = if row["phase"] == "read" {
                validate(&row["program"]).map(|_| String::new())
            } else {
                apply(
                    &row["program"],
                    row["input"].as_str().unwrap(),
                    &mut (row["work"].as_u64().unwrap() as usize),
                )
            };
            assert!(result.is_err(), "Refusal case accepted: {}", row["name"])
        }
        assert!(compile(&"a".repeat(8193), "$0", "").is_err());
        assert!(compile(&"(".repeat(33), "$0", "").is_err());
        assert!(compile("a", &"x".repeat(8193), "").is_err());
        assert!(compile("a", "$0", &"g".repeat(8193)).is_err());
    }
    #[test]
    fn frozen_empty_hrkt_aliases() {
        for property in [
            "Script=Hrkt",
            "Script=Katakana_Or_Hiragana",
            "Script_Extensions=Hrkt",
            "Script_Extensions=Katakana_Or_Hiragana",
        ] {
            let program = compile(&format!("\\p{{{property}}}"), "changed", "u").unwrap();
            assert_eq!(apply(&program, "あアabc", &mut 0).unwrap(), "あアabc");
        }
    }
    #[test]
    fn named_identifier_escapes_and_unicode17_are_canonical_without_syntax_injection(){
        for pattern in [r"(?<\u0061>a)\k<a>",r"(?<a>a)\k<\u0061>",r"(?<\u{000000061}>a)\k<a>",r"(?<\uD835\uDC9C>a)\k<𝒜>",r"(?<\u{11DB0}>a)\k<𑶰>"] {let program=compile(pattern,"$1","u").unwrap();assert_eq!(apply(&program,"aa",&mut 0).unwrap(),"a", "{pattern}");}
        let program=compile(r"(?<a>a)(?<__h0>b)\k<a>\k<__h0>","$1$2","").unwrap();assert_eq!(apply(&program,"abab",&mut 0).unwrap(),"ab");
        let duplicate=compile(r"(?:(?<a>a)|(?<\u0061>b))\k<a>","$1$2","").unwrap();assert_eq!(apply(&duplicate,"bb",&mut 0).unwrap(),"b");
        for pattern in [r"(?<\u003Dfoo>a)",r"(?<\u0030>a)",r"(?<\u{D835}\u{DC9C}>a)",r"(?<a>a)\k<other>",r"(?<a>a)(?<\u0061>b)"]{assert!(compile(pattern,"$0","").is_err(),"{pattern}");}
    }
}
</ui:copy>
<ui:file path="crates/hick-lsp/src/snippet_transform_v3.rs"><ui:paste select="#snippet-regex-v3-engine-api"/>
mod ast_lowering {
<ui:paste select="#snippet-regex-v3-lowering"/>
}

mod sets {
<ui:paste select="#snippet-regex-v3-sets"/>
}

mod bytecode {
<ui:paste select="#snippet-regex-v3-bytecode"/>
}

mod format {
<ui:paste select="#snippet-regex-v3-format"/>
}

mod reader {
<ui:paste select="#snippet-regex-v3-reader"/>
}

mod matcher {
<ui:paste select="#snippet-regex-v3-matcher"/>
}

mod formatter {
<ui:paste select="#snippet-regex-v3-formatter"/>
}

<ui:paste select="#snippet-regex-v3-engine-tests"/></ui:file>

## Use the same VM in the browser

The browser reader normalizes and freezes provider data before use. Typed opcodes keep matching policy out of renderer code. Input bytes, backtracking, assertions and final output use the same limits as the engine. The pure entry point accepts an action work budget; it never changes editor state itself. Admitted programs and frozen case maps are cached by identity, so another derived occurrence or caret does not reparse metadata or rebuild thousands of Unicode map entries. Source values and match results are never cached.
<ui:copy id="snippet-regex-v3-browser">
import frozenCase from '../../../simple-contract/snippet-regex-case17.json';
export type RegexV3Instruction =
 | ['accept'] | ['jump',number] | ['split',number,number]
 | ['mark'|'capture',number] | ['assert','start'|'end'|'word'|'nonword',boolean,boolean]
 | ['ref',number[],boolean,boolean] | ['set',number[][],number[][],boolean,boolean]
 | ['look',number,boolean,number[]] | ['repeat',number,number,number|null,boolean,number,number,number[]]
 | ['repeat_end',number,number];
export type RegexV3Format = string | ['capture',number,string] | ['if',number,RegexV3Format[],RegexV3Format[]];
export interface RegexV3Program {version:3;case_map:string;unicode_map:string;groups:number;flags:string;code:RegexV3Instruction[];subprograms:{direction:number;code:RegexV3Instruction[]}[];format:RegexV3Format[]}
export interface RegexV3CaseData {fold:[number,number[]][];upper:[number,string][];lower:[number,string][];letter:number[][];cased:number[][];ignorable:number[][];upper_letter:number[][];lower_letter:number[][];space:number[][]}
interface RegexV3Policy {canonical:(cp:number,full:boolean)=>number}
interface RegexV3State {pc:number;at:number;captures:([number,number]|null)[];marks:(number|null)[];repeat:Map<number,{count:number;at:number}>}
interface RegexV3Budget {work:number}

// Pure bounded v3 control-flow evaluator prototype. No eval/native RegExp.
const MAX_WORK=100000,MAX_STACK=1024;
const isScalar=(source:string)=>!/[\uD800-\uDFFF]/u.test(source);
const regexV3PolicyCache=new WeakMap<RegexV3CaseData,RegexV3Policy>();
export function casePolicy(data:RegexV3CaseData):RegexV3Policy{
 const cached=regexV3PolicyCache.get(data);if(cached)return cached;
 const unicode=new Map<number,number>(),legacy=new Map<number,number>();for(const [cp,peers]of data.fold){const least=Math.min(cp,...peers);unicode.set(cp,least);}for(const [cp,text]of data.upper){if(cp>65535||text.length!==1)continue;const value=text.charCodeAt(0);if(cp>=128&&value<128)continue;legacy.set(cp,value);}const policy=Object.freeze({canonical:(cp:number,full:boolean)=> (full?unicode:legacy).get(cp)??cp});regexV3PolicyCache.set(data,policy);return policy;
}
function units(source:string,unicode:boolean):number[]{if(unicode)return Array.from(source,char=>char.codePointAt(0)!);const result=[];for(let at=0;at<source.length;at++)result.push(source.charCodeAt(at));return result;}
function text(points:number[],unicode:boolean):string{let result='';for(let at=0;at<points.length;at+=4096)result+=unicode?String.fromCodePoint(...points.slice(at,at+4096)):String.fromCharCode(...points.slice(at,at+4096));return result;}
export function execute(program:RegexV3Program,input:string,policy:RegexV3Policy,budget:RegexV3Budget={work:0}){
 if(input.length>65536||!isScalar(input)||new TextEncoder().encode(input).length>65536)throw Error('Regex input budget or scalar validation failed');const unicode=program.flags.includes('u')||program.flags.includes('v'),chars=units(input,unicode),offsets:number[]=[0];for(const cp of chars)offsets.push(offsets[offsets.length-1]+(unicode&&cp>65535?2:1));let retained=0;
 const tick=(cost=1)=>{budget.work+=cost;if(!Number.isSafeInteger(budget.work)||budget.work<0||budget.work>MAX_WORK)throw Error('Regex work budget exceeded');};
 const duplicate=(state:RegexV3State):RegexV3State=>{tick(state.captures.length+state.marks.length+state.repeat.size);return {...state,captures:state.captures.slice(),marks:state.marks.slice(),repeat:new Map(state.repeat)};};
 const captureClear=(state:RegexV3State,indices:number[])=>{for(const index of indices){state.captures[index]=null;state.marks[index]=null;tick();}};
 const newline=(cp:number|undefined)=>cp===10||cp===13||cp===0x2028||cp===0x2029;
 const word=(cp:number|undefined,ignore:boolean)=>{if(cp===undefined)return false;if(ignore)cp=policy.canonical(cp,true);return cp>=65&&cp<=90||cp>=97&&cp<=122||cp>=48&&cp<=57||cp===95;};
 const member=(ranges:number[][],cp:number)=>{let low=0,high=ranges.length;while(low<high){tick();const mid=(low+high)>>>1;if(ranges[mid][0]<=cp)low=mid+1;else high=mid;}return low>0&&cp<=ranges[low-1][1];};
 const run=(code:RegexV3Instruction[],start:number,captures:([number,number]|null)[],marks:(number|null)[],direction:number,depth:number):RegexV3State|null=>{
  if(depth>32)throw Error('Regex assertion nesting budget exceeded');tick(captures.length+marks.length);let state:RegexV3State={pc:0,at:start,captures:captures.slice(),marks:marks.slice(),repeat:new Map()};const stack:RegexV3State[]=[];
  const push=(value:RegexV3State)=>{if(++retained>MAX_STACK)throw Error('Regex backtrack memory budget exceeded');stack.push(value);};
  const fail=()=>{if(!stack.length)return false;state=stack.pop()!;retained--;return true;};
  try{for(;;){tick();const instruction=code[state.pc];if(!instruction)throw Error('Regex instruction outside program');let failed=false;
   switch(instruction[0]){
    case 'accept':return state;
    case 'jump':state.pc=instruction[1];continue;
    case 'split':{const alternative=duplicate(state);alternative.pc=instruction[2];push(alternative);state.pc=instruction[1];continue;}
    case 'mark':state=duplicate(state);state.marks[instruction[1]]=state.at;state.captures[instruction[1]]=null;break;
    case 'capture':{const mark=state.marks[instruction[1]];if(mark===null||mark===undefined)throw Error('Regex capture has no start');state=duplicate(state);state.captures[instruction[1]]=[Math.min(mark,state.at),Math.max(mark,state.at)];break;}
    case 'assert':{const kind=instruction[1],before=chars[state.at-1],next=chars[state.at],boundary=word(before,instruction[3])!==word(next,instruction[3]);const yes=kind==='start'?state.at===0||instruction[2]&&newline(before):kind==='end'?state.at===chars.length||instruction[2]&&newline(next):kind==='word'?boundary:!boundary;failed=!yes;break;}
    case 'ref':{const captured=instruction[1].map(index=>state.captures[index]).filter((value):value is [number,number]=>value!==null);if(captured.length>1)throw Error('Ambiguous named regex reference');if(!captured.length)break;const [from,to]=captured[0],length=to-from,end=state.at+direction*length,begin=Math.min(state.at,end);if(begin<0||end>chars.length){failed=true;break;}for(let at=0;at<length;at++){tick();const expected=chars[from+at],actual=chars[begin+at];if(actual===undefined||(instruction[2]?policy.canonical(expected,instruction[3])!==policy.canonical(actual,instruction[3]):expected!==actual)){failed=true;break;}}if(!failed)state.at=end;break;}
    case 'set':{const candidates=[];for(const value of instruction[2]){tick();const end=state.at+direction*value.length,begin=Math.min(state.at,end);if(begin<0||end>chars.length)continue;let yes=true;for(let at=0;at<value.length;at++){tick();const actual=chars[begin+at];if((instruction[3]?policy.canonical(actual,instruction[4]):actual)!==value[at]){yes=false;break;}}if(yes)candidates.push(value.length);}const next=chars[direction===1?state.at:state.at-1];if(next!==undefined&&member(instruction[1],next))candidates.push(1);candidates.sort((a,b)=>b-a);if(!candidates.length){failed=true;break;}for(let at=candidates.length-1;at>0;at--){const alternative=duplicate(state);alternative.at+=direction*candidates[at];alternative.pc++;push(alternative);}state.at+=direction*candidates[0];break;}
    case 'look':{state=duplicate(state);captureClear(state,instruction[3]);const sub=program.subprograms[instruction[1]],result=run(sub.code,state.at,state.captures,state.marks,sub.direction,depth+1);if(instruction[2]){if(!result){failed=true;break;}state.captures=result.captures;state.marks=result.marks;}else if(result)failed=true;break;}
    case 'repeat':{const [,id,min,max,lazy,body,end,captures]=instruction,frame=state.repeat.get(id)??{count:0,at:state.at};const exit=()=>{const result=duplicate(state);result.repeat.delete(id);result.pc=end;return result;};const more=()=>{const result=duplicate(state);result.repeat.set(id,{count:frame.count,at:state.at});captureClear(result,captures);result.pc=body;return result;};const canExit=frame.count>=min,canRepeat=max===null||frame.count<max;if(canExit&&canRepeat){if(lazy){push(more());state=exit();}else{push(exit());state=more();}}else if(canExit)state=exit();else if(canRepeat)state=more();else failed=true;if(!failed)continue;break;}
    case 'repeat_end':{const id=instruction[1],entry=state.repeat.get(id);if(!entry)throw Error('Regex repeat frame missing');const repeat=code[instruction[2]];if(repeat[0]!=='repeat')throw Error('Regex repeat edge mismatch');if(entry.at===state.at&&entry.count>=repeat[2]){failed=true;break;}state=duplicate(state);state.repeat.set(id,{count:entry.count+1,at:entry.at});state.pc=instruction[2];continue;}
    default:throw Error('Unknown regex instruction');
   }
   if(failed){if(!fail())return null;}else state.pc++;
  }}finally{retained-=stack.length;}
 };
 const results:{from:number;to:number;captures:([number,number]|null)[]}[]=[];let search=0;
 while(search<=chars.length){let found:{from:number;to:number;captures:([number,number]|null)[]}|null=null;for(let at=search;at<=chars.length;at++){const result=run(program.code,at,Array(program.groups+1).fill(null),Array(program.groups+1).fill(null),1,0);if(result){found={from:at,to:result.at,captures:result.captures};found.captures[0]=[at,result.at];break;}if(program.flags.includes('y'))break;}if(!found)break;results.push(found);if(!program.flags.includes('g'))break;search=found.to===found.from?found.to+1:found.to;}
 const utf16=(at:number)=>offsets[at];
 return results.map(result=>({from:utf16(result.from),to:utf16(result.to),captures:result.captures.map(range=>range?text(chars.slice(...range),unicode):null),indices:result.captures.map(range=>range?[utf16(range[0]),utf16(range[1])]:null)}));
}

// Unicode17 TextMate format policy, using frozen data and bounded scans.
const regexV3FormatCache=new WeakMap<RegexV3CaseData,{upper:Map<number,string>;lower:Map<number,string>}>();
export function formatter(data:RegexV3CaseData,tick:()=>void=()=>{}){
 let maps=regexV3FormatCache.get(data);if(!maps){maps={upper:new Map(data.upper),lower:new Map(data.lower)};regexV3FormatCache.set(data,maps);}const {upper,lower}=maps;
 const contains=(ranges:number[][],point:number)=>{let low=0,high=ranges.length;while(low<high){tick();const mid=(low+high)>>>1;if(ranges[mid][0]<=point)low=mid+1;else high=mid;}return low>0&&point<=ranges[low-1][1];};
 const is=(kind:'letter'|'cased'|'ignorable'|'upper_letter'|'lower_letter'|'space',ch:string|undefined)=>ch!==undefined&&contains(data[kind],ch.codePointAt(0)!);
 const convert=(value:string,mode:string)=>{const chars=Array.from(value),map=mode==='upper'?upper:lower;return chars.map((ch,index)=>{tick();if(mode==='lower'&&ch==='Σ'){let before=index-1,after=index+1;while(before>=0&&is('ignorable',chars[before])){tick();before--;}while(after<chars.length&&is('ignorable',chars[after])){tick();after++;}if(before>=0&&is('cased',chars[before])&&(after===chars.length||!is('cased',chars[after])))return 'ς';}return map.get(ch.codePointAt(0)!)??ch;}).join('');};
 const digit=(ch:string|undefined)=>ch!==undefined&&ch>='0'&&ch<='9';
 const white=(ch:string|undefined)=>ch!==undefined&&(ch==='\t'||ch==='\n'||ch==='\v'||ch==='\f'||ch==='\r'||ch==='\uFEFF'||ch==='\u2028'||ch==='\u2029'||is('space',ch));
 const delimiter=(ch:string|undefined)=>ch==='_'||ch==='-'||white(ch);
 const words=(value:string)=>{const result=[];let current='';for(const ch of value){tick();if(is('letter',ch)||digit(ch))current+=ch;else if(current){result.push(current);current='';}}if(current)result.push(current);return result;};
 const kebab=(value:string)=>{const chars=Array.from(value);if(!chars.some(ch=>{tick();return is('letter',ch)||digit(ch);}))return value;let first=0,last=chars.length;while(first<last&&white(chars[first]))first++;while(last>first&&white(chars[last-1]))last--;while(first<last&&chars[first]==='_')first++;while(last>first&&chars[last-1]==='_')last--;const cleaned=chars.slice(first,last),parts=[];
  for(let at=0;at<cleaned.length;){tick();let end=at;
   let run=at;while(run<cleaned.length&&is('upper_letter',cleaned[run])){tick();run++;}
   for(let stop=run;stop-at>=2;stop--){tick();const ch=cleaned[stop];if(stop===cleaned.length||delimiter(ch)||is('upper_letter',ch)&&is('lower_letter',cleaned[stop+1])){end=stop;break;}}
   if(end===at){let stop=at;if(is('upper_letter',cleaned[stop]))stop++;const begin=stop;while(stop<cleaned.length&&is('lower_letter',cleaned[stop])){tick();stop++;}if(stop>begin){while(digit(cleaned[stop])){tick();stop++;}end=stop;}}
   if(end===at&&is('upper_letter',cleaned[at])&&is('upper_letter',cleaned[at+1])&&is('lower_letter',cleaned[at+2]))end=at+1;
   if(end===at&&is('upper_letter',cleaned[at])&&(at+1===cleaned.length||delimiter(cleaned[at+1])))end=at+1;
   if(end===at&&digit(cleaned[at])){end=at+1;while(digit(cleaned[end])){tick();end++;}}
   if(end>at){parts.push(convert(cleaned.slice(at,end).join(''),'lower'));at=end;}else at++;
  }
  if(parts.length)return parts.join('-');const fallback=[];let current='';for(const ch of cleaned){tick();if(delimiter(ch)){if(current){fallback.push(convert(current,'lower'));current='';}}else current+=ch;}if(current)fallback.push(convert(current,'lower'));return fallback.join('-');
 };
 const snake=(value:string)=>{const chars=Array.from(value),parts=[];let separator=false;for(let at=0;at<chars.length;at++){tick();const ch=chars[at];if(white(ch)||ch==='-'){if(!separator)parts.push('_');separator=true;continue;}separator=false;parts.push(ch);if(is('lower_letter',ch)&&is('upper_letter',chars[at+1]))parts.push('_');}return convert(parts.join(''),'lower');};
 return (value:string,mode:string)=>{switch(mode){case 'upcase':return convert(value,'upper');case 'downcase':return convert(value,'lower');case 'capitalize':return convert(value.slice(0,1),'upper')+value.slice(1);case 'camelcase':case 'pascalcase':{const found=words(value);return found.length?found.map((word,index)=>convert(word.slice(0,1),index===0&&mode==='camelcase'?'lower':'upper')+word.slice(1)).join(''):value;}case 'kebabcase':return kebab(value);case 'snakecase':return snake(value);default:return value;}};
}
export function appendBudget(maximum=1048576){let bytes=0,pending=false;const parts:string[]=[];return {append(value:string){for(let at=0;at<value.length;at++){const unit=value.charCodeAt(at);if(pending){if(unit<0xdc00||unit>0xdfff)throw Error('Regex result is not valid source Unicode');pending=false;bytes+=4;}else if(unit>=0xd800&&unit<=0xdbff){pending=true;}else if(unit>=0xdc00&&unit<=0xdfff)throw Error('Regex result is not valid source Unicode');else bytes+=unit<=127?1:unit<=2047?2:3;if(bytes>maximum)throw Error('Regex output budget exceeded');}parts.push(value);},finish(){if(pending)throw Error('Regex result is not valid source Unicode');return parts.join('');}};}
export function emitFormat(ops:RegexV3Format[],captures:(string|null)[],mode:(value:string,mode:string)=>string,append:(value:string)=>void,tick:()=>void,depth=0){if(depth>32)throw Error('Regex format depth budget exceeded');for(const op of ops){tick();if(typeof op==='string'){append(op);continue;}const value=captures[op[1]]??'';if(op[0]==='if')emitFormat(op[value?2:3],captures,mode,append,tick,depth+1);else append(mode(value,op[2]));}}

// Wire admission is separate from matching: malformed data never reaches the VM.
const keys=(value:unknown,names:string[]):value is Record<string,unknown> =>value!==null&&typeof value==='object'&&!Array.isArray(value)&&Object.keys(value).length===names.length&&Object.keys(value).every(key=>names.includes(key));
const fail:(message:string)=>never=(message)=>{throw Error(message);};
const integer=(raw:unknown,max:number):number=>typeof raw==='number'&&Number.isSafeInteger(raw)&&raw>=0&&raw<=max?raw:fail('Malformed regex integer');
const boolean=(raw:unknown):boolean=>typeof raw==='boolean'?raw:fail('Malformed regex boolean');
const list=(raw:unknown):unknown[]=>Array.isArray(raw)?raw:fail('Malformed regex array');
const scalar=(source:string):boolean=>{for(let at=0;at<source.length;at++){const cp=source.charCodeAt(at);if(cp>=0xd800&&cp<=0xdbff){const next=source.charCodeAt(++at);if(!(next>=0xdc00&&next<=0xdfff))return false;}else if(cp>=0xdc00&&cp<=0xdfff)return false;}return true;};
const admittedRegexV3=new WeakSet<object>();
const admittedRegexV3Bytes=new WeakMap<object,number>();
export function readRegexV3(raw:unknown):RegexV3Program{
 if(raw&&typeof raw==='object'&&admittedRegexV3.has(raw))return raw as RegexV3Program;
 if(!keys(raw,['version','case_map','unicode_map','groups','flags','code','subprograms','format'])||raw.version!==3||raw.case_map!=='hickory-ucd-17.0.0-v1'||raw.unicode_map!=='hickory-regex-ucd-17.0.0-v1')fail('Unsupported regex program');
 const groups=integer(raw.groups,32),flags=raw.flags;if(typeof flags!=='string'||flags.length>8||[...flags].some(ch=>!'dgimsuvy'.includes(ch))||new Set(flags).size!==flags.length||flags.includes('u')&&flags.includes('v'))fail('Malformed regex flags');const unicode=flags.includes('u')||flags.includes('v');
 let nodes=0,instructions=0;const tick=(n:number)=>{if(n<0||nodes>262144-n)fail('Regex metadata budget exceeded');nodes+=n;};const subs=list(raw.subprograms);if(subs.length>128)fail('Regex assertion budget exceeded');
 const indices=(value:unknown):number[]=>{const result=list(value).map(v=>integer(v,groups));if(result.length>groups||result.includes(0)||new Set(result).size!==result.length)fail('Malformed regex capture list');return result;};
 const code=(raw:unknown,direction:number)=>{const rows=list(raw);if(!rows.length||rows.length>4096||instructions>4096-rows.length)fail('Regex instruction budget exceeded');instructions+=rows.length;tick(rows.length);const pc=(v:unknown)=>integer(v,rows.length-1),group=(v:unknown)=>integer(v,groups)||fail('Regex group zero write');const result=rows.map(value=>{const op=list(value),kind=op[0];tick(op.length);const size=(n:number)=>{if(op.length!==n)fail('Malformed regex instruction');};switch(kind){
 case 'accept':size(1);return [kind];case 'jump':size(2);return [kind,pc(op[1])];case 'split':size(3);return [kind,pc(op[1]),pc(op[2])];case 'mark':case 'capture':size(2);return [kind,group(op[1])];
 case 'assert':size(4);if(typeof op[1]!=='string'||!['start','end','word','nonword'].includes(String(op[1]))||boolean(op[3])&&!unicode)fail('Malformed regex assertion');return [kind,op[1],boolean(op[2]),op[3]];
 case 'ref':size(4);if(boolean(op[3])!==unicode)fail('Regex reference mode mismatch');return [kind,indices(op[1]),boolean(op[2]),unicode];
 case 'look':size(4);if(!subs.length)fail('Missing regex assertion');return [kind,integer(op[1],subs.length-1),boolean(op[2]),indices(op[3])];
 case 'repeat':{size(8);const min=integer(op[2],Number.MAX_SAFE_INTEGER),max=op[3]===null?null:integer(op[3],Number.MAX_SAFE_INTEGER);if(max!==null&&max<min)fail('Backwards regex repeat');return [kind,integer(op[1],4095),min,max,boolean(op[4]),pc(op[5]),pc(op[6]),indices(op[7])];}
 case 'repeat_end':size(3);return [kind,integer(op[1],4095),pc(op[2])];
 case 'set':{size(5);if(boolean(op[4])!==unicode)fail('Regex set mode mismatch');const ranges=list(op[1]),strings=list(op[2]);if(ranges.length>65536||strings.length>65536)fail('Regex set budget');tick(ranges.length+strings.length);const maximum=unicode?0x10ffff:65535;let last=-2;const rs=ranges.map(raw=>{const pair=list(raw);if(pair.length!==2)fail('Malformed regex range');const low=integer(pair[0],maximum),high=integer(pair[1],maximum);if(low>high||last+1>=low)fail('Unnormalized regex ranges');last=high;return [low,high];});let prior:number[]|null=null;const ss=strings.map(raw=>{const chars=list(raw);if(chars.length>8192||chars.length===1)fail('Malformed regex set string');tick(chars.length);const value=chars.map(cp=>integer(cp,maximum));if(prior){let order=0;for(let i=0;i<Math.min(prior.length,value.length);i++)if(prior[i]!==value[i]){order=prior[i]-value[i];break;}if(prior.length<value.length||prior.length===value.length&&order>=0)fail('Unnormalized regex strings');}prior=value;return value;});return [kind,rs,ss,boolean(op[3]),unicode];}
 default:fail('Unknown regex instruction');}}) as RegexV3Instruction[];for(const op of result)if(op[0]==='repeat_end'&&(result[op[2]][0]!=='repeat'||result[op[2]][1]!==op[1]))fail('Regex repeat edge mismatch');return {direction,code:result};};
 const main=code(raw.code,1),subprograms=subs.map(sub=>{if(!keys(sub,['direction','code'])||typeof sub.direction!=="number"||![1,-1].includes(sub.direction))fail('Malformed regex subprogram');return code(sub.code,sub.direction);});
 const format=(raw:unknown,depth=0):RegexV3Format[]=>{const ops=list(raw);if(depth>32||ops.length>512)fail('Regex format budget');tick(ops.length);return ops.map(raw=>{if(typeof raw==='string'){if(raw.length>16384||!scalar(raw))fail('Regex format text budget');const bytes=new TextEncoder().encode(raw).length;if(bytes>16384)fail('Regex format text budget');tick(bytes);return raw;}const op=list(raw);if(op[0]==='capture'&&op.length===3&&typeof op[2]==='string'&&['','upcase','downcase','capitalize','camelcase','pascalcase','kebabcase','snakecase'].includes(String(op[2])))return ['capture',integer(op[1],groups+1),String(op[2])];if(op[0]==='if'&&op.length===4)return ['if',integer(op[1],groups+1),format(op[2],depth+1),format(op[3],depth+1)];fail('Unknown regex format');});};
 const result={version:3,case_map:raw.case_map,unicode_map:raw.unicode_map,groups,flags,code:main.code,subprograms,format:format(raw.format)};const byteCount=regexV3MetadataBytes(result);const freeze=(value:unknown):unknown=>{if(value&&typeof value==='object'){for(const child of Object.values(value))freeze(child);Object.freeze(value);}return value;};const admitted=freeze(result) as RegexV3Program;admittedRegexV3.add(admitted);admittedRegexV3Bytes.set(admitted,byteCount);return admitted;
}

export function applyRegexV3(program:RegexV3Program,input:string,data:RegexV3CaseData,budget:RegexV3Budget={work:0}):string {const matches=execute(program,input,casePolicy(data),budget),output=appendBudget(),tick=()=>{if(++budget.work>100000)throw Error('Regex format work budget exceeded');},mode=formatter(data,tick);if(!matches.length&&program.format.some(op=>typeof op!=='string'&&op[0]==='if'&&op[3].length)){emitFormat(program.format,[],mode,value=>output.append(value),tick);return output.finish();}let cursor=0;for(const found of matches){output.append(input.slice(cursor,found.from));emitFormat(program.format,found.captures,mode,value=>output.append(value),tick);cursor=found.to;}output.append(input.slice(cursor));return output.finish();}

function regexV3MetadataBytes(raw:unknown):number {let total=0;const add=(n:number)=>{if(n<0||total>1048576-n)throw Error('Regex metadata byte budget exceeded');total+=n;};const string=(value:string)=>{add(2);for(const char of value){const cp=char.codePointAt(0)!;add([34,92,8,9,10,12,13].includes(cp)?2:cp<32?6:cp<=127?1:cp<=2047?2:cp<=65535?3:4);}};const visit=(value:unknown,depth:number)=>{if(depth>128)throw Error('Regex metadata nesting exceeded');if(value===null)add(4);else if(typeof value==='boolean')add(value?4:5);else if(typeof value==='string')string(value);else if(typeof value==='number'){if(!Number.isSafeInteger(value))throw Error('Malformed regex metadata integer');add(String(value).length);}else if(Array.isArray(value)){add(2+Math.max(0,value.length-1));for(const child of value)visit(child,depth+1);}else if(value&&typeof value==='object'){const entries=Object.entries(value);add(2+Math.max(0,entries.length-1));for(const [key,child]of entries){string(key);add(1);visit(child,depth+1);}}else throw Error('Malformed regex metadata value');};visit(raw,0);return total;}

export const regexV3ProgramBytes=(program:RegexV3Program):number=>admittedRegexV3Bytes.get(readRegexV3(program))!;
const frozenRegexV3Case=frozenCase as unknown as RegexV3CaseData;
export const evaluateRegexV3=(program:RegexV3Program,input:string,budget:RegexV3Budget={work:0})=>applyRegexV3(program,input,frozenRegexV3Case,budget);
</ui:copy>
<ui:file path="apps/web/src/simple/SnippetTransformV3.ts"><ui:paste select="#snippet-regex-v3-browser"/></ui:file>

## Keep Apple evaluation independent of AppKit and UIKit

The pure Swift reader and evaluator run on macOS and iOS. Captures and matching offsets retain exact UTF-16 coordinates. Immutable typed case data is initialized once; no global untyped provider object crosses concurrency boundaries. Source edit ownership, revision and history checks remain in the existing adapters.
<ui:copy id="snippet-regex-v3-apple">
import CoreFoundation
import Foundation

private func regexV3Error(_ message: String) -> EngineError { EngineError(message: message) }
struct RegexV3Program {
  let groups: Int
  let flags: String
  let format: [Any]
  let bytes: Int
  fileprivate let codes: [RegexV3Code]
}
private struct RegexV3Code {
  let direction: Int
  let ops: [[Any]]
}
private struct RegexV3Meter {
  var nodes = 0
  var instructions = 0
  mutating func tick(_ amount: Int) throws {
    guard amount >= 0, nodes <= 262144 - amount else {
      throw regexV3Error("Regex metadata budget exceeded")
    }
    nodes += amount
  }
}
private func regexV3Integer(_ raw: Any, _ maximum: Int) throws -> Int {
  guard let value = raw as? NSNumber, CFGetTypeID(value) != CFBooleanGetTypeID(),
    value.doubleValue.isFinite, value.doubleValue >= 0, value.doubleValue <= Double(maximum),
    value.doubleValue.rounded(.towardZero) == value.doubleValue
  else { throw regexV3Error("Malformed regex integer") }
  return Int(value.doubleValue)
}
private func regexV3Boolean(_ raw: Any) throws -> Bool {
  guard let value = raw as? NSNumber, CFGetTypeID(value) == CFBooleanGetTypeID() else {
    throw regexV3Error("Malformed regex boolean")
  }
  return value.boolValue
}
private func regexV3Indices(_ raw: Any, _ groups: Int) throws -> [Int] {
  guard let values = raw as? [Any], values.count <= groups else {
    throw regexV3Error("Malformed regex capture list")
  }
  var result = [Int]()
  for raw in values {
    let value = try regexV3Integer(raw, groups)
    guard value > 0, !result.contains(value) else {
      throw regexV3Error("Malformed regex capture list")
    }
    result.append(value)
  }
  return result
}
private func regexV3Format(_ raw: Any, _ groups: Int, _ depth: Int, _ meter: inout RegexV3Meter)
  throws -> [Any]
{
  guard depth <= 32, let values = raw as? [Any], values.count <= 512 else {
    throw regexV3Error("Regex format budget exceeded")
  }
  try meter.tick(values.count)
  return try values.map { raw -> Any in
    if let text = raw as? String {
      guard text.utf16.prefix(16385).count <= 16384, text.utf8.prefix(16385).count <= 16384 else {
        throw regexV3Error("Regex format text budget exceeded")
      }
      try meter.tick(text.utf8.count)
      return text
    }
    guard let op = raw as? [Any], let kind = op.first as? String, kind.utf8.prefix(13).count <= 12
    else {
      throw regexV3Error("Malformed regex format")
    }
    if kind == "capture", op.count == 3, let mode = op[2] as? String,
      ["", "upcase", "downcase", "capitalize", "camelcase", "pascalcase", "kebabcase", "snakecase"]
        .contains(mode)
    {
      return [kind, try regexV3Integer(op[1], groups + 1), mode]
    }
    if kind == "if", op.count == 4 {
      return [
        kind, try regexV3Integer(op[1], groups + 1),
        try regexV3Format(op[2], groups, depth + 1, &meter),
        try regexV3Format(op[3], groups, depth + 1, &meter),
      ]
    }
    throw regexV3Error("Unknown regex format operation")
  }
}
private func regexV3Code(
  _ raw: Any, _ direction: Int, _ groups: Int, _ subs: Int, _ unicode: Bool,
  _ meter: inout RegexV3Meter
) throws -> RegexV3Code {
  guard let rows = raw as? [Any], !rows.isEmpty, rows.count <= 4096,
    meter.instructions <= 4096 - rows.count
  else { throw regexV3Error("Regex instruction budget exceeded") }
  meter.instructions += rows.count
  try meter.tick(rows.count)
  func pc(_ value: Any) throws -> Int { try regexV3Integer(value, rows.count - 1) }
  func group(_ value: Any) throws -> Int {
    let value = try regexV3Integer(value, groups)
    guard value > 0 else { throw regexV3Error("Regex group zero cannot be written") }
    return value
  }
  var normalized = [[Any]]()
  for rawOp in rows {
    guard let op = rawOp as? [Any], op.count <= 8 else {
      throw regexV3Error("Regex instruction arity budget exceeded")
    }
    try meter.tick(op.count)
    guard let kind = op.first as? String, kind.utf8.prefix(13).count <= 12 else {
      throw regexV3Error("Malformed regex instruction")
    }
    let value: [Any]
    switch kind {
    case "accept":
      guard op.count == 1 else { throw regexV3Error("Malformed regex accept") }
      value = [kind]
    case "jump":
      guard op.count == 2 else { throw regexV3Error("Malformed regex jump") }
      value = [kind, try pc(op[1])]
    case "split":
      guard op.count == 3 else { throw regexV3Error("Malformed regex split") }
      value = [kind, try pc(op[1]), try pc(op[2])]
    case "mark", "capture":
      guard op.count == 2 else { throw regexV3Error("Malformed regex capture") }
      value = [kind, try group(op[1])]
    case "assert":
      guard op.count == 4, let assertion = op[1] as? String,
        ["start", "end", "word", "nonword"].contains(assertion)
      else { throw regexV3Error("Malformed regex assertion") }
      let insensitive = try regexV3Boolean(op[3])
      guard !insensitive || unicode else { throw regexV3Error("Regex assertion mode mismatch") }
      value = [kind, assertion, try regexV3Boolean(op[2]), insensitive]
    case "ref":
      guard op.count == 4, try regexV3Boolean(op[3]) == unicode else {
        throw regexV3Error("Malformed regex reference")
      }
      value = [kind, try regexV3Indices(op[1], groups), try regexV3Boolean(op[2]), unicode]
    case "look":
      guard op.count == 4, subs > 0 else { throw regexV3Error("Missing regex assertion program") }
      value = [
        kind, try regexV3Integer(op[1], subs - 1) + 1, try regexV3Boolean(op[2]),
        try regexV3Indices(op[3], groups),
      ]
    case "repeat":
      guard op.count == 8 else { throw regexV3Error("Malformed regex repetition") }
      let min = try regexV3Integer(op[2], 9_007_199_254_740_991)
      let max = op[3] is NSNull ? nil : try regexV3Integer(op[3], 9_007_199_254_740_991)
      guard max == nil || max! >= min else { throw regexV3Error("Backwards regex repetition") }
      value = [
        kind, try regexV3Integer(op[1], 4095), min, max as Any? ?? NSNull(),
        try regexV3Boolean(op[4]), try pc(op[5]), try pc(op[6]), try regexV3Indices(op[7], groups),
      ]
    case "repeat_end":
      guard op.count == 3 else { throw regexV3Error("Malformed regex repetition edge") }
      value = [kind, try regexV3Integer(op[1], 4095), try pc(op[2])]
    case "set":
      guard op.count == 5, try regexV3Boolean(op[4]) == unicode, let rawRanges = op[1] as? [Any],
        rawRanges.count <= 65536, let rawStrings = op[2] as? [Any], rawStrings.count <= 65536
      else { throw regexV3Error("Malformed regex set") }
      try meter.tick(rawRanges.count + rawStrings.count)
      let maximum = unicode ? 0x10ffff : 65535
      var ranges = [[Int]]()
      for rawPair in rawRanges {
        guard let pair = rawPair as? [Any], pair.count == 2 else {
          throw regexV3Error("Malformed regex range")
        }
        let low = try regexV3Integer(pair[0], maximum)
        let high = try regexV3Integer(pair[1], maximum)
        guard low <= high, ranges.last.map({ $0[1] + 1 < low }) ?? true else {
          throw regexV3Error("Unnormalized regex ranges")
        }
        ranges.append([low, high])
      }
      var strings = [[Int]]()
      for rawValue in rawStrings {
        guard let raw = rawValue as? [Any], raw.count <= 8192, raw.count != 1 else {
          throw regexV3Error("Malformed regex set string")
        }
        try meter.tick(raw.count)
        let text = try raw.map { try regexV3Integer($0, maximum) }
        if let prior = strings.last {
          guard
            prior.count > text.count
              || prior.count == text.count && prior.lexicographicallyPrecedes(text)
          else { throw regexV3Error("Unnormalized regex set strings") }
        }
        strings.append(text)
      }
      value = [kind, ranges, strings, try regexV3Boolean(op[3]), unicode]
    default: throw regexV3Error("Unknown regex instruction")
    }
    normalized.append(value)
  }
  for op in normalized where op[0] as! String == "repeat_end" {
    let start = op[2] as! Int
    guard normalized[start][0] as? String == "repeat", normalized[start][1] as? Int == op[1] as? Int
    else { throw regexV3Error("Regex repeat edge mismatch") }
  }
  return RegexV3Code(direction: direction, ops: normalized)
}
func readRegexV3(_ raw: Any) throws -> RegexV3Program {
  guard let value = raw as? [String: Any], value.count == 8,
    value.keys.allSatisfy({ $0.utf8.prefix(33).count <= 32 }),
    Set(value.keys)
      == Set([
        "version", "case_map", "unicode_map", "groups", "flags", "code", "subprograms", "format",
      ]), try regexV3Integer(value["version"]!, 3) == 3,
    value["case_map"] as? String == "hickory-ucd-17.0.0-v1",
    value["unicode_map"] as? String == "hickory-regex-ucd-17.0.0-v1",
    let flags = value["flags"] as? String, flags.utf16.prefix(9).count <= 8,
    flags.allSatisfy({ "dgimsuvy".contains($0) }), Set(flags).count == flags.count,
    !(flags.contains("u") && flags.contains("v")),
    let subs = value["subprograms"] as? [Any], subs.count <= 128
  else { throw regexV3Error("Unsupported or malformed regex v3 program") }
  let groups = try regexV3Integer(value["groups"]!, 32)
  let unicode = flags.contains("u") || flags.contains("v")
  var meter = RegexV3Meter()
  let format = try regexV3Format(value["format"]!, groups, 0, &meter)
  var codes = [try regexV3Code(value["code"]!, 1, groups, subs.count, unicode, &meter)]
  for rawSub in subs {
    guard let sub = rawSub as? [String: Any], sub.count == 2,
      Set(sub.keys) == Set(["direction", "code"]), let number = sub["direction"] as? NSNumber,
      CFGetTypeID(number) != CFBooleanGetTypeID(), [-1.0, 1.0].contains(number.doubleValue)
    else { throw regexV3Error("Malformed regex assertion direction") }
    codes.append(
      try regexV3Code(sub["code"]!, number.intValue, groups, subs.count, unicode, &meter))
  }
  let bytes = try regexV3MetadataBytes(value)
  return RegexV3Program(groups: groups, flags: flags, format: format, bytes: bytes, codes: codes)
}

struct RegexV3CaseData: Sendable {
  let unicode: [Int: Int]
  let legacy: [Int: Int]
  let upper: [Int: [UInt16]]
  let lower: [Int: [UInt16]]
  let ranges: [String: [[Int]]]
  init(_ raw: [String: Any]) throws {
    guard raw["unicode"] as? String == "17.0.0" else {
      throw regexV3Error("Wrong regex case data version")
    }
    func mapping(_ name: String) throws -> [Int: [UInt16]] {
      guard let rows = raw[name] as? [[Any]] else {
        throw regexV3Error("Missing regex case mappings")
      }
      var result = [Int: [UInt16]]()
      for row in rows {
        guard row.count == 2, let text = row[1] as? String else {
          throw regexV3Error("Malformed regex case mapping")
        }
        result[try regexV3Integer(row[0], 0x10ffff)] = Array(text.utf16)
      }
      return result
    }
    upper = try mapping("upper")
    lower = try mapping("lower")
    var unicode = [Int: Int]()
    var legacy = [Int: Int]()
    guard let fold = raw["fold"] as? [[Any]] else {
      throw regexV3Error("Missing regex simple folds")
    }
    for row in fold {
      guard row.count == 2, let peers = row[1] as? [Any] else {
        throw regexV3Error("Malformed regex simple folds")
      }
      let cp = try regexV3Integer(row[0], 0x10ffff)
      let points = try peers.map { try regexV3Integer($0, 0x10ffff) } + [cp]
      let least = points.min()!
      for point in points { unicode[point] = least }
    }
    for (cp, text) in upper where cp <= 65535 && text.count == 1 {
      let value = Int(text[0])
      if cp >= 128 && value < 128 { continue }
      legacy[cp] = value
    }
    self.unicode = unicode
    self.legacy = legacy
    var ranges = [String: [[Int]]]()
    for name in ["letter", "cased", "ignorable", "space", "upper_letter", "lower_letter"] {
      guard let rows = raw[name] as? [[Any]] else {
        throw regexV3Error("Missing regex case ranges")
      }
      ranges[name] = try rows.map { row in
        guard row.count == 2 else { throw regexV3Error("Malformed regex case ranges") }
        return [try regexV3Integer(row[0], 0x10ffff), try regexV3Integer(row[1], 0x10ffff)]
      }
    }
    self.ranges = ranges
  }
  func canonical(_ point: Int, _ full: Bool) -> Int {
    (full ? unicode[point] : legacy[point]) ?? point
  }
}
private struct RegexV3Repeat {
  let count: Int
  let at: Int
}
private struct RegexV3State {
  var pc: Int
  var at: Int
  var captures: [Range<Int>?]
  var marks: [Int?]
  var repeats: [Int: RegexV3Repeat]
}
struct RegexV3Found {
  let from: Int
  let to: Int
  let captures: [Range<Int>?]
}
struct RegexV3Machine {
  let text: [Int]
  let offsets: [Int]
  let unicode: Bool
  let policy: RegexV3CaseData
  var work: Int
  private var retained = 0
  init(_ input: String, _ program: RegexV3Program, _ policy: RegexV3CaseData, work: Int = 0) throws
  {
    guard input.utf16.prefix(65537).count <= 65536, input.utf8.prefix(65537).count <= 65536 else {
      throw regexV3Error("Regex input byte budget exceeded")
    }
    unicode = program.flags.contains("u") || program.flags.contains("v")
    text = unicode ? input.unicodeScalars.map { Int($0.value) } : input.utf16.map { Int($0) }
    var offsets = [0]
    for point in text { offsets.append(offsets.last! + (unicode && point > 65535 ? 2 : 1)) }
    self.offsets = offsets
    self.policy = policy
    self.work = work
  }
  mutating func tick(_ amount: Int = 1) throws {
    guard amount >= 0, work >= 0, work <= 100000 - amount else {
      throw regexV3Error("Regex work budget exceeded")
    }
    work += amount
  }
  private mutating func copied(_ state: RegexV3State) throws -> RegexV3State {
    try tick(state.captures.count + state.marks.count + state.repeats.count)
    return state
  }
  private mutating func push(_ state: RegexV3State, _ stack: inout [RegexV3State]) throws {
    guard retained < 1024 else { throw regexV3Error("Regex backtrack memory budget exceeded") }
    retained += 1
    stack.append(state)
  }
  private mutating func clear(_ state: inout RegexV3State, _ indices: [Int]) throws {
    for index in indices {
      try tick()
      state.captures[index] = nil
      state.marks[index] = nil
    }
  }
  private func advance(_ at: Int, _ count: Int, _ direction: Int) -> Int? {
    let end = at + direction * count
    return end >= 0 && end <= text.count ? end : nil
  }
  private mutating func member(_ ranges: [[Int]], _ point: Int) throws -> Bool {
    var low = 0
    var high = ranges.count
    while low < high {
      try tick()
      let mid = (low + high) / 2
      if ranges[mid][0] <= point { low = mid + 1 } else { high = mid }
    }
    return low > 0 && point <= ranges[low - 1][1]
  }
  private func word(_ point: Int?, _ insensitive: Bool) -> Bool {
    guard var point else { return false }
    if insensitive { point = policy.canonical(point, true) }
    return (65...90).contains(point) || (97...122).contains(point) || (48...57).contains(point)
      || point == 95
  }
  private func newline(_ point: Int?) -> Bool {
    point.map { [10, 13, 0x2028, 0x2029].contains($0) } ?? false
  }
  private func point(_ at: Int) -> Int? { text.indices.contains(at) ? text[at] : nil }
  private mutating func run(
    _ program: RegexV3Program, _ which: Int, _ start: Int, _ captures: [Range<Int>?],
    _ marks: [Int?], _ depth: Int
  ) throws -> RegexV3State? {
    guard depth <= 32 else { throw regexV3Error("Regex assertion nesting budget exceeded") }
    try tick(captures.count + marks.count)
    let code = program.codes[which]
    let direction = code.direction
    var state = RegexV3State(pc: 0, at: start, captures: captures, marks: marks, repeats: [:])
    var stack = [RegexV3State]()
    defer { retained -= stack.count }
    while true {
      try tick()
      guard code.ops.indices.contains(state.pc) else {
        throw regexV3Error("Regex instruction outside program")
      }
      let op = code.ops[state.pc]
      let kind = op[0] as! String
      var failed = false
      switch kind {
      case "accept": return state
      case "jump":
        state.pc = op[1] as! Int
        continue
      case "split":
        var alternative = try copied(state)
        alternative.pc = op[2] as! Int
        try push(alternative, &stack)
        state.pc = op[1] as! Int
        continue
      case "mark":
        state = try copied(state)
        let index = op[1] as! Int
        state.marks[index] = state.at
        state.captures[index] = nil
      case "capture":
        let index = op[1] as! Int
        guard let mark = state.marks[index] else {
          throw regexV3Error("Regex capture has no start")
        }
        state = try copied(state)
        state.captures[index] = min(mark, state.at)..<max(mark, state.at)
      case "assert":
        let before = point(state.at - 1)
        let next = point(state.at)
        let ignore = op[3] as! Bool
        let boundary = word(before, ignore) != word(next, ignore)
        let name = op[1] as! String
        let multiline = op[2] as! Bool
        let yes =
          name == "start"
          ? state.at == 0 || multiline && newline(before)
          : name == "end"
            ? state.at == text.count || multiline && newline(next)
            : name == "word" ? boundary : !boundary
        failed = !yes
      case "ref":
        let indices = op[1] as! [Int]
        let captured = indices.compactMap { state.captures[$0] }
        guard captured.count <= 1 else { throw regexV3Error("Ambiguous named regex reference") }
        if let range = captured.first {
          if let end = advance(state.at, range.count, direction) {
            let begin = min(state.at, end)
            for offset in 0..<range.count {
              try tick()
              let expected = text[range.lowerBound + offset]
              let actual = text[begin + offset]
              let ignore = op[2] as! Bool
              let full = op[3] as! Bool
              if ignore
                ? policy.canonical(expected, full) != policy.canonical(actual, full)
                : expected != actual
              {
                failed = true
                break
              }
            }
            if !failed { state.at = end }
          } else {
            failed = true
          }
        }
      case "set":
        let strings = op[2] as! [[Int]]
        let ignore = op[3] as! Bool
        let full = op[4] as! Bool
        var candidates = [Int]()
        for value in strings {
          try tick()
          guard let end = advance(state.at, value.count, direction) else { continue }
          let begin = min(state.at, end)
          var yes = true
          for (index, expected) in value.enumerated() {
            try tick()
            let actual = text[begin + index]
            if (ignore ? policy.canonical(actual, full) : actual) != expected {
              yes = false
              break
            }
          }
          if yes { candidates.append(value.count) }
        }
        if let next = point(direction == 1 ? state.at : state.at - 1),
          try member(op[1] as! [[Int]], next)
        {
          candidates.append(1)
        }
        candidates.sort(by: >)
        if candidates.isEmpty {
          failed = true
        } else {
          for count in candidates.dropFirst().reversed() {
            var alternative = try copied(state)
            alternative.at = advance(state.at, count, direction)!
            alternative.pc += 1
            try push(alternative, &stack)
          }
          state.at = advance(state.at, candidates[0], direction)!
        }
      case "look":
        state = try copied(state)
        try clear(&state, op[3] as! [Int])
        let result = try run(
          program, op[1] as! Int, state.at, state.captures, state.marks, depth + 1)
        if op[2] as! Bool {
          if let result {
            state.captures = result.captures
            state.marks = result.marks
          } else {
            failed = true
          }
        } else if result != nil {
          failed = true
        }
      case "repeat":
        let id = op[1] as! Int
        let min = op[2] as! Int
        let max = op[3] as? Int
        let lazy = op[4] as! Bool
        let frame = state.repeats[id] ?? RegexV3Repeat(count: 0, at: state.at)
        let canExit = frame.count >= min
        let canRepeat = max.map { frame.count < $0 } ?? true
        var exit: RegexV3State?
        var more: RegexV3State?
        if canExit {
          exit = try copied(state)
          exit!.repeats.removeValue(forKey: id)
          exit!.pc = op[6] as! Int
        }
        if canRepeat {
          more = try copied(state)
          more!.repeats[id] = RegexV3Repeat(count: frame.count, at: state.at)
          try clear(&more!, op[7] as! [Int])
          more!.pc = op[5] as! Int
        }
        if let exit, let more {
          if lazy {
            try push(more, &stack)
            state = exit
          } else {
            try push(exit, &stack)
            state = more
          }
        } else if let exit {
          state = exit
        } else if let more {
          state = more
        } else {
          failed = true
        }
        if !failed { continue }
      case "repeat_end":
        let id = op[1] as! Int
        let start = op[2] as! Int
        guard let entry = state.repeats[id] else {
          throw regexV3Error("Regex repeat frame missing")
        }
        if entry.at == state.at && entry.count >= code.ops[start][2] as! Int {
          failed = true
        } else {
          state = try copied(state)
          state.repeats[id] = RegexV3Repeat(count: entry.count + 1, at: entry.at)
          state.pc = start
          continue
        }
      default: throw regexV3Error("Unknown regex instruction")
      }
      if failed {
        guard let previous = stack.popLast() else { return nil }
        retained -= 1
        state = previous
      } else {
        state.pc += 1
      }
    }
  }
  mutating func find(_ program: RegexV3Program) throws -> [RegexV3Found] {
    var result = [RegexV3Found]()
    var search = 0
    while search <= text.count {
      var found: RegexV3Found?
      for start in search...text.count {
        if var value = try run(
          program, 0, start, Array(repeating: nil, count: program.groups + 1),
          Array(repeating: nil, count: program.groups + 1), 0)
        {
          value.captures[0] = start..<value.at
          found = RegexV3Found(from: start, to: value.at, captures: value.captures)
          break
        }
        if program.flags.contains("y") { break }
      }
      guard let value = found else { break }
      search = value.from == value.to ? value.to + 1 : value.to
      result.append(value)
      if !program.flags.contains("g") { break }
    }
    return result
  }
  func units(_ range: Range<Int>) -> [UInt16] {
    var result = [UInt16]()
    for point in text[range] {
      if unicode && point > 65535 {
        let cp = point - 65536
        result.append(UInt16(0xd800 + (cp >> 10)))
        result.append(UInt16(0xdc00 + (cp & 1023)))
      } else {
        result.append(UInt16(point))
      }
    }
    return result
  }
}

private func regexV3Charge(_ work: inout Int, _ count: Int = 1) throws {
  guard count >= 0, work >= 0, work <= 100000 - count else {
    throw regexV3Error("Regex format work budget exceeded")
  }
  work += count
}
private func regexV3Points(_ value: [UInt16]) -> [Int] {
  var result = [Int]()
  var at = 0
  while at < value.count {
    let unit = Int(value[at])
    at += 1
    if (0xd800...0xdbff).contains(unit), at < value.count,
      (0xdc00...0xdfff).contains(Int(value[at]))
    {
      result.append(0x10000 + ((unit - 0xd800) << 10) + Int(value[at]) - 0xdc00)
      at += 1
    } else {
      result.append(unit)
    }
  }
  return result
}
private func regexV3Encoded(_ values: ArraySlice<Int>) -> [UInt16] {
  var result = [UInt16]()
  for point in values {
    if point > 65535 {
      let cp = point - 65536
      result.append(UInt16(0xd800 + (cp >> 10)))
      result.append(UInt16(0xdc00 + (cp & 1023)))
    } else {
      result.append(UInt16(point))
    }
  }
  return result
}
extension RegexV3CaseData {
  private func has(_ kind: String, _ point: Int?, _ work: inout Int) throws -> Bool {
    guard let point else { return false }
    let values = ranges[kind]!
    var low = 0
    var high = values.count
    while low < high {
      try regexV3Charge(&work)
      let mid = (low + high) / 2
      if values[mid][0] <= point { low = mid + 1 } else { high = mid }
    }
    return low > 0 && point <= values[low - 1][1]
  }
  private func convert(_ value: [UInt16], _ uppercase: Bool, _ work: inout Int) throws -> [UInt16] {
    let chars = regexV3Points(value)
    let map = uppercase ? upper : lower
    var output = [UInt16]()
    for (index, point) in chars.enumerated() {
      try regexV3Charge(&work)
      if !uppercase, point == 0x3a3 {
        var before = index - 1
        var after = index + 1
        while before >= 0, try has("ignorable", chars[before], &work) {
          try regexV3Charge(&work)
          before -= 1
        }
        while after < chars.count, try has("ignorable", chars[after], &work) {
          try regexV3Charge(&work)
          after += 1
        }
        if before >= 0, try has("cased", chars[before], &work),
          try (after == chars.count || !has("cased", chars[after], &work))
        {
          output.append(0x3c2)
          continue
        }
      }
      output += map[point] ?? regexV3Encoded([point][...])
    }
    return output
  }
  private func digit(_ point: Int?) -> Bool { point.map { (48...57).contains($0) } ?? false }
  private func white(_ point: Int?, _ work: inout Int) throws -> Bool {
    if point.map({ [9, 10, 11, 12, 13, 0xfeff, 0x2028, 0x2029].contains($0) }) ?? false {
      return true
    }
    return try has("space", point, &work)
  }
  private func delimiter(_ point: Int?, _ work: inout Int) throws -> Bool {
    if point == 45 || point == 95 { return true }
    return try white(point, &work)
  }
  private func words(_ value: [UInt16], _ work: inout Int) throws -> [[UInt16]] {
    var result = [[UInt16]]()
    var current = [UInt16]()
    for point in regexV3Points(value) {
      try regexV3Charge(&work)
      if try has("letter", point, &work) || digit(point) {
        current += regexV3Encoded([point][...])
      } else if !current.isEmpty {
        result.append(current)
        current = []
      }
    }
    if !current.isEmpty { result.append(current) }
    return result
  }
  private func joined(_ values: [[UInt16]], _ separator: UInt16) -> [UInt16] {
    var result = [UInt16]()
    for (index, value) in values.enumerated() {
      if index > 0 { result.append(separator) }
      result += value
    }
    return result
  }
  private func kebab(_ value: [UInt16], _ work: inout Int) throws -> [UInt16] {
    let chars = regexV3Points(value)
    var any = false
    for point in chars {
      try regexV3Charge(&work)
      if try has("letter", point, &work) || digit(point) {
        any = true
        break
      }
    }
    if !any { return value }
    var first = 0
    var last = chars.count
    while first < last, try white(chars[first], &work) { first += 1 }
    while last > first, try white(chars[last - 1], &work) { last -= 1 }
    while first < last, chars[first] == 95 { first += 1 }
    while last > first, chars[last - 1] == 95 { last -= 1 }
    let cleaned = Array(chars[first..<last])
    func point(_ at: Int) -> Int? { cleaned.indices.contains(at) ? cleaned[at] : nil }
    var parts = [[UInt16]]()
    var at = 0
    while at < cleaned.count {
      try regexV3Charge(&work)
      var end = at
      var run = at
      while run < cleaned.count, try has("upper_letter", cleaned[run], &work) {
        try regexV3Charge(&work)
        run += 1
      }
      if run >= at + 2 {
        for stop in stride(from: run, through: at + 2, by: -1) {
          try regexV3Charge(&work)
          if try
            (stop == cleaned.count || delimiter(point(stop), &work)
            || has("upper_letter", point(stop), &work)
              && has("lower_letter", point(stop + 1), &work))
          {
            end = stop
            break
          }
        }
      }
      if end == at {
        var stop = at
        if try has("upper_letter", point(stop), &work) { stop += 1 }
        let begin = stop
        while stop < cleaned.count, try has("lower_letter", cleaned[stop], &work) {
          try regexV3Charge(&work)
          stop += 1
        }
        if stop > begin {
          while digit(point(stop)) {
            try regexV3Charge(&work)
            stop += 1
          }
          end = stop
        }
      }
      if end == at, try has("upper_letter", point(at), &work),
        try has("upper_letter", point(at + 1), &work), try has("lower_letter", point(at + 2), &work)
      {
        end = at + 1
      }
      if end == at, try has("upper_letter", point(at), &work),
        try (at + 1 == cleaned.count || delimiter(point(at + 1), &work))
      {
        end = at + 1
      }
      if end == at, digit(point(at)) {
        end = at + 1
        while digit(point(end)) {
          try regexV3Charge(&work)
          end += 1
        }
      }
      if end > at {
        parts.append(try convert(regexV3Encoded(cleaned[at..<end]), false, &work))
        at = end
      } else {
        at += 1
      }
    }
    if !parts.isEmpty { return joined(parts, 45) }
    var fallback = [[UInt16]]()
    var current = [Int]()
    for point in cleaned {
      try regexV3Charge(&work)
      if try delimiter(point, &work) {
        if !current.isEmpty {
          fallback.append(try convert(regexV3Encoded(current[...]), false, &work))
          current = []
        }
      } else {
        current.append(point)
      }
    }
    if !current.isEmpty { fallback.append(try convert(regexV3Encoded(current[...]), false, &work)) }
    return joined(fallback, 45)
  }
  private func snake(_ value: [UInt16], _ work: inout Int) throws -> [UInt16] {
    let chars = regexV3Points(value)
    var result = [UInt16]()
    var separator = false
    for (index, point) in chars.enumerated() {
      try regexV3Charge(&work)
      if try (point == 45 || white(point, &work)) {
        if !separator { result.append(95) }
        separator = true
        continue
      }
      separator = false
      result += regexV3Encoded([point][...])
      if try has("lower_letter", point, &work),
        try has("upper_letter", index + 1 < chars.count ? chars[index + 1] : nil, &work)
      {
        result.append(95)
      }
    }
    return try convert(result, false, &work)
  }
  func mode(_ value: [UInt16], _ mode: String, _ work: inout Int) throws -> [UInt16] {
    switch mode {
    case "upcase": return try convert(value, true, &work)
    case "downcase": return try convert(value, false, &work)
    case "capitalize": return try convert(Array(value.prefix(1)), true, &work) + value.dropFirst()
    case "camelcase", "pascalcase":
      let found = try words(value, &work)
      if found.isEmpty { return value }
      var output = [UInt16]()
      for (index, word) in found.enumerated() {
        output +=
          try convert(Array(word.prefix(1)), index > 0 || mode == "pascalcase", &work)
          + word.dropFirst()
      }
      return output
    case "kebabcase": return try kebab(value, &work)
    case "snakecase": return try snake(value, &work)
    default: return value
    }
  }
}
private struct RegexV3Writer {
  var units = [UInt16]()
  var bytes = 0
  var pending = false
  mutating func append(_ value: [UInt16]) throws {
    for unit in value {
      if pending {
        guard (0xdc00...0xdfff).contains(unit) else {
          throw regexV3Error("Regex result is not valid source Unicode")
        }
        pending = false
        bytes += 4
      } else if (0xd800...0xdbff).contains(unit) {
        pending = true
      } else if (0xdc00...0xdfff).contains(unit) {
        throw regexV3Error("Regex result is not valid source Unicode")
      } else {
        bytes += unit <= 127 ? 1 : unit <= 2047 ? 2 : 3
      }
      guard bytes <= 1_048_576 else { throw regexV3Error("Regex output byte budget exceeded") }
      units.append(unit)
    }
  }
  func finish() throws -> String {
    guard !pending else { throw regexV3Error("Regex result is not valid source Unicode") }
    return String(decoding: units, as: UTF16.self)
  }
}
private func regexV3Emit(
  _ ops: [Any], _ captures: [[UInt16]?], _ policy: RegexV3CaseData, _ writer: inout RegexV3Writer,
  _ work: inout Int, _ depth: Int = 0
) throws {
  guard depth <= 32 else { throw regexV3Error("Regex format depth budget exceeded") }
  for raw in ops {
    try regexV3Charge(&work)
    if let literal = raw as? String {
      try writer.append(Array(literal.utf16))
      continue
    }
    let op = raw as! [Any]
    let index = op[1] as! Int
    let value = index < captures.count ? captures[index] ?? [] : []
    if op[0] as! String == "if" {
      try regexV3Emit(
        op[value.isEmpty ? 3 : 2] as! [Any], captures, policy, &writer, &work, depth + 1)
    } else {
      try writer.append(policy.mode(value, op[2] as! String, &work))
    }
  }
}
func applyRegexV3(
  _ program: RegexV3Program, _ input: String, _ policy: RegexV3CaseData, work: inout Int
) throws -> String {
  var machine = try RegexV3Machine(input, program, policy, work: work)
  var output: [(Int, Int, [[UInt16]?])] = []
  do {
    output = try machine.find(program).map { value in
      (
        machine.offsets[value.from], machine.offsets[value.to],
        value.captures.map { range in range.map { machine.units($0) } }
      )
    }
  } catch {
    work = machine.work
    throw error
  }
  work = machine.work
  var writer = RegexV3Writer()
  if output.isEmpty,
    program.format.contains(where: { raw in
      guard let op = raw as? [Any], op[0] as? String == "if", let values = op[3] as? [Any] else {
        return false
      }
      return !values.isEmpty
    })
  {
    try regexV3Emit(program.format, [], policy, &writer, &work)
    return try writer.finish()
  }
  let source = Array(input.utf16)
  var cursor = 0
  for (from, to, captures) in output {
    try writer.append(Array(source[cursor..<from]))
    try regexV3Emit(program.format, captures, policy, &writer, &work)
    cursor = to
  }
  try writer.append(Array(source[cursor...]))
  return try writer.finish()
}

private func regexV3MetadataBytes(_ raw: Any) throws -> Int {
  var total = 0
  func add(_ amount: Int) throws {
    guard amount >= 0, total <= 1_048_576 - amount else {
      throw regexV3Error("Regex metadata byte budget exceeded")
    }
    total += amount
  }
  func string(_ text: String) throws {
    try add(2)
    for scalar in text.unicodeScalars {
      let point = scalar.value
      try add(
        [34, 92, 8, 9, 10, 12, 13].contains(point)
          ? 2 : point < 32 ? 6 : point <= 127 ? 1 : point <= 2047 ? 2 : point <= 65535 ? 3 : 4)
    }
  }
  func visit(_ value: Any, _ depth: Int) throws {
    guard depth <= 128 else { throw regexV3Error("Regex metadata nesting exceeded") }
    if value is NSNull {
      try add(4)
    } else if let text = value as? String {
      try string(text)
    } else if let number = value as? NSNumber {
      if CFGetTypeID(number) == CFBooleanGetTypeID() {
        try add(number.boolValue ? 4 : 5)
      } else {
        let n = number.doubleValue
        guard n.isFinite, n.rounded(.towardZero) == n, abs(n) <= 9_007_199_254_740_991 else {
          throw regexV3Error("Malformed regex metadata integer")
        }
        try add(String(Int64(n)).utf8.count)
      }
    } else if let values = value as? [Any] {
      try add(2 + max(0, values.count - 1))
      for value in values { try visit(value, depth + 1) }
    } else if let values = value as? [String: Any] {
      try add(2 + max(0, values.count - 1))
      for (key, value) in values {
        try string(key)
        try add(1)
        try visit(value, depth + 1)
      }
    } else {
      throw regexV3Error("Malformed regex metadata value")
    }
  }
  try visit(raw, 0)
  return total
}
</ui:copy>
<ui:file path="apps/apple-simple/Sources/SnippetTransformV3.swift"><ui:paste select="#snippet-regex-v3-apple"/>
private enum RegexV3FrozenCase {
 static let policy:RegexV3CaseData = try! RegexV3CaseData(JSONSerialization.jsonObject(with: #"""
<ui:paste select="#snippet-regex-case17-part-0"/><ui:paste select="#snippet-regex-case17-part-1"/><ui:paste select="#snippet-regex-case17-part-2"/><ui:paste select="#snippet-regex-case17-part-3"/><ui:paste select="#snippet-regex-case17-part-4"/><ui:paste select="#snippet-regex-case17-part-5"/><ui:paste select="#snippet-regex-case17-part-6"/><ui:paste select="#snippet-regex-case17-part-7"/>
"""#.data(using:.utf8)!) as! [String:Any])
}
func applyRegexV3(_ program:RegexV3Program,_ input:String,work:inout Int)throws->String {try applyRegexV3(program,input,RegexV3FrozenCase.policy,work:&work)}
</ui:file>

## Apply the same admission and budgets on Android

Android evaluates immutable normalized instructions instead of delegating pattern text to Java regex. A UTF-16 input scan rejects lone surrogates before encoding; the early length cap also bounds that validation work. Work and state failures propagate before the controller can submit the surrounding source action.
<ui:copy id="snippet-regex-v3-android">
import org.json.*
private fun bad(message:String):Nothing=throw IllegalArgumentException(message)
private fun integer(raw:Any?,max:Long=9007199254740991L):Long {if(raw !is Number)bad("Malformed regex integer");val n=raw.toDouble();if(!n.isFinite()||n<0||n>max||n!=kotlin.math.floor(n))bad("Malformed regex integer");return n.toLong()}
private fun number(raw:Any?,max:Int)=integer(raw,max.toLong()).toInt()
private fun truth(raw:Any?):Boolean=raw as? Boolean?:bad("Malformed regex boolean")
private fun array(raw:Any?,maximum:Int=65536):List<Any?> {val a=raw as? JSONArray?:bad("Malformed regex array");if(a.length()>maximum)bad("Regex array budget exceeded");return (0 until a.length()).map{a.opt(it).takeUnless{v->v===JSONObject.NULL}}}
private fun keys(raw:JSONObject):Set<String>{if(raw.length()>8)bad("Regex object field budget exceeded");val names=raw.keys().asSequence().toList();if(names.any{it.length>32})bad("Regex object field budget exceeded");return names.toSet()}
data class RegexV3Code(val direction:Int,val ops:List<List<Any?>>)
data class RegexV3Program(val groups:Int,val flags:String,val format:List<Any?>,val codes:List<RegexV3Code>,val bytes:Int)
private class Metadata {var nodes=0;var instructions=0;fun tick(n:Int){if(n<0||nodes>262144-n)bad("Regex metadata budget exceeded");nodes+=n}}
private fun indices(raw:Any?,groups:Int):List<Int>{val list=array(raw,groups);if(list.size>groups)bad("Malformed regex captures");val result=list.map{number(it,groups)};if(result.any{it==0}||result.distinct().size!=result.size)bad("Malformed regex captures");return result}
private fun scalar(source:String):Boolean{var at=0;while(at<source.length){val c=source[at++];if(c.isHighSurrogate()){if(at>=source.length||!source[at++].isLowSurrogate())return false}else if(c.isLowSurrogate())return false};return true}
private fun format(raw:Any?,groups:Int,m:Metadata,depth:Int=0):List<Any?> {val values=array(raw,512);if(depth>32||values.size>512)bad("Regex format budget exceeded");m.tick(values.size);return values.map{item->if(item is String){if(item.length>16384||!scalar(item)||item.toByteArray(Charsets.UTF_8).size>16384)bad("Regex format text budget exceeded");m.tick(item.toByteArray(Charsets.UTF_8).size);item}else{val op=array(item,4);val kind=op.firstOrNull() as? String?:bad("Malformed regex format");if(kind.length>8)bad("Malformed regex format");when(kind){ "capture"->{if(op.size!=3||op[2] !in listOf("","upcase","downcase","capitalize","camelcase","pascalcase","kebabcase","snakecase"))bad("Malformed regex format");listOf("capture",number(op[1],groups+1),op[2])};"if"->{if(op.size!=4)bad("Malformed regex format");listOf("if",number(op[1],groups+1),format(op[2],groups,m,depth+1),format(op[3],groups,m,depth+1))};else->bad("Unknown regex format")}}}}
private fun code(raw:Any?,direction:Int,groups:Int,subs:Int,unicode:Boolean,m:Metadata):RegexV3Code {val rows=array(raw,4096).map{array(it,8)};if(rows.isEmpty()||rows.size>4096||m.instructions>4096-rows.size)bad("Regex instruction budget exceeded");m.instructions+=rows.size;m.tick(rows.size)
 fun pc(v:Any?)=number(v,rows.size-1)
 fun group(v:Any?):Int=number(v,groups).also{if(it==0)bad("Regex group zero write")}
 val ops=rows.map{op->m.tick(op.size);val kind=op.firstOrNull() as? String?:bad("Malformed regex instruction");if(kind.length>12)bad("Malformed regex instruction");fun size(n:Int){if(op.size!=n)bad("Malformed regex $kind")};when(kind){
 "accept"->{size(1);listOf(kind)};"jump"->{size(2);listOf(kind,pc(op[1]))};"split"->{size(3);listOf(kind,pc(op[1]),pc(op[2]))};"mark","capture"->{size(2);listOf(kind,group(op[1]))}
 "assert"->{size(4);if(op[1] !in listOf("start","end","word","nonword"))bad("Malformed regex assertion");val ignore=truth(op[3]);if(ignore&&!unicode)bad("Regex assertion mode mismatch");listOf(kind,op[1],truth(op[2]),ignore)}
 "ref"->{size(4);if(truth(op[3])!=unicode)bad("Regex reference mode mismatch");listOf(kind,indices(op[1],groups),truth(op[2]),unicode)}
 "look"->{size(4);if(subs==0)bad("Missing regex assertion");listOf(kind,number(op[1],subs-1)+1,truth(op[2]),indices(op[3],groups))}
 "repeat"->{size(8);val min=integer(op[2]);val max=op[3]?.let{integer(it)};if(max!=null&&max<min)bad("Backwards regex repeat");listOf(kind,number(op[1],4095),min,max,truth(op[4]),pc(op[5]),pc(op[6]),indices(op[7],groups))}
 "repeat_end"->{size(3);listOf(kind,number(op[1],4095),pc(op[2]))}
 "set"->{size(5);if(truth(op[4])!=unicode)bad("Regex set mode mismatch");val ranges=array(op[1]);val strings=array(op[2]);if(ranges.size>65536||strings.size>65536)bad("Regex set budget");m.tick(ranges.size+strings.size);val max=if(unicode)0x10ffff else 65535;var last=-2;val normalized=ranges.map{pair->val p=array(pair,2);if(p.size!=2)bad("Malformed regex range");val a=number(p[0],max);val b=number(p[1],max);if(a>b||last+1>=a)bad("Unnormalized regex ranges");last=b;intArrayOf(a,b)};var prior:IntArray?=null;val words=strings.map{word->val a=array(word,8192);if(a.size>8192||a.size==1)bad("Regex set string budget");m.tick(a.size);val v=a.map{number(it,max)}.toIntArray();prior?.let{p->if(p.size<v.size||p.size==v.size&&compare(p,v)>=0)bad("Unnormalized regex strings")};prior=v;v};listOf(kind,normalized,words,truth(op[3]),unicode)}
 else->bad("Unknown regex instruction")}}
 for(op in ops)if(op[0]=="repeat_end"){val start=ops[op[2] as Int];if(start[0]!="repeat"||start[1]!=op[1])bad("Regex repeat edge mismatch")};return RegexV3Code(direction,ops)
}
private fun compare(a:IntArray,b:IntArray):Int {for(i in 0 until minOf(a.size,b.size))if(a[i]!=b[i])return a[i].compareTo(b[i]);return a.size.compareTo(b.size)}
fun readRegexV3(raw:Any?):RegexV3Program {val p=raw as? JSONObject?:bad("Malformed regex program");if(keys(p)!=setOf("version","case_map","unicode_map","groups","flags","code","subprograms","format")||number(p.opt("version"),3)!=3||p.opt("case_map")!="hickory-ucd-17.0.0-v1"||p.opt("unicode_map")!="hickory-regex-ucd-17.0.0-v1")bad("Unsupported regex program");val groups=number(p.opt("groups"),32);val flags=p.opt("flags") as? String?:bad("Malformed regex flags");if(flags.length>8||flags.any{it !in "dgimsuvy"}||flags.toSet().size!=flags.length||'u' in flags&&'v' in flags)bad("Malformed regex flags");val subs=array(p.opt("subprograms"),128);if(subs.size>128)bad("Regex assertion budget");val m=Metadata();val unicode='u' in flags||'v' in flags;val codes=listOf(code(p.opt("code"),1,groups,subs.size,unicode,m))+subs.map{value->val sub=value as? JSONObject?:bad("Malformed regex subprogram");if(keys(sub)!=setOf("direction","code"))bad("Malformed regex subprogram");val d=sub.opt("direction") as? Number?:bad("Malformed regex direction");if(d.toDouble()!=1.0&&d.toDouble()!=-1.0)bad("Malformed regex direction");code(sub.opt("code"),d.toInt(),groups,subs.size,unicode,m)};val normalizedFormat=format(p.opt("format"),groups,m);val bytes=metadataBytes(p);return RegexV3Program(groups,flags,normalizedFormat,codes,bytes)}
class RegexV3CaseData(raw:JSONObject){val unicode=HashMap<Int,Int>();val legacy=HashMap<Int,Int>();val upper=HashMap<Int,String>();val lower=HashMap<Int,String>();val ranges=HashMap<String,List<IntArray>>()
 init{for(item in array(raw.get("fold"))){val pair=array(item);val cp=number(pair[0],0x10ffff);unicode[cp]=minOf(cp,array(pair[1]).minOf{number(it,0x10ffff)})};for((name,map)in listOf("upper" to upper,"lower" to lower))for(item in array(raw.get(name))){val pair=array(item);map[number(pair[0],0x10ffff)]=pair[1] as String};for((cp,text)in upper)if(cp<=65535&&text.length==1&&!(cp>=128&&text[0].code<128))legacy[cp]=text[0].code;for(name in listOf("letter","cased","ignorable","upper_letter","lower_letter","space"))ranges[name]=array(raw.get(name)).map{array(it).map{v->number(v,0x10ffff)}.toIntArray()}}
 fun canonical(cp:Int,full:Boolean)=(if(full)unicode else legacy)[cp]?:cp
}
data class RegexV3Match(val from:Int,val to:Int,val captures:List<Pair<Int,Int>?>)
private data class RepeatFrame(val count:Long,val at:Int)
private data class RegexState(var pc:Int,var at:Int,var captures:MutableList<Pair<Int,Int>?>,var marks:MutableList<Int?>,val repeat:MutableMap<Int,RepeatFrame>)
class RegexV3Machine(input:String,val program:RegexV3Program,val policy:RegexV3CaseData,var work:Int=0){val unicode='u' in program.flags||'v' in program.flags;val chars:IntArray;val offsets:IntArray;private var retained=0
 init{if(input.length>65536||input.toByteArray(Charsets.UTF_8).size>65536)bad("Regex input budget exceeded");var at=0;while(at<input.length){val c=input[at++];if(c.isHighSurrogate()){if(at>=input.length||!input[at++].isLowSurrogate())bad("Invalid source Unicode")}else if(c.isLowSurrogate())bad("Invalid source Unicode")};chars=if(unicode)input.codePoints().toArray() else input.map{it.code}.toIntArray();offsets=IntArray(chars.size+1);for(i in chars.indices)offsets[i+1]=offsets[i]+if(unicode&&chars[i]>65535)2 else 1}
 fun tick(n:Int=1){if(n<0||work<0||work>100000-n)bad("Regex work budget exceeded");work+=n}
 private fun copy(s:RegexState):RegexState{tick(s.captures.size+s.marks.size+s.repeat.size);return RegexState(s.pc,s.at,s.captures.toMutableList(),s.marks.toMutableList(),s.repeat.toMutableMap())}
 private fun clear(s:RegexState,indices:List<Int>){for(i in indices){tick();s.captures[i]=null;s.marks[i]=null}}
 private fun newline(cp:Int?)=cp==10||cp==13||cp==0x2028||cp==0x2029
 private fun word(raw:Int?,ignore:Boolean):Boolean{val cp=raw?.let{if(ignore)policy.canonical(it,true) else it}?:return false;return cp in 65..90||cp in 97..122||cp in 48..57||cp==95}
 private fun member(ranges:List<IntArray>,cp:Int):Boolean{var lo=0;var hi=ranges.size;while(lo<hi){tick();val mid=(lo+hi)/2;if(ranges[mid][0]<=cp)lo=mid+1 else hi=mid};return lo>0&&cp<=ranges[lo-1][1]}
 @Suppress("UNCHECKED_CAST")
 private fun run(code:RegexV3Code,start:Int,captures:MutableList<Pair<Int,Int>?>,marks:MutableList<Int?>,depth:Int):RegexState?{if(depth>32)bad("Regex assertion nesting exceeded");tick(captures.size+marks.size);var state=RegexState(0,start,captures.toMutableList(),marks.toMutableList(),mutableMapOf());val stack=ArrayList<RegexState>();fun push(s:RegexState){if(++retained>1024)bad("Regex backtrack memory exceeded");stack.add(s)}
 try{while(true){tick();val op=code.ops.getOrNull(state.pc)?:bad("Regex instruction outside program");var failed=false;when(op[0]){
 "accept"->return state
 "jump"->{state.pc=op[1] as Int;continue}
 "split"->{val alt=copy(state);alt.pc=op[2] as Int;push(alt);state.pc=op[1] as Int;continue}
 "mark"->{state=copy(state);val i=op[1] as Int;state.marks[i]=state.at;state.captures[i]=null}
 "capture"->{val i=op[1] as Int;val mark=state.marks[i]?:bad("Regex capture missing start");state=copy(state);state.captures[i]=minOf(mark,state.at) to maxOf(mark,state.at)}
 "assert"->{val before=chars.getOrNull(state.at-1);val next=chars.getOrNull(state.at);val boundary=word(before,op[3] as Boolean)!=word(next,op[3] as Boolean);val yes=when(op[1]){"start"->state.at==0||(op[2] as Boolean)&&newline(before);"end"->state.at==chars.size||(op[2] as Boolean)&&newline(next);"word"->boundary;else->!boundary};failed=!yes}
 "ref"->{val captured=(op[1] as List<Int>).mapNotNull{state.captures[it]};if(captured.size>1)bad("Ambiguous named regex reference");if(captured.isNotEmpty()){val(from,to)=captured[0];val length=to-from;val end=state.at+code.direction*length;val begin=minOf(state.at,end);if(begin<0||end>chars.size)failed=true else{for(i in 0 until length){tick();val expected=chars[from+i];val actual=chars.getOrNull(begin+i);if(actual==null||if(op[2] as Boolean)policy.canonical(expected,op[3] as Boolean)!=policy.canonical(actual,op[3] as Boolean) else expected!=actual){failed=true;break}};if(!failed)state.at=end}}}
 "set"->{val candidates=ArrayList<Int>();for(value in op[2] as List<IntArray>){tick();val end=state.at+code.direction*value.size;val begin=minOf(state.at,end);if(begin<0||end>chars.size)continue;var yes=true;for(i in value.indices){tick();val cp=chars[begin+i];if((if(op[3] as Boolean)policy.canonical(cp,op[4] as Boolean) else cp)!=value[i]){yes=false;break}};if(yes)candidates.add(value.size)};chars.getOrNull(if(code.direction==1)state.at else state.at-1)?.let{if(member(op[1] as List<IntArray>,it))candidates.add(1)};candidates.sortDescending();if(candidates.isEmpty())failed=true else{for(i in candidates.lastIndex downTo 1){val alt=copy(state);alt.at+=code.direction*candidates[i];alt.pc++;push(alt)};state.at+=code.direction*candidates[0]}}
 "look"->{state=copy(state);clear(state,op[3] as List<Int>);val result=run(program.codes[op[1] as Int],state.at,state.captures,state.marks,depth+1);if(op[2] as Boolean){if(result==null)failed=true else{state.captures=result.captures;state.marks=result.marks}}else if(result!=null)failed=true}
 "repeat"->{val id=op[1] as Int;val min=op[2] as Long;val max=op[3] as Long?;val frame=state.repeat[id]?:RepeatFrame(0,state.at);fun exit():RegexState{val result=copy(state);result.repeat.remove(id);result.pc=op[6] as Int;return result};fun more():RegexState{val result=copy(state);result.repeat[id]=RepeatFrame(frame.count,state.at);clear(result,op[7] as List<Int>);result.pc=op[5] as Int;return result};val canExit=frame.count>=min;val canRepeat=max==null||frame.count<max;if(canExit&&canRepeat){if(op[4] as Boolean){push(more());state=exit()}else{push(exit());state=more()}}else if(canExit)state=exit() else if(canRepeat)state=more() else failed=true;if(!failed)continue}
 "repeat_end"->{val id=op[1] as Int;val frame=state.repeat[id]?:bad("Regex repeat frame missing");val repeat=code.ops[op[2] as Int];if(frame.at==state.at&&frame.count>=repeat[2] as Long)failed=true else{state=copy(state);state.repeat[id]=RepeatFrame(frame.count+1,frame.at);state.pc=op[2] as Int;continue}}
 else->bad("Unknown regex instruction")};if(failed){if(stack.isEmpty())return null;state=stack.removeAt(stack.lastIndex);retained--}else state.pc++}}finally{retained-=stack.size}}
 fun find():List<RegexV3Match>{val result=ArrayList<RegexV3Match>();var search=0;while(search<=chars.size){var found:RegexV3Match?=null;for(at in search..chars.size){val state=run(program.codes[0],at,MutableList(program.groups+1){null},MutableList(program.groups+1){null},0);if(state!=null){state.captures[0]=at to state.at;found=RegexV3Match(at,state.at,state.captures);break};if('y' in program.flags)break};if(found==null)break;result.add(found);if('g' !in program.flags)break;search=if(found.to==found.from)found.to+1 else found.to};return result}
 fun units(range:Pair<Int,Int>):List<Int>{val result=ArrayList<Int>();for(i in range.first until range.second){val cp=chars[i];if(unicode&&cp>65535){result.add(0xd800+((cp-65536) shr 10));result.add(0xdc00+((cp-65536) and 1023))}else result.add(cp)};return result}
}
private fun points(value:List<Int>):List<Int>{val result=ArrayList<Int>();var at=0;while(at<value.size){val first=value[at++];if(first in 0xd800..0xdbff&&at<value.size&&value[at] in 0xdc00..0xdfff){result.add(0x10000+((first-0xd800) shl 10)+(value[at++]-0xdc00))}else result.add(first)};return result}
private fun encoded(value:List<Int>):List<Int>{val result=ArrayList<Int>();for(cp in value)if(cp>65535){result.add(0xd800+((cp-65536) shr 10));result.add(0xdc00+((cp-65536) and 1023))}else result.add(cp);return result}
private class RegexV3Formatting(val policy:RegexV3CaseData,var work:Int){fun tick(n:Int=1){if(n<0||work<0||work>100000-n)bad("Regex work budget exceeded");work+=n}
 private fun has(kind:String,point:Int?):Boolean{if(point==null)return false;val ranges=policy.ranges[kind]!!;var lo=0;var hi=ranges.size;while(lo<hi){tick();val mid=(lo+hi)/2;if(ranges[mid][0]<=point)lo=mid+1 else hi=mid};return lo>0&&point<=ranges[lo-1][1]}
 private fun convert(value:List<Int>,upper:Boolean):List<Int>{val chars=points(value);val map=if(upper)policy.upper else policy.lower;val output=ArrayList<Int>();for((index,point)in chars.withIndex()){tick();if(!upper&&point==0x3a3){var before=index-1;var after=index+1;while(before>=0&&has("ignorable",chars[before])){tick();before--};while(after<chars.size&&has("ignorable",chars[after])){tick();after++};if(before>=0&&has("cased",chars[before])&&(after==chars.size||!has("cased",chars[after]))){output.add(0x3c2);continue}};output.addAll(map[point]?.map{it.code}?:encoded(listOf(point)))};return output}
 private fun digit(cp:Int?)=cp!=null&&cp in 48..57
 private fun white(cp:Int?)=cp in listOf(9,10,11,12,13,0xfeff,0x2028,0x2029)||has("space",cp)
 private fun delimiter(cp:Int?)=cp==45||cp==95||white(cp)
 private fun words(value:List<Int>):List<List<Int>>{val result=ArrayList<List<Int>>();var current=ArrayList<Int>();for(cp in points(value)){tick();if(has("letter",cp)||digit(cp))current.addAll(encoded(listOf(cp)))else if(current.isNotEmpty()){result.add(current);current=ArrayList()}};if(current.isNotEmpty())result.add(current);return result}
 private fun join(values:List<List<Int>>,separator:Int):List<Int>{val result=ArrayList<Int>();for((i,value)in values.withIndex()){if(i>0)result.add(separator);result.addAll(value)};return result}
 private fun kebab(value:List<Int>):List<Int>{val chars=points(value);var any=false;for(cp in chars){tick();if(has("letter",cp)||digit(cp)){any=true;break}};if(!any)return value;var first=0;var last=chars.size;while(first<last&&white(chars[first]))first++;while(last>first&&white(chars[last-1]))last--;while(first<last&&chars[first]==95)first++;while(last>first&&chars[last-1]==95)last--;val cleaned=chars.subList(first,last);fun point(at:Int):Int? {return cleaned.getOrNull(at)}
 val parts=ArrayList<List<Int>>();var at=0
 while(at<cleaned.size){tick();var end=at;var run=at;while(run<cleaned.size&&has("upper_letter",cleaned[run])){tick();run++};if(run>=at+2)for(stop in run downTo at+2){tick();if(stop==cleaned.size||delimiter(point(stop))||has("upper_letter",point(stop))&&has("lower_letter",point(stop+1))){end=stop;break}}
 if(end==at){var stop=at;if(has("upper_letter",point(stop)))stop++;val begin=stop;while(stop<cleaned.size&&has("lower_letter",cleaned[stop])){tick();stop++};if(stop>begin){while(digit(point(stop))){tick();stop++};end=stop}}
 if(end==at&&has("upper_letter",point(at))&&has("upper_letter",point(at+1))&&has("lower_letter",point(at+2)))end=at+1
 if(end==at&&has("upper_letter",point(at))&&(at+1==cleaned.size||delimiter(point(at+1))))end=at+1
 if(end==at&&digit(point(at))){end=at+1;while(digit(point(end))){tick();end++}}
 if(end>at){parts.add(convert(encoded(cleaned.subList(at,end)),false));at=end}else at++}
 if(parts.isNotEmpty())return join(parts,45);val fallback=ArrayList<List<Int>>();var current=ArrayList<Int>();for(cp in cleaned){tick();if(delimiter(cp)){if(current.isNotEmpty()){fallback.add(convert(encoded(current),false));current=ArrayList()}}else current.add(cp)};if(current.isNotEmpty())fallback.add(convert(encoded(current),false));return join(fallback,45)
 }
 private fun snake(value:List<Int>):List<Int>{val chars=points(value);val result=ArrayList<Int>();var separator=false;for((i,cp)in chars.withIndex()){tick();if(cp==45||white(cp)){if(!separator)result.add(95);separator=true;continue};separator=false;result.addAll(encoded(listOf(cp)));if(has("lower_letter",cp)&&has("upper_letter",chars.getOrNull(i+1)))result.add(95)};return convert(result,false)}
 fun mode(value:List<Int>,mode:String):List<Int> = when(mode){"upcase"->convert(value,true);"downcase"->convert(value,false);"capitalize"->convert(value.take(1),true)+value.drop(1);"camelcase","pascalcase"->{val words=words(value);if(words.isEmpty())value else words.flatMapIndexed{i,word->convert(word.take(1),i>0||mode=="pascalcase")+word.drop(1)}};"kebabcase"->kebab(value);"snakecase"->snake(value);else->value}
}
private class RegexV3Writer {val units=ArrayList<Int>();var bytes=0;var pending=false;fun append(value:List<Int>){for(unit in value){if(pending){if(unit !in 0xdc00..0xdfff)bad("Regex result is not valid source Unicode");pending=false;bytes+=4}else if(unit in 0xd800..0xdbff)pending=true else if(unit in 0xdc00..0xdfff)bad("Regex result is not valid source Unicode")else bytes+=if(unit<=127)1 else if(unit<=2047)2 else 3;if(bytes>1048576)bad("Regex output byte budget exceeded");units.add(unit)}};fun finish():String{if(pending)bad("Regex result is not valid source Unicode");return String(CharArray(units.size){units[it].toChar()})}}
@Suppress("UNCHECKED_CAST")
private fun emit(ops:List<Any?>,captures:List<List<Int>?>,policy:RegexV3Formatting,writer:RegexV3Writer,depth:Int=0){if(depth>32)bad("Regex format depth budget exceeded");for(raw in ops){policy.tick();if(raw is String){writer.append(raw.map{it.code});continue};val op=raw as List<Any?>;val value=captures.getOrNull(op[1] as Int)?:emptyList();if(op[0]=="if")emit(op[if(value.isEmpty())3 else 2] as List<Any?>,captures,policy,writer,depth+1)else writer.append(policy.mode(value,op[2] as String))}}
fun applyRegexV3(program:RegexV3Program,input:String,policy:RegexV3CaseData,budget:IntArray=intArrayOf(0)):String{val machine=RegexV3Machine(input,program,policy,budget[0]);val found=try{machine.find()}finally{budget[0]=machine.work};val formatter=RegexV3Formatting(policy,machine.work);val writer=RegexV3Writer();try{if(found.isEmpty()&&program.format.any{it is List<*>&&it[0]=="if"&&(it[3] as List<*>).isNotEmpty()}){emit(program.format,emptyList(),formatter,writer);return writer.finish()};val source=input.map{it.code};var cursor=0;for(match in found){writer.append(source.subList(cursor,machine.offsets[match.from]));emit(program.format,match.captures.map{it?.let{range->machine.units(range)}},formatter,writer);cursor=machine.offsets[match.to]};writer.append(source.subList(cursor,source.size));return writer.finish()}finally{budget[0]=formatter.work}}

private fun metadataBytes(raw:Any?):Int {var total=0
 fun add(n:Int){if(n<0||total>1048576-n)bad("Regex metadata byte budget exceeded");total+=n}
 fun string(value:String){add(2);var at=0;while(at<value.length){val cp=value.codePointAt(at);at+=Character.charCount(cp);add(if(cp in listOf(34,92,8,9,10,12,13))2 else if(cp<32)6 else if(cp<=127)1 else if(cp<=2047)2 else if(cp<=65535)3 else 4)}}
 fun visit(value:Any?,depth:Int){if(depth>128)bad("Regex metadata nesting exceeded");when(value){null,JSONObject.NULL->add(4);is Boolean->add(if(value)4 else 5);is String->string(value);is Number->{val n=value.toDouble();if(!n.isFinite()||n!=kotlin.math.floor(n)||kotlin.math.abs(n)>9007199254740991.0)bad("Malformed regex integer");add(n.toLong().toString().length)};is JSONArray->{add(2+maxOf(0,value.length()-1));for(i in 0 until value.length())visit(value.opt(i),depth+1)};is JSONObject->{add(2+maxOf(0,value.length()-1));for(key in value.keys()){string(key);add(1);visit(value.opt(key),depth+1)}};else->bad("Malformed regex metadata value")}}
 visit(raw,0);return total
}
</ui:copy>
<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/SnippetTransformV3.kt">package com.hickorydocs.simple

<ui:paste select="#snippet-regex-v3-android"/>
private object RegexV3FrozenCase {val policy:RegexV3CaseData by lazy {RegexV3CaseData(JSONObject(arrayOf(
"""<ui:paste select="#snippet-regex-case17-part-0"/>""",
"""<ui:paste select="#snippet-regex-case17-part-1"/>""",
"""<ui:paste select="#snippet-regex-case17-part-2"/>""",
"""<ui:paste select="#snippet-regex-case17-part-3"/>""",
"""<ui:paste select="#snippet-regex-case17-part-4"/>""",
"""<ui:paste select="#snippet-regex-case17-part-5"/>""",
"""<ui:paste select="#snippet-regex-case17-part-6"/>""",
"""<ui:paste select="#snippet-regex-case17-part-7"/>"""
).joinToString("")))}}
internal fun applyRegexV3(program:RegexV3Program,input:String,budget:IntArray):String=applyRegexV3(program,input,RegexV3FrozenCase.policy,budget)
</ui:file>

## Retain independent semantic and refusal evidence

The pattern cases were executed by Node 24.22.0, and the format cases by the actual VSCode snippet parser with recorded source hashes. Host tests compare exact capture units and source offsets, including temporarily unpaired UTF-16 captures. These are model checks; provider and template editor flows are separate required acceptance. The refusal corpus covers hostile metadata and exhausted work, retained states, assertions, UTF-8 input, output and final Unicode. Source actions must remain unchanged on every refusal.
<ui:copy id="snippet-regex-v3-oracle-cases-compiled-json">
[{"case":{"flags":"g","input":"abbb ab","matches":[{"capture_units":[[97],[98,98,98]],"from":0,"indices":[[0,1],[1,4]],"name_units":null,"to":1},{"capture_units":[[97],[98]],"from":5,"indices":[[5,6],[6,7]],"name_units":null,"to":6}],"name":"positive-lookahead","pattern":"a(?=(b+))"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[97,97]],[],false,false],["look",0,true,[1]],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,1,null,false,2,4,[]],["set",[[98,98]],[],false,false],["repeat_end",0,1],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[98,98,98]],"from":0,"indices":[[0,1],[1,4]],"to":1},{"capture_units":[[97],[98]],"from":5,"indices":[[5,6],[6,7]],"to":6}],"rust_output":"abbb ab"},{"case":{"flags":"g","input":"ab ac a","matches":[{"capture_units":[[97]],"from":3,"indices":[[3,4]],"name_units":null,"to":4},{"capture_units":[[97]],"from":6,"indices":[[6,7]],"name_units":null,"to":7}],"name":"negative-lookahead","pattern":"a(?!b)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[97,97]],[],false,false],["look",0,false,[]],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[{"code":[["set",[[98,98]],[],false,false],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":3,"indices":[[3,4]],"to":4},{"capture_units":[[97]],"from":6,"indices":[[6,7]],"to":7}],"rust_output":"ab ac a"},{"case":{"flags":"","input":"baabac","matches":[{"capture_units":[[97,98,97],[97]],"from":2,"indices":[[2,5],[2,3]],"name_units":null,"to":5}],"name":"lookahead-is-atomic","pattern":"(?=(a+))a*b\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["repeat",1,0,null,false,2,4,[]],["set",[[97,97]],[],false,false],["repeat_end",1,1],["set",[[98,98]],[],false,false],["ref",[1],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,1,null,false,2,4,[]],["set",[[97,97]],[],false,false],["repeat_end",0,1],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,98,97],[97]],"from":2,"indices":[[2,5],[2,3]],"to":5}],"rust_output":"baabac"},{"case":{"flags":"","input":"abc","matches":[{"capture_units":[[],[97],[98,99]],"from":3,"indices":[[3,3],[0,1],[1,3]],"name_units":null,"to":3}],"name":"positive-lookbehind","pattern":"(?<=([ab]+)([bc]+))$"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1,2]],["assert","end",false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":2,"subprograms":[{"code":[["mark",2],["repeat",0,1,null,false,2,4,[]],["set",[[98,99]],[],false,false],["repeat_end",0,1],["capture",2],["mark",1],["repeat",1,1,null,false,7,9,[]],["set",[[97,98]],[],false,false],["repeat_end",1,6],["capture",1],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[],[97],[98,99]],"from":3,"indices":[[3,3],[0,1],[1,3]],"to":3}],"rust_output":"abc"},{"case":{"flags":"","input":"ab","matches":[],"name":"lookbehind-backreference-direction","pattern":"(?<=\\1(a))b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["set",[[98,98]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["set",[[97,97]],[],false,false],["capture",1],["ref",[1],false,false],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"ab"},{"case":{"flags":"g","input":"ab cb b","matches":[{"capture_units":[[98],null],"from":4,"indices":[[4,5],null],"name_units":null,"to":5},{"capture_units":[[98],null],"from":6,"indices":[[6,7],null],"name_units":null,"to":7}],"name":"negative-lookbehind","pattern":"(?<!(a))b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[98,98]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["set",[[97,97]],[],false,false],["capture",1],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[98],null],"from":4,"indices":[[4,5],null],"to":5},{"capture_units":[[98],null],"from":6,"indices":[[6,7],null],"to":7}],"rust_output":"ab cb b"},{"case":{"flags":"g","input":"same-same other-word","matches":[{"capture_units":[[115,97,109,101,45,115,97,109,101],[115,97,109,101]],"from":0,"indices":[[0,9],[0,4]],"name_units":{"word":[115,97,109,101]},"to":9}],"name":"named-capture-backreference","pattern":"(?<word>[a-z]+)-\\k<word>"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["repeat",0,1,null,false,2,4,[]],["set",[[97,122]],[],false,false],["repeat_end",0,1],["capture",1],["set",[[45,45]],[],false,false],["ref",[1],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[115,97,109,101,45,115,97,109,101],[115,97,109,101]],"from":0,"indices":[[0,9],[0,4]],"to":9}],"rust_output":"same-same other-word"},{"case":{"flags":"","input":"aa","matches":[{"capture_units":[[97,97],[97]],"from":0,"indices":[[0,2],[0,1]],"name_units":{"π":[97]},"to":2}],"name":"unicode-group-name","pattern":"(?<π>a)\\k<π>"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["set",[[97,97]],[],false,false],["capture",1],["ref",[1],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97]],"from":0,"indices":[[0,2],[0,1]],"to":2}],"rust_output":"aa"},{"case":{"flags":"","input":"a","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":{"later":[97]},"to":1}],"name":"named-forward-reference","pattern":"\\k<later>(?<later>a)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["ref",[1],false,false],["mark",1],["set",[[97,97]],[],false,false],["capture",1],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"a"},{"case":{"flags":"","input":"a","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"numeric-forward-reference","pattern":"\\1(a)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["ref",[1],false,false],["mark",1],["set",[[97,97]],[],false,false],["capture",1],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"a"},{"case":{"flags":"g","input":"ac bbc bc","matches":[{"capture_units":[[97,99],null],"from":0,"indices":[[0,2],null],"name_units":null,"to":2},{"capture_units":[[98,98,99],[98]],"from":3,"indices":[[3,6],[3,4]],"name_units":null,"to":6}],"name":"unmatched-backreference-is-empty","pattern":"(?:a|(b))\\1c"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["split",1,3],["set",[[97,97]],[],false,false],["jump",6],["mark",1],["set",[[98,98]],[],false,false],["capture",1],["ref",[1],false,false],["set",[[99,99]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,99],null],"from":0,"indices":[[0,2],null],"to":2},{"capture_units":[[98,98,99],[98]],"from":3,"indices":[[3,6],[3,4]],"to":6}],"rust_output":"ac bbc bc"},{"case":{"flags":"","input":"zaacbbbcac","matches":[{"capture_units":[[122,97,97,99,98,98,98,99,97,99],[122],[97,99],[97],null,[99]],"from":0,"indices":[[0,10],[0,1],[8,10],[8,9],null,[9,10]],"name_units":null,"to":10}],"name":"repeat-clears-captures","pattern":"(z)((a+)?(b+)?(c))*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["set",[[122,122]],[],false,false],["capture",1],["repeat",0,0,null,false,4,24,[2,3,4,5]],["mark",2],["repeat",1,0,1,false,6,12,[3]],["mark",3],["repeat",2,1,null,false,8,10,[]],["set",[[97,97]],[],false,false],["repeat_end",2,7],["capture",3],["repeat_end",1,5],["repeat",3,0,1,false,13,19,[4]],["mark",4],["repeat",4,1,null,false,15,17,[]],["set",[[98,98]],[],false,false],["repeat_end",4,14],["capture",4],["repeat_end",3,12],["mark",5],["set",[[99,99]],[],false,false],["capture",5],["capture",2],["repeat_end",0,3],["accept"]],"flags":"","format":[["capture",0,""]],"groups":5,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[122,97,97,99,98,98,98,99,97,99],[122],[97,99],[97],null,[99]],"from":0,"indices":[[0,10],[0,1],[8,10],[8,9],null,[9,10]],"to":10}],"rust_output":"zaacbbbcac"},{"case":{"flags":"","input":"b","matches":[{"capture_units":[[],null],"from":0,"indices":[[0,0],null],"name_units":null,"to":0}],"name":"empty-repeated-capture","pattern":"(a*)*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[97,97]],[],false,false],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[],null],"from":0,"indices":[[0,0],null],"to":0}],"rust_output":"b"},{"case":{"flags":"","input":"aba","matches":[{"capture_units":[[97,98,97],[97],null],"from":0,"indices":[[0,3],[2,3],null],"name_units":null,"to":3}],"name":"greedy-repetition-capture","pattern":"(a(b)?)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,10,[1,2]],["mark",1],["set",[[97,97]],[],false,false],["repeat",1,0,1,false,4,8,[2]],["mark",2],["set",[[98,98]],[],false,false],["capture",2],["repeat_end",1,3],["capture",1],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":2,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,98,97],[97],null],"from":0,"indices":[[0,3],[2,3],null],"to":3}],"rust_output":"aba"},{"case":{"flags":"g","input":"😀","matches":[{"capture_units":[[55357]],"from":0,"indices":[[0,1]],"name_units":null,"to":1},{"capture_units":[[56832]],"from":1,"indices":[[1,2]],"name_units":null,"to":2}],"name":"legacy-utf16-dot","pattern":"."},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,9],[11,12],[14,8231],[8234,65535]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[55357]],"from":0,"indices":[[0,1]],"to":1},{"capture_units":[[56832]],"from":1,"indices":[[1,2]],"to":2}],"rust_output":"😀"},{"case":{"flags":"gu","input":"😀","matches":[{"capture_units":[[55357,56832]],"from":0,"indices":[[0,2]],"name_units":null,"to":2}],"name":"unicode-scalar-dot","pattern":"."},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,9],[11,12],[14,8231],[8234,1114111]],[],false,true],["accept"]],"flags":"gu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[55357,56832]],"from":0,"indices":[[0,2]],"to":2}],"rust_output":"😀"},{"case":{"flags":"g","input":"😀","matches":[{"capture_units":[[55357]],"from":0,"indices":[[0,1]],"name_units":null,"to":1},{"capture_units":[[56832]],"from":1,"indices":[[1,2]],"name_units":null,"to":2}],"name":"legacy-class-supplementary","pattern":"[😀]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[55357,55357],[56832,56832]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[55357]],"from":0,"indices":[[0,1]],"to":1},{"capture_units":[[56832]],"from":1,"indices":[[1,2]],"to":2}],"rust_output":"😀"},{"case":{"flags":"gu","input":"😀","matches":[{"capture_units":[[55357,56832]],"from":0,"indices":[[0,2]],"name_units":null,"to":2}],"name":"unicode-class-supplementary","pattern":"[😀]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[128512,128512]],[],false,true],["accept"]],"flags":"gu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[55357,56832]],"from":0,"indices":[[0,2]],"to":2}],"rust_output":"😀"},{"case":{"flags":"g","input":"😀","matches":[{"capture_units":[[55357]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"legacy-surrogate-range","pattern":"[\\uD800-\\uDBFF]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[55296,56319]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[55357]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"😀"},{"case":{"flags":"u","input":"😀","matches":[{"capture_units":[[55357,56832]],"from":0,"indices":[[0,2]],"name_units":null,"to":2}],"name":"unicode-escaped-surrogate-pair","pattern":"\\uD83D\\uDE00"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[128512,128512]],[],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[55357,56832]],"from":0,"indices":[[0,2]],"to":2}],"rust_output":"😀"},{"case":{"flags":"u","input":"😀","matches":[{"capture_units":[[55357,56832]],"from":0,"indices":[[0,2]],"name_units":null,"to":2}],"name":"unicode-codepoint-escape","pattern":"\\u{1F600}"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[128512,128512]],[],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[55357,56832]],"from":0,"indices":[[0,2]],"to":2}],"rust_output":"😀"},{"case":{"flags":"","input":"aaa","matches":[{"capture_units":[[97,97,97]],"from":0,"indices":[[0,3]],"name_units":null,"to":3}],"name":"legacy-octal","pattern":"\\141+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,3,[]],["set",[[97,97]],[],false,false],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97,97]],"from":0,"indices":[[0,3]],"to":3}],"rust_output":"aaa"},{"case":{"flags":"","input":"\u0011","matches":[{"capture_units":[[17]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"legacy-control-class","pattern":"[\\c1]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[17,17]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[17]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"\u0011"},{"case":{"flags":"","input":"ab12cd","matches":[{"capture_units":[[97,98]],"from":0,"indices":[[0,2]],"name_units":null,"to":2}],"name":"class-negated-shorthand","pattern":"[a\\D]+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,3,[]],["set",[[0,47],[58,65535]],[],false,false],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,98]],"from":0,"indices":[[0,2]],"to":2}],"rust_output":"ab12cd"},{"case":{"flags":"gu","input":"λ漢字 123","matches":[{"capture_units":[[955,28450,23383]],"from":0,"indices":[[0,3]],"name_units":null,"to":3}],"name":"unicode-general-category","pattern":"\\p{General_Category=Letter}+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,3,[]],["set",[[65,90],[97,122],[170,170],[181,181],[186,186],[192,214],[216,246],[248,705],[710,721],[736,740],[748,748],[750,750],[880,884],[886,887],[890,893],[895,895],[902,902],[904,906],[908,908],[910,929],[931,1013],[1015,1153],[1162,1327],[1329,1366],[1369,1369],[1376,1416],[1488,1514],[1519,1522],[1568,1610],[1646,1647],[1649,1747],[1749,1749],[1765,1766],[1774,1775],[1786,1788],[1791,1791],[1808,1808],[1810,1839],[1869,1957],[1969,1969],[1994,2026],[2036,2037],[2042,2042],[2048,2069],[2074,2074],[2084,2084],[2088,2088],[2112,2136],[2144,2154],[2160,2183],[2185,2191],[2208,2249],[2308,2361],[2365,2365],[2384,2384],[2392,2401],[2417,2432],[2437,2444],[2447,2448],[2451,2472],[2474,2480],[2482,2482],[2486,2489],[2493,2493],[2510,2510],[2524,2525],[2527,2529],[2544,2545],[2556,2556],[2565,2570],[2575,2576],[2579,2600],[2602,2608],[2610,2611],[2613,2614],[2616,2617],[2649,2652],[2654,2654],[2674,2676],[2693,2701],[2703,2705],[2707,2728],[2730,2736],[2738,2739],[2741,2745],[2749,2749],[2768,2768],[2784,2785],[2809,2809],[2821,2828],[2831,2832],[2835,2856],[2858,2864],[2866,2867],[2869,2873],[2877,2877],[2908,2909],[2911,2913],[2929,2929],[2947,2947],[2949,2954],[2958,2960],[2962,2965],[2969,2970],[2972,2972],[2974,2975],[2979,2980],[2984,2986],[2990,3001],[3024,3024],[3077,3084],[3086,3088],[3090,3112],[3114,3129],[3133,3133],[3160,3162],[3164,3165],[3168,3169],[3200,3200],[3205,3212],[3214,3216],[3218,3240],[3242,3251],[3253,3257],[3261,3261],[3292,3294],[3296,3297],[3313,3314],[3332,3340],[3342,3344],[3346,3386],[3389,3389],[3406,3406],[3412,3414],[3423,3425],[3450,3455],[3461,3478],[3482,3505],[3507,3515],[3517,3517],[3520,3526],[3585,3632],[3634,3635],[3648,3654],[3713,3714],[3716,3716],[3718,3722],[3724,3747],[3749,3749],[3751,3760],[3762,3763],[3773,3773],[3776,3780],[3782,3782],[3804,3807],[3840,3840],[3904,3911],[3913,3948],[3976,3980],[4096,4138],[4159,4159],[4176,4181],[4186,4189],[4193,4193],[4197,4198],[4206,4208],[4213,4225],[4238,4238],[4256,4293],[4295,4295],[4301,4301],[4304,4346],[4348,4680],[4682,4685],[4688,4694],[4696,4696],[4698,4701],[4704,4744],[4746,4749],[4752,4784],[4786,4789],[4792,4798],[4800,4800],[4802,4805],[4808,4822],[4824,4880],[4882,4885],[4888,4954],[4992,5007],[5024,5109],[5112,5117],[5121,5740],[5743,5759],[5761,5786],[5792,5866],[5873,5880],[5888,5905],[5919,5937],[5952,5969],[5984,5996],[5998,6000],[6016,6067],[6103,6103],[6108,6108],[6176,6264],[6272,6276],[6279,6312],[6314,6314],[6320,6389],[6400,6430],[6480,6509],[6512,6516],[6528,6571],[6576,6601],[6656,6678],[6688,6740],[6823,6823],[6917,6963],[6981,6988],[7043,7072],[7086,7087],[7098,7141],[7168,7203],[7245,7247],[7258,7293],[7296,7306],[7312,7354],[7357,7359],[7401,7404],[7406,7411],[7413,7414],[7418,7418],[7424,7615],[7680,7957],[7960,7965],[7968,8005],[8008,8013],[8016,8023],[8025,8025],[8027,8027],[8029,8029],[8031,8061],[8064,8116],[8118,8124],[8126,8126],[8130,8132],[8134,8140],[8144,8147],[8150,8155],[8160,8172],[8178,8180],[8182,8188],[8305,8305],[8319,8319],[8336,8348],[8450,8450],[8455,8455],[8458,8467],[8469,8469],[8473,8477],[8484,8484],[8486,8486],[8488,8488],[8490,8493],[8495,8505],[8508,8511],[8517,8521],[8526,8526],[8579,8580],[11264,11492],[11499,11502],[11506,11507],[11520,11557],[11559,11559],[11565,11565],[11568,11623],[11631,11631],[11648,11670],[11680,11686],[11688,11694],[11696,11702],[11704,11710],[11712,11718],[11720,11726],[11728,11734],[11736,11742],[11823,11823],[12293,12294],[12337,12341],[12347,12348],[12353,12438],[12445,12447],[12449,12538],[12540,12543],[12549,12591],[12593,12686],[12704,12735],[12784,12799],[13312,19903],[19968,42124],[42192,42237],[42240,42508],[42512,42527],[42538,42539],[42560,42606],[42623,42653],[42656,42725],[42775,42783],[42786,42888],[42891,42972],[42993,43009],[43011,43013],[43015,43018],[43020,43042],[43072,43123],[43138,43187],[43250,43255],[43259,43259],[43261,43262],[43274,43301],[43312,43334],[43360,43388],[43396,43442],[43471,43471],[43488,43492],[43494,43503],[43514,43518],[43520,43560],[43584,43586],[43588,43595],[43616,43638],[43642,43642],[43646,43695],[43697,43697],[43701,43702],[43705,43709],[43712,43712],[43714,43714],[43739,43741],[43744,43754],[43762,43764],[43777,43782],[43785,43790],[43793,43798],[43808,43814],[43816,43822],[43824,43866],[43868,43881],[43888,44002],[44032,55203],[55216,55238],[55243,55291],[63744,64109],[64112,64217],[64256,64262],[64275,64279],[64285,64285],[64287,64296],[64298,64310],[64312,64316],[64318,64318],[64320,64321],[64323,64324],[64326,64433],[64467,64829],[64848,64911],[64914,64967],[65008,65019],[65136,65140],[65142,65276],[65313,65338],[65345,65370],[65382,65470],[65474,65479],[65482,65487],[65490,65495],[65498,65500],[65536,65547],[65549,65574],[65576,65594],[65596,65597],[65599,65613],[65616,65629],[65664,65786],[66176,66204],[66208,66256],[66304,66335],[66349,66368],[66370,66377],[66384,66421],[66432,66461],[66464,66499],[66504,66511],[66560,66717],[66736,66771],[66776,66811],[66816,66855],[66864,66915],[66928,66938],[66940,66954],[66956,66962],[66964,66965],[66967,66977],[66979,66993],[66995,67001],[67003,67004],[67008,67059],[67072,67382],[67392,67413],[67424,67431],[67456,67461],[67463,67504],[67506,67514],[67584,67589],[67592,67592],[67594,67637],[67639,67640],[67644,67644],[67647,67669],[67680,67702],[67712,67742],[67808,67826],[67828,67829],[67840,67861],[67872,67897],[67904,67929],[67968,68023],[68030,68031],[68096,68096],[68112,68115],[68117,68119],[68121,68149],[68192,68220],[68224,68252],[68288,68295],[68297,68324],[68352,68405],[68416,68437],[68448,68466],[68480,68497],[68608,68680],[68736,68786],[68800,68850],[68864,68899],[68938,68965],[68975,68997],[69248,69289],[69296,69297],[69314,69319],[69376,69404],[69415,69415],[69424,69445],[69488,69505],[69552,69572],[69600,69622],[69635,69687],[69745,69746],[69749,69749],[69763,69807],[69840,69864],[69891,69926],[69956,69956],[69959,69959],[69968,70002],[70006,70006],[70019,70066],[70081,70084],[70106,70106],[70108,70108],[70144,70161],[70163,70187],[70207,70208],[70272,70278],[70280,70280],[70282,70285],[70287,70301],[70303,70312],[70320,70366],[70405,70412],[70415,70416],[70419,70440],[70442,70448],[70450,70451],[70453,70457],[70461,70461],[70480,70480],[70493,70497],[70528,70537],[70539,70539],[70542,70542],[70544,70581],[70583,70583],[70609,70609],[70611,70611],[70656,70708],[70727,70730],[70751,70753],[70784,70831],[70852,70853],[70855,70855],[71040,71086],[71128,71131],[71168,71215],[71236,71236],[71296,71338],[71352,71352],[71424,71450],[71488,71494],[71680,71723],[71840,71903],[71935,71942],[71945,71945],[71948,71955],[71957,71958],[71960,71983],[71999,71999],[72001,72001],[72096,72103],[72106,72144],[72161,72161],[72163,72163],[72192,72192],[72203,72242],[72250,72250],[72272,72272],[72284,72329],[72349,72349],[72368,72440],[72640,72672],[72704,72712],[72714,72750],[72768,72768],[72818,72847],[72960,72966],[72968,72969],[72971,73008],[73030,73030],[73056,73061],[73063,73064],[73066,73097],[73112,73112],[73136,73179],[73440,73458],[73474,73474],[73476,73488],[73490,73523],[73648,73648],[73728,74649],[74880,75075],[77712,77808],[77824,78895],[78913,78918],[78944,82938],[82944,83526],[90368,90397],[92160,92728],[92736,92766],[92784,92862],[92880,92909],[92928,92975],[92992,92995],[93027,93047],[93053,93071],[93504,93548],[93760,93823],[93856,93880],[93883,93907],[93952,94026],[94032,94032],[94099,94111],[94176,94177],[94179,94179],[94194,94195],[94208,101589],[101631,101662],[101760,101874],[110576,110579],[110581,110587],[110589,110590],[110592,110882],[110898,110898],[110928,110930],[110933,110933],[110948,110951],[110960,111355],[113664,113770],[113776,113788],[113792,113800],[113808,113817],[119808,119892],[119894,119964],[119966,119967],[119970,119970],[119973,119974],[119977,119980],[119982,119993],[119995,119995],[119997,120003],[120005,120069],[120071,120074],[120077,120084],[120086,120092],[120094,120121],[120123,120126],[120128,120132],[120134,120134],[120138,120144],[120146,120485],[120488,120512],[120514,120538],[120540,120570],[120572,120596],[120598,120628],[120630,120654],[120656,120686],[120688,120712],[120714,120744],[120746,120770],[120772,120779],[122624,122654],[122661,122666],[122928,122989],[123136,123180],[123191,123197],[123214,123214],[123536,123565],[123584,123627],[124112,124139],[124368,124397],[124400,124400],[124608,124638],[124640,124642],[124644,124645],[124647,124653],[124656,124660],[124670,124671],[124896,124902],[124904,124907],[124909,124910],[124912,124926],[124928,125124],[125184,125251],[125259,125259],[126464,126467],[126469,126495],[126497,126498],[126500,126500],[126503,126503],[126505,126514],[126516,126519],[126521,126521],[126523,126523],[126530,126530],[126535,126535],[126537,126537],[126539,126539],[126541,126543],[126545,126546],[126548,126548],[126551,126551],[126553,126553],[126555,126555],[126557,126557],[126559,126559],[126561,126562],[126564,126564],[126567,126570],[126572,126578],[126580,126583],[126585,126588],[126590,126590],[126592,126601],[126603,126619],[126625,126627],[126629,126633],[126635,126651],[131072,173791],[173824,178205],[178208,183981],[183984,191456],[191472,192093],[194560,195101],[196608,201546],[201552,210041]],[],false,true],["repeat_end",0,0],["accept"]],"flags":"gu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[955,28450,23383]],"from":0,"indices":[[0,3]],"to":3}],"rust_output":"λ漢字 123"},{"case":{"flags":"gu","input":"abcλ","matches":[{"capture_units":[[97,98,99]],"from":0,"indices":[[0,3]],"name_units":null,"to":3}],"name":"unicode-binary-property","pattern":"\\p{ASCII}+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,3,[]],["set",[[0,127]],[],false,true],["repeat_end",0,0],["accept"]],"flags":"gu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,98,99]],"from":0,"indices":[[0,3]],"to":3}],"rust_output":"abcλ"},{"case":{"flags":"gu","input":"あーア","matches":[{"capture_units":[[12354,12540]],"from":0,"indices":[[0,2]],"name_units":null,"to":2}],"name":"unicode-script-extensions","pattern":"\\p{Script_Extensions=Hiragana}+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,3,[]],["set",[[12289,12291],[12296,12305],[12307,12319],[12336,12341],[12343,12343],[12348,12349],[12353,12438],[12441,12448],[12539,12540],[65093,65094],[65377,65381],[65392,65392],[65438,65439],[110593,110879],[110898,110898],[110928,110930],[127488,127488]],[],false,true],["repeat_end",0,0],["accept"]],"flags":"gu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[12354,12540]],"from":0,"indices":[[0,2]],"to":2}],"rust_output":"あーア"},{"case":{"flags":"iu","input":"a","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"unicode-negated-property-fold","pattern":"\\P{Lowercase_Letter}"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,304],[306,311],[313,328],[330,396],[398,425],[428,441],[443,445],[447,495],[497,544],[546,563],[570,596],[598,599],[601,601],[603,604],[608,609],[611,614],[616,620],[623,623],[625,626],[629,629],[637,637],[640,640],[642,643],[647,652],[658,658],[660,661],[669,670],[688,911],[913,943],[945,1019],[1021,1375],[1377,1414],[1417,7423],[7468,7530],[7544,7545],[7549,7549],[7566,7566],[7579,7829],[7835,7835],[7838,7838],[7840,8015],[8017,8017],[8019,8019],[8021,8021],[8023,8113],[8115,8115],[8117,8117],[8120,8129],[8131,8131],[8133,8133],[8136,8145],[8148,8149],[8152,8161],[8165,8165],[8168,8177],[8179,8179],[8181,8181],[8184,8457],[8459,8461],[8464,8466],[8468,8494],[8496,8499],[8501,8504],[8506,8507],[8510,8517],[8522,11376],[11378,11379],[11381,11382],[11388,11491],[11493,42799],[42802,42864],[42873,42893],[42895,42900],[42902,42926],[42928,43001],[43003,43823],[43859,43859],[43867,43871],[43881,64255],[64263,64274],[64280,119833],[119860,119885],[119893,119893],[119912,119937],[119964,119989],[119994,119994],[119996,119996],[120004,120004],[120016,120041],[120068,120093],[120120,120145],[120172,120197],[120224,120249],[120276,120301],[120328,120353],[120380,120405],[120432,120457],[120486,120513],[120539,120539],[120546,120571],[120597,120597],[120604,120629],[120655,120655],[120662,120687],[120713,120713],[120720,120745],[120771,120771],[120778,120778],[120780,122623],[122634,122634],[122655,122660],[122667,1114111]],[],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"a"},{"case":{"flags":"iv","input":"a","matches":[],"name":"sets-negated-property-fold","pattern":"\\P{Lowercase_Letter}"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,64],[91,96],[123,180],[182,191],[215,215],[247,247],[304,304],[443,443],[448,451],[660,661],[688,836],[838,879],[884,885],[888,890],[894,894],[896,901],[903,903],[907,907],[909,909],[930,930],[978,980],[1014,1014],[1154,1161],[1328,1328],[1367,1375],[1417,4255],[4294,4294],[4296,4300],[4302,4303],[4347,4348],[4352,5023],[5110,5111],[5118,7295],[7307,7311],[7355,7356],[7360,7423],[7468,7530],[7544,7544],[7579,7679],[7958,7959],[7966,7967],[8006,8007],[8014,8015],[8024,8024],[8026,8026],[8028,8028],[8030,8030],[8062,8063],[8117,8117],[8125,8125],[8127,8129],[8133,8133],[8141,8143],[8148,8149],[8156,8159],[8173,8177],[8181,8181],[8189,8457],[8459,8461],[8464,8466],[8468,8485],[8487,8489],[8492,8494],[8496,8497],[8499,8499],[8501,8504],[8506,8507],[8510,8517],[8522,8525],[8527,8578],[8581,11263],[11388,11389],[11493,11498],[11503,11505],[11508,11519],[11558,11558],[11560,11564],[11566,42559],[42606,42623],[42652,42785],[42864,42864],[42888,42890],[42895,42895],[42973,42996],[42999,43001],[43003,43823],[43867,43871],[43881,43887],[43968,64255],[64263,64274],[64280,65312],[65339,65344],[65371,66559],[66640,66735],[66772,66775],[66812,66927],[66939,66939],[66955,66955],[66963,66963],[66966,66966],[66978,66978],[66994,66994],[67002,67002],[67005,68735],[68787,68799],[68851,68943],[68966,68975],[68998,71839],[71904,93759],[93824,93855],[93881,93882],[93908,119833],[119860,119885],[119893,119893],[119912,119937],[119964,119989],[119994,119994],[119996,119996],[120004,120004],[120016,120041],[120068,120093],[120120,120145],[120172,120197],[120224,120249],[120276,120301],[120328,120353],[120380,120405],[120432,120457],[120486,120513],[120539,120539],[120546,120571],[120597,120597],[120604,120629],[120655,120655],[120662,120687],[120713,120713],[120720,120745],[120771,120771],[120778,120778],[120780,122623],[122634,122634],[122655,122660],[122667,125183],[125252,1114111]],[],true,true],["accept"]],"flags":"iv","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"a"},{"case":{"flags":"v","input":"a1Zλ","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"unicode-set-intersection","pattern":"[\\p{ASCII}&&\\p{Letter}]+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,3,[]],["set",[[65,90],[97,122]],[],false,true],["repeat_end",0,0],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"a1Zλ"},{"case":{"flags":"v","input":"abcde","matches":[{"capture_units":[[98,99,100]],"from":1,"indices":[[1,4]],"name_units":null,"to":4}],"name":"unicode-set-subtraction","pattern":"[[a-z]--[aeiou]]+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,3,[]],["set",[[98,100],[102,104],[106,110],[112,116],[118,122]],[],false,true],["repeat_end",0,0],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[98,99,100]],"from":1,"indices":[[1,4]],"to":4}],"rust_output":"abcde"},{"case":{"flags":"gv","input":"abac","matches":[{"capture_units":[[97,98]],"from":0,"indices":[[0,2]],"name_units":null,"to":2},{"capture_units":[[97]],"from":2,"indices":[[2,3]],"name_units":null,"to":3},{"capture_units":[[99]],"from":3,"indices":[[3,4]],"name_units":null,"to":4}],"name":"unicode-string-set","pattern":"[\\q{ab|a|c}]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[97,97],[99,99]],[[97,98]],false,true],["accept"]],"flags":"gv","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,98]],"from":0,"indices":[[0,2]],"to":2},{"capture_units":[[97]],"from":2,"indices":[[2,3]],"to":3},{"capture_units":[[99]],"from":3,"indices":[[3,4]],"to":4}],"rust_output":"abac"},{"case":{"flags":"v","input":"abcdef","matches":[{"capture_units":[[99,100]],"from":2,"indices":[[2,4]],"name_units":null,"to":4}],"name":"unicode-string-set-intersection","pattern":"[\\q{ab|cd}&&\\q{cd|ef}]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[],[[99,100]],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[99,100]],"from":2,"indices":[[2,4]],"to":4}],"rust_output":"abcdef"},{"case":{"flags":"v","input":"👩‍💻","matches":[{"capture_units":[[55357,56425,8205,55357,56507]],"from":0,"indices":[[0,5]],"name_units":null,"to":5}],"name":"unicode-emoji-strings","pattern":"\\p{RGI_Emoji}"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[8986,8987],[9193,9196],[9200,9200],[9203,9203],[9725,9726],[9748,9749],[9800,9811],[9855,9855],[9875,9875],[9889,9889],[9898,9899],[9917,9918],[9924,9925],[9934,9934],[9940,9940],[9962,9962],[9970,9971],[9973,9973],[9978,9978],[9981,9981],[9989,9989],[9994,9995],[10024,10024],[10060,10060],[10062,10062],[10067,10069],[10071,10071],[10133,10135],[10160,10160],[10175,10175],[11035,11036],[11088,11088],[11093,11093],[126980,126980],[127183,127183],[127374,127374],[127377,127386],[127489,127489],[127514,127514],[127535,127535],[127538,127542],[127544,127546],[127568,127569],[127744,127776],[127789,127797],[127799,127868],[127870,127891],[127904,127946],[127951,127955],[127968,127984],[127988,127988],[127992,128062],[128064,128064],[128066,128252],[128255,128317],[128331,128334],[128336,128359],[128378,128378],[128405,128406],[128420,128420],[128507,128591],[128640,128709],[128716,128716],[128720,128722],[128725,128728],[128732,128735],[128747,128748],[128756,128764],[128992,129003],[129008,129008],[129292,129338],[129340,129349],[129351,129535],[129648,129660],[129664,129674],[129678,129734],[129736,129736],[129741,129756],[129759,129770],[129775,129784]],[[128104,127995,8205,10084,65039,8205,128139,8205,128104,127995],[128104,127995,8205,10084,65039,8205,128139,8205,128104,127996],[128104,127995,8205,10084,65039,8205,128139,8205,128104,127997],[128104,127995,8205,10084,65039,8205,128139,8205,128104,127998],[128104,127995,8205,10084,65039,8205,128139,8205,128104,127999],[128104,127996,8205,10084,65039,8205,128139,8205,128104,127995],[128104,127996,8205,10084,65039,8205,128139,8205,128104,127996],[128104,127996,8205,10084,65039,8205,128139,8205,128104,127997],[128104,127996,8205,10084,65039,8205,128139,8205,128104,127998],[128104,127996,8205,10084,65039,8205,128139,8205,128104,127999],[128104,127997,8205,10084,65039,8205,128139,8205,128104,127995],[128104,127997,8205,10084,65039,8205,128139,8205,128104,127996],[128104,127997,8205,10084,65039,8205,128139,8205,128104,127997],[128104,127997,8205,10084,65039,8205,128139,8205,128104,127998],[128104,127997,8205,10084,65039,8205,128139,8205,128104,127999],[128104,127998,8205,10084,65039,8205,128139,8205,128104,127995],[128104,127998,8205,10084,65039,8205,128139,8205,128104,127996],[128104,127998,8205,10084,65039,8205,128139,8205,128104,127997],[128104,127998,8205,10084,65039,8205,128139,8205,128104,127998],[128104,127998,8205,10084,65039,8205,128139,8205,128104,127999],[128104,127999,8205,10084,65039,8205,128139,8205,128104,127995],[128104,127999,8205,10084,65039,8205,128139,8205,128104,127996],[128104,127999,8205,10084,65039,8205,128139,8205,128104,127997],[128104,127999,8205,10084,65039,8205,128139,8205,128104,127998],[128104,127999,8205,10084,65039,8205,128139,8205,128104,127999],[128105,127995,8205,10084,65039,8205,128139,8205,128104,127995],[128105,127995,8205,10084,65039,8205,128139,8205,128104,127996],[128105,127995,8205,10084,65039,8205,128139,8205,128104,127997],[128105,127995,8205,10084,65039,8205,128139,8205,128104,127998],[128105,127995,8205,10084,65039,8205,128139,8205,128104,127999],[128105,127995,8205,10084,65039,8205,128139,8205,128105,127995],[128105,127995,8205,10084,65039,8205,128139,8205,128105,127996],[128105,127995,8205,10084,65039,8205,128139,8205,128105,127997],[128105,127995,8205,10084,65039,8205,128139,8205,128105,127998],[128105,127995,8205,10084,65039,8205,128139,8205,128105,127999],[128105,127996,8205,10084,65039,8205,128139,8205,128104,127995],[128105,127996,8205,10084,65039,8205,128139,8205,128104,127996],[128105,127996,8205,10084,65039,8205,128139,8205,128104,127997],[128105,127996,8205,10084,65039,8205,128139,8205,128104,127998],[128105,127996,8205,10084,65039,8205,128139,8205,128104,127999],[128105,127996,8205,10084,65039,8205,128139,8205,128105,127995],[128105,127996,8205,10084,65039,8205,128139,8205,128105,127996],[128105,127996,8205,10084,65039,8205,128139,8205,128105,127997],[128105,127996,8205,10084,65039,8205,128139,8205,128105,127998],[128105,127996,8205,10084,65039,8205,128139,8205,128105,127999],[128105,127997,8205,10084,65039,8205,128139,8205,128104,127995],[128105,127997,8205,10084,65039,8205,128139,8205,128104,127996],[128105,127997,8205,10084,65039,8205,128139,8205,128104,127997],[128105,127997,8205,10084,65039,8205,128139,8205,128104,127998],[128105,127997,8205,10084,65039,8205,128139,8205,128104,127999],[128105,127997,8205,10084,65039,8205,128139,8205,128105,127995],[128105,127997,8205,10084,65039,8205,128139,8205,128105,127996],[128105,127997,8205,10084,65039,8205,128139,8205,128105,127997],[128105,127997,8205,10084,65039,8205,128139,8205,128105,127998],[128105,127997,8205,10084,65039,8205,128139,8205,128105,127999],[128105,127998,8205,10084,65039,8205,128139,8205,128104,127995],[128105,127998,8205,10084,65039,8205,128139,8205,128104,127996],[128105,127998,8205,10084,65039,8205,128139,8205,128104,127997],[128105,127998,8205,10084,65039,8205,128139,8205,128104,127998],[128105,127998,8205,10084,65039,8205,128139,8205,128104,127999],[128105,127998,8205,10084,65039,8205,128139,8205,128105,127995],[128105,127998,8205,10084,65039,8205,128139,8205,128105,127996],[128105,127998,8205,10084,65039,8205,128139,8205,128105,127997],[128105,127998,8205,10084,65039,8205,128139,8205,128105,127998],[128105,127998,8205,10084,65039,8205,128139,8205,128105,127999],[128105,127999,8205,10084,65039,8205,128139,8205,128104,127995],[128105,127999,8205,10084,65039,8205,128139,8205,128104,127996],[128105,127999,8205,10084,65039,8205,128139,8205,128104,127997],[128105,127999,8205,10084,65039,8205,128139,8205,128104,127998],[128105,127999,8205,10084,65039,8205,128139,8205,128104,127999],[128105,127999,8205,10084,65039,8205,128139,8205,128105,127995],[128105,127999,8205,10084,65039,8205,128139,8205,128105,127996],[128105,127999,8205,10084,65039,8205,128139,8205,128105,127997],[128105,127999,8205,10084,65039,8205,128139,8205,128105,127998],[128105,127999,8205,10084,65039,8205,128139,8205,128105,127999],[129489,127995,8205,10084,65039,8205,128139,8205,129489,127996],[129489,127995,8205,10084,65039,8205,128139,8205,129489,127997],[129489,127995,8205,10084,65039,8205,128139,8205,129489,127998],[129489,127995,8205,10084,65039,8205,128139,8205,129489,127999],[129489,127996,8205,10084,65039,8205,128139,8205,129489,127995],[129489,127996,8205,10084,65039,8205,128139,8205,129489,127997],[129489,127996,8205,10084,65039,8205,128139,8205,129489,127998],[129489,127996,8205,10084,65039,8205,128139,8205,129489,127999],[129489,127997,8205,10084,65039,8205,128139,8205,129489,127995],[129489,127997,8205,10084,65039,8205,128139,8205,129489,127996],[129489,127997,8205,10084,65039,8205,128139,8205,129489,127998],[129489,127997,8205,10084,65039,8205,128139,8205,129489,127999],[129489,127998,8205,10084,65039,8205,128139,8205,129489,127995],[129489,127998,8205,10084,65039,8205,128139,8205,129489,127996],[129489,127998,8205,10084,65039,8205,128139,8205,129489,127997],[129489,127998,8205,10084,65039,8205,128139,8205,129489,127999],[129489,127999,8205,10084,65039,8205,128139,8205,129489,127995],[129489,127999,8205,10084,65039,8205,128139,8205,129489,127996],[129489,127999,8205,10084,65039,8205,128139,8205,129489,127997],[129489,127999,8205,10084,65039,8205,128139,8205,129489,127998],[127939,127995,8205,9792,65039,8205,10145,65039],[127939,127995,8205,9794,65039,8205,10145,65039],[127939,127996,8205,9792,65039,8205,10145,65039],[127939,127996,8205,9794,65039,8205,10145,65039],[127939,127997,8205,9792,65039,8205,10145,65039],[127939,127997,8205,9794,65039,8205,10145,65039],[127939,127998,8205,9792,65039,8205,10145,65039],[127939,127998,8205,9794,65039,8205,10145,65039],[127939,127999,8205,9792,65039,8205,10145,65039],[127939,127999,8205,9794,65039,8205,10145,65039],[128104,8205,10084,65039,8205,128139,8205,128104],[128104,127995,8205,10084,65039,8205,128104,127995],[128104,127995,8205,10084,65039,8205,128104,127996],[128104,127995,8205,10084,65039,8205,128104,127997],[128104,127995,8205,10084,65039,8205,128104,127998],[128104,127995,8205,10084,65039,8205,128104,127999],[128104,127996,8205,10084,65039,8205,128104,127995],[128104,127996,8205,10084,65039,8205,128104,127996],[128104,127996,8205,10084,65039,8205,128104,127997],[128104,127996,8205,10084,65039,8205,128104,127998],[128104,127996,8205,10084,65039,8205,128104,127999],[128104,127997,8205,10084,65039,8205,128104,127995],[128104,127997,8205,10084,65039,8205,128104,127996],[128104,127997,8205,10084,65039,8205,128104,127997],[128104,127997,8205,10084,65039,8205,128104,127998],[128104,127997,8205,10084,65039,8205,128104,127999],[128104,127998,8205,10084,65039,8205,128104,127995],[128104,127998,8205,10084,65039,8205,128104,127996],[128104,127998,8205,10084,65039,8205,128104,127997],[128104,127998,8205,10084,65039,8205,128104,127998],[128104,127998,8205,10084,65039,8205,128104,127999],[128104,127999,8205,10084,65039,8205,128104,127995],[128104,127999,8205,10084,65039,8205,128104,127996],[128104,127999,8205,10084,65039,8205,128104,127997],[128104,127999,8205,10084,65039,8205,128104,127998],[128104,127999,8205,10084,65039,8205,128104,127999],[128105,8205,10084,65039,8205,128139,8205,128104],[128105,8205,10084,65039,8205,128139,8205,128105],[128105,127995,8205,10084,65039,8205,128104,127995],[128105,127995,8205,10084,65039,8205,128104,127996],[128105,127995,8205,10084,65039,8205,128104,127997],[128105,127995,8205,10084,65039,8205,128104,127998],[128105,127995,8205,10084,65039,8205,128104,127999],[128105,127995,8205,10084,65039,8205,128105,127995],[128105,127995,8205,10084,65039,8205,128105,127996],[128105,127995,8205,10084,65039,8205,128105,127997],[128105,127995,8205,10084,65039,8205,128105,127998],[128105,127995,8205,10084,65039,8205,128105,127999],[128105,127996,8205,10084,65039,8205,128104,127995],[128105,127996,8205,10084,65039,8205,128104,127996],[128105,127996,8205,10084,65039,8205,128104,127997],[128105,127996,8205,10084,65039,8205,128104,127998],[128105,127996,8205,10084,65039,8205,128104,127999],[128105,127996,8205,10084,65039,8205,128105,127995],[128105,127996,8205,10084,65039,8205,128105,127996],[128105,127996,8205,10084,65039,8205,128105,127997],[128105,127996,8205,10084,65039,8205,128105,127998],[128105,127996,8205,10084,65039,8205,128105,127999],[128105,127997,8205,10084,65039,8205,128104,127995],[128105,127997,8205,10084,65039,8205,128104,127996],[128105,127997,8205,10084,65039,8205,128104,127997],[128105,127997,8205,10084,65039,8205,128104,127998],[128105,127997,8205,10084,65039,8205,128104,127999],[128105,127997,8205,10084,65039,8205,128105,127995],[128105,127997,8205,10084,65039,8205,128105,127996],[128105,127997,8205,10084,65039,8205,128105,127997],[128105,127997,8205,10084,65039,8205,128105,127998],[128105,127997,8205,10084,65039,8205,128105,127999],[128105,127998,8205,10084,65039,8205,128104,127995],[128105,127998,8205,10084,65039,8205,128104,127996],[128105,127998,8205,10084,65039,8205,128104,127997],[128105,127998,8205,10084,65039,8205,128104,127998],[128105,127998,8205,10084,65039,8205,128104,127999],[128105,127998,8205,10084,65039,8205,128105,127995],[128105,127998,8205,10084,65039,8205,128105,127996],[128105,127998,8205,10084,65039,8205,128105,127997],[128105,127998,8205,10084,65039,8205,128105,127998],[128105,127998,8205,10084,65039,8205,128105,127999],[128105,127999,8205,10084,65039,8205,128104,127995],[128105,127999,8205,10084,65039,8205,128104,127996],[128105,127999,8205,10084,65039,8205,128104,127997],[128105,127999,8205,10084,65039,8205,128104,127998],[128105,127999,8205,10084,65039,8205,128104,127999],[128105,127999,8205,10084,65039,8205,128105,127995],[128105,127999,8205,10084,65039,8205,128105,127996],[128105,127999,8205,10084,65039,8205,128105,127997],[128105,127999,8205,10084,65039,8205,128105,127998],[128105,127999,8205,10084,65039,8205,128105,127999],[128694,127995,8205,9792,65039,8205,10145,65039],[128694,127995,8205,9794,65039,8205,10145,65039],[128694,127996,8205,9792,65039,8205,10145,65039],[128694,127996,8205,9794,65039,8205,10145,65039],[128694,127997,8205,9792,65039,8205,10145,65039],[128694,127997,8205,9794,65039,8205,10145,65039],[128694,127998,8205,9792,65039,8205,10145,65039],[128694,127998,8205,9794,65039,8205,10145,65039],[128694,127999,8205,9792,65039,8205,10145,65039],[128694,127999,8205,9794,65039,8205,10145,65039],[129486,127995,8205,9792,65039,8205,10145,65039],[129486,127995,8205,9794,65039,8205,10145,65039],[129486,127996,8205,9792,65039,8205,10145,65039],[129486,127996,8205,9794,65039,8205,10145,65039],[129486,127997,8205,9792,65039,8205,10145,65039],[129486,127997,8205,9794,65039,8205,10145,65039],[129486,127998,8205,9792,65039,8205,10145,65039],[129486,127998,8205,9794,65039,8205,10145,65039],[129486,127999,8205,9792,65039,8205,10145,65039],[129486,127999,8205,9794,65039,8205,10145,65039],[129489,127995,8205,10084,65039,8205,129489,127996],[129489,127995,8205,10084,65039,8205,129489,127997],[129489,127995,8205,10084,65039,8205,129489,127998],[129489,127995,8205,10084,65039,8205,129489,127999],[129489,127996,8205,10084,65039,8205,129489,127995],[129489,127996,8205,10084,65039,8205,129489,127997],[129489,127996,8205,10084,65039,8205,129489,127998],[129489,127996,8205,10084,65039,8205,129489,127999],[129489,127997,8205,10084,65039,8205,129489,127995],[129489,127997,8205,10084,65039,8205,129489,127996],[129489,127997,8205,10084,65039,8205,129489,127998],[129489,127997,8205,10084,65039,8205,129489,127999],[129489,127998,8205,10084,65039,8205,129489,127995],[129489,127998,8205,10084,65039,8205,129489,127996],[129489,127998,8205,10084,65039,8205,129489,127997],[129489,127998,8205,10084,65039,8205,129489,127999],[129489,127999,8205,10084,65039,8205,129489,127995],[129489,127999,8205,10084,65039,8205,129489,127996],[129489,127999,8205,10084,65039,8205,129489,127997],[129489,127999,8205,10084,65039,8205,129489,127998],[127939,8205,9792,65039,8205,10145,65039],[127939,8205,9794,65039,8205,10145,65039],[127988,917607,917602,917605,917614,917607,917631],[127988,917607,917602,917619,917603,917620,917631],[127988,917607,917602,917623,917612,917619,917631],[128104,8205,128104,8205,128102,8205,128102],[128104,8205,128104,8205,128103,8205,128102],[128104,8205,128104,8205,128103,8205,128103],[128104,8205,128105,8205,128102,8205,128102],[128104,8205,128105,8205,128103,8205,128102],[128104,8205,128105,8205,128103,8205,128103],[128104,127995,8205,128048,8205,128104,127996],[128104,127995,8205,128048,8205,128104,127997],[128104,127995,8205,128048,8205,128104,127998],[128104,127995,8205,128048,8205,128104,127999],[128104,127995,8205,129309,8205,128104,127996],[128104,127995,8205,129309,8205,128104,127997],[128104,127995,8205,129309,8205,128104,127998],[128104,127995,8205,129309,8205,128104,127999],[128104,127995,8205,129455,8205,10145,65039],[128104,127995,8205,129468,8205,10145,65039],[128104,127995,8205,129469,8205,10145,65039],[128104,127995,8205,129775,8205,128104,127996],[128104,127995,8205,129775,8205,128104,127997],[128104,127995,8205,129775,8205,128104,127998],[128104,127995,8205,129775,8205,128104,127999],[128104,127996,8205,128048,8205,128104,127995],[128104,127996,8205,128048,8205,128104,127997],[128104,127996,8205,128048,8205,128104,127998],[128104,127996,8205,128048,8205,128104,127999],[128104,127996,8205,129309,8205,128104,127995],[128104,127996,8205,129309,8205,128104,127997],[128104,127996,8205,129309,8205,128104,127998],[128104,127996,8205,129309,8205,128104,127999],[128104,127996,8205,129455,8205,10145,65039],[128104,127996,8205,129468,8205,10145,65039],[128104,127996,8205,129469,8205,10145,65039],[128104,127996,8205,129775,8205,128104,127995],[128104,127996,8205,129775,8205,128104,127997],[128104,127996,8205,129775,8205,128104,127998],[128104,127996,8205,129775,8205,128104,127999],[128104,127997,8205,128048,8205,128104,127995],[128104,127997,8205,128048,8205,128104,127996],[128104,127997,8205,128048,8205,128104,127998],[128104,127997,8205,128048,8205,128104,127999],[128104,127997,8205,129309,8205,128104,127995],[128104,127997,8205,129309,8205,128104,127996],[128104,127997,8205,129309,8205,128104,127998],[128104,127997,8205,129309,8205,128104,127999],[128104,127997,8205,129455,8205,10145,65039],[128104,127997,8205,129468,8205,10145,65039],[128104,127997,8205,129469,8205,10145,65039],[128104,127997,8205,129775,8205,128104,127995],[128104,127997,8205,129775,8205,128104,127996],[128104,127997,8205,129775,8205,128104,127998],[128104,127997,8205,129775,8205,128104,127999],[128104,127998,8205,128048,8205,128104,127995],[128104,127998,8205,128048,8205,128104,127996],[128104,127998,8205,128048,8205,128104,127997],[128104,127998,8205,128048,8205,128104,127999],[128104,127998,8205,129309,8205,128104,127995],[128104,127998,8205,129309,8205,128104,127996],[128104,127998,8205,129309,8205,128104,127997],[128104,127998,8205,129309,8205,128104,127999],[128104,127998,8205,129455,8205,10145,65039],[128104,127998,8205,129468,8205,10145,65039],[128104,127998,8205,129469,8205,10145,65039],[128104,127998,8205,129775,8205,128104,127995],[128104,127998,8205,129775,8205,128104,127996],[128104,127998,8205,129775,8205,128104,127997],[128104,127998,8205,129775,8205,128104,127999],[128104,127999,8205,128048,8205,128104,127995],[128104,127999,8205,128048,8205,128104,127996],[128104,127999,8205,128048,8205,128104,127997],[128104,127999,8205,128048,8205,128104,127998],[128104,127999,8205,129309,8205,128104,127995],[128104,127999,8205,129309,8205,128104,127996],[128104,127999,8205,129309,8205,128104,127997],[128104,127999,8205,129309,8205,128104,127998],[128104,127999,8205,129455,8205,10145,65039],[128104,127999,8205,129468,8205,10145,65039],[128104,127999,8205,129469,8205,10145,65039],[128104,127999,8205,129775,8205,128104,127995],[128104,127999,8205,129775,8205,128104,127996],[128104,127999,8205,129775,8205,128104,127997],[128104,127999,8205,129775,8205,128104,127998],[128105,8205,128105,8205,128102,8205,128102],[128105,8205,128105,8205,128103,8205,128102],[128105,8205,128105,8205,128103,8205,128103],[128105,127995,8205,128048,8205,128105,127996],[128105,127995,8205,128048,8205,128105,127997],[128105,127995,8205,128048,8205,128105,127998],[128105,127995,8205,128048,8205,128105,127999],[128105,127995,8205,129309,8205,128104,127996],[128105,127995,8205,129309,8205,128104,127997],[128105,127995,8205,129309,8205,128104,127998],[128105,127995,8205,129309,8205,128104,127999],[128105,127995,8205,129309,8205,128105,127996],[128105,127995,8205,129309,8205,128105,127997],[128105,127995,8205,129309,8205,128105,127998],[128105,127995,8205,129309,8205,128105,127999],[128105,127995,8205,129455,8205,10145,65039],[128105,127995,8205,129468,8205,10145,65039],[128105,127995,8205,129469,8205,10145,65039],[128105,127995,8205,129775,8205,128105,127996],[128105,127995,8205,129775,8205,128105,127997],[128105,127995,8205,129775,8205,128105,127998],[128105,127995,8205,129775,8205,128105,127999],[128105,127996,8205,128048,8205,128105,127995],[128105,127996,8205,128048,8205,128105,127997],[128105,127996,8205,128048,8205,128105,127998],[128105,127996,8205,128048,8205,128105,127999],[128105,127996,8205,129309,8205,128104,127995],[128105,127996,8205,129309,8205,128104,127997],[128105,127996,8205,129309,8205,128104,127998],[128105,127996,8205,129309,8205,128104,127999],[128105,127996,8205,129309,8205,128105,127995],[128105,127996,8205,129309,8205,128105,127997],[128105,127996,8205,129309,8205,128105,127998],[128105,127996,8205,129309,8205,128105,127999],[128105,127996,8205,129455,8205,10145,65039],[128105,127996,8205,129468,8205,10145,65039],[128105,127996,8205,129469,8205,10145,65039],[128105,127996,8205,129775,8205,128105,127995],[128105,127996,8205,129775,8205,128105,127997],[128105,127996,8205,129775,8205,128105,127998],[128105,127996,8205,129775,8205,128105,127999],[128105,127997,8205,128048,8205,128105,127995],[128105,127997,8205,128048,8205,128105,127996],[128105,127997,8205,128048,8205,128105,127998],[128105,127997,8205,128048,8205,128105,127999],[128105,127997,8205,129309,8205,128104,127995],[128105,127997,8205,129309,8205,128104,127996],[128105,127997,8205,129309,8205,128104,127998],[128105,127997,8205,129309,8205,128104,127999],[128105,127997,8205,129309,8205,128105,127995],[128105,127997,8205,129309,8205,128105,127996],[128105,127997,8205,129309,8205,128105,127998],[128105,127997,8205,129309,8205,128105,127999],[128105,127997,8205,129455,8205,10145,65039],[128105,127997,8205,129468,8205,10145,65039],[128105,127997,8205,129469,8205,10145,65039],[128105,127997,8205,129775,8205,128105,127995],[128105,127997,8205,129775,8205,128105,127996],[128105,127997,8205,129775,8205,128105,127998],[128105,127997,8205,129775,8205,128105,127999],[128105,127998,8205,128048,8205,128105,127995],[128105,127998,8205,128048,8205,128105,127996],[128105,127998,8205,128048,8205,128105,127997],[128105,127998,8205,128048,8205,128105,127999],[128105,127998,8205,129309,8205,128104,127995],[128105,127998,8205,129309,8205,128104,127996],[128105,127998,8205,129309,8205,128104,127997],[128105,127998,8205,129309,8205,128104,127999],[128105,127998,8205,129309,8205,128105,127995],[128105,127998,8205,129309,8205,128105,127996],[128105,127998,8205,129309,8205,128105,127997],[128105,127998,8205,129309,8205,128105,127999],[128105,127998,8205,129455,8205,10145,65039],[128105,127998,8205,129468,8205,10145,65039],[128105,127998,8205,129469,8205,10145,65039],[128105,127998,8205,129775,8205,128105,127995],[128105,127998,8205,129775,8205,128105,127996],[128105,127998,8205,129775,8205,128105,127997],[128105,127998,8205,129775,8205,128105,127999],[128105,127999,8205,128048,8205,128105,127995],[128105,127999,8205,128048,8205,128105,127996],[128105,127999,8205,128048,8205,128105,127997],[128105,127999,8205,128048,8205,128105,127998],[128105,127999,8205,129309,8205,128104,127995],[128105,127999,8205,129309,8205,128104,127996],[128105,127999,8205,129309,8205,128104,127997],[128105,127999,8205,129309,8205,128104,127998],[128105,127999,8205,129309,8205,128105,127995],[128105,127999,8205,129309,8205,128105,127996],[128105,127999,8205,129309,8205,128105,127997],[128105,127999,8205,129309,8205,128105,127998],[128105,127999,8205,129455,8205,10145,65039],[128105,127999,8205,129468,8205,10145,65039],[128105,127999,8205,129469,8205,10145,65039],[128105,127999,8205,129775,8205,128105,127995],[128105,127999,8205,129775,8205,128105,127996],[128105,127999,8205,129775,8205,128105,127997],[128105,127999,8205,129775,8205,128105,127998],[128694,8205,9792,65039,8205,10145,65039],[128694,8205,9794,65039,8205,10145,65039],[129486,8205,9792,65039,8205,10145,65039],[129486,8205,9794,65039,8205,10145,65039],[129489,8205,129489,8205,129490,8205,129490],[129489,127995,8205,128048,8205,129489,127996],[129489,127995,8205,128048,8205,129489,127997],[129489,127995,8205,128048,8205,129489,127998],[129489,127995,8205,128048,8205,129489,127999],[129489,127995,8205,129309,8205,129489,127995],[129489,127995,8205,129309,8205,129489,127996],[129489,127995,8205,129309,8205,129489,127997],[129489,127995,8205,129309,8205,129489,127998],[129489,127995,8205,129309,8205,129489,127999],[129489,127995,8205,129455,8205,10145,65039],[129489,127995,8205,129468,8205,10145,65039],[129489,127995,8205,129469,8205,10145,65039],[129489,127995,8205,129775,8205,129489,127996],[129489,127995,8205,129775,8205,129489,127997],[129489,127995,8205,129775,8205,129489,127998],[129489,127995,8205,129775,8205,129489,127999],[129489,127996,8205,128048,8205,129489,127995],[129489,127996,8205,128048,8205,129489,127997],[129489,127996,8205,128048,8205,129489,127998],[129489,127996,8205,128048,8205,129489,127999],[129489,127996,8205,129309,8205,129489,127995],[129489,127996,8205,129309,8205,129489,127996],[129489,127996,8205,129309,8205,129489,127997],[129489,127996,8205,129309,8205,129489,127998],[129489,127996,8205,129309,8205,129489,127999],[129489,127996,8205,129455,8205,10145,65039],[129489,127996,8205,129468,8205,10145,65039],[129489,127996,8205,129469,8205,10145,65039],[129489,127996,8205,129775,8205,129489,127995],[129489,127996,8205,129775,8205,129489,127997],[129489,127996,8205,129775,8205,129489,127998],[129489,127996,8205,129775,8205,129489,127999],[129489,127997,8205,128048,8205,129489,127995],[129489,127997,8205,128048,8205,129489,127996],[129489,127997,8205,128048,8205,129489,127998],[129489,127997,8205,128048,8205,129489,127999],[129489,127997,8205,129309,8205,129489,127995],[129489,127997,8205,129309,8205,129489,127996],[129489,127997,8205,129309,8205,129489,127997],[129489,127997,8205,129309,8205,129489,127998],[129489,127997,8205,129309,8205,129489,127999],[129489,127997,8205,129455,8205,10145,65039],[129489,127997,8205,129468,8205,10145,65039],[129489,127997,8205,129469,8205,10145,65039],[129489,127997,8205,129775,8205,129489,127995],[129489,127997,8205,129775,8205,129489,127996],[129489,127997,8205,129775,8205,129489,127998],[129489,127997,8205,129775,8205,129489,127999],[129489,127998,8205,128048,8205,129489,127995],[129489,127998,8205,128048,8205,129489,127996],[129489,127998,8205,128048,8205,129489,127997],[129489,127998,8205,128048,8205,129489,127999],[129489,127998,8205,129309,8205,129489,127995],[129489,127998,8205,129309,8205,129489,127996],[129489,127998,8205,129309,8205,129489,127997],[129489,127998,8205,129309,8205,129489,127998],[129489,127998,8205,129309,8205,129489,127999],[129489,127998,8205,129455,8205,10145,65039],[129489,127998,8205,129468,8205,10145,65039],[129489,127998,8205,129469,8205,10145,65039],[129489,127998,8205,129775,8205,129489,127995],[129489,127998,8205,129775,8205,129489,127996],[129489,127998,8205,129775,8205,129489,127997],[129489,127998,8205,129775,8205,129489,127999],[129489,127999,8205,128048,8205,129489,127995],[129489,127999,8205,128048,8205,129489,127996],[129489,127999,8205,128048,8205,129489,127997],[129489,127999,8205,128048,8205,129489,127998],[129489,127999,8205,129309,8205,129489,127995],[129489,127999,8205,129309,8205,129489,127996],[129489,127999,8205,129309,8205,129489,127997],[129489,127999,8205,129309,8205,129489,127998],[129489,127999,8205,129309,8205,129489,127999],[129489,127999,8205,129455,8205,10145,65039],[129489,127999,8205,129468,8205,10145,65039],[129489,127999,8205,129469,8205,10145,65039],[129489,127999,8205,129775,8205,129489,127995],[129489,127999,8205,129775,8205,129489,127996],[129489,127999,8205,129775,8205,129489,127997],[129489,127999,8205,129775,8205,129489,127998],[128104,8205,10084,65039,8205,128104],[128104,8205,129455,8205,10145,65039],[128104,8205,129468,8205,10145,65039],[128104,8205,129469,8205,10145,65039],[128105,8205,10084,65039,8205,128104],[128105,8205,10084,65039,8205,128105],[128105,8205,129455,8205,10145,65039],[128105,8205,129468,8205,10145,65039],[128105,8205,129469,8205,10145,65039],[129489,8205,129455,8205,10145,65039],[129489,8205,129468,8205,10145,65039],[129489,8205,129469,8205,10145,65039],[9977,65039,8205,9792,65039],[9977,65039,8205,9794,65039],[9977,127995,8205,9792,65039],[9977,127995,8205,9794,65039],[9977,127996,8205,9792,65039],[9977,127996,8205,9794,65039],[9977,127997,8205,9792,65039],[9977,127997,8205,9794,65039],[9977,127998,8205,9792,65039],[9977,127998,8205,9794,65039],[9977,127999,8205,9792,65039],[9977,127999,8205,9794,65039],[127939,127995,8205,9792,65039],[127939,127995,8205,9794,65039],[127939,127995,8205,10145,65039],[127939,127996,8205,9792,65039],[127939,127996,8205,9794,65039],[127939,127996,8205,10145,65039],[127939,127997,8205,9792,65039],[127939,127997,8205,9794,65039],[127939,127997,8205,10145,65039],[127939,127998,8205,9792,65039],[127939,127998,8205,9794,65039],[127939,127998,8205,10145,65039],[127939,127999,8205,9792,65039],[127939,127999,8205,9794,65039],[127939,127999,8205,10145,65039],[127940,127995,8205,9792,65039],[127940,127995,8205,9794,65039],[127940,127996,8205,9792,65039],[127940,127996,8205,9794,65039],[127940,127997,8205,9792,65039],[127940,127997,8205,9794,65039],[127940,127998,8205,9792,65039],[127940,127998,8205,9794,65039],[127940,127999,8205,9792,65039],[127940,127999,8205,9794,65039],[127946,127995,8205,9792,65039],[127946,127995,8205,9794,65039],[127946,127996,8205,9792,65039],[127946,127996,8205,9794,65039],[127946,127997,8205,9792,65039],[127946,127997,8205,9794,65039],[127946,127998,8205,9792,65039],[127946,127998,8205,9794,65039],[127946,127999,8205,9792,65039],[127946,127999,8205,9794,65039],[127947,65039,8205,9792,65039],[127947,65039,8205,9794,65039],[127947,127995,8205,9792,65039],[127947,127995,8205,9794,65039],[127947,127996,8205,9792,65039],[127947,127996,8205,9794,65039],[127947,127997,8205,9792,65039],[127947,127997,8205,9794,65039],[127947,127998,8205,9792,65039],[127947,127998,8205,9794,65039],[127947,127999,8205,9792,65039],[127947,127999,8205,9794,65039],[127948,65039,8205,9792,65039],[127948,65039,8205,9794,65039],[127948,127995,8205,9792,65039],[127948,127995,8205,9794,65039],[127948,127996,8205,9792,65039],[127948,127996,8205,9794,65039],[127948,127997,8205,9792,65039],[127948,127997,8205,9794,65039],[127948,127998,8205,9792,65039],[127948,127998,8205,9794,65039],[127948,127999,8205,9792,65039],[127948,127999,8205,9794,65039],[127987,65039,8205,9895,65039],[128065,65039,8205,128488,65039],[128104,8205,128102,8205,128102],[128104,8205,128103,8205,128102],[128104,8205,128103,8205,128103],[128104,8205,128104,8205,128102],[128104,8205,128104,8205,128103],[128104,8205,128105,8205,128102],[128104,8205,128105,8205,128103],[128104,127995,8205,9877,65039],[128104,127995,8205,9878,65039],[128104,127995,8205,9992,65039],[128104,127996,8205,9877,65039],[128104,127996,8205,9878,65039],[128104,127996,8205,9992,65039],[128104,127997,8205,9877,65039],[128104,127997,8205,9878,65039],[128104,127997,8205,9992,65039],[128104,127998,8205,9877,65039],[128104,127998,8205,9878,65039],[128104,127998,8205,9992,65039],[128104,127999,8205,9877,65039],[128104,127999,8205,9878,65039],[128104,127999,8205,9992,65039],[128105,8205,128102,8205,128102],[128105,8205,128103,8205,128102],[128105,8205,128103,8205,128103],[128105,8205,128105,8205,128102],[128105,8205,128105,8205,128103],[128105,127995,8205,9877,65039],[128105,127995,8205,9878,65039],[128105,127995,8205,9992,65039],[128105,127996,8205,9877,65039],[128105,127996,8205,9878,65039],[128105,127996,8205,9992,65039],[128105,127997,8205,9877,65039],[128105,127997,8205,9878,65039],[128105,127997,8205,9992,65039],[128105,127998,8205,9877,65039],[128105,127998,8205,9878,65039],[128105,127998,8205,9992,65039],[128105,127999,8205,9877,65039],[128105,127999,8205,9878,65039],[128105,127999,8205,9992,65039],[128110,127995,8205,9792,65039],[128110,127995,8205,9794,65039],[128110,127996,8205,9792,65039],[128110,127996,8205,9794,65039],[128110,127997,8205,9792,65039],[128110,127997,8205,9794,65039],[128110,127998,8205,9792,65039],[128110,127998,8205,9794,65039],[128110,127999,8205,9792,65039],[128110,127999,8205,9794,65039],[128111,127995,8205,9792,65039],[128111,127995,8205,9794,65039],[128111,127996,8205,9792,65039],[128111,127996,8205,9794,65039],[128111,127997,8205,9792,65039],[128111,127997,8205,9794,65039],[128111,127998,8205,9792,65039],[128111,127998,8205,9794,65039],[128111,127999,8205,9792,65039],[128111,127999,8205,9794,65039],[128112,127995,8205,9792,65039],[128112,127995,8205,9794,65039],[128112,127996,8205,9792,65039],[128112,127996,8205,9794,65039],[128112,127997,8205,9792,65039],[128112,127997,8205,9794,65039],[128112,127998,8205,9792,65039],[128112,127998,8205,9794,65039],[128112,127999,8205,9792,65039],[128112,127999,8205,9794,65039],[128113,127995,8205,9792,65039],[128113,127995,8205,9794,65039],[128113,127996,8205,9792,65039],[128113,127996,8205,9794,65039],[128113,127997,8205,9792,65039],[128113,127997,8205,9794,65039],[128113,127998,8205,9792,65039],[128113,127998,8205,9794,65039],[128113,127999,8205,9792,65039],[128113,127999,8205,9794,65039],[128115,127995,8205,9792,65039],[128115,127995,8205,9794,65039],[128115,127996,8205,9792,65039],[128115,127996,8205,9794,65039],[128115,127997,8205,9792,65039],[128115,127997,8205,9794,65039],[128115,127998,8205,9792,65039],[128115,127998,8205,9794,65039],[128115,127999,8205,9792,65039],[128115,127999,8205,9794,65039],[128119,127995,8205,9792,65039],[128119,127995,8205,9794,65039],[128119,127996,8205,9792,65039],[128119,127996,8205,9794,65039],[128119,127997,8205,9792,65039],[128119,127997,8205,9794,65039],[128119,127998,8205,9792,65039],[128119,127998,8205,9794,65039],[128119,127999,8205,9792,65039],[128119,127999,8205,9794,65039],[128129,127995,8205,9792,65039],[128129,127995,8205,9794,65039],[128129,127996,8205,9792,65039],[128129,127996,8205,9794,65039],[128129,127997,8205,9792,65039],[128129,127997,8205,9794,65039],[128129,127998,8205,9792,65039],[128129,127998,8205,9794,65039],[128129,127999,8205,9792,65039],[128129,127999,8205,9794,65039],[128130,127995,8205,9792,65039],[128130,127995,8205,9794,65039],[128130,127996,8205,9792,65039],[128130,127996,8205,9794,65039],[128130,127997,8205,9792,65039],[128130,127997,8205,9794,65039],[128130,127998,8205,9792,65039],[128130,127998,8205,9794,65039],[128130,127999,8205,9792,65039],[128130,127999,8205,9794,65039],[128134,127995,8205,9792,65039],[128134,127995,8205,9794,65039],[128134,127996,8205,9792,65039],[128134,127996,8205,9794,65039],[128134,127997,8205,9792,65039],[128134,127997,8205,9794,65039],[128134,127998,8205,9792,65039],[128134,127998,8205,9794,65039],[128134,127999,8205,9792,65039],[128134,127999,8205,9794,65039],[128135,127995,8205,9792,65039],[128135,127995,8205,9794,65039],[128135,127996,8205,9792,65039],[128135,127996,8205,9794,65039],[128135,127997,8205,9792,65039],[128135,127997,8205,9794,65039],[128135,127998,8205,9792,65039],[128135,127998,8205,9794,65039],[128135,127999,8205,9792,65039],[128135,127999,8205,9794,65039],[128373,65039,8205,9792,65039],[128373,65039,8205,9794,65039],[128373,127995,8205,9792,65039],[128373,127995,8205,9794,65039],[128373,127996,8205,9792,65039],[128373,127996,8205,9794,65039],[128373,127997,8205,9792,65039],[128373,127997,8205,9794,65039],[128373,127998,8205,9792,65039],[128373,127998,8205,9794,65039],[128373,127999,8205,9792,65039],[128373,127999,8205,9794,65039],[128581,127995,8205,9792,65039],[128581,127995,8205,9794,65039],[128581,127996,8205,9792,65039],[128581,127996,8205,9794,65039],[128581,127997,8205,9792,65039],[128581,127997,8205,9794,65039],[128581,127998,8205,9792,65039],[128581,127998,8205,9794,65039],[128581,127999,8205,9792,65039],[128581,127999,8205,9794,65039],[128582,127995,8205,9792,65039],[128582,127995,8205,9794,65039],[128582,127996,8205,9792,65039],[128582,127996,8205,9794,65039],[128582,127997,8205,9792,65039],[128582,127997,8205,9794,65039],[128582,127998,8205,9792,65039],[128582,127998,8205,9794,65039],[128582,127999,8205,9792,65039],[128582,127999,8205,9794,65039],[128583,127995,8205,9792,65039],[128583,127995,8205,9794,65039],[128583,127996,8205,9792,65039],[128583,127996,8205,9794,65039],[128583,127997,8205,9792,65039],[128583,127997,8205,9794,65039],[128583,127998,8205,9792,65039],[128583,127998,8205,9794,65039],[128583,127999,8205,9792,65039],[128583,127999,8205,9794,65039],[128587,127995,8205,9792,65039],[128587,127995,8205,9794,65039],[128587,127996,8205,9792,65039],[128587,127996,8205,9794,65039],[128587,127997,8205,9792,65039],[128587,127997,8205,9794,65039],[128587,127998,8205,9792,65039],[128587,127998,8205,9794,65039],[128587,127999,8205,9792,65039],[128587,127999,8205,9794,65039],[128589,127995,8205,9792,65039],[128589,127995,8205,9794,65039],[128589,127996,8205,9792,65039],[128589,127996,8205,9794,65039],[128589,127997,8205,9792,65039],[128589,127997,8205,9794,65039],[128589,127998,8205,9792,65039],[128589,127998,8205,9794,65039],[128589,127999,8205,9792,65039],[128589,127999,8205,9794,65039],[128590,127995,8205,9792,65039],[128590,127995,8205,9794,65039],[128590,127996,8205,9792,65039],[128590,127996,8205,9794,65039],[128590,127997,8205,9792,65039],[128590,127997,8205,9794,65039],[128590,127998,8205,9792,65039],[128590,127998,8205,9794,65039],[128590,127999,8205,9792,65039],[128590,127999,8205,9794,65039],[128675,127995,8205,9792,65039],[128675,127995,8205,9794,65039],[128675,127996,8205,9792,65039],[128675,127996,8205,9794,65039],[128675,127997,8205,9792,65039],[128675,127997,8205,9794,65039],[128675,127998,8205,9792,65039],[128675,127998,8205,9794,65039],[128675,127999,8205,9792,65039],[128675,127999,8205,9794,65039],[128692,127995,8205,9792,65039],[128692,127995,8205,9794,65039],[128692,127996,8205,9792,65039],[128692,127996,8205,9794,65039],[128692,127997,8205,9792,65039],[128692,127997,8205,9794,65039],[128692,127998,8205,9792,65039],[128692,127998,8205,9794,65039],[128692,127999,8205,9792,65039],[128692,127999,8205,9794,65039],[128693,127995,8205,9792,65039],[128693,127995,8205,9794,65039],[128693,127996,8205,9792,65039],[128693,127996,8205,9794,65039],[128693,127997,8205,9792,65039],[128693,127997,8205,9794,65039],[128693,127998,8205,9792,65039],[128693,127998,8205,9794,65039],[128693,127999,8205,9792,65039],[128693,127999,8205,9794,65039],[128694,127995,8205,9792,65039],[128694,127995,8205,9794,65039],[128694,127995,8205,10145,65039],[128694,127996,8205,9792,65039],[128694,127996,8205,9794,65039],[128694,127996,8205,10145,65039],[128694,127997,8205,9792,65039],[128694,127997,8205,9794,65039],[128694,127997,8205,10145,65039],[128694,127998,8205,9792,65039],[128694,127998,8205,9794,65039],[128694,127998,8205,10145,65039],[128694,127999,8205,9792,65039],[128694,127999,8205,9794,65039],[128694,127999,8205,10145,65039],[129318,127995,8205,9792,65039],[129318,127995,8205,9794,65039],[129318,127996,8205,9792,65039],[129318,127996,8205,9794,65039],[129318,127997,8205,9792,65039],[129318,127997,8205,9794,65039],[129318,127998,8205,9792,65039],[129318,127998,8205,9794,65039],[129318,127999,8205,9792,65039],[129318,127999,8205,9794,65039],[129333,127995,8205,9792,65039],[129333,127995,8205,9794,65039],[129333,127996,8205,9792,65039],[129333,127996,8205,9794,65039],[129333,127997,8205,9792,65039],[129333,127997,8205,9794,65039],[129333,127998,8205,9792,65039],[129333,127998,8205,9794,65039],[129333,127999,8205,9792,65039],[129333,127999,8205,9794,65039],[129335,127995,8205,9792,65039],[129335,127995,8205,9794,65039],[129335,127996,8205,9792,65039],[129335,127996,8205,9794,65039],[129335,127997,8205,9792,65039],[129335,127997,8205,9794,65039],[129335,127998,8205,9792,65039],[129335,127998,8205,9794,65039],[129335,127999,8205,9792,65039],[129335,127999,8205,9794,65039],[129336,127995,8205,9792,65039],[129336,127995,8205,9794,65039],[129336,127996,8205,9792,65039],[129336,127996,8205,9794,65039],[129336,127997,8205,9792,65039],[129336,127997,8205,9794,65039],[129336,127998,8205,9792,65039],[129336,127998,8205,9794,65039],[129336,127999,8205,9792,65039],[129336,127999,8205,9794,65039],[129337,127995,8205,9792,65039],[129337,127995,8205,9794,65039],[129337,127996,8205,9792,65039],[129337,127996,8205,9794,65039],[129337,127997,8205,9792,65039],[129337,127997,8205,9794,65039],[129337,127998,8205,9792,65039],[129337,127998,8205,9794,65039],[129337,127999,8205,9792,65039],[129337,127999,8205,9794,65039],[129340,127995,8205,9792,65039],[129340,127995,8205,9794,65039],[129340,127996,8205,9792,65039],[129340,127996,8205,9794,65039],[129340,127997,8205,9792,65039],[129340,127997,8205,9794,65039],[129340,127998,8205,9792,65039],[129340,127998,8205,9794,65039],[129340,127999,8205,9792,65039],[129340,127999,8205,9794,65039],[129341,127995,8205,9792,65039],[129341,127995,8205,9794,65039],[129341,127996,8205,9792,65039],[129341,127996,8205,9794,65039],[129341,127997,8205,9792,65039],[129341,127997,8205,9794,65039],[129341,127998,8205,9792,65039],[129341,127998,8205,9794,65039],[129341,127999,8205,9792,65039],[129341,127999,8205,9794,65039],[129342,127995,8205,9792,65039],[129342,127995,8205,9794,65039],[129342,127996,8205,9792,65039],[129342,127996,8205,9794,65039],[129342,127997,8205,9792,65039],[129342,127997,8205,9794,65039],[129342,127998,8205,9792,65039],[129342,127998,8205,9794,65039],[129342,127999,8205,9792,65039],[129342,127999,8205,9794,65039],[129464,127995,8205,9792,65039],[129464,127995,8205,9794,65039],[129464,127996,8205,9792,65039],[129464,127996,8205,9794,65039],[129464,127997,8205,9792,65039],[129464,127997,8205,9794,65039],[129464,127998,8205,9792,65039],[129464,127998,8205,9794,65039],[129464,127999,8205,9792,65039],[129464,127999,8205,9794,65039],[129465,127995,8205,9792,65039],[129465,127995,8205,9794,65039],[129465,127996,8205,9792,65039],[129465,127996,8205,9794,65039],[129465,127997,8205,9792,65039],[129465,127997,8205,9794,65039],[129465,127998,8205,9792,65039],[129465,127998,8205,9794,65039],[129465,127999,8205,9792,65039],[129465,127999,8205,9794,65039],[129485,127995,8205,9792,65039],[129485,127995,8205,9794,65039],[129485,127996,8205,9792,65039],[129485,127996,8205,9794,65039],[129485,127997,8205,9792,65039],[129485,127997,8205,9794,65039],[129485,127998,8205,9792,65039],[129485,127998,8205,9794,65039],[129485,127999,8205,9792,65039],[129485,127999,8205,9794,65039],[129486,127995,8205,9792,65039],[129486,127995,8205,9794,65039],[129486,127995,8205,10145,65039],[129486,127996,8205,9792,65039],[129486,127996,8205,9794,65039],[129486,127996,8205,10145,65039],[129486,127997,8205,9792,65039],[129486,127997,8205,9794,65039],[129486,127997,8205,10145,65039],[129486,127998,8205,9792,65039],[129486,127998,8205,9794,65039],[129486,127998,8205,10145,65039],[129486,127999,8205,9792,65039],[129486,127999,8205,9794,65039],[129486,127999,8205,10145,65039],[129487,127995,8205,9792,65039],[129487,127995,8205,9794,65039],[129487,127996,8205,9792,65039],[129487,127996,8205,9794,65039],[129487,127997,8205,9792,65039],[129487,127997,8205,9794,65039],[129487,127998,8205,9792,65039],[129487,127998,8205,9794,65039],[129487,127999,8205,9792,65039],[129487,127999,8205,9794,65039],[129489,8205,129309,8205,129489],[129489,8205,129489,8205,129490],[129489,8205,129490,8205,129490],[129489,127995,8205,9877,65039],[129489,127995,8205,9878,65039],[129489,127995,8205,9992,65039],[129489,127996,8205,9877,65039],[129489,127996,8205,9878,65039],[129489,127996,8205,9992,65039],[129489,127997,8205,9877,65039],[129489,127997,8205,9878,65039],[129489,127997,8205,9992,65039],[129489,127998,8205,9877,65039],[129489,127998,8205,9878,65039],[129489,127998,8205,9992,65039],[129489,127999,8205,9877,65039],[129489,127999,8205,9878,65039],[129489,127999,8205,9992,65039],[129492,127995,8205,9792,65039],[129492,127995,8205,9794,65039],[129492,127996,8205,9792,65039],[129492,127996,8205,9794,65039],[129492,127997,8205,9792,65039],[129492,127997,8205,9794,65039],[129492,127998,8205,9792,65039],[129492,127998,8205,9794,65039],[129492,127999,8205,9792,65039],[129492,127999,8205,9794,65039],[129494,127995,8205,9792,65039],[129494,127995,8205,9794,65039],[129494,127996,8205,9792,65039],[129494,127996,8205,9794,65039],[129494,127997,8205,9792,65039],[129494,127997,8205,9794,65039],[129494,127998,8205,9792,65039],[129494,127998,8205,9794,65039],[129494,127999,8205,9792,65039],[129494,127999,8205,9794,65039],[129495,127995,8205,9792,65039],[129495,127995,8205,9794,65039],[129495,127996,8205,9792,65039],[129495,127996,8205,9794,65039],[129495,127997,8205,9792,65039],[129495,127997,8205,9794,65039],[129495,127998,8205,9792,65039],[129495,127998,8205,9794,65039],[129495,127999,8205,9792,65039],[129495,127999,8205,9794,65039],[129496,127995,8205,9792,65039],[129496,127995,8205,9794,65039],[129496,127996,8205,9792,65039],[129496,127996,8205,9794,65039],[129496,127997,8205,9792,65039],[129496,127997,8205,9794,65039],[129496,127998,8205,9792,65039],[129496,127998,8205,9794,65039],[129496,127999,8205,9792,65039],[129496,127999,8205,9794,65039],[129497,127995,8205,9792,65039],[129497,127995,8205,9794,65039],[129497,127996,8205,9792,65039],[129497,127996,8205,9794,65039],[129497,127997,8205,9792,65039],[129497,127997,8205,9794,65039],[129497,127998,8205,9792,65039],[129497,127998,8205,9794,65039],[129497,127999,8205,9792,65039],[129497,127999,8205,9794,65039],[129498,127995,8205,9792,65039],[129498,127995,8205,9794,65039],[129498,127996,8205,9792,65039],[129498,127996,8205,9794,65039],[129498,127997,8205,9792,65039],[129498,127997,8205,9794,65039],[129498,127998,8205,9792,65039],[129498,127998,8205,9794,65039],[129498,127999,8205,9792,65039],[129498,127999,8205,9794,65039],[129499,127995,8205,9792,65039],[129499,127995,8205,9794,65039],[129499,127996,8205,9792,65039],[129499,127996,8205,9794,65039],[129499,127997,8205,9792,65039],[129499,127997,8205,9794,65039],[129499,127998,8205,9792,65039],[129499,127998,8205,9794,65039],[129499,127999,8205,9792,65039],[129499,127999,8205,9794,65039],[129500,127995,8205,9792,65039],[129500,127995,8205,9794,65039],[129500,127996,8205,9792,65039],[129500,127996,8205,9794,65039],[129500,127997,8205,9792,65039],[129500,127997,8205,9794,65039],[129500,127998,8205,9792,65039],[129500,127998,8205,9794,65039],[129500,127999,8205,9792,65039],[129500,127999,8205,9794,65039],[129501,127995,8205,9792,65039],[129501,127995,8205,9794,65039],[129501,127996,8205,9792,65039],[129501,127996,8205,9794,65039],[129501,127997,8205,9792,65039],[129501,127997,8205,9794,65039],[129501,127998,8205,9792,65039],[129501,127998,8205,9794,65039],[129501,127999,8205,9792,65039],[129501,127999,8205,9794,65039],[129777,127995,8205,129778,127996],[129777,127995,8205,129778,127997],[129777,127995,8205,129778,127998],[129777,127995,8205,129778,127999],[129777,127996,8205,129778,127995],[129777,127996,8205,129778,127997],[129777,127996,8205,129778,127998],[129777,127996,8205,129778,127999],[129777,127997,8205,129778,127995],[129777,127997,8205,129778,127996],[129777,127997,8205,129778,127998],[129777,127997,8205,129778,127999],[129777,127998,8205,129778,127995],[129777,127998,8205,129778,127996],[129777,127998,8205,129778,127997],[129777,127998,8205,129778,127999],[129777,127999,8205,129778,127995],[129777,127999,8205,129778,127996],[129777,127999,8205,129778,127997],[129777,127999,8205,129778,127998],[9939,65039,8205,128165],[10084,65039,8205,128293],[10084,65039,8205,129657],[127939,8205,9792,65039],[127939,8205,9794,65039],[127939,8205,10145,65039],[127940,8205,9792,65039],[127940,8205,9794,65039],[127946,8205,9792,65039],[127946,8205,9794,65039],[127987,65039,8205,127752],[127988,8205,9760,65039],[128059,8205,10052,65039],[128104,8205,9877,65039],[128104,8205,9878,65039],[128104,8205,9992,65039],[128104,127995,8205,127806],[128104,127995,8205,127859],[128104,127995,8205,127868],[128104,127995,8205,127891],[128104,127995,8205,127908],[128104,127995,8205,127912],[128104,127995,8205,127979],[128104,127995,8205,127981],[128104,127995,8205,128187],[128104,127995,8205,128188],[128104,127995,8205,128295],[128104,127995,8205,128300],[128104,127995,8205,128640],[128104,127995,8205,128658],[128104,127995,8205,129455],[128104,127995,8205,129456],[128104,127995,8205,129457],[128104,127995,8205,129458],[128104,127995,8205,129459],[128104,127995,8205,129468],[128104,127995,8205,129469],[128104,127996,8205,127806],[128104,127996,8205,127859],[128104,127996,8205,127868],[128104,127996,8205,127891],[128104,127996,8205,127908],[128104,127996,8205,127912],[128104,127996,8205,127979],[128104,127996,8205,127981],[128104,127996,8205,128187],[128104,127996,8205,128188],[128104,127996,8205,128295],[128104,127996,8205,128300],[128104,127996,8205,128640],[128104,127996,8205,128658],[128104,127996,8205,129455],[128104,127996,8205,129456],[128104,127996,8205,129457],[128104,127996,8205,129458],[128104,127996,8205,129459],[128104,127996,8205,129468],[128104,127996,8205,129469],[128104,127997,8205,127806],[128104,127997,8205,127859],[128104,127997,8205,127868],[128104,127997,8205,127891],[128104,127997,8205,127908],[128104,127997,8205,127912],[128104,127997,8205,127979],[128104,127997,8205,127981],[128104,127997,8205,128187],[128104,127997,8205,128188],[128104,127997,8205,128295],[128104,127997,8205,128300],[128104,127997,8205,128640],[128104,127997,8205,128658],[128104,127997,8205,129455],[128104,127997,8205,129456],[128104,127997,8205,129457],[128104,127997,8205,129458],[128104,127997,8205,129459],[128104,127997,8205,129468],[128104,127997,8205,129469],[128104,127998,8205,127806],[128104,127998,8205,127859],[128104,127998,8205,127868],[128104,127998,8205,127891],[128104,127998,8205,127908],[128104,127998,8205,127912],[128104,127998,8205,127979],[128104,127998,8205,127981],[128104,127998,8205,128187],[128104,127998,8205,128188],[128104,127998,8205,128295],[128104,127998,8205,128300],[128104,127998,8205,128640],[128104,127998,8205,128658],[128104,127998,8205,129455],[128104,127998,8205,129456],[128104,127998,8205,129457],[128104,127998,8205,129458],[128104,127998,8205,129459],[128104,127998,8205,129468],[128104,127998,8205,129469],[128104,127999,8205,127806],[128104,127999,8205,127859],[128104,127999,8205,127868],[128104,127999,8205,127891],[128104,127999,8205,127908],[128104,127999,8205,127912],[128104,127999,8205,127979],[128104,127999,8205,127981],[128104,127999,8205,128187],[128104,127999,8205,128188],[128104,127999,8205,128295],[128104,127999,8205,128300],[128104,127999,8205,128640],[128104,127999,8205,128658],[128104,127999,8205,129455],[128104,127999,8205,129456],[128104,127999,8205,129457],[128104,127999,8205,129458],[128104,127999,8205,129459],[128104,127999,8205,129468],[128104,127999,8205,129469],[128105,8205,9877,65039],[128105,8205,9878,65039],[128105,8205,9992,65039],[128105,127995,8205,127806],[128105,127995,8205,127859],[128105,127995,8205,127868],[128105,127995,8205,127891],[128105,127995,8205,127908],[128105,127995,8205,127912],[128105,127995,8205,127979],[128105,127995,8205,127981],[128105,127995,8205,128187],[128105,127995,8205,128188],[128105,127995,8205,128295],[128105,127995,8205,128300],[128105,127995,8205,128640],[128105,127995,8205,128658],[128105,127995,8205,129455],[128105,127995,8205,129456],[128105,127995,8205,129457],[128105,127995,8205,129458],[128105,127995,8205,129459],[128105,127995,8205,129468],[128105,127995,8205,129469],[128105,127996,8205,127806],[128105,127996,8205,127859],[128105,127996,8205,127868],[128105,127996,8205,127891],[128105,127996,8205,127908],[128105,127996,8205,127912],[128105,127996,8205,127979],[128105,127996,8205,127981],[128105,127996,8205,128187],[128105,127996,8205,128188],[128105,127996,8205,128295],[128105,127996,8205,128300],[128105,127996,8205,128640],[128105,127996,8205,128658],[128105,127996,8205,129455],[128105,127996,8205,129456],[128105,127996,8205,129457],[128105,127996,8205,129458],[128105,127996,8205,129459],[128105,127996,8205,129468],[128105,127996,8205,129469],[128105,127997,8205,127806],[128105,127997,8205,127859],[128105,127997,8205,127868],[128105,127997,8205,127891],[128105,127997,8205,127908],[128105,127997,8205,127912],[128105,127997,8205,127979],[128105,127997,8205,127981],[128105,127997,8205,128187],[128105,127997,8205,128188],[128105,127997,8205,128295],[128105,127997,8205,128300],[128105,127997,8205,128640],[128105,127997,8205,128658],[128105,127997,8205,129455],[128105,127997,8205,129456],[128105,127997,8205,129457],[128105,127997,8205,129458],[128105,127997,8205,129459],[128105,127997,8205,129468],[128105,127997,8205,129469],[128105,127998,8205,127806],[128105,127998,8205,127859],[128105,127998,8205,127868],[128105,127998,8205,127891],[128105,127998,8205,127908],[128105,127998,8205,127912],[128105,127998,8205,127979],[128105,127998,8205,127981],[128105,127998,8205,128187],[128105,127998,8205,128188],[128105,127998,8205,128295],[128105,127998,8205,128300],[128105,127998,8205,128640],[128105,127998,8205,128658],[128105,127998,8205,129455],[128105,127998,8205,129456],[128105,127998,8205,129457],[128105,127998,8205,129458],[128105,127998,8205,129459],[128105,127998,8205,129468],[128105,127998,8205,129469],[128105,127999,8205,127806],[128105,127999,8205,127859],[128105,127999,8205,127868],[128105,127999,8205,127891],[128105,127999,8205,127908],[128105,127999,8205,127912],[128105,127999,8205,127979],[128105,127999,8205,127981],[128105,127999,8205,128187],[128105,127999,8205,128188],[128105,127999,8205,128295],[128105,127999,8205,128300],[128105,127999,8205,128640],[128105,127999,8205,128658],[128105,127999,8205,129455],[128105,127999,8205,129456],[128105,127999,8205,129457],[128105,127999,8205,129458],[128105,127999,8205,129459],[128105,127999,8205,129468],[128105,127999,8205,129469],[128110,8205,9792,65039],[128110,8205,9794,65039],[128111,8205,9792,65039],[128111,8205,9794,65039],[128112,8205,9792,65039],[128112,8205,9794,65039],[128113,8205,9792,65039],[128113,8205,9794,65039],[128115,8205,9792,65039],[128115,8205,9794,65039],[128119,8205,9792,65039],[128119,8205,9794,65039],[128129,8205,9792,65039],[128129,8205,9794,65039],[128130,8205,9792,65039],[128130,8205,9794,65039],[128134,8205,9792,65039],[128134,8205,9794,65039],[128135,8205,9792,65039],[128135,8205,9794,65039],[128566,8205,127787,65039],[128578,8205,8596,65039],[128578,8205,8597,65039],[128581,8205,9792,65039],[128581,8205,9794,65039],[128582,8205,9792,65039],[128582,8205,9794,65039],[128583,8205,9792,65039],[128583,8205,9794,65039],[128587,8205,9792,65039],[128587,8205,9794,65039],[128589,8205,9792,65039],[128589,8205,9794,65039],[128590,8205,9792,65039],[128590,8205,9794,65039],[128675,8205,9792,65039],[128675,8205,9794,65039],[128692,8205,9792,65039],[128692,8205,9794,65039],[128693,8205,9792,65039],[128693,8205,9794,65039],[128694,8205,9792,65039],[128694,8205,9794,65039],[128694,8205,10145,65039],[129318,8205,9792,65039],[129318,8205,9794,65039],[129333,8205,9792,65039],[129333,8205,9794,65039],[129335,8205,9792,65039],[129335,8205,9794,65039],[129336,8205,9792,65039],[129336,8205,9794,65039],[129337,8205,9792,65039],[129337,8205,9794,65039],[129340,8205,9792,65039],[129340,8205,9794,65039],[129341,8205,9792,65039],[129341,8205,9794,65039],[129342,8205,9792,65039],[129342,8205,9794,65039],[129464,8205,9792,65039],[129464,8205,9794,65039],[129465,8205,9792,65039],[129465,8205,9794,65039],[129485,8205,9792,65039],[129485,8205,9794,65039],[129486,8205,9792,65039],[129486,8205,9794,65039],[129486,8205,10145,65039],[129487,8205,9792,65039],[129487,8205,9794,65039],[129489,8205,9877,65039],[129489,8205,9878,65039],[129489,8205,9992,65039],[129489,127995,8205,127806],[129489,127995,8205,127859],[129489,127995,8205,127868],[129489,127995,8205,127876],[129489,127995,8205,127891],[129489,127995,8205,127908],[129489,127995,8205,127912],[129489,127995,8205,127979],[129489,127995,8205,127981],[129489,127995,8205,128187],[129489,127995,8205,128188],[129489,127995,8205,128295],[129489,127995,8205,128300],[129489,127995,8205,128640],[129489,127995,8205,128658],[129489,127995,8205,129455],[129489,127995,8205,129456],[129489,127995,8205,129457],[129489,127995,8205,129458],[129489,127995,8205,129459],[129489,127995,8205,129468],[129489,127995,8205,129469],[129489,127995,8205,129648],[129489,127996,8205,127806],[129489,127996,8205,127859],[129489,127996,8205,127868],[129489,127996,8205,127876],[129489,127996,8205,127891],[129489,127996,8205,127908],[129489,127996,8205,127912],[129489,127996,8205,127979],[129489,127996,8205,127981],[129489,127996,8205,128187],[129489,127996,8205,128188],[129489,127996,8205,128295],[129489,127996,8205,128300],[129489,127996,8205,128640],[129489,127996,8205,128658],[129489,127996,8205,129455],[129489,127996,8205,129456],[129489,127996,8205,129457],[129489,127996,8205,129458],[129489,127996,8205,129459],[129489,127996,8205,129468],[129489,127996,8205,129469],[129489,127996,8205,129648],[129489,127997,8205,127806],[129489,127997,8205,127859],[129489,127997,8205,127868],[129489,127997,8205,127876],[129489,127997,8205,127891],[129489,127997,8205,127908],[129489,127997,8205,127912],[129489,127997,8205,127979],[129489,127997,8205,127981],[129489,127997,8205,128187],[129489,127997,8205,128188],[129489,127997,8205,128295],[129489,127997,8205,128300],[129489,127997,8205,128640],[129489,127997,8205,128658],[129489,127997,8205,129455],[129489,127997,8205,129456],[129489,127997,8205,129457],[129489,127997,8205,129458],[129489,127997,8205,129459],[129489,127997,8205,129468],[129489,127997,8205,129469],[129489,127997,8205,129648],[129489,127998,8205,127806],[129489,127998,8205,127859],[129489,127998,8205,127868],[129489,127998,8205,127876],[129489,127998,8205,127891],[129489,127998,8205,127908],[129489,127998,8205,127912],[129489,127998,8205,127979],[129489,127998,8205,127981],[129489,127998,8205,128187],[129489,127998,8205,128188],[129489,127998,8205,128295],[129489,127998,8205,128300],[129489,127998,8205,128640],[129489,127998,8205,128658],[129489,127998,8205,129455],[129489,127998,8205,129456],[129489,127998,8205,129457],[129489,127998,8205,129458],[129489,127998,8205,129459],[129489,127998,8205,129468],[129489,127998,8205,129469],[129489,127998,8205,129648],[129489,127999,8205,127806],[129489,127999,8205,127859],[129489,127999,8205,127868],[129489,127999,8205,127876],[129489,127999,8205,127891],[129489,127999,8205,127908],[129489,127999,8205,127912],[129489,127999,8205,127979],[129489,127999,8205,127981],[129489,127999,8205,128187],[129489,127999,8205,128188],[129489,127999,8205,128295],[129489,127999,8205,128300],[129489,127999,8205,128640],[129489,127999,8205,128658],[129489,127999,8205,129455],[129489,127999,8205,129456],[129489,127999,8205,129457],[129489,127999,8205,129458],[129489,127999,8205,129459],[129489,127999,8205,129468],[129489,127999,8205,129469],[129489,127999,8205,129648],[129492,8205,9792,65039],[129492,8205,9794,65039],[129494,8205,9792,65039],[129494,8205,9794,65039],[129495,8205,9792,65039],[129495,8205,9794,65039],[129496,8205,9792,65039],[129496,8205,9794,65039],[129497,8205,9792,65039],[129497,8205,9794,65039],[129498,8205,9792,65039],[129498,8205,9794,65039],[129499,8205,9792,65039],[129499,8205,9794,65039],[129500,8205,9792,65039],[129500,8205,9794,65039],[129501,8205,9792,65039],[129501,8205,9794,65039],[129502,8205,9792,65039],[129502,8205,9794,65039],[129503,8205,9792,65039],[129503,8205,9794,65039],[35,65039,8419],[42,65039,8419],[48,65039,8419],[49,65039,8419],[50,65039,8419],[51,65039,8419],[52,65039,8419],[53,65039,8419],[54,65039,8419],[55,65039,8419],[56,65039,8419],[57,65039,8419],[127812,8205,129003],[127819,8205,129001],[128008,8205,11035],[128021,8205,129466],[128038,8205,11035],[128038,8205,128293],[128104,8205,127806],[128104,8205,127859],[128104,8205,127868],[128104,8205,127891],[128104,8205,127908],[128104,8205,127912],[128104,8205,127979],[128104,8205,127981],[128104,8205,128102],[128104,8205,128103],[128104,8205,128187],[128104,8205,128188],[128104,8205,128295],[128104,8205,128300],[128104,8205,128640],[128104,8205,128658],[128104,8205,129455],[128104,8205,129456],[128104,8205,129457],[128104,8205,129458],[128104,8205,129459],[128104,8205,129468],[128104,8205,129469],[128105,8205,127806],[128105,8205,127859],[128105,8205,127868],[128105,8205,127891],[128105,8205,127908],[128105,8205,127912],[128105,8205,127979],[128105,8205,127981],[128105,8205,128102],[128105,8205,128103],[128105,8205,128187],[128105,8205,128188],[128105,8205,128295],[128105,8205,128300],[128105,8205,128640],[128105,8205,128658],[128105,8205,129455],[128105,8205,129456],[128105,8205,129457],[128105,8205,129458],[128105,8205,129459],[128105,8205,129468],[128105,8205,129469],[128558,8205,128168],[128565,8205,128171],[129489,8205,127806],[129489,8205,127859],[129489,8205,127868],[129489,8205,127876],[129489,8205,127891],[129489,8205,127908],[129489,8205,127912],[129489,8205,127979],[129489,8205,127981],[129489,8205,128187],[129489,8205,128188],[129489,8205,128295],[129489,8205,128300],[129489,8205,128640],[129489,8205,128658],[129489,8205,129455],[129489,8205,129456],[129489,8205,129457],[129489,8205,129458],[129489,8205,129459],[129489,8205,129468],[129489,8205,129469],[129489,8205,129490],[129489,8205,129648],[169,65039],[174,65039],[8252,65039],[8265,65039],[8482,65039],[8505,65039],[8596,65039],[8597,65039],[8598,65039],[8599,65039],[8600,65039],[8601,65039],[8617,65039],[8618,65039],[9000,65039],[9167,65039],[9197,65039],[9198,65039],[9199,65039],[9201,65039],[9202,65039],[9208,65039],[9209,65039],[9210,65039],[9410,65039],[9642,65039],[9643,65039],[9654,65039],[9664,65039],[9723,65039],[9724,65039],[9728,65039],[9729,65039],[9730,65039],[9731,65039],[9732,65039],[9742,65039],[9745,65039],[9752,65039],[9757,65039],[9757,127995],[9757,127996],[9757,127997],[9757,127998],[9757,127999],[9760,65039],[9762,65039],[9763,65039],[9766,65039],[9770,65039],[9774,65039],[9775,65039],[9784,65039],[9785,65039],[9786,65039],[9792,65039],[9794,65039],[9823,65039],[9824,65039],[9827,65039],[9829,65039],[9830,65039],[9832,65039],[9851,65039],[9854,65039],[9874,65039],[9876,65039],[9877,65039],[9878,65039],[9879,65039],[9881,65039],[9883,65039],[9884,65039],[9888,65039],[9895,65039],[9904,65039],[9905,65039],[9928,65039],[9935,65039],[9937,65039],[9939,65039],[9961,65039],[9968,65039],[9969,65039],[9972,65039],[9975,65039],[9976,65039],[9977,65039],[9977,127995],[9977,127996],[9977,127997],[9977,127998],[9977,127999],[9986,65039],[9992,65039],[9993,65039],[9994,127995],[9994,127996],[9994,127997],[9994,127998],[9994,127999],[9995,127995],[9995,127996],[9995,127997],[9995,127998],[9995,127999],[9996,65039],[9996,127995],[9996,127996],[9996,127997],[9996,127998],[9996,127999],[9997,65039],[9997,127995],[9997,127996],[9997,127997],[9997,127998],[9997,127999],[9999,65039],[10002,65039],[10004,65039],[10006,65039],[10013,65039],[10017,65039],[10035,65039],[10036,65039],[10052,65039],[10055,65039],[10083,65039],[10084,65039],[10145,65039],[10548,65039],[10549,65039],[11013,65039],[11014,65039],[11015,65039],[12336,65039],[12349,65039],[12951,65039],[12953,65039],[127344,65039],[127345,65039],[127358,65039],[127359,65039],[127462,127464],[127462,127465],[127462,127466],[127462,127467],[127462,127468],[127462,127470],[127462,127473],[127462,127474],[127462,127476],[127462,127478],[127462,127479],[127462,127480],[127462,127481],[127462,127482],[127462,127484],[127462,127485],[127462,127487],[127463,127462],[127463,127463],[127463,127465],[127463,127466],[127463,127467],[127463,127468],[127463,127469],[127463,127470],[127463,127471],[127463,127473],[127463,127474],[127463,127475],[127463,127476],[127463,127478],[127463,127479],[127463,127480],[127463,127481],[127463,127483],[127463,127484],[127463,127486],[127463,127487],[127464,127462],[127464,127464],[127464,127465],[127464,127467],[127464,127468],[127464,127469],[127464,127470],[127464,127472],[127464,127473],[127464,127474],[127464,127475],[127464,127476],[127464,127477],[127464,127478],[127464,127479],[127464,127482],[127464,127483],[127464,127484],[127464,127485],[127464,127486],[127464,127487],[127465,127466],[127465,127468],[127465,127471],[127465,127472],[127465,127474],[127465,127476],[127465,127487],[127466,127462],[127466,127464],[127466,127466],[127466,127468],[127466,127469],[127466,127479],[127466,127480],[127466,127481],[127466,127482],[127467,127470],[127467,127471],[127467,127472],[127467,127474],[127467,127476],[127467,127479],[127468,127462],[127468,127463],[127468,127465],[127468,127466],[127468,127467],[127468,127468],[127468,127469],[127468,127470],[127468,127473],[127468,127474],[127468,127475],[127468,127477],[127468,127478],[127468,127479],[127468,127480],[127468,127481],[127468,127482],[127468,127484],[127468,127486],[127469,127472],[127469,127474],[127469,127475],[127469,127479],[127469,127481],[127469,127482],[127470,127464],[127470,127465],[127470,127466],[127470,127473],[127470,127474],[127470,127475],[127470,127476],[127470,127478],[127470,127479],[127470,127480],[127470,127481],[127471,127466],[127471,127474],[127471,127476],[127471,127477],[127472,127466],[127472,127468],[127472,127469],[127472,127470],[127472,127474],[127472,127475],[127472,127477],[127472,127479],[127472,127484],[127472,127486],[127472,127487],[127473,127462],[127473,127463],[127473,127464],[127473,127470],[127473,127472],[127473,127479],[127473,127480],[127473,127481],[127473,127482],[127473,127483],[127473,127486],[127474,127462],[127474,127464],[127474,127465],[127474,127466],[127474,127467],[127474,127468],[127474,127469],[127474,127472],[127474,127473],[127474,127474],[127474,127475],[127474,127476],[127474,127477],[127474,127478],[127474,127479],[127474,127480],[127474,127481],[127474,127482],[127474,127483],[127474,127484],[127474,127485],[127474,127486],[127474,127487],[127475,127462],[127475,127464],[127475,127466],[127475,127467],[127475,127468],[127475,127470],[127475,127473],[127475,127476],[127475,127477],[127475,127479],[127475,127482],[127475,127487],[127476,127474],[127477,127462],[127477,127466],[127477,127467],[127477,127468],[127477,127469],[127477,127472],[127477,127473],[127477,127474],[127477,127475],[127477,127479],[127477,127480],[127477,127481],[127477,127484],[127477,127486],[127478,127462],[127479,127466],[127479,127476],[127479,127480],[127479,127482],[127479,127484],[127480,127462],[127480,127463],[127480,127464],[127480,127465],[127480,127466],[127480,127468],[127480,127469],[127480,127470],[127480,127471],[127480,127472],[127480,127473],[127480,127474],[127480,127475],[127480,127476],[127480,127479],[127480,127480],[127480,127481],[127480,127483],[127480,127485],[127480,127486],[127480,127487],[127481,127462],[127481,127464],[127481,127465],[127481,127467],[127481,127468],[127481,127469],[127481,127471],[127481,127472],[127481,127473],[127481,127474],[127481,127475],[127481,127476],[127481,127479],[127481,127481],[127481,127483],[127481,127484],[127481,127487],[127482,127462],[127482,127468],[127482,127474],[127482,127475],[127482,127480],[127482,127486],[127482,127487],[127483,127462],[127483,127464],[127483,127466],[127483,127468],[127483,127470],[127483,127475],[127483,127482],[127484,127467],[127484,127480],[127485,127472],[127486,127466],[127486,127481],[127487,127462],[127487,127474],[127487,127484],[127490,65039],[127543,65039],[127777,65039],[127780,65039],[127781,65039],[127782,65039],[127783,65039],[127784,65039],[127785,65039],[127786,65039],[127787,65039],[127788,65039],[127798,65039],[127869,65039],[127877,127995],[127877,127996],[127877,127997],[127877,127998],[127877,127999],[127894,65039],[127895,65039],[127897,65039],[127898,65039],[127899,65039],[127902,65039],[127903,65039],[127938,127995],[127938,127996],[127938,127997],[127938,127998],[127938,127999],[127939,127995],[127939,127996],[127939,127997],[127939,127998],[127939,127999],[127940,127995],[127940,127996],[127940,127997],[127940,127998],[127940,127999],[127943,127995],[127943,127996],[127943,127997],[127943,127998],[127943,127999],[127946,127995],[127946,127996],[127946,127997],[127946,127998],[127946,127999],[127947,65039],[127947,127995],[127947,127996],[127947,127997],[127947,127998],[127947,127999],[127948,65039],[127948,127995],[127948,127996],[127948,127997],[127948,127998],[127948,127999],[127949,65039],[127950,65039],[127956,65039],[127957,65039],[127958,65039],[127959,65039],[127960,65039],[127961,65039],[127962,65039],[127963,65039],[127964,65039],[127965,65039],[127966,65039],[127967,65039],[127987,65039],[127989,65039],[127991,65039],[128063,65039],[128065,65039],[128066,127995],[128066,127996],[128066,127997],[128066,127998],[128066,127999],[128067,127995],[128067,127996],[128067,127997],[128067,127998],[128067,127999],[128070,127995],[128070,127996],[128070,127997],[128070,127998],[128070,127999],[128071,127995],[128071,127996],[128071,127997],[128071,127998],[128071,127999],[128072,127995],[128072,127996],[128072,127997],[128072,127998],[128072,127999],[128073,127995],[128073,127996],[128073,127997],[128073,127998],[128073,127999],[128074,127995],[128074,127996],[128074,127997],[128074,127998],[128074,127999],[128075,127995],[128075,127996],[128075,127997],[128075,127998],[128075,127999],[128076,127995],[128076,127996],[128076,127997],[128076,127998],[128076,127999],[128077,127995],[128077,127996],[128077,127997],[128077,127998],[128077,127999],[128078,127995],[128078,127996],[128078,127997],[128078,127998],[128078,127999],[128079,127995],[128079,127996],[128079,127997],[128079,127998],[128079,127999],[128080,127995],[128080,127996],[128080,127997],[128080,127998],[128080,127999],[128102,127995],[128102,127996],[128102,127997],[128102,127998],[128102,127999],[128103,127995],[128103,127996],[128103,127997],[128103,127998],[128103,127999],[128104,127995],[128104,127996],[128104,127997],[128104,127998],[128104,127999],[128105,127995],[128105,127996],[128105,127997],[128105,127998],[128105,127999],[128107,127995],[128107,127996],[128107,127997],[128107,127998],[128107,127999],[128108,127995],[128108,127996],[128108,127997],[128108,127998],[128108,127999],[128109,127995],[128109,127996],[128109,127997],[128109,127998],[128109,127999],[128110,127995],[128110,127996],[128110,127997],[128110,127998],[128110,127999],[128111,127995],[128111,127996],[128111,127997],[128111,127998],[128111,127999],[128112,127995],[128112,127996],[128112,127997],[128112,127998],[128112,127999],[128113,127995],[128113,127996],[128113,127997],[128113,127998],[128113,127999],[128114,127995],[128114,127996],[128114,127997],[128114,127998],[128114,127999],[128115,127995],[128115,127996],[128115,127997],[128115,127998],[128115,127999],[128116,127995],[128116,127996],[128116,127997],[128116,127998],[128116,127999],[128117,127995],[128117,127996],[128117,127997],[128117,127998],[128117,127999],[128118,127995],[128118,127996],[128118,127997],[128118,127998],[128118,127999],[128119,127995],[128119,127996],[128119,127997],[128119,127998],[128119,127999],[128120,127995],[128120,127996],[128120,127997],[128120,127998],[128120,127999],[128124,127995],[128124,127996],[128124,127997],[128124,127998],[128124,127999],[128129,127995],[128129,127996],[128129,127997],[128129,127998],[128129,127999],[128130,127995],[128130,127996],[128130,127997],[128130,127998],[128130,127999],[128131,127995],[128131,127996],[128131,127997],[128131,127998],[128131,127999],[128133,127995],[128133,127996],[128133,127997],[128133,127998],[128133,127999],[128134,127995],[128134,127996],[128134,127997],[128134,127998],[128134,127999],[128135,127995],[128135,127996],[128135,127997],[128135,127998],[128135,127999],[128143,127995],[128143,127996],[128143,127997],[128143,127998],[128143,127999],[128145,127995],[128145,127996],[128145,127997],[128145,127998],[128145,127999],[128170,127995],[128170,127996],[128170,127997],[128170,127998],[128170,127999],[128253,65039],[128329,65039],[128330,65039],[128367,65039],[128368,65039],[128371,65039],[128372,65039],[128372,127995],[128372,127996],[128372,127997],[128372,127998],[128372,127999],[128373,65039],[128373,127995],[128373,127996],[128373,127997],[128373,127998],[128373,127999],[128374,65039],[128375,65039],[128376,65039],[128377,65039],[128378,127995],[128378,127996],[128378,127997],[128378,127998],[128378,127999],[128391,65039],[128394,65039],[128395,65039],[128396,65039],[128397,65039],[128400,65039],[128400,127995],[128400,127996],[128400,127997],[128400,127998],[128400,127999],[128405,127995],[128405,127996],[128405,127997],[128405,127998],[128405,127999],[128406,127995],[128406,127996],[128406,127997],[128406,127998],[128406,127999],[128421,65039],[128424,65039],[128433,65039],[128434,65039],[128444,65039],[128450,65039],[128451,65039],[128452,65039],[128465,65039],[128466,65039],[128467,65039],[128476,65039],[128477,65039],[128478,65039],[128481,65039],[128483,65039],[128488,65039],[128495,65039],[128499,65039],[128506,65039],[128581,127995],[128581,127996],[128581,127997],[128581,127998],[128581,127999],[128582,127995],[128582,127996],[128582,127997],[128582,127998],[128582,127999],[128583,127995],[128583,127996],[128583,127997],[128583,127998],[128583,127999],[128587,127995],[128587,127996],[128587,127997],[128587,127998],[128587,127999],[128588,127995],[128588,127996],[128588,127997],[128588,127998],[128588,127999],[128589,127995],[128589,127996],[128589,127997],[128589,127998],[128589,127999],[128590,127995],[128590,127996],[128590,127997],[128590,127998],[128590,127999],[128591,127995],[128591,127996],[128591,127997],[128591,127998],[128591,127999],[128675,127995],[128675,127996],[128675,127997],[128675,127998],[128675,127999],[128692,127995],[128692,127996],[128692,127997],[128692,127998],[128692,127999],[128693,127995],[128693,127996],[128693,127997],[128693,127998],[128693,127999],[128694,127995],[128694,127996],[128694,127997],[128694,127998],[128694,127999],[128704,127995],[128704,127996],[128704,127997],[128704,127998],[128704,127999],[128715,65039],[128716,127995],[128716,127996],[128716,127997],[128716,127998],[128716,127999],[128717,65039],[128718,65039],[128719,65039],[128736,65039],[128737,65039],[128738,65039],[128739,65039],[128740,65039],[128741,65039],[128745,65039],[128752,65039],[128755,65039],[129292,127995],[129292,127996],[129292,127997],[129292,127998],[129292,127999],[129295,127995],[129295,127996],[129295,127997],[129295,127998],[129295,127999],[129304,127995],[129304,127996],[129304,127997],[129304,127998],[129304,127999],[129305,127995],[129305,127996],[129305,127997],[129305,127998],[129305,127999],[129306,127995],[129306,127996],[129306,127997],[129306,127998],[129306,127999],[129307,127995],[129307,127996],[129307,127997],[129307,127998],[129307,127999],[129308,127995],[129308,127996],[129308,127997],[129308,127998],[129308,127999],[129309,127995],[129309,127996],[129309,127997],[129309,127998],[129309,127999],[129310,127995],[129310,127996],[129310,127997],[129310,127998],[129310,127999],[129311,127995],[129311,127996],[129311,127997],[129311,127998],[129311,127999],[129318,127995],[129318,127996],[129318,127997],[129318,127998],[129318,127999],[129328,127995],[129328,127996],[129328,127997],[129328,127998],[129328,127999],[129329,127995],[129329,127996],[129329,127997],[129329,127998],[129329,127999],[129330,127995],[129330,127996],[129330,127997],[129330,127998],[129330,127999],[129331,127995],[129331,127996],[129331,127997],[129331,127998],[129331,127999],[129332,127995],[129332,127996],[129332,127997],[129332,127998],[129332,127999],[129333,127995],[129333,127996],[129333,127997],[129333,127998],[129333,127999],[129334,127995],[129334,127996],[129334,127997],[129334,127998],[129334,127999],[129335,127995],[129335,127996],[129335,127997],[129335,127998],[129335,127999],[129336,127995],[129336,127996],[129336,127997],[129336,127998],[129336,127999],[129337,127995],[129337,127996],[129337,127997],[129337,127998],[129337,127999],[129340,127995],[129340,127996],[129340,127997],[129340,127998],[129340,127999],[129341,127995],[129341,127996],[129341,127997],[129341,127998],[129341,127999],[129342,127995],[129342,127996],[129342,127997],[129342,127998],[129342,127999],[129399,127995],[129399,127996],[129399,127997],[129399,127998],[129399,127999],[129461,127995],[129461,127996],[129461,127997],[129461,127998],[129461,127999],[129462,127995],[129462,127996],[129462,127997],[129462,127998],[129462,127999],[129464,127995],[129464,127996],[129464,127997],[129464,127998],[129464,127999],[129465,127995],[129465,127996],[129465,127997],[129465,127998],[129465,127999],[129467,127995],[129467,127996],[129467,127997],[129467,127998],[129467,127999],[129485,127995],[129485,127996],[129485,127997],[129485,127998],[129485,127999],[129486,127995],[129486,127996],[129486,127997],[129486,127998],[129486,127999],[129487,127995],[129487,127996],[129487,127997],[129487,127998],[129487,127999],[129489,127995],[129489,127996],[129489,127997],[129489,127998],[129489,127999],[129490,127995],[129490,127996],[129490,127997],[129490,127998],[129490,127999],[129491,127995],[129491,127996],[129491,127997],[129491,127998],[129491,127999],[129492,127995],[129492,127996],[129492,127997],[129492,127998],[129492,127999],[129493,127995],[129493,127996],[129493,127997],[129493,127998],[129493,127999],[129494,127995],[129494,127996],[129494,127997],[129494,127998],[129494,127999],[129495,127995],[129495,127996],[129495,127997],[129495,127998],[129495,127999],[129496,127995],[129496,127996],[129496,127997],[129496,127998],[129496,127999],[129497,127995],[129497,127996],[129497,127997],[129497,127998],[129497,127999],[129498,127995],[129498,127996],[129498,127997],[129498,127998],[129498,127999],[129499,127995],[129499,127996],[129499,127997],[129499,127998],[129499,127999],[129500,127995],[129500,127996],[129500,127997],[129500,127998],[129500,127999],[129501,127995],[129501,127996],[129501,127997],[129501,127998],[129501,127999],[129731,127995],[129731,127996],[129731,127997],[129731,127998],[129731,127999],[129732,127995],[129732,127996],[129732,127997],[129732,127998],[129732,127999],[129733,127995],[129733,127996],[129733,127997],[129733,127998],[129733,127999],[129776,127995],[129776,127996],[129776,127997],[129776,127998],[129776,127999],[129777,127995],[129777,127996],[129777,127997],[129777,127998],[129777,127999],[129778,127995],[129778,127996],[129778,127997],[129778,127998],[129778,127999],[129779,127995],[129779,127996],[129779,127997],[129779,127998],[129779,127999],[129780,127995],[129780,127996],[129780,127997],[129780,127998],[129780,127999],[129781,127995],[129781,127996],[129781,127997],[129781,127998],[129781,127999],[129782,127995],[129782,127996],[129782,127997],[129782,127998],[129782,127999],[129783,127995],[129783,127996],[129783,127997],[129783,127998],[129783,127999],[129784,127995],[129784,127996],[129784,127997],[129784,127998],[129784,127999]],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[55357,56425,8205,55357,56507]],"from":0,"indices":[[0,5]],"to":5}],"rust_output":"👩‍💻"},{"case":{"flags":"g","input":"Ab AB ab","matches":[{"capture_units":[[65,98]],"from":0,"indices":[[0,2]],"name_units":null,"to":2},{"capture_units":[[97,98]],"from":6,"indices":[[6,8]],"name_units":null,"to":8}],"name":"scoped-modifiers","pattern":"(?i:a)(?-i:b)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[65,65],[97,97]],[],true,false],["set",[[98,98]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[65,98]],"from":0,"indices":[[0,2]],"to":2},{"capture_units":[[97,98]],"from":6,"indices":[[6,8]],"to":8}],"rust_output":"Ab AB ab"},{"case":{"flags":"g","input":"a\nb a\rb","matches":[{"capture_units":[[97,10,98]],"from":0,"indices":[[0,3]],"name_units":null,"to":3},{"capture_units":[[97,13,98]],"from":4,"indices":[[4,7]],"name_units":null,"to":7}],"name":"scoped-dotall","pattern":"a(?s:.)b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[97,97]],[],false,false],["set",[[0,65535]],[],false,false],["set",[[98,98]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,10,98]],"from":0,"indices":[[0,3]],"to":3},{"capture_units":[[97,13,98]],"from":4,"indices":[[4,7]],"to":7}],"rust_output":"a\nb a\rb"},{"case":{"flags":"g","input":"ab","matches":[{"capture_units":[[97],[97],null],"from":0,"indices":[[0,1],[0,1],null],"name_units":{"x":[97]},"to":1},{"capture_units":[[98],null,[98]],"from":1,"indices":[[1,2],null,[1,2]],"name_units":{"x":[98]},"to":2}],"name":"duplicate-disjoint-names","pattern":"(?<x>a)|(?<x>b)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["split",1,5],["mark",1],["set",[[97,97]],[],false,false],["capture",1],["jump",8],["mark",2],["set",[[98,98]],[],false,false],["capture",2],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":2,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97],null],"from":0,"indices":[[0,1],[0,1],null],"to":1},{"capture_units":[[98],null,[98]],"from":1,"indices":[[1,2],null,[1,2]],"to":2}],"rust_output":"ab"},{"case":{"flags":"g","input":"aa bb ab","matches":[{"capture_units":[[97,97],[97],null],"from":0,"indices":[[0,2],[0,1],null],"name_units":{"x":[97]},"to":2},{"capture_units":[[98,98],null,[98]],"from":3,"indices":[[3,5],null,[3,4]],"name_units":{"x":[98]},"to":5}],"name":"named-reference-disjoint-names","pattern":"(?:(?<x>a)|(?<x>b))\\k<x>"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["split",1,5],["mark",1],["set",[[97,97]],[],false,false],["capture",1],["jump",8],["mark",2],["set",[[98,98]],[],false,false],["capture",2],["ref",[1,2],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":2,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97],null],"from":0,"indices":[[0,2],[0,1],null],"to":2},{"capture_units":[[98,98],null,[98]],"from":3,"indices":[[3,5],null,[3,4]],"to":5}],"rust_output":"aa bb ab"},{"case":{"flags":"y","input":"ba","matches":[],"name":"sticky-start-failure","pattern":"a"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[97,97]],[],false,false],["accept"]],"flags":"y","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"ba"},{"case":{"flags":"gy","input":"aaba","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1},{"capture_units":[[97]],"from":1,"indices":[[1,2]],"name_units":null,"to":2}],"name":"sticky-global","pattern":"a"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[97,97]],[],false,false],["accept"]],"flags":"gy","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1},{"capture_units":[[97]],"from":1,"indices":[[1,2]],"to":2}],"rust_output":"aaba"},{"case":{"flags":"dg","input":"aa","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1},{"capture_units":[[97],[97]],"from":1,"indices":[[1,2],[1,2]],"name_units":null,"to":2}],"name":"indices-flag-no-text-effect","pattern":"(a)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["set",[[97,97]],[],false,false],["capture",1],["accept"]],"flags":"dg","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1},{"capture_units":[[97],[97]],"from":1,"indices":[[1,2],[1,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"g","input":"a\r\n  b","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1},{"capture_units":[[98]],"from":5,"indices":[[5,6]],"name_units":null,"to":6}],"name":"line-terminators","pattern":"."},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,9],[11,12],[14,8231],[8234,65535]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1},{"capture_units":[[98]],"from":5,"indices":[[5,6]],"to":6}],"rust_output":"a\r\n  b"},{"case":{"flags":"","input":"a\n","matches":[],"name":"dollar-no-final-newline-special","pattern":"a$"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[97,97]],[],false,false],["assert","end",false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"a\n"},{"case":{"flags":"gm","input":"a\r\nb","matches":[{"capture_units":[[]],"from":0,"indices":[[0,0]],"name_units":null,"to":0},{"capture_units":[[]],"from":1,"indices":[[1,1]],"name_units":null,"to":1},{"capture_units":[[]],"from":2,"indices":[[2,2]],"name_units":null,"to":2},{"capture_units":[[]],"from":3,"indices":[[3,3]],"name_units":null,"to":3},{"capture_units":[[]],"from":4,"indices":[[4,4]],"name_units":null,"to":4}],"name":"multiline-crlf-boundaries","pattern":"^|$"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["split",1,3],["assert","start",true,false],["jump",4],["assert","end",true,false],["accept"]],"flags":"gm","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[]],"from":0,"indices":[[0,0]],"to":0},{"capture_units":[[]],"from":1,"indices":[[1,1]],"to":1},{"capture_units":[[]],"from":2,"indices":[[2,2]],"to":2},{"capture_units":[[]],"from":3,"indices":[[3,3]],"to":3},{"capture_units":[[]],"from":4,"indices":[[4,4]],"to":4}],"rust_output":"a\r\nb"},{"case":{"flags":"iu","input":"kK","matches":[{"capture_units":[[107,8490],[107]],"from":0,"indices":[[0,2],[0,1]],"name_units":null,"to":2}],"name":"unicode-insensitive-backreference","pattern":"(k)\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["set",[[75,75],[107,107],[8490,8490]],[],true,true],["capture",1],["ref",[1],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[107,8490],[107]],"from":0,"indices":[[0,2],[0,1]],"to":2}],"rust_output":"kK"},{"case":{"flags":"i","input":"kK","matches":[],"name":"legacy-insensitive-backreference","pattern":"(k)\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["set",[[75,75],[107,107]],[],true,false],["capture",1],["ref",[1],true,false],["accept"]],"flags":"i","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"kK"},{"case":{"flags":"g","input":"ab","matches":[{"capture_units":[[97],[97],null],"from":0,"indices":[[0,1],[0,1],null],"name_units":null,"to":1},{"capture_units":[[98],null,[98]],"from":1,"indices":[[1,2],null,[1,2]],"name_units":null,"to":2}],"name":"unicode-unmatched-alternative","pattern":"(a)|(b)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["split",1,5],["mark",1],["set",[[97,97]],[],false,false],["capture",1],["jump",8],["mark",2],["set",[[98,98]],[],false,false],["capture",2],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":2,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97],null],"from":0,"indices":[[0,1],[0,1],null],"to":1},{"capture_units":[[98],null,[98]],"from":1,"indices":[[1,2],null,[1,2]],"to":2}],"rust_output":"ab"},{"case":{"flags":"gu","input":"😀","matches":[{"capture_units":[[]],"from":0,"indices":[[0,0]],"name_units":null,"to":0},{"capture_units":[[]],"from":2,"indices":[[2,2]],"name_units":null,"to":2}],"name":"unicode-empty-global","pattern":"(?:)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["accept"]],"flags":"gu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[]],"from":0,"indices":[[0,0]],"to":0},{"capture_units":[[]],"from":2,"indices":[[2,2]],"to":2}],"rust_output":"😀"},{"case":{"flags":"g","input":"😀","matches":[{"capture_units":[[]],"from":0,"indices":[[0,0]],"name_units":null,"to":0},{"capture_units":[[]],"from":1,"indices":[[1,1]],"name_units":null,"to":1},{"capture_units":[[]],"from":2,"indices":[[2,2]],"name_units":null,"to":2}],"name":"legacy-empty-global","pattern":"(?:)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[]],"from":0,"indices":[[0,0]],"to":0},{"capture_units":[[]],"from":1,"indices":[[1,1]],"to":1},{"capture_units":[[]],"from":2,"indices":[[2,2]],"to":2}],"rust_output":"😀"},{"case":{"flags":"","input":"","matches":[],"name":"generated-0--0","pattern":"(aa|a)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,6],["set",[[97,97]],[],false,false],["set",[[97,97]],[],false,false],["jump",7],["set",[[97,97]],[],false,false],["capture",1],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"name_units":null,"to":2}],"name":"generated-0--2","pattern":"(aa|a)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,6],["set",[[97,97]],[],false,false],["set",[[97,97]],[],false,false],["jump",7],["set",[[97,97]],[],false,false],["capture",1],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-0--4","pattern":"(aa|a)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,6],["set",[[97,97]],[],false,false],["set",[[97,97]],[],false,false],["jump",7],["set",[[97,97]],[],false,false],["capture",1],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-0--7","pattern":"(aa|a)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,6],["set",[[97,97]],[],false,false],["set",[[97,97]],[],false,false],["jump",7],["set",[[97,97]],[],false,false],["capture",1],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[],"name":"generated-0-g-0","pattern":"(aa|a)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,6],["set",[[97,97]],[],false,false],["set",[[97,97]],[],false,false],["jump",7],["set",[[97,97]],[],false,false],["capture",1],["repeat_end",0,0],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"name_units":null,"to":2}],"name":"generated-0-g-2","pattern":"(aa|a)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,6],["set",[[97,97]],[],false,false],["set",[[97,97]],[],false,false],["jump",7],["set",[[97,97]],[],false,false],["capture",1],["repeat_end",0,0],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1},{"capture_units":[[97],[97]],"from":2,"indices":[[2,3],[2,3]],"name_units":null,"to":3}],"name":"generated-0-g-4","pattern":"(aa|a)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,6],["set",[[97,97]],[],false,false],["set",[[97,97]],[],false,false],["jump",7],["set",[[97,97]],[],false,false],["capture",1],["repeat_end",0,0],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1},{"capture_units":[[97],[97]],"from":2,"indices":[[2,3],[2,3]],"to":3}],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-0-g-7","pattern":"(aa|a)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,6],["set",[[97,97]],[],false,false],["set",[[97,97]],[],false,false],["jump",7],["set",[[97,97]],[],false,false],["capture",1],["repeat_end",0,0],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"u","input":"","matches":[],"name":"generated-0-u-0","pattern":"(aa|a)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,6],["set",[[97,97]],[],false,true],["set",[[97,97]],[],false,true],["jump",7],["set",[[97,97]],[],false,true],["capture",1],["repeat_end",0,0],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"u","input":"aa","matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"name_units":null,"to":2}],"name":"generated-0-u-2","pattern":"(aa|a)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,6],["set",[[97,97]],[],false,true],["set",[[97,97]],[],false,true],["jump",7],["set",[[97,97]],[],false,true],["capture",1],["repeat_end",0,0],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"u","input":"abac","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-0-u-4","pattern":"(aa|a)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,6],["set",[[97,97]],[],false,true],["set",[[97,97]],[],false,true],["jump",7],["set",[[97,97]],[],false,true],["capture",1],["repeat_end",0,0],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"u","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-0-u-7","pattern":"(aa|a)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,6],["set",[[97,97]],[],false,true],["set",[[97,97]],[],false,true],["jump",7],["set",[[97,97]],[],false,true],["capture",1],["repeat_end",0,0],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"iu","input":"","matches":[],"name":"generated-0-iu-0","pattern":"(aa|a)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,6],["set",[[65,65],[97,97]],[],true,true],["set",[[65,65],[97,97]],[],true,true],["jump",7],["set",[[65,65],[97,97]],[],true,true],["capture",1],["repeat_end",0,0],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"iu","input":"aa","matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"name_units":null,"to":2}],"name":"generated-0-iu-2","pattern":"(aa|a)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,6],["set",[[65,65],[97,97]],[],true,true],["set",[[65,65],[97,97]],[],true,true],["jump",7],["set",[[65,65],[97,97]],[],true,true],["capture",1],["repeat_end",0,0],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"iu","input":"abac","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-0-iu-4","pattern":"(aa|a)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,6],["set",[[65,65],[97,97]],[],true,true],["set",[[65,65],[97,97]],[],true,true],["jump",7],["set",[[65,65],[97,97]],[],true,true],["capture",1],["repeat_end",0,0],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"iu","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-0-iu-7","pattern":"(aa|a)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,6],["set",[[65,65],[97,97]],[],true,true],["set",[[65,65],[97,97]],[],true,true],["jump",7],["set",[[65,65],[97,97]],[],true,true],["capture",1],["repeat_end",0,0],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"v","input":"","matches":[],"name":"generated-0-v-0","pattern":"(aa|a)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,6],["set",[[97,97]],[],false,true],["set",[[97,97]],[],false,true],["jump",7],["set",[[97,97]],[],false,true],["capture",1],["repeat_end",0,0],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"v","input":"aa","matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"name_units":null,"to":2}],"name":"generated-0-v-2","pattern":"(aa|a)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,6],["set",[[97,97]],[],false,true],["set",[[97,97]],[],false,true],["jump",7],["set",[[97,97]],[],false,true],["capture",1],["repeat_end",0,0],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"v","input":"abac","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-0-v-4","pattern":"(aa|a)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,6],["set",[[97,97]],[],false,true],["set",[[97,97]],[],false,true],["jump",7],["set",[[97,97]],[],false,true],["capture",1],["repeat_end",0,0],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"v","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-0-v-7","pattern":"(aa|a)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,6],["set",[[97,97]],[],false,true],["set",[[97,97]],[],false,true],["jump",7],["set",[[97,97]],[],false,true],["capture",1],["repeat_end",0,0],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"","input":"","matches":[],"name":"generated-1--0","pattern":"(a|aa)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,5],["set",[[97,97]],[],false,false],["jump",7],["set",[[97,97]],[],false,false],["set",[[97,97]],[],false,false],["capture",1],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[{"capture_units":[[97,97],[97]],"from":0,"indices":[[0,2],[1,2]],"name_units":null,"to":2}],"name":"generated-1--2","pattern":"(a|aa)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,5],["set",[[97,97]],[],false,false],["jump",7],["set",[[97,97]],[],false,false],["set",[[97,97]],[],false,false],["capture",1],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97]],"from":0,"indices":[[0,2],[1,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-1--4","pattern":"(a|aa)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,5],["set",[[97,97]],[],false,false],["jump",7],["set",[[97,97]],[],false,false],["set",[[97,97]],[],false,false],["capture",1],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-1--7","pattern":"(a|aa)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,5],["set",[[97,97]],[],false,false],["jump",7],["set",[[97,97]],[],false,false],["set",[[97,97]],[],false,false],["capture",1],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[],"name":"generated-1-g-0","pattern":"(a|aa)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,5],["set",[[97,97]],[],false,false],["jump",7],["set",[[97,97]],[],false,false],["set",[[97,97]],[],false,false],["capture",1],["repeat_end",0,0],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[{"capture_units":[[97,97],[97]],"from":0,"indices":[[0,2],[1,2]],"name_units":null,"to":2}],"name":"generated-1-g-2","pattern":"(a|aa)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,5],["set",[[97,97]],[],false,false],["jump",7],["set",[[97,97]],[],false,false],["set",[[97,97]],[],false,false],["capture",1],["repeat_end",0,0],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97]],"from":0,"indices":[[0,2],[1,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1},{"capture_units":[[97],[97]],"from":2,"indices":[[2,3],[2,3]],"name_units":null,"to":3}],"name":"generated-1-g-4","pattern":"(a|aa)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,5],["set",[[97,97]],[],false,false],["jump",7],["set",[[97,97]],[],false,false],["set",[[97,97]],[],false,false],["capture",1],["repeat_end",0,0],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1},{"capture_units":[[97],[97]],"from":2,"indices":[[2,3],[2,3]],"to":3}],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-1-g-7","pattern":"(a|aa)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,5],["set",[[97,97]],[],false,false],["jump",7],["set",[[97,97]],[],false,false],["set",[[97,97]],[],false,false],["capture",1],["repeat_end",0,0],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"u","input":"","matches":[],"name":"generated-1-u-0","pattern":"(a|aa)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,5],["set",[[97,97]],[],false,true],["jump",7],["set",[[97,97]],[],false,true],["set",[[97,97]],[],false,true],["capture",1],["repeat_end",0,0],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"u","input":"aa","matches":[{"capture_units":[[97,97],[97]],"from":0,"indices":[[0,2],[1,2]],"name_units":null,"to":2}],"name":"generated-1-u-2","pattern":"(a|aa)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,5],["set",[[97,97]],[],false,true],["jump",7],["set",[[97,97]],[],false,true],["set",[[97,97]],[],false,true],["capture",1],["repeat_end",0,0],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97]],"from":0,"indices":[[0,2],[1,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"u","input":"abac","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-1-u-4","pattern":"(a|aa)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,5],["set",[[97,97]],[],false,true],["jump",7],["set",[[97,97]],[],false,true],["set",[[97,97]],[],false,true],["capture",1],["repeat_end",0,0],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"u","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-1-u-7","pattern":"(a|aa)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,5],["set",[[97,97]],[],false,true],["jump",7],["set",[[97,97]],[],false,true],["set",[[97,97]],[],false,true],["capture",1],["repeat_end",0,0],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"iu","input":"","matches":[],"name":"generated-1-iu-0","pattern":"(a|aa)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,5],["set",[[65,65],[97,97]],[],true,true],["jump",7],["set",[[65,65],[97,97]],[],true,true],["set",[[65,65],[97,97]],[],true,true],["capture",1],["repeat_end",0,0],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"iu","input":"aa","matches":[{"capture_units":[[97,97],[97]],"from":0,"indices":[[0,2],[1,2]],"name_units":null,"to":2}],"name":"generated-1-iu-2","pattern":"(a|aa)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,5],["set",[[65,65],[97,97]],[],true,true],["jump",7],["set",[[65,65],[97,97]],[],true,true],["set",[[65,65],[97,97]],[],true,true],["capture",1],["repeat_end",0,0],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97]],"from":0,"indices":[[0,2],[1,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"iu","input":"abac","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-1-iu-4","pattern":"(a|aa)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,5],["set",[[65,65],[97,97]],[],true,true],["jump",7],["set",[[65,65],[97,97]],[],true,true],["set",[[65,65],[97,97]],[],true,true],["capture",1],["repeat_end",0,0],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"iu","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-1-iu-7","pattern":"(a|aa)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,5],["set",[[65,65],[97,97]],[],true,true],["jump",7],["set",[[65,65],[97,97]],[],true,true],["set",[[65,65],[97,97]],[],true,true],["capture",1],["repeat_end",0,0],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"v","input":"","matches":[],"name":"generated-1-v-0","pattern":"(a|aa)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,5],["set",[[97,97]],[],false,true],["jump",7],["set",[[97,97]],[],false,true],["set",[[97,97]],[],false,true],["capture",1],["repeat_end",0,0],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"v","input":"aa","matches":[{"capture_units":[[97,97],[97]],"from":0,"indices":[[0,2],[1,2]],"name_units":null,"to":2}],"name":"generated-1-v-2","pattern":"(a|aa)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,5],["set",[[97,97]],[],false,true],["jump",7],["set",[[97,97]],[],false,true],["set",[[97,97]],[],false,true],["capture",1],["repeat_end",0,0],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97]],"from":0,"indices":[[0,2],[1,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"v","input":"abac","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-1-v-4","pattern":"(a|aa)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,5],["set",[[97,97]],[],false,true],["jump",7],["set",[[97,97]],[],false,true],["set",[[97,97]],[],false,true],["capture",1],["repeat_end",0,0],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"v","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-1-v-7","pattern":"(a|aa)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,9,[1]],["mark",1],["split",3,5],["set",[[97,97]],[],false,true],["jump",7],["set",[[97,97]],[],false,true],["set",[[97,97]],[],false,true],["capture",1],["repeat_end",0,0],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"","input":"","matches":[{"capture_units":[[],null,null,null],"from":0,"indices":[[0,0],null,null,null],"name_units":null,"to":0}],"name":"generated-2--0","pattern":"((a)|(b))*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,12,[1,2,3]],["mark",1],["split",3,7],["mark",2],["set",[[97,97]],[],false,false],["capture",2],["jump",10],["mark",3],["set",[[98,98]],[],false,false],["capture",3],["capture",1],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":3,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[],null,null,null],"from":0,"indices":[[0,0],null,null,null],"to":0}],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[{"capture_units":[[97,97],[97],[97],null],"from":0,"indices":[[0,2],[1,2],[1,2],null],"name_units":null,"to":2}],"name":"generated-2--2","pattern":"((a)|(b))*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,12,[1,2,3]],["mark",1],["split",3,7],["mark",2],["set",[[97,97]],[],false,false],["capture",2],["jump",10],["mark",3],["set",[[98,98]],[],false,false],["capture",3],["capture",1],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":3,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97],[97],null],"from":0,"indices":[[0,2],[1,2],[1,2],null],"to":2}],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[{"capture_units":[[97,98,97],[97],[97],null],"from":0,"indices":[[0,3],[2,3],[2,3],null],"name_units":null,"to":3}],"name":"generated-2--4","pattern":"((a)|(b))*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,12,[1,2,3]],["mark",1],["split",3,7],["mark",2],["set",[[97,97]],[],false,false],["capture",2],["jump",10],["mark",3],["set",[[98,98]],[],false,false],["capture",3],["capture",1],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":3,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,98,97],[97],[97],null],"from":0,"indices":[[0,3],[2,3],[2,3],null],"to":3}],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97],[97],null],"from":0,"indices":[[0,1],[0,1],[0,1],null],"name_units":null,"to":1}],"name":"generated-2--7","pattern":"((a)|(b))*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,12,[1,2,3]],["mark",1],["split",3,7],["mark",2],["set",[[97,97]],[],false,false],["capture",2],["jump",10],["mark",3],["set",[[98,98]],[],false,false],["capture",3],["capture",1],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":3,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97],[97],null],"from":0,"indices":[[0,1],[0,1],[0,1],null],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[{"capture_units":[[],null,null,null],"from":0,"indices":[[0,0],null,null,null],"name_units":null,"to":0}],"name":"generated-2-g-0","pattern":"((a)|(b))*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,12,[1,2,3]],["mark",1],["split",3,7],["mark",2],["set",[[97,97]],[],false,false],["capture",2],["jump",10],["mark",3],["set",[[98,98]],[],false,false],["capture",3],["capture",1],["repeat_end",0,0],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":3,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[],null,null,null],"from":0,"indices":[[0,0],null,null,null],"to":0}],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[{"capture_units":[[97,97],[97],[97],null],"from":0,"indices":[[0,2],[1,2],[1,2],null],"name_units":null,"to":2},{"capture_units":[[],null,null,null],"from":2,"indices":[[2,2],null,null,null],"name_units":null,"to":2}],"name":"generated-2-g-2","pattern":"((a)|(b))*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,12,[1,2,3]],["mark",1],["split",3,7],["mark",2],["set",[[97,97]],[],false,false],["capture",2],["jump",10],["mark",3],["set",[[98,98]],[],false,false],["capture",3],["capture",1],["repeat_end",0,0],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":3,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97],[97],null],"from":0,"indices":[[0,2],[1,2],[1,2],null],"to":2},{"capture_units":[[],null,null,null],"from":2,"indices":[[2,2],null,null,null],"to":2}],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[{"capture_units":[[97,98,97],[97],[97],null],"from":0,"indices":[[0,3],[2,3],[2,3],null],"name_units":null,"to":3},{"capture_units":[[],null,null,null],"from":3,"indices":[[3,3],null,null,null],"name_units":null,"to":3},{"capture_units":[[],null,null,null],"from":4,"indices":[[4,4],null,null,null],"name_units":null,"to":4}],"name":"generated-2-g-4","pattern":"((a)|(b))*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,12,[1,2,3]],["mark",1],["split",3,7],["mark",2],["set",[[97,97]],[],false,false],["capture",2],["jump",10],["mark",3],["set",[[98,98]],[],false,false],["capture",3],["capture",1],["repeat_end",0,0],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":3,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,98,97],[97],[97],null],"from":0,"indices":[[0,3],[2,3],[2,3],null],"to":3},{"capture_units":[[],null,null,null],"from":3,"indices":[[3,3],null,null,null],"to":3},{"capture_units":[[],null,null,null],"from":4,"indices":[[4,4],null,null,null],"to":4}],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97],[97],null],"from":0,"indices":[[0,1],[0,1],[0,1],null],"name_units":null,"to":1},{"capture_units":[[],null,null,null],"from":1,"indices":[[1,1],null,null,null],"name_units":null,"to":1},{"capture_units":[[],null,null,null],"from":2,"indices":[[2,2],null,null,null],"name_units":null,"to":2},{"capture_units":[[],null,null,null],"from":3,"indices":[[3,3],null,null,null],"name_units":null,"to":3},{"capture_units":[[],null,null,null],"from":4,"indices":[[4,4],null,null,null],"name_units":null,"to":4},{"capture_units":[[],null,null,null],"from":5,"indices":[[5,5],null,null,null],"name_units":null,"to":5},{"capture_units":[[],null,null,null],"from":6,"indices":[[6,6],null,null,null],"name_units":null,"to":6},{"capture_units":[[],null,null,null],"from":7,"indices":[[7,7],null,null,null],"name_units":null,"to":7}],"name":"generated-2-g-7","pattern":"((a)|(b))*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,12,[1,2,3]],["mark",1],["split",3,7],["mark",2],["set",[[97,97]],[],false,false],["capture",2],["jump",10],["mark",3],["set",[[98,98]],[],false,false],["capture",3],["capture",1],["repeat_end",0,0],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":3,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97],[97],null],"from":0,"indices":[[0,1],[0,1],[0,1],null],"to":1},{"capture_units":[[],null,null,null],"from":1,"indices":[[1,1],null,null,null],"to":1},{"capture_units":[[],null,null,null],"from":2,"indices":[[2,2],null,null,null],"to":2},{"capture_units":[[],null,null,null],"from":3,"indices":[[3,3],null,null,null],"to":3},{"capture_units":[[],null,null,null],"from":4,"indices":[[4,4],null,null,null],"to":4},{"capture_units":[[],null,null,null],"from":5,"indices":[[5,5],null,null,null],"to":5},{"capture_units":[[],null,null,null],"from":6,"indices":[[6,6],null,null,null],"to":6},{"capture_units":[[],null,null,null],"from":7,"indices":[[7,7],null,null,null],"to":7}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"u","input":"","matches":[{"capture_units":[[],null,null,null],"from":0,"indices":[[0,0],null,null,null],"name_units":null,"to":0}],"name":"generated-2-u-0","pattern":"((a)|(b))*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,12,[1,2,3]],["mark",1],["split",3,7],["mark",2],["set",[[97,97]],[],false,true],["capture",2],["jump",10],["mark",3],["set",[[98,98]],[],false,true],["capture",3],["capture",1],["repeat_end",0,0],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":3,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[],null,null,null],"from":0,"indices":[[0,0],null,null,null],"to":0}],"rust_output":""},{"case":{"flags":"u","input":"aa","matches":[{"capture_units":[[97,97],[97],[97],null],"from":0,"indices":[[0,2],[1,2],[1,2],null],"name_units":null,"to":2}],"name":"generated-2-u-2","pattern":"((a)|(b))*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,12,[1,2,3]],["mark",1],["split",3,7],["mark",2],["set",[[97,97]],[],false,true],["capture",2],["jump",10],["mark",3],["set",[[98,98]],[],false,true],["capture",3],["capture",1],["repeat_end",0,0],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":3,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97],[97],null],"from":0,"indices":[[0,2],[1,2],[1,2],null],"to":2}],"rust_output":"aa"},{"case":{"flags":"u","input":"abac","matches":[{"capture_units":[[97,98,97],[97],[97],null],"from":0,"indices":[[0,3],[2,3],[2,3],null],"name_units":null,"to":3}],"name":"generated-2-u-4","pattern":"((a)|(b))*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,12,[1,2,3]],["mark",1],["split",3,7],["mark",2],["set",[[97,97]],[],false,true],["capture",2],["jump",10],["mark",3],["set",[[98,98]],[],false,true],["capture",3],["capture",1],["repeat_end",0,0],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":3,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,98,97],[97],[97],null],"from":0,"indices":[[0,3],[2,3],[2,3],null],"to":3}],"rust_output":"abac"},{"case":{"flags":"u","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97],[97],null],"from":0,"indices":[[0,1],[0,1],[0,1],null],"name_units":null,"to":1}],"name":"generated-2-u-7","pattern":"((a)|(b))*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,12,[1,2,3]],["mark",1],["split",3,7],["mark",2],["set",[[97,97]],[],false,true],["capture",2],["jump",10],["mark",3],["set",[[98,98]],[],false,true],["capture",3],["capture",1],["repeat_end",0,0],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":3,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97],[97],null],"from":0,"indices":[[0,1],[0,1],[0,1],null],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"iu","input":"","matches":[{"capture_units":[[],null,null,null],"from":0,"indices":[[0,0],null,null,null],"name_units":null,"to":0}],"name":"generated-2-iu-0","pattern":"((a)|(b))*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,12,[1,2,3]],["mark",1],["split",3,7],["mark",2],["set",[[65,65],[97,97]],[],true,true],["capture",2],["jump",10],["mark",3],["set",[[66,66],[98,98]],[],true,true],["capture",3],["capture",1],["repeat_end",0,0],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":3,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[],null,null,null],"from":0,"indices":[[0,0],null,null,null],"to":0}],"rust_output":""},{"case":{"flags":"iu","input":"aa","matches":[{"capture_units":[[97,97],[97],[97],null],"from":0,"indices":[[0,2],[1,2],[1,2],null],"name_units":null,"to":2}],"name":"generated-2-iu-2","pattern":"((a)|(b))*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,12,[1,2,3]],["mark",1],["split",3,7],["mark",2],["set",[[65,65],[97,97]],[],true,true],["capture",2],["jump",10],["mark",3],["set",[[66,66],[98,98]],[],true,true],["capture",3],["capture",1],["repeat_end",0,0],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":3,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97],[97],null],"from":0,"indices":[[0,2],[1,2],[1,2],null],"to":2}],"rust_output":"aa"},{"case":{"flags":"iu","input":"abac","matches":[{"capture_units":[[97,98,97],[97],[97],null],"from":0,"indices":[[0,3],[2,3],[2,3],null],"name_units":null,"to":3}],"name":"generated-2-iu-4","pattern":"((a)|(b))*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,12,[1,2,3]],["mark",1],["split",3,7],["mark",2],["set",[[65,65],[97,97]],[],true,true],["capture",2],["jump",10],["mark",3],["set",[[66,66],[98,98]],[],true,true],["capture",3],["capture",1],["repeat_end",0,0],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":3,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,98,97],[97],[97],null],"from":0,"indices":[[0,3],[2,3],[2,3],null],"to":3}],"rust_output":"abac"},{"case":{"flags":"iu","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97],[97],null],"from":0,"indices":[[0,1],[0,1],[0,1],null],"name_units":null,"to":1}],"name":"generated-2-iu-7","pattern":"((a)|(b))*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,12,[1,2,3]],["mark",1],["split",3,7],["mark",2],["set",[[65,65],[97,97]],[],true,true],["capture",2],["jump",10],["mark",3],["set",[[66,66],[98,98]],[],true,true],["capture",3],["capture",1],["repeat_end",0,0],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":3,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97],[97],null],"from":0,"indices":[[0,1],[0,1],[0,1],null],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"v","input":"","matches":[{"capture_units":[[],null,null,null],"from":0,"indices":[[0,0],null,null,null],"name_units":null,"to":0}],"name":"generated-2-v-0","pattern":"((a)|(b))*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,12,[1,2,3]],["mark",1],["split",3,7],["mark",2],["set",[[97,97]],[],false,true],["capture",2],["jump",10],["mark",3],["set",[[98,98]],[],false,true],["capture",3],["capture",1],["repeat_end",0,0],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":3,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[],null,null,null],"from":0,"indices":[[0,0],null,null,null],"to":0}],"rust_output":""},{"case":{"flags":"v","input":"aa","matches":[{"capture_units":[[97,97],[97],[97],null],"from":0,"indices":[[0,2],[1,2],[1,2],null],"name_units":null,"to":2}],"name":"generated-2-v-2","pattern":"((a)|(b))*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,12,[1,2,3]],["mark",1],["split",3,7],["mark",2],["set",[[97,97]],[],false,true],["capture",2],["jump",10],["mark",3],["set",[[98,98]],[],false,true],["capture",3],["capture",1],["repeat_end",0,0],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":3,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97],[97],null],"from":0,"indices":[[0,2],[1,2],[1,2],null],"to":2}],"rust_output":"aa"},{"case":{"flags":"v","input":"abac","matches":[{"capture_units":[[97,98,97],[97],[97],null],"from":0,"indices":[[0,3],[2,3],[2,3],null],"name_units":null,"to":3}],"name":"generated-2-v-4","pattern":"((a)|(b))*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,12,[1,2,3]],["mark",1],["split",3,7],["mark",2],["set",[[97,97]],[],false,true],["capture",2],["jump",10],["mark",3],["set",[[98,98]],[],false,true],["capture",3],["capture",1],["repeat_end",0,0],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":3,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,98,97],[97],[97],null],"from":0,"indices":[[0,3],[2,3],[2,3],null],"to":3}],"rust_output":"abac"},{"case":{"flags":"v","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97],[97],null],"from":0,"indices":[[0,1],[0,1],[0,1],null],"name_units":null,"to":1}],"name":"generated-2-v-7","pattern":"((a)|(b))*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,12,[1,2,3]],["mark",1],["split",3,7],["mark",2],["set",[[97,97]],[],false,true],["capture",2],["jump",10],["mark",3],["set",[[98,98]],[],false,true],["capture",3],["capture",1],["repeat_end",0,0],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":3,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97],[97],null],"from":0,"indices":[[0,1],[0,1],[0,1],null],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"","input":"","matches":[],"name":"generated-3--0","pattern":"((a)?b)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,10,[1,2]],["mark",1],["repeat",1,0,1,false,3,7,[2]],["mark",2],["set",[[97,97]],[],false,false],["capture",2],["repeat_end",1,2],["set",[[98,98]],[],false,false],["capture",1],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":2,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[],"name":"generated-3--2","pattern":"((a)?b)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,10,[1,2]],["mark",1],["repeat",1,0,1,false,3,7,[2]],["mark",2],["set",[[97,97]],[],false,false],["capture",2],["repeat_end",1,2],["set",[[98,98]],[],false,false],["capture",1],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":2,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[{"capture_units":[[97,98],[97,98],[97]],"from":0,"indices":[[0,2],[0,2],[0,1]],"name_units":null,"to":2}],"name":"generated-3--4","pattern":"((a)?b)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,10,[1,2]],["mark",1],["repeat",1,0,1,false,3,7,[2]],["mark",2],["set",[[97,97]],[],false,false],["capture",2],["repeat_end",1,2],["set",[[98,98]],[],false,false],["capture",1],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":2,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,98],[97,98],[97]],"from":0,"indices":[[0,2],[0,2],[0,1]],"to":2}],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[],"name":"generated-3--7","pattern":"((a)?b)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,10,[1,2]],["mark",1],["repeat",1,0,1,false,3,7,[2]],["mark",2],["set",[[97,97]],[],false,false],["capture",2],["repeat_end",1,2],["set",[[98,98]],[],false,false],["capture",1],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":2,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[],"name":"generated-3-g-0","pattern":"((a)?b)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,10,[1,2]],["mark",1],["repeat",1,0,1,false,3,7,[2]],["mark",2],["set",[[97,97]],[],false,false],["capture",2],["repeat_end",1,2],["set",[[98,98]],[],false,false],["capture",1],["repeat_end",0,0],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":2,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[],"name":"generated-3-g-2","pattern":"((a)?b)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,10,[1,2]],["mark",1],["repeat",1,0,1,false,3,7,[2]],["mark",2],["set",[[97,97]],[],false,false],["capture",2],["repeat_end",1,2],["set",[[98,98]],[],false,false],["capture",1],["repeat_end",0,0],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":2,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[{"capture_units":[[97,98],[97,98],[97]],"from":0,"indices":[[0,2],[0,2],[0,1]],"name_units":null,"to":2}],"name":"generated-3-g-4","pattern":"((a)?b)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,10,[1,2]],["mark",1],["repeat",1,0,1,false,3,7,[2]],["mark",2],["set",[[97,97]],[],false,false],["capture",2],["repeat_end",1,2],["set",[[98,98]],[],false,false],["capture",1],["repeat_end",0,0],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":2,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,98],[97,98],[97]],"from":0,"indices":[[0,2],[0,2],[0,1]],"to":2}],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[],"name":"generated-3-g-7","pattern":"((a)?b)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,10,[1,2]],["mark",1],["repeat",1,0,1,false,3,7,[2]],["mark",2],["set",[[97,97]],[],false,false],["capture",2],["repeat_end",1,2],["set",[[98,98]],[],false,false],["capture",1],["repeat_end",0,0],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":2,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"u","input":"","matches":[],"name":"generated-3-u-0","pattern":"((a)?b)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,10,[1,2]],["mark",1],["repeat",1,0,1,false,3,7,[2]],["mark",2],["set",[[97,97]],[],false,true],["capture",2],["repeat_end",1,2],["set",[[98,98]],[],false,true],["capture",1],["repeat_end",0,0],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":2,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"u","input":"aa","matches":[],"name":"generated-3-u-2","pattern":"((a)?b)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,10,[1,2]],["mark",1],["repeat",1,0,1,false,3,7,[2]],["mark",2],["set",[[97,97]],[],false,true],["capture",2],["repeat_end",1,2],["set",[[98,98]],[],false,true],["capture",1],["repeat_end",0,0],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":2,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"u","input":"abac","matches":[{"capture_units":[[97,98],[97,98],[97]],"from":0,"indices":[[0,2],[0,2],[0,1]],"name_units":null,"to":2}],"name":"generated-3-u-4","pattern":"((a)?b)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,10,[1,2]],["mark",1],["repeat",1,0,1,false,3,7,[2]],["mark",2],["set",[[97,97]],[],false,true],["capture",2],["repeat_end",1,2],["set",[[98,98]],[],false,true],["capture",1],["repeat_end",0,0],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":2,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,98],[97,98],[97]],"from":0,"indices":[[0,2],[0,2],[0,1]],"to":2}],"rust_output":"abac"},{"case":{"flags":"u","input":"aKſ😀\r\n","matches":[],"name":"generated-3-u-7","pattern":"((a)?b)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,10,[1,2]],["mark",1],["repeat",1,0,1,false,3,7,[2]],["mark",2],["set",[[97,97]],[],false,true],["capture",2],["repeat_end",1,2],["set",[[98,98]],[],false,true],["capture",1],["repeat_end",0,0],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":2,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"iu","input":"","matches":[],"name":"generated-3-iu-0","pattern":"((a)?b)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,10,[1,2]],["mark",1],["repeat",1,0,1,false,3,7,[2]],["mark",2],["set",[[65,65],[97,97]],[],true,true],["capture",2],["repeat_end",1,2],["set",[[66,66],[98,98]],[],true,true],["capture",1],["repeat_end",0,0],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":2,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"iu","input":"aa","matches":[],"name":"generated-3-iu-2","pattern":"((a)?b)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,10,[1,2]],["mark",1],["repeat",1,0,1,false,3,7,[2]],["mark",2],["set",[[65,65],[97,97]],[],true,true],["capture",2],["repeat_end",1,2],["set",[[66,66],[98,98]],[],true,true],["capture",1],["repeat_end",0,0],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":2,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"iu","input":"abac","matches":[{"capture_units":[[97,98],[97,98],[97]],"from":0,"indices":[[0,2],[0,2],[0,1]],"name_units":null,"to":2}],"name":"generated-3-iu-4","pattern":"((a)?b)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,10,[1,2]],["mark",1],["repeat",1,0,1,false,3,7,[2]],["mark",2],["set",[[65,65],[97,97]],[],true,true],["capture",2],["repeat_end",1,2],["set",[[66,66],[98,98]],[],true,true],["capture",1],["repeat_end",0,0],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":2,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,98],[97,98],[97]],"from":0,"indices":[[0,2],[0,2],[0,1]],"to":2}],"rust_output":"abac"},{"case":{"flags":"iu","input":"aKſ😀\r\n","matches":[],"name":"generated-3-iu-7","pattern":"((a)?b)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,10,[1,2]],["mark",1],["repeat",1,0,1,false,3,7,[2]],["mark",2],["set",[[65,65],[97,97]],[],true,true],["capture",2],["repeat_end",1,2],["set",[[66,66],[98,98]],[],true,true],["capture",1],["repeat_end",0,0],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":2,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"v","input":"","matches":[],"name":"generated-3-v-0","pattern":"((a)?b)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,10,[1,2]],["mark",1],["repeat",1,0,1,false,3,7,[2]],["mark",2],["set",[[97,97]],[],false,true],["capture",2],["repeat_end",1,2],["set",[[98,98]],[],false,true],["capture",1],["repeat_end",0,0],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":2,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"v","input":"aa","matches":[],"name":"generated-3-v-2","pattern":"((a)?b)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,10,[1,2]],["mark",1],["repeat",1,0,1,false,3,7,[2]],["mark",2],["set",[[97,97]],[],false,true],["capture",2],["repeat_end",1,2],["set",[[98,98]],[],false,true],["capture",1],["repeat_end",0,0],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":2,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"v","input":"abac","matches":[{"capture_units":[[97,98],[97,98],[97]],"from":0,"indices":[[0,2],[0,2],[0,1]],"name_units":null,"to":2}],"name":"generated-3-v-4","pattern":"((a)?b)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,10,[1,2]],["mark",1],["repeat",1,0,1,false,3,7,[2]],["mark",2],["set",[[97,97]],[],false,true],["capture",2],["repeat_end",1,2],["set",[[98,98]],[],false,true],["capture",1],["repeat_end",0,0],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":2,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,98],[97,98],[97]],"from":0,"indices":[[0,2],[0,2],[0,1]],"to":2}],"rust_output":"abac"},{"case":{"flags":"v","input":"aKſ😀\r\n","matches":[],"name":"generated-3-v-7","pattern":"((a)?b)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,10,[1,2]],["mark",1],["repeat",1,0,1,false,3,7,[2]],["mark",2],["set",[[97,97]],[],false,true],["capture",2],["repeat_end",1,2],["set",[[98,98]],[],false,true],["capture",1],["repeat_end",0,0],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":2,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"","input":"","matches":[{"capture_units":[[],null],"from":0,"indices":[[0,0],null],"name_units":null,"to":0}],"name":"generated-4--0","pattern":"(a*)*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[97,97]],[],false,false],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[],null],"from":0,"indices":[[0,0],null],"to":0}],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"name_units":null,"to":2}],"name":"generated-4--2","pattern":"(a*)*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[97,97]],[],false,false],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-4--4","pattern":"(a*)*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[97,97]],[],false,false],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-4--7","pattern":"(a*)*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[97,97]],[],false,false],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[{"capture_units":[[],null],"from":0,"indices":[[0,0],null],"name_units":null,"to":0}],"name":"generated-4-g-0","pattern":"(a*)*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[97,97]],[],false,false],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[],null],"from":0,"indices":[[0,0],null],"to":0}],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"name_units":null,"to":2},{"capture_units":[[],null],"from":2,"indices":[[2,2],null],"name_units":null,"to":2}],"name":"generated-4-g-2","pattern":"(a*)*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[97,97]],[],false,false],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"to":2},{"capture_units":[[],null],"from":2,"indices":[[2,2],null],"to":2}],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1},{"capture_units":[[],null],"from":1,"indices":[[1,1],null],"name_units":null,"to":1},{"capture_units":[[97],[97]],"from":2,"indices":[[2,3],[2,3]],"name_units":null,"to":3},{"capture_units":[[],null],"from":3,"indices":[[3,3],null],"name_units":null,"to":3},{"capture_units":[[],null],"from":4,"indices":[[4,4],null],"name_units":null,"to":4}],"name":"generated-4-g-4","pattern":"(a*)*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[97,97]],[],false,false],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1},{"capture_units":[[],null],"from":1,"indices":[[1,1],null],"to":1},{"capture_units":[[97],[97]],"from":2,"indices":[[2,3],[2,3]],"to":3},{"capture_units":[[],null],"from":3,"indices":[[3,3],null],"to":3},{"capture_units":[[],null],"from":4,"indices":[[4,4],null],"to":4}],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1},{"capture_units":[[],null],"from":1,"indices":[[1,1],null],"name_units":null,"to":1},{"capture_units":[[],null],"from":2,"indices":[[2,2],null],"name_units":null,"to":2},{"capture_units":[[],null],"from":3,"indices":[[3,3],null],"name_units":null,"to":3},{"capture_units":[[],null],"from":4,"indices":[[4,4],null],"name_units":null,"to":4},{"capture_units":[[],null],"from":5,"indices":[[5,5],null],"name_units":null,"to":5},{"capture_units":[[],null],"from":6,"indices":[[6,6],null],"name_units":null,"to":6},{"capture_units":[[],null],"from":7,"indices":[[7,7],null],"name_units":null,"to":7}],"name":"generated-4-g-7","pattern":"(a*)*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[97,97]],[],false,false],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1},{"capture_units":[[],null],"from":1,"indices":[[1,1],null],"to":1},{"capture_units":[[],null],"from":2,"indices":[[2,2],null],"to":2},{"capture_units":[[],null],"from":3,"indices":[[3,3],null],"to":3},{"capture_units":[[],null],"from":4,"indices":[[4,4],null],"to":4},{"capture_units":[[],null],"from":5,"indices":[[5,5],null],"to":5},{"capture_units":[[],null],"from":6,"indices":[[6,6],null],"to":6},{"capture_units":[[],null],"from":7,"indices":[[7,7],null],"to":7}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"u","input":"","matches":[{"capture_units":[[],null],"from":0,"indices":[[0,0],null],"name_units":null,"to":0}],"name":"generated-4-u-0","pattern":"(a*)*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[97,97]],[],false,true],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[],null],"from":0,"indices":[[0,0],null],"to":0}],"rust_output":""},{"case":{"flags":"u","input":"aa","matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"name_units":null,"to":2}],"name":"generated-4-u-2","pattern":"(a*)*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[97,97]],[],false,true],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"u","input":"abac","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-4-u-4","pattern":"(a*)*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[97,97]],[],false,true],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"u","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-4-u-7","pattern":"(a*)*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[97,97]],[],false,true],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"iu","input":"","matches":[{"capture_units":[[],null],"from":0,"indices":[[0,0],null],"name_units":null,"to":0}],"name":"generated-4-iu-0","pattern":"(a*)*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[65,65],[97,97]],[],true,true],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[],null],"from":0,"indices":[[0,0],null],"to":0}],"rust_output":""},{"case":{"flags":"iu","input":"aa","matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"name_units":null,"to":2}],"name":"generated-4-iu-2","pattern":"(a*)*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[65,65],[97,97]],[],true,true],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"iu","input":"abac","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-4-iu-4","pattern":"(a*)*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[65,65],[97,97]],[],true,true],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"iu","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-4-iu-7","pattern":"(a*)*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[65,65],[97,97]],[],true,true],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"v","input":"","matches":[{"capture_units":[[],null],"from":0,"indices":[[0,0],null],"name_units":null,"to":0}],"name":"generated-4-v-0","pattern":"(a*)*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[97,97]],[],false,true],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[],null],"from":0,"indices":[[0,0],null],"to":0}],"rust_output":""},{"case":{"flags":"v","input":"aa","matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"name_units":null,"to":2}],"name":"generated-4-v-2","pattern":"(a*)*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[97,97]],[],false,true],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"v","input":"abac","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-4-v-4","pattern":"(a*)*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[97,97]],[],false,true],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"v","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-4-v-7","pattern":"(a*)*"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[97,97]],[],false,true],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"","input":"","matches":[{"capture_units":[[],[]],"from":0,"indices":[[0,0],[0,0]],"name_units":null,"to":0}],"name":"generated-5--0","pattern":"(a*)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[97,97]],[],false,false],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[],[]],"from":0,"indices":[[0,0],[0,0]],"to":0}],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"name_units":null,"to":2}],"name":"generated-5--2","pattern":"(a*)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[97,97]],[],false,false],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-5--4","pattern":"(a*)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[97,97]],[],false,false],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-5--7","pattern":"(a*)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[97,97]],[],false,false],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[{"capture_units":[[],[]],"from":0,"indices":[[0,0],[0,0]],"name_units":null,"to":0}],"name":"generated-5-g-0","pattern":"(a*)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[97,97]],[],false,false],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[],[]],"from":0,"indices":[[0,0],[0,0]],"to":0}],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"name_units":null,"to":2},{"capture_units":[[],[]],"from":2,"indices":[[2,2],[2,2]],"name_units":null,"to":2}],"name":"generated-5-g-2","pattern":"(a*)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[97,97]],[],false,false],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"to":2},{"capture_units":[[],[]],"from":2,"indices":[[2,2],[2,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1},{"capture_units":[[],[]],"from":1,"indices":[[1,1],[1,1]],"name_units":null,"to":1},{"capture_units":[[97],[97]],"from":2,"indices":[[2,3],[2,3]],"name_units":null,"to":3},{"capture_units":[[],[]],"from":3,"indices":[[3,3],[3,3]],"name_units":null,"to":3},{"capture_units":[[],[]],"from":4,"indices":[[4,4],[4,4]],"name_units":null,"to":4}],"name":"generated-5-g-4","pattern":"(a*)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[97,97]],[],false,false],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1},{"capture_units":[[],[]],"from":1,"indices":[[1,1],[1,1]],"to":1},{"capture_units":[[97],[97]],"from":2,"indices":[[2,3],[2,3]],"to":3},{"capture_units":[[],[]],"from":3,"indices":[[3,3],[3,3]],"to":3},{"capture_units":[[],[]],"from":4,"indices":[[4,4],[4,4]],"to":4}],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1},{"capture_units":[[],[]],"from":1,"indices":[[1,1],[1,1]],"name_units":null,"to":1},{"capture_units":[[],[]],"from":2,"indices":[[2,2],[2,2]],"name_units":null,"to":2},{"capture_units":[[],[]],"from":3,"indices":[[3,3],[3,3]],"name_units":null,"to":3},{"capture_units":[[],[]],"from":4,"indices":[[4,4],[4,4]],"name_units":null,"to":4},{"capture_units":[[],[]],"from":5,"indices":[[5,5],[5,5]],"name_units":null,"to":5},{"capture_units":[[],[]],"from":6,"indices":[[6,6],[6,6]],"name_units":null,"to":6},{"capture_units":[[],[]],"from":7,"indices":[[7,7],[7,7]],"name_units":null,"to":7}],"name":"generated-5-g-7","pattern":"(a*)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[97,97]],[],false,false],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1},{"capture_units":[[],[]],"from":1,"indices":[[1,1],[1,1]],"to":1},{"capture_units":[[],[]],"from":2,"indices":[[2,2],[2,2]],"to":2},{"capture_units":[[],[]],"from":3,"indices":[[3,3],[3,3]],"to":3},{"capture_units":[[],[]],"from":4,"indices":[[4,4],[4,4]],"to":4},{"capture_units":[[],[]],"from":5,"indices":[[5,5],[5,5]],"to":5},{"capture_units":[[],[]],"from":6,"indices":[[6,6],[6,6]],"to":6},{"capture_units":[[],[]],"from":7,"indices":[[7,7],[7,7]],"to":7}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"u","input":"","matches":[{"capture_units":[[],[]],"from":0,"indices":[[0,0],[0,0]],"name_units":null,"to":0}],"name":"generated-5-u-0","pattern":"(a*)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[97,97]],[],false,true],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[],[]],"from":0,"indices":[[0,0],[0,0]],"to":0}],"rust_output":""},{"case":{"flags":"u","input":"aa","matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"name_units":null,"to":2}],"name":"generated-5-u-2","pattern":"(a*)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[97,97]],[],false,true],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"u","input":"abac","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-5-u-4","pattern":"(a*)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[97,97]],[],false,true],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"u","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-5-u-7","pattern":"(a*)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[97,97]],[],false,true],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"iu","input":"","matches":[{"capture_units":[[],[]],"from":0,"indices":[[0,0],[0,0]],"name_units":null,"to":0}],"name":"generated-5-iu-0","pattern":"(a*)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[65,65],[97,97]],[],true,true],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[],[]],"from":0,"indices":[[0,0],[0,0]],"to":0}],"rust_output":""},{"case":{"flags":"iu","input":"aa","matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"name_units":null,"to":2}],"name":"generated-5-iu-2","pattern":"(a*)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[65,65],[97,97]],[],true,true],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"iu","input":"abac","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-5-iu-4","pattern":"(a*)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[65,65],[97,97]],[],true,true],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"iu","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-5-iu-7","pattern":"(a*)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[65,65],[97,97]],[],true,true],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"v","input":"","matches":[{"capture_units":[[],[]],"from":0,"indices":[[0,0],[0,0]],"name_units":null,"to":0}],"name":"generated-5-v-0","pattern":"(a*)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[97,97]],[],false,true],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[],[]],"from":0,"indices":[[0,0],[0,0]],"to":0}],"rust_output":""},{"case":{"flags":"v","input":"aa","matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"name_units":null,"to":2}],"name":"generated-5-v-2","pattern":"(a*)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[97,97]],[],false,true],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"v","input":"abac","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-5-v-4","pattern":"(a*)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[97,97]],[],false,true],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"v","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-5-v-7","pattern":"(a*)+"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,1,null,false,1,7,[1]],["mark",1],["repeat",1,0,null,false,3,5,[]],["set",[[97,97]],[],false,true],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"","input":"","matches":[{"capture_units":[[],null],"from":0,"indices":[[0,0],null],"name_units":null,"to":0}],"name":"generated-6--0","pattern":"((?:a?){0,2})?"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,1,false,1,9,[1]],["mark",1],["repeat",1,0,2,false,3,7,[]],["repeat",2,0,1,false,4,6,[]],["set",[[97,97]],[],false,false],["repeat_end",2,3],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[],null],"from":0,"indices":[[0,0],null],"to":0}],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"name_units":null,"to":2}],"name":"generated-6--2","pattern":"((?:a?){0,2})?"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,1,false,1,9,[1]],["mark",1],["repeat",1,0,2,false,3,7,[]],["repeat",2,0,1,false,4,6,[]],["set",[[97,97]],[],false,false],["repeat_end",2,3],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-6--4","pattern":"((?:a?){0,2})?"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,1,false,1,9,[1]],["mark",1],["repeat",1,0,2,false,3,7,[]],["repeat",2,0,1,false,4,6,[]],["set",[[97,97]],[],false,false],["repeat_end",2,3],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-6--7","pattern":"((?:a?){0,2})?"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,1,false,1,9,[1]],["mark",1],["repeat",1,0,2,false,3,7,[]],["repeat",2,0,1,false,4,6,[]],["set",[[97,97]],[],false,false],["repeat_end",2,3],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[{"capture_units":[[],null],"from":0,"indices":[[0,0],null],"name_units":null,"to":0}],"name":"generated-6-g-0","pattern":"((?:a?){0,2})?"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,1,false,1,9,[1]],["mark",1],["repeat",1,0,2,false,3,7,[]],["repeat",2,0,1,false,4,6,[]],["set",[[97,97]],[],false,false],["repeat_end",2,3],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[],null],"from":0,"indices":[[0,0],null],"to":0}],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"name_units":null,"to":2},{"capture_units":[[],null],"from":2,"indices":[[2,2],null],"name_units":null,"to":2}],"name":"generated-6-g-2","pattern":"((?:a?){0,2})?"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,1,false,1,9,[1]],["mark",1],["repeat",1,0,2,false,3,7,[]],["repeat",2,0,1,false,4,6,[]],["set",[[97,97]],[],false,false],["repeat_end",2,3],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"to":2},{"capture_units":[[],null],"from":2,"indices":[[2,2],null],"to":2}],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1},{"capture_units":[[],null],"from":1,"indices":[[1,1],null],"name_units":null,"to":1},{"capture_units":[[97],[97]],"from":2,"indices":[[2,3],[2,3]],"name_units":null,"to":3},{"capture_units":[[],null],"from":3,"indices":[[3,3],null],"name_units":null,"to":3},{"capture_units":[[],null],"from":4,"indices":[[4,4],null],"name_units":null,"to":4}],"name":"generated-6-g-4","pattern":"((?:a?){0,2})?"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,1,false,1,9,[1]],["mark",1],["repeat",1,0,2,false,3,7,[]],["repeat",2,0,1,false,4,6,[]],["set",[[97,97]],[],false,false],["repeat_end",2,3],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1},{"capture_units":[[],null],"from":1,"indices":[[1,1],null],"to":1},{"capture_units":[[97],[97]],"from":2,"indices":[[2,3],[2,3]],"to":3},{"capture_units":[[],null],"from":3,"indices":[[3,3],null],"to":3},{"capture_units":[[],null],"from":4,"indices":[[4,4],null],"to":4}],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1},{"capture_units":[[],null],"from":1,"indices":[[1,1],null],"name_units":null,"to":1},{"capture_units":[[],null],"from":2,"indices":[[2,2],null],"name_units":null,"to":2},{"capture_units":[[],null],"from":3,"indices":[[3,3],null],"name_units":null,"to":3},{"capture_units":[[],null],"from":4,"indices":[[4,4],null],"name_units":null,"to":4},{"capture_units":[[],null],"from":5,"indices":[[5,5],null],"name_units":null,"to":5},{"capture_units":[[],null],"from":6,"indices":[[6,6],null],"name_units":null,"to":6},{"capture_units":[[],null],"from":7,"indices":[[7,7],null],"name_units":null,"to":7}],"name":"generated-6-g-7","pattern":"((?:a?){0,2})?"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,1,false,1,9,[1]],["mark",1],["repeat",1,0,2,false,3,7,[]],["repeat",2,0,1,false,4,6,[]],["set",[[97,97]],[],false,false],["repeat_end",2,3],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1},{"capture_units":[[],null],"from":1,"indices":[[1,1],null],"to":1},{"capture_units":[[],null],"from":2,"indices":[[2,2],null],"to":2},{"capture_units":[[],null],"from":3,"indices":[[3,3],null],"to":3},{"capture_units":[[],null],"from":4,"indices":[[4,4],null],"to":4},{"capture_units":[[],null],"from":5,"indices":[[5,5],null],"to":5},{"capture_units":[[],null],"from":6,"indices":[[6,6],null],"to":6},{"capture_units":[[],null],"from":7,"indices":[[7,7],null],"to":7}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"u","input":"","matches":[{"capture_units":[[],null],"from":0,"indices":[[0,0],null],"name_units":null,"to":0}],"name":"generated-6-u-0","pattern":"((?:a?){0,2})?"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,1,false,1,9,[1]],["mark",1],["repeat",1,0,2,false,3,7,[]],["repeat",2,0,1,false,4,6,[]],["set",[[97,97]],[],false,true],["repeat_end",2,3],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[],null],"from":0,"indices":[[0,0],null],"to":0}],"rust_output":""},{"case":{"flags":"u","input":"aa","matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"name_units":null,"to":2}],"name":"generated-6-u-2","pattern":"((?:a?){0,2})?"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,1,false,1,9,[1]],["mark",1],["repeat",1,0,2,false,3,7,[]],["repeat",2,0,1,false,4,6,[]],["set",[[97,97]],[],false,true],["repeat_end",2,3],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"u","input":"abac","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-6-u-4","pattern":"((?:a?){0,2})?"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,1,false,1,9,[1]],["mark",1],["repeat",1,0,2,false,3,7,[]],["repeat",2,0,1,false,4,6,[]],["set",[[97,97]],[],false,true],["repeat_end",2,3],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"u","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-6-u-7","pattern":"((?:a?){0,2})?"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,1,false,1,9,[1]],["mark",1],["repeat",1,0,2,false,3,7,[]],["repeat",2,0,1,false,4,6,[]],["set",[[97,97]],[],false,true],["repeat_end",2,3],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"iu","input":"","matches":[{"capture_units":[[],null],"from":0,"indices":[[0,0],null],"name_units":null,"to":0}],"name":"generated-6-iu-0","pattern":"((?:a?){0,2})?"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,1,false,1,9,[1]],["mark",1],["repeat",1,0,2,false,3,7,[]],["repeat",2,0,1,false,4,6,[]],["set",[[65,65],[97,97]],[],true,true],["repeat_end",2,3],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[],null],"from":0,"indices":[[0,0],null],"to":0}],"rust_output":""},{"case":{"flags":"iu","input":"aa","matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"name_units":null,"to":2}],"name":"generated-6-iu-2","pattern":"((?:a?){0,2})?"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,1,false,1,9,[1]],["mark",1],["repeat",1,0,2,false,3,7,[]],["repeat",2,0,1,false,4,6,[]],["set",[[65,65],[97,97]],[],true,true],["repeat_end",2,3],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"iu","input":"abac","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-6-iu-4","pattern":"((?:a?){0,2})?"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,1,false,1,9,[1]],["mark",1],["repeat",1,0,2,false,3,7,[]],["repeat",2,0,1,false,4,6,[]],["set",[[65,65],[97,97]],[],true,true],["repeat_end",2,3],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"iu","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-6-iu-7","pattern":"((?:a?){0,2})?"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,1,false,1,9,[1]],["mark",1],["repeat",1,0,2,false,3,7,[]],["repeat",2,0,1,false,4,6,[]],["set",[[65,65],[97,97]],[],true,true],["repeat_end",2,3],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"v","input":"","matches":[{"capture_units":[[],null],"from":0,"indices":[[0,0],null],"name_units":null,"to":0}],"name":"generated-6-v-0","pattern":"((?:a?){0,2})?"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,1,false,1,9,[1]],["mark",1],["repeat",1,0,2,false,3,7,[]],["repeat",2,0,1,false,4,6,[]],["set",[[97,97]],[],false,true],["repeat_end",2,3],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[],null],"from":0,"indices":[[0,0],null],"to":0}],"rust_output":""},{"case":{"flags":"v","input":"aa","matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"name_units":null,"to":2}],"name":"generated-6-v-2","pattern":"((?:a?){0,2})?"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,1,false,1,9,[1]],["mark",1],["repeat",1,0,2,false,3,7,[]],["repeat",2,0,1,false,4,6,[]],["set",[[97,97]],[],false,true],["repeat_end",2,3],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"v","input":"abac","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-6-v-4","pattern":"((?:a?){0,2})?"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,1,false,1,9,[1]],["mark",1],["repeat",1,0,2,false,3,7,[]],["repeat",2,0,1,false,4,6,[]],["set",[[97,97]],[],false,true],["repeat_end",2,3],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"v","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-6-v-7","pattern":"((?:a?){0,2})?"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,1,false,1,9,[1]],["mark",1],["repeat",1,0,2,false,3,7,[]],["repeat",2,0,1,false,4,6,[]],["set",[[97,97]],[],false,true],["repeat_end",2,3],["repeat_end",1,2],["capture",1],["repeat_end",0,0],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"","input":"","matches":[{"capture_units":[[],[]],"from":0,"indices":[[0,0],[0,0]],"name_units":null,"to":0}],"name":"generated-7--0","pattern":"(?=(a*))a*\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["repeat",1,0,null,false,2,4,[]],["set",[[97,97]],[],false,false],["repeat_end",1,1],["ref",[1],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[97,97]],[],false,false],["repeat_end",0,1],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[],[]],"from":0,"indices":[[0,0],[0,0]],"to":0}],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"name_units":null,"to":2}],"name":"generated-7--2","pattern":"(?=(a*))a*\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["repeat",1,0,null,false,2,4,[]],["set",[[97,97]],[],false,false],["repeat_end",1,1],["ref",[1],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[97,97]],[],false,false],["repeat_end",0,1],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-7--4","pattern":"(?=(a*))a*\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["repeat",1,0,null,false,2,4,[]],["set",[[97,97]],[],false,false],["repeat_end",1,1],["ref",[1],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[97,97]],[],false,false],["repeat_end",0,1],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-7--7","pattern":"(?=(a*))a*\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["repeat",1,0,null,false,2,4,[]],["set",[[97,97]],[],false,false],["repeat_end",1,1],["ref",[1],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[97,97]],[],false,false],["repeat_end",0,1],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[{"capture_units":[[],[]],"from":0,"indices":[[0,0],[0,0]],"name_units":null,"to":0}],"name":"generated-7-g-0","pattern":"(?=(a*))a*\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["repeat",1,0,null,false,2,4,[]],["set",[[97,97]],[],false,false],["repeat_end",1,1],["ref",[1],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[97,97]],[],false,false],["repeat_end",0,1],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[],[]],"from":0,"indices":[[0,0],[0,0]],"to":0}],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"name_units":null,"to":2},{"capture_units":[[],[]],"from":2,"indices":[[2,2],[2,2]],"name_units":null,"to":2}],"name":"generated-7-g-2","pattern":"(?=(a*))a*\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["repeat",1,0,null,false,2,4,[]],["set",[[97,97]],[],false,false],["repeat_end",1,1],["ref",[1],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[97,97]],[],false,false],["repeat_end",0,1],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"to":2},{"capture_units":[[],[]],"from":2,"indices":[[2,2],[2,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1},{"capture_units":[[],[]],"from":1,"indices":[[1,1],[1,1]],"name_units":null,"to":1},{"capture_units":[[97],[97]],"from":2,"indices":[[2,3],[2,3]],"name_units":null,"to":3},{"capture_units":[[],[]],"from":3,"indices":[[3,3],[3,3]],"name_units":null,"to":3},{"capture_units":[[],[]],"from":4,"indices":[[4,4],[4,4]],"name_units":null,"to":4}],"name":"generated-7-g-4","pattern":"(?=(a*))a*\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["repeat",1,0,null,false,2,4,[]],["set",[[97,97]],[],false,false],["repeat_end",1,1],["ref",[1],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[97,97]],[],false,false],["repeat_end",0,1],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1},{"capture_units":[[],[]],"from":1,"indices":[[1,1],[1,1]],"to":1},{"capture_units":[[97],[97]],"from":2,"indices":[[2,3],[2,3]],"to":3},{"capture_units":[[],[]],"from":3,"indices":[[3,3],[3,3]],"to":3},{"capture_units":[[],[]],"from":4,"indices":[[4,4],[4,4]],"to":4}],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1},{"capture_units":[[],[]],"from":1,"indices":[[1,1],[1,1]],"name_units":null,"to":1},{"capture_units":[[],[]],"from":2,"indices":[[2,2],[2,2]],"name_units":null,"to":2},{"capture_units":[[],[]],"from":3,"indices":[[3,3],[3,3]],"name_units":null,"to":3},{"capture_units":[[],[]],"from":4,"indices":[[4,4],[4,4]],"name_units":null,"to":4},{"capture_units":[[],[]],"from":5,"indices":[[5,5],[5,5]],"name_units":null,"to":5},{"capture_units":[[],[]],"from":6,"indices":[[6,6],[6,6]],"name_units":null,"to":6},{"capture_units":[[],[]],"from":7,"indices":[[7,7],[7,7]],"name_units":null,"to":7}],"name":"generated-7-g-7","pattern":"(?=(a*))a*\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["repeat",1,0,null,false,2,4,[]],["set",[[97,97]],[],false,false],["repeat_end",1,1],["ref",[1],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[97,97]],[],false,false],["repeat_end",0,1],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1},{"capture_units":[[],[]],"from":1,"indices":[[1,1],[1,1]],"to":1},{"capture_units":[[],[]],"from":2,"indices":[[2,2],[2,2]],"to":2},{"capture_units":[[],[]],"from":3,"indices":[[3,3],[3,3]],"to":3},{"capture_units":[[],[]],"from":4,"indices":[[4,4],[4,4]],"to":4},{"capture_units":[[],[]],"from":5,"indices":[[5,5],[5,5]],"to":5},{"capture_units":[[],[]],"from":6,"indices":[[6,6],[6,6]],"to":6},{"capture_units":[[],[]],"from":7,"indices":[[7,7],[7,7]],"to":7}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"u","input":"","matches":[{"capture_units":[[],[]],"from":0,"indices":[[0,0],[0,0]],"name_units":null,"to":0}],"name":"generated-7-u-0","pattern":"(?=(a*))a*\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["repeat",1,0,null,false,2,4,[]],["set",[[97,97]],[],false,true],["repeat_end",1,1],["ref",[1],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[97,97]],[],false,true],["repeat_end",0,1],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[],[]],"from":0,"indices":[[0,0],[0,0]],"to":0}],"rust_output":""},{"case":{"flags":"u","input":"aa","matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"name_units":null,"to":2}],"name":"generated-7-u-2","pattern":"(?=(a*))a*\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["repeat",1,0,null,false,2,4,[]],["set",[[97,97]],[],false,true],["repeat_end",1,1],["ref",[1],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[97,97]],[],false,true],["repeat_end",0,1],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"u","input":"abac","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-7-u-4","pattern":"(?=(a*))a*\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["repeat",1,0,null,false,2,4,[]],["set",[[97,97]],[],false,true],["repeat_end",1,1],["ref",[1],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[97,97]],[],false,true],["repeat_end",0,1],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"u","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-7-u-7","pattern":"(?=(a*))a*\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["repeat",1,0,null,false,2,4,[]],["set",[[97,97]],[],false,true],["repeat_end",1,1],["ref",[1],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[97,97]],[],false,true],["repeat_end",0,1],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"iu","input":"","matches":[{"capture_units":[[],[]],"from":0,"indices":[[0,0],[0,0]],"name_units":null,"to":0}],"name":"generated-7-iu-0","pattern":"(?=(a*))a*\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["repeat",1,0,null,false,2,4,[]],["set",[[65,65],[97,97]],[],true,true],["repeat_end",1,1],["ref",[1],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[65,65],[97,97]],[],true,true],["repeat_end",0,1],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[],[]],"from":0,"indices":[[0,0],[0,0]],"to":0}],"rust_output":""},{"case":{"flags":"iu","input":"aa","matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"name_units":null,"to":2}],"name":"generated-7-iu-2","pattern":"(?=(a*))a*\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["repeat",1,0,null,false,2,4,[]],["set",[[65,65],[97,97]],[],true,true],["repeat_end",1,1],["ref",[1],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[65,65],[97,97]],[],true,true],["repeat_end",0,1],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"iu","input":"abac","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-7-iu-4","pattern":"(?=(a*))a*\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["repeat",1,0,null,false,2,4,[]],["set",[[65,65],[97,97]],[],true,true],["repeat_end",1,1],["ref",[1],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[65,65],[97,97]],[],true,true],["repeat_end",0,1],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"iu","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-7-iu-7","pattern":"(?=(a*))a*\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["repeat",1,0,null,false,2,4,[]],["set",[[65,65],[97,97]],[],true,true],["repeat_end",1,1],["ref",[1],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[65,65],[97,97]],[],true,true],["repeat_end",0,1],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"v","input":"","matches":[{"capture_units":[[],[]],"from":0,"indices":[[0,0],[0,0]],"name_units":null,"to":0}],"name":"generated-7-v-0","pattern":"(?=(a*))a*\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["repeat",1,0,null,false,2,4,[]],["set",[[97,97]],[],false,true],["repeat_end",1,1],["ref",[1],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[97,97]],[],false,true],["repeat_end",0,1],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[],[]],"from":0,"indices":[[0,0],[0,0]],"to":0}],"rust_output":""},{"case":{"flags":"v","input":"aa","matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"name_units":null,"to":2}],"name":"generated-7-v-2","pattern":"(?=(a*))a*\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["repeat",1,0,null,false,2,4,[]],["set",[[97,97]],[],false,true],["repeat_end",1,1],["ref",[1],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[97,97]],[],false,true],["repeat_end",0,1],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97,97]],"from":0,"indices":[[0,2],[0,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"v","input":"abac","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-7-v-4","pattern":"(?=(a*))a*\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["repeat",1,0,null,false,2,4,[]],["set",[[97,97]],[],false,true],["repeat_end",1,1],["ref",[1],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[97,97]],[],false,true],["repeat_end",0,1],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"v","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":null,"to":1}],"name":"generated-7-v-7","pattern":"(?=(a*))a*\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["repeat",1,0,null,false,2,4,[]],["set",[[97,97]],[],false,true],["repeat_end",1,1],["ref",[1],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[97,97]],[],false,true],["repeat_end",0,1],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"","input":"","matches":[],"name":"generated-8--0","pattern":"(?!(a))b\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[98,98]],[],false,false],["ref",[1],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["set",[[97,97]],[],false,false],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[],"name":"generated-8--2","pattern":"(?!(a))b\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[98,98]],[],false,false],["ref",[1],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["set",[[97,97]],[],false,false],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[{"capture_units":[[98],null],"from":1,"indices":[[1,2],null],"name_units":null,"to":2}],"name":"generated-8--4","pattern":"(?!(a))b\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[98,98]],[],false,false],["ref",[1],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["set",[[97,97]],[],false,false],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[98],null],"from":1,"indices":[[1,2],null],"to":2}],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[],"name":"generated-8--7","pattern":"(?!(a))b\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[98,98]],[],false,false],["ref",[1],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["set",[[97,97]],[],false,false],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[],"name":"generated-8-g-0","pattern":"(?!(a))b\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[98,98]],[],false,false],["ref",[1],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["set",[[97,97]],[],false,false],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[],"name":"generated-8-g-2","pattern":"(?!(a))b\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[98,98]],[],false,false],["ref",[1],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["set",[[97,97]],[],false,false],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[{"capture_units":[[98],null],"from":1,"indices":[[1,2],null],"name_units":null,"to":2}],"name":"generated-8-g-4","pattern":"(?!(a))b\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[98,98]],[],false,false],["ref",[1],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["set",[[97,97]],[],false,false],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[98],null],"from":1,"indices":[[1,2],null],"to":2}],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[],"name":"generated-8-g-7","pattern":"(?!(a))b\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[98,98]],[],false,false],["ref",[1],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["set",[[97,97]],[],false,false],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"u","input":"","matches":[],"name":"generated-8-u-0","pattern":"(?!(a))b\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[98,98]],[],false,true],["ref",[1],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["set",[[97,97]],[],false,true],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"u","input":"aa","matches":[],"name":"generated-8-u-2","pattern":"(?!(a))b\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[98,98]],[],false,true],["ref",[1],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["set",[[97,97]],[],false,true],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"u","input":"abac","matches":[{"capture_units":[[98],null],"from":1,"indices":[[1,2],null],"name_units":null,"to":2}],"name":"generated-8-u-4","pattern":"(?!(a))b\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[98,98]],[],false,true],["ref",[1],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["set",[[97,97]],[],false,true],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[98],null],"from":1,"indices":[[1,2],null],"to":2}],"rust_output":"abac"},{"case":{"flags":"u","input":"aKſ😀\r\n","matches":[],"name":"generated-8-u-7","pattern":"(?!(a))b\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[98,98]],[],false,true],["ref",[1],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["set",[[97,97]],[],false,true],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"iu","input":"","matches":[],"name":"generated-8-iu-0","pattern":"(?!(a))b\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[66,66],[98,98]],[],true,true],["ref",[1],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["set",[[65,65],[97,97]],[],true,true],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"iu","input":"aa","matches":[],"name":"generated-8-iu-2","pattern":"(?!(a))b\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[66,66],[98,98]],[],true,true],["ref",[1],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["set",[[65,65],[97,97]],[],true,true],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"iu","input":"abac","matches":[{"capture_units":[[98],null],"from":1,"indices":[[1,2],null],"name_units":null,"to":2}],"name":"generated-8-iu-4","pattern":"(?!(a))b\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[66,66],[98,98]],[],true,true],["ref",[1],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["set",[[65,65],[97,97]],[],true,true],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[98],null],"from":1,"indices":[[1,2],null],"to":2}],"rust_output":"abac"},{"case":{"flags":"iu","input":"aKſ😀\r\n","matches":[],"name":"generated-8-iu-7","pattern":"(?!(a))b\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[66,66],[98,98]],[],true,true],["ref",[1],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["set",[[65,65],[97,97]],[],true,true],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"v","input":"","matches":[],"name":"generated-8-v-0","pattern":"(?!(a))b\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[98,98]],[],false,true],["ref",[1],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["set",[[97,97]],[],false,true],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"v","input":"aa","matches":[],"name":"generated-8-v-2","pattern":"(?!(a))b\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[98,98]],[],false,true],["ref",[1],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["set",[[97,97]],[],false,true],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"v","input":"abac","matches":[{"capture_units":[[98],null],"from":1,"indices":[[1,2],null],"name_units":null,"to":2}],"name":"generated-8-v-4","pattern":"(?!(a))b\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[98,98]],[],false,true],["ref",[1],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["set",[[97,97]],[],false,true],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[98],null],"from":1,"indices":[[1,2],null],"to":2}],"rust_output":"abac"},{"case":{"flags":"v","input":"aKſ😀\r\n","matches":[],"name":"generated-8-v-7","pattern":"(?!(a))b\\1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[98,98]],[],false,true],["ref",[1],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["set",[[97,97]],[],false,true],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"","input":"","matches":[],"name":"generated-9--0","pattern":"(?<=a(b?))c"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["set",[[99,99]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,1,false,2,4,[]],["set",[[98,98]],[],false,false],["repeat_end",0,1],["capture",1],["set",[[97,97]],[],false,false],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[],"name":"generated-9--2","pattern":"(?<=a(b?))c"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["set",[[99,99]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,1,false,2,4,[]],["set",[[98,98]],[],false,false],["repeat_end",0,1],["capture",1],["set",[[97,97]],[],false,false],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[{"capture_units":[[99],[]],"from":3,"indices":[[3,4],[3,3]],"name_units":null,"to":4}],"name":"generated-9--4","pattern":"(?<=a(b?))c"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["set",[[99,99]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,1,false,2,4,[]],["set",[[98,98]],[],false,false],["repeat_end",0,1],["capture",1],["set",[[97,97]],[],false,false],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[99],[]],"from":3,"indices":[[3,4],[3,3]],"to":4}],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[],"name":"generated-9--7","pattern":"(?<=a(b?))c"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["set",[[99,99]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,1,false,2,4,[]],["set",[[98,98]],[],false,false],["repeat_end",0,1],["capture",1],["set",[[97,97]],[],false,false],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[],"name":"generated-9-g-0","pattern":"(?<=a(b?))c"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["set",[[99,99]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,1,false,2,4,[]],["set",[[98,98]],[],false,false],["repeat_end",0,1],["capture",1],["set",[[97,97]],[],false,false],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[],"name":"generated-9-g-2","pattern":"(?<=a(b?))c"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["set",[[99,99]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,1,false,2,4,[]],["set",[[98,98]],[],false,false],["repeat_end",0,1],["capture",1],["set",[[97,97]],[],false,false],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[{"capture_units":[[99],[]],"from":3,"indices":[[3,4],[3,3]],"name_units":null,"to":4}],"name":"generated-9-g-4","pattern":"(?<=a(b?))c"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["set",[[99,99]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,1,false,2,4,[]],["set",[[98,98]],[],false,false],["repeat_end",0,1],["capture",1],["set",[[97,97]],[],false,false],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[99],[]],"from":3,"indices":[[3,4],[3,3]],"to":4}],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[],"name":"generated-9-g-7","pattern":"(?<=a(b?))c"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["set",[[99,99]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,1,false,2,4,[]],["set",[[98,98]],[],false,false],["repeat_end",0,1],["capture",1],["set",[[97,97]],[],false,false],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"u","input":"","matches":[],"name":"generated-9-u-0","pattern":"(?<=a(b?))c"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["set",[[99,99]],[],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,1,false,2,4,[]],["set",[[98,98]],[],false,true],["repeat_end",0,1],["capture",1],["set",[[97,97]],[],false,true],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"u","input":"aa","matches":[],"name":"generated-9-u-2","pattern":"(?<=a(b?))c"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["set",[[99,99]],[],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,1,false,2,4,[]],["set",[[98,98]],[],false,true],["repeat_end",0,1],["capture",1],["set",[[97,97]],[],false,true],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"u","input":"abac","matches":[{"capture_units":[[99],[]],"from":3,"indices":[[3,4],[3,3]],"name_units":null,"to":4}],"name":"generated-9-u-4","pattern":"(?<=a(b?))c"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["set",[[99,99]],[],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,1,false,2,4,[]],["set",[[98,98]],[],false,true],["repeat_end",0,1],["capture",1],["set",[[97,97]],[],false,true],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[99],[]],"from":3,"indices":[[3,4],[3,3]],"to":4}],"rust_output":"abac"},{"case":{"flags":"u","input":"aKſ😀\r\n","matches":[],"name":"generated-9-u-7","pattern":"(?<=a(b?))c"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["set",[[99,99]],[],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,1,false,2,4,[]],["set",[[98,98]],[],false,true],["repeat_end",0,1],["capture",1],["set",[[97,97]],[],false,true],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"iu","input":"","matches":[],"name":"generated-9-iu-0","pattern":"(?<=a(b?))c"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["set",[[67,67],[99,99]],[],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,1,false,2,4,[]],["set",[[66,66],[98,98]],[],true,true],["repeat_end",0,1],["capture",1],["set",[[65,65],[97,97]],[],true,true],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"iu","input":"aa","matches":[],"name":"generated-9-iu-2","pattern":"(?<=a(b?))c"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["set",[[67,67],[99,99]],[],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,1,false,2,4,[]],["set",[[66,66],[98,98]],[],true,true],["repeat_end",0,1],["capture",1],["set",[[65,65],[97,97]],[],true,true],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"iu","input":"abac","matches":[{"capture_units":[[99],[]],"from":3,"indices":[[3,4],[3,3]],"name_units":null,"to":4}],"name":"generated-9-iu-4","pattern":"(?<=a(b?))c"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["set",[[67,67],[99,99]],[],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,1,false,2,4,[]],["set",[[66,66],[98,98]],[],true,true],["repeat_end",0,1],["capture",1],["set",[[65,65],[97,97]],[],true,true],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[99],[]],"from":3,"indices":[[3,4],[3,3]],"to":4}],"rust_output":"abac"},{"case":{"flags":"iu","input":"aKſ😀\r\n","matches":[],"name":"generated-9-iu-7","pattern":"(?<=a(b?))c"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["set",[[67,67],[99,99]],[],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,1,false,2,4,[]],["set",[[66,66],[98,98]],[],true,true],["repeat_end",0,1],["capture",1],["set",[[65,65],[97,97]],[],true,true],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"v","input":"","matches":[],"name":"generated-9-v-0","pattern":"(?<=a(b?))c"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["set",[[99,99]],[],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,1,false,2,4,[]],["set",[[98,98]],[],false,true],["repeat_end",0,1],["capture",1],["set",[[97,97]],[],false,true],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"v","input":"aa","matches":[],"name":"generated-9-v-2","pattern":"(?<=a(b?))c"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["set",[[99,99]],[],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,1,false,2,4,[]],["set",[[98,98]],[],false,true],["repeat_end",0,1],["capture",1],["set",[[97,97]],[],false,true],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"v","input":"abac","matches":[{"capture_units":[[99],[]],"from":3,"indices":[[3,4],[3,3]],"name_units":null,"to":4}],"name":"generated-9-v-4","pattern":"(?<=a(b?))c"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["set",[[99,99]],[],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,1,false,2,4,[]],["set",[[98,98]],[],false,true],["repeat_end",0,1],["capture",1],["set",[[97,97]],[],false,true],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[99],[]],"from":3,"indices":[[3,4],[3,3]],"to":4}],"rust_output":"abac"},{"case":{"flags":"v","input":"aKſ😀\r\n","matches":[],"name":"generated-9-v-7","pattern":"(?<=a(b?))c"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[1]],["set",[[99,99]],[],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,1,false,2,4,[]],["set",[[98,98]],[],false,true],["repeat_end",0,1],["capture",1],["set",[[97,97]],[],false,true],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"","input":"","matches":[],"name":"generated-10--0","pattern":"(?<!(a*))b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[98,98]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[97,97]],[],false,false],["repeat_end",0,1],["capture",1],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[],"name":"generated-10--2","pattern":"(?<!(a*))b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[98,98]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[97,97]],[],false,false],["repeat_end",0,1],["capture",1],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[],"name":"generated-10--4","pattern":"(?<!(a*))b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[98,98]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[97,97]],[],false,false],["repeat_end",0,1],["capture",1],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[],"name":"generated-10--7","pattern":"(?<!(a*))b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[98,98]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[97,97]],[],false,false],["repeat_end",0,1],["capture",1],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[],"name":"generated-10-g-0","pattern":"(?<!(a*))b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[98,98]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[97,97]],[],false,false],["repeat_end",0,1],["capture",1],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[],"name":"generated-10-g-2","pattern":"(?<!(a*))b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[98,98]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[97,97]],[],false,false],["repeat_end",0,1],["capture",1],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[],"name":"generated-10-g-4","pattern":"(?<!(a*))b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[98,98]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[97,97]],[],false,false],["repeat_end",0,1],["capture",1],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[],"name":"generated-10-g-7","pattern":"(?<!(a*))b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[98,98]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[97,97]],[],false,false],["repeat_end",0,1],["capture",1],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"u","input":"","matches":[],"name":"generated-10-u-0","pattern":"(?<!(a*))b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[98,98]],[],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[97,97]],[],false,true],["repeat_end",0,1],["capture",1],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"u","input":"aa","matches":[],"name":"generated-10-u-2","pattern":"(?<!(a*))b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[98,98]],[],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[97,97]],[],false,true],["repeat_end",0,1],["capture",1],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"u","input":"abac","matches":[],"name":"generated-10-u-4","pattern":"(?<!(a*))b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[98,98]],[],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[97,97]],[],false,true],["repeat_end",0,1],["capture",1],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"u","input":"aKſ😀\r\n","matches":[],"name":"generated-10-u-7","pattern":"(?<!(a*))b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[98,98]],[],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[97,97]],[],false,true],["repeat_end",0,1],["capture",1],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"iu","input":"","matches":[],"name":"generated-10-iu-0","pattern":"(?<!(a*))b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[66,66],[98,98]],[],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[65,65],[97,97]],[],true,true],["repeat_end",0,1],["capture",1],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"iu","input":"aa","matches":[],"name":"generated-10-iu-2","pattern":"(?<!(a*))b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[66,66],[98,98]],[],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[65,65],[97,97]],[],true,true],["repeat_end",0,1],["capture",1],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"iu","input":"abac","matches":[],"name":"generated-10-iu-4","pattern":"(?<!(a*))b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[66,66],[98,98]],[],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[65,65],[97,97]],[],true,true],["repeat_end",0,1],["capture",1],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"iu","input":"aKſ😀\r\n","matches":[],"name":"generated-10-iu-7","pattern":"(?<!(a*))b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[66,66],[98,98]],[],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[65,65],[97,97]],[],true,true],["repeat_end",0,1],["capture",1],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"v","input":"","matches":[],"name":"generated-10-v-0","pattern":"(?<!(a*))b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[98,98]],[],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[97,97]],[],false,true],["repeat_end",0,1],["capture",1],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"v","input":"aa","matches":[],"name":"generated-10-v-2","pattern":"(?<!(a*))b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[98,98]],[],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[97,97]],[],false,true],["repeat_end",0,1],["capture",1],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"v","input":"abac","matches":[],"name":"generated-10-v-4","pattern":"(?<!(a*))b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[98,98]],[],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[97,97]],[],false,true],["repeat_end",0,1],["capture",1],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"v","input":"aKſ😀\r\n","matches":[],"name":"generated-10-v-7","pattern":"(?<!(a*))b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,false,[1]],["set",[[98,98]],[],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[97,97]],[],false,true],["repeat_end",0,1],["capture",1],["accept"]],"direction":-1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"","input":"","matches":[{"capture_units":[[]],"from":0,"indices":[[0,0]],"name_units":null,"to":0}],"name":"generated-11--0","pattern":"(?=a)?"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,1,false,1,3,[]],["look",0,true,[]],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[{"code":[["set",[[97,97]],[],false,false],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[]],"from":0,"indices":[[0,0]],"to":0}],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[{"capture_units":[[]],"from":0,"indices":[[0,0]],"name_units":null,"to":0}],"name":"generated-11--2","pattern":"(?=a)?"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,1,false,1,3,[]],["look",0,true,[]],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[{"code":[["set",[[97,97]],[],false,false],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[]],"from":0,"indices":[[0,0]],"to":0}],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[{"capture_units":[[]],"from":0,"indices":[[0,0]],"name_units":null,"to":0}],"name":"generated-11--4","pattern":"(?=a)?"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,1,false,1,3,[]],["look",0,true,[]],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[{"code":[["set",[[97,97]],[],false,false],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[]],"from":0,"indices":[[0,0]],"to":0}],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[{"capture_units":[[]],"from":0,"indices":[[0,0]],"name_units":null,"to":0}],"name":"generated-11--7","pattern":"(?=a)?"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,1,false,1,3,[]],["look",0,true,[]],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[{"code":[["set",[[97,97]],[],false,false],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[]],"from":0,"indices":[[0,0]],"to":0}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[{"capture_units":[[]],"from":0,"indices":[[0,0]],"name_units":null,"to":0}],"name":"generated-11-g-0","pattern":"(?=a)?"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,1,false,1,3,[]],["look",0,true,[]],["repeat_end",0,0],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[{"code":[["set",[[97,97]],[],false,false],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[]],"from":0,"indices":[[0,0]],"to":0}],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[{"capture_units":[[]],"from":0,"indices":[[0,0]],"name_units":null,"to":0},{"capture_units":[[]],"from":1,"indices":[[1,1]],"name_units":null,"to":1},{"capture_units":[[]],"from":2,"indices":[[2,2]],"name_units":null,"to":2}],"name":"generated-11-g-2","pattern":"(?=a)?"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,1,false,1,3,[]],["look",0,true,[]],["repeat_end",0,0],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[{"code":[["set",[[97,97]],[],false,false],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[]],"from":0,"indices":[[0,0]],"to":0},{"capture_units":[[]],"from":1,"indices":[[1,1]],"to":1},{"capture_units":[[]],"from":2,"indices":[[2,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[{"capture_units":[[]],"from":0,"indices":[[0,0]],"name_units":null,"to":0},{"capture_units":[[]],"from":1,"indices":[[1,1]],"name_units":null,"to":1},{"capture_units":[[]],"from":2,"indices":[[2,2]],"name_units":null,"to":2},{"capture_units":[[]],"from":3,"indices":[[3,3]],"name_units":null,"to":3},{"capture_units":[[]],"from":4,"indices":[[4,4]],"name_units":null,"to":4}],"name":"generated-11-g-4","pattern":"(?=a)?"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,1,false,1,3,[]],["look",0,true,[]],["repeat_end",0,0],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[{"code":[["set",[[97,97]],[],false,false],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[]],"from":0,"indices":[[0,0]],"to":0},{"capture_units":[[]],"from":1,"indices":[[1,1]],"to":1},{"capture_units":[[]],"from":2,"indices":[[2,2]],"to":2},{"capture_units":[[]],"from":3,"indices":[[3,3]],"to":3},{"capture_units":[[]],"from":4,"indices":[[4,4]],"to":4}],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[{"capture_units":[[]],"from":0,"indices":[[0,0]],"name_units":null,"to":0},{"capture_units":[[]],"from":1,"indices":[[1,1]],"name_units":null,"to":1},{"capture_units":[[]],"from":2,"indices":[[2,2]],"name_units":null,"to":2},{"capture_units":[[]],"from":3,"indices":[[3,3]],"name_units":null,"to":3},{"capture_units":[[]],"from":4,"indices":[[4,4]],"name_units":null,"to":4},{"capture_units":[[]],"from":5,"indices":[[5,5]],"name_units":null,"to":5},{"capture_units":[[]],"from":6,"indices":[[6,6]],"name_units":null,"to":6},{"capture_units":[[]],"from":7,"indices":[[7,7]],"name_units":null,"to":7}],"name":"generated-11-g-7","pattern":"(?=a)?"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,1,false,1,3,[]],["look",0,true,[]],["repeat_end",0,0],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[{"code":[["set",[[97,97]],[],false,false],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[]],"from":0,"indices":[[0,0]],"to":0},{"capture_units":[[]],"from":1,"indices":[[1,1]],"to":1},{"capture_units":[[]],"from":2,"indices":[[2,2]],"to":2},{"capture_units":[[]],"from":3,"indices":[[3,3]],"to":3},{"capture_units":[[]],"from":4,"indices":[[4,4]],"to":4},{"capture_units":[[]],"from":5,"indices":[[5,5]],"to":5},{"capture_units":[[]],"from":6,"indices":[[6,6]],"to":6},{"capture_units":[[]],"from":7,"indices":[[7,7]],"to":7}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"","input":"","matches":[],"name":"generated-12--0","pattern":"(?=(a)){2}"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,2,2,false,1,3,[1]],["look",0,true,[1]],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["set",[[97,97]],[],false,false],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[{"capture_units":[[],[97]],"from":0,"indices":[[0,0],[0,1]],"name_units":null,"to":0}],"name":"generated-12--2","pattern":"(?=(a)){2}"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,2,2,false,1,3,[1]],["look",0,true,[1]],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["set",[[97,97]],[],false,false],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[],[97]],"from":0,"indices":[[0,0],[0,1]],"to":0}],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[{"capture_units":[[],[97]],"from":0,"indices":[[0,0],[0,1]],"name_units":null,"to":0}],"name":"generated-12--4","pattern":"(?=(a)){2}"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,2,2,false,1,3,[1]],["look",0,true,[1]],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["set",[[97,97]],[],false,false],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[],[97]],"from":0,"indices":[[0,0],[0,1]],"to":0}],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[{"capture_units":[[],[97]],"from":0,"indices":[[0,0],[0,1]],"name_units":null,"to":0}],"name":"generated-12--7","pattern":"(?=(a)){2}"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,2,2,false,1,3,[1]],["look",0,true,[1]],["repeat_end",0,0],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["set",[[97,97]],[],false,false],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[],[97]],"from":0,"indices":[[0,0],[0,1]],"to":0}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[],"name":"generated-12-g-0","pattern":"(?=(a)){2}"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,2,2,false,1,3,[1]],["look",0,true,[1]],["repeat_end",0,0],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["set",[[97,97]],[],false,false],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[{"capture_units":[[],[97]],"from":0,"indices":[[0,0],[0,1]],"name_units":null,"to":0},{"capture_units":[[],[97]],"from":1,"indices":[[1,1],[1,2]],"name_units":null,"to":1}],"name":"generated-12-g-2","pattern":"(?=(a)){2}"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,2,2,false,1,3,[1]],["look",0,true,[1]],["repeat_end",0,0],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["set",[[97,97]],[],false,false],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[],[97]],"from":0,"indices":[[0,0],[0,1]],"to":0},{"capture_units":[[],[97]],"from":1,"indices":[[1,1],[1,2]],"to":1}],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[{"capture_units":[[],[97]],"from":0,"indices":[[0,0],[0,1]],"name_units":null,"to":0},{"capture_units":[[],[97]],"from":2,"indices":[[2,2],[2,3]],"name_units":null,"to":2}],"name":"generated-12-g-4","pattern":"(?=(a)){2}"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,2,2,false,1,3,[1]],["look",0,true,[1]],["repeat_end",0,0],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["set",[[97,97]],[],false,false],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[],[97]],"from":0,"indices":[[0,0],[0,1]],"to":0},{"capture_units":[[],[97]],"from":2,"indices":[[2,2],[2,3]],"to":2}],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[{"capture_units":[[],[97]],"from":0,"indices":[[0,0],[0,1]],"name_units":null,"to":0}],"name":"generated-12-g-7","pattern":"(?=(a)){2}"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,2,2,false,1,3,[1]],["look",0,true,[1]],["repeat_end",0,0],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[{"code":[["mark",1],["set",[[97,97]],[],false,false],["capture",1],["accept"]],"direction":1}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[],[97]],"from":0,"indices":[[0,0],[0,1]],"to":0}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"","input":"","matches":[],"name":"generated-13--0","pattern":"\\k<x>(?<x>a)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["ref",[1],false,false],["mark",1],["set",[[97,97]],[],false,false],["capture",1],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":{"x":[97]},"to":1}],"name":"generated-13--2","pattern":"\\k<x>(?<x>a)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["ref",[1],false,false],["mark",1],["set",[[97,97]],[],false,false],["capture",1],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":{"x":[97]},"to":1}],"name":"generated-13--4","pattern":"\\k<x>(?<x>a)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["ref",[1],false,false],["mark",1],["set",[[97,97]],[],false,false],["capture",1],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":{"x":[97]},"to":1}],"name":"generated-13--7","pattern":"\\k<x>(?<x>a)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["ref",[1],false,false],["mark",1],["set",[[97,97]],[],false,false],["capture",1],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[],"name":"generated-13-g-0","pattern":"\\k<x>(?<x>a)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["ref",[1],false,false],["mark",1],["set",[[97,97]],[],false,false],["capture",1],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":{"x":[97]},"to":1},{"capture_units":[[97],[97]],"from":1,"indices":[[1,2],[1,2]],"name_units":{"x":[97]},"to":2}],"name":"generated-13-g-2","pattern":"\\k<x>(?<x>a)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["ref",[1],false,false],["mark",1],["set",[[97,97]],[],false,false],["capture",1],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1},{"capture_units":[[97],[97]],"from":1,"indices":[[1,2],[1,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":{"x":[97]},"to":1},{"capture_units":[[97],[97]],"from":2,"indices":[[2,3],[2,3]],"name_units":{"x":[97]},"to":3}],"name":"generated-13-g-4","pattern":"\\k<x>(?<x>a)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["ref",[1],false,false],["mark",1],["set",[[97,97]],[],false,false],["capture",1],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1},{"capture_units":[[97],[97]],"from":2,"indices":[[2,3],[2,3]],"to":3}],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":{"x":[97]},"to":1}],"name":"generated-13-g-7","pattern":"\\k<x>(?<x>a)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["ref",[1],false,false],["mark",1],["set",[[97,97]],[],false,false],["capture",1],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"u","input":"","matches":[],"name":"generated-13-u-0","pattern":"\\k<x>(?<x>a)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["ref",[1],false,true],["mark",1],["set",[[97,97]],[],false,true],["capture",1],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"u","input":"aa","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":{"x":[97]},"to":1}],"name":"generated-13-u-2","pattern":"\\k<x>(?<x>a)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["ref",[1],false,true],["mark",1],["set",[[97,97]],[],false,true],["capture",1],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"aa"},{"case":{"flags":"u","input":"abac","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":{"x":[97]},"to":1}],"name":"generated-13-u-4","pattern":"\\k<x>(?<x>a)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["ref",[1],false,true],["mark",1],["set",[[97,97]],[],false,true],["capture",1],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"u","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":{"x":[97]},"to":1}],"name":"generated-13-u-7","pattern":"\\k<x>(?<x>a)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["ref",[1],false,true],["mark",1],["set",[[97,97]],[],false,true],["capture",1],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"iu","input":"","matches":[],"name":"generated-13-iu-0","pattern":"\\k<x>(?<x>a)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["ref",[1],true,true],["mark",1],["set",[[65,65],[97,97]],[],true,true],["capture",1],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"iu","input":"aa","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":{"x":[97]},"to":1}],"name":"generated-13-iu-2","pattern":"\\k<x>(?<x>a)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["ref",[1],true,true],["mark",1],["set",[[65,65],[97,97]],[],true,true],["capture",1],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"aa"},{"case":{"flags":"iu","input":"abac","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":{"x":[97]},"to":1}],"name":"generated-13-iu-4","pattern":"\\k<x>(?<x>a)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["ref",[1],true,true],["mark",1],["set",[[65,65],[97,97]],[],true,true],["capture",1],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"iu","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":{"x":[97]},"to":1}],"name":"generated-13-iu-7","pattern":"\\k<x>(?<x>a)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["ref",[1],true,true],["mark",1],["set",[[65,65],[97,97]],[],true,true],["capture",1],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"v","input":"","matches":[],"name":"generated-13-v-0","pattern":"\\k<x>(?<x>a)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["ref",[1],false,true],["mark",1],["set",[[97,97]],[],false,true],["capture",1],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"v","input":"aa","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":{"x":[97]},"to":1}],"name":"generated-13-v-2","pattern":"\\k<x>(?<x>a)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["ref",[1],false,true],["mark",1],["set",[[97,97]],[],false,true],["capture",1],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"aa"},{"case":{"flags":"v","input":"abac","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":{"x":[97]},"to":1}],"name":"generated-13-v-4","pattern":"\\k<x>(?<x>a)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["ref",[1],false,true],["mark",1],["set",[[97,97]],[],false,true],["capture",1],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"v","input":"aKſ😀\r\n","matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"name_units":{"x":[97]},"to":1}],"name":"generated-13-v-7","pattern":"\\k<x>(?<x>a)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["ref",[1],false,true],["mark",1],["set",[[97,97]],[],false,true],["capture",1],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"","input":"","matches":[],"name":"generated-14--0","pattern":"(?<x>a)\\k<x>"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["set",[[97,97]],[],false,false],["capture",1],["ref",[1],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[{"capture_units":[[97,97],[97]],"from":0,"indices":[[0,2],[0,1]],"name_units":{"x":[97]},"to":2}],"name":"generated-14--2","pattern":"(?<x>a)\\k<x>"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["set",[[97,97]],[],false,false],["capture",1],["ref",[1],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97]],"from":0,"indices":[[0,2],[0,1]],"to":2}],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[],"name":"generated-14--4","pattern":"(?<x>a)\\k<x>"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["set",[[97,97]],[],false,false],["capture",1],["ref",[1],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[],"name":"generated-14--7","pattern":"(?<x>a)\\k<x>"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["set",[[97,97]],[],false,false],["capture",1],["ref",[1],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[],"name":"generated-14-g-0","pattern":"(?<x>a)\\k<x>"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["set",[[97,97]],[],false,false],["capture",1],["ref",[1],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[{"capture_units":[[97,97],[97]],"from":0,"indices":[[0,2],[0,1]],"name_units":{"x":[97]},"to":2}],"name":"generated-14-g-2","pattern":"(?<x>a)\\k<x>"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["set",[[97,97]],[],false,false],["capture",1],["ref",[1],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97]],"from":0,"indices":[[0,2],[0,1]],"to":2}],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[],"name":"generated-14-g-4","pattern":"(?<x>a)\\k<x>"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["set",[[97,97]],[],false,false],["capture",1],["ref",[1],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[],"name":"generated-14-g-7","pattern":"(?<x>a)\\k<x>"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["set",[[97,97]],[],false,false],["capture",1],["ref",[1],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"u","input":"","matches":[],"name":"generated-14-u-0","pattern":"(?<x>a)\\k<x>"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["set",[[97,97]],[],false,true],["capture",1],["ref",[1],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"u","input":"aa","matches":[{"capture_units":[[97,97],[97]],"from":0,"indices":[[0,2],[0,1]],"name_units":{"x":[97]},"to":2}],"name":"generated-14-u-2","pattern":"(?<x>a)\\k<x>"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["set",[[97,97]],[],false,true],["capture",1],["ref",[1],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97]],"from":0,"indices":[[0,2],[0,1]],"to":2}],"rust_output":"aa"},{"case":{"flags":"u","input":"abac","matches":[],"name":"generated-14-u-4","pattern":"(?<x>a)\\k<x>"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["set",[[97,97]],[],false,true],["capture",1],["ref",[1],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"u","input":"aKſ😀\r\n","matches":[],"name":"generated-14-u-7","pattern":"(?<x>a)\\k<x>"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["set",[[97,97]],[],false,true],["capture",1],["ref",[1],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"iu","input":"","matches":[],"name":"generated-14-iu-0","pattern":"(?<x>a)\\k<x>"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["set",[[65,65],[97,97]],[],true,true],["capture",1],["ref",[1],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"iu","input":"aa","matches":[{"capture_units":[[97,97],[97]],"from":0,"indices":[[0,2],[0,1]],"name_units":{"x":[97]},"to":2}],"name":"generated-14-iu-2","pattern":"(?<x>a)\\k<x>"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["set",[[65,65],[97,97]],[],true,true],["capture",1],["ref",[1],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97]],"from":0,"indices":[[0,2],[0,1]],"to":2}],"rust_output":"aa"},{"case":{"flags":"iu","input":"abac","matches":[],"name":"generated-14-iu-4","pattern":"(?<x>a)\\k<x>"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["set",[[65,65],[97,97]],[],true,true],["capture",1],["ref",[1],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"iu","input":"aKſ😀\r\n","matches":[],"name":"generated-14-iu-7","pattern":"(?<x>a)\\k<x>"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["set",[[65,65],[97,97]],[],true,true],["capture",1],["ref",[1],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"v","input":"","matches":[],"name":"generated-14-v-0","pattern":"(?<x>a)\\k<x>"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["set",[[97,97]],[],false,true],["capture",1],["ref",[1],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"v","input":"aa","matches":[{"capture_units":[[97,97],[97]],"from":0,"indices":[[0,2],[0,1]],"name_units":{"x":[97]},"to":2}],"name":"generated-14-v-2","pattern":"(?<x>a)\\k<x>"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["set",[[97,97]],[],false,true],["capture",1],["ref",[1],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97],[97]],"from":0,"indices":[[0,2],[0,1]],"to":2}],"rust_output":"aa"},{"case":{"flags":"v","input":"abac","matches":[],"name":"generated-14-v-4","pattern":"(?<x>a)\\k<x>"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["set",[[97,97]],[],false,true],["capture",1],["ref",[1],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"v","input":"aKſ😀\r\n","matches":[],"name":"generated-14-v-7","pattern":"(?<x>a)\\k<x>"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["set",[[97,97]],[],false,true],["capture",1],["ref",[1],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"","input":"","matches":[],"name":"generated-15--0","pattern":"\\11"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[9,9]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[],"name":"generated-15--2","pattern":"\\11"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[9,9]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[],"name":"generated-15--4","pattern":"\\11"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[9,9]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[],"name":"generated-15--7","pattern":"\\11"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[9,9]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[],"name":"generated-15-g-0","pattern":"\\11"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[9,9]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[],"name":"generated-15-g-2","pattern":"\\11"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[9,9]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[],"name":"generated-15-g-4","pattern":"\\11"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[9,9]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[],"name":"generated-15-g-7","pattern":"\\11"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[9,9]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"","input":"","matches":[],"name":"generated-16--0","pattern":"\\8"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[56,56]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[],"name":"generated-16--2","pattern":"\\8"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[56,56]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[],"name":"generated-16--4","pattern":"\\8"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[56,56]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[],"name":"generated-16--7","pattern":"\\8"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[56,56]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[],"name":"generated-16-g-0","pattern":"\\8"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[56,56]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[],"name":"generated-16-g-2","pattern":"\\8"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[56,56]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[],"name":"generated-16-g-4","pattern":"\\8"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[56,56]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[],"name":"generated-16-g-7","pattern":"\\8"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[56,56]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"","input":"","matches":[],"name":"generated-17--0","pattern":"\\9"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[57,57]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[],"name":"generated-17--2","pattern":"\\9"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[57,57]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[],"name":"generated-17--4","pattern":"\\9"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[57,57]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[],"name":"generated-17--7","pattern":"\\9"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[57,57]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[],"name":"generated-17-g-0","pattern":"\\9"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[57,57]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[],"name":"generated-17-g-2","pattern":"\\9"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[57,57]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[],"name":"generated-17-g-4","pattern":"\\9"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[57,57]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[],"name":"generated-17-g-7","pattern":"\\9"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[57,57]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"","input":"","matches":[],"name":"generated-18--0","pattern":"\\08"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,0]],[],false,false],["set",[[56,56]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[],"name":"generated-18--2","pattern":"\\08"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,0]],[],false,false],["set",[[56,56]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[],"name":"generated-18--4","pattern":"\\08"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,0]],[],false,false],["set",[[56,56]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[],"name":"generated-18--7","pattern":"\\08"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,0]],[],false,false],["set",[[56,56]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[],"name":"generated-18-g-0","pattern":"\\08"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,0]],[],false,false],["set",[[56,56]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[],"name":"generated-18-g-2","pattern":"\\08"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,0]],[],false,false],["set",[[56,56]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[],"name":"generated-18-g-4","pattern":"\\08"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,0]],[],false,false],["set",[[56,56]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[],"name":"generated-18-g-7","pattern":"\\08"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,0]],[],false,false],["set",[[56,56]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"","input":"","matches":[],"name":"generated-19--0","pattern":"\\377"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[255,255]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[],"name":"generated-19--2","pattern":"\\377"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[255,255]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[],"name":"generated-19--4","pattern":"\\377"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[255,255]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[],"name":"generated-19--7","pattern":"\\377"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[255,255]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[],"name":"generated-19-g-0","pattern":"\\377"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[255,255]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[],"name":"generated-19-g-2","pattern":"\\377"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[255,255]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[],"name":"generated-19-g-4","pattern":"\\377"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[255,255]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[],"name":"generated-19-g-7","pattern":"\\377"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[255,255]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"","input":"","matches":[],"name":"generated-20--0","pattern":"\\400"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[32,32]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[],"name":"generated-20--2","pattern":"\\400"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[32,32]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[],"name":"generated-20--4","pattern":"\\400"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[32,32]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[],"name":"generated-20--7","pattern":"\\400"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[32,32]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[],"name":"generated-20-g-0","pattern":"\\400"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[32,32]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[],"name":"generated-20-g-2","pattern":"\\400"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[32,32]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[],"name":"generated-20-g-4","pattern":"\\400"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[32,32]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[],"name":"generated-20-g-7","pattern":"\\400"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[32,32]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"","input":"","matches":[],"name":"generated-21--0","pattern":"[\\c0-\\c9]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[16,25]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[],"name":"generated-21--2","pattern":"[\\c0-\\c9]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[16,25]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[],"name":"generated-21--4","pattern":"[\\c0-\\c9]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[16,25]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[],"name":"generated-21--7","pattern":"[\\c0-\\c9]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[16,25]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[],"name":"generated-21-g-0","pattern":"[\\c0-\\c9]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[16,25]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[],"name":"generated-21-g-2","pattern":"[\\c0-\\c9]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[16,25]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[],"name":"generated-21-g-4","pattern":"[\\c0-\\c9]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[16,25]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[],"name":"generated-21-g-7","pattern":"[\\c0-\\c9]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[16,25]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"","input":"","matches":[],"name":"generated-22--0","pattern":"[\\c_]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[31,31]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[],"name":"generated-22--2","pattern":"[\\c_]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[31,31]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[],"name":"generated-22--4","pattern":"[\\c_]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[31,31]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[],"name":"generated-22--7","pattern":"[\\c_]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[31,31]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[],"name":"generated-22-g-0","pattern":"[\\c_]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[31,31]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[],"name":"generated-22-g-2","pattern":"[\\c_]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[31,31]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[],"name":"generated-22-g-4","pattern":"[\\c_]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[31,31]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[],"name":"generated-22-g-7","pattern":"[\\c_]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[31,31]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"","input":"","matches":[],"name":"generated-23--0","pattern":"\\c1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[92,92]],[],false,false],["set",[[99,99]],[],false,false],["set",[[49,49]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[],"name":"generated-23--2","pattern":"\\c1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[92,92]],[],false,false],["set",[[99,99]],[],false,false],["set",[[49,49]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[],"name":"generated-23--4","pattern":"\\c1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[92,92]],[],false,false],["set",[[99,99]],[],false,false],["set",[[49,49]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[],"name":"generated-23--7","pattern":"\\c1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[92,92]],[],false,false],["set",[[99,99]],[],false,false],["set",[[49,49]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[],"name":"generated-23-g-0","pattern":"\\c1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[92,92]],[],false,false],["set",[[99,99]],[],false,false],["set",[[49,49]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[],"name":"generated-23-g-2","pattern":"\\c1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[92,92]],[],false,false],["set",[[99,99]],[],false,false],["set",[[49,49]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[],"name":"generated-23-g-4","pattern":"\\c1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[92,92]],[],false,false],["set",[[99,99]],[],false,false],["set",[[49,49]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[],"name":"generated-23-g-7","pattern":"\\c1"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[92,92]],[],false,false],["set",[[99,99]],[],false,false],["set",[[49,49]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"","input":"","matches":[],"name":"generated-24--0","pattern":"\\xzz"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[120,120]],[],false,false],["set",[[122,122]],[],false,false],["set",[[122,122]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[],"name":"generated-24--2","pattern":"\\xzz"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[120,120]],[],false,false],["set",[[122,122]],[],false,false],["set",[[122,122]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[],"name":"generated-24--4","pattern":"\\xzz"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[120,120]],[],false,false],["set",[[122,122]],[],false,false],["set",[[122,122]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[],"name":"generated-24--7","pattern":"\\xzz"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[120,120]],[],false,false],["set",[[122,122]],[],false,false],["set",[[122,122]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[],"name":"generated-24-g-0","pattern":"\\xzz"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[120,120]],[],false,false],["set",[[122,122]],[],false,false],["set",[[122,122]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[],"name":"generated-24-g-2","pattern":"\\xzz"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[120,120]],[],false,false],["set",[[122,122]],[],false,false],["set",[[122,122]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[],"name":"generated-24-g-4","pattern":"\\xzz"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[120,120]],[],false,false],["set",[[122,122]],[],false,false],["set",[[122,122]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[],"name":"generated-24-g-7","pattern":"\\xzz"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[120,120]],[],false,false],["set",[[122,122]],[],false,false],["set",[[122,122]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"","input":"","matches":[],"name":"generated-25--0","pattern":"\\uZZZZ"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[117,117]],[],false,false],["set",[[90,90]],[],false,false],["set",[[90,90]],[],false,false],["set",[[90,90]],[],false,false],["set",[[90,90]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[],"name":"generated-25--2","pattern":"\\uZZZZ"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[117,117]],[],false,false],["set",[[90,90]],[],false,false],["set",[[90,90]],[],false,false],["set",[[90,90]],[],false,false],["set",[[90,90]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[],"name":"generated-25--4","pattern":"\\uZZZZ"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[117,117]],[],false,false],["set",[[90,90]],[],false,false],["set",[[90,90]],[],false,false],["set",[[90,90]],[],false,false],["set",[[90,90]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[],"name":"generated-25--7","pattern":"\\uZZZZ"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[117,117]],[],false,false],["set",[[90,90]],[],false,false],["set",[[90,90]],[],false,false],["set",[[90,90]],[],false,false],["set",[[90,90]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[],"name":"generated-25-g-0","pattern":"\\uZZZZ"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[117,117]],[],false,false],["set",[[90,90]],[],false,false],["set",[[90,90]],[],false,false],["set",[[90,90]],[],false,false],["set",[[90,90]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[],"name":"generated-25-g-2","pattern":"\\uZZZZ"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[117,117]],[],false,false],["set",[[90,90]],[],false,false],["set",[[90,90]],[],false,false],["set",[[90,90]],[],false,false],["set",[[90,90]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[],"name":"generated-25-g-4","pattern":"\\uZZZZ"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[117,117]],[],false,false],["set",[[90,90]],[],false,false],["set",[[90,90]],[],false,false],["set",[[90,90]],[],false,false],["set",[[90,90]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[],"name":"generated-25-g-7","pattern":"\\uZZZZ"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[117,117]],[],false,false],["set",[[90,90]],[],false,false],["set",[[90,90]],[],false,false],["set",[[90,90]],[],false,false],["set",[[90,90]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"","input":"","matches":[],"name":"generated-26--0","pattern":"[-a]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[97,97]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-26--2","pattern":"[-a]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[97,97]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-26--4","pattern":"[-a]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[97,97]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-26--7","pattern":"[-a]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[97,97]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[],"name":"generated-26-g-0","pattern":"[-a]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[97,97]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1},{"capture_units":[[97]],"from":1,"indices":[[1,2]],"name_units":null,"to":2}],"name":"generated-26-g-2","pattern":"[-a]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[97,97]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1},{"capture_units":[[97]],"from":1,"indices":[[1,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1},{"capture_units":[[97]],"from":2,"indices":[[2,3]],"name_units":null,"to":3}],"name":"generated-26-g-4","pattern":"[-a]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[97,97]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1},{"capture_units":[[97]],"from":2,"indices":[[2,3]],"to":3}],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-26-g-7","pattern":"[-a]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[97,97]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"u","input":"","matches":[],"name":"generated-26-u-0","pattern":"[-a]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[97,97]],[],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"u","input":"aa","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-26-u-2","pattern":"[-a]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[97,97]],[],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aa"},{"case":{"flags":"u","input":"abac","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-26-u-4","pattern":"[-a]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[97,97]],[],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"u","input":"aKſ😀\r\n","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-26-u-7","pattern":"[-a]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[97,97]],[],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"iu","input":"","matches":[],"name":"generated-26-iu-0","pattern":"[-a]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[65,65],[97,97]],[],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"iu","input":"aa","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-26-iu-2","pattern":"[-a]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[65,65],[97,97]],[],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aa"},{"case":{"flags":"iu","input":"abac","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-26-iu-4","pattern":"[-a]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[65,65],[97,97]],[],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"iu","input":"aKſ😀\r\n","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-26-iu-7","pattern":"[-a]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[65,65],[97,97]],[],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"","input":"","matches":[],"name":"generated-27--0","pattern":"[a-]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[97,97]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-27--2","pattern":"[a-]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[97,97]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-27--4","pattern":"[a-]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[97,97]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-27--7","pattern":"[a-]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[97,97]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[],"name":"generated-27-g-0","pattern":"[a-]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[97,97]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1},{"capture_units":[[97]],"from":1,"indices":[[1,2]],"name_units":null,"to":2}],"name":"generated-27-g-2","pattern":"[a-]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[97,97]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1},{"capture_units":[[97]],"from":1,"indices":[[1,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1},{"capture_units":[[97]],"from":2,"indices":[[2,3]],"name_units":null,"to":3}],"name":"generated-27-g-4","pattern":"[a-]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[97,97]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1},{"capture_units":[[97]],"from":2,"indices":[[2,3]],"to":3}],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-27-g-7","pattern":"[a-]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[97,97]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"u","input":"","matches":[],"name":"generated-27-u-0","pattern":"[a-]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[97,97]],[],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"u","input":"aa","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-27-u-2","pattern":"[a-]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[97,97]],[],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aa"},{"case":{"flags":"u","input":"abac","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-27-u-4","pattern":"[a-]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[97,97]],[],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"u","input":"aKſ😀\r\n","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-27-u-7","pattern":"[a-]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[97,97]],[],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"iu","input":"","matches":[],"name":"generated-27-iu-0","pattern":"[a-]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[65,65],[97,97]],[],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"iu","input":"aa","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-27-iu-2","pattern":"[a-]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[65,65],[97,97]],[],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aa"},{"case":{"flags":"iu","input":"abac","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-27-iu-4","pattern":"[a-]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[65,65],[97,97]],[],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"iu","input":"aKſ😀\r\n","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-27-iu-7","pattern":"[a-]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[65,65],[97,97]],[],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"","input":"","matches":[],"name":"generated-28--0","pattern":"[]]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[],[],false,false],["set",[[93,93]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[],"name":"generated-28--2","pattern":"[]]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[],[],false,false],["set",[[93,93]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[],"name":"generated-28--4","pattern":"[]]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[],[],false,false],["set",[[93,93]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[],"name":"generated-28--7","pattern":"[]]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[],[],false,false],["set",[[93,93]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[],"name":"generated-28-g-0","pattern":"[]]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[],[],false,false],["set",[[93,93]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[],"name":"generated-28-g-2","pattern":"[]]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[],[],false,false],["set",[[93,93]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[],"name":"generated-28-g-4","pattern":"[]]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[],[],false,false],["set",[[93,93]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[],"name":"generated-28-g-7","pattern":"[]]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[],[],false,false],["set",[[93,93]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"","input":"","matches":[],"name":"generated-29--0","pattern":"[^]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,65535]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-29--2","pattern":"[^]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,65535]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-29--4","pattern":"[^]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,65535]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-29--7","pattern":"[^]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,65535]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[],"name":"generated-29-g-0","pattern":"[^]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,65535]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1},{"capture_units":[[97]],"from":1,"indices":[[1,2]],"name_units":null,"to":2}],"name":"generated-29-g-2","pattern":"[^]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,65535]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1},{"capture_units":[[97]],"from":1,"indices":[[1,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1},{"capture_units":[[98]],"from":1,"indices":[[1,2]],"name_units":null,"to":2},{"capture_units":[[97]],"from":2,"indices":[[2,3]],"name_units":null,"to":3},{"capture_units":[[99]],"from":3,"indices":[[3,4]],"name_units":null,"to":4}],"name":"generated-29-g-4","pattern":"[^]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,65535]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1},{"capture_units":[[98]],"from":1,"indices":[[1,2]],"to":2},{"capture_units":[[97]],"from":2,"indices":[[2,3]],"to":3},{"capture_units":[[99]],"from":3,"indices":[[3,4]],"to":4}],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1},{"capture_units":[[8490]],"from":1,"indices":[[1,2]],"name_units":null,"to":2},{"capture_units":[[383]],"from":2,"indices":[[2,3]],"name_units":null,"to":3},{"capture_units":[[55357]],"from":3,"indices":[[3,4]],"name_units":null,"to":4},{"capture_units":[[56832]],"from":4,"indices":[[4,5]],"name_units":null,"to":5},{"capture_units":[[13]],"from":5,"indices":[[5,6]],"name_units":null,"to":6},{"capture_units":[[10]],"from":6,"indices":[[6,7]],"name_units":null,"to":7}],"name":"generated-29-g-7","pattern":"[^]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,65535]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1},{"capture_units":[[8490]],"from":1,"indices":[[1,2]],"to":2},{"capture_units":[[383]],"from":2,"indices":[[2,3]],"to":3},{"capture_units":[[55357]],"from":3,"indices":[[3,4]],"to":4},{"capture_units":[[56832]],"from":4,"indices":[[4,5]],"to":5},{"capture_units":[[13]],"from":5,"indices":[[5,6]],"to":6},{"capture_units":[[10]],"from":6,"indices":[[6,7]],"to":7}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"u","input":"","matches":[],"name":"generated-29-u-0","pattern":"[^]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,1114111]],[],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"u","input":"aa","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-29-u-2","pattern":"[^]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,1114111]],[],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aa"},{"case":{"flags":"u","input":"abac","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-29-u-4","pattern":"[^]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,1114111]],[],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"u","input":"aKſ😀\r\n","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-29-u-7","pattern":"[^]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,1114111]],[],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"iu","input":"","matches":[],"name":"generated-29-iu-0","pattern":"[^]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,1114111]],[],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"iu","input":"aa","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-29-iu-2","pattern":"[^]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,1114111]],[],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aa"},{"case":{"flags":"iu","input":"abac","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-29-iu-4","pattern":"[^]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,1114111]],[],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"iu","input":"aKſ😀\r\n","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-29-iu-7","pattern":"[^]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,1114111]],[],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"v","input":"","matches":[],"name":"generated-29-v-0","pattern":"[^]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,1114111]],[],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"v","input":"aa","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-29-v-2","pattern":"[^]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,1114111]],[],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aa"},{"case":{"flags":"v","input":"abac","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-29-v-4","pattern":"[^]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,1114111]],[],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"v","input":"aKſ😀\r\n","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-29-v-7","pattern":"[^]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,1114111]],[],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"","input":"","matches":[],"name":"generated-30--0","pattern":"[\\D\\d]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,65535]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-30--2","pattern":"[\\D\\d]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,65535]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-30--4","pattern":"[\\D\\d]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,65535]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-30--7","pattern":"[\\D\\d]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,65535]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[],"name":"generated-30-g-0","pattern":"[\\D\\d]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,65535]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1},{"capture_units":[[97]],"from":1,"indices":[[1,2]],"name_units":null,"to":2}],"name":"generated-30-g-2","pattern":"[\\D\\d]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,65535]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1},{"capture_units":[[97]],"from":1,"indices":[[1,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1},{"capture_units":[[98]],"from":1,"indices":[[1,2]],"name_units":null,"to":2},{"capture_units":[[97]],"from":2,"indices":[[2,3]],"name_units":null,"to":3},{"capture_units":[[99]],"from":3,"indices":[[3,4]],"name_units":null,"to":4}],"name":"generated-30-g-4","pattern":"[\\D\\d]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,65535]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1},{"capture_units":[[98]],"from":1,"indices":[[1,2]],"to":2},{"capture_units":[[97]],"from":2,"indices":[[2,3]],"to":3},{"capture_units":[[99]],"from":3,"indices":[[3,4]],"to":4}],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1},{"capture_units":[[8490]],"from":1,"indices":[[1,2]],"name_units":null,"to":2},{"capture_units":[[383]],"from":2,"indices":[[2,3]],"name_units":null,"to":3},{"capture_units":[[55357]],"from":3,"indices":[[3,4]],"name_units":null,"to":4},{"capture_units":[[56832]],"from":4,"indices":[[4,5]],"name_units":null,"to":5},{"capture_units":[[13]],"from":5,"indices":[[5,6]],"name_units":null,"to":6},{"capture_units":[[10]],"from":6,"indices":[[6,7]],"name_units":null,"to":7}],"name":"generated-30-g-7","pattern":"[\\D\\d]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,65535]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1},{"capture_units":[[8490]],"from":1,"indices":[[1,2]],"to":2},{"capture_units":[[383]],"from":2,"indices":[[2,3]],"to":3},{"capture_units":[[55357]],"from":3,"indices":[[3,4]],"to":4},{"capture_units":[[56832]],"from":4,"indices":[[4,5]],"to":5},{"capture_units":[[13]],"from":5,"indices":[[5,6]],"to":6},{"capture_units":[[10]],"from":6,"indices":[[6,7]],"to":7}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"u","input":"","matches":[],"name":"generated-30-u-0","pattern":"[\\D\\d]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,1114111]],[],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"u","input":"aa","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-30-u-2","pattern":"[\\D\\d]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,1114111]],[],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aa"},{"case":{"flags":"u","input":"abac","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-30-u-4","pattern":"[\\D\\d]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,1114111]],[],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"u","input":"aKſ😀\r\n","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-30-u-7","pattern":"[\\D\\d]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,1114111]],[],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"iu","input":"","matches":[],"name":"generated-30-iu-0","pattern":"[\\D\\d]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,1114111]],[],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"iu","input":"aa","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-30-iu-2","pattern":"[\\D\\d]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,1114111]],[],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aa"},{"case":{"flags":"iu","input":"abac","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-30-iu-4","pattern":"[\\D\\d]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,1114111]],[],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"iu","input":"aKſ😀\r\n","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-30-iu-7","pattern":"[\\D\\d]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,1114111]],[],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"v","input":"","matches":[],"name":"generated-30-v-0","pattern":"[\\D\\d]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,1114111]],[],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"v","input":"aa","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-30-v-2","pattern":"[\\D\\d]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,1114111]],[],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aa"},{"case":{"flags":"v","input":"abac","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-30-v-4","pattern":"[\\D\\d]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,1114111]],[],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"v","input":"aKſ😀\r\n","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-30-v-7","pattern":"[\\D\\d]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,1114111]],[],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"","input":"","matches":[],"name":"generated-31--0","pattern":"[a-\\d]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[48,57],[97,97]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-31--2","pattern":"[a-\\d]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[48,57],[97,97]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-31--4","pattern":"[a-\\d]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[48,57],[97,97]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-31--7","pattern":"[a-\\d]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[48,57],[97,97]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[],"name":"generated-31-g-0","pattern":"[a-\\d]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[48,57],[97,97]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1},{"capture_units":[[97]],"from":1,"indices":[[1,2]],"name_units":null,"to":2}],"name":"generated-31-g-2","pattern":"[a-\\d]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[48,57],[97,97]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1},{"capture_units":[[97]],"from":1,"indices":[[1,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1},{"capture_units":[[97]],"from":2,"indices":[[2,3]],"name_units":null,"to":3}],"name":"generated-31-g-4","pattern":"[a-\\d]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[48,57],[97,97]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1},{"capture_units":[[97]],"from":2,"indices":[[2,3]],"to":3}],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-31-g-7","pattern":"[a-\\d]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[45,45],[48,57],[97,97]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"","input":"","matches":[],"name":"generated-32--0","pattern":"a$"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[97,97]],[],false,false],["assert","end",false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[{"capture_units":[[97]],"from":1,"indices":[[1,2]],"name_units":null,"to":2}],"name":"generated-32--2","pattern":"a$"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[97,97]],[],false,false],["assert","end",false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":1,"indices":[[1,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[],"name":"generated-32--4","pattern":"a$"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[97,97]],[],false,false],["assert","end",false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[],"name":"generated-32--7","pattern":"a$"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[97,97]],[],false,false],["assert","end",false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[],"name":"generated-32-g-0","pattern":"a$"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[97,97]],[],false,false],["assert","end",false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[{"capture_units":[[97]],"from":1,"indices":[[1,2]],"name_units":null,"to":2}],"name":"generated-32-g-2","pattern":"a$"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[97,97]],[],false,false],["assert","end",false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":1,"indices":[[1,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[],"name":"generated-32-g-4","pattern":"a$"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[97,97]],[],false,false],["assert","end",false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[],"name":"generated-32-g-7","pattern":"a$"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[97,97]],[],false,false],["assert","end",false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"u","input":"","matches":[],"name":"generated-32-u-0","pattern":"a$"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[97,97]],[],false,true],["assert","end",false,false],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"u","input":"aa","matches":[{"capture_units":[[97]],"from":1,"indices":[[1,2]],"name_units":null,"to":2}],"name":"generated-32-u-2","pattern":"a$"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[97,97]],[],false,true],["assert","end",false,false],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":1,"indices":[[1,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"u","input":"abac","matches":[],"name":"generated-32-u-4","pattern":"a$"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[97,97]],[],false,true],["assert","end",false,false],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"u","input":"aKſ😀\r\n","matches":[],"name":"generated-32-u-7","pattern":"a$"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[97,97]],[],false,true],["assert","end",false,false],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"iu","input":"","matches":[],"name":"generated-32-iu-0","pattern":"a$"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[65,65],[97,97]],[],true,true],["assert","end",false,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"iu","input":"aa","matches":[{"capture_units":[[97]],"from":1,"indices":[[1,2]],"name_units":null,"to":2}],"name":"generated-32-iu-2","pattern":"a$"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[65,65],[97,97]],[],true,true],["assert","end",false,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":1,"indices":[[1,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"iu","input":"abac","matches":[],"name":"generated-32-iu-4","pattern":"a$"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[65,65],[97,97]],[],true,true],["assert","end",false,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"iu","input":"aKſ😀\r\n","matches":[],"name":"generated-32-iu-7","pattern":"a$"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[65,65],[97,97]],[],true,true],["assert","end",false,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"v","input":"","matches":[],"name":"generated-32-v-0","pattern":"a$"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[97,97]],[],false,true],["assert","end",false,false],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"v","input":"aa","matches":[{"capture_units":[[97]],"from":1,"indices":[[1,2]],"name_units":null,"to":2}],"name":"generated-32-v-2","pattern":"a$"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[97,97]],[],false,true],["assert","end",false,false],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":1,"indices":[[1,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"v","input":"abac","matches":[],"name":"generated-32-v-4","pattern":"a$"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[97,97]],[],false,true],["assert","end",false,false],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"v","input":"aKſ😀\r\n","matches":[],"name":"generated-32-v-7","pattern":"a$"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[97,97]],[],false,true],["assert","end",false,false],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"","input":"","matches":[],"name":"generated-33--0","pattern":"^a"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","start",false,false],["set",[[97,97]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-33--2","pattern":"^a"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","start",false,false],["set",[[97,97]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-33--4","pattern":"^a"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","start",false,false],["set",[[97,97]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-33--7","pattern":"^a"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","start",false,false],["set",[[97,97]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[],"name":"generated-33-g-0","pattern":"^a"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","start",false,false],["set",[[97,97]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-33-g-2","pattern":"^a"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","start",false,false],["set",[[97,97]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-33-g-4","pattern":"^a"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","start",false,false],["set",[[97,97]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-33-g-7","pattern":"^a"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","start",false,false],["set",[[97,97]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"u","input":"","matches":[],"name":"generated-33-u-0","pattern":"^a"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","start",false,false],["set",[[97,97]],[],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"u","input":"aa","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-33-u-2","pattern":"^a"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","start",false,false],["set",[[97,97]],[],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aa"},{"case":{"flags":"u","input":"abac","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-33-u-4","pattern":"^a"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","start",false,false],["set",[[97,97]],[],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"u","input":"aKſ😀\r\n","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-33-u-7","pattern":"^a"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","start",false,false],["set",[[97,97]],[],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"iu","input":"","matches":[],"name":"generated-33-iu-0","pattern":"^a"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","start",false,true],["set",[[65,65],[97,97]],[],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"iu","input":"aa","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-33-iu-2","pattern":"^a"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","start",false,true],["set",[[65,65],[97,97]],[],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aa"},{"case":{"flags":"iu","input":"abac","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-33-iu-4","pattern":"^a"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","start",false,true],["set",[[65,65],[97,97]],[],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"iu","input":"aKſ😀\r\n","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-33-iu-7","pattern":"^a"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","start",false,true],["set",[[65,65],[97,97]],[],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"v","input":"","matches":[],"name":"generated-33-v-0","pattern":"^a"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","start",false,false],["set",[[97,97]],[],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"v","input":"aa","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-33-v-2","pattern":"^a"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","start",false,false],["set",[[97,97]],[],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aa"},{"case":{"flags":"v","input":"abac","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-33-v-4","pattern":"^a"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","start",false,false],["set",[[97,97]],[],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"abac"},{"case":{"flags":"v","input":"aKſ😀\r\n","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-33-v-7","pattern":"^a"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","start",false,false],["set",[[97,97]],[],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"","input":"","matches":[],"name":"generated-34--0","pattern":"\\b.\\b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","word",false,false],["set",[[0,9],[11,12],[14,8231],[8234,65535]],[],false,false],["assert","word",false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[],"name":"generated-34--2","pattern":"\\b.\\b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","word",false,false],["set",[[0,9],[11,12],[14,8231],[8234,65535]],[],false,false],["assert","word",false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[],"name":"generated-34--4","pattern":"\\b.\\b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","word",false,false],["set",[[0,9],[11,12],[14,8231],[8234,65535]],[],false,false],["assert","word",false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-34--7","pattern":"\\b.\\b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","word",false,false],["set",[[0,9],[11,12],[14,8231],[8234,65535]],[],false,false],["assert","word",false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[],"name":"generated-34-g-0","pattern":"\\b.\\b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","word",false,false],["set",[[0,9],[11,12],[14,8231],[8234,65535]],[],false,false],["assert","word",false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[],"name":"generated-34-g-2","pattern":"\\b.\\b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","word",false,false],["set",[[0,9],[11,12],[14,8231],[8234,65535]],[],false,false],["assert","word",false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[],"name":"generated-34-g-4","pattern":"\\b.\\b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","word",false,false],["set",[[0,9],[11,12],[14,8231],[8234,65535]],[],false,false],["assert","word",false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-34-g-7","pattern":"\\b.\\b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","word",false,false],["set",[[0,9],[11,12],[14,8231],[8234,65535]],[],false,false],["assert","word",false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"u","input":"","matches":[],"name":"generated-34-u-0","pattern":"\\b.\\b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","word",false,false],["set",[[0,9],[11,12],[14,8231],[8234,1114111]],[],false,true],["assert","word",false,false],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"u","input":"aa","matches":[],"name":"generated-34-u-2","pattern":"\\b.\\b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","word",false,false],["set",[[0,9],[11,12],[14,8231],[8234,1114111]],[],false,true],["assert","word",false,false],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"u","input":"abac","matches":[],"name":"generated-34-u-4","pattern":"\\b.\\b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","word",false,false],["set",[[0,9],[11,12],[14,8231],[8234,1114111]],[],false,true],["assert","word",false,false],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"u","input":"aKſ😀\r\n","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-34-u-7","pattern":"\\b.\\b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","word",false,false],["set",[[0,9],[11,12],[14,8231],[8234,1114111]],[],false,true],["assert","word",false,false],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"iu","input":"","matches":[],"name":"generated-34-iu-0","pattern":"\\b.\\b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","word",false,true],["set",[[0,9],[11,12],[14,8231],[8234,1114111]],[],true,true],["assert","word",false,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"iu","input":"aa","matches":[],"name":"generated-34-iu-2","pattern":"\\b.\\b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","word",false,true],["set",[[0,9],[11,12],[14,8231],[8234,1114111]],[],true,true],["assert","word",false,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"iu","input":"abac","matches":[],"name":"generated-34-iu-4","pattern":"\\b.\\b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","word",false,true],["set",[[0,9],[11,12],[14,8231],[8234,1114111]],[],true,true],["assert","word",false,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"iu","input":"aKſ😀\r\n","matches":[],"name":"generated-34-iu-7","pattern":"\\b.\\b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","word",false,true],["set",[[0,9],[11,12],[14,8231],[8234,1114111]],[],true,true],["assert","word",false,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"v","input":"","matches":[],"name":"generated-34-v-0","pattern":"\\b.\\b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","word",false,false],["set",[[0,9],[11,12],[14,8231],[8234,1114111]],[],false,true],["assert","word",false,false],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"v","input":"aa","matches":[],"name":"generated-34-v-2","pattern":"\\b.\\b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","word",false,false],["set",[[0,9],[11,12],[14,8231],[8234,1114111]],[],false,true],["assert","word",false,false],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"v","input":"abac","matches":[],"name":"generated-34-v-4","pattern":"\\b.\\b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","word",false,false],["set",[[0,9],[11,12],[14,8231],[8234,1114111]],[],false,true],["assert","word",false,false],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"v","input":"aKſ😀\r\n","matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"name_units":null,"to":1}],"name":"generated-34-v-7","pattern":"\\b.\\b"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["assert","word",false,false],["set",[[0,9],[11,12],[14,8231],[8234,1114111]],[],false,true],["assert","word",false,false],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97]],"from":0,"indices":[[0,1]],"to":1}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"","input":"","matches":[],"name":"generated-35--0","pattern":"[\\q{a|ab}]a"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[97,98],[113,113],[123,125]],[],false,false],["set",[[97,97]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[{"capture_units":[[97,97]],"from":0,"indices":[[0,2]],"name_units":null,"to":2}],"name":"generated-35--2","pattern":"[\\q{a|ab}]a"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[97,98],[113,113],[123,125]],[],false,false],["set",[[97,97]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97]],"from":0,"indices":[[0,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[{"capture_units":[[98,97]],"from":1,"indices":[[1,3]],"name_units":null,"to":3}],"name":"generated-35--4","pattern":"[\\q{a|ab}]a"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[97,98],[113,113],[123,125]],[],false,false],["set",[[97,97]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[98,97]],"from":1,"indices":[[1,3]],"to":3}],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[],"name":"generated-35--7","pattern":"[\\q{a|ab}]a"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[97,98],[113,113],[123,125]],[],false,false],["set",[[97,97]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[],"name":"generated-35-g-0","pattern":"[\\q{a|ab}]a"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[97,98],[113,113],[123,125]],[],false,false],["set",[[97,97]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[{"capture_units":[[97,97]],"from":0,"indices":[[0,2]],"name_units":null,"to":2}],"name":"generated-35-g-2","pattern":"[\\q{a|ab}]a"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[97,98],[113,113],[123,125]],[],false,false],["set",[[97,97]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97]],"from":0,"indices":[[0,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[{"capture_units":[[98,97]],"from":1,"indices":[[1,3]],"name_units":null,"to":3}],"name":"generated-35-g-4","pattern":"[\\q{a|ab}]a"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[97,98],[113,113],[123,125]],[],false,false],["set",[[97,97]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[98,97]],"from":1,"indices":[[1,3]],"to":3}],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[],"name":"generated-35-g-7","pattern":"[\\q{a|ab}]a"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[97,98],[113,113],[123,125]],[],false,false],["set",[[97,97]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"v","input":"","matches":[],"name":"generated-35-v-0","pattern":"[\\q{a|ab}]a"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[97,97]],[[97,98]],false,true],["set",[[97,97]],[],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"v","input":"aa","matches":[{"capture_units":[[97,97]],"from":0,"indices":[[0,2]],"name_units":null,"to":2}],"name":"generated-35-v-2","pattern":"[\\q{a|ab}]a"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[97,97]],[[97,98]],false,true],["set",[[97,97]],[],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97]],"from":0,"indices":[[0,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"v","input":"abac","matches":[{"capture_units":[[97,98,97]],"from":0,"indices":[[0,3]],"name_units":null,"to":3}],"name":"generated-35-v-4","pattern":"[\\q{a|ab}]a"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[97,97]],[[97,98]],false,true],["set",[[97,97]],[],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,98,97]],"from":0,"indices":[[0,3]],"to":3}],"rust_output":"abac"},{"case":{"flags":"v","input":"aKſ😀\r\n","matches":[],"name":"generated-35-v-7","pattern":"[\\q{a|ab}]a"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[97,97]],[[97,98]],false,true],["set",[[97,97]],[],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"","input":"","matches":[],"name":"generated-36--0","pattern":"[\\q{}]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[113,113],[123,123],[125,125]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[],"name":"generated-36--2","pattern":"[\\q{}]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[113,113],[123,123],[125,125]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[],"name":"generated-36--4","pattern":"[\\q{}]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[113,113],[123,123],[125,125]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[],"name":"generated-36--7","pattern":"[\\q{}]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[113,113],[123,123],[125,125]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[],"name":"generated-36-g-0","pattern":"[\\q{}]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[113,113],[123,123],[125,125]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[],"name":"generated-36-g-2","pattern":"[\\q{}]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[113,113],[123,123],[125,125]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[],"name":"generated-36-g-4","pattern":"[\\q{}]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[113,113],[123,123],[125,125]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[],"name":"generated-36-g-7","pattern":"[\\q{}]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[113,113],[123,123],[125,125]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"v","input":"","matches":[{"capture_units":[[]],"from":0,"indices":[[0,0]],"name_units":null,"to":0}],"name":"generated-36-v-0","pattern":"[\\q{}]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[],[[]],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[]],"from":0,"indices":[[0,0]],"to":0}],"rust_output":""},{"case":{"flags":"v","input":"aa","matches":[{"capture_units":[[]],"from":0,"indices":[[0,0]],"name_units":null,"to":0}],"name":"generated-36-v-2","pattern":"[\\q{}]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[],[[]],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[]],"from":0,"indices":[[0,0]],"to":0}],"rust_output":"aa"},{"case":{"flags":"v","input":"abac","matches":[{"capture_units":[[]],"from":0,"indices":[[0,0]],"name_units":null,"to":0}],"name":"generated-36-v-4","pattern":"[\\q{}]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[],[[]],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[]],"from":0,"indices":[[0,0]],"to":0}],"rust_output":"abac"},{"case":{"flags":"v","input":"aKſ😀\r\n","matches":[{"capture_units":[[]],"from":0,"indices":[[0,0]],"name_units":null,"to":0}],"name":"generated-36-v-7","pattern":"[\\q{}]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[],[[]],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[]],"from":0,"indices":[[0,0]],"to":0}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"","input":"","matches":[],"name":"generated-37--0","pattern":"[[a-z]--[aeiou]]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[91,91],[97,122]],[],false,false],["set",[[45,45]],[],false,false],["set",[[45,45]],[],false,false],["set",[[97,97],[101,101],[105,105],[111,111],[117,117]],[],false,false],["set",[[93,93]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[],"name":"generated-37--2","pattern":"[[a-z]--[aeiou]]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[91,91],[97,122]],[],false,false],["set",[[45,45]],[],false,false],["set",[[45,45]],[],false,false],["set",[[97,97],[101,101],[105,105],[111,111],[117,117]],[],false,false],["set",[[93,93]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[],"name":"generated-37--4","pattern":"[[a-z]--[aeiou]]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[91,91],[97,122]],[],false,false],["set",[[45,45]],[],false,false],["set",[[45,45]],[],false,false],["set",[[97,97],[101,101],[105,105],[111,111],[117,117]],[],false,false],["set",[[93,93]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[],"name":"generated-37--7","pattern":"[[a-z]--[aeiou]]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[91,91],[97,122]],[],false,false],["set",[[45,45]],[],false,false],["set",[[45,45]],[],false,false],["set",[[97,97],[101,101],[105,105],[111,111],[117,117]],[],false,false],["set",[[93,93]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[],"name":"generated-37-g-0","pattern":"[[a-z]--[aeiou]]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[91,91],[97,122]],[],false,false],["set",[[45,45]],[],false,false],["set",[[45,45]],[],false,false],["set",[[97,97],[101,101],[105,105],[111,111],[117,117]],[],false,false],["set",[[93,93]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[],"name":"generated-37-g-2","pattern":"[[a-z]--[aeiou]]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[91,91],[97,122]],[],false,false],["set",[[45,45]],[],false,false],["set",[[45,45]],[],false,false],["set",[[97,97],[101,101],[105,105],[111,111],[117,117]],[],false,false],["set",[[93,93]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[],"name":"generated-37-g-4","pattern":"[[a-z]--[aeiou]]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[91,91],[97,122]],[],false,false],["set",[[45,45]],[],false,false],["set",[[45,45]],[],false,false],["set",[[97,97],[101,101],[105,105],[111,111],[117,117]],[],false,false],["set",[[93,93]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[],"name":"generated-37-g-7","pattern":"[[a-z]--[aeiou]]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[91,91],[97,122]],[],false,false],["set",[[45,45]],[],false,false],["set",[[45,45]],[],false,false],["set",[[97,97],[101,101],[105,105],[111,111],[117,117]],[],false,false],["set",[[93,93]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"v","input":"","matches":[],"name":"generated-37-v-0","pattern":"[[a-z]--[aeiou]]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[98,100],[102,104],[106,110],[112,116],[118,122]],[],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"v","input":"aa","matches":[],"name":"generated-37-v-2","pattern":"[[a-z]--[aeiou]]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[98,100],[102,104],[106,110],[112,116],[118,122]],[],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"v","input":"abac","matches":[{"capture_units":[[98]],"from":1,"indices":[[1,2]],"name_units":null,"to":2}],"name":"generated-37-v-4","pattern":"[[a-z]--[aeiou]]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[98,100],[102,104],[106,110],[112,116],[118,122]],[],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[98]],"from":1,"indices":[[1,2]],"to":2}],"rust_output":"abac"},{"case":{"flags":"v","input":"aKſ😀\r\n","matches":[],"name":"generated-37-v-7","pattern":"[[a-z]--[aeiou]]"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[98,100],[102,104],[106,110],[112,116],[118,122]],[],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"","input":"","matches":[],"name":"generated-38--0","pattern":"(?i:a)(?-i:b)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[65,65],[97,97]],[],true,false],["set",[[98,98]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[],"name":"generated-38--2","pattern":"(?i:a)(?-i:b)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[65,65],[97,97]],[],true,false],["set",[[98,98]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[{"capture_units":[[97,98]],"from":0,"indices":[[0,2]],"name_units":null,"to":2}],"name":"generated-38--4","pattern":"(?i:a)(?-i:b)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[65,65],[97,97]],[],true,false],["set",[[98,98]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,98]],"from":0,"indices":[[0,2]],"to":2}],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[],"name":"generated-38--7","pattern":"(?i:a)(?-i:b)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[65,65],[97,97]],[],true,false],["set",[[98,98]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[],"name":"generated-38-g-0","pattern":"(?i:a)(?-i:b)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[65,65],[97,97]],[],true,false],["set",[[98,98]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[],"name":"generated-38-g-2","pattern":"(?i:a)(?-i:b)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[65,65],[97,97]],[],true,false],["set",[[98,98]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[{"capture_units":[[97,98]],"from":0,"indices":[[0,2]],"name_units":null,"to":2}],"name":"generated-38-g-4","pattern":"(?i:a)(?-i:b)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[65,65],[97,97]],[],true,false],["set",[[98,98]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,98]],"from":0,"indices":[[0,2]],"to":2}],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[],"name":"generated-38-g-7","pattern":"(?i:a)(?-i:b)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[65,65],[97,97]],[],true,false],["set",[[98,98]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"u","input":"","matches":[],"name":"generated-38-u-0","pattern":"(?i:a)(?-i:b)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[65,65],[97,97]],[],true,true],["set",[[98,98]],[],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"u","input":"aa","matches":[],"name":"generated-38-u-2","pattern":"(?i:a)(?-i:b)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[65,65],[97,97]],[],true,true],["set",[[98,98]],[],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"u","input":"abac","matches":[{"capture_units":[[97,98]],"from":0,"indices":[[0,2]],"name_units":null,"to":2}],"name":"generated-38-u-4","pattern":"(?i:a)(?-i:b)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[65,65],[97,97]],[],true,true],["set",[[98,98]],[],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,98]],"from":0,"indices":[[0,2]],"to":2}],"rust_output":"abac"},{"case":{"flags":"u","input":"aKſ😀\r\n","matches":[],"name":"generated-38-u-7","pattern":"(?i:a)(?-i:b)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[65,65],[97,97]],[],true,true],["set",[[98,98]],[],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"iu","input":"","matches":[],"name":"generated-38-iu-0","pattern":"(?i:a)(?-i:b)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[65,65],[97,97]],[],true,true],["set",[[98,98]],[],false,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"iu","input":"aa","matches":[],"name":"generated-38-iu-2","pattern":"(?i:a)(?-i:b)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[65,65],[97,97]],[],true,true],["set",[[98,98]],[],false,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"iu","input":"abac","matches":[{"capture_units":[[97,98]],"from":0,"indices":[[0,2]],"name_units":null,"to":2}],"name":"generated-38-iu-4","pattern":"(?i:a)(?-i:b)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[65,65],[97,97]],[],true,true],["set",[[98,98]],[],false,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,98]],"from":0,"indices":[[0,2]],"to":2}],"rust_output":"abac"},{"case":{"flags":"iu","input":"aKſ😀\r\n","matches":[],"name":"generated-38-iu-7","pattern":"(?i:a)(?-i:b)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[65,65],[97,97]],[],true,true],["set",[[98,98]],[],false,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"v","input":"","matches":[],"name":"generated-38-v-0","pattern":"(?i:a)(?-i:b)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[65,65],[97,97]],[],true,true],["set",[[98,98]],[],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"v","input":"aa","matches":[],"name":"generated-38-v-2","pattern":"(?i:a)(?-i:b)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[65,65],[97,97]],[],true,true],["set",[[98,98]],[],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aa"},{"case":{"flags":"v","input":"abac","matches":[{"capture_units":[[97,98]],"from":0,"indices":[[0,2]],"name_units":null,"to":2}],"name":"generated-38-v-4","pattern":"(?i:a)(?-i:b)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[65,65],[97,97]],[],true,true],["set",[[98,98]],[],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,98]],"from":0,"indices":[[0,2]],"to":2}],"rust_output":"abac"},{"case":{"flags":"v","input":"aKſ😀\r\n","matches":[],"name":"generated-38-v-7","pattern":"(?i:a)(?-i:b)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[65,65],[97,97]],[],true,true],["set",[[98,98]],[],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"","input":"","matches":[],"name":"generated-39--0","pattern":"(?s:.)(?-s:.)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,65535]],[],false,false],["set",[[0,9],[11,12],[14,8231],[8234,65535]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"","input":"aa","matches":[{"capture_units":[[97,97]],"from":0,"indices":[[0,2]],"name_units":null,"to":2}],"name":"generated-39--2","pattern":"(?s:.)(?-s:.)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,65535]],[],false,false],["set",[[0,9],[11,12],[14,8231],[8234,65535]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97]],"from":0,"indices":[[0,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"","input":"abac","matches":[{"capture_units":[[97,98]],"from":0,"indices":[[0,2]],"name_units":null,"to":2}],"name":"generated-39--4","pattern":"(?s:.)(?-s:.)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,65535]],[],false,false],["set",[[0,9],[11,12],[14,8231],[8234,65535]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,98]],"from":0,"indices":[[0,2]],"to":2}],"rust_output":"abac"},{"case":{"flags":"","input":"aKſ😀\r\n","matches":[{"capture_units":[[97,8490]],"from":0,"indices":[[0,2]],"name_units":null,"to":2}],"name":"generated-39--7","pattern":"(?s:.)(?-s:.)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,65535]],[],false,false],["set",[[0,9],[11,12],[14,8231],[8234,65535]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,8490]],"from":0,"indices":[[0,2]],"to":2}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"g","input":"","matches":[],"name":"generated-39-g-0","pattern":"(?s:.)(?-s:.)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,65535]],[],false,false],["set",[[0,9],[11,12],[14,8231],[8234,65535]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"g","input":"aa","matches":[{"capture_units":[[97,97]],"from":0,"indices":[[0,2]],"name_units":null,"to":2}],"name":"generated-39-g-2","pattern":"(?s:.)(?-s:.)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,65535]],[],false,false],["set",[[0,9],[11,12],[14,8231],[8234,65535]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97]],"from":0,"indices":[[0,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"g","input":"abac","matches":[{"capture_units":[[97,98]],"from":0,"indices":[[0,2]],"name_units":null,"to":2},{"capture_units":[[97,99]],"from":2,"indices":[[2,4]],"name_units":null,"to":4}],"name":"generated-39-g-4","pattern":"(?s:.)(?-s:.)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,65535]],[],false,false],["set",[[0,9],[11,12],[14,8231],[8234,65535]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,98]],"from":0,"indices":[[0,2]],"to":2},{"capture_units":[[97,99]],"from":2,"indices":[[2,4]],"to":4}],"rust_output":"abac"},{"case":{"flags":"g","input":"aKſ😀\r\n","matches":[{"capture_units":[[97,8490]],"from":0,"indices":[[0,2]],"name_units":null,"to":2},{"capture_units":[[383,55357]],"from":2,"indices":[[2,4]],"name_units":null,"to":4}],"name":"generated-39-g-7","pattern":"(?s:.)(?-s:.)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,65535]],[],false,false],["set",[[0,9],[11,12],[14,8231],[8234,65535]],[],false,false],["accept"]],"flags":"g","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,8490]],"from":0,"indices":[[0,2]],"to":2},{"capture_units":[[383,55357]],"from":2,"indices":[[2,4]],"to":4}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"u","input":"","matches":[],"name":"generated-39-u-0","pattern":"(?s:.)(?-s:.)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,1114111]],[],false,true],["set",[[0,9],[11,12],[14,8231],[8234,1114111]],[],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"u","input":"aa","matches":[{"capture_units":[[97,97]],"from":0,"indices":[[0,2]],"name_units":null,"to":2}],"name":"generated-39-u-2","pattern":"(?s:.)(?-s:.)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,1114111]],[],false,true],["set",[[0,9],[11,12],[14,8231],[8234,1114111]],[],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97]],"from":0,"indices":[[0,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"u","input":"abac","matches":[{"capture_units":[[97,98]],"from":0,"indices":[[0,2]],"name_units":null,"to":2}],"name":"generated-39-u-4","pattern":"(?s:.)(?-s:.)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,1114111]],[],false,true],["set",[[0,9],[11,12],[14,8231],[8234,1114111]],[],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,98]],"from":0,"indices":[[0,2]],"to":2}],"rust_output":"abac"},{"case":{"flags":"u","input":"aKſ😀\r\n","matches":[{"capture_units":[[97,8490]],"from":0,"indices":[[0,2]],"name_units":null,"to":2}],"name":"generated-39-u-7","pattern":"(?s:.)(?-s:.)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,1114111]],[],false,true],["set",[[0,9],[11,12],[14,8231],[8234,1114111]],[],false,true],["accept"]],"flags":"u","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,8490]],"from":0,"indices":[[0,2]],"to":2}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"iu","input":"","matches":[],"name":"generated-39-iu-0","pattern":"(?s:.)(?-s:.)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,1114111]],[],true,true],["set",[[0,9],[11,12],[14,8231],[8234,1114111]],[],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"iu","input":"aa","matches":[{"capture_units":[[97,97]],"from":0,"indices":[[0,2]],"name_units":null,"to":2}],"name":"generated-39-iu-2","pattern":"(?s:.)(?-s:.)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,1114111]],[],true,true],["set",[[0,9],[11,12],[14,8231],[8234,1114111]],[],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97]],"from":0,"indices":[[0,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"iu","input":"abac","matches":[{"capture_units":[[97,98]],"from":0,"indices":[[0,2]],"name_units":null,"to":2}],"name":"generated-39-iu-4","pattern":"(?s:.)(?-s:.)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,1114111]],[],true,true],["set",[[0,9],[11,12],[14,8231],[8234,1114111]],[],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,98]],"from":0,"indices":[[0,2]],"to":2}],"rust_output":"abac"},{"case":{"flags":"iu","input":"aKſ😀\r\n","matches":[{"capture_units":[[97,8490]],"from":0,"indices":[[0,2]],"name_units":null,"to":2}],"name":"generated-39-iu-7","pattern":"(?s:.)(?-s:.)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,1114111]],[],true,true],["set",[[0,9],[11,12],[14,8231],[8234,1114111]],[],true,true],["accept"]],"flags":"iu","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,8490]],"from":0,"indices":[[0,2]],"to":2}],"rust_output":"aKſ😀\r\n"},{"case":{"flags":"v","input":"","matches":[],"name":"generated-39-v-0","pattern":"(?s:.)(?-s:.)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,1114111]],[],false,true],["set",[[0,9],[11,12],[14,8231],[8234,1114111]],[],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":""},{"case":{"flags":"v","input":"aa","matches":[{"capture_units":[[97,97]],"from":0,"indices":[[0,2]],"name_units":null,"to":2}],"name":"generated-39-v-2","pattern":"(?s:.)(?-s:.)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,1114111]],[],false,true],["set",[[0,9],[11,12],[14,8231],[8234,1114111]],[],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,97]],"from":0,"indices":[[0,2]],"to":2}],"rust_output":"aa"},{"case":{"flags":"v","input":"abac","matches":[{"capture_units":[[97,98]],"from":0,"indices":[[0,2]],"name_units":null,"to":2}],"name":"generated-39-v-4","pattern":"(?s:.)(?-s:.)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,1114111]],[],false,true],["set",[[0,9],[11,12],[14,8231],[8234,1114111]],[],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,98]],"from":0,"indices":[[0,2]],"to":2}],"rust_output":"abac"},{"case":{"flags":"v","input":"aKſ😀\r\n","matches":[{"capture_units":[[97,8490]],"from":0,"indices":[[0,2]],"name_units":null,"to":2}],"name":"generated-39-v-7","pattern":"(?s:.)(?-s:.)"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,1114111]],[],false,true],["set",[[0,9],[11,12],[14,8231],[8234,1114111]],[],false,true],["accept"]],"flags":"v","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,8490]],"from":0,"indices":[[0,2]],"to":2}],"rust_output":"aKſ😀\r\n"},{"case":{"error":true,"flags":"","input":"","name":"invalid-637","pattern":"(?<x>a)(?<x>b)"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Duplicated capturing group names"},{"case":{"error":true,"flags":"u","input":"","name":"invalid-638","pattern":"\\k<missing>"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid named reference"},{"case":{"error":true,"flags":"u","input":"","name":"invalid-639","pattern":"\\2(a)"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"u","input":"","name":"invalid-640","pattern":"[a-\\d]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Character class range with invalid atom"},{"case":{"error":true,"flags":"uv","input":"","name":"invalid-641","pattern":"a"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid unicode flags combination `u` and `v`"},{"case":{"error":true,"flags":"gg","input":"","name":"invalid-642","pattern":"a"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Duplicated flag: `g` found"},{"case":{"error":true,"flags":"","input":"","name":"invalid-643","pattern":"(?i-i:a)"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid modifiers"},{"case":{"error":true,"flags":"v","input":"","name":"invalid-644","pattern":"[a&&b--c]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Unexpected character inside of class intersection"},{"case":{"error":true,"flags":"v","input":"","name":"invalid-645","pattern":"[^\\q{ab}]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid character class with strings unicode property"},{"case":{"error":true,"flags":"u","input":"","name":"generated-11-u-0","pattern":"(?=a)?"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Lone quantifier found, expected with `Atom`"},{"case":{"error":true,"flags":"u","input":"aa","name":"generated-11-u-2","pattern":"(?=a)?"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Lone quantifier found, expected with `Atom`"},{"case":{"error":true,"flags":"u","input":"abac","name":"generated-11-u-4","pattern":"(?=a)?"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Lone quantifier found, expected with `Atom`"},{"case":{"error":true,"flags":"u","input":"aKſ😀\r\n","name":"generated-11-u-7","pattern":"(?=a)?"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Lone quantifier found, expected with `Atom`"},{"case":{"error":true,"flags":"iu","input":"","name":"generated-11-iu-0","pattern":"(?=a)?"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Lone quantifier found, expected with `Atom`"},{"case":{"error":true,"flags":"iu","input":"aa","name":"generated-11-iu-2","pattern":"(?=a)?"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Lone quantifier found, expected with `Atom`"},{"case":{"error":true,"flags":"iu","input":"abac","name":"generated-11-iu-4","pattern":"(?=a)?"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Lone quantifier found, expected with `Atom`"},{"case":{"error":true,"flags":"iu","input":"aKſ😀\r\n","name":"generated-11-iu-7","pattern":"(?=a)?"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Lone quantifier found, expected with `Atom`"},{"case":{"error":true,"flags":"v","input":"","name":"generated-11-v-0","pattern":"(?=a)?"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Lone quantifier found, expected with `Atom`"},{"case":{"error":true,"flags":"v","input":"aa","name":"generated-11-v-2","pattern":"(?=a)?"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Lone quantifier found, expected with `Atom`"},{"case":{"error":true,"flags":"v","input":"abac","name":"generated-11-v-4","pattern":"(?=a)?"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Lone quantifier found, expected with `Atom`"},{"case":{"error":true,"flags":"v","input":"aKſ😀\r\n","name":"generated-11-v-7","pattern":"(?=a)?"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Lone quantifier found, expected with `Atom`"},{"case":{"error":true,"flags":"u","input":"","name":"generated-12-u-0","pattern":"(?=(a)){2}"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Lone quantifier found, expected with `Atom`"},{"case":{"error":true,"flags":"u","input":"aa","name":"generated-12-u-2","pattern":"(?=(a)){2}"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Lone quantifier found, expected with `Atom`"},{"case":{"error":true,"flags":"u","input":"abac","name":"generated-12-u-4","pattern":"(?=(a)){2}"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Lone quantifier found, expected with `Atom`"},{"case":{"error":true,"flags":"u","input":"aKſ😀\r\n","name":"generated-12-u-7","pattern":"(?=(a)){2}"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Lone quantifier found, expected with `Atom`"},{"case":{"error":true,"flags":"iu","input":"","name":"generated-12-iu-0","pattern":"(?=(a)){2}"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Lone quantifier found, expected with `Atom`"},{"case":{"error":true,"flags":"iu","input":"aa","name":"generated-12-iu-2","pattern":"(?=(a)){2}"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Lone quantifier found, expected with `Atom`"},{"case":{"error":true,"flags":"iu","input":"abac","name":"generated-12-iu-4","pattern":"(?=(a)){2}"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Lone quantifier found, expected with `Atom`"},{"case":{"error":true,"flags":"iu","input":"aKſ😀\r\n","name":"generated-12-iu-7","pattern":"(?=(a)){2}"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Lone quantifier found, expected with `Atom`"},{"case":{"error":true,"flags":"v","input":"","name":"generated-12-v-0","pattern":"(?=(a)){2}"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Lone quantifier found, expected with `Atom`"},{"case":{"error":true,"flags":"v","input":"aa","name":"generated-12-v-2","pattern":"(?=(a)){2}"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Lone quantifier found, expected with `Atom`"},{"case":{"error":true,"flags":"v","input":"abac","name":"generated-12-v-4","pattern":"(?=(a)){2}"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Lone quantifier found, expected with `Atom`"},{"case":{"error":true,"flags":"v","input":"aKſ😀\r\n","name":"generated-12-v-7","pattern":"(?=(a)){2}"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Lone quantifier found, expected with `Atom`"},{"case":{"error":true,"flags":"u","input":"","name":"generated-15-u-0","pattern":"\\11"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"u","input":"aa","name":"generated-15-u-2","pattern":"\\11"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"u","input":"abac","name":"generated-15-u-4","pattern":"\\11"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"u","input":"aKſ😀\r\n","name":"generated-15-u-7","pattern":"\\11"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"iu","input":"","name":"generated-15-iu-0","pattern":"\\11"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"iu","input":"aa","name":"generated-15-iu-2","pattern":"\\11"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"iu","input":"abac","name":"generated-15-iu-4","pattern":"\\11"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"iu","input":"aKſ😀\r\n","name":"generated-15-iu-7","pattern":"\\11"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"v","input":"","name":"generated-15-v-0","pattern":"\\11"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"v","input":"aa","name":"generated-15-v-2","pattern":"\\11"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"v","input":"abac","name":"generated-15-v-4","pattern":"\\11"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"v","input":"aKſ😀\r\n","name":"generated-15-v-7","pattern":"\\11"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"u","input":"","name":"generated-16-u-0","pattern":"\\8"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"u","input":"aa","name":"generated-16-u-2","pattern":"\\8"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"u","input":"abac","name":"generated-16-u-4","pattern":"\\8"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"u","input":"aKſ😀\r\n","name":"generated-16-u-7","pattern":"\\8"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"iu","input":"","name":"generated-16-iu-0","pattern":"\\8"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"iu","input":"aa","name":"generated-16-iu-2","pattern":"\\8"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"iu","input":"abac","name":"generated-16-iu-4","pattern":"\\8"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"iu","input":"aKſ😀\r\n","name":"generated-16-iu-7","pattern":"\\8"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"v","input":"","name":"generated-16-v-0","pattern":"\\8"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"v","input":"aa","name":"generated-16-v-2","pattern":"\\8"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"v","input":"abac","name":"generated-16-v-4","pattern":"\\8"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"v","input":"aKſ😀\r\n","name":"generated-16-v-7","pattern":"\\8"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"u","input":"","name":"generated-17-u-0","pattern":"\\9"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"u","input":"aa","name":"generated-17-u-2","pattern":"\\9"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"u","input":"abac","name":"generated-17-u-4","pattern":"\\9"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"u","input":"aKſ😀\r\n","name":"generated-17-u-7","pattern":"\\9"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"iu","input":"","name":"generated-17-iu-0","pattern":"\\9"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"iu","input":"aa","name":"generated-17-iu-2","pattern":"\\9"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"iu","input":"abac","name":"generated-17-iu-4","pattern":"\\9"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"iu","input":"aKſ😀\r\n","name":"generated-17-iu-7","pattern":"\\9"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"v","input":"","name":"generated-17-v-0","pattern":"\\9"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"v","input":"aa","name":"generated-17-v-2","pattern":"\\9"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"v","input":"abac","name":"generated-17-v-4","pattern":"\\9"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"v","input":"aKſ😀\r\n","name":"generated-17-v-7","pattern":"\\9"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"u","input":"","name":"generated-18-u-0","pattern":"\\08"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"u","input":"aa","name":"generated-18-u-2","pattern":"\\08"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"u","input":"abac","name":"generated-18-u-4","pattern":"\\08"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"u","input":"aKſ😀\r\n","name":"generated-18-u-7","pattern":"\\08"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"iu","input":"","name":"generated-18-iu-0","pattern":"\\08"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"iu","input":"aa","name":"generated-18-iu-2","pattern":"\\08"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"iu","input":"abac","name":"generated-18-iu-4","pattern":"\\08"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"iu","input":"aKſ😀\r\n","name":"generated-18-iu-7","pattern":"\\08"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"v","input":"","name":"generated-18-v-0","pattern":"\\08"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"v","input":"aa","name":"generated-18-v-2","pattern":"\\08"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"v","input":"abac","name":"generated-18-v-4","pattern":"\\08"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"v","input":"aKſ😀\r\n","name":"generated-18-v-7","pattern":"\\08"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"u","input":"","name":"generated-19-u-0","pattern":"\\377"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"u","input":"aa","name":"generated-19-u-2","pattern":"\\377"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"u","input":"abac","name":"generated-19-u-4","pattern":"\\377"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"u","input":"aKſ😀\r\n","name":"generated-19-u-7","pattern":"\\377"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"iu","input":"","name":"generated-19-iu-0","pattern":"\\377"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"iu","input":"aa","name":"generated-19-iu-2","pattern":"\\377"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"iu","input":"abac","name":"generated-19-iu-4","pattern":"\\377"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"iu","input":"aKſ😀\r\n","name":"generated-19-iu-7","pattern":"\\377"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"v","input":"","name":"generated-19-v-0","pattern":"\\377"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"v","input":"aa","name":"generated-19-v-2","pattern":"\\377"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"v","input":"abac","name":"generated-19-v-4","pattern":"\\377"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"v","input":"aKſ😀\r\n","name":"generated-19-v-7","pattern":"\\377"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"u","input":"","name":"generated-20-u-0","pattern":"\\400"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"u","input":"aa","name":"generated-20-u-2","pattern":"\\400"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"u","input":"abac","name":"generated-20-u-4","pattern":"\\400"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"u","input":"aKſ😀\r\n","name":"generated-20-u-7","pattern":"\\400"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"iu","input":"","name":"generated-20-iu-0","pattern":"\\400"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"iu","input":"aa","name":"generated-20-iu-2","pattern":"\\400"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"iu","input":"abac","name":"generated-20-iu-4","pattern":"\\400"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"iu","input":"aKſ😀\r\n","name":"generated-20-iu-7","pattern":"\\400"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"v","input":"","name":"generated-20-v-0","pattern":"\\400"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"v","input":"aa","name":"generated-20-v-2","pattern":"\\400"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"v","input":"abac","name":"generated-20-v-4","pattern":"\\400"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"v","input":"aKſ😀\r\n","name":"generated-20-v-7","pattern":"\\400"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid indexed reference"},{"case":{"error":true,"flags":"u","input":"","name":"generated-21-u-0","pattern":"[\\c0-\\c9]"},"compile_error":"Invalid ECMAScript Unicode class-control escape"},{"case":{"error":true,"flags":"u","input":"aa","name":"generated-21-u-2","pattern":"[\\c0-\\c9]"},"compile_error":"Invalid ECMAScript Unicode class-control escape"},{"case":{"error":true,"flags":"u","input":"abac","name":"generated-21-u-4","pattern":"[\\c0-\\c9]"},"compile_error":"Invalid ECMAScript Unicode class-control escape"},{"case":{"error":true,"flags":"u","input":"aKſ😀\r\n","name":"generated-21-u-7","pattern":"[\\c0-\\c9]"},"compile_error":"Invalid ECMAScript Unicode class-control escape"},{"case":{"error":true,"flags":"iu","input":"","name":"generated-21-iu-0","pattern":"[\\c0-\\c9]"},"compile_error":"Invalid ECMAScript Unicode class-control escape"},{"case":{"error":true,"flags":"iu","input":"aa","name":"generated-21-iu-2","pattern":"[\\c0-\\c9]"},"compile_error":"Invalid ECMAScript Unicode class-control escape"},{"case":{"error":true,"flags":"iu","input":"abac","name":"generated-21-iu-4","pattern":"[\\c0-\\c9]"},"compile_error":"Invalid ECMAScript Unicode class-control escape"},{"case":{"error":true,"flags":"iu","input":"aKſ😀\r\n","name":"generated-21-iu-7","pattern":"[\\c0-\\c9]"},"compile_error":"Invalid ECMAScript Unicode class-control escape"},{"case":{"error":true,"flags":"v","input":"","name":"generated-21-v-0","pattern":"[\\c0-\\c9]"},"compile_error":"Invalid ECMAScript Unicode class-control escape"},{"case":{"error":true,"flags":"v","input":"aa","name":"generated-21-v-2","pattern":"[\\c0-\\c9]"},"compile_error":"Invalid ECMAScript Unicode class-control escape"},{"case":{"error":true,"flags":"v","input":"abac","name":"generated-21-v-4","pattern":"[\\c0-\\c9]"},"compile_error":"Invalid ECMAScript Unicode class-control escape"},{"case":{"error":true,"flags":"v","input":"aKſ😀\r\n","name":"generated-21-v-7","pattern":"[\\c0-\\c9]"},"compile_error":"Invalid ECMAScript Unicode class-control escape"},{"case":{"error":true,"flags":"u","input":"","name":"generated-22-u-0","pattern":"[\\c_]"},"compile_error":"Invalid ECMAScript Unicode class-control escape"},{"case":{"error":true,"flags":"u","input":"aa","name":"generated-22-u-2","pattern":"[\\c_]"},"compile_error":"Invalid ECMAScript Unicode class-control escape"},{"case":{"error":true,"flags":"u","input":"abac","name":"generated-22-u-4","pattern":"[\\c_]"},"compile_error":"Invalid ECMAScript Unicode class-control escape"},{"case":{"error":true,"flags":"u","input":"aKſ😀\r\n","name":"generated-22-u-7","pattern":"[\\c_]"},"compile_error":"Invalid ECMAScript Unicode class-control escape"},{"case":{"error":true,"flags":"iu","input":"","name":"generated-22-iu-0","pattern":"[\\c_]"},"compile_error":"Invalid ECMAScript Unicode class-control escape"},{"case":{"error":true,"flags":"iu","input":"aa","name":"generated-22-iu-2","pattern":"[\\c_]"},"compile_error":"Invalid ECMAScript Unicode class-control escape"},{"case":{"error":true,"flags":"iu","input":"abac","name":"generated-22-iu-4","pattern":"[\\c_]"},"compile_error":"Invalid ECMAScript Unicode class-control escape"},{"case":{"error":true,"flags":"iu","input":"aKſ😀\r\n","name":"generated-22-iu-7","pattern":"[\\c_]"},"compile_error":"Invalid ECMAScript Unicode class-control escape"},{"case":{"error":true,"flags":"v","input":"","name":"generated-22-v-0","pattern":"[\\c_]"},"compile_error":"Invalid ECMAScript Unicode class-control escape"},{"case":{"error":true,"flags":"v","input":"aa","name":"generated-22-v-2","pattern":"[\\c_]"},"compile_error":"Invalid ECMAScript Unicode class-control escape"},{"case":{"error":true,"flags":"v","input":"abac","name":"generated-22-v-4","pattern":"[\\c_]"},"compile_error":"Invalid ECMAScript Unicode class-control escape"},{"case":{"error":true,"flags":"v","input":"aKſ😀\r\n","name":"generated-22-v-7","pattern":"[\\c_]"},"compile_error":"Invalid ECMAScript Unicode class-control escape"},{"case":{"error":true,"flags":"u","input":"","name":"generated-23-u-0","pattern":"\\c1"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"u","input":"aa","name":"generated-23-u-2","pattern":"\\c1"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"u","input":"abac","name":"generated-23-u-4","pattern":"\\c1"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"u","input":"aKſ😀\r\n","name":"generated-23-u-7","pattern":"\\c1"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"iu","input":"","name":"generated-23-iu-0","pattern":"\\c1"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"iu","input":"aa","name":"generated-23-iu-2","pattern":"\\c1"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"iu","input":"abac","name":"generated-23-iu-4","pattern":"\\c1"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"iu","input":"aKſ😀\r\n","name":"generated-23-iu-7","pattern":"\\c1"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"v","input":"","name":"generated-23-v-0","pattern":"\\c1"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"v","input":"aa","name":"generated-23-v-2","pattern":"\\c1"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"v","input":"abac","name":"generated-23-v-4","pattern":"\\c1"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"v","input":"aKſ😀\r\n","name":"generated-23-v-7","pattern":"\\c1"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"u","input":"","name":"generated-24-u-0","pattern":"\\xzz"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"u","input":"aa","name":"generated-24-u-2","pattern":"\\xzz"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"u","input":"abac","name":"generated-24-u-4","pattern":"\\xzz"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"u","input":"aKſ😀\r\n","name":"generated-24-u-7","pattern":"\\xzz"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"iu","input":"","name":"generated-24-iu-0","pattern":"\\xzz"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"iu","input":"aa","name":"generated-24-iu-2","pattern":"\\xzz"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"iu","input":"abac","name":"generated-24-iu-4","pattern":"\\xzz"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"iu","input":"aKſ😀\r\n","name":"generated-24-iu-7","pattern":"\\xzz"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"v","input":"","name":"generated-24-v-0","pattern":"\\xzz"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"v","input":"aa","name":"generated-24-v-2","pattern":"\\xzz"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"v","input":"abac","name":"generated-24-v-4","pattern":"\\xzz"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"v","input":"aKſ😀\r\n","name":"generated-24-v-7","pattern":"\\xzz"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"u","input":"","name":"generated-25-u-0","pattern":"\\uZZZZ"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid unicode escape sequence"},{"case":{"error":true,"flags":"u","input":"aa","name":"generated-25-u-2","pattern":"\\uZZZZ"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid unicode escape sequence"},{"case":{"error":true,"flags":"u","input":"abac","name":"generated-25-u-4","pattern":"\\uZZZZ"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid unicode escape sequence"},{"case":{"error":true,"flags":"u","input":"aKſ😀\r\n","name":"generated-25-u-7","pattern":"\\uZZZZ"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid unicode escape sequence"},{"case":{"error":true,"flags":"iu","input":"","name":"generated-25-iu-0","pattern":"\\uZZZZ"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid unicode escape sequence"},{"case":{"error":true,"flags":"iu","input":"aa","name":"generated-25-iu-2","pattern":"\\uZZZZ"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid unicode escape sequence"},{"case":{"error":true,"flags":"iu","input":"abac","name":"generated-25-iu-4","pattern":"\\uZZZZ"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid unicode escape sequence"},{"case":{"error":true,"flags":"iu","input":"aKſ😀\r\n","name":"generated-25-iu-7","pattern":"\\uZZZZ"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid unicode escape sequence"},{"case":{"error":true,"flags":"v","input":"","name":"generated-25-v-0","pattern":"\\uZZZZ"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid unicode escape sequence"},{"case":{"error":true,"flags":"v","input":"aa","name":"generated-25-v-2","pattern":"\\uZZZZ"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid unicode escape sequence"},{"case":{"error":true,"flags":"v","input":"abac","name":"generated-25-v-4","pattern":"\\uZZZZ"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid unicode escape sequence"},{"case":{"error":true,"flags":"v","input":"aKſ😀\r\n","name":"generated-25-v-7","pattern":"\\uZZZZ"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid unicode escape sequence"},{"case":{"error":true,"flags":"v","input":"","name":"generated-26-v-0","pattern":"[-a]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Expected nonempty class set expression"},{"case":{"error":true,"flags":"v","input":"aa","name":"generated-26-v-2","pattern":"[-a]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Expected nonempty class set expression"},{"case":{"error":true,"flags":"v","input":"abac","name":"generated-26-v-4","pattern":"[-a]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Expected nonempty class set expression"},{"case":{"error":true,"flags":"v","input":"aKſ😀\r\n","name":"generated-26-v-7","pattern":"[-a]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Expected nonempty class set expression"},{"case":{"error":true,"flags":"v","input":"","name":"generated-27-v-0","pattern":"[a-]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Unterminated character class"},{"case":{"error":true,"flags":"v","input":"aa","name":"generated-27-v-2","pattern":"[a-]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Unterminated character class"},{"case":{"error":true,"flags":"v","input":"abac","name":"generated-27-v-4","pattern":"[a-]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Unterminated character class"},{"case":{"error":true,"flags":"v","input":"aKſ😀\r\n","name":"generated-27-v-7","pattern":"[a-]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Unterminated character class"},{"case":{"error":true,"flags":"u","input":"","name":"generated-28-u-0","pattern":"[]]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"u","input":"aa","name":"generated-28-u-2","pattern":"[]]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"u","input":"abac","name":"generated-28-u-4","pattern":"[]]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"u","input":"aKſ😀\r\n","name":"generated-28-u-7","pattern":"[]]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"iu","input":"","name":"generated-28-iu-0","pattern":"[]]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"iu","input":"aa","name":"generated-28-iu-2","pattern":"[]]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"iu","input":"abac","name":"generated-28-iu-4","pattern":"[]]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"iu","input":"aKſ😀\r\n","name":"generated-28-iu-7","pattern":"[]]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"v","input":"","name":"generated-28-v-0","pattern":"[]]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"v","input":"aa","name":"generated-28-v-2","pattern":"[]]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"v","input":"abac","name":"generated-28-v-4","pattern":"[]]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"v","input":"aKſ😀\r\n","name":"generated-28-v-7","pattern":"[]]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"u","input":"","name":"generated-31-u-0","pattern":"[a-\\d]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Character class range with invalid atom"},{"case":{"error":true,"flags":"u","input":"aa","name":"generated-31-u-2","pattern":"[a-\\d]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Character class range with invalid atom"},{"case":{"error":true,"flags":"u","input":"abac","name":"generated-31-u-4","pattern":"[a-\\d]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Character class range with invalid atom"},{"case":{"error":true,"flags":"u","input":"aKſ😀\r\n","name":"generated-31-u-7","pattern":"[a-\\d]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Character class range with invalid atom"},{"case":{"error":true,"flags":"iu","input":"","name":"generated-31-iu-0","pattern":"[a-\\d]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Character class range with invalid atom"},{"case":{"error":true,"flags":"iu","input":"aa","name":"generated-31-iu-2","pattern":"[a-\\d]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Character class range with invalid atom"},{"case":{"error":true,"flags":"iu","input":"abac","name":"generated-31-iu-4","pattern":"[a-\\d]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Character class range with invalid atom"},{"case":{"error":true,"flags":"iu","input":"aKſ😀\r\n","name":"generated-31-iu-7","pattern":"[a-\\d]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Character class range with invalid atom"},{"case":{"error":true,"flags":"v","input":"","name":"generated-31-v-0","pattern":"[a-\\d]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Unterminated character class"},{"case":{"error":true,"flags":"v","input":"aa","name":"generated-31-v-2","pattern":"[a-\\d]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Unterminated character class"},{"case":{"error":true,"flags":"v","input":"abac","name":"generated-31-v-4","pattern":"[a-\\d]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Unterminated character class"},{"case":{"error":true,"flags":"v","input":"aKſ😀\r\n","name":"generated-31-v-7","pattern":"[a-\\d]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Unterminated character class"},{"case":{"error":true,"flags":"u","input":"","name":"generated-35-u-0","pattern":"[\\q{a|ab}]a"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid class atom"},{"case":{"error":true,"flags":"u","input":"aa","name":"generated-35-u-2","pattern":"[\\q{a|ab}]a"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid class atom"},{"case":{"error":true,"flags":"u","input":"abac","name":"generated-35-u-4","pattern":"[\\q{a|ab}]a"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid class atom"},{"case":{"error":true,"flags":"u","input":"aKſ😀\r\n","name":"generated-35-u-7","pattern":"[\\q{a|ab}]a"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid class atom"},{"case":{"error":true,"flags":"iu","input":"","name":"generated-35-iu-0","pattern":"[\\q{a|ab}]a"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid class atom"},{"case":{"error":true,"flags":"iu","input":"aa","name":"generated-35-iu-2","pattern":"[\\q{a|ab}]a"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid class atom"},{"case":{"error":true,"flags":"iu","input":"abac","name":"generated-35-iu-4","pattern":"[\\q{a|ab}]a"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid class atom"},{"case":{"error":true,"flags":"iu","input":"aKſ😀\r\n","name":"generated-35-iu-7","pattern":"[\\q{a|ab}]a"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid class atom"},{"case":{"error":true,"flags":"u","input":"","name":"generated-36-u-0","pattern":"[\\q{}]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid class atom"},{"case":{"error":true,"flags":"u","input":"aa","name":"generated-36-u-2","pattern":"[\\q{}]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid class atom"},{"case":{"error":true,"flags":"u","input":"abac","name":"generated-36-u-4","pattern":"[\\q{}]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid class atom"},{"case":{"error":true,"flags":"u","input":"aKſ😀\r\n","name":"generated-36-u-7","pattern":"[\\q{}]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid class atom"},{"case":{"error":true,"flags":"iu","input":"","name":"generated-36-iu-0","pattern":"[\\q{}]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid class atom"},{"case":{"error":true,"flags":"iu","input":"aa","name":"generated-36-iu-2","pattern":"[\\q{}]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid class atom"},{"case":{"error":true,"flags":"iu","input":"abac","name":"generated-36-iu-4","pattern":"[\\q{}]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid class atom"},{"case":{"error":true,"flags":"iu","input":"aKſ😀\r\n","name":"generated-36-iu-7","pattern":"[\\q{}]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Invalid class atom"},{"case":{"error":true,"flags":"u","input":"","name":"generated-37-u-0","pattern":"[[a-z]--[aeiou]]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"u","input":"aa","name":"generated-37-u-2","pattern":"[[a-z]--[aeiou]]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"u","input":"abac","name":"generated-37-u-4","pattern":"[[a-z]--[aeiou]]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"u","input":"aKſ😀\r\n","name":"generated-37-u-7","pattern":"[[a-z]--[aeiou]]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"iu","input":"","name":"generated-37-iu-0","pattern":"[[a-z]--[aeiou]]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"iu","input":"aa","name":"generated-37-iu-2","pattern":"[[a-z]--[aeiou]]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"iu","input":"abac","name":"generated-37-iu-4","pattern":"[[a-z]--[aeiou]]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"},{"case":{"error":true,"flags":"iu","input":"aKſ😀\r\n","name":"generated-37-iu-7","pattern":"[[a-z]--[aeiou]]"},"compile_error":"Invalid ECMAScript regex: Invalid regular expression: Could not parse the entire pattern"}]
</ui:copy>
<ui:file path="apps/simple-contract/snippet-regex-pattern-cases.json"><ui:paste select="#snippet-regex-v3-oracle-cases-compiled-json"/></ui:file>
<ui:copy id="snippet-regex-v3-format-oracle-cases-compiled-json">
[{"case":{"flags":"","format":"$0|$1|${1}","input":"foo","output":"foo|foo|foo","pattern":"(.*)","source":"${1/(.*)/$0|$1|${1}/}"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[0,9],[11,12],[14,8231],[8234,65535]],[],false,false],["repeat_end",0,1],["capture",1],["accept"]],"flags":"","format":[["capture",0,""],"|",["capture",1,""],"|",["capture",1,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[102,111,111],[102,111,111]],"from":0,"indices":[[0,3],[0,3]],"to":3}],"rust_output":"foo|foo|foo"},{"case":{"flags":"","format":"$$|$&|$`|$'|$<name>","input":"foo","output":"$$|$&|$`|$'|$<name>","pattern":"(.*)","source":"${1/(.*)/$$|$&|$`|$'|$<name>/}"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[0,9],[11,12],[14,8231],[8234,65535]],[],false,false],["repeat_end",0,1],["capture",1],["accept"]],"flags":"","format":["$$|$&|$`|$'|$<name>"],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[102,111,111],[102,111,111]],"from":0,"indices":[[0,3],[0,3]],"to":3}],"rust_output":"$$|$&|$`|$'|$<name>"},{"case":{"flags":"","format":"${1:+yes}|${1:-no}|${1:?yes:no}|${1:default}","input":"ab","output":"yes|a|yes|a","pattern":"(a)?b","source":"${1/(a)?b/${1:+yes}|${1:-no}|${1:?yes:no}|${1:default}/}"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,1,false,1,5,[1]],["mark",1],["set",[[97,97]],[],false,false],["capture",1],["repeat_end",0,0],["set",[[98,98]],[],false,false],["accept"]],"flags":"","format":[["if",1,["yes"],[]],"|",["if",1,[["capture",1,""]],["no"]],"|",["if",1,["yes"],["no"]],"|",["if",1,[["capture",1,""]],["default"]]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,98],[97]],"from":0,"indices":[[0,2],[0,1]],"to":2}],"rust_output":"yes|a|yes|a"},{"case":{"flags":"","format":"${1:+yes}|${1:-no}|${1:?yes:no}|${1:default}","input":"b","output":"|no|no|default","pattern":"(a)?b","source":"${1/(a)?b/${1:+yes}|${1:-no}|${1:?yes:no}|${1:default}/}"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,1,false,1,5,[1]],["mark",1],["set",[[97,97]],[],false,false],["capture",1],["repeat_end",0,0],["set",[[98,98]],[],false,false],["accept"]],"flags":"","format":[["if",1,["yes"],[]],"|",["if",1,[["capture",1,""]],["no"]],"|",["if",1,["yes"],["no"]],"|",["if",1,[["capture",1,""]],["default"]]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[98],null],"from":0,"indices":[[0,1],null],"to":1}],"rust_output":"|no|no|default"},{"case":{"flags":"","format":"${1:+yes}|${1:-no}|${1:?yes:no}|${1:default}","input":"x","output":"|no|no|default","pattern":"(a)?b","source":"${1/(a)?b/${1:+yes}|${1:-no}|${1:?yes:no}|${1:default}/}"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,1,false,1,5,[1]],["mark",1],["set",[[97,97]],[],false,false],["capture",1],["repeat_end",0,0],["set",[[98,98]],[],false,false],["accept"]],"flags":"","format":[["if",1,["yes"],[]],"|",["if",1,[["capture",1,""]],["no"]],"|",["if",1,["yes"],["no"]],"|",["if",1,[["capture",1,""]],["default"]]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"|no|no|default"},{"case":{"flags":"","format":"${1:+}|${1:-}|${1:?:}","input":"ab","output":"${1:+}|${1:-}|${1:?:}","pattern":"(a)?b","source":"${1/(a)?b/${1:+}|${1:-}|${1:?:}/}"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,1,false,1,5,[1]],["mark",1],["set",[[97,97]],[],false,false],["capture",1],["repeat_end",0,0],["set",[[98,98]],[],false,false],["accept"]],"flags":"","format":["${1:+}|${1:-}|${1:?:}"],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97,98],[97]],"from":0,"indices":[[0,2],[0,1]],"to":2}],"rust_output":"${1:+}|${1:-}|${1:?:}"},{"case":{"flags":"","format":"${1:+}|${1:-}|${1:?:}","input":"b","output":"${1:+}|${1:-}|${1:?:}","pattern":"(a)?b","source":"${1/(a)?b/${1:+}|${1:-}|${1:?:}/}"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,1,false,1,5,[1]],["mark",1],["set",[[97,97]],[],false,false],["capture",1],["repeat_end",0,0],["set",[[98,98]],[],false,false],["accept"]],"flags":"","format":["${1:+}|${1:-}|${1:?:}"],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[98],null],"from":0,"indices":[[0,1],null],"to":1}],"rust_output":"${1:+}|${1:-}|${1:?:}"},{"case":{"flags":"","format":"${1:+}|${1:-}|${1:?:}","input":"x","output":"x","pattern":"(a)?b","source":"${1/(a)?b/${1:+}|${1:-}|${1:?:}/}"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,0,1,false,1,5,[1]],["mark",1],["set",[[97,97]],[],false,false],["capture",1],["repeat_end",0,0],["set",[[98,98]],[],false,false],["accept"]],"flags":"","format":["${1:+}|${1:-}|${1:?:}"],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"x"},{"case":{"flags":"","format":"${1:/unknown}","input":"foo","output":"foo","pattern":"(.*)","source":"${1/(.*)/${1:/unknown}/}"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[0,9],[11,12],[14,8231],[8234,65535]],[],false,false],["repeat_end",0,1],["capture",1],["accept"]],"flags":"","format":[["capture",1,""]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[102,111,111],[102,111,111]],"from":0,"indices":[[0,3],[0,3]],"to":3}],"rust_output":"foo"},{"case":{"flags":"","format":"\\$1|\\}|\\q|\\\\|\\/","input":"foo","output":"\\foo|\\}|\\q|\\|/","pattern":"(.*)","source":"${1/(.*)/\\$1|\\}|\\q|\\\\|\\//}"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[0,9],[11,12],[14,8231],[8234,65535]],[],false,false],["repeat_end",0,1],["capture",1],["accept"]],"flags":"","format":["\\",["capture",1,""],"|\\}|\\q|\\|/"],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[102,111,111],[102,111,111]],"from":0,"indices":[[0,3],[0,3]],"to":3}],"rust_output":"\\foo|\\}|\\q|\\|/"},{"case":{"flags":"","format":"${1:+literal$2}|${1:?a\\:b:c}","input":"foo","output":"literal$2|${1:?a\\:b:c}","pattern":"(.*)","source":"${1/(.*)/${1:+literal$2}|${1:?a\\:b:c}/}"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[0,9],[11,12],[14,8231],[8234,65535]],[],false,false],["repeat_end",0,1],["capture",1],["accept"]],"flags":"","format":[["if",1,["literal$2"],[]],"|${1:?a\\:b:c}"],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[102,111,111],[102,111,111]],"from":0,"indices":[[0,3],[0,3]],"to":3}],"rust_output":"literal$2|${1:?a\\:b:c}"},{"case":{"flags":"","format":"${1:/upcase}|${1:/downcase}|${1:/capitalize}","input":"ßa","output":"SSA|ßa|SSa","pattern":"(.*)","source":"${1/(.*)/${1:/upcase}|${1:/downcase}|${1:/capitalize}/}"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[0,9],[11,12],[14,8231],[8234,65535]],[],false,false],["repeat_end",0,1],["capture",1],["accept"]],"flags":"","format":[["capture",1,"upcase"],"|",["capture",1,"downcase"],"|",["capture",1,"capitalize"]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[223,97],[223,97]],"from":0,"indices":[[0,2],[0,2]],"to":2}],"rust_output":"SSA|ßa|SSa"},{"case":{"flags":"","format":"${1:/upcase}|${1:/downcase}|${1:/capitalize}","input":"ΣΟΣ","output":"ΣΟΣ|σος|ΣΟΣ","pattern":"(.*)","source":"${1/(.*)/${1:/upcase}|${1:/downcase}|${1:/capitalize}/}"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[0,9],[11,12],[14,8231],[8234,65535]],[],false,false],["repeat_end",0,1],["capture",1],["accept"]],"flags":"","format":[["capture",1,"upcase"],"|",["capture",1,"downcase"],"|",["capture",1,"capitalize"]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[931,927,931],[931,927,931]],"from":0,"indices":[[0,3],[0,3]],"to":3}],"rust_output":"ΣΟΣ|σος|ΣΟΣ"},{"case":{"flags":"","format":"${1:/upcase}|${1:/downcase}|${1:/capitalize}","input":"𐐨name","output":"𐐀NAME|𐐨name|𐐨name","pattern":"(.*)","source":"${1/(.*)/${1:/upcase}|${1:/downcase}|${1:/capitalize}/}"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[0,9],[11,12],[14,8231],[8234,65535]],[],false,false],["repeat_end",0,1],["capture",1],["accept"]],"flags":"","format":[["capture",1,"upcase"],"|",["capture",1,"downcase"],"|",["capture",1,"capitalize"]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[55297,56360,110,97,109,101],[55297,56360,110,97,109,101]],"from":0,"indices":[[0,6],[0,6]],"to":6}],"rust_output":"𐐀NAME|𐐨name|𐐨name"},{"case":{"flags":"","format":"${1:/camelcase}|${1:/pascalcase}|${1:/kebabcase}|${1:/snakecase}","input":"HTTPServer","output":"hTTPServer|HTTPServer|http-server|httpserver","pattern":"(.*)","source":"${1/(.*)/${1:/camelcase}|${1:/pascalcase}|${1:/kebabcase}|${1:/snakecase}/}"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[0,9],[11,12],[14,8231],[8234,65535]],[],false,false],["repeat_end",0,1],["capture",1],["accept"]],"flags":"","format":[["capture",1,"camelcase"],"|",["capture",1,"pascalcase"],"|",["capture",1,"kebabcase"],"|",["capture",1,"snakecase"]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[72,84,84,80,83,101,114,118,101,114],[72,84,84,80,83,101,114,118,101,114]],"from":0,"indices":[[0,10],[0,10]],"to":10}],"rust_output":"hTTPServer|HTTPServer|http-server|httpserver"},{"case":{"flags":"","format":"${1:/camelcase}|${1:/pascalcase}|${1:/kebabcase}|${1:/snakecase}","input":"HTTP2","output":"hTTP2|HTTP2|2|http2","pattern":"(.*)","source":"${1/(.*)/${1:/camelcase}|${1:/pascalcase}|${1:/kebabcase}|${1:/snakecase}/}"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[0,9],[11,12],[14,8231],[8234,65535]],[],false,false],["repeat_end",0,1],["capture",1],["accept"]],"flags":"","format":[["capture",1,"camelcase"],"|",["capture",1,"pascalcase"],"|",["capture",1,"kebabcase"],"|",["capture",1,"snakecase"]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[72,84,84,80,50],[72,84,84,80,50]],"from":0,"indices":[[0,5],[0,5]],"to":5}],"rust_output":"hTTP2|HTTP2|2|http2"},{"case":{"flags":"","format":"${1:/camelcase}|${1:/pascalcase}|${1:/kebabcase}|${1:/snakecase}","input":"__hello_world__","output":"helloWorld|HelloWorld|hello-world|__hello_world__","pattern":"(.*)","source":"${1/(.*)/${1:/camelcase}|${1:/pascalcase}|${1:/kebabcase}|${1:/snakecase}/}"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[0,9],[11,12],[14,8231],[8234,65535]],[],false,false],["repeat_end",0,1],["capture",1],["accept"]],"flags":"","format":[["capture",1,"camelcase"],"|",["capture",1,"pascalcase"],"|",["capture",1,"kebabcase"],"|",["capture",1,"snakecase"]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[95,95,104,101,108,108,111,95,119,111,114,108,100,95,95],[95,95,104,101,108,108,111,95,119,111,114,108,100,95,95]],"from":0,"indices":[[0,15],[0,15]],"to":15}],"rust_output":"helloWorld|HelloWorld|hello-world|__hello_world__"},{"case":{"flags":"","format":"${1:/camelcase}|${1:/pascalcase}|${1:/kebabcase}|${1:/snakecase}","input":"éCLAIR δValue","output":"éCLAIRΔValue|ÉCLAIRΔValue|é-clair-δ-value|é_clair_δ_value","pattern":"(.*)","source":"${1/(.*)/${1:/camelcase}|${1:/pascalcase}|${1:/kebabcase}|${1:/snakecase}/}"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[0,9],[11,12],[14,8231],[8234,65535]],[],false,false],["repeat_end",0,1],["capture",1],["accept"]],"flags":"","format":[["capture",1,"camelcase"],"|",["capture",1,"pascalcase"],"|",["capture",1,"kebabcase"],"|",["capture",1,"snakecase"]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[233,67,76,65,73,82,32,948,86,97,108,117,101],[233,67,76,65,73,82,32,948,86,97,108,117,101]],"from":0,"indices":[[0,13],[0,13]],"to":13}],"rust_output":"éCLAIRΔValue|ÉCLAIRΔValue|é-clair-δ-value|é_clair_δ_value"},{"case":{"flags":"","format":"${1:/camelcase}|${1:/pascalcase}|${1:/kebabcase}|${1:/snakecase}","input":"𐐨name_test","output":"𐐨nameTest|𐐨nameTest|𐐨name-test|𐐨name_test","pattern":"(.*)","source":"${1/(.*)/${1:/camelcase}|${1:/pascalcase}|${1:/kebabcase}|${1:/snakecase}/}"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["repeat",0,0,null,false,2,4,[]],["set",[[0,9],[11,12],[14,8231],[8234,65535]],[],false,false],["repeat_end",0,1],["capture",1],["accept"]],"flags":"","format":[["capture",1,"camelcase"],"|",["capture",1,"pascalcase"],"|",["capture",1,"kebabcase"],"|",["capture",1,"snakecase"]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[55297,56360,110,97,109,101,95,116,101,115,116],[55297,56360,110,97,109,101,95,116,101,115,116]],"from":0,"indices":[[0,11],[0,11]],"to":11}],"rust_output":"𐐨nameTest|𐐨nameTest|𐐨name-test|𐐨name_test"},{"case":{"flags":"","format":"$0|$1|$2|${1:/upcase}","input":"a","output":"a|a||A","pattern":"(?<word>a)","source":"${1/(?<word>a)/$0|$1|$2|${1:/upcase}/}"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["set",[[97,97]],[],false,false],["capture",1],["accept"]],"flags":"","format":[["capture",0,""],"|",["capture",1,""],"|",["capture",2,""],"|",["capture",1,"upcase"]],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"a|a||A"},{"case":{"flags":"","format":"prefix${1:-else}suffix","input":"absent","output":"prefixelsesuffix","pattern":"(x)","source":"${1/(x)/prefix${1:-else}suffix/}"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["set",[[120,120]],[],false,false],["capture",1],["accept"]],"flags":"","format":["prefix",["if",1,[["capture",1,""]],["else"]],"suffix"],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"prefixelsesuffix"},{"case":{"flags":"","format":"${1:?true:${2:-fallback}}","input":"a","output":"true}","pattern":"(a)","source":"${1/(a)/${1:?true:${2:-fallback}}/}"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["set",[[97,97]],[],false,false],["capture",1],["accept"]],"flags":"","format":[["if",1,["true"],["${2:-fallback"]],"}"],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[{"capture_units":[[97],[97]],"from":0,"indices":[[0,1],[0,1]],"to":1}],"rust_output":"true}"},{"case":{"flags":"","format":"${1:?true:${2:-fallback}}","input":"x","output":"${2:-fallback}","pattern":"(a)","source":"${1/(a)/${1:?true:${2:-fallback}}/}"},"program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["set",[[97,97]],[],false,false],["capture",1],["accept"]],"flags":"","format":[["if",1,["true"],["${2:-fallback"]],"}"],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"rust_matches":[],"rust_output":"${2:-fallback}"}]
</ui:copy>
<ui:file path="apps/simple-contract/snippet-regex-format-cases.json"><ui:paste select="#snippet-regex-v3-format-oracle-cases-compiled-json"/></ui:file>
<ui:copy id="snippet-regex-v3-budget-cases-json">
{"cases":[{"name":"foreign case policy","phase":"read","program":{"case_map":"foreign","code":[["set",[[0,65535]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3}},{"name":"unknown version","phase":"read","program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,65535]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":4}},{"name":"boolean capture count","phase":"read","program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,65535]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":true,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3}},{"name":"fractional capture count","phase":"read","program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,65535]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":1.5,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3}},{"name":"unsafe repeat count","phase":"read","program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat",0,9007199254740992,null,false,1,1,[]],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3}},{"name":"unknown operation","phase":"read","program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["execute","provider command"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3}},{"name":"outside jump","phase":"read","program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["jump",1]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3}},{"name":"unmerged ranges","phase":"read","program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[1,2],[3,4]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3}},{"name":"unsorted strings","phase":"read","program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[],[[98,98],[97,97]],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3}},{"name":"foreign assertion","phase":"read","program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[]],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3}},{"name":"wrong repeat edge","phase":"read","program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["repeat_end",0,1],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3}},{"name":"unknown format","phase":"read","program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,65535]],[],false,false],["accept"]],"flags":"","format":[["execute",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3}},{"name":"extra top-level effect","phase":"read","program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,65535]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3,"command":{"command":"arbitrary"}}},{"name":"mixed Unicode modes","phase":"read","program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,65535]],[],false,false],["accept"]],"flags":"uv","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3}},{"name":"nonboolean mode","phase":"read","program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[],[],0,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3}},{"name":"control cycle work","phase":"run","program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["jump",0]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"input":"a","work":0},{"name":"backtrack state budget","phase":"run","program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["split",0,0]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"input":"a","work":0},{"name":"assertion depth budget","phase":"run","program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["look",0,true,[]],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[{"direction":1,"code":[["look",0,true,[]],["accept"]]}],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"input":"a","work":0},{"name":"UTF8 byte input budget","phase":"run","program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,65535]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"input":"😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀","work":0},{"name":"output byte budget","phase":"run","program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,65535]],[],false,false],["accept"]],"flags":"g","format":["xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"input":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","work":0},{"name":"unpaired captured output","phase":"run","program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["mark",1],["set",[[0,65535]],[],false,false],["capture",1],["accept"]],"flags":"","format":[["capture",1,""],"x"],"groups":1,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"input":"😀","work":0},{"name":"shared action work budget","phase":"run","program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[[0,65535]],[],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"input":"a","work":99999},{"name":"skipped string candidates spend shared work","phase":"run","program":{"case_map":"hickory-ucd-17.0.0-v1","code":[["set",[],[[65,65],[66,65],[67,65],[68,65],[69,65],[70,65],[71,65],[72,65],[73,65],[74,65],[75,65],[76,65],[77,65],[78,65],[79,65],[80,65],[81,65],[82,65],[83,65],[84,65],[85,65],[86,65],[87,65],[88,65],[89,65],[90,65],[91,65],[92,65],[93,65],[94,65],[95,65],[96,65]],false,false],["accept"]],"flags":"","format":[["capture",0,""]],"groups":0,"subprograms":[],"unicode_map":"hickory-regex-ucd-17.0.0-v1","version":3},"input":"","work":99990}]}
</ui:copy>
<ui:file path="apps/simple-contract/snippet-regex-budget-cases.json"><ui:paste select="#snippet-regex-v3-budget-cases-json"/></ui:file>
<ui:copy id="snippet-regex-v3-browser-tests">
import {describe,it,expect} from 'vitest';
import patterns from '../../../simple-contract/snippet-regex-pattern-cases.json';
import formats from '../../../simple-contract/snippet-regex-format-cases.json';
import refusals from '../../../simple-contract/snippet-regex-budget-cases.json';
import data from '../../../simple-contract/snippet-regex-case17.json';
import {readRegexV3,execute,casePolicy,evaluateRegexV3,regexV3ProgramBytes,type RegexV3CaseData} from './SnippetTransformV3';
const policy=casePolicy(data as unknown as RegexV3CaseData);
describe('ECMAScript data-only transforms',()=>{
 it('matches independent exact UTF16 captures and spans',()=>{let count=0,syntaxRefusals=0;for(const raw of patterns){const row=raw as unknown as {case:{name:string;error?:boolean;input:string;matches?:Record<string,unknown>[]};program?:unknown;compile_error?:string};if(row.case.error){expect(row.compile_error,row.case.name).toBeTypeOf('string');syntaxRefusals++;continue;}const actual=execute(readRegexV3(row.program),row.case.input,policy).map(value=>({from:value.from,to:value.to,indices:value.indices,capture_units:value.captures.map(text=>text===null?null:Array.from({length:text.length},(_,i)=>text.charCodeAt(i)))}));const expected=row.case.matches!.map(value=>{const result={...value};delete result.name_units;return result;});expect(actual,row.case.name).toEqual(expected);count++;}expect(count).toBe(637);expect(syntaxRefusals).toBe(221);});
 it('preserves actual TextMate formatting including literal conditionals',()=>{for(const row of formats)expect(evaluateRegexV3(readRegexV3(row.program),row.case.input),row.case.source).toBe(row.case.output);expect(formats).toHaveLength(23);});
 it('refuses hostile metadata and exhausted budgets atomically',()=>{for(const row of refusals.cases)expect(()=>{const program=readRegexV3(row.program);if(row.phase==='run')evaluateRegexV3(program,row.input!,{work:row.work!});},row.name).toThrow();expect(refusals.cases).toHaveLength(23);});
it('admits canonical numeric metadata without walking excessive instruction arrays',()=>{const base=structuredClone(formats[0].program) as unknown as Record<string,unknown>;base.groups=0;base.flags='u';base.format=[['capture',0,'']];base.subprograms=[];const set=['set',Array.from({length:55000},(_,i)=>[i*16,i*16]),[],false,true];base.code=[set,['accept']];const admitted=readRegexV3(base);expect(regexV3ProgramBytes(admitted)).toBeLessThan(1048576);expect(regexV3ProgramBytes(admitted)).toBe(new TextEncoder().encode(JSON.stringify(admitted)).length);expect(readRegexV3(admitted)).toBe(admitted);expect(Object.isFrozen(admitted.code)).toBe(true);expect(()=>admitted.code.push(['accept'])).toThrow();base.code=[set,set,['accept']];expect(()=>readRegexV3(base)).toThrow('metadata byte budget');let visited=0;base.code=new Proxy([],{get(_target,key){if(key==='length')return 4097;visited++;throw Error('Unbounded array access');}});expect(()=>readRegexV3(base)).toThrow('instruction budget');expect(visited).toBe(0);});
});
</ui:copy>
<ui:file path="apps/web/src/simple/SnippetTransformV3.test.ts"><ui:paste select="#snippet-regex-v3-browser-tests"/></ui:file>
<ui:copy id="snippet-regex-v3-apple-tests">
import Foundation
import XCTest

@testable import HickorySimple

final class SnippetTransformV3Tests: XCTestCase {
  func load(_ name: String) throws -> Any {
    let names = [
      "case-17.json": "snippet-regex-case17.json",
      "oracle-cases-compiled.json": "snippet-regex-pattern-cases.json",
      "format-oracle-cases-compiled.json": "snippet-regex-format-cases.json",
      "budget-cases.json": "snippet-regex-budget-cases.json",
    ]
    let base = URL(fileURLWithPath: #filePath).deletingLastPathComponent()
      .deletingLastPathComponent().deletingLastPathComponent().appendingPathComponent(
        "simple-contract")
    return try JSONSerialization.jsonObject(
      with: Data(contentsOf: base.appendingPathComponent(names[name]!)))
  }
  func testIndependentPatternsFormatsAndBudgetRefusals() throws {
    let policy = try RegexV3CaseData(load("case-17.json") as! [String: Any])
    let rows = try load("oracle-cases-compiled.json") as! [[String: Any]]
    var passed = 0
    var failed = 0
    for row in rows {
      let test = row["case"] as! [String: Any]
      do {
        if test["error"] as? Bool == true {
          guard row["compile_error"] is String else {
            throw EngineError(message: "Invalid regex compiled")
          }
          passed += 1
          continue
        }
        let program = try readRegexV3(row["program"]!)
        let input = test["input"] as! String
        var machine = try RegexV3Machine(input, program, policy)
        let result = try machine.find(program).map { value -> [String: Any] in
          [
            "from": machine.offsets[value.from], "to": machine.offsets[value.to],
            "indices": value.captures.map { range -> Any in
              range.map { [machine.offsets[$0.lowerBound], machine.offsets[$0.upperBound]] } as Any?
                ?? NSNull()
            },
            "capture_units": value.captures.map { range -> Any in
              range.map { machine.units($0) } as Any? ?? NSNull()
            },
          ]
        }
        let expected = (test["matches"] as! [[String: Any]]).map { value -> [String: Any] in
          var value = value
          value.removeValue(forKey: "name_units")
          return value
        }
        let actual = try JSONSerialization.data(withJSONObject: result, options: .sortedKeys)
        let golden = try JSONSerialization.data(withJSONObject: expected, options: .sortedKeys)
        guard actual == golden else {
          throw EngineError(
            message: "Capture/match mismatch: " + String(data: actual, encoding: .utf8)!)
        }
        passed += 1
      } catch {
        failed += 1
        print("FAIL \(test["name"]!) \(error)")
      }
    }
    print("Swift v3 patterns \(passed) PASS \(failed) FAIL")
    XCTAssertEqual(failed, 0)
    let formats = try load("format-oracle-cases-compiled.json") as! [[String: Any]]
    var formatPassed = 0
    for row in formats {
      let test = row["case"] as! [String: Any]
      do {
        let program = try readRegexV3(row["program"]!)
        var work = 0
        let actual = try applyRegexV3(program, test["input"] as! String, policy, work: &work)
        guard actual == test["output"] as! String else {
          throw EngineError(message: "Expected \(test["output"]!), got \(actual)")
        }
        formatPassed += 1
      } catch {
        failed += 1
        print("FORMAT FAIL \(test["source"]!) \(error)")
      }
    }
    print("Swift v3 formats \(formatPassed) PASS \(failed) FAIL")
    XCTAssertEqual(failed, 0)
    let budgets = (try load("budget-cases.json") as! [String: Any])["cases"] as! [[String: Any]]
    var budgetPassed = 0
    for row in budgets {
      do {
        let program = try readRegexV3(row["program"]!)
        if row["phase"] as? String == "run" {
          var work = row["work"] as! Int
          _ = try applyRegexV3(program, row["input"] as! String, policy, work: &work)
        }
        print("BUDGET FAIL \(row["name"]!) accepted")
        failed += 1
      } catch { budgetPassed += 1 }
    }
    print("Swift v3 budget/refusal \(budgetPassed) PASS \(failed) FAIL")
    XCTAssertEqual(failed, 0)

    XCTAssertEqual(passed, 858)
    XCTAssertEqual(formatPassed, 23)
    XCTAssertEqual(budgetPassed, 23)
  }
  func testCanonicalNumericMetadataAndOuterInstructionBudget() throws {
    let rows = try load("budget-cases.json") as! [String: Any]
    var raw = ((rows["cases"] as! [[String: Any]])[0]["program"] as! [String: Any])
    raw["case_map"] = "hickory-ucd-17.0.0-v1"
    raw["version"] = 3.0
    raw["groups"] = 0.0
    raw["flags"] = "u"
    let ranges = (0..<55000).map { [Double($0 * 16), Double($0 * 16)] }
    let set: [Any] = ["set", ranges, [], false, true]
    raw["code"] = [set, ["accept"]]
    XCTAssertLessThan(try readRegexV3(raw).bytes, 1_048_576)
    raw["code"] = [set, set, ["accept"]]
    XCTAssertThrowsError(try readRegexV3(raw))
    raw["code"] = Array(repeating: ["accept"], count: 4097)
    XCTAssertThrowsError(try readRegexV3(raw))
  }
}
</ui:copy>
<ui:file path="apps/apple-simple/Tests/SnippetTransformV3Tests.swift"><ui:paste select="#snippet-regex-v3-apple-tests"/></ui:file>
<ui:copy id="snippet-regex-v3-android-tests">
package com.hickorydocs.simple
import org.json.*
import java.io.File
import org.junit.Test
import org.junit.Assert.assertEquals
private fun regexJSONEqual(a:Any?,b:Any?):Boolean = when {
 a is JSONObject && b is JSONObject -> a.keys().asSequence().toSet()==b.keys().asSequence().toSet() && a.keys().asSequence().all{regexJSONEqual(a.get(it),b.get(it))}
 a is JSONArray && b is JSONArray -> a.length()==b.length() && (0 until a.length()).all{regexJSONEqual(a.get(it),b.get(it))}
 a is Number && b is Number -> a.toDouble()==b.toDouble()
 else -> a==b
}
class SnippetTransformV3Test {
@Test fun independentPatternsFormatsAndBudgetRefusals(){val dir=generateSequence(File(System.getProperty("user.dir")).absoluteFile){it.parentFile}.map{File(it,"apps/simple-contract")}.firstOrNull{it.isDirectory}?:error("Shared contract missing");val policy=RegexV3CaseData(JSONObject(File(dir,"snippet-regex-case17.json").readText()));val rows=JSONArray(File(dir,"snippet-regex-pattern-cases.json").readText());var passed=0;var failed=0;for(i in 0 until rows.length()){val row=rows.getJSONObject(i);val test=row.getJSONObject("case");try{if(test.optBoolean("error")){check(row.has("compile_error"));passed++;continue};val program=readRegexV3(row.get("program"));val machine=RegexV3Machine(test.getString("input"),program,policy);val result=JSONArray();for(value in machine.find()){val indices=JSONArray();val units=JSONArray();for(range in value.captures){indices.put(range?.let{JSONArray(listOf(machine.offsets[it.first],machine.offsets[it.second]))}?:JSONObject.NULL);units.put(range?.let{JSONArray(machine.units(it))}?:JSONObject.NULL)};result.put(JSONObject().put("from",machine.offsets[value.from]).put("to",machine.offsets[value.to]).put("indices",indices).put("capture_units",units))};val expected=test.getJSONArray("matches");for(n in 0 until expected.length())expected.getJSONObject(n).remove("name_units");check(regexJSONEqual(result,expected)){"Mismatch $result"};passed++}catch(e:Exception){failed++;println("FAIL ${test.opt("name")} $e")}};println("Kotlin v3 patterns $passed PASS $failed FAIL");check(failed==0)
val formats=JSONArray(File(dir,"snippet-regex-format-cases.json").readText());var formatPassed=0;for(i in 0 until formats.length()){val row=formats.getJSONObject(i);val test=row.getJSONObject("case");try{val actual=applyRegexV3(readRegexV3(row.get("program")),test.getString("input"),policy);check(actual==test.getString("output")){"Expected ${test.getString("output")}, got $actual"};formatPassed++}catch(e:Exception){failed++;println("FORMAT FAIL ${test.opt("source")} $e")}};println("Kotlin v3 formats $formatPassed PASS $failed FAIL");check(failed==0);val budgets=JSONObject(File(dir,"snippet-regex-budget-cases.json").readText()).getJSONArray("cases");var budgetPassed=0;for(i in 0 until budgets.length()){val row=budgets.getJSONObject(i);try{val program=readRegexV3(row.get("program"));if(row.getString("phase")=="run")applyRegexV3(program,row.getString("input"),policy,intArrayOf(row.getInt("work")));failed++;println("BUDGET FAIL ${row.getString("name")} accepted")}catch(e:Exception){budgetPassed++}};println("Kotlin v3 budget/refusal $budgetPassed PASS $failed FAIL");check(failed==0)}


@Test fun admissionChecksLengthsBeforeReadingOrEncoding(){val dir=generateSequence(File(System.getProperty("user.dir")).absoluteFile){it.parentFile}.map{File(it,"apps/simple-contract")}.first{it.isDirectory};val seed=JSONObject(File(dir,"snippet-regex-budget-cases.json").readText()).getJSONArray("cases").getJSONObject(0).getJSONObject("program");seed.put("case_map","hickory-ucd-17.0.0-v1");var visited=0;val huge=object:JSONArray(){override fun length()=100000;override fun opt(index:Int):Any?{visited++;error("Unbounded array access")}};val poisoned=JSONObject(seed.toString()).put("code",huge);val refusal=runCatching{readRegexV3(poisoned)}.exceptionOrNull();check(refusal?.message?.contains("array budget")==true&&visited==0){"Early array refusal failed: $refusal"};var encoded=0;val watched=object:JSONObject(seed.toString()){override fun toString():String{encoded++;error("Whole-object serialization")}};readRegexV3(watched);check(encoded==0);val ranges=JSONArray();for(i in 0 until 55000)ranges.put(JSONArray().put(i*16.0).put(i*16.0));val numeric=JSONObject(seed.toString()).put("version",3.0).put("groups",0.0).put("flags","u").put("code",JSONArray().put(JSONArray().put("set").put(ranges).put(JSONArray()).put(false).put(true)).put(JSONArray().put("accept")));readRegexV3(numeric);numeric.put("code",JSONArray().put(numeric.getJSONArray("code").getJSONArray(0)).put(numeric.getJSONArray("code").getJSONArray(0)).put(JSONArray().put("accept")));check(runCatching{readRegexV3(numeric)}.isFailure);println("Kotlin bounded admission and canonical numeric metadata PASS")}
}
</ui:copy>
<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/SnippetTransformV3Test.kt"><ui:paste select="#snippet-regex-v3-android-tests"/></ui:file>
</ui:doc>
