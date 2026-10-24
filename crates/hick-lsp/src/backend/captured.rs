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
