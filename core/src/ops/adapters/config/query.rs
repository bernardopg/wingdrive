//! Get adapter config fields query

use crate::{
	context::CoreContext,
	infra::query::{LibraryQuery, QueryError, QueryResult},
};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetAdapterConfigInput {
	pub adapter_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AdapterConfigField {
	pub key: String,
	pub name: String,
	pub description: String,
	pub field_type: String,
	pub required: bool,
	pub secret: bool,
	pub default: Option<String>,
	/// `folder` or `file` when the field holds a path, so the form can offer a picker.
	pub path_type: Option<String>,
	/// Values the adapter found on this machine, best first; the form pre-fills the first.
	pub suggestions: Vec<String>,
	/// `name` and `description` in other languages, keyed by language code (`pt`).
	pub translations: std::collections::BTreeMap<String, AdapterConfigFieldText>,
}

/// A config field's label and help in one language.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AdapterConfigFieldText {
	pub name: Option<String>,
	pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetAdapterConfigQuery {
	pub input: GetAdapterConfigInput,
}

impl LibraryQuery for GetAdapterConfigQuery {
	type Input = GetAdapterConfigInput;
	type Output = Vec<AdapterConfigField>;

	fn from_input(input: Self::Input) -> QueryResult<Self> {
		if input.adapter_id.trim().is_empty() {
			return Err(QueryError::Validation {
				field: "adapter_id".to_string(),
				message: "adapter_id cannot be empty".to_string(),
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

		let fields = source_manager
			.adapter_config_fields(&self.input.adapter_id)
			.map_err(|e| QueryError::Internal(e))?;

		let translations = source_manager.config_translations(&self.input.adapter_id);
		let mut suggestions = source_manager
			.detect_config(&self.input.adapter_id)
			.await
			.unwrap_or_default();

		Ok(fields
			.into_iter()
			.map(|f| AdapterConfigField {
				suggestions: suggestions.remove(&f.key).unwrap_or_default(),
				translations: translations
					.iter()
					.filter_map(|(lang, fields)| {
						fields.get(&f.key).map(|text| {
							(
								lang.clone(),
								AdapterConfigFieldText {
									name: text.name.clone(),
									description: text.description.clone(),
								},
							)
						})
					})
					.collect(),
				path_type: f.path_type,
				key: f.key,
				name: f.name,
				description: f.description,
				field_type: f.field_type,
				required: f.required,
				secret: f.secret,
				default: f.default.map(|d| match d {
					toml::Value::String(text) => text,
					other => other.to_string(),
				}),
			})
			.collect())
	}
}

crate::register_library_query!(GetAdapterConfigQuery, "adapters.config");
