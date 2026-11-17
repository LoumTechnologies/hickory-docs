//! Bounded data-only snippet transforms shared with the pure host evaluators.
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
