use super::*;
use futures::{SinkExt, StreamExt};
use std::{
    sync::{Mutex, OnceLock},
    time::Duration,
};
use tokio::sync::oneshot;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, tungstenite::Message};
type Socket = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(super) struct Script {
    pub url: String,
    pub source: Option<String>,
    pub map: Option<Value>,
    pub map_url: Option<String>,
}
pub(super) struct Connection {
    socket: Socket,
    scripts: BTreeMap<String, Script>,
    pub version: String,
    page: String,
    epoch: f64,
    next: u64,
}
type Sessions = Mutex<BTreeMap<String, Option<oneshot::Sender<()>>>>;
fn sessions() -> &'static Sessions {
    static S: OnceLock<Sessions> = OnceLock::new();
    S.get_or_init(|| Mutex::new(BTreeMap::new()))
}
fn key(root: &Path, id: &str) -> String {
    format!("{}:{id}", root.display())
}
pub(super) fn active(root: &Path, id: &str) -> bool {
    sessions().lock().unwrap().contains_key(&key(root, id))
}
pub(super) fn stop(root: &Path, id: &str) -> bool {
    sessions()
        .lock()
        .unwrap()
        .get_mut(&key(root, id))
        .and_then(Option::take)
        .is_some_and(|s| s.send(()).is_ok())
}
fn notification(v: &Value, scripts: &mut BTreeMap<String, Script>) {
    if v["method"] == "Debugger.scriptParsed" {
        let p = &v["params"];
        if let (Some(id), Some(url)) = (p["scriptId"].as_str(), p["url"].as_str())
            && scripts.len() < 20_000
        {
            scripts.insert(
                id.into(),
                Script {
                    url: url.into(),
                    map_url: p["sourceMapURL"].as_str().map(str::to_string),
                    ..Default::default()
                },
            );
        }
    }
}
impl Connection {
    async fn call(&mut self, method: &str, params: Value) -> Result<Value> {
        self.next += 1;
        let id = self.next;
        self.socket
            .send(Message::Text(
                json!({"id":id,"method":method,"params":params}).to_string(),
            ))
            .await?;
        tokio::time::timeout(Duration::from_secs(15), async {
            while let Some(message) = self.socket.next().await {
                let message = message?;
                if !message.is_text() {
                    continue;
                }
                let v: Value = serde_json::from_str(message.to_text()?)?;
                if v["id"] == id {
                    ensure!(
                        v.get("error").is_none(),
                        "Chrome profiler refused {method}: {}",
                        v["error"]
                    );
                    return Ok(v["result"].clone());
                }
                notification(&v, &mut self.scripts);
            }
            bail!("Browser profiler disconnected")
        })
        .await?
    }
}
pub(super) async fn connect(endpoint: &str, selector: Option<&str>) -> Result<Connection> {
    let endpoint = reqwest::Url::parse(endpoint)?;
    ensure!(
        endpoint.scheme() == "http"
            && matches!(
                endpoint.host_str(),
                Some("localhost" | "127.0.0.1" | "[::1]")
            ),
        "Use a loopback Chrome CDP HTTP endpoint"
    );
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let list: Value = client
        .get(endpoint.join("/json/list")?)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let pages: Vec<_> = list
        .as_array()
        .context("Invalid CDP target list")?
        .iter()
        .filter(|p| {
            p["type"] == "page"
                && selector.is_none_or(|s| p["url"].as_str().is_some_and(|p| p.contains(s)))
        })
        .collect();
    ensure!(
        pages.len() == 1,
        "Select exactly one browser page; found {}",
        pages.len()
    );
    let socket_url = pages[0]["webSocketDebuggerUrl"]
        .as_str()
        .context("No page debugger socket")?;
    let url = reqwest::Url::parse(socket_url)?;
    ensure!(
        url.scheme() == "ws"
            && url.host_str() == endpoint.host_str()
            && url.port_or_known_default() == endpoint.port_or_known_default(),
        "Debugger socket is outside requested endpoint"
    );
    let version: Value = client
        .get(endpoint.join("/json/version")?)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let (socket, _) = tokio_tungstenite::connect_async(socket_url).await?;
    let mut c = Connection {
        socket,
        scripts: BTreeMap::new(),
        version: version["Browser"]
            .as_str()
            .unwrap_or("unknown Chrome")
            .into(),
        page: pages[0]["url"].as_str().unwrap_or("").into(),
        epoch: 0.,
        next: 0,
    };
    c.call("Debugger.enable", json!({})).await?;
    c.call("Profiler.enable", json!({})).await?;
    c.call("Profiler.setSamplingInterval", json!({"interval":1000}))
        .await?;
    c.epoch = c
        .call(
            "Runtime.evaluate",
            json!({"expression":"performance.timeOrigin","returnByValue":true}),
        )
        .await?["result"]["value"]
        .as_f64()
        .context("Page clock unavailable")?;
    c.call("Profiler.start", json!({})).await?;
    Ok(c)
}
pub(super) fn start(root: PathBuf, mut run: Record, mut connection: Connection) -> Result<()> {
    let (tx, mut rx) = oneshot::channel();
    sessions()
        .lock()
        .unwrap()
        .insert(key(&root, &run.id), Some(tx));
    tokio::spawn(async move {
        let result=async {
            let timeout=tokio::time::sleep(Duration::from_secs(900));tokio::pin!(timeout);
            loop {tokio::select!{_= &mut rx=>break,_=&mut timeout=>bail!("Browser profiling exceeded 15 minutes"),message=connection.socket.next()=>{let m=message.context("Browser profiler disconnected")??;if m.is_text(){notification(&serde_json::from_str::<Value>(m.to_text()?)?,&mut connection.scripts);}}}}
            let raw=connection.call("Profiler.stop",json!({})).await?;
            run.report=Some(hick_profile::parse("chrome-cpu",&raw.to_string())?);
            let epoch=connection.call("Runtime.evaluate",json!({"expression":"performance.timeOrigin","returnByValue":true})).await?["result"]["value"].as_f64();
            ensure!(epoch==Some(connection.epoch),"Page reloaded during profile; start a new session");
            let ids:std::collections::BTreeSet<_>=raw["profile"]["nodes"].as_array().context("Missing CPU nodes")?.iter().filter_map(|n|n["callFrame"]["scriptId"].as_str().map(str::to_string)).collect();
            let mut scripts=BTreeMap::new();let mut total=0;
            for id in ids {
                if let Some(mut script)=connection.scripts.get(&id).cloned() {
                    if let Ok(source)=connection.call("Debugger.getScriptSource",json!({"scriptId":id})).await && let Some(text)=source["scriptSource"].as_str(){total+=text.len();ensure!(total<=16*1024*1024,"Sampled script sources exceed 16 MiB");script.source=Some(text.into());}
                    if let Some(map)=script.map_url.as_deref(){script.map=load_map(&connection.page,&script.url,map).await.ok();}
                    scripts.insert(id,script);
                }
            }
            let wrapped=json!({"profile":raw["profile"],"scripts":scripts});let raw=wrapped.to_string();
            run.report=Some(hick_profile::parse("chrome-cpu",&raw)?);resolve::bind(&root,&mut run,Some(&scripts))?;run.status="completed".into();Ok(())
        }.await;
        let _ = connection.socket.close(None).await;
        store::finish(&root, &mut run, result);
        sessions().lock().unwrap().remove(&key(&root, &run.id));
    });
    Ok(())
}
async fn load_map(page: &str, script: &str, map: &str) -> Result<Value> {
    use base64::Engine;
    let raw = if map.starts_with("data:") {
        let (metadata, data) = map.split_once(',').context("Invalid source-map data URL")?;
        ensure!(
            metadata.ends_with(";base64"),
            "Only base64 source-map data URLs supported"
        );
        base64::engine::general_purpose::STANDARD.decode(data)?
    } else {
        let page = reqwest::Url::parse(page)?;
        let script = reqwest::Url::parse(script)?;
        let url = script.join(map)?;
        ensure!(
            url.origin() == page.origin()
                && url.scheme() == "http"
                && matches!(url.host_str(), Some("127.0.0.1" | "localhost" | "[::1]")),
            "Source map is not on the profiled loopback origin"
        );
        let response = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .redirect(reqwest::redirect::Policy::none())
            .build()?
            .get(url)
            .send()
            .await?
            .error_for_status()?;
        ensure!(
            response
                .content_length()
                .is_none_or(|n| n <= 8 * 1024 * 1024),
            "Source map too large"
        );
        let mut stream = response.bytes_stream();
        let mut bytes = Vec::new();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            ensure!(
                bytes.len() + chunk.len() <= 8 * 1024 * 1024,
                "Source map too large"
            );
            bytes.extend_from_slice(&chunk);
        }
        bytes
    };
    ensure!(raw.len() <= 8 * 1024 * 1024, "Source map too large");
    Ok(serde_json::from_slice(&raw)?)
}
