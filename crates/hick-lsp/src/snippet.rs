use std::collections::{BTreeMap, BTreeSet};

const MAX_BYTES: usize = 262_144;
const MAX_OUTPUT: usize = 1_048_576;
const MAX_FIELDS: usize = 4096;
const MAX_DEPTH: usize = 32;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnippetField {
    pub index: u32,
    pub from: usize,
    pub to: usize,
    pub choices: Vec<String>,
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
}
struct Parser<'a> {
    text: &'a str,
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
                                choices.push(std::mem::take(&mut value));
                            }
                            Some('|') if self.take() == Some('}') => {
                                choices.push(value);
                                break;
                            }
                            Some(ch) => value.push(ch),
                            None => return self.fail("Unclosed snippet choice"),
                        }
                        if choices.len() > MAX_FIELDS {
                            return self.fail("Snippet choice limit exceeded");
                        }
                    }
                }
                Some('/') => {
                    return self.fail("Snippet transforms are not supported by this model");
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
            _ => {}
        }
    }
    Ok(())
}
struct Renderer<'a> {
    definitions: BTreeMap<u32, Definition>,
    variables: &'a BTreeMap<String, Option<String>>,
    unknown: BTreeMap<String, u32>,
    next: u32,
    active: BTreeSet<u32>,
    text: String,
    offset: usize,
    fields: Vec<SnippetField>,
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
        let from = self.offset;
        let definition = self.definitions.get(&index).cloned();
        if let Some(definition) = &definition {
            self.render(&definition.nodes, depth + 1)?;
        }
        self.active.remove(&index);
        if self.fields.len() >= MAX_FIELDS {
            return Err("Snippet expansion field limit exceeded".into());
        }
        self.fields.push(SnippetField {
            index,
            from,
            to: self.offset,
            choices: definition.map(|value| value.choices).unwrap_or_default(),
        });
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
    if text.len() > MAX_BYTES {
        return Err("Snippet input limit exceeded".into());
    }
    let nodes = Parser {
        text,
        at: 0,
        nodes: 0,
    }
    .sequence(false, 0)?;
    let mut definitions_map = BTreeMap::new();
    let mut max = 0;
    definitions(&nodes, &mut definitions_map, &mut max)?;
    let mut renderer = Renderer {
        definitions: definitions_map,
        variables,
        unknown: BTreeMap::new(),
        next: max,
        active: BTreeSet::new(),
        text: String::new(),
        offset: 0,
        fields: vec![],
    };
    renderer.render(&nodes, 0)?;
    if !renderer.fields.iter().any(|field| field.index == 0) {
        renderer.fields.push(SnippetField {
            index: 0,
            from: renderer.offset,
            to: renderer.offset,
            choices: vec![],
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
    fn malformed_cyclic_unsupported_and_excessive_input_refuse() {
        for text in [
            "${1",
            "${1|a,b}",
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
