use serde::Serialize;

// Re-export types from wing-core
pub use wing_core::domain::addressing::SdPath;
pub use wing_core::domain::content_identity::ContentIdentity;
pub use wing_core::domain::file::{File, Sidecar};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct QueryRequest {
	pub method: String,
	pub library_id: Option<String>,
	pub payload: serde_json::Value,
}
