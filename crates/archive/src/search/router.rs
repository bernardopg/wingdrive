//! Query router: fan-out search across sources.
//!
//! This is a simplified version that only uses FTS5 search.
//! Hybrid search with vector embeddings will be added when LanceDB is integrated.

use std::sync::Arc;

use crate::db::TemporalFilter;
use crate::embed::EmbeddingModel;
use crate::error::Result;
use crate::registry::Registry;
use crate::search::fts::match_expression;
use crate::search::{SearchFilter, SearchResult};
use crate::source::SourceManager;

const DEFAULT_LIMIT: usize = 20;

/// Routes search queries across all sources.
pub struct SearchRouter {
	pub(crate) registry: Arc<Registry>,
	pub(crate) sources: Arc<SourceManager>,
	pub(crate) _embedding: Arc<EmbeddingModel>,
}

impl SearchRouter {
	pub fn new(
		registry: Arc<Registry>,
		sources: Arc<SourceManager>,
		embedding: Arc<EmbeddingModel>,
	) -> Self {
		Self {
			registry,
			sources,
			_embedding: embedding,
		}
	}

	/// Search across all (or filtered) sources using FTS5.
	///
	/// `query` is the text a person typed, not FTS5 syntax: every word must match, as a prefix
	/// (see [`match_expression`]). Only records the safety screening let into the index match.
	pub async fn search(
		&self,
		query: &str,
		filter: Option<SearchFilter>,
	) -> Result<Vec<SearchResult>> {
		let filter = filter.unwrap_or_default();
		let limit = filter.limit.unwrap_or(DEFAULT_LIMIT);
		let Some(expression) = match_expression(query) else {
			return Ok(Vec::new());
		};

		let all_sources = self.registry.list_sources().await?;
		let sources_to_search: Vec<_> = all_sources
			.into_iter()
			.filter(|s| {
				if let Some(ref source_id) = filter.source_id {
					return &s.id == source_id;
				}
				if let Some(ref dt) = filter.data_type {
					return &s.data_type == dt;
				}
				true
			})
			.collect();

		if sources_to_search.is_empty() {
			return Ok(Vec::new());
		}

		let mut all_results = Vec::new();

		for source_info in &sources_to_search {
			let db = match self.sources.open(&source_info.id).await {
				Ok(db) => db,
				Err(e) => {
					tracing::warn!(source_id = %source_info.id, error = %e, "failed to open source for search");
					continue;
				}
			};

			let temporal = if filter.date_after.is_some() || filter.date_before.is_some() {
				Some(TemporalFilter {
					date_after: filter.date_after.as_deref(),
					date_before: filter.date_before.as_deref(),
				})
			} else {
				None
			};

			let fts_hits = match db.fts_search(&expression, limit, temporal).await {
				Ok(hits) => hits,
				Err(e) => {
					tracing::warn!(source_id = %source_info.id, error = %e, "FTS search failed");
					Vec::new()
				}
			};

			for hit in fts_hits {
				all_results.push(SearchResult {
					id: hit.id,
					external_id: hit.external_id,
					title: hit.title,
					preview: hit.preview.unwrap_or_default(),
					subtitle: hit.subtitle,
					snippet: hit.snippet,
					rank: hit.rank,
					source_id: source_info.id.clone(),
					source_name: source_info.name.clone(),
					adapter_id: source_info.adapter_id.clone(),
					data_type: source_info.data_type.clone(),
					data_type_icon: None,
					date: hit.date,
					trust_tier: source_info.trust_tier,
					safety_verdict: hit.safety_verdict,
					safety_score: hit.safety_score,
				});
			}
		}

		if filter.sort_by_date {
			all_results.sort_by(|a, b| {
				let da = a.date.as_deref().unwrap_or("");
				let db = b.date.as_deref().unwrap_or("");
				db.cmp(da)
			});
		} else {
			// bm25 ranks are negative and lower is better.
			all_results.sort_by(|a, b| a.rank.total_cmp(&b.rank));
		}
		all_results.truncate(limit);

		Ok(all_results)
	}
}
