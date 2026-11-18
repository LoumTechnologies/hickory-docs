use std::collections::{BTreeMap, BTreeSet};

const MAX_BYTES: usize = 262_144;
const MAX_OUTPUT: usize = 1_048_576;
const MAX_FIELDS: usize = 4096;
const MAX_CHOICES: usize = 4096;
const CHOICE_OVERHEAD: usize = 32;
const MAX_DEPTH: usize = 32;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnippetField {
    pub id: usize,
    pub parent: Option<usize>,
    pub index: u32,
    pub from: usize,
    pub to: usize,
    pub choices: Vec<String>,
    pub transform: Option<serde_json::Value>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnippetPlan {
    pub text: String,
    pub fields: Vec<SnippetField>,
    pub order: Vec<u32>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
enum Node {
    Text(String),
    Field(u32, Option<Vec<Node>>, Vec<String>),
    Variable(String, Option<Vec<Node>>),
    FieldTransform(u32, serde_json::Value),
    VariableTransform(String, serde_json::Value),
}
struct Parser<'a> {
    text: &'a str,
    transforms: bool,
    at: usize,
    nodes: usize,
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
    fn fail<T>(&self, message: &str) -> Result<T, String> {
        Err(format!("{message} at snippet byte {}", self.at))
    }
    fn sequence(&mut self, nested: bool, depth: usize) -> Result<Vec<Node>, String> {
        if depth > MAX_DEPTH {
            return self.fail("Snippet nesting limit exceeded");
        }
        let mut result = vec![];
        let mut literal = String::new();
        loop {
            match self.peek() {
                None if nested => return self.fail("Unclosed snippet placeholder"),
                None => break,
                Some('}') if nested => {
                    self.take();
                    break;
                }
                Some('\\') => {
                    self.take();
                    match self.take() {
                        Some(ch @ ('$' | '}' | '\\')) => literal.push(ch),
                        _ => return self.fail("Unsupported snippet escape"),
                    }
                }
                Some('$') => {
                    self.take();
                    if !literal.is_empty() {
                        result.push(Node::Text(std::mem::take(&mut literal)));
                    }
                    self.nodes += 1;
                    if self.nodes > MAX_FIELDS {
                        return self.fail("Snippet field limit exceeded");
                    }
                    result.push(self.construct(depth + 1)?);
                }
                Some(ch) => {
                    self.take();
                    literal.push(ch)
                }
            }
        }
        if !literal.is_empty() {
            result.push(Node::Text(literal));
        }
        Ok(result)
    }
    // A format may contain nested ${capture:/case} slashes. Escapes remain
    // intact for the common transform compiler rather than being reinterpreted.
    fn transform_segment(&mut self, format: bool) -> Result<String, String> {
        let mut text = String::new();
        let mut braces = 0usize;
        while let Some(ch) = self.take() {
            match ch {
                '\\' => {
                    text.push(ch);
                    text.push(self.take().ok_or("Unclosed transform escape")?);
                }
                '/' if braces == 0 => return Ok(text),
                '$' if format && self.peek() == Some('{') => {
                    text.push(ch);
                    text.push(self.take().unwrap());
                    braces += 1;
                }
                '}' if format && braces > 0 => {
                    braces -= 1;
                    text.push(ch);
                }
                _ => text.push(ch),
            }
            if text.len() > 8192 {
                return self.fail("Transform syntax limit exceeded");
            }
        }
        self.fail("Unclosed snippet transform")
    }
    fn construct(&mut self, depth: usize) -> Result<Node, String> {
        let braced = self.peek() == Some('{');
        if braced {
            self.take();
        }
        let start = self.at;
        let numeric = self.peek().is_some_and(|ch| ch.is_ascii_digit());
        if numeric {
            while self.peek().is_some_and(|ch| ch.is_ascii_digit()) {
                self.take();
            }
        } else {
            if !self
                .peek()
                .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_')
            {
                return self.fail("Unsupported snippet construct");
            }
            self.take();
            while self
                .peek()
                .is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_')
            {
                self.take();
            }
        }
        let name = &self.text[start..self.at];
        let index = if numeric {
            let index = name
                .parse::<u32>()
                .map_err(|_| "Snippet field index overflow".to_string())?;
            if index > 1_000_000 {
                return self.fail("Snippet field index limit exceeded");
            }
            Some(index)
        } else {
            None
        };
        let mut default = None;
        let mut choices = vec![];
        if braced {
            match self.take() {
                Some('}') => {}
                Some(':') if index != Some(0) => {
                    default = Some(self.sequence(true, depth)?);
                }
                Some('|') if index.is_some_and(|index| index > 0) => {
                    let mut value = String::new();
                    loop {
                        match self.take() {
                            Some('\\') => match self.take() {
                                Some(ch @ (',' | '|' | '\\')) => value.push(ch),
                                _ => return self.fail("Unsupported snippet choice escape"),
                            },
                            Some(',') => {
                                if choices.len() >= MAX_CHOICES {
                                    return self.fail("Snippet choice limit exceeded");
                                }
                                choices.push(std::mem::take(&mut value));
                            }
                            Some('|') => {
                                if self.take() != Some('}') {
                                    return self.fail("Malformed snippet choice terminator");
                                }
                                if choices.len() >= MAX_CHOICES {
                                    return self.fail("Snippet choice limit exceeded");
                                }
                                choices.push(value);
                                break;
                            }
                            Some(ch) => value.push(ch),
                            None => return self.fail("Unclosed snippet choice"),
                        }
                    }
                }
                Some('/') => {
                    if !self.transforms {
                        return self.fail("Snippet transforms require the complete source model");
                    }
                    let pattern = self.transform_segment(false)?;
                    let format = self.transform_segment(true)?;
                    let start = self.at;
                    while self.peek().is_some_and(|c| c.is_ascii_alphabetic()) {
                        self.take();
                    }
                    let flags = &self.text[start..self.at];
                    if self.take() != Some('}') {
                        return self.fail("Malformed snippet transform terminator");
                    }
                    let program = crate::snippet_transform::compile(&pattern, &format, flags)?;
                    return Ok(match index {
                        Some(index) => Node::FieldTransform(index, program),
                        None => Node::VariableTransform(name.to_owned(), program),
                    });
                }
                _ => return self.fail("Malformed snippet placeholder"),
            }
        }
        Ok(match index {
            Some(index) => Node::Field(index, default, choices),
            None => Node::Variable(name.to_owned(), default),
        })
    }
}
#[derive(Clone)]
struct Definition {
    nodes: Vec<Node>,
    choices: Vec<String>,
}
fn definitions(
    nodes: &[Node],
    values: &mut BTreeMap<u32, Definition>,
    max: &mut u32,
) -> Result<(), String> {
    for node in nodes {
        match node {
            Node::Field(index, default, choices) => {
                *max = (*max).max(*index);
                if default.is_some() || !choices.is_empty() {
                    let nodes = default
                        .clone()
                        .unwrap_or_else(|| vec![Node::Text(choices[0].clone())]);
                    if let Some(previous) = values.get(index) {
                        if previous.nodes != nodes || previous.choices != *choices {
                            return Err("Conflicting snippet field defaults".into());
                        }
                    } else {
                        values.insert(
                            *index,
                            Definition {
                                nodes,
                                choices: choices.clone(),
                            },
                        );
                    }
                }
                if let Some(default) = default {
                    definitions(default, values, max)?;
                }
            }
            Node::Variable(_, Some(default)) => definitions(default, values, max)?,
            Node::FieldTransform(index, _) => *max = (*max).max(*index),
            _ => {}
        }
    }
    Ok(())
}
struct Renderer<'a> {
    ordinary: BTreeSet<u32>,
    value_active: BTreeSet<u32>,
    value_work: usize,
    transform_work: usize,
    definitions: BTreeMap<u32, Definition>,
    variables: &'a BTreeMap<String, Option<String>>,
    unknown: BTreeMap<String, u32>,
    next: u32,
    active: BTreeSet<u32>,
    text: String,
    offset: usize,
    fields: Vec<SnippetField>,
    parents: Vec<usize>,
    next_occurrence: usize,
    metadata_bytes: usize,
}
impl Renderer<'_> {
    fn text(&mut self, text: &str) -> Result<(), String> {
        if self.text.len() + text.len() > MAX_OUTPUT {
            return Err("Snippet expansion limit exceeded".into());
        }
        self.text.push_str(text);
        self.offset += text.encode_utf16().count();
        Ok(())
    }
    fn field(&mut self, index: u32, depth: usize) -> Result<(), String> {
        if !self.active.insert(index) {
            return Err("Cyclic snippet field default".into());
        }
        if self.next_occurrence >= MAX_FIELDS {
            return Err("Snippet expansion field limit exceeded".into());
        }
        let id = self.next_occurrence;
        self.next_occurrence += 1;
        let parent = self.parents.last().copied();
        self.parents.push(id);
        let from = self.offset;
        // Reserve each alternative's storage even when its text is empty,
        // before cloning a definition for this mirrored occurrence.
        let metadata = match self.definitions.get(&index) {
            Some(value) => value
                .choices
                .iter()
                .try_fold(64usize, |cost, choice| {
                    cost.checked_add(CHOICE_OVERHEAD)?.checked_add(choice.len())
                })
                .ok_or("Snippet metadata expansion limit exceeded")?,
            None => 64,
        };
        let total = self
            .metadata_bytes
            .checked_add(metadata)
            .filter(|bytes| *bytes <= MAX_OUTPUT)
            .ok_or("Snippet metadata expansion limit exceeded")?;
        self.metadata_bytes = total;
        let definition = self.definitions.get(&index).cloned();
        if let Some(definition) = &definition {
            self.render(&definition.nodes, depth + 1)?;
        }
        self.active.remove(&index);
        self.parents.pop();
        if self.fields.len() >= MAX_FIELDS {
            return Err("Snippet expansion field limit exceeded".into());
        }
        self.fields.push(SnippetField {
            id,
            parent,
            index,
            from,
            to: self.offset,
            choices: definition.map(|value| value.choices).unwrap_or_default(),
            transform: None,
        });
        Ok(())
    }
    fn value_tick(&mut self, count: usize) -> Result<(), String> {
        self.value_work = self
            .value_work
            .checked_add(count)
            .filter(|n| *n <= 100_000)
            .ok_or("Snippet transform value work limit exceeded")?;
        Ok(())
    }
    fn value_field(&mut self, index: u32, depth: usize) -> Result<String, String> {
        if depth > MAX_DEPTH || !self.value_active.insert(index) {
            return Err("Cyclic or excessive snippet transform source".into());
        }
        let definition = self.definitions.get(&index).cloned();
        let result = match definition {
            Some(definition) => self.value_nodes(&definition.nodes, depth + 1),
            None => Ok(String::new()),
        };
        self.value_active.remove(&index);
        result
    }
    fn value_nodes(&mut self, nodes: &[Node], depth: usize) -> Result<String, String> {
        if depth > MAX_DEPTH {
            return Err("Snippet transform source nesting limit exceeded".into());
        }
        let mut result = String::new();
        for node in nodes {
            self.value_tick(1)?;
            let value = match node {
                Node::Text(text) => text.clone(),
                Node::Field(index, _, _) => self.value_field(*index, depth + 1)?,
                Node::Variable(name, default) => match self.variables.get(name) {
                    Some(Some(value)) => value.clone(),
                    Some(None) => match default {
                        Some(default) => self.value_nodes(default, depth + 1)?,
                        None => String::new(),
                    },
                    None => match default {
                        Some(default) => self.value_nodes(default, depth + 1)?,
                        None => name.clone(),
                    },
                },
                Node::FieldTransform(index, program) => {
                    let value = self.value_field(*index, depth + 1)?;
                    crate::snippet_transform::apply_with_budget(
                        program,
                        &value,
                        &mut self.transform_work,
                    )?
                }
                Node::VariableTransform(name, program) => {
                    let value = self
                        .variables
                        .get(name)
                        .and_then(Option::as_deref)
                        .unwrap_or("");
                    crate::snippet_transform::apply_with_budget(
                        program,
                        value,
                        &mut self.transform_work,
                    )?
                }
            };
            self.value_tick(value.len())?;
            if value.len() > MAX_OUTPUT - result.len() {
                return Err("Snippet transform source limit exceeded".into());
            }
            result.push_str(&value)
        }
        Ok(result)
    }
    fn transformed_field(
        &mut self,
        index: u32,
        program: &serde_json::Value,
        depth: usize,
    ) -> Result<(), String> {
        let source = self.value_field(index, depth + 1)?;
        let value = crate::snippet_transform::apply_with_budget(
            program,
            &source,
            &mut self.transform_work,
        )?;
        let from = self.offset;
        let parent = self.parents.last().copied();
        let live = index > 0 && self.ordinary.contains(&index);
        let id = self.next_occurrence;
        if live {
            let bytes = 64
                + serde_json::to_vec(program)
                    .map_err(|_| "Malformed transform metadata")?
                    .len();
            if self.next_occurrence >= MAX_FIELDS || bytes > MAX_OUTPUT - self.metadata_bytes {
                return Err("Snippet transform metadata limit exceeded".into());
            }
            self.metadata_bytes += bytes;
            self.next_occurrence += 1
        }
        self.text(&value)?;
        if live {
            self.fields.push(SnippetField {
                id,
                parent,
                index,
                from,
                to: self.offset,
                choices: vec![],
                transform: Some(program.clone()),
            })
        }
        Ok(())
    }
    fn render(&mut self, nodes: &[Node], depth: usize) -> Result<(), String> {
        if depth > MAX_DEPTH {
            return Err("Snippet expansion nesting limit exceeded".into());
        }
        for node in nodes {
            match node {
                Node::Text(text) => self.text(text)?,
                Node::Field(index, _, _) => self.field(*index, depth)?,
                Node::FieldTransform(index, program) => {
                    self.transformed_field(*index, program, depth)?
                }
                Node::VariableTransform(name, program) => {
                    let value = self
                        .variables
                        .get(name)
                        .and_then(Option::as_deref)
                        .unwrap_or("");
                    let value = crate::snippet_transform::apply_with_budget(
                        program,
                        value,
                        &mut self.transform_work,
                    )?;
                    self.text(&value)?;
                }
                Node::Variable(name, default) => match self.variables.get(name) {
                    Some(Some(value)) => self.text(value)?,
                    Some(None) => {
                        if let Some(default) = default {
                            self.render(default, depth + 1)?;
                        }
                    }
                    None => {
                        if let Some(default) = default {
                            self.render(default, depth + 1)?;
                        } else {
                            let index = if let Some(index) = self.unknown.get(name) {
                                *index
                            } else {
                                self.next += 1;
                                let index = self.next;
                                self.unknown.insert(name.clone(), index);
                                self.definitions.insert(
                                    index,
                                    Definition {
                                        nodes: vec![Node::Text(name.clone())],
                                        choices: vec![],
                                    },
                                );
                                index
                            };
                            self.field(index, depth)?;
                        }
                    }
                },
            }
        }
        Ok(())
    }
}
/// Parse provider snippet text or a source-owned live template without editing a document.
/// Known-but-unset variables use `None`; unknown names become linked editable fields.
pub fn expand_snippet(
    text: &str,
    variables: &BTreeMap<String, Option<String>>,
) -> Result<SnippetPlan, String> {
    expand(text, variables, false)
}
/// Opt in only after the caller negotiates complete computed-field host behavior.
pub fn expand_snippet_v2(
    text: &str,
    variables: &BTreeMap<String, Option<String>>,
) -> Result<SnippetPlan, String> {
    expand(text, variables, true)
}
pub fn requested_variables(text: &str) -> Result<BTreeSet<String>, String> {
    if text.len() > MAX_BYTES {
        return Err("Snippet input limit exceeded".into());
    }
    let nodes = Parser {
        text,
        at: 0,
        nodes: 0,
        transforms: true,
    }
    .sequence(false, 0)?;
    fn visit(nodes: &[Node], result: &mut BTreeSet<String>) {
        for node in nodes {
            match node {
                Node::Variable(name, default) => {
                    result.insert(name.clone());
                    if let Some(default) = default {
                        visit(default, result)
                    }
                }
                Node::VariableTransform(name, _) => {
                    result.insert(name.clone());
                }
                Node::Field(_, Some(default), _) => visit(default, result),
                _ => {}
            }
        }
    }
    let mut result = BTreeSet::new();
    visit(&nodes, &mut result);
    Ok(result)
}
/// Read only variables that participate in the selected expansion branch.
/// A known value hides its default; numeric mirrors use the chosen definition.
pub fn effective_requested_variables(
    text: &str,
    variables: &BTreeMap<String, Option<String>>,
) -> Result<BTreeSet<String>, String> {
    if text.len() > MAX_BYTES {
        return Err("Snippet input limit exceeded".into());
    }
    let nodes = Parser {
        text,
        at: 0,
        nodes: 0,
        transforms: true,
    }
    .sequence(false, 0)?;
    let mut definitions_map = BTreeMap::new();
    let mut max = 0;
    definitions(&nodes, &mut definitions_map, &mut max)?;
    struct Demand<'a> {
        definitions: &'a BTreeMap<u32, Definition>,
        variables: &'a BTreeMap<String, Option<String>>,
        active: BTreeSet<u32>,
        names: BTreeSet<String>,
        work: usize,
    }
    impl Demand<'_> {
        fn field(&mut self, index: u32, depth: usize) -> Result<(), String> {
            if !self.active.insert(index) {
                return Err("Cyclic snippet variable demand".into());
            }
            if let Some(nodes) = self
                .definitions
                .get(&index)
                .map(|definition| definition.nodes.clone())
            {
                self.nodes(&nodes, depth + 1)?
            }
            self.active.remove(&index);
            Ok(())
        }
        fn nodes(&mut self, nodes: &[Node], depth: usize) -> Result<(), String> {
            if depth > MAX_DEPTH {
                return Err("Snippet variable demand nesting limit exceeded".into());
            }
            for node in nodes {
                self.work += 1;
                if self.work > 100_000 {
                    return Err("Snippet variable demand work limit exceeded".into());
                }
                match node {
                    Node::Variable(name, default) => {
                        self.names.insert(name.clone());
                        if !matches!(self.variables.get(name), Some(Some(_))) {
                            if let Some(default) = default {
                                self.nodes(default, depth + 1)?
                            }
                        }
                    }
                    Node::VariableTransform(name, _) => {
                        self.names.insert(name.clone());
                    }
                    Node::Field(index, _, _) | Node::FieldTransform(index, _) => {
                        self.field(*index, depth + 1)?
                    }
                    Node::Text(_) => {}
                }
            }
            Ok(())
        }
    }
    let mut demand = Demand {
        definitions: &definitions_map,
        variables,
        active: BTreeSet::new(),
        names: BTreeSet::new(),
        work: 0,
    };
    demand.nodes(&nodes, 0)?;
    Ok(demand.names)
}
impl SnippetPlan {
    pub fn metadata(&self, syntax: &str) -> serde_json::Value {
        serde_json::json!({"version":if self.fields.iter().any(|field|field.transform.is_some()){2}else{1},"syntax":syntax,"text":self.text,"fields":self.fields.iter().map(|field|serde_json::json!({"id":field.id,"parent":field.parent,"index":field.index,"from":field.from,"to":field.to,"choices":field.choices,"transform":field.transform})).collect::<Vec<_>>()})
    }
}
fn expand(
    text: &str,
    variables: &BTreeMap<String, Option<String>>,
    transforms: bool,
) -> Result<SnippetPlan, String> {
    if text.len() > MAX_BYTES {
        return Err("Snippet input limit exceeded".into());
    }
    let nodes = Parser {
        text,
        transforms,
        at: 0,
        nodes: 0,
    }
    .sequence(false, 0)?;
    let mut definitions_map = BTreeMap::new();
    let mut max = 0;
    definitions(&nodes, &mut definitions_map, &mut max)?;
    fn ordinary(nodes: &[Node], indices: &mut BTreeSet<u32>) {
        for node in nodes {
            match node {
                Node::Field(index, default, _) => {
                    indices.insert(*index);
                    if let Some(default) = default {
                        ordinary(default, indices)
                    }
                }
                Node::Variable(_, Some(default)) => ordinary(default, indices),
                _ => {}
            }
        }
    }
    let mut indices = BTreeSet::new();
    ordinary(&nodes, &mut indices);
    let mut renderer = Renderer {
        ordinary: indices,
        value_active: BTreeSet::new(),
        value_work: 0,
        transform_work: 0,
        definitions: definitions_map,
        variables,
        unknown: BTreeMap::new(),
        next: max,
        active: BTreeSet::new(),
        text: String::new(),
        offset: 0,
        fields: vec![],
        parents: vec![],
        next_occurrence: 0,
        metadata_bytes: 0,
    };
    renderer.render(&nodes, 0)?;
    // A known variable can hide a default containing the only editable source
    // occurrence. Keep its transformed text, but do not publish an orphan link.
    let rendered_sources = renderer
        .fields
        .iter()
        .filter(|field| field.transform.is_none())
        .map(|field| field.index)
        .collect::<BTreeSet<_>>();
    renderer
        .fields
        .retain(|field| field.transform.is_none() || rendered_sources.contains(&field.index));
    if !renderer.fields.iter().any(|field| field.index == 0) {
        if renderer.next_occurrence >= MAX_FIELDS || renderer.metadata_bytes + 64 > MAX_OUTPUT {
            return Err("Snippet expansion field limit exceeded".into());
        }
        renderer.fields.push(SnippetField {
            id: renderer.next_occurrence,
            parent: None,
            index: 0,
            from: renderer.offset,
            to: renderer.offset,
            choices: vec![],
            transform: None,
        });
    }
    if renderer
        .fields
        .iter()
        .filter(|field| field.index == 0)
        .count()
        > 1
    {
        return Err("Multiple snippet exit positions".into());
    }
    let mut order: Vec<_> = renderer
        .fields
        .iter()
        .filter(|field| field.transform.is_none())
        .map(|field| field.index)
        .filter(|index| *index > 0)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    order.push(0);
    Ok(SnippetPlan {
        text: renderer.text,
        fields: renderer.fields,
        order,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn v2_numeric_transforms_retain_computed_occurrences_and_forward_sources() {
        let syntax = r"${1:foo}_${1/(.*)/${1:/upcase}/}$0";
        assert!(expand_snippet(syntax, &BTreeMap::new()).is_err());
        let plan = expand_snippet_v2(syntax, &BTreeMap::new()).unwrap();
        assert_eq!(plan.text, "foo_FOO");
        assert_eq!(plan.order, vec![1, 0]);
        assert_eq!(plan.metadata(syntax)["version"], 2);
        let computed = plan.fields.iter().find(|f| f.transform.is_some()).unwrap();
        assert_eq!((computed.index, computed.from, computed.to), (1, 4, 7));
        assert_eq!(
            crate::snippet_transform::apply(computed.transform.as_ref().unwrap(), "ßa").unwrap(),
            "SSA"
        );
        let forward =
            expand_snippet_v2(r"${1/(.*)/prefix${1}/}${1:λ😀}$0", &BTreeMap::new()).unwrap();
        assert_eq!(forward.text, "prefixλ😀λ😀");
        let source = forward
            .fields
            .iter()
            .find(|f| f.index == 1 && f.transform.is_none())
            .unwrap();
        assert_eq!((source.from, source.to), (9, 12));
    }
    #[test]
    fn v2_nested_dependencies_variable_literals_and_orphans_share_initial_values() {
        let nested = expand_snippet_v2(
            r"${1:${2:foo} ${2/(.*)/${1:/upcase}/}} => ${1/(.*)/${1:/upcase}/}$0",
            &BTreeMap::new(),
        )
        .unwrap();
        assert_eq!(nested.text, "foo FOO => FOO FOO");
        assert_eq!(
            nested
                .fields
                .iter()
                .filter(|f| f.transform.is_some())
                .count(),
            2
        );
        assert!(
            nested
                .fields
                .iter()
                .any(|f| f.index == 2 && f.transform.is_some() && f.parent.is_some())
        );
        let variables = BTreeMap::from([
            ("KNOWN".into(), Some("$1 λ".into())),
            ("UNSET".into(), None),
        ]);
        let value=expand_snippet_v2(r"${KNOWN/(.*)/${1:/upcase}/} ${UNSET/(.+)/${1:-fallback}/} ${UNKNOWN/(.+)/${1:-missing}/}$0",&variables).unwrap();
        assert_eq!(value.text, "$1 Λ fallback missing");
        assert_eq!(value.fields.len(), 1);
        assert_eq!(value.metadata("")["version"], 1);
        let orphan = expand_snippet_v2(r"${1/(.*)/prefix${1}/}$0", &BTreeMap::new()).unwrap();
        assert_eq!(orphan.text, "prefix");
        assert_eq!(orphan.fields.len(), 1);
        assert_eq!(orphan.metadata("")["version"], 1);
        assert_eq!(
            requested_variables(
                r"$CLIPBOARD ${TM_FILENAME/(.*)/${1:/upcase}/} ${1:${INNER:default}}"
            )
            .unwrap(),
            BTreeSet::from(["CLIPBOARD".into(), "TM_FILENAME".into(), "INNER".into()])
        );
    }
    #[test]
    fn v2_hidden_variable_defaults_cannot_publish_orphan_computed_links() {
        let variables = BTreeMap::from([("KNOWN".into(), Some("literal".into()))]);
        let syntax = r"${KNOWN:${1:foo}} ${1/(.*)/${1:/upcase}/}$0";
        let plan = expand_snippet_v2(syntax, &variables).unwrap();
        assert_eq!(plan.text, "literal FOO");
        assert!(plan.fields.iter().all(|field| field.transform.is_none()));
        assert_eq!(plan.metadata(syntax)["version"], 1);
        let visible = expand_snippet_v2(
            r"${KNOWN:${1:foo}} ${1/(.*)/${1:/upcase}/} $1$0",
            &variables,
        )
        .unwrap();
        assert_eq!(visible.text, "literal FOO foo");
        assert_eq!(
            visible
                .fields
                .iter()
                .filter(|field| field.transform.is_some())
                .count(),
            1
        );
    }
    #[test]
    fn variable_demand_skips_hidden_defaults_but_follows_visible_mirrors() {
        let variables = BTreeMap::from([
            ("KNOWN".into(), Some("literal".into())),
            ("UNSET".into(), None),
        ]);
        assert_eq!(
            effective_requested_variables(r"${KNOWN:${CLIPBOARD:secret}}$0", &variables).unwrap(),
            BTreeSet::from(["KNOWN".into()])
        );
        assert_eq!(
            effective_requested_variables(r"${UNSET:${CLIPBOARD:default}}$0", &variables).unwrap(),
            BTreeSet::from(["UNSET".into(), "CLIPBOARD".into()])
        );
        assert_eq!(
            effective_requested_variables(r"${KNOWN:${1:$CLIPBOARD}} $1$0", &variables).unwrap(),
            BTreeSet::from(["KNOWN".into(), "CLIPBOARD".into()])
        );
        assert!(effective_requested_variables(r"${1:$1}", &variables).is_err());
    }
    #[test]
    fn v2_malformed_cycles_and_amplification_refuse_the_complete_plan() {
        for syntax in [
            r"${1/a/b/",
            r"${1/a/b/gg}",
            r"${1/(?=a)/b/}",
            r"${1:${1/(.*)/$1/}}",
            r"${1:${2/(.*)/$1/}}${2:$1}",
        ] {
            assert!(
                expand_snippet_v2(syntax, &BTreeMap::new()).is_err(),
                "{syntax}"
            )
        }
        let syntax = format!("${{1:a}}{}$0", r"${1/([a-z]+)/${1:/upcase}/}".repeat(4096));
        assert!(expand_snippet_v2(&syntax, &BTreeMap::new()).is_err());
    }

    #[test]
    fn forward_mirrors_nested_fields_and_utf16_positions() {
        let plan = expand_snippet("$2 ${1:😀 ${2:name}} = $1;$0", &BTreeMap::new()).unwrap();
        assert_eq!(plan.text, "name 😀 name = 😀 name;");
        assert_eq!(plan.order, vec![1, 2, 0]);
        assert_eq!(
            plan.fields
                .iter()
                .filter(|f| f.index == 1)
                .map(|f| (f.from, f.to))
                .collect::<Vec<_>>(),
            vec![(5, 12), (15, 22)]
        );
        for field in plan.fields.iter().filter(|f| f.index == 2) {
            assert_eq!(
                plan.text
                    .encode_utf16()
                    .skip(field.from)
                    .take(field.to - field.from)
                    .collect::<Vec<_>>(),
                "name".encode_utf16().collect::<Vec<_>>()
            );
        }
    }
    #[test]
    fn choices_escapes_and_variables_preserve_literal_values() {
        let vars = BTreeMap::from([
            ("TM_FILENAME".into(), Some("$file.rs".into())),
            ("TM_SELECTED_TEXT".into(), None),
        ]);
        let plan = expand_snippet(
            r"${1|a\,b,c\|d|}:$1 \$ ${TM_FILENAME} ${TM_SELECTED_TEXT:body} $UNKNOWN/$UNKNOWN",
            &vars,
        )
        .unwrap();
        assert_eq!(plan.text, "a,b:a,b $ $file.rs body UNKNOWN/UNKNOWN");
        assert_eq!(plan.order, vec![1, 2, 0]);
        assert_eq!(plan.fields[0].choices, vec!["a,b", "c|d"]);
        assert_eq!(plan.fields.iter().filter(|f| f.index == 2).count(), 2);
        assert_eq!(expand_snippet("${1:}", &vars).unwrap().fields[0].from, 0);
    }
    #[test]
    fn occurrence_parents_distinguish_nested_and_adjacent_empty_fields() {
        let plan = expand_snippet("${1:${2:}}$3$0", &BTreeMap::new()).unwrap();
        assert_eq!(
            plan.fields
                .iter()
                .map(|field| (field.id, field.parent, field.index))
                .collect::<Vec<_>>(),
            vec![(1, Some(0), 2), (0, None, 1), (2, None, 3), (3, None, 0)]
        );
        assert!(
            plan.fields
                .iter()
                .all(|field| field.from == 0 && field.to == 0)
        );
        assert!(expand_snippet(&"$1".repeat(MAX_FIELDS), &BTreeMap::new()).is_err());
        assert_eq!(
            expand_snippet(&"$1".repeat(MAX_FIELDS - 1), &BTreeMap::new())
                .unwrap()
                .fields
                .len(),
            MAX_FIELDS
        );
    }
    #[test]
    fn empty_choice_mirrors_cannot_amplify_unbounded_metadata() {
        let source = format!("${{1|,{}|}}{}$0", "x".repeat(1024), "$1".repeat(1029));
        assert!(source.len() < MAX_BYTES);
        assert!(
            expand_snippet(&source, &BTreeMap::new())
                .unwrap_err()
                .contains("metadata")
        );
    }
    #[test]
    fn every_empty_alternative_is_charged_before_mirrored_expansion() {
        let list = format!("${{1|{}|}}", ",".repeat(MAX_CHOICES - 1));
        let one = expand_snippet(&list, &BTreeMap::new()).unwrap();
        assert_eq!(one.fields[0].choices.len(), MAX_CHOICES);
        let mirrored = format!("{list}{}", "$1".repeat(8));
        assert!(mirrored.len() < MAX_BYTES);
        assert!(
            expand_snippet(&mirrored, &BTreeMap::new())
                .unwrap_err()
                .contains("metadata")
        );
        let one_too_many = format!("${{1|{}|}}", ",".repeat(MAX_CHOICES));
        assert!(
            expand_snippet(&one_too_many, &BTreeMap::new())
                .unwrap_err()
                .contains("choice limit")
        );
    }
    #[test]
    fn malformed_cyclic_unsupported_and_excessive_input_refuse() {
        for text in [
            "${1",
            "${1|a,b}",
            "${1|a|x,b|}",
            "${1:${1}}",
            "${1:a}${1:b}",
            "${0:no}",
            "$0$0",
            "${NAME/x/y/}",
            "${1/x/y/}",
            "$9999999999999999999999",
        ] {
            assert!(expand_snippet(text, &BTreeMap::new()).is_err(), "{text}");
        }
        assert!(expand_snippet(&"x".repeat(MAX_BYTES + 1), &BTreeMap::new()).is_err());
        let nested = format!(
            "{}x{}",
            "${1:".repeat(MAX_DEPTH + 1),
            "}".repeat(MAX_DEPTH + 1)
        );
        assert!(expand_snippet(&nested, &BTreeMap::new()).is_err());
    }
}
