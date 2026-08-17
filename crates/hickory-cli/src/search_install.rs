//! `hick search --install-model` — the one place search touches the network.
//!
//! Same doctrine as `hick lsp install`: fetching things is never automatic.
//! Search works offline (lexically) from the first run; this download is an
//! explicit upgrade to semantic ranking, and everything lands under
//! `.hick-cache/` which `hick init` already keeps out of git.

use std::path::Path;

use anyhow::{Context as _, Result, bail};

/// Download the default Model2Vec model into the project's model folder.
/// Idempotent: files already present are kept, a partial download never
/// replaces a good file (temp + rename).
pub async fn install_model(root: &Path) -> Result<()> {
    let dir = hick_search::model_dir(root);
    std::fs::create_dir_all(&dir).with_context(|| format!("could not create {}", dir.display()))?;

    let client = reqwest::Client::new();
    for file in hick_search::MODEL_FILES {
        let target = dir.join(file);
        if target.is_file() {
            println!("  {file} — already present");
            continue;
        }
        let url = format!(
            "https://huggingface.co/{}/resolve/main/{file}",
            hick_search::DEFAULT_MODEL_REPO
        );
        println!("  fetching {url}");
        let response = client.get(&url).send().await.with_context(|| {
            format!(
                "could not reach huggingface.co to fetch {file}.\n  \
                 Search still works without the model (lexical ranking); re-run \
                 `hick search --install-model` when you are online."
            )
        })?;
        if !response.status().is_success() {
            bail!(
                "downloading {file} failed: HTTP {} from {url}\n  \
                 If the model repo moved, this binary is out of date — check for a \
                 newer release. Search still works without the model.",
                response.status()
            );
        }
        let bytes = response.bytes().await.context("download interrupted")?;
        let tmp = dir.join(format!("{file}.partial"));
        std::fs::write(&tmp, &bytes)
            .with_context(|| format!("could not write {}", tmp.display()))?;
        std::fs::rename(&tmp, &target)
            .with_context(|| format!("could not move {} into place", tmp.display()))?;
        println!("  {file} — {} KiB", bytes.len() / 1024);
    }

    println!(
        "Model installed under {}. Searches in this project now rank \
         semantically as well as lexically.",
        dir.display()
    );
    Ok(())
}
