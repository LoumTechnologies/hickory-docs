<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/engine/mcp.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Keep a prediction separate from the answer it is tested against](../../../../../../guarantees/verification/predictions-precede-managed-observation/contract.md)
- [Private edit proposals open review before writing](../../../../../../guarantees/verification/private-proposals-wait-for-review/contract.md)

Captured SHA-256: `190bba110bbcae1f5df92774a796e893fc19bbec45f784f39734f6430b597e80`.

<book:entry path="crates/hickory-cli/src/engine/mcp.rs">
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
#[derive(Serialize, Deserialize)]
pub(crate) struct Call {
    pub client: String,
    pub options: Options,
    pub params: Value,
}
pub(crate) struct Remote {
    connection: Connection,
    _guard: ClientGuard,
    options: Options,
}
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
pub(crate) fn refusal(error: anyhow::Error) -> Value {
    json!({"content":[{"type":"text","text":format!("{error:#}")}],"isError":true})
}
</book:entry>

</book:doc>
