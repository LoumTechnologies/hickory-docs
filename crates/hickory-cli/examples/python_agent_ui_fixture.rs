//! Browser acceptance fixture: real supervisor, worker and confinement; scripted model only.
use hickory_cli::{
    ExecutorChoice,
    serve::{ServeOptions, prepare},
};
use serde_json::json;
use std::{path::PathBuf, sync::Arc};
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let root = PathBuf::from(std::env::args().nth(1).expect("scratch project"));
    let prepared = prepare(ServeOptions {
        target: root.clone(),
        port: 0,
        params: vec![],
        executor: ExecutorChoice::Local,
        key_store_path: Some(root.join("keys.json")),
        ui_settings_path: Some(root.join("ui.json")),
    })
    .await?;
    prepared.state.agent.set_llm_override(Arc::new(hickory_agent::ScriptedLlmClient::new([
        json!({"python":"tool('read_doc', {'doc':'note.md'})\ntool('edit_doc', {'doc':'note.md','after':'^','input':'# Python agent proposal'})"}).to_string(),
        json!({"python":r#"import json
s = json.loads(tool('changeset_status', {})['content'][0]['text'])
tool('submit_changeset', {'boundary':s['boundary'], 'expected':s['expected'], 'entry':s['entry'], 'sources':{s['entry']:s['template']}})"#}).to_string(),
        json!({"done":"Proposed the prefix. No checks were run."}).to_string(),
        json!({"python":"tool('create_doc', {'path':'cancelled.md','input':'This needs approval'})"}).to_string(),
    ])));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    println!("http://{}", listener.local_addr()?);
    axum::serve(listener, prepared.router).await?;
    Ok(())
}
