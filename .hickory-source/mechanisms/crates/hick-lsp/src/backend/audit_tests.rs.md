<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-lsp/src/backend/audit_tests.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-c123076e88e3"></a>
## `crates/hick-lsp/src/backend/audit_tests.rs`

<a id="f-c123076e88e3-provider_rename_refusal_is_an_error_instead_of_no_changes"></a>
### `provider_rename_refusal_is_an_error_instead_of_no_changes`

The frontmatter witness keeps malformed YAML available to the provider, maps
UTF-16 diagnostic/completion columns in both directions, and translates a whole
body formatter edit up to the closing delimiter without consuming prose. BOM,
CRLF, alternate closing markers, empty metadata and path collisions exercise the
projection boundaries; this does not claim the YAML provider is installed.

<storycode:copy id="f-c123076e88e3-provider_rename_refusal_is_an_error_instead_of_no_changes">
use super::*;
use crate::virtual_file::VirtualSegment;
use hick_lang::SourceSpan;

#[test]
fn frontmatter_yaml_maps_features_and_whole_body_formatting() {
    let source = "\u{feff}---\r\ntitle: [broken\r\n# metadata comment\r\n...\r\n# Prose\n";
    let vf = HickBackend::frontmatter_file(source, &[]).unwrap();
    assert_eq!(vf.language_id, Some("yaml"));
    assert_eq!(vf.content(), "title: [broken\r\n# metadata comment\r\n");
    let map = PositionMap::build(&vf.segments);
    assert_eq!(map.to_source(0, 8), Some((1, 8)));
    assert_eq!(map.to_virtual(1, 8), Some((0, 8)));
    assert_eq!(map.to_virtual(4, 0), None);
    let edits = translate_edits(
        &serde_json::json!([{
            "range": {"start": {"line": 0, "character": 0}, "end": {"line": 2, "character": 0}},
            "newText": "title: fixed\r\n# metadata comment\r\n"
        }]),
        &map,
    )
    .unwrap();
    assert_eq!(
        edits[0].range,
        Range::new(Position::new(1, 0), Position::new(3, 0))
    );
    assert_eq!(edits[0].new_text, "title: fixed\r\n# metadata comment\r\n");
    let other = HickBackend::frontmatter_file(source, &[vf.clone()]).unwrap();
    assert_ne!(other.path, vf.path);
    assert!(HickBackend::frontmatter_file("# Body\n---\na: b\n---\n", &[]).is_none());
    assert!(HickBackend::frontmatter_file("---\na: b\n", &[]).is_none());
    assert_eq!(
        HickBackend::frontmatter_file("---\n---\n", &[])
            .unwrap()
            .content(),
        ""
    );
}

/// Run explicitly with yaml-language-server on PATH: exercises the actual
/// child protocol, translated diagnostics, formatter and provider retirement.
#[tokio::test]
#[ignore = "requires yaml-language-server on PATH"]
async fn real_yaml_frontmatter_diagnostics_formatting_and_removal() {
    let directory = tempfile::tempdir().unwrap();
    let uri = Url::from_file_path(directory.path().join("notes.md")).unwrap();
    let (service, _socket) = tower_lsp::LspService::new(HickBackend::new);
    let backend = service.inner();
    backend.spawn_notification_handler();
    let broken = "---\ntitle: [broken\n---\n# Prose\n";
    assert!(is_hick_document(&uri, broken));
    backend.process_document(&uri, broken).await;
    tokio::time::timeout(std::time::Duration::from_secs(15), async {
        loop {
            if backend
                .child_diagnostics
                .read()
                .await
                .get(&uri)
                .is_some_and(|files| {
                    files
                        .values()
                        .flatten()
                        .any(|diag| diag.severity == Some(DiagnosticSeverity::ERROR))
                })
            {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("YAML syntax errors must reach Markdown diagnostics");
    assert_eq!(
        backend
            .virtual_target(&uri, Position::new(1, 3))
            .await
            .unwrap()
            .1,
        "yaml"
    );
    assert!(
        backend
            .virtual_target(&uri, Position::new(3, 0))
            .await
            .is_none()
    );
    let valid = "---\ntitle:   hello\nitems: [a,b]\n---\n# Prose\n";
    backend.process_document(&uri, valid).await;
    let edits = backend
        .formatting(
            serde_json::from_value(serde_json::json!({
                "textDocument": {"uri": uri}, "options": {"tabSize": 2, "insertSpaces": true}
            }))
            .unwrap(),
        )
        .await
        .unwrap()
        .expect("real YAML formatter must return edits");
    assert!(!edits.is_empty());
    assert!(
        edits
            .iter()
            .all(|edit| edit.range.start.line >= 1 && edit.range.end.line <= 3)
    );
    assert!(
        edits
            .iter()
            .any(|edit| edit.new_text.contains("title: hello"))
    );
    backend.process_document(&uri, "# Prose\n").await;
    assert!(
        !backend
            .vfile_index
            .read()
            .await
            .values()
            .any(|mapping| mapping.hick_uri == uri && mapping.language_id == "yaml")
    );
    assert!(
        !backend
            .child_diagnostics
            .read()
            .await
            .get(&uri)
            .is_some_and(|files| files
                .values()
                .flatten()
                .any(|diag| diag.source.as_deref() == Some("YAML")))
    );
}

#[test]
fn provider_rename_refusal_is_an_error_instead_of_no_changes() {
    let error = child_request_error(crate::child_lsp::ChildLspError::JsonRpc {
        code: -32001,
        message: "Index results do not match current content".into(),
    });
    assert_eq!(
        error.code,
        tower_lsp::jsonrpc::ErrorCode::ServerError(-32001)
    );
    assert!(error.message.contains("Index results do not match"));
}
</storycode:copy>
<a id="f-c123076e88e3-code_actions_keep_exact_child_diagnostic_identity_and_coordinates"></a>
### `code_actions_keep_exact_child_diagnostic_identity_and_coordinates`

<storycode:copy id="f-c123076e88e3-code_actions_keep_exact_child_diagnostic_identity_and_coordinates">
#[test]
fn code_actions_keep_exact_child_diagnostic_identity_and_coordinates() {
    let diagnostic = serde_json::json!({"range":{"start":{"line":0,"character":22},"end":{"line":0,"character":23}},"code":"expected_semi_after_stmt","message":"Expected semicolon","source":"clang","data":{"provider-token":17}});
    // The cursor may precede the diagnostic for a missing insertion.
    // A mapped literate request must use virtual line 0 here, not its
    // parent Markdown line. No editor-supplied diagnostics are involved.
    let selected = provider_action_diagnostics(
        &[diagnostic.clone()],
        Range::new(Position::new(0, 21), Position::new(0, 21)),
    );
    assert_eq!(selected, vec![diagnostic.clone()]);
    assert!(
        provider_action_diagnostics(
            &[diagnostic.clone()],
            Range::new(Position::new(8, 21), Position::new(8, 21))
        )
        .is_empty()
    );
    assert!(
        provider_action_diagnostics(
            &[diagnostic],
            Range::new(Position::new(0, 0), Position::new(0, 5))
        )
        .is_empty()
    );
    assert!(
        provider_action_diagnostics(
            &[serde_json::json!({"message":"missing range"})],
            Range::default()
        )
        .is_empty()
    );
}

</storycode:copy>
<a id="f-c123076e88e3-seg"></a>
### `seg`

<storycode:copy id="f-c123076e88e3-seg">
fn seg(text: &str, start_line: usize, col: usize) -> VirtualSegment {
    VirtualSegment {
        text: text.to_string(),
        source_span: Some(SourceSpan::new(0, text.len(), start_line, col)),
        source_line: start_line,
        source_column: col,
    }
}

</storycode:copy>
<a id="f-c123076e88e3-index_with"></a>
### `index_with`

<storycode:copy id="f-c123076e88e3-index_with">
fn index_with(vf: &str, hick: &str, map: PositionMap) -> HashMap<String, (Url, PositionMap)> {
    let mut index = HashMap::new();
    index.insert(uri_key(vf), (Url::parse(hick).unwrap(), map));
    index
}

</storycode:copy>
<a id="f-c123076e88e3-a_child_that_spells_a_uri_differently_still_maps_home"></a>
### `a_child_that_spells_a_uri_differently_still_maps_home`

<storycode:copy id="f-c123076e88e3-a_child_that_spells_a_uri_differently_still_maps_home">
#[test]
fn a_child_that_spells_a_uri_differently_still_maps_home() {
    // basedpyright on Windows answers `file:///c%3A/Users/...` for the file
    // we staged as `file:///C:/Users/...` — lowercase drive, percent-encoded
    // colon. Keyed by the raw string those are different files, the
    // location is never translated, and go-to-definition lands the user in
    // a staging directory instead of their own document.
    assert_eq!(
        uri_key("file:///C:/Users/x/app.py"),
        uri_key("file:///c%3A/Users/x/app.py"),
        "two spellings of one path must key the same"
    );
    // Different files stay different.
    assert_ne!(
        uri_key("file:///C:/Users/x/app.py"),
        uri_key("file:///C:/Users/x/other.py")
    );
    // Something that is not a file URI is kept whole rather than mangled.
    assert_eq!(uri_key("untitled:Untitled-1"), "untitled:Untitled-1");
}

</storycode:copy>
<a id="f-c123076e88e3-references_result_translates_to_hick_coordinates"></a>
### `references_result_translates_to_hick_coordinates`

<storycode:copy id="f-c123076e88e3-references_result_translates_to_hick_coordinates">
#[test]
fn references_result_translates_to_hick_coordinates() {
    // Virtual file lines 0..3 map to .md source lines 9..12 (1-based
    // 10..13), each indented 4 columns.
    let map = PositionMap::build(&[seg("fn a() {}\nfn b() {\n    a();\n}\n", 10, 4)]);
    let index = index_with("file:///tmp/vf/main.rs", "hick:///doc.md", map);

    let child_result = serde_json::json!([
        { "uri": "file:///tmp/vf/main.rs",
          "range": { "start": { "line": 0, "character": 3 },
                     "end":   { "line": 0, "character": 4 } } },
        { "uri": "file:///tmp/vf/main.rs",
          "range": { "start": { "line": 2, "character": 4 },
                     "end":   { "line": 2, "character": 5 } } },
    ]);

    let translated = translate_locations(child_result, &index);
    let locs: Vec<Location> = serde_json::from_value(translated).unwrap();
    assert_eq!(locs.len(), 2);
    assert_eq!(locs[0].uri.as_str(), "hick:///doc.md");
    assert_eq!(locs[0].range.start, Position::new(9, 7));
    assert_eq!(locs[0].range.end, Position::new(9, 8));
    assert_eq!(locs[1].range.start, Position::new(11, 8));
    assert_eq!(locs[1].range.end, Position::new(11, 9));
}

</storycode:copy>
<a id="f-c123076e88e3-references_in_unknown_vfile_pass_through"></a>
### `references_in_unknown_vfile_pass_through`

<storycode:copy id="f-c123076e88e3-references_in_unknown_vfile_pass_through">
#[test]
fn references_in_unknown_vfile_pass_through() {
    let map = PositionMap::build(&[seg("x\n", 1, 0)]);
    let index = index_with("file:///tmp/vf/main.rs", "hick:///doc.md", map);

    let child_result = serde_json::json!([
        { "uri": "file:///somewhere/else.rs",
          "range": { "start": { "line": 5, "character": 0 },
                     "end":   { "line": 5, "character": 3 } } },
    ]);

    let translated = translate_locations(child_result.clone(), &index);
    assert_eq!(translated, child_result);
}

</storycode:copy>
<a id="f-c123076e88e3-references_on_untranslatable_lines_keep_virtual_location"></a>
### `references_on_untranslatable_lines_keep_virtual_location`

<storycode:copy id="f-c123076e88e3-references_on_untranslatable_lines_keep_virtual_location">
#[test]
fn references_on_untranslatable_lines_keep_virtual_location() {
    // Map covers only virtual line 0; a result on line 7 (e.g. inside
    // pasted content beyond the mapped segment) must survive untouched so
    // the server-side bridge can map it through provenance.
    let map = PositionMap::build(&[seg("only one line\n", 3, 0)]);
    let index = index_with("file:///tmp/vf/main.rs", "hick:///doc.md", map);

    let child_result = serde_json::json!([
        { "uri": "file:///tmp/vf/main.rs",
          "range": { "start": { "line": 7, "character": 2 },
                     "end":   { "line": 7, "character": 6 } } },
    ]);

    let translated = translate_locations(child_result.clone(), &index);
    assert_eq!(translated, child_result);
}

</storycode:copy>
<a id="f-c123076e88e3-location_links_translate_target_ranges"></a>
### `location_links_translate_target_ranges`

<storycode:copy id="f-c123076e88e3-location_links_translate_target_ranges">
#[test]
fn location_links_translate_target_ranges() {
    let map = PositionMap::build(&[seg("fn a() {}\n", 5, 2)]);
    let index = index_with("file:///tmp/vf/main.rs", "hick:///doc.md", map);

    let child_result = serde_json::json!([
        { "targetUri": "file:///tmp/vf/main.rs",
          "targetRange": { "start": { "line": 0, "character": 0 },
                           "end":   { "line": 0, "character": 9 } },
          "targetSelectionRange": { "start": { "line": 0, "character": 3 },
                                    "end":   { "line": 0, "character": 4 } } },
    ]);

    let translated = translate_locations(child_result, &index);
    assert_eq!(
        translated[0]["targetUri"].as_str().unwrap(),
        "hick:///doc.md"
    );
    assert_eq!(translated[0]["targetRange"]["start"]["line"], 4);
    assert_eq!(translated[0]["targetRange"]["start"]["character"], 2);
    assert_eq!(
        translated[0]["targetSelectionRange"]["start"]["character"],
        5
    );
}

</storycode:copy>
<a id="f-c123076e88e3-bare_ranges_translate_or_drop"></a>
### `bare_ranges_translate_or_drop`

<storycode:copy id="f-c123076e88e3-bare_ranges_translate_or_drop">
#[test]
fn bare_ranges_translate_or_drop() {
    let map = PositionMap::build(&[seg("hello\n", 4, 2)]);
    let hover = serde_json::json!({
        "contents": { "kind": "plaintext", "value": "info" },
        "range": { "start": { "line": 0, "character": 0 },
                   "end":   { "line": 0, "character": 5 } },
    });
    let translated = translate_bare_ranges(hover, &map);
    assert_eq!(translated["range"]["start"]["line"], 3);
    assert_eq!(translated["range"]["start"]["character"], 2);

    let unmappable = serde_json::json!({
        "contents": "x",
        "range": { "start": { "line": 9, "character": 0 },
                   "end":   { "line": 9, "character": 1 } },
    });
    let translated = translate_bare_ranges(unmappable, &map);
    assert!(translated.get("range").is_none());
    assert_eq!(translated["contents"], "x");
}

</storycode:copy>
<a id="f-c123076e88e3-an_inlay_hints_bare_position_is_translated_too"></a>
### `an_inlay_hints_bare_position_is_translated_too`

<storycode:copy id="f-c123076e88e3-an_inlay_hints_bare_position_is_translated_too">
#[test]
fn an_inlay_hints_bare_position_is_translated_too() {
    // An inlay hint carries a POSITION, not a range, and the walker used
    // to step straight over it — leaving the hint at the virtual file's
    // line number, which is near the top of the document instead of
    // beside the code it describes.
    let map = PositionMap::build(&[seg("def load(path):\n", 6, 0)]);
    let hint = serde_json::json!([{
        "label": "-> str",
        "position": { "line": 0, "character": 14 },
    }]);
    let translated = translate_bare_ranges(hint, &map);
    assert_eq!(translated[0]["position"]["line"], 5, "{translated}");
    assert_eq!(translated[0]["position"]["character"], 14);
}

</storycode:copy>
<a id="f-c123076e88e3-a_hint_with_no_source_is_dropped_rather_than_placed_at_line_zero"></a>
### `a_hint_with_no_source_is_dropped_rather_than_placed_at_line_zero`

<storycode:copy id="f-c123076e88e3-a_hint_with_no_source_is_dropped_rather_than_placed_at_line_zero">
#[test]
fn a_hint_with_no_source_is_dropped_rather_than_placed_at_line_zero() {
    let map = PositionMap::build(&[seg("x\n", 2, 0)]);
    let hint =
        serde_json::json!([{ "label": ": int", "position": { "line": 40, "character": 0 } }]);
    let translated = translate_bare_ranges(hint, &map);
    assert!(translated[0].get("position").is_none(), "{translated}");
}

</storycode:copy>
<a id="f-c123076e88e3-a_position_that_is_not_a_position_is_left_alone"></a>
### `a_position_that_is_not_a_position_is_left_alone`

<storycode:copy id="f-c123076e88e3-a_position_that_is_not_a_position_is_left_alone">
#[test]
fn a_position_that_is_not_a_position_is_left_alone() {
    // Only the key `position` with a line and a character is treated as
    // one; a server's own `position` field of some other shape must
    // survive untouched.
    let map = PositionMap::build(&[seg("x\n", 2, 0)]);
    let value = serde_json::json!({ "position": "after", "data": { "position": 3 } });
    let translated = translate_bare_ranges(value, &map);
    assert_eq!(translated["position"], "after");
    assert_eq!(translated["data"]["position"], 3);
}

</storycode:copy>
<a id="f-c123076e88e3-a_virtual_files_extent_is_its_line_count"></a>
### `a_virtual_files_extent_is_its_line_count`

<storycode:copy id="f-c123076e88e3-a_virtual_files_extent_is_its_line_count">
#[test]
fn a_virtual_files_extent_is_its_line_count() {
    // What the inlay-hint request sends as its range. A child asked
    // about the DOCUMENT's range is asked about lines its file does not
    // have, and answers nothing.
    let map = PositionMap::build(&[seg("a\nb\nc\n", 3, 0)]);
    assert_eq!(map.virtual_lines(), 3);
}

</storycode:copy>
<a id="f-c123076e88e3-a_formatting_edit_gets_the_blocks_indentation_back"></a>
### `a_formatting_edit_gets_the_blocks_indentation_back`

<storycode:copy id="f-c123076e88e3-a_formatting_edit_gets_the_blocks_indentation_back">
#[test]
fn a_formatting_edit_gets_the_blocks_indentation_back() {
    // The block sits four columns in; the formatter saw it dedented and
    // answers in those coordinates, with no indentation in its text.
    let map = PositionMap::build(&[seg("fn a(){\n}\n", 10, 4)]);
    let edits = serde_json::json!([{
        "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 2, "character": 0 } },
        "newText": "fn a() {\n    x\n}\n",
    }]);
    let out = translate_edits(&edits, &map).expect("mapped");
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].range.start, Position::new(9, 4));
    // One past the block's last line, column 0: the closing tag's line.
    assert_eq!(out[0].range.end, Position::new(11, 0));
    // Every inserted line indented; the trailing newline left bare, since
    // the closing tag's own indentation is outside the range.
    assert_eq!(out[0].new_text, "fn a() {\n        x\n    }\n");
}

</storycode:copy>
<a id="f-c123076e88e3-a_plain_files_formatting_edit_is_untouched"></a>
### `a_plain_files_formatting_edit_is_untouched`

<storycode:copy id="f-c123076e88e3-a_plain_files_formatting_edit_is_untouched">
#[test]
fn a_plain_files_formatting_edit_is_untouched() {
    let map = PositionMap::identity(3);
    let edits = serde_json::json!([{
        "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 3, "character": 0 } },
        "newText": "a\nb\n",
    }]);
    let out = translate_edits(&edits, &map).expect("mapped");
    assert_eq!(out[0].new_text, "a\nb\n");
    assert_eq!(out[0].range.end, Position::new(3, 0));
}

</storycode:copy>
<a id="f-c123076e88e3-an_edit_that_cannot_be_mapped_withholds_the_whole_answer"></a>
### `an_edit_that_cannot_be_mapped_withholds_the_whole_answer`

<storycode:copy id="f-c123076e88e3-an_edit_that_cannot_be_mapped_withholds_the_whole_answer">
#[test]
fn an_edit_that_cannot_be_mapped_withholds_the_whole_answer() {
    // Half a block formatted is corruption with a formatter's name on it.
    let map = PositionMap::build(&[seg("x\n", 2, 0)]);
    let edits = serde_json::json!([
        { "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 1 } }, "newText": "y" },
        { "range": { "start": { "line": 7, "character": 0 }, "end": { "line": 7, "character": 1 } }, "newText": "z" },
    ]);
    assert!(translate_edits(&edits, &map).is_none());
}

</storycode:copy>
<a id="f-c123076e88e3-a_markdown_note_with_hick_elements_uses_the_hick_pipeline"></a>
### `a_markdown_note_with_hick_elements_uses_the_hick_pipeline`

Check the routing boundary with ordinary Markdown, the default prefix, an XML namespace alias and a frontmatter alias. This prevents a publication containing rebound literate tags from silently being analysed as prose.

<storycode:copy id="f-c123076e88e3-a_markdown_note_with_hick_elements_uses_the_hick_pipeline">
#[test]
fn a_markdown_note_with_hick_elements_uses_the_hick_pipeline() {
    let uri = Url::parse("file:///notes/today.md").unwrap();
    assert!(is_hick_document(&uri, "<hick:paste from=\"#test1\" />"));
    assert!(!is_hick_document(&uri, "# ordinary Markdown\n"));
    assert!(is_hick_document(
        &uri,
        "<book:doc xmlns:book=\"http://www.hickorydocs.com/1.0\"><book:file path=\"a.ts\">const count = 1;</book:file></book:doc>"
    ));
    assert!(is_hick_document(
        &uri,
        "---\nprefix: code\n---\n<code:file path=\"a.py\">pass</code:file>"
    ));
}

</storycode:copy>
<a id="f-c123076e88e3-unchanged_plain_sync_keeps_real_clangd_quickfix_and_changed_source_invalidates"></a>
### `unchanged_plain_sync_keeps_real_clangd_quickfix_and_changed_source_invalidates`

Real clangd deduplicates diagnostics for byte-identical changes. The
retained payload must still drive a quick fix after a pane resynchronizes.

Drive the public document-processing path for both unchanged and changed plain text. The child must retain its open document and same-source diagnostic identity across resync, then invalidate the old quick-fix context when bytes change. Calling process_file_at directly would miss an erroneous didClose in its caller.

<storycode:copy id="f-c123076e88e3-unchanged_plain_sync_keeps_real_clangd_quickfix_and_changed_source_invalidates">
/// Real clangd deduplicates diagnostics for byte-identical changes. The
/// retained payload must still drive a quick fix after a pane resynchronizes.
#[tokio::test]
#[ignore = "requires a real clangd on PATH; run this focused provider integration explicitly"]
async fn unchanged_plain_sync_keeps_real_clangd_quickfix_and_changed_source_invalidates() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("main.cpp");
    let source =
        "int square(int value) { return value * value; }\nint main() { return square(3) }\n";
    std::fs::write(&path, source).unwrap();
    let uri = Url::from_file_path(&path).unwrap();
    let (service, _socket) = tower_lsp::LspService::new(HickBackend::new);
    let backend = service.inner();
    backend.spawn_notification_handler();
    backend.process_document(&uri, source).await;
    tokio::time::timeout(std::time::Duration::from_secs(15), async {
        loop {
            if backend
                .vfile_index
                .read()
                .await
                .get(&uri)
                .is_some_and(|mapping| !mapping.provider_diagnostics.is_empty())
            {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("real clangd must publish the missing-semicolon diagnostic");
    let before = backend.vfile_index.read().await[&uri]
        .provider_diagnostics
        .clone();
    backend.process_document(&uri, source).await;
    assert_eq!(
        backend.vfile_index.read().await[&uri].provider_diagnostics,
        before
    );
    let params: CodeActionParams = serde_json::from_value(serde_json::json!({
        "textDocument":{"uri":uri},
        "range":{"start":{"line":1,"character":27},"end":{"line":1,"character":27}},
        "context":{"diagnostics":[]}
    }))
    .unwrap();
    let actions = backend.code_action(params).await.unwrap().unwrap();
    assert!(
        actions.iter().any(|action| match action {
            CodeActionOrCommand::CodeAction(action) => action.title.contains("insert ';'"),
            _ => false,
        }),
        "same-source resync must retain the real provider fix: {actions:?}"
    );
    let changed = source.replace("square(3) }", "square(3); }");
    backend.process_document(&uri, &changed).await;
    assert!(
        backend.vfile_index.read().await[&uri]
            .provider_diagnostics
            .is_empty()
    );
    backend.dispatcher.lock().await.shutdown_all().await;
}
</storycode:copy>

### Assemble `crates/hick-lsp/src/backend/audit_tests.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

## Keep or refuse a complete edit

The ordered resource witness keeps create, text, move, text and delete in their
original sequence, drops virtual versions, preserves review annotations and
refuses external or generated whole-file targets. A disabled resource action
must not remove a usable sibling. This checks projection only; engine publication
and frontend receipt adoption have separate focused witnesses.

These focused witnesses distinguish a whole rename from independently available code actions. They require both source targets, exact range projection for each map, refusal of an unmapped target and resource operations, preservation of opaque resolve tokens, and retirement of virtual version counters. No test applies edits to disk.

<storycode:copy id="source-edit-complete-proposal-tests">
#[test]
fn source_edit_proposals_keep_all_targets_or_refuse() {
    use serde_json::json;
    let one = "file:///stage/one.py";
    let two = "file:///stage/two.py";
    let mut index = index_with(
        one,
        "hick:///one.md",
        PositionMap::build(&[seg("name\n", 4, 0)]),
    );
    index.extend(index_with(
        two,
        "hick:///two.md",
        PositionMap::build(&[seg("name\n", 9, 0)]),
    ));
    let edit = json!({"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":4}},"newText":"changed"});
    let result = translate_edit_uris(
        json!({"changes":{one:[edit.clone()],two:[edit.clone()]}}),
        &index,
    )
    .unwrap();
    let changes = result["documentChanges"].as_array().unwrap();
    assert_eq!(changes.len(), 2);
    assert_eq!(changes[0]["textDocument"]["uri"], "hick:///one.md");
    assert_eq!(changes[0]["edits"][0]["range"]["start"]["line"], 3);
    assert_eq!(changes[1]["edits"][0]["range"]["start"]["line"], 8);
    assert!(
        translate_edit_uris(
            json!({"changes":{one:[edit.clone()],"file:///unmapped.py":[edit.clone()]}}),
            &index
        )
        .unwrap_err()
        .contains("Cannot map all edit targets")
    );
    let versioned = translate_edit_uris(json!({"documentChanges":[{"textDocument":{"uri":one,"version":99},"edits":[edit.clone()]}]}), &index).unwrap();
    assert!(versioned["documentChanges"][0]["textDocument"]["version"].is_null());
    assert!(
        translate_edit_uris(
            json!({"documentChanges":[{"kind":"rename","oldUri":one,"newUri":two}]}),
            &index
        )
        .is_err()
    );
    assert!(translate_edit_uris(json!({"changes":{one:[edit.clone(),{"range":{"start":{"line":0,"character":1},"end":{"line":0,"character":3}},"newText":"conflict"}]}}), &index).is_err());
    let malformed = json!({"changes":{one:[{"range":{"start":{"line":4294967296u64,"character":0},"end":{"line":4294967296u64,"character":1}},"newText":"bad"}]}});
    assert!(translate_edit_uris(malformed, &index).is_err());
    let root = Url::parse("file:///workspace/").unwrap();
    let plain = index_with(
        "file:///workspace/a.py",
        "hick:///a.py",
        PositionMap::identity(4),
    );
    let ordered = json!({"documentChanges":[
        {"kind":"create","uri":"file:///workspace/new.py","annotationId":"create"},
        {"textDocument":{"uri":"file:///workspace/new.py","version":42},"edits":[edit.clone()]},
        {"kind":"rename","oldUri":"file:///workspace/a.py","newUri":"file:///workspace/moved.py"},
        {"textDocument":{"uri":"file:///workspace/moved.py","version":99},"edits":[edit.clone()]},
        {"kind":"delete","uri":"file:///workspace/new.py"}
    ],"changeAnnotations":{"create":{"label":"New source"}}});
    let projected = translate_provider_edits(ordered.clone(), &plain, Some(&root)).unwrap();
    let rows = projected["documentChanges"].as_array().unwrap();
    assert_eq!(rows.len(), 5);
    assert_eq!(rows[0]["uri"], "hick:///new.py");
    assert_eq!(rows[0]["annotationId"], "create");
    assert_eq!(rows[1]["textDocument"]["uri"], "hick:///new.py");
    assert!(rows[1]["textDocument"]["version"].is_null());
    assert_eq!(rows[2]["oldUri"], "hick:///a.py");
    assert_eq!(rows[2]["newUri"], "hick:///moved.py");
    assert_eq!(rows[3]["textDocument"]["uri"], "hick:///moved.py");
    assert_eq!(rows[4]["uri"], "hick:///new.py");
    assert_eq!(projected["changeAnnotations"], ordered["changeAnnotations"]);
    assert!(translate_provider_edits(ordered, &plain, None).is_err());
    for uri in [
        "file:///outside.py",
        "file:///workspace/",
        "https://example.com/source",
        "file:///workspace/a.py?other",
    ] {
        assert!(
            translate_provider_edits(
                json!({"documentChanges":[{"kind":"create","uri":uri}]}),
                &plain,
                Some(&root)
            )
            .is_err()
        );
    }
    // Generated fragments cannot gain whole-file ownership; siblings remain usable.
    let actions = json!([{"title":"bad","edit":{"documentChanges":[{"kind":"delete","uri":one}]}},{"title":"good","edit":{"documentChanges":[{"kind":"create","uri":"file:///workspace/new.py"}]}}]);
    let projected = translate_provider_edits(actions, &index, Some(&root)).unwrap();
    assert!(projected[0].get("edit").is_none());
    assert!(
        projected[0]["disabled"]["reason"]
            .as_str()
            .unwrap()
            .contains("generated")
    );
    assert_eq!(
        projected[1]["edit"]["documentChanges"][0]["uri"],
        "hick:///new.py"
    );
    let literal = Url::from_file_path("/workspace/%2F λ?#.py").unwrap();
    let projected = translate_provider_edits(
        json!({"documentChanges":[{"kind":"create","uri":literal}]}),
        &plain,
        Some(&root),
    )
    .unwrap();
    assert_eq!(
        projected["documentChanges"][0]["uri"],
        "hick:///%252F%20%CE%BB%3F%23.py"
    );
    let wire = index_with(
        "file:///workspace/a.py",
        "file:///workspace/a.py",
        PositionMap::identity(4),
    );
    let projected = translate_provider_edits(
        json!({"documentChanges":[{"kind":"rename","oldUri":"file:///workspace/a.py","newUri":"file:///workspace/moved.py"}]}),
        &wire,
        Some(&root),
    ).unwrap();
    assert_eq!(projected["documentChanges"][0]["oldUri"], "hick:///a.py");
}
#[test]
fn source_edit_proposals_disable_only_incomplete_actions_and_preserve_data() {
    use serde_json::json;
    let uri = "file:///stage/a.py";
    let index = index_with(uri, "hick:///a.py", PositionMap::identity(1));
    let edit = json!({"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":1}},"newText":"x"});
    let data = json!({"uri":uri,"changes":{uri:[edit.clone()]}});
    let command = json!({"title":"Finish","command":"provider.finish","arguments":[data.clone()]});
    let actions = json!([
        {"title":"Complete","edit":{"changes":{uri:[edit.clone()]}},"data":data.clone(),"command":command.clone()},
        {"title":"Incomplete","edit":{"changes":{uri:[edit.clone()],"file:///unknown.py":[edit]}},"command":command.clone()},
        {"title":"Opaque command","command":command.clone()}
    ]);
    let answer = translate_edit_uris(actions, &index).unwrap();
    assert_eq!(answer[0]["data"], data);
    assert_eq!(answer[0]["command"], command);
    assert_eq!(
        answer[0]["edit"]["documentChanges"][0]["textDocument"]["uri"],
        "hick:///a.py"
    );
    assert!(answer[1].get("edit").is_none());
    assert!(answer[1].get("command").is_none());
    assert!(
        answer[1]["disabled"]["reason"]
            .as_str()
            .unwrap()
            .contains("Cannot map all edit targets")
    );
    assert_eq!(answer[2]["command"], command);
}
</storycode:copy>

## Project edits follow exact compiler origins

Project copies may contribute different authored documents to one generated file. Verify those identities survive edit projection, repeated copies produce one authored replacement, and a range spanning contributors is refused. A separate Unicode witness refuses a half-surrogate coordinate; a segment-map witness refuses a range that would swallow unconstructed prose.

<storycode:copy id="source-edit-project-provenance-tests">
#[test]
fn source_edit_proposals_follow_project_origins_and_deduplicate_copies() {
    use serde_json::json;
    let mut entry =
        hick_lang::ProjectEntry::file(std::sync::Arc::<[u8]>::from(&b"one\none\ntwo\n"[..]));
    entry.origins = vec![
        hick_lang::ByteOrigin {
            output: (0, 4),
            document: "a.md".into(),
            source: (7, 11),
        },
        hick_lang::ByteOrigin {
            output: (4, 8),
            document: "a.md".into(),
            source: (7, 11),
        },
        hick_lang::ByteOrigin {
            output: (8, 12),
            document: "b.md".into(),
            source: (0, 4),
        },
    ];
    let sources = std::sync::Arc::new(std::collections::BTreeMap::from([
        ("a.md".into(), "Header\none\n".into()),
        ("b.md".into(), "two\n".into()),
    ]));
    let map = PositionMap::for_project(&entry, sources, "a.md").unwrap();
    let uri = "file:///stage/project.py";
    let index = index_with(uri, "hick:///project.md", map);
    let replacement = |line| json!({"range":{"start":{"line":line,"character":0},"end":{"line":line,"character":3}},"newText":"new"});
    let answer = translate_edit_uris(
        json!({"changes":{uri:[replacement(0),replacement(1),replacement(2)]}}),
        &index,
    )
    .unwrap();
    let changes = answer["documentChanges"].as_array().unwrap();
    assert_eq!(changes.len(), 2);
    assert_eq!(changes[0]["textDocument"]["uri"], "hick:///a.md");
    assert_eq!(changes[0]["edits"].as_array().unwrap().len(), 1);
    assert_eq!(changes[0]["edits"][0]["range"]["start"]["line"], 1);
    assert_eq!(changes[1]["textDocument"]["uri"], "hick:///b.md");
    let cross = json!({"changes":{uri:[{"range":{"start":{"line":1,"character":0},"end":{"line":2,"character":3}},"newText":"lost contributor"}]}});
    assert!(
        translate_edit_uris(cross, &index)
            .unwrap_err()
            .contains("crosses source fragments")
    );
}
#[test]
fn source_edit_ranges_require_contiguous_scalar_aligned_source() {
    let legacy = PositionMap::build(&[seg("one\n", 2, 0), seg("two\n", 12, 0)]);
    assert!(
        legacy
            .source_edit_range(Range::new(Position::new(0, 0), Position::new(1, 3)))
            .is_none()
    );
    let contiguous = PositionMap::build(&[seg("one\ntwo\n", 2, 0)]);
    assert_eq!(
        contiguous
            .source_edit_range(Range::new(Position::new(0, 0), Position::new(2, 0)))
            .unwrap()
            .1,
        Range::new(Position::new(1, 0), Position::new(3, 0))
    );
    let text = "é😀 old";
    let mut entry = hick_lang::ProjectEntry::file(std::sync::Arc::<[u8]>::from(text.as_bytes()));
    entry.origins = vec![hick_lang::ByteOrigin {
        output: (0, text.len()),
        document: "code.md".into(),
        source: (5, 5 + text.len()),
    }];
    let sources = std::sync::Arc::new(std::collections::BTreeMap::from([(
        "code.md".into(),
        format!("note\n{text}"),
    )]));
    let map = PositionMap::for_project(&entry, sources, "code.md").unwrap();
    assert_eq!(
        map.source_edit_range(Range::new(Position::new(0, 4), Position::new(0, 7)))
            .unwrap(),
        (
            Some("code.md".into()),
            Range::new(Position::new(1, 4), Position::new(1, 7))
        )
    );
    assert!(
        map.source_edit_range(Range::new(Position::new(0, 2), Position::new(0, 7)))
            .is_none()
    );
}
</storycode:copy>

## Reject obsolete reports without losing readable unknown messages

These focused notification-path witnesses inspect the merged diagnostic store after real bridge translation. A current child counter earns the exact authored digest; an obsolete counter cannot replace it; an unversioned message stays readable with null evidence and is not reused as code-action context. A compiler project witness requires different contributors to carry their own hashes. Provider-supplied lookalike metadata stays nested and cannot become engine evidence.

<storycode:copy id="diagnostic-engine-revision-tests">
#[tokio::test]
async fn diagnostic_revisions_bind_child_versions_and_preserve_unknown_messages() {
    use serde_json::json;
    let (service, _socket) = tower_lsp::LspService::new(HickBackend::new);
    let backend = service.inner();
    let child = Url::parse("file:///stage/problem.py").unwrap();
    let authored = Url::parse("file:///workspace/problem.py").unwrap();
    let source = "amount: int = \"wrong\"\n";
    let hash = diagnostic_source_digest(source);
    backend.vfile_index.write().await.insert(
        child.clone(),
        VFileMapping {
            source_revision: crate::source_identity::SourceRevision::fresh(),
            hick_uri: authored.clone(),
            language_id: "python".into(),
            root_uri: "file:///workspace/".into(),
            position_map: PositionMap::identity(2),
            provider_diagnostics: vec![],
            provider_source: source.into(),
            provider_version: Some(7),
            primary_source_hash: hash.clone(),
        },
    );
    let range = json!({"start":{"line":0,"character":0},"end":{"line":0,"character":6}});
    let payload = json!({"hickory":{"source_hash":"forged"},"token":42});
    let mut notice = ChildNotification {
        language_id: "python".into(),
        method: "textDocument/publishDiagnostics".into(),
        params: json!({"uri":child,"version":7,"diagnostics":[{"range":range,"message":"current report","data":payload}]}),
    };
    handle_child_diagnostics(
        &backend.client,
        &backend.vfile_index,
        &backend.child_diagnostics,
        &notice,
    )
    .await
    .unwrap();
    let current = backend.child_diagnostics.read().await[&authored][&child][0].clone();
    assert_eq!(
        current.data.as_ref().unwrap()["hickory"]["source_hash"],
        hash
    );
    assert_eq!(current.data.as_ref().unwrap()["provider"]["token"], 42);
    assert_eq!(
        current.data.as_ref().unwrap()["provider"]["hickory"]["source_hash"],
        "forged"
    );
    assert_eq!(
        backend.vfile_index.read().await[&child]
            .provider_diagnostics
            .len(),
        1
    );
    notice.params["version"] = json!(6);
    notice.params["diagnostics"][0]["message"] = json!("obsolete report");
    handle_child_diagnostics(
        &backend.client,
        &backend.vfile_index,
        &backend.child_diagnostics,
        &notice,
    )
    .await
    .unwrap();
    assert_eq!(
        backend.child_diagnostics.read().await[&authored][&child][0].message,
        "current report"
    );
    notice.params.as_object_mut().unwrap().remove("version");
    notice.params["diagnostics"][0]["message"] = json!("unversioned report");
    handle_child_diagnostics(
        &backend.client,
        &backend.vfile_index,
        &backend.child_diagnostics,
        &notice,
    )
    .await
    .unwrap();
    let unknown = backend.child_diagnostics.read().await[&authored][&child][0].clone();
    assert_eq!(unknown.message, "unversioned report");
    assert!(unknown.data.unwrap()["hickory"]["source_hash"].is_null());
    assert!(
        backend.vfile_index.read().await[&child]
            .provider_diagnostics
            .is_empty()
    );
}
#[tokio::test]
async fn diagnostic_revisions_follow_each_compiler_contributor() {
    use serde_json::json;
    let (service, _socket) = tower_lsp::LspService::new(HickBackend::new);
    let backend = service.inner();
    let mut entry = hick_lang::ProjectEntry::file(std::sync::Arc::<[u8]>::from(&b"one\ntwo\n"[..]));
    entry.origins = vec![
        hick_lang::ByteOrigin {
            output: (0, 4),
            document: "a.md".into(),
            source: (7, 11),
        },
        hick_lang::ByteOrigin {
            output: (4, 8),
            document: "b.md".into(),
            source: (0, 4),
        },
    ];
    let sources = std::sync::Arc::new(std::collections::BTreeMap::from([
        ("a.md".into(), "Header\none\n".into()),
        ("b.md".into(), "two\n".into()),
    ]));
    let map = PositionMap::for_project(&entry, sources, "a.md").unwrap();
    let child = Url::parse("file:///stage/project.py").unwrap();
    let primary = Url::parse("file:///workspace/project.md").unwrap();
    backend.vfile_index.write().await.insert(
        child.clone(),
        VFileMapping {
            source_revision: crate::source_identity::SourceRevision::fresh(),
            hick_uri: primary,
            language_id: "python".into(),
            root_uri: "file:///workspace/".into(),
            position_map: map,
            provider_diagnostics: vec![],
            provider_source: "one\ntwo\n".into(),
            provider_version: Some(3),
            primary_source_hash: "unused primary digest".into(),
        },
    );
    let diagnostic = |line| json!({"range":{"start":{"line":line,"character":0},"end":{"line":line,"character":3}},"message":"provider report"});
    let notice = ChildNotification {
        language_id: "python".into(),
        method: "textDocument/publishDiagnostics".into(),
        params: json!({"uri":child,"version":3,"diagnostics":[diagnostic(0),diagnostic(1)]}),
    };
    handle_child_diagnostics(
        &backend.client,
        &backend.vfile_index,
        &backend.child_diagnostics,
        &notice,
    )
    .await
    .unwrap();
    let store = backend.child_diagnostics.read().await;
    for (path, source, line) in [("a.md", "Header\none\n", 1), ("b.md", "two\n", 0)] {
        let uri = Url::parse(&format!("file:///workspace/{path}")).unwrap();
        let report = &store[&uri][&child][0];
        assert_eq!(report.range.start.line, line);
        assert_eq!(
            report.data.as_ref().unwrap()["hickory"]["source_hash"],
            diagnostic_source_digest(source)
        );
    }
}
</storycode:copy>

<storycode:file path="crates/hick-lsp/src/backend/audit_tests.rs"><storycode:paste select="#f-c123076e88e3-provider_rename_refusal_is_an_error_instead_of_no_changes"/><storycode:paste select="#f-c123076e88e3-code_actions_keep_exact_child_diagnostic_identity_and_coordinates"/><storycode:paste select="#f-c123076e88e3-seg"/><storycode:paste select="#f-c123076e88e3-index_with"/><storycode:paste select="#f-c123076e88e3-a_child_that_spells_a_uri_differently_still_maps_home"/><storycode:paste select="#f-c123076e88e3-references_result_translates_to_hick_coordinates"/><storycode:paste select="#f-c123076e88e3-references_in_unknown_vfile_pass_through"/><storycode:paste select="#f-c123076e88e3-references_on_untranslatable_lines_keep_virtual_location"/><storycode:paste select="#f-c123076e88e3-location_links_translate_target_ranges"/><storycode:paste select="#f-c123076e88e3-bare_ranges_translate_or_drop"/><storycode:paste select="#f-c123076e88e3-an_inlay_hints_bare_position_is_translated_too"/><storycode:paste select="#f-c123076e88e3-a_hint_with_no_source_is_dropped_rather_than_placed_at_line_zero"/><storycode:paste select="#f-c123076e88e3-a_position_that_is_not_a_position_is_left_alone"/><storycode:paste select="#f-c123076e88e3-a_virtual_files_extent_is_its_line_count"/><storycode:paste select="#f-c123076e88e3-a_formatting_edit_gets_the_blocks_indentation_back"/><storycode:paste select="#f-c123076e88e3-a_plain_files_formatting_edit_is_untouched"/><storycode:paste select="#f-c123076e88e3-an_edit_that_cannot_be_mapped_withholds_the_whole_answer"/><storycode:paste select="#f-c123076e88e3-a_markdown_note_with_hick_elements_uses_the_hick_pipeline"/><storycode:paste select="#f-c123076e88e3-unchanged_plain_sync_keeps_real_clangd_quickfix_and_changed_source_invalidates"/><storycode:paste select="#source-edit-complete-proposal-tests"/><storycode:paste select="#source-edit-project-provenance-tests"/><storycode:paste select="#diagnostic-engine-revision-tests"/></storycode:file>

</storycode:doc>
