//! Adapter source content is a stopped-frame observation, not workspace authority.
use super::{Failure, Response, observed_scope};
use crate::debug_sessions::Registry;
use serde::Deserialize;
use std::sync::Arc;
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub session: String,
    pub stop: u64,
    pub thread: i64,
    pub frame: i64,
    pub reference: i64,
    pub request_id: String,
}
type References = std::collections::HashMap<(u64, i64, i64), i64>;
pub(super) fn reported(frames: &[hick_dap::Frame], stop: u64, thread: i64) -> References {
    frames.iter().filter_map(|frame| frame.source_reference.filter(|reference| (1..=i32::MAX as i64).contains(reference)).map(|reference| ((stop, thread, frame.id), reference))).collect()
}
fn require_reference(references: &References, stop: u64, thread: i64, frame: i64, reference: i64) -> anyhow::Result<()> {
    anyhow::ensure!((1..=i32::MAX as i64).contains(&reference) && references.get(&(stop, thread, frame)) == Some(&reference), "This source reference was not reported in the selected stopped frame. Read its stack again.");
    Ok(())
}
pub(super) async fn read(registry: &Arc<Registry>, request: Request) -> Result<Vec<Response>, Failure> {
    let id = request.session.clone();
    let result: anyhow::Result<Vec<Response>> = async {
        let live = registry.get(&id).await?;
        {
            let _admission = live.operation.lock().await;
            observed_scope(&live, Some(request.stop), Some(request.thread), Some(&request.request_id)).await?;
            require_reference(&*live.adapter_sources.lock().await, request.stop, request.thread, request.frame, request.reference)?;
        }
        // An optional source provider must not hold the movement gate. A late
        // response is readonly and is rejected after a changed pause/reference.
        let (content, mime_type) = live.session.source_content(request.reference).await?;
        let source_hash = super::super::workspace_fs::view::hash(content.as_bytes());
        let _confirmation = live.operation.try_lock().map_err(|_| anyhow::anyhow!("The debugger is changing or reading another frame. Read adapter source again after the current pause is confirmed."))?;
        observed_scope(&live, Some(request.stop), Some(request.thread), Some(&request.request_id)).await?;
        require_reference(&*live.adapter_sources.lock().await, request.stop, request.thread, request.frame, request.reference)?;
        Ok(vec![Response::AdapterSource {session:id.clone(),stop:request.stop,thread:request.thread,frame:request.frame,reference:request.reference,request_id:request.request_id,content,source_hash,mime_type}])
    }.await;
    result.map_err(|error| (Some(id), error))
}
#[cfg(test)]
mod tests {
    use super::*;
    fn frame(id:i64,reference:Option<i64>)->hick_dap::Frame {
        serde_json::from_value(serde_json::json!({"id":id,"name":"runtime frame","line":null,"source":"same-name.py","source_line":0,"source_hash":null,"source_reference":reference,"in_document":false})).unwrap()
    }
    #[test]
    fn references_belong_to_the_reported_frame_and_never_borrow_a_workspace_name() {
        let frames=vec![frame(1,Some(123)),frame(2,Some(456)),frame(3,None)];
        let references=reported(&frames,7,9);
        assert!(require_reference(&references,7,9,1,123).is_ok());
        for (id,reference) in [(1,456),(2,123),(3,123),(4,123),(1,0),(1,-1),(1,i32::MAX as i64+1)] {assert!(require_reference(&references,7,9,id,reference).is_err());}
        assert!(require_reference(&references,8,9,1,123).is_err());
        assert!(require_reference(&references,7,10,1,123).is_err());
        assert!(require_reference(&References::new(),7,9,1,123).is_err());
    }
    #[test]
    fn wire_requires_a_complete_explicit_observation_and_rejects_unknown_fields() {
        let base=serde_json::json!({"op":"source","session":"owned","stop":1,"thread":2,"frame":3,"reference":123,"request_id":"source-request"});
        assert!(serde_json::from_value::<super::super::Request>(base.clone()).is_ok());
        for field in ["session","stop","thread","frame","reference","request_id"] {let mut value=base.clone();value.as_object_mut().unwrap().remove(field);assert!(serde_json::from_value::<super::super::Request>(value).is_err());}
        let mut value=base;value["path"]=serde_json::json!("same-name.py");assert!(serde_json::from_value::<super::super::Request>(value).is_err());
    }
}
