//! Source config query: what the edit form starts from.

use crate::{
	context::CoreContext,
	infra::query::{LibraryQuery, QueryError, QueryResult},
};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetSourceConfigInput {
	pub source_id: String,
}

/// A source's config without its secrets.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SourceConfig {
	pub adapter_id: String,
	/// Stored config values. Secret fields never appear here.
	pub config: serde_json::Value,
	/// Secret fields that have a stored value, so the form can say "saved".
	pub stored_secrets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSourceConfigQuery {
	input: GetSourceConfigInput,
}

impl LibraryQuery for GetSourceConfigQuery {
	type Input = GetSourceConfigInput;
	type Output = SourceConfig;

	fn from_input(input: Self::Input) -> QueryResult<Self> {
		if input.source_id.trim().is_empty() {
			return Err(QueryError::Validation {
				field: "source_id".to_string(),
				message: "source_id cannot be empty".to_string(),
			});
		}
		Ok(Self { input })
	}

	async fn execute(
		self,
		context: Arc<CoreContext>,
		session: crate::infra::api::SessionContext,
	) -> QueryResult<Self::Output> {
		let library_id = session
			.current_library_id
			.ok_or_else(|| QueryError::Internal("No library in session".to_string()))?;
		let library = context
			.libraries()
			.await
			.get_library(library_id)
			.await
			.ok_or_else(|| QueryError::Internal("Library not found".to_string()))?;
		if library.source_manager().is_none() {
			library
				.init_source_manager()
				.await
				.map_err(|e| QueryError::Internal(format!("Failed to init source manager: {e}")))?;
		}
		let source_manager = library
			.source_manager()
			.ok_or_else(|| QueryError::Internal("Source manager not available".to_string()))?;

		source_manager
			.source_config(&self.input.source_id)
			.await
			.map(|(adapter_id, config, stored_secrets)| SourceConfig {
				adapter_id,
				config,
				stored_secrets,
			})
			.map_err(QueryError::Internal)
	}
}

crate::register_library_query!(GetSourceConfigQuery, "sources.config");
