<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-lsp/src/backend/audit_tests.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `a1cdc9f9aa068e8a14cc6afe1d336eae42892b49ee7642856a34caa1df9e64aa`.

<book:entry path="crates/hick-lsp/src/backend/audit_tests.rs">
use super::*;
use crate::virtual_file::VirtualSegment;
use hick_lang::SourceSpan;

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

fn seg(text: &str, start_line: usize, col: usize) -> VirtualSegment {
    VirtualSegment {
        text: text.to_string(),
        source_span: Some(SourceSpan::new(0, text.len(), start_line, col)),
        source_line: start_line,
        source_column: col,
    }
}

fn index_with(vf: &str, hick: &str, map: PositionMap) -> HashMap<String, (Url, PositionMap)> {
    let mut index = HashMap::new();
    index.insert(uri_key(vf), (Url::parse(hick).unwrap(), map));
    index
}

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

#[test]
fn a_hint_with_no_source_is_dropped_rather_than_placed_at_line_zero() {
    let map = PositionMap::build(&[seg("x\n", 2, 0)]);
    let hint =
        serde_json::json!([{ "label": ": int", "position": { "line": 40, "character": 0 } }]);
    let translated = translate_bare_ranges(hint, &map);
    assert!(translated[0].get("position").is_none(), "{translated}");
}

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

#[test]
fn a_virtual_files_extent_is_its_line_count() {
    // What the inlay-hint request sends as its range. A child asked
    // about the DOCUMENT's range is asked about lines its file does not
    // have, and answers nothing.
    let map = PositionMap::build(&[seg("a\nb\nc\n", 3, 0)]);
    assert_eq!(map.virtual_lines(), 3);
}

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

#[test]
fn a_markdown_note_with_hick_elements_uses_the_hick_pipeline() {
    let uri = Url::parse("file:///notes/today.md").unwrap();
    assert!(is_hick_document(&uri, "<hick:paste from=\"#test1\" />"));
    assert!(!is_hick_document(&uri, "# ordinary Markdown\n"));
}

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
    let root = Url::from_directory_path(directory.path())
        .unwrap()
        .to_string();
    let (service, _socket) = tower_lsp::LspService::new(HickBackend::new);
    let backend = service.inner();
    backend.spawn_notification_handler();
    backend
        .process_file_at(&uri, &uri, source, "cpp", root.clone())
        .await;
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
    backend
        .process_file_at(&uri, &uri, source, "cpp", root.clone())
        .await;
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
    backend
        .process_file_at(&uri, &uri, &changed, "cpp", root)
        .await;
    assert!(
        backend.vfile_index.read().await[&uri]
            .provider_diagnostics
            .is_empty()
    );
    backend.dispatcher.lock().await.shutdown_all().await;
}
</book:entry>

</book:doc>
