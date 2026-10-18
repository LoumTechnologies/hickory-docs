//! Bounded, read-only dataset previews for notebook embeds.
use super::{
    LocalState,
    api::{ApiError, ApiResult},
};
use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::io::AsyncWriteExt;

#[derive(Deserialize)]
pub struct DatasetQuery {
    pub src: String,
    pub format: Option<String>,
    pub doc_path: Option<String>,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

pub async fn preview(
    State(state): State<LocalState>,
    Query(q): Query<DatasetQuery>,
) -> ApiResult<Json<Value>> {
    let root = state
        .index
        .root()
        .canonicalize()
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let format = q.format.as_deref().unwrap_or("parquet");
    if !matches!(format, "parquet" | "delta") {
        return Err(ApiError::bad_request("format must be parquet or delta"));
    }
    let src = if q.src.starts_with("s3://") {
        // Credentials are resolved by the reader from the host environment.
        // Never accept credentials in document attributes or return them.
        if q.src.contains('@') || q.src.contains('?') || q.src.contains('#') {
            return Err(ApiError::bad_request(
                "S3 references must be bucket/key paths without credentials or query strings",
            ));
        }
        q.src.clone()
    } else {
        if q.src.contains("://") || std::path::Path::new(&q.src).is_absolute() {
            return Err(ApiError::bad_request(
                "use a document-relative path or s3://bucket/key",
            ));
        }
        let parent = q
            .doc_path
            .as_deref()
            .and_then(|p| std::path::Path::new(p).parent())
            .unwrap_or(std::path::Path::new(""));
        let path = root
            .join(parent)
            .join(&q.src)
            .canonicalize()
            .map_err(|_| ApiError::not_found("dataset does not exist"))?;
        if !path.starts_with(&root) {
            return Err(ApiError::forbidden(
                "dataset resolves outside the open folder",
            ));
        }
        path.to_string_lossy().into_owned()
    };
    let limit = q.limit.unwrap_or(100).clamp(1, 1000);
    let offset = q.offset.unwrap_or(0);
    if offset > 10_000_000 {
        return Err(ApiError::bad_request("preview offset exceeds 10000000"));
    }
    let python = hick_project_env::python_interpreter(&root, &root)
        .or_else(|| {
            // A conventional venv does not require a particular package manager.
            let path = root.join(if cfg!(windows) {
                ".venv/Scripts/python.exe"
            } else {
                ".venv/bin/python"
            });
            path.is_file().then_some(path)
        })
        .unwrap_or_else(|| {
            if cfg!(windows) {
                "python".into()
            } else {
                "python3".into()
            }
        });
    let program = concat!(
        include_str!("../../../hick-exec/python/notebook.py"),
        "\n",
        include_str!("../../../hick-exec/python/dataset.py")
    );
    let mut child = tokio::process::Command::new(python)
        .arg("-c")
        .arg(program)
        .current_dir(&root)
        .kill_on_drop(true)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| ApiError::unprocessable(format!("Cannot start notebook Python: {e}")))?;
    let request = json!({"src": src, "format": format, "limit": limit, "offset": offset});
    let mut stdin = child.stdin.take().unwrap();
    stdin
        .write_all(request.to_string().as_bytes())
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    drop(stdin);
    let output = tokio::time::timeout(std::time::Duration::from_secs(30), child.wait_with_output())
        .await
        .map_err(|_| ApiError::unprocessable("Dataset preview timed out after 30 seconds"))?
        .map_err(|e| ApiError::internal(e.to_string()))?;
    if !output.status.success() {
        // Reader errors can contain signed URLs/credentials. Show a useful
        // generic failure, except for the fixed dependency installation hint.
        let hint = if String::from_utf8_lossy(&output.stderr).contains("Install notebook readers") {
            "Install polars, deltalake and pyarrow in the project's Python environment."
        } else {
            "Could not read dataset. Check its format, permissions and local/S3 credentials."
        };
        return Err(ApiError::unprocessable(hint));
    }
    if output.stdout.len() > 8 * 1024 * 1024 {
        return Err(ApiError::unprocessable(
            "Preview exceeds 8 MiB; request fewer rows",
        ));
    }
    let value = serde_json::from_slice(&output.stdout)
        .map_err(|_| ApiError::internal("Invalid dataset reader response"))?;
    Ok(Json(value))
}
