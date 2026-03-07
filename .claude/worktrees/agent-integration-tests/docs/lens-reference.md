# Lens Reference

hick-lens bridges generated output files and their `.hick` source positions.
Given a converged output file and its `ProvenanceMap`, it uses tree-sitter to
parse the output, identifies structural elements (functions, structs, classes,
imports), and maps them back to their source `.hick` positions.

---

## Table of Contents

1. [Language](#1-language)
2. [AstMapping](#2-astmapping)
3. [AstElement](#3-astelement)
4. [AstElementKind](#4-astelementkind)
5. [Selector](#5-selector)
6. [SelectorMatch](#6-selectormatch)
7. [SourceEdit](#7-sourceedit)
8. [EditConfidence](#8-editconfidence)
9. [Annotation](#9-annotation)
10. [StructuralKind Tables](#10-structuralkind-tables)

---

## 1. Language

**Module:** `hick_lens::language`

Supported output languages for tree-sitter parsing and structural element
detection.

### Variants

| Variant | Extension | Tree-sitter Root |
|---------|-----------|-----------------|
| `Rust` | `.rs` | `source_file` |
| `TypeScript` | `.ts` | `program` |
| `Tsx` | `.tsx` | `program` |
| `Python` | `.py` | `module` |

### Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `ts_language` | `(&self) -> TsLanguage` | Get the tree-sitter language grammar for parsing |
| `from_extension` | `(ext: &str) -> Option<Self>` | Detect language from a file extension |
| `from_path` | `(path: &str) -> Option<Self>` | Detect language from a file path by extracting its extension |
| `structural_kinds` | `(&self) -> &[StructuralKind]` | Get the table of tree-sitter node kinds that represent structural elements |

### Example

```rust,ignore
use hick_lens::Language;

let lang = Language::from_path("src/main.rs"); // Some(Language::Rust)
let lang = Language::from_extension("tsx");     // Some(Language::Tsx)
```

---

## 2. AstMapping

**Module:** `hick_lens::mapping`

The result of parsing an output file with tree-sitter and correlating its
structural elements with provenance origins. This is the central type that
ties together AST analysis and source traceability.

### Construction

```rust,ignore
use hick_lens::{AstMapping, Language};
use hick_flow::provenance::ProvenanceMap;

let mapping = AstMapping::build(output_text, provenance_map, Language::Rust)?;
```

### Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `build` | `(output: &str, provenance: ProvenanceMap, language: Language) -> Result<Self>` | Parse output with tree-sitter and identify structural elements |
| `elements` | `(&self) -> &[AstElement]` | Get all structural elements found in the output |
| `provenance` | `(&self) -> &ProvenanceMap` | Get the provenance map used for this mapping |
| `source` | `(&self) -> &str` | Get the output source text |
| `find_by_name` | `(&self, name: &str) -> Vec<&AstElement>` | Find elements matching a given name |
| `find_by_kind` | `(&self, kind: &AstElementKind) -> Vec<&AstElement>` | Find elements of a specific kind |
| `origins_for_element` | `(&self, element: &AstElement) -> Vec<&SourceOrigin>` | Find provenance origins that overlap an element's byte range |

### AstMappingError

| Variant | Description |
|---------|-------------|
| `LanguageError(String)` | Failed to set the tree-sitter language grammar |
| `ParseFailed` | Tree-sitter parse returned no tree |

---

## 3. AstElement

**Module:** `hick_lens::mapping`

A structural element found in the output AST.

| Field | Type | Description |
|-------|------|-------------|
| `kind` | `AstElementKind` | What kind of element this is |
| `name` | `String` | Element name (e.g. function name, struct name) |
| `output_start` | `usize` | Byte offset in the output file (start, inclusive) |
| `output_end` | `usize` | Byte offset in the output file (end, exclusive) |
| `start_line` | `usize` | Start line in the output file (0-based) |
| `end_line` | `usize` | End line in the output file (0-based) |

---

## 4. AstElementKind

**Module:** `hick_lens::mapping`

The kind of structural element. Mirrors language constructs across Rust,
TypeScript, and Python.

| Variant | Languages |
|---------|-----------|
| `Function` | Rust (`fn`), TypeScript (`function`), Python (`def`) |
| `Struct` | Rust |
| `Enum` | Rust, TypeScript |
| `Trait` | Rust |
| `Impl` | Rust |
| `Class` | TypeScript, Python |
| `Interface` | TypeScript |
| `Import` | Rust (`use`), TypeScript (`import`), Python (`import`, `from...import`) |
| `Const` | Rust |
| `TypeAlias` | Rust (`type`), TypeScript (`type`) |
| `Module` | Rust (`mod`) |

---

## 5. Selector

**Module:** `hick_lens::selector`

A parsed selector for querying output elements. Supports structural queries
by keyword + name, CSS-style selectors, and bare name matching.

### Selector Syntax

| Format | Variant | Example |
|--------|---------|---------|
| `fn <name>` | `Function` | `fn main` |
| `struct <Name>` | `Struct` | `struct Config` |
| `enum <Name>` | `Enum` | `enum Status` |
| `trait <Name>` | `Trait` | `trait Handler` |
| `impl <Name>` | `Impl` | `impl Handler` |
| `class <Name>` | `Class` | `class App` |
| `interface <Name>` | `Interface` | `interface Props` |
| `const <NAME>` | `Const` | `const MAX_SIZE` |
| `type <Name>` | `TypeAlias` | `type Result` |
| `mod <name>` | `Module` | `mod utils` |
| `<name>` | `ByName` | `greet` (matches any kind) |
| `#<id>` | `Id` | `#version` (delegates to paste resolution) |
| `.<class>` | `CssClass` | `.imports` (delegates to paste resolution) |

### Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `parse` | `(input: &str) -> Result<Self, SelectorParseError>` | Parse a selector string |
| `matches` | `(&self, element: &AstElement) -> bool` | Check if an AST element matches this selector |
| `query` | `(&self, mapping: &AstMapping) -> Vec<SelectorMatch>` | Query an AST mapping for all matching elements |

### SelectorParseError

| Variant | Description |
|---------|-------------|
| `Empty` | Empty selector string |
| `MissingName(String)` | Keyword provided but no name followed |
| `UnknownKeyword(String)` | Unrecognized keyword |

### Example

```rust,ignore
use hick_lens::{Selector, AstMapping, Language};
use hick_flow::provenance::ProvenanceMap;

let source = "fn greet() {}\nfn main() {}\n";
let mapping = AstMapping::build(source, ProvenanceMap::new(), Language::Rust)?;

let selector = Selector::parse("fn greet")?;
let matches = selector.query(&mapping);
// matches[0].element.name == "greet"
// matches[0].output_text contains "fn greet() {}"
```

---

## 6. SelectorMatch

**Module:** `hick_lens::selector`

A match result from a selector query.

| Field | Type | Description |
|-------|------|-------------|
| `element` | `&AstElement` | The matched AST element |
| `output_text` | `&str` | The output text slice for this element |

---

## 7. SourceEdit

**Module:** `hick_lens::edit`

A computed edit to apply to a `.hick` source file. Returned by
`compute_edits` -- does not modify files directly.

| Field | Type | Description |
|-------|------|-------------|
| `file` | `Arc<str>` | Path to the `.hick` source file |
| `source_start` | `usize` | Byte offset in the `.hick` source (start, inclusive) |
| `source_end` | `usize` | Byte offset in the `.hick` source (end, exclusive) |
| `new_content` | `String` | The new content to insert at this position |
| `confidence` | `EditConfidence` | Confidence level of this edit |
| `source_span` | `Option<SourceSpan>` | The original source span that was matched |

### compute_edits

```rust,ignore
pub fn compute_edits(
    element: &AstElement,
    new_content: &str,
    provenance: &ProvenanceMap,
) -> Vec<SourceEdit>
```

Given an element found via selector query and the provenance map, determines
which `.hick` source byte ranges need to be modified:

1. **No overlapping provenance spans** -- returns a single `Unmapped` edit.
2. **All overlapping spans are `Literal` from the same file** -- computes
   the enclosing source range and returns a single `Exact` edit.
3. **Mixed origins** (Literal + Paste/Exec/etc.) -- groups Literal spans by
   file and returns `Approximate` edits per file.

---

## 8. EditConfidence

**Module:** `hick_lens::edit`

Confidence level of a computed source edit.

| Variant | Description |
|---------|-------------|
| `Exact` | Provenance maps directly to a `.hick` literal span. Exact byte-range replacement is safe. |
| `Approximate` | The element spans multiple provenance origins. The edit targets the enclosing hick element and may need review. |
| `Unmapped` | No provenance information is available. The edit cannot be mapped back to source. |

---

## 9. Annotation

**Module:** `hick_lens::annotate`

Virtual structural annotations for output files. These are never written to
disk -- they exist only in query responses (e.g. via hick-mcp).

### Annotation

| Field | Type | Description |
|-------|------|-------------|
| `tag` | `String` | Virtual tag name (e.g. `hick:fn`, `hick:struct`) |
| `name` | `String` | Element name |
| `output_start` | `usize` | Byte offset where the element starts |
| `output_end` | `usize` | Byte offset where the element ends |
| `start_line` | `usize` | Start line (0-based) |
| `end_line` | `usize` | End line (0-based) |

### Virtual Tag Names

| AstElementKind | Virtual Tag |
|----------------|-------------|
| `Function` | `hick:fn` |
| `Struct` | `hick:struct` |
| `Enum` | `hick:enum` |
| `Trait` | `hick:trait` |
| `Impl` | `hick:impl` |
| `Class` | `hick:class` |
| `Interface` | `hick:interface` |
| `Import` | `hick:import` |
| `Const` | `hick:const` |
| `TypeAlias` | `hick:type` |
| `Module` | `hick:mod` |

### Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `annotate` | `(mapping: &AstMapping) -> Vec<Annotation>` | Generate virtual annotations from an AST mapping |
| `structure_summary` | `(mapping: &AstMapping, depth: Option<usize>) -> String` | Build a tree-structured text summary of structural elements |

### Summary Format

```
fn main [L1-L5]
fn helper [L7-L10]
struct Config [L12-L15]
```

---

## 10. StructuralKind Tables

**Module:** `hick_lens::language`

Each language defines a table mapping tree-sitter node kinds to
`AstElementKind` values and the child field name that holds the element's
identifier.

### Rust

| Node Kind | Element Kind | Name Field |
|-----------|-------------|------------|
| `function_item` | `Function` | `name` |
| `struct_item` | `Struct` | `name` |
| `enum_item` | `Enum` | `name` |
| `trait_item` | `Trait` | `name` |
| `impl_item` | `Impl` | `type` |
| `const_item` | `Const` | `name` |
| `type_item` | `TypeAlias` | `name` |
| `mod_item` | `Module` | `name` |
| `use_declaration` | `Import` | `argument` |

### TypeScript / TSX

| Node Kind | Element Kind | Name Field |
|-----------|-------------|------------|
| `function_declaration` | `Function` | `name` |
| `class_declaration` | `Class` | `name` |
| `interface_declaration` | `Interface` | `name` |
| `type_alias_declaration` | `TypeAlias` | `name` |
| `enum_declaration` | `Enum` | `name` |
| `import_statement` | `Import` | `source` |

### Python

| Node Kind | Element Kind | Name Field |
|-----------|-------------|------------|
| `function_definition` | `Function` | `name` |
| `class_definition` | `Class` | `name` |
| `import_statement` | `Import` | `name` |
| `import_from_statement` | `Import` | `module_name` |
