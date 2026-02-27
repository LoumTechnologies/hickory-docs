//! Drift Signal — unified app core for multi-frontend agent with waymark sync.
//!
//! This crate contains:
//! - Domain sub-apps (chat, memory, task) as waymark SubApp implementations
//! - Capabilities sub-app for cross-device invocation
//! - `AppDriftSignal` facade wrapping SyncApp + agent engine
//!
//! Each frontend (web, iOS, macOS, Android) embeds this crate and provides
//! its own UI layer on top.

pub mod app;
pub mod capability;
pub mod query;
pub mod sub_apps;

pub use app::{AppDriftSignal, DriftSignalConfig};
pub use capability::{
    AppCapabilityDispatcher, AppCapabilityHandler, AppDeviceCapability, new_pending_queue,
};
pub use capability::handlers::FileSystemCapabilityHandler;
pub use sub_apps::{CapabilitiesSubApp, ChatSubApp, MemorySubApp, TaskSubApp};

// Re-export key waymark types for downstream use
pub use waymark_core::{EntityId, NamespaceUri, SyncError};
pub use waymark_sdk::SyncApp;
