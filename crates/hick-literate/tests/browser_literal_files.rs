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
