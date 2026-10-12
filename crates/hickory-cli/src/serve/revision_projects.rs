//! Freeze revision providers into the one portable comparison tree contract.
use super::{
    LocalState,
    api::{ApiError, ApiResult},
};
use axum::{Json, extract::State};
use hick_lang::{ProjectInventory, ProjectSnapshot, ProjectTree};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Revision {
    Git {
        revision: String,
    },
    Mercurial {
        revision: String,
    },
    Directory {
        worktree: Option<String>,
    },
    Crdt {
        identity: String,
        updates: Vec<(String, Vec<u8>)>,
    },
}
pub async fn capture(
    State(state): State<LocalState>,
    Json(body): Json<Revision>,
) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    let value=tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let (identity,tree)=match body {
            Revision::Git{revision} => {
                let tree=if revision==":empty" {ProjectTree::default()} else {crate::project_revision::git(&root,&revision)?};
                (json!({"kind":"git","revision":revision}),tree)
            },
            Revision::Mercurial{revision} => (json!({"kind":"mercurial","revision":revision}),crate::project_revision::mercurial(&root,&revision)?),
            Revision::Directory{worktree} => {
                let path=if let Some(name)=&worktree {
                    let matches:Vec<_>=super::merged::worktrees(&root).into_iter().filter(|w|&w.name==name).collect();
                    anyhow::ensure!(matches.len()==1,"Worktree must name one unambiguous directory: {name}");
                    std::path::PathBuf::from(&matches[0].path)
                } else {root};
                (json!({"kind":"directory","identity":path}),crate::project_revision::directory(&path,Some(&ProjectInventory::default()))?)
            },
            Revision::Crdt{identity,updates} => (json!({"kind":"crdt","snapshot":identity}),crate::project_revision::persisted_crdt(&updates)?),
        };
        // Native contexts consume output inventory, never VCS private metadata.
        let tree=ProjectInventory::default().select(&tree).map_err(anyhow::Error::msg)?;
        let files:Vec<_>=tree.entries().iter().map(|(path,e)|json!({"path":path,"bytes":e.bytes.as_ref(),"kind":if e.kind==hick_lang::EntryKind::File{"file"}else{"symlink"},"executable":e.executable,"origins":e.origins})).collect();
        Ok(json!({"identity":identity,"snapshot":ProjectSnapshot::capture(&tree),"files":files,"evidence":"unverified","immutable":true}))
    }).await.map_err(|e|ApiError::internal(e.to_string()))?.map_err(ApiError::from)?;
    Ok(Json(value))
}
