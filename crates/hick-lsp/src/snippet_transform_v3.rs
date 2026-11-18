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

mod ast_lowering {
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

}

mod sets {
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

}

mod bytecode {
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

}

mod format {
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

}

mod reader {
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

}

mod matcher {
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

}

mod formatter {
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

}

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
