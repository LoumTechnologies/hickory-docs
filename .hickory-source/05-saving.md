<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# 5. Save source without changing its captured base

Editor guards protect an interaction in one view. A second client or a native
save can still write source, so we must put the same capture rule at the shared
disk boundary. We will validate the proposed source before publishing any bytes.

## Validate, then publish the whole save

Read the previous document, preserve every existing capture and only then call
the atomic writer. A missing file starts empty; another read error must propagate.
The test rehashes a modified capture and proves the previous disk bytes survive,
then saves incomplete authored text around an intact base. Saving a draft does
not require the rest of the document to compile.

<a id="immutable-scaffold-write-document"></a>
<book:copy id="immutable-scaffold-write-document">
pub fn write_document(path: &Path, source: &str) -> Result<()> {
    let previous = match std::fs::read_to_string(path) {
        Ok(previous) => previous,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error.into()),
    };
    hick_lang::preserve_scaffolds(&previous, source).map_err(anyhow::Error::msg)?;
    write_atomic(path, source.as_bytes())
}

#[cfg(test)]
#[test]
fn immutable_scaffold_save_refuses_rehash_without_touching_disk() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("source.md");
    let base =
        "<hick:ingested sha256=\"base\"><hick:file path=\"x\">old</hick:file></hick:ingested>";
    write_document(&path, base).unwrap();
    let tampered = base.replace("old", "new").replace("base", "new-hash");
    assert!(
        write_document(&path, &tampered)
            .unwrap_err()
            .to_string()
            .contains("Immutable")
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), base);
    let with_prose = format!("{base}\n<hick:copy");
    write_document(&path, &with_prose).unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), with_prose);
}
</book:copy>

Both local saves and collaborative persistence must use this boundary. The rest
of the store supplies atomic writing and the shared document-store implementation.
<book:include file="mechanisms/crates/hickory-cli/src/serve/store.rs.md" />

## Wire the store into native serving

Connect editor rooms and saved source through the common store. Keep generation
an explicit project operation: opening a file or observing a watcher event does
not authorize output construction. This lets a user read or edit source without
silently replacing the working program.

<book:include file="mechanisms/crates/hickory-cli/src/serve/mod.rs.md" />

```sh
cargo test --locked -p hickory-cli --lib immutable_scaffold_save
```

We now preserve the construction across both editor actions and disk saves.
Next, [assemble the command-line, browser and desktop application](06-application.md).
</book:doc>
