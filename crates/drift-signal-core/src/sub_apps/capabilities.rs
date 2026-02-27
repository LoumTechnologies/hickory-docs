//! Capabilities sub-app — device capability registry and cross-device dispatch.
//!
//! Each device registers the capabilities it provides (iMessage, desktop
//! automation, camera, etc.). When a capability is invoked from any device,
//! the event is synced via waymark gossip to the target device which
//! executes it locally and emits a result event.
//!
//! Event streams:
//! - `CapabilityRegistered` — device advertises a capability
//! - `CapabilityUnregistered` — device removes a capability
//! - `CapabilityInvoked` — request to execute a capability
//! - `CapabilityResult` — result of a capability execution

use std::collections::BTreeMap;

use waymark_core::{
    AnyProjection, CollectionDef, CommandDef, EventStreamDef, HandlerContext, HandlerEffect,
    Migration, NamespaceUri, ProjectionOp, SqlParam, SubApp, TagHandler,
};

use crate::capability::types::CAPABILITY_NS;

// ── Sub-app ──────────────────────────────────────────────────────────

pub struct CapabilitiesSubApp {
    namespace: NamespaceUri,
}

impl CapabilitiesSubApp {
    pub fn new() -> Self {
        Self {
            namespace: NamespaceUri::new(CAPABILITY_NS),
        }
    }
}

impl Default for CapabilitiesSubApp {
    fn default() -> Self {
        Self::new()
    }
}

impl SubApp for CapabilitiesSubApp {
    fn name(&self) -> &str {
        "capabilities"
    }

    fn namespace(&self) -> &NamespaceUri {
        &self.namespace
    }

    fn ns_prefix(&self) -> &str {
        "cap"
    }

    fn event_streams(&self) -> Vec<EventStreamDef> {
        vec![EventStreamDef {
            name: "capability_events".into(),
            doc_id: "grove:capabilities:events".into(),
            description: "Device capability registration and invocation events".into(),
        }]
    }

    fn commands(&self) -> Vec<CommandDef> {
        vec![
            CommandDef::new(
                "RegisterCapability",
                "cap:CapabilityRegistered",
                "Register a device capability",
            )
            .with_required(vec![
                "id".into(),
                "device_id".into(),
                "cap_type".into(),
            ])
            .with_optional(vec!["description".into()]),
            CommandDef::new(
                "UnregisterCapability",
                "cap:CapabilityUnregistered",
                "Remove a device capability",
            )
            .with_required(vec!["id".into()]),
            CommandDef::new(
                "InvokeCapability",
                "cap:CapabilityInvoked",
                "Request execution of a capability on a target device",
            )
            .with_required(vec![
                "id".into(),
                "cap_type".into(),
            ])
            .with_optional(vec![
                "target_device".into(),
                "params".into(),
            ]),
            CommandDef::new(
                "ReportResult",
                "cap:CapabilityResult",
                "Report the result of a capability execution",
            )
            .with_required(vec![
                "invocation_id".into(),
                "status".into(),
            ])
            .with_optional(vec!["result".into(), "error".into()]),
        ]
    }

    fn collections(&self) -> Vec<CollectionDef> {
        vec![
            CollectionDef::new("registered", "cap", 1),
            CollectionDef::new("invocations", "inv", 1),
        ]
    }

    fn projections(&self) -> Vec<Box<dyn AnyProjection>> {
        vec![Box::new(CapabilityProjection)]
    }

    fn tag_handlers(&self) -> Vec<Box<dyn TagHandler>> {
        vec![Box::new(CapabilityResultLogger)]
    }

    fn migrations(&self) -> Vec<Migration> {
        vec![Migration::new(
            1,
            "Create capability registry and invocations tables",
            "CREATE TABLE IF NOT EXISTS cap_registered (
                id TEXT PRIMARY KEY,
                device_id TEXT NOT NULL,
                cap_type TEXT NOT NULL,
                description TEXT DEFAULT '',
                registered_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_cap_registered_type
                ON cap_registered(cap_type);
            CREATE INDEX IF NOT EXISTS idx_cap_registered_device
                ON cap_registered(device_id);
            CREATE TABLE IF NOT EXISTS cap_invocations (
                id TEXT PRIMARY KEY,
                cap_type TEXT NOT NULL,
                target_device TEXT,
                params TEXT,
                status TEXT NOT NULL DEFAULT 'pending',
                result TEXT,
                error TEXT,
                invoked_at TEXT NOT NULL,
                completed_at TEXT
            );",
        )]
    }

    fn expected_columns(&self) -> Vec<(&str, Vec<(String, String)>)> {
        vec![
            (
                "cap_registered",
                vec![
                    ("id".into(), "TEXT PRIMARY KEY".into()),
                    ("device_id".into(), "TEXT NOT NULL".into()),
                    ("cap_type".into(), "TEXT NOT NULL".into()),
                    ("description".into(), "TEXT DEFAULT ''".into()),
                    ("registered_at".into(), "TEXT NOT NULL".into()),
                ],
            ),
            (
                "cap_invocations",
                vec![
                    ("id".into(), "TEXT PRIMARY KEY".into()),
                    ("cap_type".into(), "TEXT NOT NULL".into()),
                    ("target_device".into(), "TEXT".into()),
                    ("params".into(), "TEXT".into()),
                    ("status".into(), "TEXT NOT NULL DEFAULT 'pending'".into()),
                    ("result".into(), "TEXT".into()),
                    ("error".into(), "TEXT".into()),
                    ("invoked_at".into(), "TEXT NOT NULL".into()),
                    ("completed_at".into(), "TEXT".into()),
                ],
            ),
        ]
    }
}

// ── Projection ───────────────────────────────────────────────────────

struct CapabilityProjection;

impl AnyProjection for CapabilityProjection {
    fn name(&self) -> &str {
        "capability_projection"
    }

    fn project_event(
        &self,
        tag: &str,
        attrs: &BTreeMap<String, String>,
        _text: Option<&str>,
        event_id: &str,
        timestamp: i64,
    ) -> Vec<ProjectionOp> {
        let _ = event_id;
        let ts = chrono::DateTime::from_timestamp(timestamp, 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_default();

        match tag {
            t if t.ends_with(":CapabilityRegistered") => {
                let id = attrs.get("id").cloned().unwrap_or_default();
                let device_id = attrs.get("device_id").cloned().unwrap_or_default();
                let cap_type = attrs.get("cap_type").cloned().unwrap_or_default();
                let description = attrs.get("description").cloned().unwrap_or_default();
                vec![ProjectionOp::sql(
                    "INSERT OR REPLACE INTO cap_registered (id, device_id, cap_type, description, registered_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                    vec![
                        SqlParam::Text(id),
                        SqlParam::Text(device_id),
                        SqlParam::Text(cap_type),
                        SqlParam::Text(description),
                        SqlParam::Text(ts),
                    ],
                )]
            }
            t if t.ends_with(":CapabilityUnregistered") => {
                let id = attrs.get("id").cloned().unwrap_or_default();
                vec![ProjectionOp::sql(
                    "DELETE FROM cap_registered WHERE id = ?1",
                    vec![SqlParam::Text(id)],
                )]
            }
            t if t.ends_with(":CapabilityInvoked") => {
                let id = attrs.get("id").cloned().unwrap_or_default();
                let cap_type = attrs.get("cap_type").cloned().unwrap_or_default();
                let target_device = attrs.get("target_device").cloned();
                let params = attrs.get("params").cloned();
                vec![ProjectionOp::sql(
                    "INSERT OR REPLACE INTO cap_invocations (id, cap_type, target_device, params, status, invoked_at) VALUES (?1, ?2, ?3, ?4, 'pending', ?5)",
                    vec![
                        SqlParam::Text(id),
                        SqlParam::Text(cap_type),
                        target_device.map_or(SqlParam::Null, SqlParam::Text),
                        params.map_or(SqlParam::Null, SqlParam::Text),
                        SqlParam::Text(ts),
                    ],
                )]
            }
            t if t.ends_with(":CapabilityResult") => {
                let invocation_id = attrs.get("invocation_id").cloned().unwrap_or_default();
                let status = attrs.get("status").cloned().unwrap_or_default();
                let result = attrs.get("result").cloned();
                let error = attrs.get("error").cloned();
                vec![ProjectionOp::sql(
                    "UPDATE cap_invocations SET status = ?1, result = ?2, error = ?3, completed_at = ?4 WHERE id = ?5",
                    vec![
                        SqlParam::Text(status),
                        result.map_or(SqlParam::Null, SqlParam::Text),
                        error.map_or(SqlParam::Null, SqlParam::Text),
                        SqlParam::Text(ts),
                        SqlParam::Text(invocation_id),
                    ],
                )]
            }
            _ => vec![],
        }
    }
}

// ── Tag handler: log capability results ──────────────────────────────

struct CapabilityResultLogger;

impl TagHandler for CapabilityResultLogger {
    fn tag(&self) -> &str {
        "cap:CapabilityResult"
    }

    fn handle_event(
        &self,
        _tag: &str,
        attrs: &BTreeMap<String, String>,
        _text: Option<&str>,
        _ctx: &HandlerContext,
    ) -> Result<Vec<HandlerEffect>, waymark_core::SyncError> {
        let invocation_id = attrs.get("invocation_id").cloned().unwrap_or_default();
        let status = attrs.get("status").cloned().unwrap_or_default();
        Ok(vec![HandlerEffect::Log(format!(
            "Capability invocation {invocation_id} completed with status: {status}"
        ))])
    }
}
