//! # Source search
//!
//! `sources.search` finds items of the library's sources by the words a person typed, so the
//! library search reaches notes, issues and sessions that adapters archived, not only files.
//! Each source keeps its own FTS5 index; the archive engine asks every one of them and merges
//! the hits best match first. Only records the safety screening let into the index can match,
//! so a quarantined record is never returned.
//!
//! ## Example
//! ```rust,no_run
//! use wing_core::ops::sources::{SearchSourcesInput, SearchSourcesQuery};
//! use wing_core::infra::query::LibraryQuery;
//!
//! let query = SearchSourcesQuery::from_input(SearchSourcesInput {
//! 	query: "weekly review".to_string(),
//! 	source_id: None,
//! 	limit: Some(20),
//! })?;
//! # Ok::<(), wing_core::infra::query::QueryError>(())
//! ```

use crate::{
	context::CoreContext,
	infra::query::{LibraryQuery, QueryError, QueryResult},
};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::Arc;

/// Hits returned when the input names no limit.
pub const DEFAULT_SOURCE_SEARCH_LIMIT: u32 = 50;
/// Most hits one search returns.
pub const MAX_SOURCE_SEARCH_LIMIT: u32 = 200;
/// Characters of an item's preview a hit carries; `sources.get_item` returns the whole item.
pub const SOURCE_HIT_PREVIEW_CHARS: usize = 280;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SearchSourcesInput {
	/// Words to find. Every word must match, as a prefix; FTS5 syntax is searched as text.
	pub query: String,
	/// Search one source only.
	pub source_id: Option<String>,
	/// Most hits to return, `DEFAULT_SOURCE_SEARCH_LIMIT` when absent, at most `MAX_SOURCE_SEARCH_LIMIT`.
	pub limit: Option<u32>,
}

/// One item of a source that matched the search.
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
pub struct SourceSearchHit {
	/// The item's id in its source, as `sources.list_items` and `sources.get_item` use it.
	pub id: String,
	/// The item's id in the service it came from.
	pub external_id: String,
	pub title: String,
	pub subtitle: Option<String>,
	/// The start of the item's preview text, at most `SOURCE_HIT_PREVIEW_CHARS` characters.
	pub preview: Option<String>,
	/// Text around the match. Each matched term sits between U+0002 and U+0003, and `…` marks
	/// cut text.
	pub snippet: Option<String>,
	/// The item's date as the adapter stored it (`[search] date_field`), if any.
	pub date: Option<String>,
	pub source_id: String,
	pub source_name: String,
	pub adapter_id: String,
	pub data_type: String,
	/// The safety screening found something suspicious but kept the item searchable.
	pub flagged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
pub struct SearchSourcesOutput {
	/// Best match first, across every searched source.
	pub hits: Vec<SourceSearchHit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSourcesQuery {
	pub input: SearchSourcesInput,
}

impl SearchSourcesQuery {
	fn limit(&self) -> usize {
		self.input
			.limit
			.unwrap_or(DEFAULT_SOURCE_SEARCH_LIMIT)
			.clamp(1, MAX_SOURCE_SEARCH_LIMIT) as usize
	}
}

/// The first `max` characters of `text`, with `…` when it was longer.
fn truncate_chars(text: &str, max: usize) -> String {
	match text.char_indices().nth(max) {
		Some((cut, _)) => format!("{}…", text[..cut].trim_end()),
		None => text.to_string(),
	}
}

/// Turns archive search results into hits small enough to send for every keystroke.
pub fn hits_from_results(results: Vec<wing_archive::SearchResult>) -> Vec<SourceSearchHit> {
	results
		.into_iter()
		.map(|result| SourceSearchHit {
			id: result.id,
			external_id: result.external_id,
			title: result.title,
			subtitle: result.subtitle.filter(|s| !s.trim().is_empty()),
			preview: Some(result.preview)
				.filter(|p| !p.trim().is_empty())
				.map(|p| truncate_chars(&p, SOURCE_HIT_PREVIEW_CHARS)),
			snippet: result.snippet.filter(|s| !s.trim().is_empty()),
			date: result.date,
			source_id: result.source_id,
			source_name: result.source_name,
			adapter_id: result.adapter_id,
			data_type: result.data_type,
			flagged: result.safety_verdict.as_deref() == Some("flagged"),
		})
		.collect()
}

impl LibraryQuery for SearchSourcesQuery {
	type Input = SearchSourcesInput;
	type Output = SearchSourcesOutput;

	fn from_input(input: Self::Input) -> QueryResult<Self> {
		if input
			.source_id
			.as_deref()
			.is_some_and(|id| id.trim().is_empty())
		{
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
		if wing_archive::search::fts::match_expression(&self.input.query).is_none() {
			return Ok(SearchSourcesOutput { hits: Vec::new() });
		}
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
			// A library that never had a source has no archive; searching files must not create one.
			if !library.path().join("archive").join("registry.db").exists() {
				return Ok(SearchSourcesOutput { hits: Vec::new() });
			}
			library
				.init_source_manager()
				.await
				.map_err(|e| QueryError::Internal(format!("Failed to init source manager: {e}")))?;
		}
		let source_manager = library
			.source_manager()
			.ok_or_else(|| QueryError::Internal("Source manager not available".to_string()))?;

		let limit = self.limit();
		let results = source_manager
			.search(&self.input.query, self.input.source_id, limit)
			.await
			.map_err(QueryError::Internal)?;
		Ok(SearchSourcesOutput {
			hits: hits_from_results(results),
		})
	}
}

crate::register_library_query!(SearchSourcesQuery, "sources.search");

#[cfg(test)]
mod tests {
	use super::*;

	fn result(preview: &str, verdict: Option<&str>) -> wing_archive::SearchResult {
		wing_archive::SearchResult {
			id: "item".into(),
			external_id: "ext".into(),
			title: "Weekly review".into(),
			preview: preview.into(),
			subtitle: Some(" ".into()),
			snippet: Some("…the \u{2}harness\u{3} work…".into()),
			rank: -1.5,
			source_id: "source".into(),
			source_name: "Vault".into(),
			adapter_id: "obsidian".into(),
			data_type: "note".into(),
			data_type_icon: None,
			date: Some("2026-11-20T09:00:00+00:00".into()),
			trust_tier: wing_archive::TrustTier::Authored,
			safety_verdict: verdict.map(str::to_string),
			safety_score: None,
		}
	}

	#[test]
	fn hits_carry_a_short_preview_and_the_flag() {
		let long = "é".repeat(SOURCE_HIT_PREVIEW_CHARS + 10);
		let hits = hits_from_results(vec![
			result(&long, Some("flagged")),
			result("", Some("safe")),
		]);
		let preview = hits[0].preview.as_deref().unwrap();
		assert_eq!(preview.chars().count(), SOURCE_HIT_PREVIEW_CHARS + 1);
		assert!(preview.ends_with('…'));
		assert!(hits[0].flagged);
		assert_eq!(hits[0].subtitle, None, "a blank subtitle is dropped");
		assert_eq!(hits[0].adapter_id, "obsidian");
		assert_eq!(hits[0].external_id, "ext");
		assert_eq!(hits[1].preview, None);
		assert!(!hits[1].flagged);
		assert_eq!(truncate_chars("short", 10), "short");
	}

	#[test]
	fn limits_are_clamped_and_blank_source_ids_refused() {
		let query = |limit| {
			SearchSourcesQuery::from_input(SearchSourcesInput {
				query: "x".into(),
				source_id: None,
				limit,
			})
			.unwrap()
			.limit()
		};
		assert_eq!(query(None), DEFAULT_SOURCE_SEARCH_LIMIT as usize);
		assert_eq!(query(Some(0)), 1);
		assert_eq!(query(Some(10_000)), MAX_SOURCE_SEARCH_LIMIT as usize);
		assert!(SearchSourcesQuery::from_input(SearchSourcesInput {
			query: "x".into(),
			source_id: Some(" ".into()),
			limit: None,
		})
		.is_err());
	}
}
