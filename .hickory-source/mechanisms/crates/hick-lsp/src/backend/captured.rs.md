<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-lsp/src/backend/captured.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-db705f2dcba7"></a>
## `crates/hick-lsp/src/backend/captured.rs`

<a id="f-db705f2dcba7-impl-HickBackend"></a>
### `impl-HickBackend`

Native staging for immutable comparison values.

<storycode:copy id="f-db705f2dcba7-impl-HickBackend">
//! Native staging for immutable comparison values.
use super::*;
impl HickBackend {
    pub(super) async fn process_captured_file(&self, uri: &Url, source: &str) {
        let result = (|| -> std::result::Result<_, String> {
            let (id, path, context) = self.projects.get(uri)?;
            let entry = context.tree.get(&path).ok_or("Missing captured file")?;
            if entry.kind != hick_lang::EntryKind::File || entry.bytes.as_ref() != source.as_bytes()
            {
                return Err("Captured project changed; regenerate the comparison before asking native tools".into());
            }
            let directory = context.materialize()?;
            let child = Url::from_file_path(directory.join(&path))
                .map_err(|_| "Native project needs a file URI")?;
            let root = Url::from_directory_path(&directory)
                .map_err(|_| "Native project needs a root URI")?
                .to_string();
            Ok((id, path, context, directory, child, root))
        })();
        let (id, path, context, directory, child, root) = match result {
            Ok(v) => v,
            Err(e) => {
                self.client.log_message(MessageType::ERROR, e).await;
                return;
            }
        };
        let mut index = self.vfile_index.write().await;
        for (path, entry) in context.tree.entries() {
            if entry.kind != hick_lang::EntryKind::File {
                continue;
            }
            let Ok(text) = std::str::from_utf8(&entry.bytes) else {
                continue;
            };
            let Ok(physical) = Url::from_file_path(directory.join(path)) else {
                continue;
            };
            let Ok(display) = Url::parse(&format!("hick-project:///{id}/{path}")) else {
                continue;
            };
            index.insert(
                physical,
                VFileMapping {
                    provider_diagnostics: Vec::new(),
                    provider_source: text.to_string(),
                    hick_uri: display,
                    language_id: crate::lang_detect::language_id(path)
                        .unwrap_or("plaintext")
                        .into(),
                    root_uri: root.clone(),
                    position_map: PositionMap::identity(text.split('\n').count() as u32),
                },
            );
        }
        drop(index);
        if let Some(language) = crate::lang_detect::language_id(&path)
            && lsp_command(language).is_ok()
        {
            self.process_file_at(uri, &child, source, language, root)
                .await;
        }
    }
}
</storycode:copy>

### Assemble `crates/hick-lsp/src/backend/captured.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-lsp/src/backend/captured.rs"><storycode:paste select="#f-db705f2dcba7-impl-HickBackend"/></storycode:file>

</storycode:doc>
