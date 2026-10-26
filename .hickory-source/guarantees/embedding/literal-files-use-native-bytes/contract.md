<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Literal files use native bytes

For: Host developers embedding the document editor without a native execution service.

Given an unconditional top-level literal `file` in a document,
when the browser materializes it for local debugging,
then the Rust parser, native opening-break operation and native dedent operation
produce its bytes. Each emitted segment points to exactly equal UTF-8 source
bytes, including Unicode, BOMs and normalized indented CRLF. Nonliteral file
children, includes, conditions, duplicate file declarations, and declared
project/patch/entry construction are refused rather than executed as an incomplete
program. Complete project generation uses the separate pure evaluator with its
full Markdown closure; this does not add browser container or native execution.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/embedding/literal-files-use-native-bytes.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

<a id="output-da80d0b93723"></a>
## `crates/hick-lang/src/literal_files.rs`

Turn declared document constructions into immutable project trees, retaining byte origins so generated edits can return to their source.

Related contracts: [Literal files use native bytes](contract.md).

<a id="f-da80d0b93723-strip_opening_break"></a>
### `strip_opening_break`

The portable, non-executing literal-file subset. Unsupported dependencies
are refused, never silently dropped. Coordinates remain source UTF-8 bytes.

<storycode:copy id="f-da80d0b93723-strip_opening_break">
//! The portable, non-executing literal-file subset. Unsupported dependencies
//! are refused, never silently dropped. Coordinates remain source UTF-8 bytes.
use crate::{HickNode, SourceSpan, dedent, parse};

/// Remove the opening tag's line break, preserving the source coordinate.
pub fn strip_opening_break<'a>(
    text: &'a str,
    span: Option<&SourceSpan>,
) -> (&'a str, Option<SourceSpan>) {
    let stripped = text
        .strip_prefix("\r\n")
        .map(|s| (s, 2))
        .or_else(|| text.strip_prefix('\n').map(|s| (s, 1)));
    let Some((rest, taken)) = stripped else {
        return (text, span.copied());
    };
    (
        rest,
        span.map(|s| SourceSpan {
            start: s.start + taken,
            start_line: s.start_line + 1,
            start_col: 0,
            ..*s
        }),
    )
}

</storycode:copy>
<a id="f-da80d0b93723-LiteralSegment"></a>
### `LiteralSegment`

<storycode:copy id="f-da80d0b93723-LiteralSegment">
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct LiteralSegment {
    pub output: (usize, usize),
    pub source: (usize, usize),
}

</storycode:copy>
<a id="f-da80d0b93723-LiteralFile"></a>
### `LiteralFile`

<storycode:copy id="f-da80d0b93723-LiteralFile">
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct LiteralFile {
    pub path: String,
    pub content: String,
    pub segments: Vec<LiteralSegment>,
}

</storycode:copy>
<a id="f-da80d0b93723-literal_files"></a>
### `literal_files`

Materialize unconditional top-level files whose children are literal text.
This uses the native file path's opening-break and dedent operations.

<storycode:copy id="f-da80d0b93723-literal_files">
/// Materialize unconditional top-level files whose children are literal text.
/// This uses the native file path's opening-break and dedent operations.
pub fn literal_files(source: &str) -> Result<Vec<LiteralFile>, String> {
    let doc = parse(source).map_err(|e| e.to_string())?;
    if doc
        .all_tags()
        .iter()
        .any(|tag| matches!(tag.name.as_str(), "project" | "patch" | "entry"))
    {
        return Err("declared projects and patch/entry construction require complete project evaluation with the full Markdown closure".into());
    }
    let shift = usize::from(source.starts_with('\u{feff}')) * 3;
    let mut files = Vec::new();
    for node in &doc.nodes {
        let HickNode::Tag(tag) = node else { continue };
        if matches!(tag.name.as_str(), "when" | "include" | "upstream") {
            return Err(format!(
                "<{}:{}> requires workspace/fact resolution",
                doc.prefix, tag.name
            ));
        }
        if tag.name != "file" {
            continue;
        }
        let path = tag
            .get_attribute("path")
            .ok_or("file requires path")?
            .to_string();
        if files.iter().any(|f: &LiteralFile| f.path == path) {
            return Err(format!(
                "multiple file declarations for {path} require full weave"
            ));
        }
        let mut file = LiteralFile {
            path,
            content: String::new(),
            segments: Vec::new(),
        };
        for (i, child) in tag.children.iter().enumerate() {
            let HickNode::Text(text, span) = child else {
                return Err(format!(
                    "{} has nonliteral children requiring full weave",
                    file.path
                ));
            };
            let (text, span) = if i == 0 {
                strip_opening_break(text, span.as_ref())
            } else {
                (text.as_str(), *span)
            };
            let mut at = span.ok_or("literal text has no source span")?.start + shift;
            if tag.source_column == 0 {
                let start = file.content.len();
                file.content.push_str(text);
                if !text.is_empty() {
                    file.segments.push(LiteralSegment {
                        output: (start, file.content.len()),
                        source: (at, at + text.len()),
                    });
                }
            } else {
                let woven = dedent(text, tag.source_column);
                let (text, skipped) = text.strip_prefix('\n').map_or((text, 0), |rest| (rest, 1));
                at += skipped;
                let start = file.content.len();
                let mut output = start;
                let mut previous_newline = None;
                for (i, line) in text.lines().enumerate() {
                    if i > 0 {
                        let newline = previous_newline.expect("a prior line has a newline");
                        file.segments.push(LiteralSegment {
                            output: (output, output + 1),
                            source: (newline, newline + 1),
                        });
                        output += 1;
                    }
                    let removed = line
                        .bytes()
                        .take_while(|b| *b == b' ')
                        .count()
                        .min(tag.source_column);
                    let length = line.len() - removed;
                    if length > 0 {
                        file.segments.push(LiteralSegment {
                            output: (output, output + length),
                            source: (at + removed, at + line.len()),
                        });
                    }
                    output += length;
                    let consumed = text[at - (span.unwrap().start + shift + skipped)..]
                        .find('\n')
                        .map(|n| n + 1);
                    if let Some(consumed) = consumed {
                        previous_newline = Some(at + consumed - 1);
                        at += consumed;
                    }
                }
                if output < start + woven.len() {
                    let newline =
                        previous_newline.expect("trailing output break has a source break");
                    file.segments.push(LiteralSegment {
                        output: (output, output + 1),
                        source: (newline, newline + 1),
                    });
                }
                file.content.push_str(&woven);
            }
        }
        files.push(file);
    }
    Ok(files)
}

</storycode:copy>
<a id="f-da80d0b93723-tests"></a>
### `tests`

<storycode:copy id="f-da80d0b93723-tests">
#[cfg(test)]
mod tests {
    use super::*;
    // Guarantee: docs/guarantees/embedding/literal-files-use-native-bytes.md
    #[test]
    fn materializes_unicode_crlf_and_rebound_prefix_with_exact_lineage() {
        let source = "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\">\n🦀\n  <h:file path=\"a.js\">\r\n  var x = 'é';\r\n  </h:file>\n</h:doc>";
        let file = literal_files(source).unwrap().remove(0);
        assert_eq!(file.content, "var x = 'é';\n");
        for s in file.segments {
            assert_eq!(
                &file.content[s.output.0..s.output.1],
                &source[s.source.0..s.source.1]
            );
        }
    }
    #[test]
    fn dependent_files_are_refused() {
        assert!(
            literal_files("<hick:file path=\"a\"><hick:paste select=\"#x\"/></hick:file>").is_err()
        );
        assert!(
            literal_files("<hick:when test=\"x\"><hick:file path=\"a\">x</hick:file></hick:when>")
                .is_err()
        );
    }
}
</storycode:copy>

### Assemble `crates/hick-lang/src/literal_files.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-lang/src/literal_files.rs"><storycode:paste select="#f-da80d0b93723-strip_opening_break"/><storycode:paste select="#f-da80d0b93723-LiteralSegment"/><storycode:paste select="#f-da80d0b93723-LiteralFile"/><storycode:paste select="#f-da80d0b93723-literal_files"/><storycode:paste select="#f-da80d0b93723-tests"/></storycode:file>

<a id="output-484d9d41c41a"></a>
## `crates/hick-literate/tests/browser_literal_files.rs`

Related contracts: [Literal files use native bytes](contract.md).

<a id="f-484d9d41c41a-browser_literal_materialization_matches_native_nonexecuting_weave"></a>
### `browser_literal_materialization_matches_native_nonexecuting_weave`

<storycode:copy id="f-484d9d41c41a-browser_literal_materialization_matches_native_nonexecuting_weave">
// Guarantee: docs/guarantees/embedding/literal-files-use-native-bytes.md
use hick_exec::node::FileContent;
use hick_literate::run_pipeline_weave;

#[tokio::test]
async fn browser_literal_materialization_matches_native_nonexecuting_weave() {
    let sources = [
        "# Bare 🦀\n<hick:file path=\"a.js\">\nvar n = 1;\n</hick:file>\n",
        "---\ntitle: A note\n---\n<hick:file path=\"a.js\">var n = 2;</hick:file>",
        "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\">\n  <h:file path=\"a.js\">\r\n  var text = 'é🦀';\r\n  </h:file>\n</h:doc>",
        "  <hick:file path=\"a.js\">\n\n\n  var n = 3;\n  </hick:file>",
    ];
    for source in sources {
        let portable = hick_lang::literal_files(source).unwrap().remove(0);
        let native = run_pipeline_weave(&[("note.md", source)], &[], None)
            .await
            .unwrap();
        let FileContent::Text(content) = &native.files[&portable.path] else {
            panic!("literal text is a text file")
        };
        assert_eq!(portable.content, *content, "source: {source}");
        for segment in &portable.segments {
            assert_eq!(
                &portable.content[segment.output.0..segment.output.1],
                &source[segment.source.0..segment.source.1]
            );
        }
    }
}
</storycode:copy>

### Assemble `crates/hick-literate/tests/browser_literal_files.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-literate/tests/browser_literal_files.rs"><storycode:paste select="#f-484d9d41c41a-browser_literal_materialization_matches_native_nonexecuting_weave"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/debug/browser/compile.ts](../../debugging/browser-debugging-runs-edited-source/contract.md#output-7988c63e93d2)
- [crates/hick-lang-wasm/src/lib.rs](../../../mechanisms/crates/hick-lang-wasm/src/lib.rs.md#output-9a9e430ed3f3)
- [crates/hick-lang/src/literal_files.rs](contract.md#output-da80d0b93723)

### Tests and fixtures

- [apps/web/src/debug/browser/runtime.test.ts](../../../mechanisms/apps/web/src/debug/browser/runtime.ts.md#output-de529cc60d5d)
- [apps/web/src/elements/patch/view.test.tsx](../../../mechanisms/apps/web/src/elements/patch/view.tsx.md#output-059795295090)
- [apps/web/src/embed/DocumentEmbed.test.tsx](../../../mechanisms/apps/web/src/embed/DocumentEmbed.test.tsx.md#output-bb54f1265d97)
- [crates/hick-lang/tests/project.rs](../../../mechanisms/crates/hick-lang/tests/project.rs.md#output-005c1e9344c7)
- [crates/hick-literate/tests/browser_literal_files.rs](contract.md#output-484d9d41c41a)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
