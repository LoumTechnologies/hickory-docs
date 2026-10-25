//! Opt-in, focused collector witnesses against tiny temporary projects.
//! Run with HICK_COVERAGE_REAL=1 and collectors on PATH. Never tests this workspace.
use super::*;
fn req(v: Value) -> Request {
    serde_json::from_value(v).unwrap()
}
async fn finished(root: &Path, id: &str) -> Value {
    for _ in 0..1800 {
        let v = dispatch(root, req(json!({"action":"get","id":id})))
            .await
            .unwrap();
        if v["status"] != "running" {
            return v;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    panic!("Collector fixture timed out")
}
#[tokio::test]
#[ignore = "requires installed pytest-cov, Vitest coverage, Coverlet, cargo-llvm-cov"]
async fn real_collectors_retain_automated_results_and_coverage() {
    assert_eq!(std::env::var("HICK_COVERAGE_REAL").as_deref(), Ok("1"));
    for language in ["python", "typescript", "javascript", "rust", "csharp"] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        match language {
            "python" => {
                std::fs::write(
                    root.join("pyproject.toml"),
                    "[project]\nname='coverage-fixture'\nversion='0.0.0'\n",
                )
                .unwrap();
                std::fs::write(root.join("app.py"),"def classify(value):\n    if value < 0:\n        return 'negative'\n    return 'positive'\n").unwrap();
                std::fs::write(root.join("test_app.py"),"from app import classify\ndef test_positive():\n    assert classify(1) == 'positive'\n").unwrap();
                #[cfg(unix)]
                std::os::unix::fs::symlink(
                    std::env::var("HICK_COVERAGE_PYTHON_ENV")
                        .expect("HICK_COVERAGE_PYTHON_ENV must point to a venv with pytest-cov"),
                    root.join(".venv"),
                )
                .unwrap();
            }
            "typescript" => {
                std::fs::write(root.join("package.json"),r#"{"type":"module","devDependencies":{"vitest":"3.2.7","@vitest/coverage-v8":"3.2.7"}}"#).unwrap();
                std::fs::write(root.join("app.ts"),"export function classify(n: number) { if (n < 0) return 'negative'; return 'positive'; }\n").unwrap();
                std::fs::write(root.join("app.test.ts"),"import {test,expect} from 'vitest'; import {classify} from './app'; test('positive',()=>expect(classify(1)).toBe('positive'));\n").unwrap();
                #[cfg(unix)]
                std::os::unix::fs::symlink(
                    std::env::var("HICK_COVERAGE_NODE_MODULES").expect(
                        "HICK_COVERAGE_NODE_MODULES must contain Vitest and its coverage plugin",
                    ),
                    root.join("node_modules"),
                )
                .unwrap();
            }
            "javascript" => {
                std::fs::write(
                    root.join("package.json"),
                    r#"{"devDependencies":{"jest":"29.7.0"}}"#,
                )
                .unwrap();
                std::fs::write(
                    root.join("app.js"),
                    "exports.classify=(n)=>{if(n<0)return 'negative';return 'positive';};\n",
                )
                .unwrap();
                std::fs::write(root.join("app.test.js"),"const {classify}=require('./app');test('positive',()=>expect(classify(1)).toBe('positive'));\n").unwrap();
                #[cfg(unix)]
                std::os::unix::fs::symlink(
                    std::env::var("HICK_COVERAGE_NODE_MODULES").unwrap(),
                    root.join("node_modules"),
                )
                .unwrap();
            }
            "rust" => {
                std::fs::write(
                    root.join("Cargo.toml"),
                    "[package]\nname='coverage-fixture'\nversion='0.0.0'\nedition='2024'\n",
                )
                .unwrap();
                std::fs::create_dir(root.join("src")).unwrap();
                std::fs::write(root.join("src/lib.rs"),"pub fn classify(n:i32)->bool { if n<0 { false } else { true } }\n#[test]\nfn positive(){ assert!(classify(1)); }\n").unwrap();
            }
            "csharp" => {
                std::fs::write(root.join("App.Tests.csproj"),r#"<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net10.0</TargetFramework><IsTestProject>true</IsTestProject></PropertyGroup><ItemGroup><Compile Remove="Product/**/*.cs"/><ProjectReference Include="Product/Product.csproj"/><PackageReference Include="Microsoft.NET.Test.Sdk" Version="17.14.1"/><PackageReference Include="xunit" Version="2.9.3"/><PackageReference Include="xunit.runner.visualstudio" Version="3.1.1"/><PackageReference Include="coverlet.collector" Version="6.0.4"/></ItemGroup></Project>"#).unwrap();
                std::fs::create_dir(root.join("Product")).unwrap();
                std::fs::write(root.join("Product/Product.csproj"),r#"<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net10.0</TargetFramework></PropertyGroup></Project>"#).unwrap();
                std::fs::write(root.join("Product/Code.cs"),"public class Product { public static int Classify(int n) { if(n<0) return -1; return 1; } }\n").unwrap();
                std::fs::write(root.join("App.cs"),"using Xunit; public class App { [Fact] public void Positive() { Assert.Equal(1,Product.Classify(1)); } }\n").unwrap();
            }
            _ => unreachable!(),
        }
        let discovered = dispatch(
            &root,
            req(json!({"action":"discover","path":".","language":language})),
        )
        .await
        .unwrap();
        assert!(
            !discovered["tests"].as_array().unwrap().is_empty(),
            "{language}: {discovered}"
        );
        let start = dispatch(
            &root,
            req(json!({"action":"run","path":".","language":language,"labels":["happy-path"]})),
        )
        .await
        .unwrap();
        let end = finished(&root, start["id"].as_str().unwrap()).await;
        assert_eq!(end["status"], "completed", "{language}: {end}");
        assert!(
            !end["tests"].as_array().unwrap().is_empty(),
            "{language}: {end}"
        );
        assert!(
            end["tests"]
                .as_array()
                .unwrap()
                .iter()
                .all(|t| t["status"] == "passed"),
            "{language}: {end}"
        );
        assert!(
            !end["evidence"]["report"]["files"]
                .as_object()
                .unwrap()
                .is_empty(),
            "{language}: {end}"
        );
        println!("{language}: discovery, test results and real collector report passed");
    }
}

#[tokio::test]
#[ignore = "requires Chrome and istanbul-lib-instrument; exercises an actual app button"]
async fn real_manual_browser_collects_only_paths_exercised_after_start() {
    use futures::{SinkExt, StreamExt};
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let app = "function classify(n) { if (n < 0) return 'negative'; return 'positive'; } document.querySelector('#reject').onclick=()=>{ document.querySelector('#result').textContent=classify(-1); };";
    std::fs::write(root.join("app.js"), app).unwrap();
    let module = std::env::var("HICK_COVERAGE_INSTRUMENTER")
        .expect("Set HICK_COVERAGE_INSTRUMENTER to istanbul-lib-instrument's module path");
    let script = r#"const fs=require('fs');const instrumenter=require(process.argv[1]).createInstrumenter();const code=instrumenter.instrumentSync(fs.readFileSync(process.argv[2],'utf8'),process.argv[2]);fs.writeFileSync(process.argv[3],'<button id="reject">Reject invalid input</button><output id="result"></output><script>'+code+'</script>');"#;
    let output = std::process::Command::new("node")
        .args([
            "-e",
            script,
            &module,
            root.join("app.js").to_str().unwrap(),
            root.join("index.html").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let chrome = tempfile::tempdir().unwrap();
    let mut child = tokio::process::Command::new(
        std::env::var("HICK_COVERAGE_CHROME").unwrap_or_else(|_| "google-chrome".into()),
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
    assert!(!port.is_empty(), "Chrome did not expose CDP");
    let url = format!("http://127.0.0.1:{port}");
    // Allow the page to register its event handler before taking the baseline.
    let mut started = None;
    for _ in 0..50 {
        match dispatch(&root,req(json!({"action":"manual-start","path":".","language":"typescript","url":url,"selector":"index.html","labels":["adversarial"]}))).await {Ok(v)=>{started=Some(v);break;},Err(_)=>tokio::time::sleep(std::time::Duration::from_millis(100)).await}
    }
    let start = started.expect("Instrumented browser session should start");
    let pages: Value = reqwest::get(format!("{url}/json/list"))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let page = pages
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["type"] == "page")
        .unwrap();
    let (mut socket, _) =
        tokio_tungstenite::connect_async(page["webSocketDebuggerUrl"].as_str().unwrap())
            .await
            .unwrap();
    socket.send(tokio_tungstenite::tungstenite::Message::Text(json!({"id":9,"method":"Runtime.evaluate","params":{"expression":"document.querySelector('#reject').click(); document.querySelector('#result').textContent","returnByValue":true}}).to_string())).await.unwrap();
    loop {
        let m = socket.next().await.unwrap().unwrap();
        if !m.is_text() {
            continue;
        }
        let v: Value = serde_json::from_str(m.to_text().unwrap()).unwrap();
        if v["id"] == 9 {
            assert_eq!(v["result"]["result"]["value"], "negative");
            break;
        }
    }
    socket.close(None).await.unwrap();
    let checkpoint=dispatch(&root,req(json!({"action":"browser-checkpoint","id":start["id"],"url":url,"selector":"index.html"}))).await.unwrap();
    let file = &checkpoint["evidence"]["report"]["files"]["app.js"];
    let branches = file["branches"].as_array().unwrap();
    assert!(branches.iter().any(|b| b["covered"] == true));
    assert!(branches.iter().any(|b| b["covered"] == false));
    assert_eq!(checkpoint["evidence"]["attribution"], "group");
    assert!(checkpoint["manual_verdict"].is_null());
    dispatch(&root, req(json!({"action":"stop","id":start["id"]})))
        .await
        .unwrap();
    child.kill().await.unwrap();
    println!(
        "Real manual app button reached the rejection branch; the alternate branch stayed uncovered"
    );
}

#[tokio::test]
#[ignore = "requires pytest-cov; launches and stops a tiny instrumented app"]
async fn real_manual_python_process_flushes_coverage_on_stop() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let python = PathBuf::from(std::env::var("HICK_COVERAGE_PYTHON_ENV").unwrap())
        .join("bin/python")
        .to_string_lossy()
        .into_owned();
    std::fs::write(root.join("app.py"),"import time\nfrom pathlib import Path\ndef classify(n):\n    if n < 0:\n        return -1\n    return 1\nclassify(1)\nPath('.hick-cache/ready').write_text('ready')\nwhile True:\n    time.sleep(0.05)\n").unwrap();
    std::fs::write(root.join(".hick-tests.json"),json!({"manual":{"app":{"argv":[python,"-m","coverage","run","--branch","--data-file","{root}/.hick-cache/coverage/{run}/data","app.py"],"report":".hick-cache/coverage/{run}/manual.json","format":"coverage-py","finish":[python,"-m","coverage","json","--data-file","{root}/.hick-cache/coverage/{run}/data","-o","{report}"],"collector":"coverage.py","version":"fixture-installed"}}}).to_string()).unwrap();
    let start=dispatch(&root,req(json!({"action":"manual-start","path":".","language":"python","profile":"app","labels":["happy-path"]}))).await.unwrap();
    for _ in 0..100 {
        if root.join(".hick-cache/ready").exists() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    assert!(
        root.join(".hick-cache/ready").exists(),
        "Instrumented app did not become ready"
    );
    dispatch(&root, req(json!({"action":"stop","id":start["id"]})))
        .await
        .unwrap();
    let end = finished(&root, start["id"].as_str().unwrap()).await;
    assert_eq!(end["status"], "completed", "{end}");
    assert_eq!(end["source_changed_during_run"], false);
    assert!(
        end["evidence"]["report"]["files"]["app.py"]["branches"]
            .as_array()
            .unwrap()
            .iter()
            .any(|b| b["covered"] == false)
    );
    assert!(end["manual_verdict"].is_null());
}

#[tokio::test]
#[ignore = "requires pytest-cov; verifies failed-run evidence and targeted reruns"]
async fn real_failed_test_keeps_coverage_and_reruns_only_the_failure() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(
        std::env::var("HICK_COVERAGE_PYTHON_ENV").unwrap(),
        root.join(".venv"),
    )
    .unwrap();
    std::fs::write(
        root.join("app.py"),
        "def classify(n):\n    if n<0:\n        return 'negative'\n    return 'positive'\n",
    )
    .unwrap();
    let tests = "from app import classify\ndef test_positive():\n    assert classify(1) == 'positive'\ndef test_rejection():\n    assert classify(-1) == 'WRONG'\n";
    std::fs::write(root.join("test_app.py"), tests).unwrap();
    let started = dispatch(
        &root,
        req(json!({"action":"run","path":".","language":"python"})),
    )
    .await
    .unwrap();
    let failed = finished(&root, started["id"].as_str().unwrap()).await;
    assert_eq!(failed["status"], "failed", "{failed}");
    assert!(!failed["evidence"].is_null());
    assert_eq!(
        failed["tests"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|t| t["status"] == "failed")
            .count(),
        1
    );
    std::fs::write(
        root.join("test_app.py"),
        tests.replace("'WRONG'", "'negative'"),
    )
    .unwrap();
    let started = dispatch(
        &root,
        req(json!({"action":"rerun-failed","id":failed["id"]})),
    )
    .await
    .unwrap();
    let rerun = finished(&root, started["id"].as_str().unwrap()).await;
    assert_eq!(rerun["status"], "completed", "{rerun}");
    assert_eq!(rerun["tests"].as_array().unwrap().len(), 1);
    assert_eq!(rerun["tests"][0]["id"], "test_app.py::test_rejection");
    assert_eq!(
        store::view(&root, failed["id"].as_str().unwrap()).unwrap()["source_state"],
        "historical"
    );
}
