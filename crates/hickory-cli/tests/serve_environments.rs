// Protects docs/guarantees/editor-intelligence/a-project-environment-offers-its-own-sync.md
//! Real package manager, real HTTP action, real terminal, selected interpreter.
use hickory_cli::{
    ExecutorChoice,
    serve::{LocalState, ServeOptions, prepare},
};
use serde_json::{Value, json};
use std::{path::Path, process::Command, time::Duration};

async fn start(root: &Path) -> (String, LocalState, tokio::task::JoinHandle<()>) {
    let prepared = prepare(ServeOptions {
        target: root.to_path_buf(),
        port: 0,
        params: vec![],
        executor: ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: None,
    })
    .await
    .unwrap();
    let state = prepared.state;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });
    (url, state, task)
}

async fn status(url: &str) -> Value {
    reqwest::get(format!("{url}/api/environments?refresh=true"))
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap()
}

fn run(root: &Path, program: &Path, args: &[&str]) {
    let out = Command::new(program)
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[tokio::test]
async fn uv_sync_in_a_terminal_prepares_the_interpreter_used_by_project_tests() {
    let Some(uv) = hick_project_env::host::Host::default().executable("uv") else {
        eprintln!("uv is unavailable; real-manager integration is exercised on the uv CI job");
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let found = Command::new(&uv)
        .args(["python", "find", "--offline"])
        .output()
        .unwrap();
    assert!(found.status.success());
    let python = std::path::PathBuf::from(String::from_utf8(found.stdout).unwrap().trim());
    // A local wheel avoids network and caches. The dependency is only importable
    // from the environment prepared by the button, not from system Python.
    run(
        &root,
        &python,
        &[
            "-c",
            r#"
import zipfile
with zipfile.ZipFile('fixture_dep-0.1.0-py3-none-any.whl', 'w') as z:
    z.writestr('fixture_dep.py', 'VALUE = 42\n')
    z.writestr('fixture_dep-0.1.0.dist-info/METADATA', 'Metadata-Version: 2.1\nName: fixture-dep\nVersion: 0.1.0\n')
    z.writestr('fixture_dep-0.1.0.dist-info/WHEEL', 'Wheel-Version: 1.0\nGenerator: fixture\nRoot-Is-Purelib: true\nTag: py3-none-any\n')
    z.writestr('fixture_dep-0.1.0.dist-info/RECORD', '')
"#,
        ],
    );
    std::fs::write(root.join("pyproject.toml"), "[project]\nname='environment-fixture'\nversion='0.1.0'\nrequires-python='>=3.11'\ndependencies=['fixture-dep']\n[tool.uv.sources]\nfixture-dep={path='fixture_dep-0.1.0-py3-none-any.whl'}\n").unwrap();
    std::fs::write(
        root.join("test_fixture.py"),
        "def test_dependency():\n    import fixture_dep\n    assert fixture_dep.VALUE == 42\n",
    )
    .unwrap();
    run(&root, &uv, &["lock", "--offline", "--no-python-downloads"]);
    let lock = std::fs::read(root.join("uv.lock")).unwrap();
    let manifest = std::fs::read(root.join("pyproject.toml")).unwrap();
    let (url, state, server) = start(&root).await;
    let initial = status(&url).await;
    let finding = &initial["findings"][0];
    assert_eq!(finding["state"], "environment-missing", "{finding:#}");
    assert!(
        !root.join(".venv").exists(),
        "checking created an environment"
    );
    assert_eq!(std::fs::read(root.join("uv.lock")).unwrap(), lock);
    let body = json!({"project":finding["project"],"manager":finding["manager"],"revision":finding["revision"],"action":finding["actions"][0]["id"]});
    let response = reqwest::Client::new()
        .post(format!("{url}/api/environments/actions"))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert!(
        response.status().is_success(),
        "{}",
        response.text().await.unwrap()
    );
    let session: Value = response.json().await.unwrap();
    let terminal = state
        .terminals
        .get(session["id"].as_str().unwrap())
        .unwrap();
    tokio::time::timeout(Duration::from_secs(20), async {
        while terminal.exit_code().is_none() {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .unwrap();
    assert_eq!(terminal.exit_code(), Some(0), "{}", terminal.screen_text());
    assert!(
        !terminal.attach().0.is_empty(),
        "installation output was discarded"
    );
    let ready = status(&url).await;
    assert_eq!(ready["findings"][0]["state"], "ready", "{ready:#}");
    let command = hickory_cli::serve::test_run::test_command(
        &root,
        &root.join("test_fixture.py"),
        "test_dependency",
        "python",
    )
    .unwrap();
    assert!(command.argv[0].contains(".venv"), "{command:?}");
    run(
        &root,
        Path::new(&command.argv[0]),
        &["-c", "import test_fixture; test_fixture.test_dependency()"],
    );
    assert_eq!(std::fs::read(root.join("uv.lock")).unwrap(), lock);
    assert_eq!(
        std::fs::read(root.join("pyproject.toml")).unwrap(),
        manifest
    );

    // A stale action must not launch again after the environment changes.
    let response = reqwest::Client::new()
        .post(format!("{url}/api/environments/actions"))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 409);
    // An externally changed manifest produces its own lockfile diagnosis.
    std::fs::write(
        root.join("pyproject.toml"),
        String::from_utf8(manifest)
            .unwrap()
            .replace("dependencies=['fixture-dep']", "dependencies=[]"),
    )
    .unwrap();
    let stale = status(&url).await;
    assert_eq!(stale["findings"][0]["state"], "lock-stale", "{stale:#}");
    server.abort();
}

#[tokio::test]
async fn action_routes_reject_undiscovered_paths_and_arbitrary_commands() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let (url, _, server) = start(&root).await;
    let response = reqwest::Client::new()
        .post(format!("{url}/api/environments/actions"))
        .json(&json!({"project":"/tmp","revision":"anything","action":"rm -rf","manager":"uv"}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 404);
    server.abort();
}
