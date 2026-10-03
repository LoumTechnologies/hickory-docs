use super::*;

impl EditSession {
    /// Resolve a tool's anchors against exact buffer bytes, without any I/O.
    pub fn preview_text(source: &str, invocation: &ToolInvocation) -> Result<String, String> {
        let edit = resolve_edit(&LineIndex::new(source), invocation, false)?;
        let mut next = source.to_string();
        next.replace_range(edit.start..edit.end, &edit.text);
        Ok(next)
    }

    /// Preview a hashline edit without writing or executing the document.
    pub async fn preview_edit(
        &mut self,
        inv: &ToolInvocation,
    ) -> Result<(String, String, String), String> {
        let resynced = self.sync_with_disk().await?;
        let (path, content) = if inv.name == "edit_output" {
            let path = inv.arg("path").ok_or("Pass an output path.")?;
            (
                path.to_string(),
                self.weave
                    .files
                    .get(path)
                    .cloned()
                    .ok_or("Read the output before editing it.")?,
            )
        } else if let Some(upstream) = inv.arg("doc") {
            let path = self
                .upstream
                .keys()
                .find(|p| {
                    p.to_string_lossy() == upstream || p.file_name().is_some_and(|n| n == upstream)
                })
                .ok_or("Read the upstream document before editing it.")?;
            (
                path.display().to_string(),
                std::fs::read_to_string(path).map_err(|e| e.to_string())?,
            )
        } else {
            (self.doc_name.clone(), self.source.clone())
        };
        let edit = resolve_edit(&LineIndex::new(&content), inv, resynced)?;
        let mut next = content.clone();
        next.replace_range(edit.start..edit.end, &edit.text);
        Ok((path, content, next))
    }
}
