<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Verify the immutable scaffold base

For maintainers preserving immutable scaffold captures while editing authored changes.

## fingerprint

One fingerprint covers a sorted set of paths and literal UTF-8 bytes. Length prefixes prevent a path/content boundary from producing the same digest. Reuse the original ingestion format so authentic captures keep their hashes.

The tutorial builds this part in [the construction step](../../../../02-scaffolds.md#immutable-scaffold-fingerprint).


## verify

Verify the base before applying any patch. File names, missing files and content changes alter the digest. Reject nested construction and output metadata that the original text-only capture hash cannot authenticate. Recorded stdout with a cache key is a different kind of ingestion and keeps its existing semantics.

<storycode:copy id="immutable-scaffold-verify">
/// Validate before evaluation or execution, including captures in included docs.
pub fn validate_scaffolds(doc: &HickDocument) -> Result<(), String> {
    for tag in doc.all_tags() {
        if tag.name != "ingested" || tag.get_attribute("key").is_some() {
            continue;
        }
        let mut files = BTreeMap::new();
        for node in &tag.children {
            let file = match node {
                HickNode::Text(text, _) if text.trim().is_empty() => continue,
                HickNode::Tag(file) if file.name == "file" => file,
                _ => {
                    return Err("Immutable scaffold capture must contain only literal files".into());
                }
            };
            if file
                .attributes
                .iter()
                .any(|(name, _)| !matches!(name.as_str(), "path" | "id" | "language"))
            {
                return Err("Immutable scaffold file has unsupported metadata".into());
            }
            let path = file
                .get_attribute("path")
                .ok_or("Scaffold file requires path")?;
            crate::validate_project_path(path)?;
            let mut content = String::new();
            for (i, child) in file.children.iter().enumerate() {
                let HickNode::Text(text, span) = child else {
                    return Err(
                        "Immutable scaffold file cannot contain executable or pasted content"
                            .into(),
                    );
                };
                let text = if i == 0 {
                    crate::strip_opening_break(text, span.as_ref()).0
                } else {
                    text
                };
                content.push_str(text);
            }
            // Ingestion writes file tags at column zero. Do not allow moving
            // a capture to an indentation that changes the produced bytes.
            if file.source_column > 0 && crate::dedent(&content, file.source_column) != content {
                return Err(
                    "Immutable scaffold indentation would change its captured bytes".into(),
                );
            }
            if files.insert(path.to_owned(), content).is_some() {
                return Err(format!("Duplicate immutable scaffold path: {path}"));
            }
        }
        let expected = tag
            .get_attribute("sha256")
            .ok_or("Immutable scaffold capture requires sha256")?;
        if expected != scaffold_fingerprint(&files) {
            return Err(format!(
                "Immutable scaffold hash mismatch at line {}; change an authored patch instead",
                tag.source_line
            ));
        }
    }
    Ok(())
}
</storycode:copy>

## preserve

A source editor must not evade the lock by replacing both the bytes and their hash or deleting the capture. Require every old capture to survive byte-for-byte; surrounding prose, patches and new captures may change.

<storycode:copy id="immutable-scaffold-preserve">
/// Source editors must not rewrite, delete, or re-hash an existing capture.
/// New captures may be introduced by ingestion; patches remain ordinary source.
pub fn preserve_scaffolds(before: &str, after: &str) -> Result<(), String> {
    fn captures(source: &str) -> Vec<String> {
        // Like the editor, allow incomplete syntax elsewhere during typing.
        // A capture that disappears behind a namespace/raw boundary is absent
        // from the candidate list and therefore cannot bypass preservation.
        crate::structure(source)
            .blocks
            .into_iter()
            .filter(|tag| {
                tag.name == "ingested" && !tag.attrs.iter().any(|(name, _)| name == "key")
            })
            .map(|tag| source[tag.from..tag.to].to_owned())
            .collect()
    }
    let mut retained = captures(after);
    for base in captures(before) {
        let Some(at) = retained.iter().position(|candidate| candidate == &base) else {
            return Err("Immutable scaffold output cannot be edited or removed; add an authored patch instead".into());
        };
        retained.remove(at);
    }
    Ok(())
}
</storycode:copy>

## Prove the base is immutable and patches remain editable

Exercise exact bytes (including BOM, CRLF, Unicode and missing final newlines), path changes, metadata and nested-operation refusals. Apply a patch over the verified base, reverse-edit the patch, then prove neither direct base edits nor re-hashing/deleting the capture is accepted.

<storycode:copy id="immutable-scaffold-tests">
#[cfg(test)]
mod tests {
    use super::*;

    fn source(files: &BTreeMap<String, String>) -> String {
        let mut source = format!(
            "<hick:project/>\n<hick:exec id=\"base\" container=\"sdk\">\n<hick:ingested sha256=\"{}\">\n",
            scaffold_fingerprint(files)
        );
        for (path, content) in files {
            source.push_str(&format!(
                "<hick:file path=\"{path}\">\n{content}</hick:file>\n"
            ));
        }
        source.push_str("</hick:ingested>\n</hick:exec>\n");
        source
    }

    #[test]
    fn verifies_exact_scaffold_bytes_paths_and_literal_shape() {
        let files = BTreeMap::from([
            ("a.rs".into(), "\u{feff}hello λ\r\n".into()),
            ("b.rs".into(), "no final newline".into()),
        ]);
        let source = source(&files);
        validate_scaffolds(&crate::parse(&source).unwrap()).unwrap();
        for changed in [
            source.replace('λ', "x"),
            source.replace("a.rs", "c.rs"),
            source.replace("\r\n", "\n"),
            source.replace("no final newline", "no final newline\n"),
            source.replace("path=\"a.rs\"", "path=\"a.rs\" executable=\"true\""),
            source.replace("no final newline", "<hick:paste select=\"#other\"/>"),
            source.replace("path=\"b.rs\"", "path=\"a.rs\""),
            source.replace(
                "</hick:ingested>",
                "<hick:patch>bad</hick:patch></hick:ingested>",
            ),
        ] {
            assert!(
                validate_scaffolds(&crate::parse(&changed).unwrap()).is_err(),
                "accepted {changed}"
            );
        }
        let mut unresolved = crate::parse(&source.replace(
            "no final newline",
            "<hick:include file=\"__must_not_be_read__.md\"/>",
        ))
        .unwrap();
        let error = crate::resolve_includes(
            &mut unresolved,
            std::path::Path::new("."),
            &mut std::collections::HashSet::new(),
        )
        .unwrap_err();
        assert!(error.to_string().contains("Immutable scaffold"));
        let hash = scaffold_fingerprint(&files);
        assert!(
            validate_scaffolds(&crate::parse(&source.replace(&hash, &"0".repeat(64))).unwrap())
                .is_err()
        );
        // Kept stdout uses an input cache key and is outside this file-base check.
        validate_scaffolds(&crate::parse("<hick:exec container=\"c\"><hick:ingested key=\"stdout\">recording</hick:ingested></hick:exec>").unwrap()).unwrap();
    }

    #[test]
    fn patches_apply_above_the_base_and_only_authored_bytes_reverse_edit() {
        let source = source(&BTreeMap::from([("main.rs".into(), "old\n".into())]));
        let patched = format!(
            "{source}<hick:patch id=\"change\" after=\"base\">\n--- a/main.rs\n+++ b/main.rs\n@@ -1 +1 @@\n-old\n+new\n</hick:patch>\n"
        );
        let sources = BTreeMap::from([("project.md".into(), patched.clone())]);
        let tree = crate::evaluate_project("project.md", &sources, &crate::ProjectTree::default())
            .unwrap();
        assert_eq!(&*tree.new.get("main.rs").unwrap().bytes, b"new\n");
        let edited = crate::reverse_project_edit(
            "project.md",
            &sources,
            &crate::ProjectTree::default(),
            &tree.new,
            "main.rs",
            (0, 3),
            "authored",
        )
        .unwrap();
        preserve_scaffolds(&patched, &edited["project.md"]).unwrap();
        let edited =
            crate::evaluate_project("project.md", &edited, &crate::ProjectTree::default()).unwrap();
        assert_eq!(&*edited.new.get("main.rs").unwrap().bytes, b"authored\n");
        let sources = BTreeMap::from([("project.md".into(), source.clone())]);
        let base = crate::evaluate_project("project.md", &sources, &crate::ProjectTree::default())
            .unwrap();
        let error = crate::reverse_project_edit(
            "project.md",
            &sources,
            &crate::ProjectTree::default(),
            &base.new,
            "main.rs",
            (0, 3),
            "wrong",
        )
        .unwrap_err();
        assert!(error.contains("immutable"), "{error}");
        let tampered = BTreeMap::from([("project.md".into(), source.replace("old\n", "bad\n"))]);
        assert!(
            crate::evaluate_project("project.md", &tampered, &crate::ProjectTree::default())
                .unwrap_err()
                .contains("hash mismatch")
        );
    }

    #[test]
    fn source_edits_cannot_delete_or_rehash_an_existing_capture() {
        let source = source(&BTreeMap::from([("main.rs".into(), "old\n".into())]));
        preserve_scaffolds(
            &source,
            &format!("A new explanation.\n{source}\nMore prose."),
        )
        .unwrap();
        let replacement = self::source(&BTreeMap::from([("main.rs".into(), "new\n".into())]));
        validate_scaffolds(&crate::parse(&replacement).unwrap()).unwrap();
        assert!(
            preserve_scaffolds(&source, &replacement)
                .unwrap_err()
                .contains("Immutable")
        );
        assert!(preserve_scaffolds(&source, "no capture").is_err());
    }
}
</storycode:copy>

<storycode:file path="crates/hick-lang/src/scaffold.rs"><storycode:paste select="#immutable-scaffold-fingerprint"/><storycode:paste select="#immutable-scaffold-verify"/><storycode:paste select="#immutable-scaffold-preserve"/><storycode:paste select="#immutable-scaffold-tests"/></storycode:file>
</storycode:doc>
