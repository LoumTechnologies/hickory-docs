//! The element registry.
//!
//! A `.hick` document is text plus namespaced tags, and the vocabulary of
//! those tags used to be declared nowhere: about fifty names matched as
//! string literals across the crates, four kinds in the block model the app
//! draws, six on the editor's card rail, twenty in the editor's own parser,
//! sixty-two in the Insert menu. Adding one element touched all of them.
//!
//! This crate is the one place an element is declared. An [`Element`] says
//! what its tag is called, which attributes it takes, how it **renders** —
//! from a parsed tag and whatever facts a run produced, to one [`Block`] a
//! component draws — and which **actions** it answers. A [`Registry`] holds
//! the elements and walks a document into its blocks.
//!
//! Three things it is not:
//!
//! - **Not a parser.** `hick-lang` decides what is a tag. The registry only
//!   decides what a tag *means* once parsed, and a tag it does not know is
//!   drawn as nothing, exactly as before.
//! - **Not a runtime.** Rendering reads facts; it never produces them. What
//!   a cell's transcript is comes from the pipeline, behind `Executor`, and
//!   arrives here as context. An action may *ask* for a run; it does not
//!   perform one.
//! - **Not opinionated about the facts.** The context an element renders
//!   from is a type parameter, `Cx`, chosen by whoever builds the registry:
//!   the pipeline crate hands its run results, a lens hands whatever it
//!   views. The registry itself depends on the parser and nothing else.
//!
//! A block is `{kind, span, ...props}`: `kind` names the component that
//! draws it (the same string on both sides of the wire), `span` is the
//! tag's byte span in the source — bytes, because provenance is
//! byte-precise and nothing here changes that — and the props are whatever
//! the element chose to say. See `docs/specs/freeform/the-minimal-core.md`.

use std::collections::BTreeMap;

use hick_lang::{HickDocument, HickNode, HickTag};
use serde::Serialize;
use serde_json::{Map, Value};

/// One attribute an element accepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AttrSpec {
    pub name: &'static str,
    pub required: bool,
    /// What the attribute means, in a sentence.
    pub doc: &'static str,
}

impl AttrSpec {
    pub const fn required(name: &'static str, doc: &'static str) -> Self {
        Self {
            name,
            required: true,
            doc,
        }
    }

    pub const fn optional(name: &'static str, doc: &'static str) -> Self {
        Self {
            name,
            required: false,
            doc,
        }
    }
}

/// One block the app draws: the component's name, where in the source it
/// comes from, and what the element chose to say about it.
///
/// Serialises flat — `{"kind": …, "span": [a, b], …props}` — which is the
/// shape the app has always read.
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub kind: String,
    /// Byte span of the source this block stands for.
    pub span: (usize, usize),
    pub props: Map<String, Value>,
}

impl Block {
    /// A block with no props yet; add them with [`Block::with`].
    pub fn new(kind: impl Into<String>, span: (usize, usize)) -> Self {
        Self {
            kind: kind.into(),
            span,
            props: Map::new(),
        }
    }

    /// Set one prop. A `None` value is left out rather than written as
    /// `null`, so a reader can test presence.
    pub fn with(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        match serde_json::to_value(value) {
            Ok(Value::Null) => {}
            Ok(v) => {
                self.props.insert(key.into(), v);
            }
            Err(_) => {}
        }
        self
    }

    /// Read one prop back.
    pub fn prop(&self, key: &str) -> Option<&Value> {
        self.props.get(key)
    }

    /// Read one string prop back.
    pub fn str_prop(&self, key: &str) -> Option<&str> {
        self.props.get(key).and_then(Value::as_str)
    }
}

impl Serialize for Block {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(self.props.len() + 2))?;
        map.serialize_entry("kind", &self.kind)?;
        map.serialize_entry("span", &[self.span.0, self.span.1])?;
        for (k, v) in &self.props {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

/// Which of a tag's children the walk visits after rendering it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Descend {
    /// Every child, in order — a `when` gate, a `file` whose nested cells
    /// are blocks of their own.
    All,
    /// No child: the element's content is its own.
    None,
    /// Only the children whose tag name is in the list, and their subtrees.
    /// An `exec` renders as a cell and then shows the files it `ingested`.
    Named(&'static [&'static str]),
}

/// An element: one tag name, declared once.
///
/// `Cx` is whatever the registry's owner renders from — run results,
/// a commit, nothing. The trait says nothing about it.
pub trait Element<Cx>: Send + Sync {
    /// The tag name, without prefix: `"exec"`, `"file"`.
    fn name(&self) -> &'static str;

    /// The component that draws this element's block. Defaults to the tag
    /// name; an element whose block is drawn by another element's
    /// component says so here.
    fn kind(&self) -> &'static str {
        self.name()
    }

    /// The attributes this element accepts.
    fn attributes(&self) -> &'static [AttrSpec] {
        &[]
    }

    /// What this tag looks like as a block, or `None` to draw nothing —
    /// a declaration, a gate, a fragment somebody else pastes.
    fn render(&self, tag: &HickTag, cx: &Cx) -> Option<Block>;

    /// Which children the walk visits after this tag.
    fn descend(&self) -> Descend {
        Descend::None
    }

    /// The actions this element answers, by name.
    fn actions(&self) -> &'static [&'static str] {
        &[]
    }

    /// The provenance this element declares about its block. The default
    /// declares none.
    fn links(&self, _tag: &HickTag, _cx: &Cx) -> Vec<Link> {
        Vec::new()
    }

    /// Answer one action. The default answers none.
    fn act(
        &self,
        action: &str,
        _tag: &HickTag,
        _cx: &Cx,
        _body: Value,
    ) -> Result<ActionOutcome, ActionError> {
        Err(ActionError::Unknown {
            element: self.name(),
            action: action.to_string(),
        })
    }
}

/// What an action came to.
///
/// An element never runs anything and never writes the document; it
/// *asks*. The host that owns the registry — the server, the CLI — carries
/// the request out with the machinery it already has, so a cell's `run`
/// goes through the same `Executor` and the same run record as the Run
/// button always did.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "outcome", rename_all = "kebab-case")]
pub enum ActionOutcome {
    /// A plain answer.
    Answer { value: Value },
    /// Run these cells (by the ids their blocks carry), or the whole
    /// document when empty.
    Run { cells: Vec<String> },
    /// Replace this byte span of the source with this text.
    Edit {
        span: (usize, usize),
        replacement: String,
    },
}

/// Which kind of provenance a link is. Kept apart on purpose — see
/// `docs/specs/freeform/three-provenances.md`: lineage is derived from the
/// weave, context from the session record, declared is what a block says
/// about itself — and never drawn alike.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Family {
    Lineage,
    Context,
    Declared,
}

/// Where a link's far end is: a path in the open folder, and lines in it
/// when the record has them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LinkTarget {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines: Option<(usize, usize)>,
}

/// One provenance connection an element declares about its own block: from
/// a byte span of this document to a place in another (or the same) file.
///
/// An element says what it *knows*: a `read` knows the file it showed the
/// model, a `wrote` knows the lines it left, an assistant's prose knows what
/// it pointed at. What is drawn, and how, is the overlay's; what is true is
/// the element's, which is why this is declared beside `render` rather than
/// derived somewhere that has to know every element.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Link {
    pub family: Family,
    /// Byte span of the source this link starts from.
    pub span: (usize, usize),
    pub to: LinkTarget,
    /// What the link means, in one sentence, for the hover.
    pub title: String,
}

/// Why an action was not carried out.
#[derive(Debug, thiserror::Error)]
pub enum ActionError {
    #[error("<hick:{element}> has no action '{action}'")]
    Unknown {
        element: &'static str,
        action: String,
    },
    #[error("{0}")]
    Refused(String),
    #[error(transparent)]
    Failed(#[from] Box<dyn std::error::Error + Send + Sync>),
}

/// What a registry says about one element, for anything that needs the
/// vocabulary without the code: an Insert menu, a completion list, docs.
#[derive(Debug, Clone, Serialize)]
pub struct ElementDescription {
    pub name: &'static str,
    pub kind: &'static str,
    pub attributes: &'static [AttrSpec],
    pub actions: &'static [&'static str],
}

/// Renders a document's prose — the text between tags — to a block, or to
/// nothing.
pub type TextRenderer<Cx> = Box<dyn Fn(&str, (usize, usize), &Cx) -> Option<Block> + Send + Sync>;

/// The elements, by tag name, and the walk that turns a document into
/// blocks.
pub struct Registry<Cx> {
    elements: BTreeMap<&'static str, Box<dyn Element<Cx>>>,
    text: Option<TextRenderer<Cx>>,
}

impl<Cx> Default for Registry<Cx> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Cx> Registry<Cx> {
    pub fn new() -> Self {
        Self {
            elements: BTreeMap::new(),
            text: None,
        }
    }

    /// Declare an element. Declaring a name twice is a programming error
    /// and panics: two answers to "what is `<hick:exec>`" is the bug this
    /// crate exists to make impossible.
    pub fn register(&mut self, element: impl Element<Cx> + 'static) -> &mut Self {
        let name = element.name();
        if self.elements.insert(name, Box::new(element)).is_some() {
            panic!("element <hick:{name}> registered twice");
        }
        self
    }

    /// How the text between tags renders. Without one, prose is skipped.
    pub fn text(&mut self, renderer: TextRenderer<Cx>) -> &mut Self {
        self.text = Some(renderer);
        self
    }

    pub fn get(&self, name: &str) -> Option<&dyn Element<Cx>> {
        self.elements.get(name).map(|e| e.as_ref())
    }

    /// Every element, in name order.
    pub fn describe(&self) -> Vec<ElementDescription> {
        self.elements
            .values()
            .map(|e| ElementDescription {
                name: e.name(),
                kind: e.kind(),
                attributes: e.attributes(),
                actions: e.actions(),
            })
            .collect()
    }

    /// The document as blocks, in source order. The root tag of a wrapped
    /// document is not a block; its children are.
    pub fn blocks(&self, doc: &HickDocument, cx: &Cx) -> Vec<Block> {
        let mut out = Vec::new();
        self.walk(&doc.nodes, cx, &mut out);
        out
    }

    /// Every link every element declares, in source order, following the
    /// same descent the block walk does.
    pub fn links(&self, doc: &HickDocument, cx: &Cx) -> Vec<Link> {
        let mut out = Vec::new();
        self.walk_links(&doc.nodes, cx, &mut out);
        out
    }

    fn walk_links(&self, nodes: &[HickNode], cx: &Cx, out: &mut Vec<Link>) {
        for node in nodes {
            let HickNode::Tag(tag) = node else { continue };
            let Some(element) = self.get(&tag.name) else {
                continue;
            };
            out.extend(element.links(tag, cx));
            match element.descend() {
                Descend::All => self.walk_links(&tag.children, cx, out),
                Descend::None => {}
                Descend::Named(names) => {
                    for child in tag.child_tags() {
                        if names.contains(&child.name.as_str()) {
                            self.walk_links(&child.children, cx, out);
                        }
                    }
                }
            }
        }
    }

    fn walk(&self, nodes: &[HickNode], cx: &Cx, out: &mut Vec<Block>) {
        for node in nodes {
            match node {
                HickNode::Text(text, span) => {
                    if let Some(render) = &self.text
                        && let Some(block) =
                            render(text, span.map(|s| (s.start, s.end)).unwrap_or((0, 0)), cx)
                    {
                        out.push(block);
                    }
                }
                HickNode::Tag(tag) => {
                    let Some(element) = self.get(&tag.name) else {
                        continue;
                    };
                    if let Some(block) = element.render(tag, cx) {
                        out.push(block);
                    }
                    match element.descend() {
                        Descend::All => self.walk(&tag.children, cx, out),
                        Descend::None => {}
                        Descend::Named(names) => {
                            for child in tag.child_tags() {
                                if names.contains(&child.name.as_str()) {
                                    self.walk(&child.children, cx, out);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Answer an action on the tag that starts at `at` (a byte offset).
    ///
    /// Addressed by span rather than by ordinal so that an action lands on
    /// the block a person pointed at even after a block is inserted above
    /// it — the same reason the editor keys its rendered blocks by mapped
    /// position.
    pub fn act(
        &self,
        doc: &HickDocument,
        at: usize,
        action: &str,
        cx: &Cx,
        body: Value,
    ) -> Result<ActionOutcome, ActionError> {
        let Some(tag) = find_tag_at(&doc.nodes, at) else {
            return Err(ActionError::Refused(format!(
                "no element starts at byte {at} of this document"
            )));
        };
        let Some(element) = self.get(&tag.name) else {
            return Err(ActionError::Refused(format!(
                "<hick:{}> at byte {at} is not an element this registry knows",
                tag.name
            )));
        };
        element.act(action, tag, cx, body)
    }
}

fn find_tag_at(nodes: &[HickNode], at: usize) -> Option<&HickTag> {
    for node in nodes {
        if let HickNode::Tag(tag) = node {
            if tag.source_span.is_some_and(|s| s.start == at) {
                return Some(tag);
            }
            if let Some(found) = find_tag_at(&tag.children, at) {
                return Some(found);
            }
        }
    }
    None
}

/// The value of one attribute, owned.
pub fn attr(tag: &HickTag, name: &str) -> Option<String> {
    tag.get_attribute(name).map(str::to_string)
}

/// The tag's opening span as `(start, end)`, or `(0, 0)` for a tag built
/// by hand.
pub fn span_of(tag: &HickTag) -> (usize, usize) {
    tag.source_span.map(|s| (s.start, s.end)).unwrap_or((0, 0))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Note;
    impl Element<()> for Note {
        fn name(&self) -> &'static str {
            "note"
        }
        fn attributes(&self) -> &'static [AttrSpec] {
            const ATTRS: &[AttrSpec] = &[AttrSpec::required("id", "the note's name")];
            ATTRS
        }
        fn render(&self, tag: &HickTag, _: &()) -> Option<Block> {
            Some(Block::new("note", span_of(tag)).with("id", attr(tag, "id")))
        }
        fn descend(&self) -> Descend {
            Descend::All
        }
        fn actions(&self) -> &'static [&'static str] {
            &["shout"]
        }
        fn act(
            &self,
            action: &str,
            tag: &HickTag,
            _: &(),
            _: Value,
        ) -> Result<ActionOutcome, ActionError> {
            match action {
                "shout" => Ok(ActionOutcome::Answer {
                    value: Value::String(tag.text_content().to_uppercase()),
                }),
                other => Err(ActionError::Unknown {
                    element: "note",
                    action: other.to_string(),
                }),
            }
        }
    }

    struct Gate;
    impl Element<()> for Gate {
        fn name(&self) -> &'static str {
            "gate"
        }
        fn render(&self, _: &HickTag, _: &()) -> Option<Block> {
            None
        }
        fn descend(&self) -> Descend {
            Descend::Named(&["inner"])
        }
    }

    struct Inner;
    impl Element<()> for Inner {
        fn name(&self) -> &'static str {
            "inner"
        }
        fn render(&self, _: &HickTag, _: &()) -> Option<Block> {
            None
        }
        fn descend(&self) -> Descend {
            Descend::All
        }
    }

    fn registry() -> Registry<()> {
        let mut r = Registry::new();
        r.register(Note).register(Gate).register(Inner);
        r.text(Box::new(|text, span, _| {
            (!text.trim().is_empty()).then(|| Block::new("prose", span).with("text", text.trim()))
        }));
        r
    }

    #[test]
    fn a_block_serialises_flat() {
        let block = Block::new("note", (3, 9))
            .with("id", "a")
            .with("missing", None::<String>);
        let json = serde_json::to_value(&block).unwrap();
        assert_eq!(
            json,
            serde_json::json!({"kind": "note", "span": [3, 9], "id": "a"})
        );
    }

    #[test]
    fn the_walk_renders_known_tags_and_prose_and_skips_the_rest() {
        let doc = hick_lang::parse("hello\n<hick:note id=\"a\">x</hick:note>\n<hick:other/>\nbye")
            .unwrap();
        let blocks = registry().blocks(&doc, &());
        let kinds: Vec<&str> = blocks.iter().map(|b| b.kind.as_str()).collect();
        assert_eq!(kinds, vec!["prose", "note", "prose", "prose"]);
        assert_eq!(blocks[1].str_prop("id"), Some("a"));
        assert_eq!(blocks[1].span, (6, 24));
    }

    #[test]
    fn descent_is_the_elements_choice() {
        let doc = hick_lang::parse(
            "<hick:gate><hick:inner><hick:note id=\"in\"/></hick:inner><hick:note id=\"out\"/></hick:gate>",
        )
        .unwrap();
        let blocks = registry().blocks(&doc, &());
        let ids: Vec<&str> = blocks.iter().filter_map(|b| b.str_prop("id")).collect();
        assert_eq!(ids, vec!["in"], "only the named child is entered");
    }

    #[test]
    fn an_action_is_addressed_by_span_and_answered_by_the_element() {
        let doc = hick_lang::parse("x\n<hick:note id=\"a\">quiet</hick:note>").unwrap();
        let r = registry();
        let answer = r.act(&doc, 2, "shout", &(), Value::Null).unwrap();
        assert_eq!(
            answer,
            ActionOutcome::Answer {
                value: Value::String("QUIET".into())
            }
        );
        assert_eq!(
            serde_json::to_value(&answer).unwrap(),
            serde_json::json!({"outcome": "answer", "value": "QUIET"})
        );
        let err = r.act(&doc, 2, "whisper", &(), Value::Null).unwrap_err();
        assert_eq!(err.to_string(), "<hick:note> has no action 'whisper'");
        let err = r.act(&doc, 0, "shout", &(), Value::Null).unwrap_err();
        assert!(err.to_string().contains("no element starts at byte 0"));
    }

    struct Cites;
    impl Element<()> for Cites {
        fn name(&self) -> &'static str {
            "cites"
        }
        fn render(&self, tag: &HickTag, _: &()) -> Option<Block> {
            Some(Block::new("cites", span_of(tag)))
        }
        fn links(&self, tag: &HickTag, _: &()) -> Vec<Link> {
            vec![Link {
                family: Family::Declared,
                span: span_of(tag),
                to: LinkTarget {
                    path: attr(tag, "file").unwrap_or_default(),
                    lines: Some((1, 2)),
                },
                title: "says so".into(),
            }]
        }
    }

    #[test]
    fn an_element_declares_its_own_links_and_the_walk_collects_them() {
        let mut r: Registry<()> = Registry::new();
        r.register(Cites).register(Gate).register(Inner);
        let doc = hick_lang::parse(
            "<hick:cites file=\"a.md\"/>\n<hick:gate><hick:inner><hick:cites file=\"b.md\"/></hick:inner></hick:gate>",
        )
        .unwrap();
        let links = r.links(&doc, &());
        let paths: Vec<&str> = links.iter().map(|l| l.to.path.as_str()).collect();
        assert_eq!(paths, vec!["a.md", "b.md"]);
        assert_eq!(links[0].family, Family::Declared);
        assert_eq!(links[0].span, (0, "<hick:cites file=\"a.md\"/>".len()));
        assert_eq!(
            serde_json::to_value(&links[0]).unwrap()["family"],
            "declared"
        );
    }

    #[test]
    fn describe_lists_the_vocabulary() {
        let described = registry().describe();
        let names: Vec<&str> = described.iter().map(|d| d.name).collect();
        assert_eq!(names, vec!["gate", "inner", "note"]);
        assert_eq!(described[2].attributes[0].name, "id");
        assert_eq!(described[2].actions, &["shout"]);
    }

    #[test]
    #[should_panic(expected = "registered twice")]
    fn registering_a_name_twice_is_refused() {
        let mut r: Registry<()> = Registry::new();
        r.register(Note).register(Note);
    }
}
