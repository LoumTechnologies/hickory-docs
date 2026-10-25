<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/performance/real_tests.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `d8a1725b92421ec920c7ab84d82f98f37db890c57108ff4fd453157dfeae8c2b`.

<book:entry path="crates/hickory-cli/src/performance/real_tests.rs">
//! Focused opt-in real provider witnesses; never profiles this repository's suite.
use super::*;
use futures::{SinkExt, StreamExt};
fn req(v: Value) -> Request {
    serde_json::from_value(v).unwrap()
}
async fn finished(root: &Path, id: &str) -> Value {
    for _ in 0..600 {
        let run = store::view(root, id, true).unwrap();
        if run["status"] != "running" {
            return run;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    panic!("Profile witness timed out")
}
#[tokio::test]
#[ignore = "requires Chrome and Node; records a real TypeScript UI workload with an inline source map"]
async fn browser_ui_workload_maps_samples_to_original_typescript_and_literate_source() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let chrome = tempfile::tempdir().unwrap();
    let source = "function hot(n: number): number {\n    let value = 0;\n    for (let i = 0; i < n; i++) value += Math.sqrt(i);\n    return value;\n}\ndocument.querySelector('#work')!.addEventListener('click', () => {\n    const end = performance.now() + 250;\n    while (performance.now() < end) hot(10000);\n});\n";
    std::fs::write(root.join("app.ts"), source).unwrap();
    std::fs::write(
        root.join("app.md"),
        format!("# Profile fixture\n<hick:file path=\"app.ts\">\n{source}</hick:file>\n"),
    )
    .unwrap();
    let typescript = std::env::var("HICK_PROFILE_TYPESCRIPT")
        .expect("Set HICK_PROFILE_TYPESCRIPT to installed typescript module path");
    let script = "const fs=require('fs'),ts=require(process.argv[1]);const source=fs.readFileSync(process.argv[2],'utf8');fs.writeFileSync(process.argv[3],ts.transpileModule(source,{fileName:'app.ts',compilerOptions:{target:ts.ScriptTarget.ES2022,inlineSourceMap:true,inlineSources:true}}).outputText);";
    let status = std::process::Command::new("node")
        .args([
            "-e",
            script,
            &typescript,
            root.join("app.ts").to_str().unwrap(),
            root.join("app.js").to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success());
    std::fs::write(
        root.join("index.html"),
        "<button id='work'>Run workload</button><script src='app.js'></script>",
    )
    .unwrap();
    let mut child = tokio::process::Command::new(
        std::env::var("HICK_PROFILE_CHROME").unwrap_or_else(|_| "google-chrome".into()),
    )
    .args([
        "--headless",
        "--no-sandbox",
        "--disable-gpu",
        "--remote-debugging-port=0",
        &format!("--user-data-dir={}", chrome.path().display()),
        &format!("file://{}", root.join("index.html").display()),
    ])
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::null())
    .kill_on_drop(true)
    .spawn()
    .unwrap();
    let mut port = String::new();
    for _ in 0..100 {
        if let Ok(text) = std::fs::read_to_string(chrome.path().join("DevToolsActivePort")) {
            port = text.lines().next().unwrap().into();
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    assert!(!port.is_empty());
    let url = format!("http://127.0.0.1:{port}");
    let start=dispatch(&root,req(json!({"action":"browser-start","path":".","url":url,"selector":"index.html","labels":["manual"]}))).await.unwrap();
    let list: Value = reqwest::get(format!("{url}/json/list"))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let page = list
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["type"] == "page")
        .unwrap();
    let (mut socket, _) =
        tokio_tungstenite::connect_async(page["webSocketDebuggerUrl"].as_str().unwrap())
            .await
            .unwrap();
    socket.send(tokio_tungstenite::tungstenite::Message::Text(json!({"id":1,"method":"Runtime.evaluate","params":{"expression":"document.querySelector('#work').click()","returnByValue":true}}).to_string())).await.unwrap();
    loop {
        let message = socket.next().await.unwrap().unwrap();
        if message.is_text() {
            let v: Value = serde_json::from_str(message.to_text().unwrap()).unwrap();
            if v["id"] == 1 {
                assert!(v.get("error").is_none());
                break;
            }
        }
    }
    socket.close(None).await.unwrap();
    dispatch(&root, req(json!({"action":"stop","id":start["id"]})))
        .await
        .unwrap();
    let run = finished(&root, start["id"].as_str().unwrap()).await;
    assert_eq!(run["status"], "completed", "{run}");
    let rows = run["report"]["threads"][0]["rows"].as_array().unwrap();
    let hot = rows
        .iter()
        .find(|r| r["frame"]["name"] == "hot")
        .expect("Workload hotspot must be sampled");
    assert!(hot["self_weight"].as_f64().unwrap() > 0.);
    assert_eq!(hot["frame"]["file"], "app.ts");
    assert!(hot["total_calls"].is_null());
    let line = hot["frame"]["line"].as_u64().unwrap().to_string();
    assert_eq!(run["mappings"]["app.ts"][&line]["path"], "app.md");
    assert!(
        run["sources"]["app.md"]
            .as_str()
            .unwrap()
            .contains("function hot")
    );
    child.kill().await.unwrap();
    println!("Real V8 samples mapped through TypeScript source maps to the literate document");
}
#[tokio::test]
#[ignore = "requires the supported cell sandbox and CPython"]
async fn confined_python_profile_reuses_existing_provider_and_captures_function_calls() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let path = root.join("app.md");
    std::fs::write(&path,"# Profile\n<hick:file path=\"app.py\">\ndef hot():\n    return sum(range(10000))\nfor _ in range(3):\n    hot()\n</hick:file>\n").unwrap();
    let run = dispatch(
        &root,
        req(json!({"action":"capture-python","path":"app.md"})),
    )
    .await
    .unwrap();
    assert_eq!(run["status"], "completed", "{run}");
    let rows = run["report"]["threads"][0]["rows"].as_array().unwrap();
    let hot = rows.iter().find(|r| r["frame"]["name"] == "hot").unwrap();
    assert_eq!(hot["total_calls"], 3);
    assert_eq!(hot["frame"]["file"], "app.md");
    assert!(run["environment"]["runtime_run"].is_string());
}

#[tokio::test]
#[ignore = "requires Node; runs a tiny native collector profile and verifies exact source mapping"]
async fn node_collector_profile_runs_an_automated_workload_and_keeps_real_samples() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let source = "function hot(n) { let v=0; for(let i=0;i<n;i++) v+=Math.sqrt(i); return v; }\nconst end=Date.now()+150; while(Date.now()<end) hot(10000);\n";
    std::fs::write(root.join("app.js"), source).unwrap();
    std::fs::write(
        root.join("app.md"),
        format!("# Native profile\n<hick:file path=\"app.js\">\n{source}</hick:file>\n"),
    )
    .unwrap();
    std::fs::write(root.join(".hick-profile.json"),json!({"node-workload":{"argv":["node","--cpu-prof","--cpu-prof-dir={root}/.hick-cache/profiles/{run}","--cpu-prof-name=run.cpuprofile","app.js"],"format":"chrome-cpu","report":".hick-cache/profiles/{run}/run.cpuprofile","collector":"node-v8","version_argv":["node","--version"]}}).to_string()).unwrap();
    let start = dispatch(
        &root,
        req(json!({"action":"run","path":".","profile":"node-workload","labels":["automated"]})),
    )
    .await
    .unwrap();
    let run = finished(&root, start["id"].as_str().unwrap()).await;
    assert_eq!(run["status"], "completed", "{run}");
    assert_eq!(run["source_changed"], false);
    let hot = run["report"]["threads"][0]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["frame"]["name"] == "hot")
        .unwrap();
    assert_eq!(hot["frame"]["file"], "app.js");
    assert!(hot["samples"].as_u64().unwrap() > 0);
    assert!(hot["total_calls"].is_null());
    assert_eq!(run["mappings"]["app.js"]["1"]["path"], "app.md");
}

#[tokio::test]
#[ignore = "requires dotnet and dotnet-trace; records actual Speedscope scopes from a tiny C# workload"]
async fn dotnet_collector_preserves_sampled_scopes_and_unresolved_symbols() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    std::fs::write(root.join("App.csproj"),"<Project Sdk=\"Microsoft.NET.Sdk\"><PropertyGroup><OutputType>Exe</OutputType><TargetFramework>net10.0</TargetFramework></PropertyGroup></Project>").unwrap();
    std::fs::write(root.join("Program.cs"),"using System; class App { static double Hot(int n) { double total=0;for(int i=0;i<n;i++)total+=Math.Sqrt(i);return total; } static void Main(){var watch=System.Diagnostics.Stopwatch.StartNew();double value=0;while(watch.ElapsedMilliseconds<500)value+=Hot(10000);Console.WriteLine(value);} }").unwrap();
    let build = crate::coverage::execute(
        &[
            "dotnet".into(),
            "build".into(),
            "--configuration".into(),
            "Release".into(),
        ],
        &root,
        None,
    )
    .await
    .unwrap();
    assert_eq!(build.code, Some(0), "{}", build.text);
    std::fs::write(root.join(".hick-profile.json"),json!({"dotnet-workload":{"argv":["dotnet-trace","collect","--format","Speedscope","--output","{root}/.hick-cache/profiles/{run}/trace.nettrace","--","dotnet","{root}/bin/Release/net10.0/App.dll"],"format":"speedscope","report":".hick-cache/profiles/{run}/trace.speedscope.json","collector":"dotnet-trace","version_argv":["dotnet-trace","--version"]}}).to_string()).unwrap();
    let start = dispatch(
        &root,
        req(json!({"action":"run","path":".","profile":"dotnet-workload"})),
    )
    .await
    .unwrap();
    let run = finished(&root, start["id"].as_str().unwrap()).await;
    assert_eq!(run["status"], "completed", "{run}");
    let threads = run["report"]["threads"].as_array().unwrap();
    assert!(!threads.is_empty());
    assert!(
        threads
            .iter()
            .flat_map(|t| t["rows"].as_array().unwrap())
            .any(|r| r["frame"]["name"].as_str().unwrap().contains("App"))
    );
    assert!(
        threads
            .iter()
            .flat_map(|t| t["rows"].as_array().unwrap())
            .all(|r| r["total_calls"].is_null())
    );
}
</book:entry>

</book:doc>
