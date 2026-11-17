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
#[cfg(unix)]
#[tokio::test]
async fn owned_collector_retains_outcome_without_writing_replacement_roots_or_stores(){
 use std::{sync::Arc,time::Duration};
 for replace_root in [true,false] {
  let temporary=tempfile::tempdir().unwrap();let root=temporary.path().join("workspace");std::fs::create_dir(&root).unwrap();let root=root.canonicalize().unwrap();std::fs::write(root.join("app.py"),"answer=42\n").unwrap();
  let scope=Arc::new(crate::serve::resource_inputs::RootIdentity::bind(&root));
  let request=Request{action:"run".into(),path:".".into(),language:"python".into(),..Default::default()};
  let mut record=store::new_record(&root,&root,&request).unwrap();let mut plan=commands::automated(&root,&root,&mut record,&[]).unwrap();
  let ready=temporary.path().join("ready");let release=temporary.path().join("release");
  let script="touch \"$1\"; while [ ! -f \"$2\" ]; do sleep 0.02; done; mkdir -p .hick-cache; printf original > .hick-cache/collector-write; printf '%s' '{\"files\":{\"app.py\":{\"executed_lines\":[1],\"missing_lines\":[]}}}' > \"$3\"; printf '%s' '<testsuite><testcase classname=\"app\" name=\"works\" file=\"app.py\" line=\"0\"/></testsuite>' > \"$4\"";
  plan.argv=vec!["/bin/sh".into(),"-c".into(),script.into(),"owned-collector".into(),ready.display().to_string(),release.display().to_string(),plan.report.display().to_string(),plan.result.display().to_string()];plan.version.clear();
  store::save(&root,&record).unwrap();runner::start(root.clone(),root.clone(),record.clone(),plan).unwrap();
  for _ in 0..200 {if ready.exists(){break}tokio::time::sleep(Duration::from_millis(10)).await;}assert!(ready.exists());
  let moved=temporary.path().join("original");
  if replace_root {std::fs::rename(&root,&moved).unwrap();std::fs::create_dir(&root).unwrap();std::fs::write(root.join("app.py"),"answer=42\n").unwrap();}
  else {std::fs::rename(root.join(".hick-cache/coverage"),&moved).unwrap();std::fs::create_dir(root.join(".hick-cache/coverage")).unwrap();}
  std::fs::write(&release,"release").unwrap();
  for _ in 0..200 {if !runner::active(&root,&record.id){break}tokio::time::sleep(Duration::from_millis(10)).await;}assert!(!runner::active(&root,&record.id));
  let retained=runner::retained(&root,&record.id).unwrap();assert_eq!(retained.status,"completed");assert_eq!(retained.tests.len(),1);assert!(retained.evidence.is_some());
  if replace_root {assert!(runner::owns_outcome(&root,&scope,&record.id));assert!(!runner::owns_outcome(&root,&crate::serve::resource_inputs::RootIdentity::bind(&root),&record.id));assert!(!root.join(".hick-cache").exists());assert_eq!(std::fs::read_to_string(moved.join(".hick-cache/collector-write")).unwrap(),"original");let value=dispatch_scoped(&root,scope.clone(),Request{action:"get".into(),id:Some(record.id.clone()),..Default::default()}).await.unwrap();assert_eq!(value["authority"],"historical-unverified");assert_eq!(value["status"],"completed");assert!(dispatch_scoped(&root,scope,request).await.is_err());}
  else {assert_eq!(std::fs::read_dir(root.join(".hick-cache/coverage")).unwrap().count(),0);assert_eq!(store::view(&root,&record.id).unwrap()["status"],"completed");assert!(store::list(&root).unwrap()["runs"].as_array().unwrap().iter().any(|value|value["id"]==record.id&&value["status"]=="completed"));}
 }
}

#[cfg(unix)]
#[test]
fn private_artifact_replacement_retires_report_authority_and_preserves_foreign_cleanup(){
 let temporary=tempfile::tempdir().unwrap();let root=temporary.path().canonicalize().unwrap();std::fs::write(root.join("app.py"),"value=1\n").unwrap();let request=Request{action:"run".into(),path:".".into(),language:"python".into(),..Default::default()};let mut record=store::new_record(&root,&root,&request).unwrap();let plan=commands::automated(&root,&root,&mut record,&[]).unwrap();
 let artifacts=plan.artifacts.as_ref().unwrap();let path=artifacts.path().to_path_buf();let moved=root.join("original-artifacts");std::fs::rename(&path,&moved).unwrap();std::fs::create_dir(&path).unwrap();std::fs::write(path.join("foreign"),"retain").unwrap();assert!(artifacts.check().is_err());drop(plan);assert_eq!(std::fs::read_to_string(path.join("foreign")).unwrap(),"retain");std::fs::remove_dir_all(path).unwrap();
}

#[test]
fn missing_collector_history_reads_do_not_create_storage(){
 let temporary=tempfile::tempdir().unwrap();let root=temporary.path().canonicalize().unwrap();assert!(store::load(&root,&"1".repeat(32)).is_err());assert!(!root.join(".hick-cache").exists());assert_eq!(store::list(&root).unwrap()["total"],0);assert!(!root.join(".hick-cache").exists());
}

#[cfg(unix)]
#[tokio::test]
#[ignore="requires installed pytest-cov; replaces a disposable root while real pytest is running"]
async fn real_python_root_replacement_retains_outcome_without_foreign_collector_caches(){
 use std::{sync::Arc,time::Duration};
 let temporary=tempfile::tempdir().unwrap();let root=temporary.path().join("workspace");std::fs::create_dir(&root).unwrap();let root=root.canonicalize().unwrap();let ready=temporary.path().join("ready");let release=temporary.path().join("release");
 let tests=format!("from pathlib import Path\nimport time\ndef test_held_directory():\n    Path({:?}).write_text('ready')\n    while not Path({:?}).exists(): time.sleep(0.02)\n    import late\n    Path('.hick-cache/collector-relative').write_text('original')\n    assert late.value() == 42\n",ready.to_str().unwrap(),release.to_str().unwrap());
 let sources=[("late.py","def value():\n    return 42\n".to_string()),("test_app.py",tests),("pyproject.toml","[project]\nname='owned-authority'\nversion='0.0.0'\n[tool.pytest.ini_options]\njunit_family='xunit1'\n".to_string())];
 let environment=std::env::var("HICK_COVERAGE_PYTHON_ENV").expect("HICK_COVERAGE_PYTHON_ENV must point to installed pytest-cov");
 for (path,text) in &sources{std::fs::write(root.join(path),text).unwrap();}std::os::unix::fs::symlink(&environment,root.join(".venv")).unwrap();
 let scope=Arc::new(crate::serve::resource_inputs::RootIdentity::bind(&root));let started=dispatch_scoped(&root,scope.clone(),req(json!({"action":"run","path":".","language":"python"}))).await.unwrap();let id=started["id"].as_str().unwrap();
 for _ in 0..600{if ready.exists(){break}tokio::time::sleep(Duration::from_millis(20)).await;}assert!(ready.exists(),"Real pytest body never reached the admission marker");
 let moved=temporary.path().join("original");std::fs::rename(&root,&moved).unwrap();std::fs::create_dir(&root).unwrap();for (path,text) in &sources{std::fs::write(root.join(path),text).unwrap();}std::os::unix::fs::symlink(&environment,root.join(".venv")).unwrap();std::fs::write(release,"release").unwrap();
 let mut result=json!(null);for _ in 0..600{result=dispatch_scoped(&root,scope.clone(),req(json!({"action":"get","id":id}))).await.unwrap();if result["status"]!="running"{break}tokio::time::sleep(Duration::from_millis(20)).await;}
 assert_eq!(result["status"],"completed","{result}");assert_eq!(result["tests"].as_array().unwrap().len(),1);assert!(!result["evidence"].is_null());assert_eq!(result["authority"],"historical-unverified");assert_eq!(result["source_state"],"historical");for cache in [".hick-cache",".pytest_cache","__pycache__"]{assert!(!root.join(cache).exists(),"Real collector wrote foreign cache {cache}");}assert_eq!(std::fs::read_to_string(moved.join(".hick-cache/collector-relative")).unwrap(),"original");assert!(dispatch_scoped(&root,scope,req(json!({"action":"run","path":".","language":"python"}))).await.is_err());
}

#[tokio::test]
#[ignore="requires .NET SDK10.0.401 and pinned NuGet packages/reachable restore"]
async fn real_csharp_discovery_failed_coverage_and_failed_only_rerun(){
 let temporary=tempfile::tempdir().unwrap();let root=temporary.path().canonicalize().unwrap();std::fs::create_dir(root.join("Product")).unwrap();
 std::fs::write(root.join("global.json"),r#"{"sdk":{"version":"10.0.401","rollForward":"disable"}}"#).unwrap();
 std::fs::write(root.join("App.Tests.csproj"),r#"<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net10.0</TargetFramework><IsTestProject>true</IsTestProject><Nullable>enable</Nullable></PropertyGroup><ItemGroup><Compile Remove="Product/**/*.cs"/><ProjectReference Include="Product/Product.csproj"/><PackageReference Include="Microsoft.NET.Test.Sdk" Version="17.14.1"/><PackageReference Include="xunit" Version="2.9.3"/><PackageReference Include="xunit.runner.visualstudio" Version="3.1.1"/><PackageReference Include="coverlet.collector" Version="6.0.4"/></ItemGroup></Project>"#).unwrap();
 std::fs::write(root.join("Product/Product.csproj"),r#"<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net10.0</TargetFramework></PropertyGroup></Project>"#).unwrap();std::fs::write(root.join("Product/Code.cs"),"public class Product {public static int Classify(int n){if(n<0)return -1;return 1;}}\n").unwrap();
 let events=root.join(".hick-cache/executions");let test=r#"using Xunit;using System.IO;public class App {static void Record(string name){var path=@"__EVENTS__";Directory.CreateDirectory(Path.GetDirectoryName(path)!);File.AppendAllText(path,name+"\n");}[Fact]public void Positive(){Record("positive");Assert.Equal(1,Product.Classify(1));}[Fact]public void Negative(){Record("negative");Assert.Equal(1,Product.Classify(-1));}}"#.replace("__EVENTS__",&events.to_str().unwrap().replace('"',"\"\""));std::fs::write(root.join("App.cs"),test).unwrap();let before=snapshot(&root).unwrap();
 let listing=dispatch(&root,req(json!({"action":"discover","path":".","language":"csharp"}))).await.unwrap();assert_eq!(listing["tests"].as_array().unwrap().iter().map(|test|test["id"].as_str().unwrap()).collect::<std::collections::BTreeSet<_>>(),std::collections::BTreeSet::from(["App.Positive","App.Negative"]));assert!(!events.exists(),"Discovery executed a test body");
 let started=dispatch(&root,req(json!({"action":"run","path":".","language":"csharp"}))).await.unwrap();let failed=finished(&root,started["id"].as_str().unwrap()).await;assert_eq!(failed["status"],"failed","{failed}");assert_eq!(failed["tests"].as_array().unwrap().len(),2);assert_eq!(failed["tests"].as_array().unwrap().iter().filter(|test|test["status"]=="failed").count(),1);assert!(failed["evidence"]["report"]["files"].as_object().unwrap().contains_key("Product/Code.cs"));
 let started=dispatch(&root,req(json!({"action":"rerun-failed","id":failed["id"]}))).await.unwrap();let rerun=finished(&root,started["id"].as_str().unwrap()).await;assert_eq!(rerun["status"],"failed","{rerun}");assert_eq!(rerun["tests"].as_array().unwrap().len(),1);assert_eq!(rerun["tests"][0]["id"],"App.Negative");assert!(rerun["evidence"]["report"]["files"].as_object().unwrap().contains_key("Product/Code.cs"));let mut executions=std::fs::read_to_string(events).unwrap().lines().map(str::to_string).collect::<Vec<_>>();executions.sort();assert_eq!(executions,["negative","negative","positive"]);assert_eq!(store::list(&root).unwrap()["runs"].as_array().unwrap().len(),2);assert_eq!(snapshot(&root).unwrap(),before,"Collector changed authored fixture bytes");
 let product=root.join("Product/Code.cs");let fixed=std::fs::read_to_string(&product).unwrap().replace("return -1;","return 1;");std::fs::write(&product,fixed).unwrap();let expected=snapshot(&root).unwrap();let started=dispatch(&root,req(json!({"action":"rerun-failed","id":failed["id"]}))).await.unwrap();let fixed=finished(&root,started["id"].as_str().unwrap()).await;assert_eq!(fixed["status"],"completed","{fixed}");assert_eq!(fixed["tests"].as_array().unwrap().len(),1);assert_eq!(fixed["tests"][0]["id"],"App.Negative");assert_eq!(fixed["tests"][0]["status"],"passed");assert!(!fixed["evidence"].is_null());assert!(fixed["notes"].as_str().unwrap().contains("runner discovery"));assert_eq!(store::view(&root,failed["id"].as_str().unwrap()).unwrap()["source_state"],"historical");assert_eq!(store::list(&root).unwrap()["runs"].as_array().unwrap().len(),3);assert_eq!(snapshot(&root).unwrap(),expected);let mut executions=std::fs::read_to_string(root.join(".hick-cache/executions")).unwrap().lines().map(str::to_string).collect::<Vec<_>>();executions.sort();assert_eq!(executions,["negative","negative","negative","positive"]);
 let app=root.join("App.cs");let original=std::fs::read_to_string(&app).unwrap();let split=original.rfind('}').unwrap();let mut ambiguous=original[..split].to_string();ambiguous.push_str("[Fact]public void negative(){Record(\"unexpected-case-only\");Assert.True(true);}}");std::fs::write(&app,ambiguous).unwrap();let expected=snapshot(&root).unwrap();let error=dispatch(&root,req(json!({"action":"rerun-failed","id":failed["id"]}))).await.unwrap_err();assert!(error.to_string().contains(".NET filter"),"{error:#}");let mut executions=std::fs::read_to_string(root.join(".hick-cache/executions")).unwrap().lines().map(str::to_string).collect::<Vec<_>>();executions.sort();assert_eq!(executions,["negative","negative","negative","positive"]);assert_eq!(store::list(&root).unwrap()["runs"].as_array().unwrap().len(),3);assert_eq!(snapshot(&root).unwrap(),expected);


}

#[cfg(unix)]
#[test]
fn provider_report_cleanup_keeps_held_private_parent_and_refuses_aliased_children(){
 for replacement in ["none","directory","inside-link","outside-link","leaf-link"] {
  let temporary=tempfile::tempdir().unwrap();let root=temporary.path().canonicalize().unwrap();std::fs::write(root.join("package.json"),r#"{"devDependencies":{"vitest":"3.2.7"}}"#).unwrap();let request=Request{action:"run".into(),path:".".into(),language:"typescript".into(),..Default::default()};let mut record=store::new_record(&root,&root,&request).unwrap();let plan=commands::automated(&root,&root,&mut record,&[]).unwrap();let private=plan.artifacts.as_ref().unwrap();let held=private.path().to_path_buf();let reports=plan.report.parent().unwrap().to_path_buf();assert_ne!(reports,held);assert_eq!(plan.result.parent().unwrap(),held);assert!(plan.argv.iter().any(|arg|arg=="--coverage.clean=false"));std::fs::write(&plan.report,"original").unwrap();assert_eq!(private.read(&plan.report,1024).unwrap(),"original");
  if replacement=="none" {std::fs::remove_file(&plan.report).unwrap();std::fs::write(&plan.report,"new report").unwrap();assert_eq!(private.read(&plan.report,1024).unwrap(),"new report");drop(plan);assert!(!held.exists());continue}
  let foreign=if replacement=="outside-link"{root.join("foreign-report")}else{held.join("foreign-report")};std::fs::create_dir(&foreign).unwrap();std::fs::write(foreign.join("lcov.info"),"foreign").unwrap();
  if replacement=="leaf-link" {std::fs::remove_file(&plan.report).unwrap();std::os::unix::fs::symlink(foreign.join("lcov.info"),&plan.report).unwrap();assert!(private.read(&plan.report,1024).is_err());drop(plan);continue}
  std::fs::rename(&reports,held.join("original-report")).unwrap();if replacement=="directory"{std::fs::create_dir(&reports).unwrap();std::fs::write(reports.join("lcov.info"),"foreign").unwrap();}else{std::os::unix::fs::symlink(&foreign,&reports).unwrap();}
  assert!(private.read(&plan.report,1024).is_err());assert!(private.check().is_err());drop(plan);assert!(held.exists(),"Cleanup deleted a substituted report namespace");assert_eq!(std::fs::read_to_string(foreign.join("lcov.info")).unwrap(),"foreign");std::fs::remove_dir_all(held).unwrap();
 }
}

#[cfg(unix)]
#[tokio::test]
async fn collector_source_change_during_version_preflight_refuses_before_bodies(){
 let temp=tempfile::tempdir().unwrap();let root=temp.path().canonicalize().unwrap();std::fs::write(root.join("app.py"),"before\n").unwrap();let record=store::new_record(&root,&root,&req(json!({"action":"run","language":"python","path":"."}))).unwrap();let id=record.id.clone();let artifacts=commands::PrivateArtifacts::new().unwrap();let plan=commands::Plan{argv:vec!["sh".into(),"-c".into(),"touch body-ran".into()],exact_results:false,format:"lcov".into(),report:artifacts.path().join("coverage"),result:artifacts.path().join("results"),finish:vec![],version:vec!["sh".into(),"-c".into(),"printf changed > app.py; printf version-1".into()],artifacts:Some(artifacts)};store::save(&root,&record).unwrap();runner::start(root.clone(),root.clone(),record,plan).unwrap();let result=finished(&root,&id).await;assert_eq!(result["status"],"unavailable","{result}");assert!(result["error"].as_str().unwrap().contains("Saved source changed before collector execution"));assert_eq!(result["source_changed_during_run"],true);assert!(!root.join("body-ran").exists());assert_eq!(result["source_state"],"historical");
}

#[tokio::test]
#[ignore = "requires pinned .NET SDK/xUnit/Coverlet; checks new duplicate harness admission"]
async fn real_csharp_new_harness_ambiguity_refuses_before_any_rerun_bodies(){
 let temp=tempfile::tempdir().unwrap();let root=temp.path().canonicalize().unwrap();std::fs::write(root.join("global.json"),r#"{"sdk":{"version":"10.0.401","rollForward":"disable"}}"#).unwrap();std::fs::create_dir(root.join("Product")).unwrap();std::fs::write(root.join("Product/Product.csproj"),"<Project Sdk=\"Microsoft.NET.Sdk\"><PropertyGroup><TargetFramework>net10.0</TargetFramework></PropertyGroup></Project>").unwrap();std::fs::write(root.join("Product/Code.cs"),"public class Product {public static int Classify(int n){if(n<0)return -1;return 1;}}").unwrap();
 for name in ["A","B"]{std::fs::create_dir(root.join(name)).unwrap();let project=r#"<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net10.0</TargetFramework><IsTestProject>true</IsTestProject></PropertyGroup><ItemGroup><ProjectReference Include="../Product/Product.csproj"/><PackageReference Include="Microsoft.NET.Test.Sdk" Version="17.14.1"/><PackageReference Include="xunit" Version="2.9.3"/><PackageReference Include="xunit.runner.visualstudio" Version="3.1.1"/><PackageReference Include="coverlet.collector" Version="6.0.4"/></ItemGroup></Project>"#;std::fs::write(root.join(name).join(format!("{name}.Tests.csproj")),project).unwrap();let events=root.join(format!(".hick-cache/{name}-events"));let source=r#"using Xunit;using System.IO;public class App {static void Record(string name){var path=@"__EVENTS__";Directory.CreateDirectory(Path.GetDirectoryName(path)!);File.AppendAllText(path,name+"\n");}[Fact]public void Positive(){Record("positive");Assert.Equal(1,Product.Classify(1));}[Fact]public void Negative(){Record("negative");Assert.Equal(1,Product.Classify(-1));}}"#.replace("__EVENTS__",&events.to_str().unwrap().replace('"',"\"\""));std::fs::write(root.join(name).join("App.cs"),source).unwrap();}
 std::fs::write(root.join("Cases.slnx"),"<Solution><Project Path=\"A/A.Tests.csproj\"/></Solution>").unwrap();let started=dispatch(&root,req(json!({"action":"run","path":".","language":"csharp"}))).await.unwrap();let failed=finished(&root,started["id"].as_str().unwrap()).await;assert_eq!(failed["status"],"failed","{failed}");assert!(!failed["evidence"].is_null());let before=std::fs::read(root.join(".hick-cache/A-events")).unwrap();assert!(!root.join(".hick-cache/B-events").exists());std::fs::write(root.join("Cases.slnx"),"<Solution><Project Path=\"A/A.Tests.csproj\"/><Project Path=\"B/B.Tests.csproj\"/></Solution>").unwrap();let expected=snapshot(&root).unwrap();let error=dispatch(&root,req(json!({"action":"rerun-failed","id":failed["id"]}))).await.unwrap_err();assert!(error.to_string().contains("harness"),"{error:#}");assert_eq!(std::fs::read(root.join(".hick-cache/A-events")).unwrap(),before);assert!(!root.join(".hick-cache/B-events").exists());assert_eq!(snapshot(&root).unwrap(),expected);assert_eq!(store::list(&root).unwrap()["runs"].as_array().unwrap().len(),1);assert_eq!(store::view(&root,failed["id"].as_str().unwrap()).unwrap()["source_state"],"historical");
}
