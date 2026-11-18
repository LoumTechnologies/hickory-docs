<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Share bounded snippet transforms without editor-specific regex behavior

The engine compiles source syntax into a data-only regular-expression and replacement program. Each host evaluates that same program in its pure snippet model before submitting one complete source action. No provider request runs per keystroke. Programs, captures, match alternatives, evaluator work and output are bounded; unsupported syntax or exhausted work refuses the complete linked action.

The common expression language includes scalar literals, dot, ASCII character shorthands, explicit ranges, captures, alternatives, anchors, word boundaries and greedy or lazy repetitions. Backreferences, lookaround, named groups and Unicode property escapes are explicit unsupported syntax. This is a measured common transform language, not a full ECMAScript-regex parity claim. Replacement capture references, casing and conditional branches are compiled separately from matching. Input indexing uses Unicode scalars internally; field/session ranges remain exact UTF-16 positions. Program version 2 also carries the frozen Unicode case-policy identifier. Insensitive classes are expanded once in the engine; every host checks only those literal ranges and the declared word-boundary policy. Legacy insensitive matching preserves ECMAScript’s non-ASCII-to-ASCII exclusion; Unicode-insensitive matching includes the simple-fold closure. Replacement casing and camel/pascal token boundaries use the same versioned data, independent of the OS locale and Unicode library. Lone UTF-16 surrogates refuse before producing an action.

Explicitly negotiated program 3 uses the bounded ECMAScript machine in [the regex compiler and host reading](snippet-regex-v3.md). Existing program 2 keeps its frozen Unicode 16 behavior. The wrappers below preserve the apply interface so input adapters share their existing whole-action refusal and history paths. Admission checks the program version before any edit; advertising a reader alone does not prove the provider or template path.

<ui:copy id="snippet-transform-engine">
//! Bounded data-only snippet transforms shared with the pure host evaluators.
#[path = "snippet_transform_v3.rs"]
mod v3;
use crate::snippet_case;
use serde_json::{Value, json};
const MAX_NODES: usize = 512;
const MAX_STATES: usize = 1024;
const MAX_WORK: usize = 100_000;
const MAX_TEXT: usize = 65_536;
const MAX_OUTPUT: usize = 1_048_576;

struct RegexParser {
    chars: Vec<char>,
    at: usize,
    nodes: usize,
    groups: usize,
}
impl RegexParser {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.at).copied()
    }
    fn take(&mut self) -> Option<char> {
        let value = self.peek()?;
        self.at += 1;
        Some(value)
    }
    fn node(&mut self, value: Value) -> Result<Value, String> {
        self.nodes += 1;
        if self.nodes > MAX_NODES {
            return Err("Snippet transform program limit exceeded".into());
        }
        Ok(value)
    }
    fn sequence(&mut self, depth: usize) -> Result<Value, String> {
        if depth > 32 {
            return Err("Snippet transform nesting limit exceeded".into());
        }
        let mut nodes = vec![];
        while !matches!(self.peek(), None | Some(')' | '|')) {
            let mut atom = self.atom(depth + 1)?;
            let repetition = match self.peek() {
                Some('*') => {
                    self.take();
                    Some((0, None))
                }
                Some('+') => {
                    self.take();
                    Some((1, None))
                }
                Some('?') => {
                    self.take();
                    Some((0, Some(1)))
                }
                Some('{') => {
                    self.take();
                    let min = self.number()?;
                    let max = match self.take() {
                        Some('}') => Some(min),
                        Some(',') => {
                            let max = if self.peek() == Some('}') {
                                None
                            } else {
                                Some(self.number()?)
                            };
                            if self.take() != Some('}') {
                                return Err("Malformed transform repetition".into());
                            }
                            max
                        }
                        _ => return Err("Malformed transform repetition".into()),
                    };
                    if min > 256 || max.is_some_and(|max| max < min || max > 256) {
                        return Err("Transform repetition limit exceeded".into());
                    }
                    Some((min, max))
                }
                _ => None,
            };
            if let Some((min, max)) = repetition {
                let lazy = self.peek() == Some('?');
                if lazy {
                    self.take();
                }
                atom = self.node(json!(["repeat", atom, min, max, lazy]))?;
                if matches!(self.peek(), Some('*' | '+' | '?' | '{')) {
                    return Err("Repeated transform quantifier".into());
                }
            }
            nodes.push(atom);
        }
        self.node(json!(["seq", nodes]))
    }
    fn expression(&mut self, depth: usize) -> Result<Value, String> {
        let mut branches = vec![self.sequence(depth)?];
        while self.peek() == Some('|') {
            self.take();
            branches.push(self.sequence(depth)?)
        }
        if branches.len() == 1 {
            Ok(branches.remove(0))
        } else {
            self.node(json!(["alt", branches]))
        }
    }
    fn number(&mut self) -> Result<usize, String> {
        let start = self.at;
        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.take();
        }
        if self.at == start {
            return Err("Missing transform integer".into());
        }
        self.chars[start..self.at]
            .iter()
            .collect::<String>()
            .parse()
            .map_err(|_| "Transform integer overflow".into())
    }
    fn escaped(&mut self, class: bool) -> Result<Value, String> {
        let ch = self.take().ok_or("Unclosed transform escape")?;
        let ranges = match ch {
            'd' | 'D' => Some(vec![(48, 57)]),
            'w' | 'W' => Some(vec![(48, 57), (65, 90), (95, 95), (97, 122)]),
            's' | 'S' => Some(vec![
                (9, 13),
                (32, 32),
                (160, 160),
                (0x1680, 0x1680),
                (0x2000, 0x200a),
                (0x2028, 0x2029),
                (0x202f, 0x202f),
                (0x205f, 0x205f),
                (0x3000, 0x3000),
                (0xfeff, 0xfeff),
            ]),
            _ => None,
        };
        if let Some(ranges) = ranges {
            return self.node(json!(["class", ranges, ch.is_ascii_uppercase()]));
        }
        if !class && matches!(ch, 'b' | 'B') {
            return self.node(json!([
                "assert",
                if ch == 'b' { "word" } else { "nonword" }
            ]));
        }
        let literal = match ch {
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            'f' => '\u{c}',
            'v' => '\u{b}',
            'b' if class => '\u{8}',
            'x' | 'u' => {
                let count = if ch == 'x' { 2 } else { 4 };
                let mut value = 0u32;
                for _ in 0..count {
                    value = value * 16
                        + self
                            .take()
                            .and_then(|ch| ch.to_digit(16))
                            .ok_or("Malformed transform Unicode escape")?;
                }
                char::from_u32(value).ok_or("Transform escape is not a Unicode scalar")?
            }
            ch if ch.is_ascii_alphanumeric() => {
                return Err("Unsupported transform escape or backreference".into());
            }
            ch => ch,
        };
        self.node(json!(["class", [[literal as u32, literal as u32]], false]))
    }
    fn atom(&mut self, depth: usize) -> Result<Value, String> {
        match self.take().ok_or("Missing transform atom")? {
            '(' => {
                let capture = if self.peek() == Some('?') {
                    self.take();
                    if self.take() != Some(':') {
                        return Err("Transform lookaround and named groups are unsupported".into());
                    }
                    None
                } else {
                    self.groups += 1;
                    if self.groups > 32 {
                        return Err("Transform capture limit exceeded".into());
                    }
                    Some(self.groups)
                };
                let node = self.expression(depth)?;
                if self.take() != Some(')') {
                    return Err("Unclosed transform group".into());
                }
                match capture {
                    Some(index) => self.node(json!(["capture", index, node])),
                    None => Ok(node),
                }
            }
            '[' => {
                let negate = self.peek() == Some('^');
                if negate {
                    self.take();
                }
                let mut ranges = vec![];
                while self.peek() != Some(']') {
                    if self.peek().is_none() {
                        return Err("Unclosed transform class".into());
                    }
                    let first = if self.take_if('\\') {
                        self.escaped(true)?
                    } else {
                        let ch = self.take().unwrap();
                        self.node(json!(["class", [[ch as u32, ch as u32]], false]))?
                    };
                    if first[2].as_bool() != Some(false) {
                        return Err(
                            "Negated shorthand inside a transform class is unsupported".into()
                        );
                    }
                    let entries = first[1].as_array().ok_or("Invalid transform class")?;
                    if entries.len() == 1
                        && self.peek() == Some('-')
                        && self.chars.get(self.at + 1) != Some(&']')
                    {
                        self.take();
                        let last = if self.take_if('\\') {
                            self.escaped(true)?
                        } else {
                            let ch = self.take().ok_or("Unclosed transform range")?;
                            json!(["class", [[ch as u32, ch as u32]], false])
                        };
                        if last[2].as_bool() != Some(false)
                            || last[1].as_array().map(Vec::len) != Some(1)
                        {
                            return Err("Invalid transform range".into());
                        }
                        let low = entries[0][0].as_u64().unwrap();
                        let high = last[1][0][0].as_u64().unwrap();
                        if low > high {
                            return Err("Backwards transform range".into());
                        }
                        ranges.push(json!([low, high]));
                    } else {
                        ranges.extend(entries.iter().cloned());
                    }
                }
                self.take();
                self.node(json!(["class", ranges, negate]))
            }
            '\\' => self.escaped(false),
            '.' => self.node(json!(["dot"])),
            '^' => self.node(json!(["assert", "start"])),
            '$' => self.node(json!(["assert", "end"])),
            ch @ ('*' | '+' | '?' | '{' | '}') => Err(format!("Unsupported transform atom {ch}")),
            ch => self.node(json!(["class", [[ch as u32, ch as u32]], false])),
        }
    }
    fn take_if(&mut self, ch: char) -> bool {
        if self.peek() == Some(ch) {
            self.take();
            true
        } else {
            false
        }
    }
}

struct FormatParser {
    chars: Vec<char>,
    at: usize,
    nodes: usize,
}
impl FormatParser {
    fn sequence(&mut self, stop: Option<char>, depth: usize) -> Result<Vec<Value>, String> {
        if depth > 32 {
            return Err("Transform format nesting limit exceeded".into());
        }
        let mut result = vec![];
        let mut text = String::new();
        while let Some(&ch) = self.chars.get(self.at) {
            if Some(ch) == stop {
                break;
            }
            self.at += 1;
            match ch {
                '\\' => {
                    let next = *self
                        .chars
                        .get(self.at)
                        .ok_or("Unclosed transform format escape")?;
                    self.at += 1;
                    if !matches!(next, '$' | '/' | '\\' | '}' | ':') {
                        return Err("Unsupported transform format escape".into());
                    }
                    text.push(next)
                }
                '$' => {
                    if !text.is_empty() {
                        result.push(json!(std::mem::take(&mut text)))
                    }
                    let braced = self.chars.get(self.at) == Some(&'{');
                    if braced {
                        self.at += 1;
                    }
                    let start = self.at;
                    while self.chars.get(self.at).is_some_and(|c| c.is_ascii_digit()) {
                        self.at += 1;
                    }
                    if self.at == start {
                        return Err("Missing transform capture reference".into());
                    }
                    let index: usize = self.chars[start..self.at]
                        .iter()
                        .collect::<String>()
                        .parse()
                        .map_err(|_| "Transform capture overflow")?;
                    if index > 32 {
                        return Err("Transform capture limit exceeded".into());
                    }
                    let value = if !braced {
                        json!(["capture", index, ""])
                    } else {
                        match self.chars.get(self.at) {
                            Some('}') => {
                                json!(["capture", index, ""])
                            }
                            Some(':') => {
                                self.at += 1;
                                match self.chars.get(self.at) {
                                    Some('/') => {
                                        self.at += 1;
                                        let start = self.at;
                                        while self
                                            .chars
                                            .get(self.at)
                                            .is_some_and(|c| c.is_ascii_alphabetic())
                                        {
                                            self.at += 1;
                                        }
                                        let mode =
                                            self.chars[start..self.at].iter().collect::<String>();
                                        if !matches!(
                                            mode.as_str(),
                                            "upcase"
                                                | "downcase"
                                                | "capitalize"
                                                | "camelcase"
                                                | "pascalcase"
                                        ) {
                                            return Err("Unsupported transform casing".into());
                                        }
                                        json!(["capture", index, mode])
                                    }
                                    Some('+') => {
                                        self.at += 1;
                                        json!([
                                            "if",
                                            index,
                                            self.sequence(Some('}'), depth + 1)?,
                                            []
                                        ])
                                    }
                                    Some('?') => {
                                        self.at += 1;
                                        let yes = self.sequence(Some(':'), depth + 1)?;
                                        if self.chars.get(self.at) != Some(&':') {
                                            return Err(
                                                "Missing transform conditional branch".into()
                                            );
                                        }
                                        self.at += 1;
                                        let no = self.sequence(Some('}'), depth + 1)?;
                                        json!(["if", index, yes, no])
                                    }
                                    _ => {
                                        if self.chars.get(self.at) == Some(&'-') {
                                            self.at += 1;
                                        }
                                        json!([
                                            "if",
                                            index,
                                            [["capture", index, ""]],
                                            self.sequence(Some('}'), depth + 1)?
                                        ])
                                    }
                                }
                            }
                            _ => return Err("Malformed transform capture format".into()),
                        }
                    };
                    if braced {
                        if self.chars.get(self.at) != Some(&'}') {
                            return Err("Unclosed transform format".into());
                        }
                        self.at += 1;
                    }
                    self.nodes += 1;
                    if self.nodes > MAX_NODES {
                        return Err("Transform format limit exceeded".into());
                    }
                    result.push(value);
                }
                ch => text.push(ch),
            }
        }
        if !text.is_empty() {
            result.push(json!(text))
        }
        Ok(result)
    }
}

/// Compile once in the engine; hosts accept only this versioned bounded program.
/// The client must opt into a complete program evaluator; never fall back after a v3 error.
pub fn validate_program_versions(programs:&[u32])->Result<(),String>{if programs.is_empty()||programs.len()>2||programs.iter().any(|n|![2,3].contains(n))||programs.iter().enumerate().any(|(i,n)|programs[..i].contains(n)){return Err("Unsupported snippet transform program versions".into())};Ok(())}
pub fn compile_supported(pattern:&str,format:&str,flags:&str,programs:&[u32])->Result<Value,String>{validate_program_versions(programs)?;if programs.contains(&3){v3::compile(pattern,format,flags)}else{compile(pattern,format,flags)}}
pub fn compile(pattern: &str, format: &str, flags: &str) -> Result<Value, String> {
    if pattern.len() + format.len() > 8192 {
        return Err("Transform syntax limit exceeded".into());
    }
    let mut seen = std::collections::BTreeSet::new();
    for flag in flags.chars() {
        if !matches!(flag, 'g' | 'i' | 'm' | 's' | 'u') || !seen.insert(flag) {
            return Err("Unsupported or duplicate transform flag".into());
        }
    }
    let mut parser = RegexParser {
        chars: pattern.chars().collect(),
        at: 0,
        nodes: 0,
        groups: 0,
    };
    let mut regex = parser.expression(0)?;
    if parser.at != parser.chars.len() {
        return Err("Unexpected transform group terminator".into());
    }
    // Expand once from the frozen simple-fold relation. Runtime hosts only test ranges.
    fn canonical(cp: u32) -> u32 {
        let ch = char::from_u32(cp).unwrap();
        let upper = snippet_case::convert(&ch.to_string(), true);
        let mut chars = upper.chars();
        let first = chars.next().unwrap();
        if chars.next().is_some() || cp >= 128 && (first as u32) < 128 {
            cp
        } else {
            first as u32
        }
    }
    fn fold(node: &mut Value, unicode: bool, work: &mut usize) -> Result<(), String> {
        match node[0].as_str() {
            Some("seq" | "alt") => {
                for child in node[1].as_array_mut().unwrap() {
                    fold(child, unicode, work)?
                }
            }
            Some("capture") => fold(&mut node[2], unicode, work)?,
            Some("repeat") => fold(&mut node[1], unicode, work)?,
            Some("class") => {
                let original = node[1]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| [v[0].as_u64().unwrap() as u32, v[1].as_u64().unwrap() as u32])
                    .collect::<Vec<_>>();
                let contains = |cp: u32| original.iter().any(|r| r[0] <= cp && cp <= r[1]);
                let mut ranges = original.clone();
                for (cp, other) in snippet_case::simple_folds() {
                    *work += original.len() + other.len();
                    if *work > 2_000_000 {
                        return Err("Transform compile work limit exceeded".into());
                    }
                    if contains(*cp) {
                        continue;
                    }
                    if other.iter().any(|candidate| {
                        contains(*candidate) && (unicode || canonical(*cp) == canonical(*candidate))
                    }) {
                        ranges.push([*cp, *cp])
                    }
                }
                ranges.sort_unstable();
                let mut merged: Vec<[u32; 2]> = vec![];
                for range in ranges {
                    if let Some(last) = merged.last_mut() {
                        if range[0] <= last[1] + 1 {
                            last[1] = last[1].max(range[1]);
                            continue;
                        }
                    }
                    merged.push(range)
                }
                node[1] = json!(merged)
            }
            _ => {}
        }
        Ok(())
    }
    if flags.contains('i') {
        fold(&mut regex, flags.contains('u'), &mut 0)?
    }
    let mut replacement = FormatParser {
        chars: format.chars().collect(),
        at: 0,
        nodes: 0,
    };
    let format = replacement.sequence(None, 0)?;
    fn regex_cost(node: &Value) -> usize {
        1 + match node[0].as_str() {
            Some("seq" | "alt") => node[1].as_array().unwrap().iter().map(regex_cost).sum(),
            Some("capture") => regex_cost(&node[2]),
            Some("repeat") => regex_cost(&node[1]),
            Some("class") => node[1].as_array().unwrap().len(),
            _ => 0,
        }
    }
    fn format_cost(ops: &[Value]) -> usize {
        ops.iter()
            .map(|op| {
                1 + if op[0] == "if" {
                    format_cost(op[2].as_array().unwrap()) + format_cost(op[3].as_array().unwrap())
                } else {
                    0
                }
            })
            .sum()
    }
    if regex_cost(&regex) + format_cost(&format) > MAX_NODES {
        return Err("Transform program limit exceeded".into());
    }
    Ok(
        json!({"version":2,"case_map":snippet_case::VERSION,"regex":regex,"format":format,"flags":flags,"groups":parser.groups}),
    )
}

#[derive(Clone)]
struct MatchState {
    at: usize,
    captures: Vec<Option<(usize, usize)>>,
}
struct Evaluator {
    text: Vec<char>,
    flags: String,
    work: usize,
}
impl Evaluator {
    fn tick(&mut self, count: usize) -> Result<(), String> {
        self.work = self
            .work
            .checked_add(count)
            .ok_or("Transform work limit exceeded")?;
        if self.work > MAX_WORK {
            return Err("Transform work limit exceeded".into());
        }
        Ok(())
    }
    fn word(&self, ch: Option<char>) -> bool {
        ch.is_some_and(|c| {
            c.is_ascii_alphanumeric()
                || c == '_'
                || self.flags.contains('i') && self.flags.contains('u') && matches!(c, 'ſ' | 'K')
        })
    }
    fn matches(
        &mut self,
        node: &Value,
        state: MatchState,
        depth: usize,
    ) -> Result<Vec<MatchState>, String> {
        self.tick(1)?;
        if depth > 64 {
            return Err("Transform evaluation nesting limit exceeded".into());
        }
        let nodes = node.as_array().ok_or("Malformed transform program")?;
        let op = nodes
            .first()
            .and_then(Value::as_str)
            .ok_or("Malformed transform operation")?;
        let result = match op {
            "seq" => {
                let mut states = vec![state];
                for node in nodes[1].as_array().ok_or("Malformed transform sequence")? {
                    let mut next = vec![];
                    for state in states {
                        next.extend(self.matches(node, state, depth + 1)?);
                        if next.len() > MAX_STATES {
                            return Err("Transform state limit exceeded".into());
                        }
                    }
                    states = next;
                    if states.is_empty() {
                        break;
                    }
                }
                states
            }
            "alt" => {
                let mut states = vec![];
                for node in nodes[1]
                    .as_array()
                    .ok_or("Malformed transform alternatives")?
                {
                    states.extend(self.matches(node, state.clone(), depth + 1)?);
                    if states.len() > MAX_STATES {
                        return Err("Transform state limit exceeded".into());
                    }
                }
                states
            }
            "capture" => {
                let index = nodes[1].as_u64().ok_or("Invalid transform capture")? as usize;
                let start = state.at;
                let mut states = self.matches(&nodes[2], state, depth + 1)?;
                for state in &mut states {
                    *state
                        .captures
                        .get_mut(index)
                        .ok_or("Invalid transform capture")? = Some((start, state.at))
                }
                states
            }
            "repeat" => {
                let min = nodes[2].as_u64().ok_or("Invalid transform minimum")? as usize;
                let max = nodes[3]
                    .as_u64()
                    .map(|n| n as usize)
                    .unwrap_or(self.text.len() + 1)
                    .min(4096);
                let lazy = nodes[4].as_bool().ok_or("Invalid transform greediness")?;
                let mut levels = vec![vec![state]];
                for _ in 0..max {
                    let mut next = vec![];
                    for state in levels.last().unwrap() {
                        let matched = self.matches(&nodes[1], state.clone(), depth + 1)?;
                        for value in matched {
                            if value.at != state.at {
                                next.push(value)
                            } else if levels.len() <= min {
                                next.push(value)
                            }
                        }
                        if next.len() > MAX_STATES {
                            return Err("Transform state limit exceeded".into());
                        }
                    }
                    if next.is_empty() {
                        break;
                    }
                    levels.push(next);
                    if levels.iter().map(Vec::len).sum::<usize>() > MAX_STATES {
                        return Err("Transform state limit exceeded".into());
                    }
                }
                let mut result = vec![];
                let indexes: Vec<_> = if lazy {
                    (min..levels.len()).collect()
                } else {
                    (min..levels.len()).rev().collect()
                };
                for index in indexes {
                    result.extend(levels[index].iter().cloned())
                }
                result
            }
            "assert" => {
                let before = state
                    .at
                    .checked_sub(1)
                    .and_then(|i| self.text.get(i))
                    .copied();
                let next = self.text.get(state.at).copied();
                let boundary = self.word(before) != self.word(next);
                let yes = match nodes[1].as_str() {
                    Some("start") => {
                        state.at == 0
                            || self.flags.contains('m')
                                && before.is_some_and(|c| {
                                    matches!(c, '\n' | '\r' | '\u{2028}' | '\u{2029}')
                                })
                    }
                    Some("end") => {
                        state.at == self.text.len()
                            || next
                                .is_some_and(|c| matches!(c, '\n' | '\r' | '\u{2028}' | '\u{2029}'))
                                && (self.flags.contains('m')
                                    || state.at + 1 == self.text.len()
                                    || state.at + 2 == self.text.len()
                                        && next == Some('\r')
                                        && self.text.last() == Some(&'\n'))
                    }
                    Some("word") => boundary,
                    Some("nonword") => !boundary,
                    _ => return Err("Invalid transform assertion".into()),
                };
                if yes { vec![state] } else { vec![] }
            }
            "class" | "dot" => {
                if let Some(&ch) = self.text.get(state.at) {
                    let yes = if op == "dot" {
                        self.flags.contains('s')
                            || !matches!(ch, '\n' | '\r' | '\u{2028}' | '\u{2029}')
                    } else {
                        let ranges = nodes[1].as_array().ok_or("Malformed transform class")?;
                        self.tick(ranges.len())?;
                        let contains = |ch: char| {
                            ranges.iter().any(|range| {
                                range[0].as_u64().is_some_and(|low| low <= ch as u64)
                                    && range[1].as_u64().is_some_and(|high| ch as u64 <= high)
                            })
                        };
                        let found = contains(ch);
                        found
                            != nodes[2]
                                .as_bool()
                                .ok_or("Invalid transform class negation")?
                    };
                    if yes {
                        vec![MatchState {
                            at: state.at + 1,
                            ..state
                        }]
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                }
            }
            _ => return Err("Unsupported transform operation".into()),
        };
        self.tick(result.len())?;
        Ok(result)
    }
    fn format(
        &mut self,
        ops: &Value,
        captures: &[Option<(usize, usize)>],
        depth: usize,
    ) -> Result<String, String> {
        if depth > 32 {
            return Err("Transform format nesting limit exceeded".into());
        }
        let mut output = String::new();
        for op in ops.as_array().ok_or("Malformed transform format")? {
            self.tick(1)?;
            let text = if let Some(text) = op.as_str() {
                text.to_owned()
            } else {
                let index = op[1].as_u64().ok_or("Invalid format capture")? as usize;
                let value = captures
                    .get(index)
                    .copied()
                    .flatten()
                    .map(|(from, to)| self.text[from..to].iter().collect::<String>())
                    .unwrap_or_default();
                match op[0].as_str() {
                    Some("capture") => match op[2].as_str() {
                        Some("") => value,
                        Some("upcase") => snippet_case::convert(&value, true),
                        Some("downcase") => snippet_case::convert(&value, false),
                        Some("capitalize") => {
                            let mut chars = value.chars();
                            chars
                                .next()
                                .map(|first| {
                                    snippet_case::convert(&first.to_string(), true) + chars.as_str()
                                })
                                .unwrap_or_default()
                        }
                        Some("camelcase" | "pascalcase") => {
                            let words = snippet_case::words(&value);
                            let pascal = op[2] == "pascalcase";
                            if words.is_empty() {
                                value
                            } else {
                                words
                                    .iter()
                                    .enumerate()
                                    .map(|(i, word)| {
                                        let mut chars = word.chars();
                                        chars
                                            .next()
                                            .map(|first| {
                                                (if i == 0 && !pascal {
                                                    snippet_case::convert(&first.to_string(), false)
                                                } else {
                                                    snippet_case::convert(&first.to_string(), true)
                                                }) + chars.as_str()
                                            })
                                            .unwrap_or_default()
                                    })
                                    .collect()
                            }
                        }
                        _ => return Err("Unsupported transform casing".into()),
                    },
                    Some("if") => self.format(
                        if value.is_empty() { &op[3] } else { &op[2] },
                        captures,
                        depth + 1,
                    )?,
                    _ => return Err("Invalid transform format operation".into()),
                }
            };
            if output.len() + text.len() > MAX_OUTPUT {
                return Err("Transform output limit exceeded".into());
            }
            output.push_str(&text);
        }
        Ok(output)
    }
}
/// Evaluate the compiled program with deterministic work and output budgets.
pub fn apply(program: &Value, input: &str) -> Result<String, String> {
    let mut work = 0;
    apply_with_budget(program, input, &mut work)
}
/// Share one work budget across every derived occurrence in an atomic action.
pub fn apply_with_budget(program: &Value, input: &str, work: &mut usize) -> Result<String, String> {
    if program["version"].as_f64()==Some(3.0){return v3::apply(program,input,work)}
    if program["version"] != 2
        || program["case_map"] != snippet_case::VERSION
        || input.len() > MAX_TEXT
    {
        return Err("Invalid or excessive transform input".into());
    }
    let groups = program["groups"]
        .as_u64()
        .filter(|n| *n <= 32)
        .ok_or("Invalid transform capture count")? as usize;
    let flags = program["flags"]
        .as_str()
        .ok_or("Invalid transform flags")?
        .to_owned();
    let mut evaluator = Evaluator {
        text: input.chars().collect(),
        flags,
        work: *work,
    };
    let mut output = String::new();
    let mut cursor = 0;
    let mut search = 0;
    let mut matched = false;
    let length = evaluator.text.len();
    while search <= length {
        let mut found = None;
        for start in search..=length {
            let initial = MatchState {
                at: start,
                captures: vec![None; groups + 1],
            };
            if let Some(mut state) = evaluator
                .matches(&program["regex"], initial, 0)?
                .into_iter()
                .next()
            {
                state.captures[0] = Some((start, state.at));
                found = Some((start, state));
                break;
            }
        }
        let Some((start, state)) = found else { break };
        matched = true;
        let prefix = evaluator.text[cursor..start].iter().collect::<String>();
        let replacement = evaluator.format(&program["format"], &state.captures, 0)?;
        if output.len() + prefix.len() + replacement.len() > MAX_OUTPUT {
            return Err("Transform output limit exceeded".into());
        }
        output.push_str(&prefix);
        output.push_str(&replacement);
        cursor = state.at;
        if !evaluator.flags.contains('g') {
            break;
        }
        search = if state.at == start {
            state.at + 1
        } else {
            state.at
        };
    }
    if !matched
        && program["format"].as_array().is_some_and(|ops| {
            ops.iter().any(|op| {
                op[0] == "if" && op[3].as_array().is_some_and(|branch| !branch.is_empty())
            })
        })
    {
        let value = evaluator.format(&program["format"], &vec![None; groups + 1], 0)?;
        *work = evaluator.work;
        return Ok(value);
    }
    let suffix = evaluator.text[cursor..].iter().collect::<String>();
    if output.len() + suffix.len() > MAX_OUTPUT {
        return Err("Transform output limit exceeded".into());
    }
    output.push_str(&suffix);
    *work = evaluator.work;
    Ok(output)
}

#[cfg(test)]
mod common_vectors {
    use super::*;
    #[test]
    fn shared_transform_vectors_match_the_compiler_and_evaluator() {
        let cases: Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../apps/simple-contract/snippet-transform-cases.json"
        )))
        .unwrap();
        for case in cases.as_array().unwrap() {
            let program = compile(
                case["pattern"].as_str().unwrap(),
                case["format"].as_str().unwrap(),
                case["flags"].as_str().unwrap(),
            )
            .unwrap();
            assert_eq!(program, case["program"], "{}", case["name"]);
            let result = apply(&program, case["input"].as_str().unwrap());
            if case["reject"] == true {
                assert!(result.is_err(), "{}", case["name"])
            } else {
                assert_eq!(
                    result.unwrap(),
                    case["expected"].as_str().unwrap(),
                    "{}",
                    case["name"]
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn captures_alternatives_repetition_and_replacement_casing() {
        let value = compile(r"([a-z]+)[_-]([a-z]+)", "${1:/capitalize}${2:/upcase}", "g").unwrap();
        assert_eq!(
            apply(&value, "foo_bar and zip-zap").unwrap(),
            "FooBAR and ZipZAP"
        );
        let lazy = compile("(a+?)(a)", "<$1:$2>", "").unwrap();
        assert_eq!(apply(&lazy, "aaaa").unwrap(), "<a:a>aa");
    }
    #[test]
    fn conditionals_zero_width_unicode_and_flags() {
        let value = compile("(foo)?bar", "${1:+yes}${1:?Y:N}${1:-none}", "g").unwrap();
        assert_eq!(apply(&value, "foobar bar").unwrap(), "yesYfoo Nnone");
        assert_eq!(
            apply(&compile("^|$", "|", "g").unwrap(), "😀λ").unwrap(),
            "|😀λ|"
        );
        assert_eq!(
            apply(&compile("([a-z]+)", "${1:/upcase}", "gi").unwrap(), "aBc").unwrap(),
            "ABC"
        );
    }
    #[test]
    fn else_formats_apply_when_the_regex_has_no_match() {
        assert_eq!(
            apply(&compile("(.+)", "${1:?yes:no}", "").unwrap(), "").unwrap(),
            "no"
        );
        assert_eq!(
            apply(&compile("(a)", "${1:+x${1}}", "").unwrap(), "a").unwrap(),
            "xa"
        );
        assert_eq!(
            apply(
                &compile("(.*)", "${1:/camelcase}", "").unwrap(),
                "HELLO world"
            )
            .unwrap(),
            "hELLOWorld"
        );
    }
    #[test]
    fn unsupported_or_excessive_patterns_refuse_without_partial_output() {
        for pattern in ["(?=a)", "(a)\\1", "[z-a]", "a{300}", "\\p{L}"] {
            assert!(compile(pattern, "$1", "").is_err(), "{pattern}")
        }
        assert!(compile("a", "$1", "gg").is_err());
        assert!(apply(&compile("(a|aa)*z", "x", "").unwrap(), &"a".repeat(40)).is_err());
    }
}
</ui:copy>
<ui:file path="crates/hick-lsp/src/snippet_transform.rs"><ui:paste select="#snippet-transform-engine"/></ui:file>

## Evaluate the compiled program in browser source models

Admission copies and freezes a bounded program before it can join source history. The evaluator counts node, range and result work across all derived occurrences in one action. Scalar arrays keep a match from splitting a surrogate pair; the surrounding session still validates exact UTF-16 edit boundaries. Exhaustion throws before any source action is submitted.

<ui:copy id="snippet-transform-browser">
import {snippetCase,snippetCaseWords,snippetCaseVersion} from './SnippetCase';
import {readRegexV3,evaluateRegexV3,regexV3ProgramBytes,type RegexV3Program} from './SnippetTransformV3';
export const supportedSnippetTransformPrograms=Object.freeze([2,3] as const);
const scalarText=(text:string)=>!/[\uD800-\uDFFF]/u.test(text);
export interface SnippetTransformV2 {readonly version:2;readonly case_map:typeof snippetCaseVersion;readonly regex:readonly unknown[];readonly format:readonly unknown[];readonly flags:string;readonly groups:number}
export type SnippetTransform=SnippetTransformV2|(RegexV3Program & {readonly regex?:never});
export interface SnippetTransformBudget {work:number}
const maximum=1048576;
export function readSnippetTransform(raw:unknown):SnippetTransform {
 if(raw&&typeof raw==='object'&&'version' in raw&&raw.version===3)return readRegexV3(raw);
 let count=0,bytes=0;const tick=(n=1)=>{count+=n;if(count>512)throw Error('Snippet transform program limit exceeded');};
 const integer=(v:unknown,max=1114111)=>{if(typeof v!=='number'||!Number.isSafeInteger(v)||v<0||v>max)throw Error('Malformed snippet transform integer');return v;};
 const text=(v:unknown)=>{if(typeof v!=='string'||!scalarText(v))throw Error('Malformed snippet transform text');bytes+=new TextEncoder().encode(v).length;if(bytes>16384)throw Error('Snippet transform metadata limit exceeded');return v;};
 if(!raw||typeof raw!=='object'||Array.isArray(raw))throw Error('Malformed snippet transform');const value=raw as Record<string,unknown>;
 if(value.version!==2||value.case_map!==snippetCaseVersion)throw Error('Unsupported snippet transform version or case policy');const groups=integer(value.groups,32),flags=text(value.flags);if(!/^[gimsu]*$/.test(flags)||new Set(flags).size!==flags.length)throw Error('Unsupported transform flags');
 const node=(v:unknown,depth:number):readonly unknown[]=>{
  tick();if(depth>64||!Array.isArray(v)||typeof v[0]!=='string')throw Error('Malformed snippet transform node');const op=v[0];let result:unknown[];
  switch(op){
   case 'seq':case 'alt':if(v.length!==2||!Array.isArray(v[1])||v[1].length>512)throw Error('Malformed transform branches');result=[op,Object.freeze(v[1].map(n=>node(n,depth+1)))];break;
   case 'capture':if(v.length!==3)throw Error('Malformed transform capture');result=[op,integer(v[1],groups),node(v[2],depth+1)];break;
   case 'repeat':if(v.length!==5||typeof v[4]!=='boolean')throw Error('Malformed transform repetition');{const min=integer(v[2],256),max=v[3]===null?null:integer(v[3],256);if(max!==null&&max<min)throw Error('Backwards transform repetition');result=[op,node(v[1],depth+1),min,max,v[4]];}break;
   case 'class':if(v.length!==3||!Array.isArray(v[1])||typeof v[2]!=='boolean')throw Error('Malformed transform class');result=[op,Object.freeze(v[1].map(range=>{tick();if(!Array.isArray(range)||range.length!==2)throw Error('Malformed transform range');const low=integer(range[0]),high=integer(range[1]);if(low>high)throw Error('Backwards transform range');return Object.freeze([low,high]);})),v[2]];break;
   case 'dot':if(v.length!==1)throw Error('Malformed transform dot');result=[op];break;
   case 'assert':if(v.length!==2||!['start','end','word','nonword'].includes(v[1]))throw Error('Malformed transform anchor');result=[op,v[1]];break;
   default:throw Error('Unsupported snippet transform operation');
  }return Object.freeze(result);
 };
 const format=(v:unknown,depth:number):readonly unknown[]=>{if(depth>32||!Array.isArray(v)||v.length>512)throw Error('Malformed transform format');return Object.freeze(v.map(op=>{tick();if(typeof op==='string')return text(op);if(!Array.isArray(op))throw Error('Malformed transform format');if(op[0]==='capture'&&op.length===3&&['','upcase','downcase','capitalize','camelcase','pascalcase'].includes(op[2]))return Object.freeze(['capture',integer(op[1],32),op[2]]);if(op[0]==='if'&&op.length===4)return Object.freeze(['if',integer(op[1],32),format(op[2],depth+1),format(op[3],depth+1)]);throw Error('Unsupported transform format');}));};
 return Object.freeze({version:2,case_map:snippetCaseVersion,regex:node(value.regex,0),format:format(value.format,0),flags,groups});
}
export function snippetTransformBytes(program:SnippetTransform):number{return program.version===3?regexV3ProgramBytes(program):new TextEncoder().encode(JSON.stringify(program)).length;}
interface MatchState {at:number;captures:({from:number;to:number}|null)[]}
const newline=(c:string|undefined)=>c!==undefined&&['\n','\r','\u2028','\u2029'].includes(c),word=(c:string|undefined)=>c!==undefined&&/^[A-Za-z0-9_]$/.test(c);
export function applySnippetTransform(raw:SnippetTransform,input:string,budget:SnippetTransformBudget={work:0}):string {
 const program=readSnippetTransform(raw);if(program.version===3)return evaluateRegexV3(program,input,budget);if(!scalarText(input)||new TextEncoder().encode(input).length>65536)throw Error('Snippet transform input limit exceeded');const chars=Array.from(input),flags=program.flags;
 const tick=(n=1)=>{budget.work+=n;if(!Number.isSafeInteger(budget.work)||budget.work<0||budget.work>100000)throw Error('Snippet transform work limit exceeded');};
 const matches=(node:readonly unknown[],state:MatchState,depth:number):MatchState[]=>{
  tick();if(depth>64)throw Error('Snippet transform evaluation depth exceeded');const op=node[0];let result:MatchState[]=[];
  switch(op){
   case 'seq':{let states=[state];for(const child of node[1] as readonly (readonly unknown[])[]){const next:MatchState[]=[];for(const state of states){next.push(...matches(child,state,depth+1));if(next.length>1024)throw Error('Transform state limit exceeded');}states=next;if(!states.length)break;}result=states;break;}
   case 'alt':for(const child of node[1] as readonly (readonly unknown[])[]){result.push(...matches(child,state,depth+1));if(result.length>1024)throw Error('Transform state limit exceeded');}break;
   case 'capture':result=matches(node[2] as readonly unknown[],state,depth+1).map(value=>{const captures=[...value.captures];captures[node[1] as number]={from:state.at,to:value.at};return {...value,captures};});break;
   case 'repeat':{const min=node[2] as number,max=Math.min((node[3] as number|null)??chars.length+1,4096),levels:MatchState[][]=[[state]];let total=1;for(let i=0;i<max;i++){const next:MatchState[]=[];for(const state of levels.at(-1)!){for(const value of matches(node[1] as readonly unknown[],state,depth+1)){if(value.at!==state.at||levels.length<=min)next.push(value);}if(next.length>1024)throw Error('Transform state limit exceeded');}if(!next.length)break;levels.push(next);total+=next.length;if(total>1024)throw Error('Transform state limit exceeded');}const indexes=Array.from({length:Math.max(0,levels.length-min)},(_,i)=>i+min);if(!node[4])indexes.reverse();for(const i of indexes)result.push(...levels[i]);break;}
   case 'assert':{const before=chars[state.at-1],next=chars[state.at],boundary=(word(before)||flags.includes('i')&&flags.includes('u')&&(before==='ſ'||before==='K'))!==(word(next)||flags.includes('i')&&flags.includes('u')&&(next==='ſ'||next==='K')),yes=node[1]==='start'?state.at===0||flags.includes('m')&&newline(before):node[1]==='end'?state.at===chars.length||newline(next)&&(flags.includes('m')||state.at+1===chars.length||state.at+2===chars.length&&next==='\r'&&chars.at(-1)==='\n'):node[1]==='word'?boundary:!boundary;if(yes)result=[state];break;}
   case 'class':case 'dot':{const ch=chars[state.at];if(ch!==undefined){let yes:boolean;if(op==='dot')yes=flags.includes('s')||!newline(ch);else{const ranges=node[1] as readonly (readonly number[])[];tick(ranges.length);const contains=(c:string)=>ranges.some(([low,high])=>low<=c.codePointAt(0)!&&c.codePointAt(0)!<=high),found=contains(ch);yes=found!==node[2];}if(yes)result=[{at:state.at+1,captures:state.captures}];}break;}
  }tick(result.length);return result;
 };
 const format=(ops:readonly unknown[],captures:MatchState['captures'],depth:number):string=>{if(depth>32)throw Error('Transform format depth exceeded');let output='';for(const raw of ops){tick();let value:string;if(typeof raw==='string')value=raw;else{const op=raw as readonly unknown[],range=captures[op[1] as number],captured=range?chars.slice(range.from,range.to).join(''):'';if(op[0]==='if')value=format(op[captured?2:3] as readonly unknown[],captures,depth+1);else switch(op[2]){case 'upcase':value=snippetCase(captured,'upper');break;case 'downcase':value=snippetCase(captured,'lower');break;case 'capitalize':{const scalar=Array.from(captured);value=(snippetCase(scalar.shift()??'','upper'))+scalar.join('');break;}case 'camelcase':case 'pascalcase':{const words=snippetCaseWords(captured);value=words.length?words.map((word,i)=>(i===0&&op[2]==='camelcase'?snippetCase(Array.from(word)[0],'lower'):snippetCase(Array.from(word)[0],'upper'))+Array.from(word).slice(1).join('')).join(''):captured;break;}default:value=captured;}}if(new TextEncoder().encode(output).length+new TextEncoder().encode(value).length>maximum)throw Error('Transform output limit exceeded');output+=value;}return output;};
 let output='',cursor=0,search=0,matched=false;while(search<=chars.length){let found:{start:number;state:MatchState}|undefined;for(let start=search;start<=chars.length;start++){const state=matches(program.regex,{at:start,captures:Array(program.groups+1).fill(null)},0)[0];if(state){state.captures=[...state.captures];state.captures[0]={from:start,to:state.at};found={start,state};break;}}if(!found)break;matched=true;const {start,state}=found,prefix=chars.slice(cursor,start).join(''),replacement=format(program.format,state.captures,0);if(new TextEncoder().encode(output+prefix+replacement).length>maximum)throw Error('Transform output limit exceeded');output+=prefix+replacement;cursor=state.at;if(!flags.includes('g'))break;search=state.at===start?state.at+1:state.at;}if(!matched&&program.format.some(op=>Array.isArray(op)&&op[0]==='if'&&(op[3] as readonly unknown[]).length>0))return format(program.format,Array(program.groups+1).fill(null),0);const suffix=chars.slice(cursor).join('');if(new TextEncoder().encode(output+suffix).length>maximum)throw Error('Transform output limit exceeded');return output+suffix;
}
</ui:copy>
<ui:file path="apps/web/src/simple/SnippetTransform.ts"><ui:paste select="#snippet-transform-browser"/></ui:file>

## Evaluate the same program in Apple source models

Apple retains only normalized bounded arrays. The evaluator uses Unicode scalar positions for captures and shares a work budget across the complete linked source action; it never calls platform regular-expression APIs.

<ui:copy id="snippet-transform-apple">
import Foundation
import CoreFoundation
let supportedSnippetTransformPrograms=[2,3]
struct SnippetTransform {let regex:[Any];let format:[Any];let flags:String;let groups:Int;let bytes:Int;var v3:RegexV3Program?=nil}
struct SnippetTransformBudget {var work=0}
func readSnippetTransform(_ raw:Any)throws->SnippetTransform {
 if let value=raw as? [String:Any],let version=value["version"] as? NSNumber,CFGetTypeID(version) != CFBooleanGetTypeID(),version.doubleValue==3{let program=try readRegexV3(value);return SnippetTransform(regex:[],format:program.format,flags:program.flags,groups:program.groups,bytes:program.bytes,v3:program)}
 var count=0,bytes=0
 func tick(_ n:Int=1)throws{count+=n;guard count<=512 else{throw EngineError(message:"Snippet transform program limit exceeded")}}
 func integer(_ raw:Any,_ max:Int=1114111)throws->Int{guard let n=raw as? NSNumber,CFGetTypeID(n) != CFBooleanGetTypeID(),let value=Int(n.stringValue),value>=0,value<=max else{throw EngineError(message:"Malformed transform integer")};return value}
 func boolean(_ raw:Any)throws->Bool{guard let n=raw as? NSNumber,CFGetTypeID(n)==CFBooleanGetTypeID() else{throw EngineError(message:"Malformed transform Boolean")};return n.boolValue}
 func text(_ raw:Any)throws->String{guard let text=raw as? String else{throw EngineError(message:"Malformed transform text")};bytes+=text.utf8.count;guard bytes<=16384 else{throw EngineError(message:"Transform metadata limit exceeded")};return text}
 guard let raw=raw as? [String:Any],try integer(raw["version"] as Any)==2,raw["case_map"] as? String==snippetCaseVersion else{throw EngineError(message:"Unsupported snippet transform version")};let groups=try integer(raw["groups"] as Any,32),flags=try text(raw["flags"] as Any)
 guard flags.allSatisfy({"gimsu".contains($0)}),Set(flags).count==flags.count else{throw EngineError(message:"Unsupported transform flags")}
 func node(_ raw:Any,_ depth:Int)throws->[Any]{try tick();guard depth<=64,let v=raw as? [Any],let op=v.first as? String else{throw EngineError(message:"Malformed transform node")}
  switch op {
  case "seq","alt":guard v.count==2,let children=v[1] as? [Any],children.count<=512 else{throw EngineError(message:"Malformed transform branches")};return [op,try children.map{try node($0,depth+1)}]
  case "capture":guard v.count==3 else{throw EngineError(message:"Malformed transform capture")};return [op,try integer(v[1],groups),try node(v[2],depth+1)]
  case "repeat":guard v.count==5 else{throw EngineError(message:"Malformed transform repetition")};let min=try integer(v[2],256),max=v[3] is NSNull ? nil:try integer(v[3],256);guard max==nil||max!>=min else{throw EngineError(message:"Backwards transform repetition")};return [op,try node(v[1],depth+1),min,max as Any? ?? NSNull(),try boolean(v[4])]
  case "class":guard v.count==3,let ranges=v[1] as? [Any] else{throw EngineError(message:"Malformed transform class")};let values=try ranges.map{raw->[Int] in try tick();guard let range=raw as? [Any],range.count==2 else{throw EngineError(message:"Malformed transform range")};let low=try integer(range[0]),high=try integer(range[1]);guard low<=high else{throw EngineError(message:"Backwards transform range")};return [low,high]};return [op,values,try boolean(v[2])]
  case "dot":guard v.count==1 else{throw EngineError(message:"Malformed transform dot")};return [op]
  case "assert":guard v.count==2,let kind=v[1] as? String,["start","end","word","nonword"].contains(kind) else{throw EngineError(message:"Malformed transform assertion")};return [op,kind]
  default:throw EngineError(message:"Unsupported transform operation")
  }
 }
 func format(_ raw:Any,_ depth:Int)throws->[Any]{guard depth<=32,let values=raw as? [Any],values.count<=512 else{throw EngineError(message:"Malformed transform format")};return try values.map{raw->Any in try tick();if raw is String{return try text(raw)};guard let op=raw as? [Any],let kind=op.first as? String else{throw EngineError(message:"Malformed transform format")};if kind=="capture",op.count==3,let mode=op[2] as? String,["","upcase","downcase","capitalize","camelcase","pascalcase"].contains(mode){return [kind,try integer(op[1],32),mode]};if kind=="if",op.count==4{return [kind,try integer(op[1],32),try format(op[2],depth+1),try format(op[3],depth+1)]};throw EngineError(message:"Unsupported transform format")}}
 let regex=try node(raw["regex"] as Any,0),format=try format(raw["format"] as Any,0),data=try JSONSerialization.data(withJSONObject:["version":2,"case_map":snippetCaseVersion,"regex":regex,"format":format,"groups":groups,"flags":flags]);return SnippetTransform(regex:regex,format:format,flags:flags,groups:groups,bytes:data.count)
}
private struct TransformMatch {var at:Int;var captures:[Range<Int>?]}
private struct TransformEvaluator {
 let text:[Unicode.Scalar];let flags:String;var work:Int
 mutating func tick(_ n:Int=1)throws{guard n>=0,work>=0,work<=100000-n else{throw EngineError(message:"Snippet transform work limit exceeded")};work+=n}
 func newline(_ c:Unicode.Scalar?)->Bool{c.map{[10,13,0x2028,0x2029].contains(Int($0.value))} ?? false}
 func word(_ c:Unicode.Scalar?)->Bool{guard let c else{return false};return c.value>=48&&c.value<=57||c.value>=65&&c.value<=90||c.value==95||c.value>=97&&c.value<=122||flags.contains("i")&&flags.contains("u")&&(c.value==383||c.value==8490)}
 func string(_ from:Int,_ to:Int)->String{text[from..<to].map(String.init).joined()}
 mutating func matches(_ node:[Any],_ state:TransformMatch,_ depth:Int)throws->[TransformMatch]{try tick();guard depth<=64 else{throw EngineError(message:"Transform evaluation nesting limit exceeded")};let op=node[0] as! String;var result:[TransformMatch]=[]
  switch op {
  case "seq":var states=[state];for child in node[1] as! [[Any]]{var next:[TransformMatch]=[];for state in states{next+=try matches(child,state,depth+1);guard next.count<=1024 else{throw EngineError(message:"Transform state limit exceeded")}};states=next;if states.isEmpty{break}};result=states
  case "alt":for child in node[1] as! [[Any]]{result+=try matches(child,state,depth+1);guard result.count<=1024 else{throw EngineError(message:"Transform state limit exceeded")}}
  case "capture":let start=state.at,index=node[1] as! Int;result=try matches(node[2] as! [Any],state,depth+1);for i in result.indices{result[i].captures[index]=start..<result[i].at}
  case "repeat":let min=node[2] as! Int,max=Swift.min(node[3] as? Int ?? text.count+1,4096),lazy=node[4] as! Bool;var levels=[[state]],total=1
   for _ in 0..<max{var next:[TransformMatch]=[];let previous=levels.last!;for state in previous{for value in try matches(node[1] as! [Any],state,depth+1){if value.at != state.at || levels.count<=min{next.append(value)}};guard next.count<=1024 else{throw EngineError(message:"Transform state limit exceeded")}};if next.isEmpty{break};levels.append(next);total+=next.count;guard total<=1024 else{throw EngineError(message:"Transform state limit exceeded")}}
   if min<levels.count{let indexes=Array(min..<levels.count);for i in lazy ? indexes:Array(indexes.reversed()){result+=levels[i]}}
  case "assert":let before=state.at>0 ? text[state.at-1]:nil,next=state.at<text.count ? text[state.at]:nil,boundary=word(before) != word(next),kind=node[1] as! String
   let yes=kind=="start" ? state.at==0 || flags.contains("m")&&newline(before):kind=="end" ? state.at==text.count || newline(next)&&(flags.contains("m")||state.at+1==text.count||state.at+2==text.count&&next?.value==13&&text.last?.value==10):kind=="word" ? boundary:!boundary;if yes{result=[state]}
  case "class","dot":if state.at<text.count{let ch=text[state.at];let yes:Bool;if op=="dot"{yes=flags.contains("s") || !newline(ch)}else{let ranges=node[1] as! [[Int]];try tick(ranges.count);func contains(_ c:Unicode.Scalar)->Bool{ranges.contains{$0[0]<=Int(c.value)&&Int(c.value)<=$0[1]}};let found=contains(ch);yes=found != (node[2] as! Bool)};if yes{var next=state;next.at+=1;result=[next]}}
  default:throw EngineError(message:"Unsupported transform operation")
  };try tick(result.count);return result
 }
 mutating func format(_ ops:[Any],_ captures:[Range<Int>?],_ depth:Int)throws->String{guard depth<=32 else{throw EngineError(message:"Transform format nesting limit exceeded")};var output=""
  for raw in ops{try tick();let value:String;if let literal=raw as? String{value=literal}else{let op=raw as! [Any],index=op[1] as! Int,captured=index<captures.count ? captures[index].map{string($0.lowerBound,$0.upperBound)} ?? "":"";if op[0] as! String=="if"{value=try format(op[captured.isEmpty ? 3:2] as! [Any],captures,depth+1)}else{switch op[2] as! String {
   case "upcase":value=snippetCase(captured,"upper")
   case "downcase":value=snippetCase(captured,"lower")
   case "capitalize":value=captured.unicodeScalars.first.map{snippetCase(String($0),"upper")+captured.unicodeScalars.dropFirst().map(String.init).joined()} ?? ""
   case "camelcase","pascalcase":let words=snippetCaseWords(captured);let camel=op[2] as! String=="camelcase";value=words.isEmpty ? captured:words.enumerated().map{index,word in let scalar=word.unicodeScalars.first!;return (index==0&&camel ? snippetCase(String(scalar),"lower"):snippetCase(String(scalar),"upper"))+word.unicodeScalars.dropFirst().map(String.init).joined()}.joined()
   default:value=captured
  }}};guard value.utf8.count<=1048576-output.utf8.count else{throw EngineError(message:"Transform output limit exceeded")};output+=value};return output
 }
}
func applySnippetTransform(_ program:SnippetTransform,_ input:String,budget:inout SnippetTransformBudget)throws->String {
 if let v3=program.v3{return try applyRegexV3(v3,input,work:&budget.work)}
 guard input.utf8.count<=65536 else{throw EngineError(message:"Transform input limit exceeded")};var evaluator=TransformEvaluator(text:Array(input.unicodeScalars),flags:program.flags,work:budget.work),output="",cursor=0,search=0,matched=false;let count=evaluator.text.count
 while search<=count{var found:(Int,TransformMatch)?;for start in search...count{let initial=TransformMatch(at:start,captures:Array(repeating:nil,count:program.groups+1));if var state=try evaluator.matches(program.regex,initial,0).first{state.captures[0]=start..<state.at;found=(start,state);break}};guard let(start,state)=found else{break};matched=true;let prefix=evaluator.string(cursor,start),replacement=try evaluator.format(program.format,state.captures,0);guard prefix.utf8.count+replacement.utf8.count<=1048576-output.utf8.count else{throw EngineError(message:"Transform output limit exceeded")};output+=prefix+replacement;cursor=state.at;if !program.flags.contains("g"){break};search=state.at==start ? state.at+1:state.at}
 if !matched,program.format.contains(where:{raw in guard let op=raw as? [Any],op.first as? String=="if",let branch=op[3] as? [Any] else{return false};return !branch.isEmpty}){let result=try evaluator.format(program.format,Array(repeating:nil,count:program.groups+1),0);budget.work=evaluator.work;return result}
 let suffix=evaluator.string(cursor,count);guard suffix.utf8.count<=1048576-output.utf8.count else{throw EngineError(message:"Transform output limit exceeded")};budget.work=evaluator.work;return output+suffix
}
</ui:copy>
<ui:file path="apps/apple-simple/Sources/SnippetTransform.swift"><ui:paste select="#snippet-transform-apple"/></ui:file>

## Evaluate the same program in Android source models

Android copies admitted programs into immutable lists and evaluates Unicode scalar arrays with the same work/output limits. Capture and replacement policy belongs to this pure model, independently of Compose or a language provider.

<ui:copy id="snippet-transform-android">
@file:Suppress("UNCHECKED_CAST")
package com.hickorydocs.simple
import org.json.JSONArray
import org.json.JSONObject
import java.util.Locale
internal val supportedSnippetTransformPrograms=listOf(2,3)
@ConsistentCopyVisibility
data class SnippetTransform internal constructor(val regex:List<Any?>,val format:List<Any?>,val flags:String,val groups:Int,val bytes:Int,internal val v3:RegexV3Program?=null)
internal data class SnippetTransformBudget(var work:Int=0)
private fun scalarText(value:String):Boolean {var at=0;while(at<value.length){val c=value[at++];if(c.isHighSurrogate()){if(at==value.length||!value[at++].isLowSurrogate())return false}else if(c.isLowSurrogate())return false};return true}
internal fun readSnippetTransform(raw:Any?):SnippetTransform {
 if(raw is JSONObject&&(raw.opt("version") as? Number)?.toDouble()==3.0){val program=readRegexV3(raw);return SnippetTransform(emptyList(),program.format,program.flags,program.groups,program.bytes,program)}
 var count=0;var bytes=0
 fun tick(n:Int=1){count+=n;check(count<=512){"Transform program limit exceeded"}}
 fun integer(raw:Any?,max:Int=1114111):Int {val value=(raw as? Number)?.toString()?.toLongOrNull();check(value!=null&&value in 0..max.toLong()){"Malformed transform integer"};return value.toInt()}
 fun text(raw:Any?):String {check(raw is String&&scalarText(raw)){"Malformed transform text"};bytes+=raw.toByteArray(Charsets.UTF_8).size;check(bytes<=16384){"Transform metadata limit exceeded"};return raw}
 fun bool(raw:Any?):Boolean {check(raw is Boolean){"Malformed transform Boolean"};return raw}
 check(raw is JSONObject&&integer(raw.opt("version"))==2&&raw.opt("case_map")==snippetCaseVersion){"Unsupported snippet transform version"};val groups=integer(raw.opt("groups"),32);val flags=text(raw.opt("flags"));check(flags.all{it in "gimsu"}&&flags.toSet().size==flags.length){"Unsupported transform flags"}
 fun node(raw:Any?,depth:Int):List<Any?> {tick();check(depth<=64&&raw is JSONArray&&raw.opt(0) is String){"Malformed transform node"};val v=raw as JSONArray;val op=v.getString(0)
  return when(op){
   "seq","alt"->{val children=v.optJSONArray(1);check(v.length()==2&&children!=null&&children.length()<=512){"Malformed transform branches"};listOf(op,(0 until children!!.length()).map{node(children.get(it),depth+1)})}
   "capture"->{check(v.length()==3){"Malformed transform capture"};listOf(op,integer(v.opt(1),groups),node(v.opt(2),depth+1))}
   "repeat"->{check(v.length()==5){"Malformed transform repetition"};val min=integer(v.opt(2),256);val max=if(v.isNull(3))null else integer(v.opt(3),256);check(max==null||max>=min){"Backwards transform repetition"};listOf(op,node(v.opt(1),depth+1),min,max,bool(v.opt(4)))}
   "class"->{val ranges=v.optJSONArray(1);check(v.length()==3&&ranges!=null){"Malformed transform class"};val copied=(0 until ranges!!.length()).map{tick();val range=ranges.optJSONArray(it);check(range!=null&&range.length()==2){"Malformed transform range"};val low=integer(range!!.opt(0));val high=integer(range.opt(1));check(low<=high){"Backwards transform range"};listOf(low,high)};listOf(op,copied,bool(v.opt(2)))}
   "dot"->{check(v.length()==1){"Malformed transform dot"};listOf(op)}
   "assert"->{check(v.length()==2&&v.opt(1) in listOf("start","end","word","nonword")){"Malformed transform assertion"};listOf(op,v.getString(1))}
   else->error("Unsupported transform operation")
  }
 }
 fun format(raw:Any?,depth:Int):List<Any?> {check(depth<=32&&raw is JSONArray&&raw.length()<=512){"Malformed transform format"};return (0 until raw.length()).map{index->tick();val value=raw.get(index);if(value is String)text(value) else {check(value is JSONArray){"Malformed transform format"};when{value.opt(0)=="capture"&&value.length()==3&&value.opt(2) in listOf("","upcase","downcase","capitalize","camelcase","pascalcase")->listOf("capture",integer(value.opt(1),32),value.getString(2));value.opt(0)=="if"&&value.length()==4->listOf("if",integer(value.opt(1),32),format(value.opt(2),depth+1),format(value.opt(3),depth+1));else->error("Unsupported transform format")}}}}
 val regex=node(raw.opt("regex"),0);val format=format(raw.opt("format"),0);val normalized=JSONObject(mapOf("version" to 2,"case_map" to snippetCaseVersion,"regex" to regex,"format" to format,"flags" to flags,"groups" to groups));return SnippetTransform(regex,format,flags,groups,normalized.toString().toByteArray(Charsets.UTF_8).size)
}
private data class TransformCapture(val from:Int,val to:Int)
private data class TransformMatch(val at:Int,val captures:List<TransformCapture?>)
private class TransformEvaluator(val text:IntArray,val flags:String,var work:Int) {
 fun tick(n:Int=1){check(n>=0&&work>=0&&work<=100000-n){"Transform work limit exceeded"};work+=n}
 fun newline(c:Int?)=c in listOf(10,13,0x2028,0x2029)
 fun word(c:Int?)=c!=null&&(c in 48..57||c in 65..90||c==95||c in 97..122||'i' in flags&&'u' in flags&&(c==383||c==8490))
 fun string(from:Int,to:Int)=buildString{for(i in from until to)appendCodePoint(text[i])}
 fun matches(node:List<Any?>,state:TransformMatch,depth:Int):List<TransformMatch>{tick();check(depth<=64){"Transform evaluation nesting limit exceeded"};val op=node[0] as String
  val result:List<TransformMatch> = when(op){
   "seq"->{var states=listOf(state);for(child in node[1] as List<List<Any?>>){val next=mutableListOf<TransformMatch>();for(value in states){next+=matches(child,value,depth+1);check(next.size<=1024){"Transform state limit exceeded"}};states=next;if(states.isEmpty())break};states}
   "alt"->{val states=mutableListOf<TransformMatch>();for(child in node[1] as List<List<Any?>>){states+=matches(child,state,depth+1);check(states.size<=1024){"Transform state limit exceeded"}};states}
   "capture"->matches(node[2] as List<Any?>,state,depth+1).map{value->val captures=value.captures.toMutableList();captures[node[1] as Int]=TransformCapture(state.at,value.at);value.copy(captures=captures.toList())}
   "repeat"->{val min=node[2] as Int;val max=minOf(node[3] as? Int ?: text.size+1,4096);val lazy=node[4] as Boolean;val levels=mutableListOf(listOf(state));var total=1;for(i in 0 until max){val next=mutableListOf<TransformMatch>();for(value in levels.last()){for(found in matches(node[1] as List<Any?>,value,depth+1)){if(found.at!=value.at||levels.size<=min)next+=found};check(next.size<=1024){"Transform state limit exceeded"}};if(next.isEmpty())break;levels+=next.toList();total+=next.size;check(total<=1024){"Transform state limit exceeded"}};val indexes=(min until levels.size).toList();(if(lazy)indexes else indexes.reversed()).flatMap{levels[it]}}
   "assert"->{val before=text.getOrNull(state.at-1);val next=text.getOrNull(state.at);val boundary=word(before)!=word(next);val yes=when(node[1]){"start"->state.at==0||'m' in flags&&newline(before);"end"->state.at==text.size||newline(next)&&('m' in flags||state.at+1==text.size||state.at+2==text.size&&next==13&&text.lastOrNull()==10);"word"->boundary;else->!boundary};if(yes)listOf(state) else emptyList()}
   "class","dot"->{val ch=text.getOrNull(state.at);if(ch==null)emptyList() else {val yes=if(op=="dot")'s' in flags||!newline(ch) else {val ranges=node[1] as List<List<Int>>;tick(ranges.size)
    fun contains(c:Int)=ranges.any{c in it[0]..it[1]}
    val found=contains(ch);found!=(node[2] as Boolean)};if(yes)listOf(state.copy(at=state.at+1)) else emptyList()}}
   else->error("Unsupported transform operation")
  };tick(result.size);return result
 }
 fun format(ops:List<Any?>,captures:List<TransformCapture?>,depth:Int):String{check(depth<=32){"Transform format nesting limit exceeded"};var output="";for(raw in ops){tick();val value=if(raw is String)raw else {val op=raw as List<Any?>;val range=captures.getOrNull(op[1] as Int);val captured=range?.let{string(it.from,it.to)}?:"";if(op[0]=="if")format(op[if(captured.isEmpty())3 else 2] as List<Any?>,captures,depth+1) else when(op[2]){
   "upcase"->snippetCase(captured,"upper")
   "downcase"->snippetCase(captured,"lower")
   "capitalize"->if(captured.isEmpty())"" else {val first=captured.codePointAt(0);snippetCase(String(Character.toChars(first)),"upper")+captured.substring(Character.charCount(first))}
   "camelcase","pascalcase"->{val words=snippetCaseWords(captured);if(words.isEmpty())captured else words.mapIndexed{index,word->val c=word.codePointAt(0);val first=String(Character.toChars(c));(if(index==0&&op[2]=="camelcase")snippetCase(first,"lower") else snippetCase(first,"upper"))+word.substring(Character.charCount(c))}.joinToString("")}
   else->captured
  }};check(value.toByteArray(Charsets.UTF_8).size<=1048576-output.toByteArray(Charsets.UTF_8).size){"Transform output limit exceeded"};output+=value};return output}
}
internal fun applySnippetTransform(program:SnippetTransform,input:String,budget:SnippetTransformBudget=SnippetTransformBudget()):String {
 program.v3?.let{val work=intArrayOf(budget.work);try{return applyRegexV3(it,input,work)}finally{budget.work=work[0]}}
 check(scalarText(input)&&input.toByteArray(Charsets.UTF_8).size<=65536){"Transform input limit exceeded"};val evaluator=TransformEvaluator(input.codePoints().toArray(),program.flags,budget.work);var output="";var cursor=0;var search=0;var matched=false;val count=evaluator.text.size
 while(search<=count){var found:Pair<Int,TransformMatch>?=null;for(start in search..count){val initial=TransformMatch(start,List(program.groups+1){null});val state=evaluator.matches(program.regex,initial,0).firstOrNull();if(state!=null){val captures=state.captures.toMutableList();captures[0]=TransformCapture(start,state.at);found=start to state.copy(captures=captures.toList());break}};val selected=found?:break;matched=true;val (start,state)=selected;val prefix=evaluator.string(cursor,start);val replacement=evaluator.format(program.format,state.captures,0);check((output+prefix+replacement).toByteArray(Charsets.UTF_8).size<=1048576){"Transform output limit exceeded"};output+=prefix+replacement;cursor=state.at;if('g' !in program.flags)break;search=if(state.at==start)state.at+1 else state.at}
 if(!matched&&program.format.any{it is List<*>&&it[0]=="if"&&(it[3] as List<*>).isNotEmpty()}){val value=evaluator.format(program.format,List(program.groups+1){null},0);budget.work=evaluator.work;return value};val suffix=evaluator.string(cursor,count);check((output+suffix).toByteArray(Charsets.UTF_8).size<=1048576){"Transform output limit exceeded"};budget.work=evaluator.work;return output+suffix
}
</ui:copy>
<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/SnippetTransform.kt"><ui:paste select="#snippet-transform-android"/></ui:file>

## Run the same independently expected vectors on every host

The shared cases retain source patterns, compiled programs and expected strings. Rust checks both compilation and execution; hosts consume the exact same compiled program. Captures, zero-width global matching, scalar input, conditional fallbacks and budget exhaustion are compared directly. Host-only admission tests refuse malformed descriptors before they can enter history. These pure witnesses do not establish an editor path; the expansion/session and actual-host acceptance below provide that separate evidence.

<ui:copy id="snippet-transform-vectors">
[
  {
    "name": "capture",
    "pattern": "([a-z]+)[_-]([a-z]+)",
    "format": "${1:/capitalize}${2:/upcase}",
    "flags": "g",
    "input": "foo_bar and zip-zap",
    "expected": "FooBAR and ZipZAP",
    "program": {
      "version": 2,
      "case_map": "hickory-ucd-16.0.0-v1",
      "regex": [
        "seq",
        [
          [
            "capture",
            1,
            [
              "seq",
              [
                [
                  "repeat",
                  [
                    "class",
                    [
                      [
                        97,
                        122
                      ]
                    ],
                    false
                  ],
                  1,
                  null,
                  false
                ]
              ]
            ]
          ],
          [
            "class",
            [
              [
                95,
                95
              ],
              [
                45,
                45
              ]
            ],
            false
          ],
          [
            "capture",
            2,
            [
              "seq",
              [
                [
                  "repeat",
                  [
                    "class",
                    [
                      [
                        97,
                        122
                      ]
                    ],
                    false
                  ],
                  1,
                  null,
                  false
                ]
              ]
            ]
          ]
        ]
      ],
      "format": [
        [
          "capture",
          1,
          "capitalize"
        ],
        [
          "capture",
          2,
          "upcase"
        ]
      ],
      "flags": "g",
      "groups": 2
    }
  },
  {
    "name": "lazy",
    "pattern": "(a+?)(a)",
    "format": "<$1:$2>",
    "flags": "",
    "input": "aaaa",
    "expected": "<a:a>aa",
    "program": {
      "version": 2,
      "case_map": "hickory-ucd-16.0.0-v1",
      "regex": [
        "seq",
        [
          [
            "capture",
            1,
            [
              "seq",
              [
                [
                  "repeat",
                  [
                    "class",
                    [
                      [
                        97,
                        97
                      ]
                    ],
                    false
                  ],
                  1,
                  null,
                  true
                ]
              ]
            ]
          ],
          [
            "capture",
            2,
            [
              "seq",
              [
                [
                  "class",
                  [
                    [
                      97,
                      97
                    ]
                  ],
                  false
                ]
              ]
            ]
          ]
        ]
      ],
      "format": [
        "<",
        [
          "capture",
          1,
          ""
        ],
        ":",
        [
          "capture",
          2,
          ""
        ],
        ">"
      ],
      "flags": "",
      "groups": 2
    }
  },
  {
    "name": "conditions",
    "pattern": "(foo)?bar",
    "format": "${1:+yes}${1:?Y:N}${1:-none}",
    "flags": "g",
    "input": "foobar bar",
    "expected": "yesYfoo Nnone",
    "program": {
      "version": 2,
      "case_map": "hickory-ucd-16.0.0-v1",
      "regex": [
        "seq",
        [
          [
            "repeat",
            [
              "capture",
              1,
              [
                "seq",
                [
                  [
                    "class",
                    [
                      [
                        102,
                        102
                      ]
                    ],
                    false
                  ],
                  [
                    "class",
                    [
                      [
                        111,
                        111
                      ]
                    ],
                    false
                  ],
                  [
                    "class",
                    [
                      [
                        111,
                        111
                      ]
                    ],
                    false
                  ]
                ]
              ]
            ],
            0,
            1,
            false
          ],
          [
            "class",
            [
              [
                98,
                98
              ]
            ],
            false
          ],
          [
            "class",
            [
              [
                97,
                97
              ]
            ],
            false
          ],
          [
            "class",
            [
              [
                114,
                114
              ]
            ],
            false
          ]
        ]
      ],
      "format": [
        [
          "if",
          1,
          [
            "yes"
          ],
          []
        ],
        [
          "if",
          1,
          [
            "Y"
          ],
          [
            "N"
          ]
        ],
        [
          "if",
          1,
          [
            [
              "capture",
              1,
              ""
            ]
          ],
          [
            "none"
          ]
        ]
      ],
      "flags": "g",
      "groups": 1
    }
  },
  {
    "name": "zero",
    "pattern": "^|$",
    "format": "|",
    "flags": "g",
    "input": "😀λ",
    "expected": "|😀λ|",
    "program": {
      "version": 2,
      "case_map": "hickory-ucd-16.0.0-v1",
      "regex": [
        "alt",
        [
          [
            "seq",
            [
              [
                "assert",
                "start"
              ]
            ]
          ],
          [
            "seq",
            [
              [
                "assert",
                "end"
              ]
            ]
          ]
        ]
      ],
      "format": [
        "|"
      ],
      "flags": "g",
      "groups": 0
    }
  },
  {
    "name": "ignorecase",
    "pattern": "([a-z]+)",
    "format": "${1:/upcase}",
    "flags": "gi",
    "input": "aBc",
    "expected": "ABC",
    "program": {
      "version": 2,
      "case_map": "hickory-ucd-16.0.0-v1",
      "regex": [
        "seq",
        [
          [
            "capture",
            1,
            [
              "seq",
              [
                [
                  "repeat",
                  [
                    "class",
                    [
                      [
                        65,
                        90
                      ],
                      [
                        97,
                        122
                      ]
                    ],
                    false
                  ],
                  1,
                  null,
                  false
                ]
              ]
            ]
          ]
        ]
      ],
      "format": [
        [
          "capture",
          1,
          "upcase"
        ]
      ],
      "flags": "gi",
      "groups": 1
    }
  },
  {
    "name": "empty-else",
    "pattern": "(.+)",
    "format": "${1:?yes:no}",
    "flags": "",
    "input": "",
    "expected": "no",
    "program": {
      "version": 2,
      "case_map": "hickory-ucd-16.0.0-v1",
      "regex": [
        "seq",
        [
          [
            "capture",
            1,
            [
              "seq",
              [
                [
                  "repeat",
                  [
                    "dot"
                  ],
                  1,
                  null,
                  false
                ]
              ]
            ]
          ]
        ]
      ],
      "format": [
        [
          "if",
          1,
          [
            "yes"
          ],
          [
            "no"
          ]
        ]
      ],
      "flags": "",
      "groups": 1
    }
  },
  {
    "name": "nested-format",
    "pattern": "(a)",
    "format": "${1:+x${1}}",
    "flags": "",
    "input": "a",
    "expected": "xa",
    "program": {
      "version": 2,
      "case_map": "hickory-ucd-16.0.0-v1",
      "regex": [
        "seq",
        [
          [
            "capture",
            1,
            [
              "seq",
              [
                [
                  "class",
                  [
                    [
                      97,
                      97
                    ]
                  ],
                  false
                ]
              ]
            ]
          ]
        ]
      ],
      "format": [
        [
          "if",
          1,
          [
            "x",
            [
              "capture",
              1,
              ""
            ]
          ],
          []
        ]
      ],
      "flags": "",
      "groups": 1
    }
  },
  {
    "name": "camel",
    "pattern": "(.*)",
    "format": "${1:/camelcase}",
    "flags": "",
    "input": "HELLO world",
    "expected": "hELLOWorld",
    "program": {
      "version": 2,
      "case_map": "hickory-ucd-16.0.0-v1",
      "regex": [
        "seq",
        [
          [
            "capture",
            1,
            [
              "seq",
              [
                [
                  "repeat",
                  [
                    "dot"
                  ],
                  0,
                  null,
                  false
                ]
              ]
            ]
          ]
        ]
      ],
      "format": [
        [
          "capture",
          1,
          "camelcase"
        ]
      ],
      "flags": "",
      "groups": 1
    }
  },
  {
    "name": "pascal",
    "pattern": "(.*)",
    "format": "${1:/pascalcase}",
    "flags": "",
    "input": "hello_λworld",
    "expected": "HelloΛworld",
    "program": {
      "version": 2,
      "case_map": "hickory-ucd-16.0.0-v1",
      "regex": [
        "seq",
        [
          [
            "capture",
            1,
            [
              "seq",
              [
                [
                  "repeat",
                  [
                    "dot"
                  ],
                  0,
                  null,
                  false
                ]
              ]
            ]
          ]
        ]
      ],
      "format": [
        [
          "capture",
          1,
          "pascalcase"
        ]
      ],
      "flags": "",
      "groups": 1
    }
  },
  {
    "name": "emoji-case",
    "pattern": "(.*)",
    "format": "${1:/camelcase}",
    "flags": "",
    "input": "😀",
    "expected": "😀",
    "program": {
      "version": 2,
      "case_map": "hickory-ucd-16.0.0-v1",
      "regex": [
        "seq",
        [
          [
            "capture",
            1,
            [
              "seq",
              [
                [
                  "repeat",
                  [
                    "dot"
                  ],
                  0,
                  null,
                  false
                ]
              ]
            ]
          ]
        ]
      ],
      "format": [
        [
          "capture",
          1,
          "camelcase"
        ]
      ],
      "flags": "",
      "groups": 1
    }
  },
  {
    "name": "unicode-capture",
    "pattern": "(.)",
    "format": "<$1>",
    "flags": "g",
    "input": "😀λ",
    "expected": "<😀><λ>",
    "program": {
      "version": 2,
      "case_map": "hickory-ucd-16.0.0-v1",
      "regex": [
        "seq",
        [
          [
            "capture",
            1,
            [
              "seq",
              [
                [
                  "dot"
                ]
              ]
            ]
          ]
        ]
      ],
      "format": [
        "<",
        [
          "capture",
          1,
          ""
        ],
        ">"
      ],
      "flags": "g",
      "groups": 1
    }
  },
  {
    "name": "optional",
    "pattern": "(x)?(y)",
    "format": "$1:$2",
    "flags": "",
    "input": "y",
    "expected": ":y",
    "program": {
      "version": 2,
      "case_map": "hickory-ucd-16.0.0-v1",
      "regex": [
        "seq",
        [
          [
            "repeat",
            [
              "capture",
              1,
              [
                "seq",
                [
                  [
                    "class",
                    [
                      [
                        120,
                        120
                      ]
                    ],
                    false
                  ]
                ]
              ]
            ],
            0,
            1,
            false
          ],
          [
            "capture",
            2,
            [
              "seq",
              [
                [
                  "class",
                  [
                    [
                      121,
                      121
                    ]
                  ],
                  false
                ]
              ]
            ]
          ]
        ]
      ],
      "format": [
        [
          "capture",
          1,
          ""
        ],
        ":",
        [
          "capture",
          2,
          ""
        ]
      ],
      "flags": "",
      "groups": 2
    }
  },
  {
    "name": "range",
    "pattern": "[^a-c]+",
    "format": "X",
    "flags": "g",
    "input": "ab😀λcZ",
    "expected": "abXcX",
    "program": {
      "version": 2,
      "case_map": "hickory-ucd-16.0.0-v1",
      "regex": [
        "seq",
        [
          [
            "repeat",
            [
              "class",
              [
                [
                  97,
                  99
                ]
              ],
              true
            ],
            1,
            null,
            false
          ]
        ]
      ],
      "format": [
        "X"
      ],
      "flags": "g",
      "groups": 0
    }
  },
  {
    "name": "digits",
    "pattern": "(\\d+)",
    "format": "<$1>",
    "flags": "g",
    "input": "a12b3",
    "expected": "a<12>b<3>",
    "program": {
      "version": 2,
      "case_map": "hickory-ucd-16.0.0-v1",
      "regex": [
        "seq",
        [
          [
            "capture",
            1,
            [
              "seq",
              [
                [
                  "repeat",
                  [
                    "class",
                    [
                      [
                        48,
                        57
                      ]
                    ],
                    false
                  ],
                  1,
                  null,
                  false
                ]
              ]
            ]
          ]
        ]
      ],
      "format": [
        "<",
        [
          "capture",
          1,
          ""
        ],
        ">"
      ],
      "flags": "g",
      "groups": 1
    }
  },
  {
    "name": "word",
    "pattern": "\\b([a-z]+)\\b",
    "format": "${1:/upcase}",
    "flags": "g",
    "input": "a-b_2 😀x",
    "expected": "A-b_2 😀X",
    "program": {
      "version": 2,
      "case_map": "hickory-ucd-16.0.0-v1",
      "regex": [
        "seq",
        [
          [
            "assert",
            "word"
          ],
          [
            "capture",
            1,
            [
              "seq",
              [
                [
                  "repeat",
                  [
                    "class",
                    [
                      [
                        97,
                        122
                      ]
                    ],
                    false
                  ],
                  1,
                  null,
                  false
                ]
              ]
            ]
          ],
          [
            "assert",
            "word"
          ]
        ]
      ],
      "format": [
        [
          "capture",
          1,
          "upcase"
        ]
      ],
      "flags": "g",
      "groups": 1
    }
  },
  {
    "name": "multiline",
    "pattern": "^([a-z]+)$",
    "format": "<$1>",
    "flags": "gm",
    "input": "a\nλ\nb",
    "expected": "<a>\nλ\n<b>",
    "program": {
      "version": 2,
      "case_map": "hickory-ucd-16.0.0-v1",
      "regex": [
        "seq",
        [
          [
            "assert",
            "start"
          ],
          [
            "capture",
            1,
            [
              "seq",
              [
                [
                  "repeat",
                  [
                    "class",
                    [
                      [
                        97,
                        122
                      ]
                    ],
                    false
                  ],
                  1,
                  null,
                  false
                ]
              ]
            ]
          ],
          [
            "assert",
            "end"
          ]
        ]
      ],
      "format": [
        "<",
        [
          "capture",
          1,
          ""
        ],
        ">"
      ],
      "flags": "gm",
      "groups": 1
    }
  },
  {
    "name": "dotall",
    "pattern": "a(.*)b",
    "format": "$1",
    "flags": "s",
    "input": "a\n😀b",
    "expected": "\n😀",
    "program": {
      "version": 2,
      "case_map": "hickory-ucd-16.0.0-v1",
      "regex": [
        "seq",
        [
          [
            "class",
            [
              [
                97,
                97
              ]
            ],
            false
          ],
          [
            "capture",
            1,
            [
              "seq",
              [
                [
                  "repeat",
                  [
                    "dot"
                  ],
                  0,
                  null,
                  false
                ]
              ]
            ]
          ],
          [
            "class",
            [
              [
                98,
                98
              ]
            ],
            false
          ]
        ]
      ],
      "format": [
        [
          "capture",
          1,
          ""
        ]
      ],
      "flags": "s",
      "groups": 1
    }
  },
  {
    "name": "repetition",
    "pattern": "(a{2,3})",
    "format": "<$1>",
    "flags": "g",
    "input": "aaaaa",
    "expected": "<aaa><aa>",
    "program": {
      "version": 2,
      "case_map": "hickory-ucd-16.0.0-v1",
      "regex": [
        "seq",
        [
          [
            "capture",
            1,
            [
              "seq",
              [
                [
                  "repeat",
                  [
                    "class",
                    [
                      [
                        97,
                        97
                      ]
                    ],
                    false
                  ],
                  2,
                  3,
                  false
                ]
              ]
            ]
          ]
        ]
      ],
      "format": [
        "<",
        [
          "capture",
          1,
          ""
        ],
        ">"
      ],
      "flags": "g",
      "groups": 1
    }
  },
  {
    "name": "escaped",
    "pattern": "([a-z]+)",
    "format": "\\$1-$1-\\/",
    "flags": "g",
    "input": "a",
    "expected": "$1-a-/",
    "program": {
      "version": 2,
      "case_map": "hickory-ucd-16.0.0-v1",
      "regex": [
        "seq",
        [
          [
            "capture",
            1,
            [
              "seq",
              [
                [
                  "repeat",
                  [
                    "class",
                    [
                      [
                        97,
                        122
                      ]
                    ],
                    false
                  ],
                  1,
                  null,
                  false
                ]
              ]
            ]
          ]
        ]
      ],
      "format": [
        "$1-",
        [
          "capture",
          1,
          ""
        ],
        "-/"
      ],
      "flags": "g",
      "groups": 1
    }
  },
  {
    "name": "no-match",
    "pattern": "z",
    "format": "replace",
    "flags": "",
    "input": "abc",
    "expected": "abc",
    "program": {
      "version": 2,
      "case_map": "hickory-ucd-16.0.0-v1",
      "regex": [
        "seq",
        [
          [
            "class",
            [
              [
                122,
                122
              ]
            ],
            false
          ]
        ]
      ],
      "format": [
        "replace"
      ],
      "flags": "",
      "groups": 0
    }
  },
  {
    "name": "fallback-unmatched",
    "pattern": "(z)",
    "format": "${1:-fallback}",
    "flags": "",
    "input": "abc",
    "expected": "fallback",
    "program": {
      "version": 2,
      "case_map": "hickory-ucd-16.0.0-v1",
      "regex": [
        "seq",
        [
          [
            "capture",
            1,
            [
              "seq",
              [
                [
                  "class",
                  [
                    [
                      122,
                      122
                    ]
                  ],
                  false
                ]
              ]
            ]
          ]
        ]
      ],
      "format": [
        [
          "if",
          1,
          [
            [
              "capture",
              1,
              ""
            ]
          ],
          [
            "fallback"
          ]
        ]
      ],
      "flags": "",
      "groups": 1
    }
  },
  {
    "name": "unicode-hex",
    "pattern": "\\u03bb",
    "format": "L",
    "flags": "",
    "input": "λ",
    "expected": "L",
    "program": {
      "version": 2,
      "case_map": "hickory-ucd-16.0.0-v1",
      "regex": [
        "seq",
        [
          [
            "class",
            [
              [
                955,
                955
              ]
            ],
            false
          ]
        ]
      ],
      "format": [
        "L"
      ],
      "flags": "",
      "groups": 0
    }
  },
  {
    "name": "empty-match",
    "pattern": "a*",
    "format": "X",
    "flags": "g",
    "input": "b",
    "expected": "XbX",
    "program": {
      "version": 2,
      "case_map": "hickory-ucd-16.0.0-v1",
      "regex": [
        "seq",
        [
          [
            "repeat",
            [
              "class",
              [
                [
                  97,
                  97
                ]
              ],
              false
            ],
            0,
            null,
            false
          ]
        ]
      ],
      "format": [
        "X"
      ],
      "flags": "g",
      "groups": 0
    }
  },
  {
    "name": "work-limit",
    "pattern": "(a|aa)*z",
    "format": "x",
    "flags": "",
    "input": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    "reject": true,
    "program": {
      "version": 2,
      "case_map": "hickory-ucd-16.0.0-v1",
      "regex": [
        "seq",
        [
          [
            "repeat",
            [
              "capture",
              1,
              [
                "alt",
                [
                  [
                    "seq",
                    [
                      [
                        "class",
                        [
                          [
                            97,
                            97
                          ]
                        ],
                        false
                      ]
                    ]
                  ],
                  [
                    "seq",
                    [
                      [
                        "class",
                        [
                          [
                            97,
                            97
                          ]
                        ],
                        false
                      ],
                      [
                        "class",
                        [
                          [
                            97,
                            97
                          ]
                        ],
                        false
                      ]
                    ]
                  ]
                ]
              ]
            ],
            0,
            null,
            false
          ],
          [
            "class",
            [
              [
                122,
                122
              ]
            ],
            false
          ]
        ]
      ],
      "format": [
        "x"
      ],
      "flags": "",
      "groups": 1
    }
  },
  {
    "name": "unicode fold includes Kelvin and long s",
    "pattern": "([ks]+)",
    "format": "${1:/upcase}",
    "flags": "giu",
    "input": "Kſ",
    "expected": "KS",
    "program": {
      "version": 2,
      "case_map": "hickory-ucd-16.0.0-v1",
      "regex": [
        "seq",
        [
          [
            "capture",
            1,
            [
              "seq",
              [
                [
                  "repeat",
                  [
                    "class",
                    [
                      [
                        75,
                        75
                      ],
                      [
                        83,
                        83
                      ],
                      [
                        107,
                        107
                      ],
                      [
                        115,
                        115
                      ],
                      [
                        383,
                        383
                      ],
                      [
                        8490,
                        8490
                      ]
                    ],
                    false
                  ],
                  1,
                  null,
                  false
                ]
              ]
            ]
          ]
        ]
      ],
      "format": [
        [
          "capture",
          1,
          "upcase"
        ]
      ],
      "flags": "giu",
      "groups": 1
    }
  },
  {
    "name": "legacy fold excludes Kelvin and long s",
    "pattern": "[ks]",
    "format": "x",
    "flags": "gi",
    "input": "Kſ",
    "expected": "Kſ",
    "program": {
      "version": 2,
      "case_map": "hickory-ucd-16.0.0-v1",
      "regex": [
        "seq",
        [
          [
            "class",
            [
              [
                75,
                75
              ],
              [
                83,
                83
              ],
              [
                107,
                107
              ],
              [
                115,
                115
              ]
            ],
            false
          ]
        ]
      ],
      "format": [
        "x"
      ],
      "flags": "gi",
      "groups": 0
    }
  },
  {
    "name": "insensitive sharp s never truncates to S",
    "pattern": "[s]",
    "format": "x",
    "flags": "giu",
    "input": "ßS",
    "expected": "ßx",
    "program": {
      "version": 2,
      "case_map": "hickory-ucd-16.0.0-v1",
      "regex": [
        "seq",
        [
          [
            "class",
            [
              [
                83,
                83
              ],
              [
                115,
                115
              ],
              [
                383,
                383
              ]
            ],
            false
          ]
        ]
      ],
      "format": [
        "x"
      ],
      "flags": "giu",
      "groups": 0
    }
  },
  {
    "name": "unicode word assertion sees simple fold",
    "pattern": "\\b[ks]+\\b",
    "format": "x",
    "flags": "giu",
    "input": "Kſ",
    "expected": "x",
    "program": {
      "version": 2,
      "case_map": "hickory-ucd-16.0.0-v1",
      "regex": [
        "seq",
        [
          [
            "assert",
            "word"
          ],
          [
            "repeat",
            [
              "class",
              [
                [
                  75,
                  75
                ],
                [
                  83,
                  83
                ],
                [
                  107,
                  107
                ],
                [
                  115,
                  115
                ],
                [
                  383,
                  383
                ],
                [
                  8490,
                  8490
                ]
              ],
              false
            ],
            1,
            null,
            false
          ],
          [
            "assert",
            "word"
          ]
        ]
      ],
      "format": [
        "x"
      ],
      "flags": "giu",
      "groups": 0
    }
  },
  {
    "name": "legacy word assertion excludes simple fold",
    "pattern": "\\b[ks]+\\b",
    "format": "x",
    "flags": "gi",
    "input": "Kſ",
    "expected": "Kſ",
    "program": {
      "version": 2,
      "case_map": "hickory-ucd-16.0.0-v1",
      "regex": [
        "seq",
        [
          [
            "assert",
            "word"
          ],
          [
            "repeat",
            [
              "class",
              [
                [
                  75,
                  75
                ],
                [
                  83,
                  83
                ],
                [
                  107,
                  107
                ],
                [
                  115,
                  115
                ]
              ],
              false
            ],
            1,
            null,
            false
          ],
          [
            "assert",
            "word"
          ]
        ]
      ],
      "format": [
        "x"
      ],
      "flags": "gi",
      "groups": 0
    }
  },
  {
    "name": "default lower final sigma uses original context",
    "pattern": "(.+)",
    "format": "${1:/downcase}",
    "flags": "",
    "input": "ΟΣ́",
    "expected": "ος́",
    "program": {
      "version": 2,
      "case_map": "hickory-ucd-16.0.0-v1",
      "regex": [
        "seq",
        [
          [
            "capture",
            1,
            [
              "seq",
              [
                [
                  "repeat",
                  [
                    "dot"
                  ],
                  1,
                  null,
                  false
                ]
              ]
            ]
          ]
        ]
      ],
      "format": [
        [
          "capture",
          1,
          "downcase"
        ]
      ],
      "flags": "",
      "groups": 1
    }
  },
  {
    "name": "default lower dotted I expands",
    "pattern": "(.+)",
    "format": "${1:/downcase}",
    "flags": "",
    "input": "İ",
    "expected": "i̇",
    "program": {
      "version": 2,
      "case_map": "hickory-ucd-16.0.0-v1",
      "regex": [
        "seq",
        [
          [
            "capture",
            1,
            [
              "seq",
              [
                [
                  "repeat",
                  [
                    "dot"
                  ],
                  1,
                  null,
                  false
                ]
              ]
            ]
          ]
        ]
      ],
      "format": [
        [
          "capture",
          1,
          "downcase"
        ]
      ],
      "flags": "",
      "groups": 1
    }
  },
  {
    "name": "frozen casing expands ligatures",
    "pattern": "(.+)",
    "format": "${1:/upcase}",
    "flags": "",
    "input": "ßﬃ",
    "expected": "SSFFI",
    "program": {
      "version": 2,
      "case_map": "hickory-ucd-16.0.0-v1",
      "regex": [
        "seq",
        [
          [
            "capture",
            1,
            [
              "seq",
              [
                [
                  "repeat",
                  [
                    "dot"
                  ],
                  1,
                  null,
                  false
                ]
              ]
            ]
          ]
        ]
      ],
      "format": [
        [
          "capture",
          1,
          "upcase"
        ]
      ],
      "flags": "",
      "groups": 1
    }
  }
]
</ui:copy>
<ui:file path="apps/simple-contract/snippet-transform-cases.json"><ui:paste select="#snippet-transform-vectors"/></ui:file>

<ui:copy id="snippet-transform-browser-tests">
import {describe,it,expect} from 'vitest';
import cases from '../../../simple-contract/snippet-transform-cases.json';
import {readSnippetTransform,applySnippetTransform} from './SnippetTransform';
describe('shared bounded snippet transforms',()=>{
 for(const row of cases)it(row.name,()=>{const program=readSnippetTransform(row.program);if('reject' in row&&row.reject)expect(()=>applySnippetTransform(program,row.input)).toThrow();else expect(applySnippetTransform(program,row.input)).toBe('expected' in row?row.expected:undefined);});
 it('refuses malformed descriptors and shares an action work budget',()=>{for(const raw of [{...cases[0].program,version:1},{...cases[0].program,case_map:"other"},{...cases[0].program,groups:true},{...cases[0].program,flags:'gg'},{...cases[0].program,regex:['class',[[5,1]],false]},{...cases[0].program,regex:['capture',true,['dot']]}])expect(()=>readSnippetTransform(raw)).toThrow();const program=readSnippetTransform(cases[0].program);expect(()=>applySnippetTransform(program,'foo_bar',{work:99999})).toThrow();expect(Object.isFrozen(program.regex)).toBe(true);for(const input of ['\ud800','\udfff'])expect(()=>applySnippetTransform(program,input)).toThrow();expect(()=>readSnippetTransform({...cases[0].program,format:['\ud800']})).toThrow();});
});
</ui:copy>
<ui:file path="apps/web/src/simple/SnippetTransform.test.ts"><ui:paste select="#snippet-transform-browser-tests"/></ui:file>

<ui:copy id="snippet-transform-apple-tests">
import XCTest
@testable import HickorySimple
final class SnippetTransformTests:XCTestCase {
 func testSharedTransformVectors()throws {
  let root=URL(fileURLWithPath:#filePath).deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent(),cases=try XCTUnwrap(JSONSerialization.jsonObject(with:Data(contentsOf:root.appendingPathComponent("apps/simple-contract/snippet-transform-cases.json"))) as? [[String:Any]])
  for row in cases {let program=try readSnippetTransform(row["program"]!),input=try XCTUnwrap(row["input"] as? String);var budget=SnippetTransformBudget();if row["reject"] as? Bool==true {XCTAssertThrowsError(try applySnippetTransform(program,input,budget:&budget),row["name"] as? String ?? "case")}else{XCTAssertEqual(Array(try applySnippetTransform(program,input,budget:&budget).utf8),Array((row["expected"] as! String).utf8),row["name"] as? String ?? "case")}}
 }
 func testMalformedDescriptorsAndSharedActionWorkLimit()throws {
  let raw:[String:Any]=["version":2,"case_map":snippetCaseVersion,"regex":["dot"],"format":[["capture",0,""]],"flags":"","groups":0]
  for(key,value)in [("version",1),("case_map","other"),("groups",true),("flags","gg"),("regex",["class",[[5,1]],false]),("regex",["capture",true,["dot"]])] as [(String,Any)] {var changed=raw;changed[key]=value;XCTAssertThrowsError(try readSnippetTransform(changed))}
  let program=try readSnippetTransform(raw);var budget=SnippetTransformBudget(work:99999);XCTAssertThrowsError(try applySnippetTransform(program,"a",budget:&budget))
 }
}
</ui:copy>
<ui:file path="apps/apple-simple/Tests/SnippetTransformTests.swift"><ui:paste select="#snippet-transform-apple-tests"/></ui:file>

<ui:copy id="snippet-transform-android-tests">
package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import org.json.JSONArray
import org.json.JSONObject
class SnippetTransformTest {
 @Test fun sharedTransformVectors(){val root=generateSequence(java.io.File(System.getProperty("user.dir")).absoluteFile){it.parentFile}.first{it.resolve("apps/simple-contract/snippet-transform-cases.json").isFile};val rows=JSONArray(root.resolve("apps/simple-contract/snippet-transform-cases.json").readText());for(i in 0 until rows.length()){val row=rows.getJSONObject(i);val program=readSnippetTransform(row.getJSONObject("program"));if(row.optBoolean("reject"))assertTrue(row.getString("name"),runCatching{applySnippetTransform(program,row.getString("input"))}.isFailure)else assertEquals(row.getString("name"),row.getString("expected"),applySnippetTransform(program,row.getString("input")))}}
 @Test fun malformedDescriptorsAndSharedActionWorkLimit(){val raw=JSONObject("""{"version":2,"case_map":"hickory-ucd-16.0.0-v1","regex":["dot"],"format":[["capture",0,""]],"flags":"","groups":0}""");for((key,value)in listOf("version" to 1,"case_map" to "other","groups" to true,"flags" to "gg","regex" to JSONArray("""["class",[[5,1]],false]"""),"regex" to JSONArray("""["capture",true,["dot"]]""")))assertTrue(key,runCatching{readSnippetTransform(JSONObject(raw.toString()).put(key,value))}.isFailure);assertTrue(runCatching{applySnippetTransform(readSnippetTransform(raw),"a",SnippetTransformBudget(99999))}.isFailure);for(input in listOf("\uD800","\uDFFF"))assertTrue(runCatching{applySnippetTransform(readSnippetTransform(raw),input)}.isFailure);assertTrue(runCatching{readSnippetTransform(JSONObject(raw.toString()).put("format",JSONArray().put("\uD800")))}.isFailure)}
}
</ui:copy>
<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/SnippetTransformTest.kt"><ui:paste select="#snippet-transform-android-tests"/></ui:file>
</ui:doc>
