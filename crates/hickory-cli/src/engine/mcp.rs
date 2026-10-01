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
