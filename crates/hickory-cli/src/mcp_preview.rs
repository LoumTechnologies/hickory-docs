//! Read-only edit previews for client-owned review.
use super::*;

impl Server {
    pub(crate) fn set_default_doc(&mut self, doc: PathBuf) {
        self.default_doc = Some(doc);
    }

    pub(crate) async fn preview_edit(
        &mut self,
        name: &str,
        args: &Value,
    ) -> Result<(String, String, String), String> {
        if name == "create_doc" {
            let path = args["path"].as_str().ok_or("Pass a document path.")?;
            let mut path = PathBuf::from(path);
            if path.extension().is_none_or(|e| e != "md") {
                path.set_extension("md");
            }
            if self.root.join(&path).exists() {
                return Err("This document already exists; use edit_doc.".into());
            }
            let input = args["input"].as_str().unwrap_or("");
            let input = if input.is_empty() || input.ends_with('\n') {
                input.into()
            } else {
                format!("{input}\n")
            };
            return Ok((path.display().to_string(), String::new(), input));
        }
        let doc = self.resolve_doc(args)?;
        let inv = Self::invocation(name, args)?;
        self.session_for(&doc).await?.preview_edit(&inv).await
    }
}
