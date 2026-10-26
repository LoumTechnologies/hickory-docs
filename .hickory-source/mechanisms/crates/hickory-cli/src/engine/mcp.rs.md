<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/engine/mcp.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-d68d35ce1f7a"></a>
## `crates/hickory-cli/src/engine/mcp.rs`

Related contracts: [Keep a prediction separate from the answer it is tested against](../../../../../guarantees/verification/predictions-precede-managed-observation/contract.md), [Private edit proposals open review before writing](../../../../../guarantees/verification/private-proposals-wait-for-review/contract.md).

<a id="f-d68d35ce1f7a-Options"></a>
### `Options`

A stdio MCP conversation is a client, and its edit sessions live in the engine.

<storycode:copy id="f-d68d35ce1f7a-Options">
//! A stdio MCP conversation is a client, and its edit sessions live in the engine.
use super::{Attach, ClientGuard, Connection};
use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::PathBuf;

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Options {
    pub root: PathBuf,
    pub doc: Option<PathBuf>,
    pub params: Vec<(String, String)>,
    pub executor: crate::ExecutorChoice,
    pub session: Option<PathBuf>,
}
</storycode:copy>
<a id="f-d68d35ce1f7a-Call"></a>
### `Call`

<storycode:copy id="f-d68d35ce1f7a-Call">
#[derive(Serialize, Deserialize)]
pub(crate) struct Call {
    pub client: String,
    pub options: Options,
    pub params: Value,
}
</storycode:copy>
<a id="f-d68d35ce1f7a-Remote"></a>
### `Remote`

<storycode:copy id="f-d68d35ce1f7a-Remote">
pub(crate) struct Remote {
    connection: Connection,
    _guard: ClientGuard,
    options: Options,
}
</storycode:copy>
<a id="f-d68d35ce1f7a-impl-Remote"></a>
### `impl-Remote`

<storycode:copy id="f-d68d35ce1f7a-impl-Remote">
impl Remote {
    pub async fn new(options: Options) -> Result<Self> {
        let mut attach = Attach::new(options.root.clone(), options.executor);
        attach.params = options.params.clone();
        let connection = super::connect(attach).await?;
        let guard = connection.guard();
        Ok(Self {
            connection,
            _guard: guard,
            options,
        })
    }
    pub async fn private_mode(&self) -> Result<bool> {
        let path = format!(
            "/clients/{}/api/prediction/state",
            self.connection.attach.client
        );
        let response = self
            .connection
            .request(reqwest::Method::GET, &path)
            .await
            .send()
            .await?;
        if response.status() == reqwest::StatusCode::UNPROCESSABLE_ENTITY
            || response.status() == reqwest::StatusCode::NOT_FOUND
        {
            return Ok(false);
        }
        if !response.status().is_success() {
            bail!("Private prediction capability unavailable");
        }
        let value: Value = response.json().await?;
        if value["actual_values"] != "withheld" {
            bail!("Invalid private prediction capability response");
        }
        anyhow::ensure!(
            value["actor"]["kind"] == "agent",
            "Private agent capabilities require an agent-owned prediction binding"
        );
        Ok(true)
    }
    pub async fn private_catalogue(&self) -> Result<Value> {
        let state = self
            .private_request(reqwest::Method::GET, "state", &serde_json::json!({}))
            .await?;
        anyhow::ensure!(
            state["actual_values"] == "withheld",
            "Invalid private capability response"
        );
        Ok(crate::serve::private_predictions::catalogue(
            state["retained_plan"].is_object(),
        ))
    }
    pub async fn private_request(
        &self,
        method: reqwest::Method,
        surface: &str,
        body: &Value,
    ) -> Result<Value> {
        anyhow::ensure!(
            ["state", "history", "preview", "completion"].contains(&surface),
            "Unsupported private host surface"
        );
        let path = format!(
            "/clients/{}/api/prediction/{surface}",
            self.connection.attach.client
        );
        let mut request = self.connection.request(method.clone(), &path).await;
        if method == reqwest::Method::POST {
            request = request.json(body);
        }
        let response = request.send().await?;
        anyhow::ensure!(
            response.status().is_success(),
            "{}",
            if surface == "preview" {
                "Captured edit preview refused. Reread the current source and snapshot. For a complete proposal pass whole=\"true\", that snapshot, and input with the complete revised source; run/after require SHA-256 line anchors, never source text. The host still requires independent review before writing."
            } else {
                "Private host request refused"
            }
        );
        Ok(response.json().await?)
    }
    pub async fn call(&self, params: &Value) -> Result<Value> {
        let body = Call {
            client: self.connection.attach.client.clone(),
            options: self.options.clone(),
            params: params.clone(),
        };
        let response = self
            .connection
            .request(reqwest::Method::POST, "/tool")
            .await
            .json(&body)
            .send()
            .await?;
        if !response.status().is_success() {
            bail!("{}", response.text().await?);
        }
        Ok(response.json().await?)
    }
}
</storycode:copy>
<a id="f-d68d35ce1f7a-refusal"></a>
### `refusal`

<storycode:copy id="f-d68d35ce1f7a-refusal">
pub(crate) fn refusal(error: anyhow::Error) -> Value {
    json!({"content":[{"type":"text","text":format!("{error:#}")}],"isError":true})
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/engine/mcp.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/engine/mcp.rs"><storycode:paste select="#f-d68d35ce1f7a-Options"/><storycode:paste select="#f-d68d35ce1f7a-Call"/><storycode:paste select="#f-d68d35ce1f7a-Remote"/><storycode:paste select="#f-d68d35ce1f7a-impl-Remote"/><storycode:paste select="#f-d68d35ce1f7a-refusal"/></storycode:file>

</storycode:doc>
