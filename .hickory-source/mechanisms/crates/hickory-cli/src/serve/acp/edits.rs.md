<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/serve/acp/edits.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-076ee922e67a"></a>
## `crates/hickory-cli/src/serve/acp/edits.rs`

Related contracts: [Finish an agent turn against its captured project boundary](../../../../../../guarantees/agent/changeset-completion-is-negotiated-and-checked/contract.md), [Conversation edits use the client's review policy](../../../../../../guarantees/agent/conversation-edits-use-the-client-review-policy/contract.md), [Separate model snippets from trusted host operations](../../../../../../guarantees/agent/python-execution-has-two-boundaries/contract.md).

<a id="f-076ee922e67a-Mode"></a>
### `Mode`

Conversation-owned edit policy. The agent calls the same tools in both modes.

<storycode:copy id="f-076ee922e67a-Mode">
//! Conversation-owned edit policy. The agent calls the same tools in both modes.
use super::super::agent_context::EditorBuffer;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Mutex;
use tokio::sync::oneshot;

#[derive(Clone, Copy, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Mode {
    #[default]
    Review,
    AutoAccept,
}
</storycode:copy>
<a id="f-076ee922e67a-Change"></a>
### `Change`

<storycode:copy id="f-076ee922e67a-Change">
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Change {
    pub id: String,
    pub name: String,
    pub buffer: Option<String>,
    pub path: Option<String>,
    pub old_text: String,
    pub new_text: String,
    pub editor: bool,
    pub status: String,
}
</storycode:copy>
<a id="f-076ee922e67a-Edits"></a>
### `Edits`

<storycode:copy id="f-076ee922e67a-Edits">
#[derive(Default)]
pub struct Edits {
    pub live_context: std::sync::atomic::AtomicBool,
    pub mode: Mutex<Mode>,
    pub buffers: Mutex<Vec<EditorBuffer>>,
    changes: Mutex<Vec<Change>>,
    replies: Mutex<HashMap<String, oneshot::Sender<Result<(), String>>>>,
    revised: Mutex<HashMap<String, String>>,
    refresh: Mutex<Option<(String, oneshot::Sender<Vec<EditorBuffer>>)>>,
}
</storycode:copy>
<a id="f-076ee922e67a-impl-Edits"></a>
### `impl-Edits`

<storycode:copy id="f-076ee922e67a-impl-Edits">
impl Edits {
    pub async fn refresh(&self) -> Result<()> {
        if !self.live_context.load(std::sync::atomic::Ordering::Relaxed)
            && self.buffers.lock().unwrap().is_empty()
        {
            return Ok(());
        }
        let id = format!("{:016x}", super::super::rand_id());
        let (tx, rx) = oneshot::channel();
        *self.refresh.lock().unwrap() = Some((id.clone(), tx));
        let result = tokio::time::timeout(std::time::Duration::from_secs(10), rx).await;
        let mut pending = self.refresh.lock().unwrap();
        if pending.as_ref().is_some_and(|(key, _)| key == &id) {
            pending.take();
        }
        drop(pending);
        let buffers = result.context("Editor capture timed out. Keep the workspace window open and retry changeset_status.")?.context("Editor capture was cancelled; retry changeset_status.")?;
        *self.buffers.lock().unwrap() = buffers;
        Ok(())
    }
    pub fn refreshed(&self, id: &str, buffers: Vec<EditorBuffer>) -> Result<()> {
        let mut pending = self.refresh.lock().unwrap();
        anyhow::ensure!(
            pending.as_ref().is_some_and(|(key, _)| key == id),
            "Expired editor capture request"
        );
        let (_, tx) = pending.take().expect("checked request");
        let _ = tx.send(buffers);
        Ok(())
    }
    pub fn revise(&self, id: &str, replacement: String) -> Result<()> {
        let mut changes = self.changes.lock().unwrap();
        let change = changes
            .iter_mut()
            .find(|c| c.id == id && c.status == "pending")
            .context("This proposal is no longer editable.")?;
        if change.new_text != replacement {
            change.new_text = replacement.clone();
            self.revised.lock().unwrap().insert(id.into(), replacement);
        }
        Ok(())
    }
    pub fn replacement(&self, id: &str) -> Option<String> {
        self.revised.lock().unwrap().get(id).cloned()
    }
    pub fn snapshot(&self) -> Value {
        json!({"mode":*self.mode.lock().unwrap(),"changes":*self.changes.lock().unwrap(),"captureRequest":self.refresh.lock().unwrap().as_ref().map(|(id,_)|id.clone())})
    }
    pub fn buffer(&self, args: &Value) -> Option<EditorBuffer> {
        let buffers = self.buffers.lock().unwrap();
        if let Some(id) = args["buffer"].as_str() {
            return buffers
                .iter()
                .find(|b| b.id.as_deref() == Some(id))
                .cloned();
        }
        if let Some(path) = args["doc"].as_str() {
            return buffers
                .iter()
                .find(|b| b.path.as_deref() == Some(path))
                .cloned();
        }
        buffers.iter().find(|b| b.focused).cloned()
    }
    pub async fn submit(
        &self,
        name: String,
        path: Option<String>,
        old_text: String,
        new_text: String,
        editor: bool,
    ) -> Result<String> {
        self.submit_with_policy(name, path, old_text, new_text, editor, false)
            .await
    }
    /// Mandatory approval remains independent of a conversation's auto-accept policy.
    pub async fn submit_review(
        &self,
        name: String,
        path: Option<String>,
        old_text: String,
        new_text: String,
    ) -> Result<String> {
        self.submit_with_policy(name, path, old_text, new_text, false, true)
            .await
    }
    async fn submit_with_policy(
        &self,
        name: String,
        path: Option<String>,
        old_text: String,
        new_text: String,
        editor: bool,
        force_review: bool,
    ) -> Result<String> {
        let auto = !force_review && *self.mode.lock().unwrap() == Mode::AutoAccept;
        let id = format!("{:016x}", super::super::rand_id());
        let (tx, rx) = oneshot::channel();
        self.replies.lock().unwrap().insert(id.clone(), tx);
        let buffer_id = if editor { Some(name.clone()) } else { None };
        let name = if editor {
            self.buffers
                .lock()
                .unwrap()
                .iter()
                .find(|b| b.id.as_deref() == Some(name.as_str()))
                .map(|b| b.name.clone())
                .unwrap_or(name)
        } else {
            name
        };
        self.changes.lock().unwrap().push(Change {
            id: id.clone(),
            buffer: buffer_id,
            name,
            path,
            old_text,
            new_text,
            editor,
            status: if auto && editor {
                "applying"
            } else {
                "pending"
            }
            .into(),
        });
        if auto && !editor {
            self.resolve(&id, Ok(()))?;
        }
        rx.await
            .context("The edit was cancelled.")?
            .map_err(anyhow::Error::msg)?;
        Ok(id)
    }
    pub fn resolve(&self, id: &str, result: Result<(), String>) -> Result<()> {
        let tx = self
            .replies
            .lock()
            .unwrap()
            .remove(id)
            .context("This edit is no longer pending.")?;
        if let Some(change) = self.changes.lock().unwrap().iter_mut().find(|c| c.id == id) {
            change.status = if result.is_ok() {
                "accepted"
            } else {
                "rejected"
            }
            .into();
        }
        let _ = tx.send(result);
        Ok(())
    }
    pub fn refresh_rejected_buffer(&self, id: &str, content: String) {
        let change = self
            .changes
            .lock()
            .unwrap()
            .iter()
            .find(|c| c.id == id && c.editor && matches!(c.status.as_str(), "pending" | "applying"))
            .cloned();
        if let Some(change) = change {
            let mut buffers = self.buffers.lock().unwrap();
            if let Some(buffer) = buffers
                .iter_mut()
                .find(|b| b.id == change.buffer && b.path == change.path)
            {
                buffer.content = content;
            }
        }
    }
    pub fn finished(&self, id: &str, ok: bool) {
        if let Some(change) = self.changes.lock().unwrap().iter_mut().find(|c| c.id == id) {
            change.status = if ok { "applied" } else { "failed" }.into();
        }
    }
    pub fn cancel(&self) {
        self.refresh.lock().unwrap().take();
        let ids: Vec<_> = self.replies.lock().unwrap().keys().cloned().collect();
        for id in ids {
            let _ = self.resolve(
                &id,
                Err("The turn stopped; this edit was not applied.".into()),
            );
        }
    }
    pub fn updated(&self, buffer: &EditorBuffer, text: String) {
        if let Some(current) = self
            .buffers
            .lock()
            .unwrap()
            .iter_mut()
            .find(|b| b.id == buffer.id && b.path == buffer.path && b.name == buffer.name)
        {
            current.content = text;
        }
    }
}

</storycode:copy>
<a id="f-076ee922e67a-catalogue"></a>
### `catalogue`

<storycode:copy id="f-076ee922e67a-catalogue">
pub fn catalogue() -> Vec<Value> {
    vec![
        json!({"name":"read_buffer","description":"Read a live open editor buffer, including an untitled or unsaved note. Omit buffer for the current focused editor; otherwise pass its id from editor context. Returns exact content and hashline anchors for edit_buffer. This does not save the buffer.","inputSchema":{"type":"object","properties":{"buffer":{"type":"string"}}}}),
        json!({"name":"edit_buffer","description":"Edit the live editor buffer using run/after content hashes from read_buffer. Use this for requested changes to the current document, including unsaved and untitled notes, instead of printing a replacement document in your answer. The client handles approval; wait for the tool result. Does not create a disk path or explicitly save an untitled note.","inputSchema":{"type":"object","properties":{"buffer":{"type":"string"},"snapshot":{"type":"string","description":"Whole-buffer SHA-256 from the latest read"},"run":{"type":"string"},"after":{"type":"string"},"occurrence":{"type":"integer"},"input":{"type":"string"}}}}),
    ]
}

</storycode:copy>
<a id="f-076ee922e67a-replacement"></a>
### `replacement`

Reuse the document tool's hashline resolver and replacement semantics.

<storycode:copy id="f-076ee922e67a-replacement">
/// Reuse the document tool's hashline resolver and replacement semantics.
pub fn replacement(source: &str, args: &Value) -> Result<String> {
    let mut arguments = Vec::new();
    for key in ["run", "after", "occurrence", "snapshot"] {
        if let Some(value) = args.get(key) {
            arguments.push((
                key.to_string(),
                value
                    .as_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| value.to_string()),
            ));
        }
    }
    let inv = hickory_agent::ToolInvocation::synthetic(
        "edit_doc",
        arguments,
        args["input"].as_str().map(str::to_string),
    )
    .map_err(anyhow::Error::msg)?;
    hickory_agent::EditSession::preview_text(source, &inv).map_err(anyhow::Error::msg)
}

</storycode:copy>
<a id="f-076ee922e67a-tests"></a>
### `tests`

<storycode:copy id="f-076ee922e67a-tests">
#[cfg(test)]
mod tests {
    use super::*;
    // Guarantee: docs/guarantees/agent/changeset-completion-is-negotiated-and-checked.md
    #[tokio::test]
    async fn fresh_context_is_token_bound_and_cancellable_even_without_initial_editors() {
        let edits = std::sync::Arc::new(Edits::default());
        edits
            .live_context
            .store(true, std::sync::atomic::Ordering::Relaxed);
        let worker = edits.clone();
        let pending = tokio::spawn(async move { worker.refresh().await });
        let token = loop {
            if let Some(id) = edits.snapshot()["captureRequest"].as_str() {
                break id.to_owned();
            }
            tokio::task::yield_now().await;
        };
        assert!(edits.refreshed("old-turn", vec![]).is_err());
        edits
            .refreshed(
                &token,
                serde_json::from_value(
                    json!([{"id":"new","name":"Untitled","path":null,"content":"fresh"}]),
                )
                .unwrap(),
            )
            .unwrap();
        pending.await.unwrap().unwrap();
        assert_eq!(
            edits.buffer(&json!({"buffer":"new"})).unwrap().content,
            "fresh"
        );
        assert!(edits.refreshed(&token, vec![]).is_err());
        let worker = edits.clone();
        let pending = tokio::spawn(async move { worker.refresh().await });
        while edits.snapshot()["captureRequest"].is_null() {
            tokio::task::yield_now().await;
        }
        edits.cancel();
        assert!(pending.await.unwrap().is_err());
    }
    // Guarantee: docs/guarantees/agent/conversation-edits-use-the-client-review-policy.md
    #[tokio::test]
    async fn a_stale_rejection_refreshes_only_the_target_buffer() {
        let edits = std::sync::Arc::new(Edits::default());
        *edits.buffers.lock().unwrap() = serde_json::from_value(json!([
            {"id":"target","name":"Note","path":null,"content":"before","focused":true},
            {"id":"other","name":"Another","path":null,"content":"untouched","focused":false}
        ]))
        .unwrap();
        let worker = edits.clone();
        let waiting = tokio::spawn(async move {
            worker
                .submit(
                    "target".into(),
                    None,
                    "before".into(),
                    "proposal".into(),
                    true,
                )
                .await
        });
        let id = loop {
            if let Some(change) = edits.changes.lock().unwrap().first() {
                break change.id.clone();
            }
            tokio::task::yield_now().await;
        };
        edits.refresh_rejected_buffer(&id, "typed during review".into());
        edits.resolve(&id, Err("Read again".into())).unwrap();
        assert!(waiting.await.unwrap().is_err());
        assert_eq!(
            edits.buffer(&json!({})).unwrap().content,
            "typed during review"
        );
        assert_eq!(
            edits.buffer(&json!({"buffer":"other"})).unwrap().content,
            "untouched"
        );
    }
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/acp/edits.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/serve/acp/edits.rs"><storycode:paste select="#f-076ee922e67a-Mode"/><storycode:paste select="#f-076ee922e67a-Change"/><storycode:paste select="#f-076ee922e67a-Edits"/><storycode:paste select="#f-076ee922e67a-impl-Edits"/><storycode:paste select="#f-076ee922e67a-catalogue"/><storycode:paste select="#f-076ee922e67a-replacement"/><storycode:paste select="#f-076ee922e67a-tests"/></storycode:file>

</storycode:doc>
