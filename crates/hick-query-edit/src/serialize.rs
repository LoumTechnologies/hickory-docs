//! JSON-serializable DTOs for CLI output.
//!
//! These types decouple the JSON API from internal crate representations,
//! allowing the CLI output format to evolve independently.

use serde::{Deserialize, Serialize};

use hick_flow::node::SourceOrigin;
use hick_flow::provenance::ProvenanceSpan;
use hick_lang::SourceSpan;
use hick_lens::edit::{EditConfidence, SourceEdit};
use hick_lens::mapping::{AstElement, AstElementKind};

// ---------------------------------------------------------------------------
// Parse output
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct ParseOutput {
    pub prefix: String,
    pub weave_path: Option<String>,
    pub nodes: Vec<ParseNode>,
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum ParseNode {
    Text {
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<SpanDto>,
    },
    Tag {
        name: String,
        attributes: Vec<AttributeDto>,
        children: Vec<ParseNode>,
        self_closing: bool,
        source_line: usize,
    },
}

#[derive(Serialize)]
pub struct AttributeDto {
    pub name: String,
    pub value: String,
}

#[derive(Serialize, Clone)]
pub struct SpanDto {
    pub start: usize,
    pub end: usize,
    pub start_line: usize,
    pub start_col: usize,
}

// ---------------------------------------------------------------------------
// Query output
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct QueryMatchOutput {
    pub element_kind: String,
    pub element_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    pub start_line: usize,
    pub end_line: usize,
    pub output_text: String,
    pub origins: Vec<OriginDto>,
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum OriginDto {
    Literal { file: String, span: SpanDto },
    Exec { container: String, tag_line: usize },
    Paste { selector: String },
    Variable { name: String },
    Script { tag_line: usize },
    Synthetic,
}

// ---------------------------------------------------------------------------
// Edit output
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize)]
pub struct SourceEditDto {
    pub file: String,
    pub source_start: usize,
    pub source_end: usize,
    pub new_content: String,
    pub confidence: String,
}

// ---------------------------------------------------------------------------
// Conversions
// ---------------------------------------------------------------------------

impl From<&SourceSpan> for SpanDto {
    fn from(s: &SourceSpan) -> Self {
        SpanDto {
            start: s.start,
            end: s.end,
            start_line: s.start_line,
            start_col: s.start_col,
        }
    }
}

impl From<&SourceOrigin> for OriginDto {
    fn from(o: &SourceOrigin) -> Self {
        match o {
            SourceOrigin::Literal { file, span } => OriginDto::Literal {
                file: file.to_string(),
                span: SpanDto::from(span),
            },
            SourceOrigin::Exec {
                container,
                tag_line,
            } => OriginDto::Exec {
                container: container.to_string(),
                tag_line: *tag_line,
            },
            SourceOrigin::Paste { selector } => OriginDto::Paste {
                selector: selector.to_string(),
            },
            SourceOrigin::Variable { name } => OriginDto::Variable {
                name: name.to_string(),
            },
            SourceOrigin::Script { tag_line } => OriginDto::Script {
                tag_line: *tag_line,
            },
            SourceOrigin::Synthetic => OriginDto::Synthetic,
        }
    }
}

pub fn element_kind_str(kind: &AstElementKind) -> &'static str {
    match kind {
        AstElementKind::Function => "function",
        AstElementKind::Struct => "struct",
        AstElementKind::Enum => "enum",
        AstElementKind::Trait => "trait",
        AstElementKind::Impl => "impl",
        AstElementKind::Class => "class",
        AstElementKind::Interface => "interface",
        AstElementKind::Import => "import",
        AstElementKind::Const => "const",
        AstElementKind::TypeAlias => "type_alias",
        AstElementKind::Module => "module",
    }
}

pub fn query_match_output(
    element: &AstElement,
    output_text: &str,
    origins: Vec<&SourceOrigin>,
    file: Option<&str>,
) -> QueryMatchOutput {
    QueryMatchOutput {
        element_kind: element_kind_str(&element.kind).to_string(),
        element_name: element.name.clone(),
        file: file.map(|f| f.to_string()),
        start_line: element.start_line,
        end_line: element.end_line,
        output_text: output_text.to_string(),
        origins: origins.iter().map(|o| OriginDto::from(*o)).collect(),
    }
}

pub fn source_edit_dto(edit: &SourceEdit) -> SourceEditDto {
    SourceEditDto {
        file: edit.file.to_string(),
        source_start: edit.source_start,
        source_end: edit.source_end,
        new_content: edit.new_content.clone(),
        confidence: match edit.confidence {
            EditConfidence::Exact => "exact".to_string(),
            EditConfidence::Approximate => "approximate".to_string(),
            EditConfidence::Unmapped => "unmapped".to_string(),
        },
    }
}

pub fn provenance_span_dto(span: &ProvenanceSpan) -> serde_json::Value {
    serde_json::json!({
        "output_start": span.output_start,
        "output_end": span.output_end,
        "origin": OriginDto::from(&span.origin),
    })
}

pub fn convert_hick_node(node: &hick_lang::HickNode) -> ParseNode {
    match node {
        hick_lang::HickNode::Text(content, span) => ParseNode::Text {
            content: content.clone(),
            span: span.as_ref().map(SpanDto::from),
        },
        hick_lang::HickNode::Tag(tag) => ParseNode::Tag {
            name: tag.name.clone(),
            attributes: tag
                .attributes
                .iter()
                .map(|(k, v)| AttributeDto {
                    name: k.clone(),
                    value: v.clone(),
                })
                .collect(),
            children: tag.children.iter().map(convert_hick_node).collect(),
            self_closing: tag.self_closing,
            source_line: tag.source_line,
        },
    }
}
