<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/tests/open_selected_file.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `461eb02f890511c3f26a56d2a8e22c2073f409ea15355c7feafd3486cc73817d`.

<book:entry path="crates/hickory-cli/tests/open_selected_file.rs">
//! File-menu selection over real HTTP; the workspace remains open.
use hickory_cli::{
    ExecutorChoice,
    serve::{ServeOptions, prepare, prepare_without_folder},
};
use serde_json::{Value, json};

#[tokio::test]
async fn selected_files_open_and_save_in_the_existing_session() {
    for folder_open in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let plain = outside.path().join("note.txt");
        std::fs::write(&plain, "first").unwrap();
        let plain = plain.canonicalize().unwrap();
        let document = outside.path().join("note.md");
        std::fs::write(&document, "# A note\n").unwrap();
        let opts = ServeOptions {
            target: root.path().to_path_buf(),
            port: 0,
            params: vec![],
            executor: ExecutorChoice::Local,
            key_store_path: None,
            ui_settings_path: None,
        };
        let prepared = if folder_open {
            prepare(opts).await
        } else {
            prepare_without_folder(opts).await
        }
        .unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let task =
            tokio::spawn(async move { axum::serve(listener, prepared.router).await.unwrap() });
        let http = reqwest::Client::new();
        assert_eq!(
            http.get(format!("{url}/api/file"))
                .query(&[("path", &plain)])
                .send()
                .await
                .unwrap()
                .status(),
            403
        );
        let opened = http
            .post(format!("{url}/api/open-file"))
            .json(&json!({"path": plain}))
            .send()
            .await
            .unwrap();
        assert!(opened.status().is_success());
        let path = opened.json::<Value>().await.unwrap()["path"]
            .as_str()
            .unwrap()
            .to_string();
        let loaded = http
            .get(format!("{url}/api/file"))
            .query(&[("path", &path)])
            .send()
            .await
            .unwrap()
            .json::<Value>()
            .await
            .unwrap();
        assert_eq!(loaded["content"], "first");
        let saved = http
            .put(format!("{url}/api/file"))
            .json(&json!({"path": path, "content": "second", "base_hash": loaded["hash"]}))
            .send()
            .await
            .unwrap();
        assert!(saved.status().is_success());
        assert_eq!(std::fs::read_to_string(&plain).unwrap(), "second");
        let selected = http
            .post(format!("{url}/api/open-file"))
            .json(&json!({"path": document}))
            .send()
            .await
            .unwrap()
            .json::<Value>()
            .await
            .unwrap();
        let doc = http
            .get(format!(
                "{url}/api/docs/{}",
                selected["doc_id"].as_str().unwrap()
            ))
            .send()
            .await
            .unwrap()
            .json::<Value>()
            .await
            .unwrap();
        assert_eq!(doc["source"], "# A note\n");
        let projects = http
            .get(format!("{url}/api/projects"))
            .send()
            .await
            .unwrap()
            .json::<Value>()
            .await
            .unwrap();
        let new_path = outside.path().join("new.md");
        let created = http
            .post(format!(
                "{url}/api/projects/{}/docs",
                projects[0]["id"].as_str().unwrap()
            ))
            .json(&json!({"path": new_path, "source": "# New note\n"}))
            .send()
            .await
            .unwrap();
        assert_eq!(
            created.status(),
            201,
            "saving outside must work even without a shell restart hook"
        );
        assert_eq!(std::fs::read_to_string(new_path).unwrap(), "# New note\n");
        let binary = outside.path().join("binary.txt");
        std::fs::write(&binary, [0xff]).unwrap();
        assert_eq!(
            http.post(format!("{url}/api/open-file"))
                .json(&json!({"path": binary}))
                .send()
                .await
                .unwrap()
                .status(),
            422
        );
        let files = http
            .get(format!("{url}/api/files"))
            .send()
            .await
            .unwrap()
            .json::<Value>()
            .await
            .unwrap();
        assert_eq!(files["folder_open"], folder_open);
        task.abort();
    }
}
</book:entry>

</book:doc>
