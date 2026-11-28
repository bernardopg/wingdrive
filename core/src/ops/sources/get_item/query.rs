//! Single source item query implementation

use crate::{
	context::CoreContext,
	infra::query::{LibraryQuery, QueryError, QueryResult},
	ops::sources::SourceItem,
};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GetSourceItemInput {
	pub source_id: String,
	/// The item's id in its source, as `sources.list_items` and `sources.search` return it.
	pub item_id: String,
}

/// Reads one item with its whole preview, for the item a person opened from a list or a search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSourceItemQuery {
	pub input: GetSourceItemInput,
}

impl LibraryQuery for GetSourceItemQuery {
	type Input = GetSourceItemInput;
	type Output = Option<SourceItem>;

	fn from_input(input: Self::Input) -> QueryResult<Self> {
		for (field, value) in [("source_id", &input.source_id), ("item_id", &input.item_id)] {
			if value.trim().is_empty() {
				return Err(QueryError::Validation {
					field: field.to_string(),
					message: format!("{field} cannot be empty"),
				});
			}
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

		let item = source_manager
			.get_item(&self.input.source_id, &self.input.item_id)
			.await
			.map_err(QueryError::Internal)?;
		Ok(item.map(|item| SourceItem {
			id: item.id,
			external_id: item.external_id,
			title: item.title,
			preview: item.preview,
			subtitle: item.subtitle,
			date: item.date,
		}))
	}
}

crate::register_library_query!(GetSourceItemQuery, "sources.get_item");
