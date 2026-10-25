<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/tests/notebook.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `66d2e8ff2ffe9e2ff69f3dd7765d5e5cc7eba26c9a32a32cc152be0d826a38d0`.

<book:entry path="crates/hickory-cli/tests/notebook.rs">
//! Notebook cells run through the same public path as the CLI.
use hickory_cli::{ExecutorChoice, RunMode, run_doc};

#[tokio::test(flavor = "multi_thread")]
async fn python_fence_displays_and_persists_outputs_with_xml_attributes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notebook.md");
    std::fs::write(&path, "```python exec=\"true\" id=\"summary\"\nprint('before')\ndisplay('hello')\ndisplay('<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>')\n```\n").unwrap();
    let run = run_doc(&path, &[], RunMode::Execute, ExecutorChoice::Local)
        .await
        .unwrap();
    let entry = run.result.transcripts.values().flatten().next().unwrap();
    assert!(entry.output.contains("before"));
    assert!(entry.output.contains("HICKORY_DISPLAY"));
    assert!(entry.output.contains("image/svg+xml"));
    assert!(hickory_cli::output::refresh_outputs(&run).unwrap() > 0);
    let saved = std::fs::read_to_string(&path).unwrap();
    assert!(saved.contains("output-for=\"#summary\""));
    let doc = hick_lang::parse(&saved).unwrap();
    assert_eq!(doc.find_tags("exec").len(), 1);
    assert_eq!(doc.find_tags("output").len(), 1);
    let again = run_doc(&path, &[], RunMode::Execute, ExecutorChoice::Local)
        .await
        .unwrap();
    assert!(hickory_cli::output::verify_outputs(&again).is_empty());
    hickory_cli::output::refresh_outputs(&again).unwrap();
    assert_eq!(
        hick_lang::parse(&std::fs::read_to_string(path).unwrap())
            .unwrap()
            .find_tags("output")
            .len(),
        1
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn preview_reads_real_parquet_and_delta_with_bounded_pages() {
    use hickory_cli::serve::{ServeOptions, prepare};
    use serde_json::Value;
    let python = std::env::var("HICKORY_NOTEBOOK_TEST_PYTHON").unwrap_or_else(|_| "python3".into());
    if !std::process::Command::new(&python)
        .args(["-c", "import polars, deltalake"])
        .output()
        .is_ok_and(|o| o.status.success())
    {
        eprintln!(
            "Install notebook requirements and set HICKORY_NOTEBOOK_TEST_PYTHON to run dataset integration"
        );
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    std::fs::write(root.join("notes.md"), "# Notes\n").unwrap();
    std::fs::write(
        root.join("pyproject.toml"),
        "[project]\nname='notebook-test'\nversion='0.0.0'\n",
    )
    .unwrap();
    let destination = root.join(if cfg!(windows) {
        ".venv/Scripts/python.exe"
    } else {
        ".venv/bin/python"
    });
    std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
    let executable = std::process::Command::new(&python)
        .args(["-c", "import sys; print(sys.executable)"])
        .output()
        .unwrap();
    let executable = String::from_utf8(executable.stdout).unwrap();
    // A script wrapper keeps the chosen interpreter's environment intact.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::write(
            &destination,
            format!(
                "#!/bin/sh\nexec '{}' \"$@\"\n",
                executable.trim().replace('\'', "'\\''")
            ),
        )
        .unwrap();
        std::fs::set_permissions(&destination, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    #[cfg(windows)]
    std::fs::copy(executable.trim(), &destination).unwrap();
    assert!(std::process::Command::new(&python).current_dir(&root).args(["-c", "import polars as pl; from deltalake import DeltaTable; f=pl.DataFrame({'n':range(205)}); f.write_parquet('data.parquet'); f.write_delta('delta'); DeltaTable('delta').delete('n < 5')"]).status().unwrap().success());
    let prepared = prepare(ServeOptions {
        target: root,
        port: 0,
        params: vec![],
        executor: ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: None,
    })
    .await
    .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });
    let client = reqwest::Client::new();
    for (src, format, first) in [("data.parquet", "parquet", 0), ("delta", "delta", 5)] {
        let response = client
            .get(format!("{base}/api/dataset"))
            .query(&[("src", src), ("format", format)])
            .send()
            .await
            .unwrap();
        assert!(
            response.status().is_success(),
            "{}",
            response.text().await.unwrap()
        );
        let data: Value = response.json().await.unwrap();
        assert_eq!(data["schema"][0]["dtype"], "Int64");
        assert_eq!(data["rows"][0][0], first);
        assert_eq!(data["rows"].as_array().unwrap().len(), 100);
        assert_eq!(data["has_more"], true);
    }
    let last: Value = client
        .get(format!("{base}/api/dataset?src=data.parquet&offset=200"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(last["rows"].as_array().unwrap().len(), 5);
    assert_eq!(last["has_more"], false);
    for query in [
        "src=../outside.parquet",
        "src=data.parquet&format=unknown",
        "src=s3://user@bucket/key&format=delta",
        "src=data.parquet&offset=10000001",
    ] {
        assert!(
            client
                .get(format!("{base}/api/dataset?{query}"))
                .send()
                .await
                .unwrap()
                .status()
                .is_client_error()
        );
    }
    server.abort();
}
</book:entry>

</book:doc>
