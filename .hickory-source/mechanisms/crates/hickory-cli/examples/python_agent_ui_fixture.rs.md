<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/examples/python_agent_ui_fixture.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-d3f7fdd423b1"></a>
## `crates/hickory-cli/examples/python_agent_ui_fixture.rs`

<a id="f-d3f7fdd423b1-main"></a>
### `main`

Browser acceptance engine: scripted by default; explicit --live-provider uses normal keys.
This is the native engine in a browser, not a packaged desktop-shell witness.

<storycode:copy id="f-d3f7fdd423b1-main">
//! Browser acceptance engine: scripted by default; explicit --live-provider uses normal keys.
//! This is the native engine in a browser, not a packaged desktop-shell witness.
use hickory_cli::{
    ExecutorChoice,
    serve::{ServeOptions, prepare},
};
use serde_json::json;
use std::{path::PathBuf, sync::Arc};
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let root = PathBuf::from(args.next().expect("scratch project"));
    let live = match args.next().as_deref() {
        None => false,
        Some("--live-provider") => true,
        Some(other) => anyhow::bail!("Unknown fixture option: {other}"),
    };
    anyhow::ensure!(args.next().is_none(), "Unexpected fixture arguments");
    anyhow::ensure!(root.is_dir(), "The explicit scratch project must exist");
    let prepared = prepare(ServeOptions {
        target: root.clone(),
        port: 0,
        params: vec![],
        executor: ExecutorChoice::Local,
        key_store_path: if live {
            hickory_agent::KeyStore::desktop_path()
        } else {
            Some(root.join("keys.json"))
        },
        ui_settings_path: Some(root.join("ui.json")),
    })
    .await?;
    if !live {
        prepared.state.agent.set_llm_override(Arc::new(hickory_agent::ScriptedLlmClient::new([
        json!({"python":"tool('read_doc', {'doc':'note.md'})\ntool('edit_doc', {'doc':'note.md','after':'^','input':'# Python agent proposal'})"}).to_string(),
        json!({"python":r#"import json
s = json.loads(tool('changeset_status', {})['content'][0]['text'])
tool('submit_changeset', {'boundary':s['boundary'], 'expected':s['expected'], 'entry':s['entry'], 'sources':{s['entry']:s['template']}})"#}).to_string(),
        json!({"done":"Proposed the prefix. No checks were run."}).to_string(),
        json!({"python":"tool('create_doc', {'path':'cancelled.md','input':'This needs approval'})"}).to_string(),
      ])));
    } else {
        eprintln!(
            "Live provider calls enabled for this explicit scratch project. No desktop shell or release witness is implied."
        );
    }
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    println!("http://{}", listener.local_addr()?);
    axum::serve(listener, prepared.router).await?;
    Ok(())
}
</storycode:copy>

### Assemble `crates/hickory-cli/examples/python_agent_ui_fixture.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/examples/python_agent_ui_fixture.rs"><storycode:paste select="#f-d3f7fdd423b1-main"/></storycode:file>

</storycode:doc>
