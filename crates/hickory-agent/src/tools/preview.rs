use super::*;

impl EditSession {
    /// Build a complete replacement pinned to the reviewed surface, preserving
    /// the existing tool's source/output routing and validation.
    pub fn reviewed_invocation(
        invocation: &ToolInvocation,
        before: &str,
        after: String,
    ) -> Result<ToolInvocation, String> {
        let mut args: Vec<_> = invocation
            .args
            .iter()
            .filter(|(name, _)| matches!(name.as_str(), "doc" | "path"))
            .cloned()
            .collect();
        args.push(("snapshot".into(), LineIndex::new(before).snapshot));
        args.push(("whole".into(), "true".into()));
        ToolInvocation::synthetic(invocation.name.clone(), args, Some(after))
    }
    /// Revision of the source closure, independent of the agent's account of it.
    pub fn revision(&self) -> String {
        sha256_hex(
            serde_json::to_string(&self.source_snapshots())
                .unwrap()
                .as_bytes(),
        )
    }
    pub fn source_snapshots(&self) -> Vec<serde_json::Value> {
        if self.prediction.is_some() {
            return vec![
                serde_json::json!({"path":self.doc_path.display().to_string(),
                "sha256":sha256_hex(self.source.as_bytes()), "captured":true,
                "current":std::fs::read_to_string(&self.doc_path).ok().as_ref() == Some(&self.source)}),
            ];
        }
        let mut paths = vec![self.doc_path.clone()];
        paths.extend(self.upstream.keys().cloned());
        paths.sort();
        paths
            .into_iter()
            .map(|path| {
                let hash = std::fs::read(&path).ok().map(|bytes| sha256_hex(&bytes));
                serde_json::json!({"path":path.display().to_string(),"sha256":hash})
            })
            .collect()
    }
    pub fn has_executable_cells(&self) -> bool {
        std::iter::once(&self.source)
            .chain(self.upstream.values())
            .any(|source| {
                hick_lang::parse(source)
                    .is_ok_and(|doc| doc.all_tags().iter().any(|tag| tag.name == "exec"))
            })
    }
    pub async fn refresh_source(&mut self) -> Result<bool, String> {
        self.sync_with_disk().await
    }
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
        if self.prediction.is_some() && inv.name != "edit_doc" {
            return Err("Private prediction policy withholds generated output previews".into());
        }
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
