//! The hosted [`DocStore`]: a room's durable state in Postgres and git.
//!
//! `hickory-collab` owns everything about a collaborative room except *where
//! the two persisted values live*. Here they live in the `docs` row
//! (`source` + `crdt_state`, migration 0003) and, for the text alone, in the
//! project's git repository — which is the durable truth per
//! `architecture.md`, the database being the fast path the API reads.
//!
//! The local counterpart, which puts both in the filesystem, is
//! `hickory_cli::serve` (see `docs/specs/freeform/local-collaboration.md`).

use std::sync::Arc;

use anyhow::{Context as _, Result};
use async_trait::async_trait;
use hickory_collab::{DocKey, DocStore};
use uuid::Uuid;

use crate::gitstore::GitStore;

pub struct PostgresDocStore {
    db: sqlx::PgPool,
    git: GitStore,
}

impl PostgresDocStore {
    pub fn new(db: sqlx::PgPool, git: GitStore) -> Arc<Self> {
        Arc::new(Self { db, git })
    }
}

/// Room keys are doc ids. Parsing here rather than at every call site keeps
/// the collab crate free of any opinion about what a key means.
fn doc_id(key: &DocKey) -> Result<Uuid> {
    Uuid::parse_str(key).with_context(|| format!("room key {key:?} is not a document id"))
}

#[async_trait]
impl DocStore for PostgresDocStore {
    async fn load_source(&self, key: &DocKey) -> Result<String> {
        let id = doc_id(key)?;
        sqlx::query_scalar::<_, String>("SELECT source FROM docs WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.db)
            .await?
            .with_context(|| format!("document {id} no longer exists"))
    }

    async fn load_crdt(&self, key: &DocKey) -> Result<Option<Vec<u8>>> {
        let id = doc_id(key)?;
        Ok(
            sqlx::query_scalar::<_, Option<Vec<u8>>>("SELECT crdt_state FROM docs WHERE id = $1")
                .bind(id)
                .fetch_optional(&self.db)
                .await?
                .flatten(),
        )
    }

    async fn save(&self, key: &DocKey, source: &str, crdt: &[u8]) -> Result<()> {
        let id = doc_id(key)?;
        // The CRDT state is stored WITH the text so a re-created room resumes
        // this operation history instead of minting a rival one (0003).
        let row: Option<(Uuid, String)> = sqlx::query_as(
            "UPDATE docs SET source = $1, crdt_state = $2, updated_at = now()
             WHERE id = $3
             RETURNING project_id, path",
        )
        .bind(source)
        .bind(crdt)
        .bind(id)
        .fetch_optional(&self.db)
        .await?;

        let Some((project_id, path)) = row else {
            anyhow::bail!("document {id} no longer exists");
        };

        // Git is the durable truth, but a failure to commit must not lose the
        // row write that already succeeded — report and carry on, exactly as
        // this did when it lived in ws.rs.
        if let Err(e) = self
            .git
            .save_file(
                project_id,
                &path,
                source,
                &format!("Collaborative edit: {path}"),
            )
            .await
        {
            log::error!("git persist for doc {id} failed: {e:#}");
        }
        Ok(())
    }
}
