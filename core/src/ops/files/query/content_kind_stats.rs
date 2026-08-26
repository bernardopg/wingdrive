//! Query to get content kind statistics
//!
//! This query returns file counts grouped by content kind (image, video, audio, etc.).
//! Counts come from indexed file entries so orphaned content identities cannot appear as files.

use crate::infra::query::{QueryError, QueryResult};
use crate::{
	context::CoreContext, domain::ContentKind, infra::db::entities::content_kind,
	infra::query::LibraryQuery,
};
use sea_orm::{ConnectionTrait, DatabaseConnection, EntityTrait, Order, QueryOrder, Statement};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::Arc;
use uuid::Uuid;

/// Input for content kind statistics query
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ContentKindStatsInput {}

/// A single content kind with its file count
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ContentKindStat {
	/// The content kind (image, video, audio, etc.)
	pub kind: ContentKind,
	/// The name of the content kind
	pub name: String,
	/// The number of files with this content kind
	pub file_count: i64,
}

/// Output containing content kind statistics
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ContentKindStatsOutput {
	/// Statistics for each content kind
	pub stats: Vec<ContentKindStat>,
	/// Total number of files across all content kinds
	pub total_files: i64,
}

/// Query to get content kind statistics
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ContentKindStatsQuery {
	pub input: ContentKindStatsInput,
}

async fn load_file_counts(
	db: &DatabaseConnection,
) -> Result<std::collections::HashMap<i32, i64>, sea_orm::DbErr> {
	let rows = db
		.query_all(Statement::from_string(
			sea_orm::DatabaseBackend::Sqlite,
			r#"
				SELECT ci.kind_id, COUNT(*) AS file_count
				FROM entries e
				INNER JOIN content_identities ci ON e.content_id = ci.id
				WHERE e.kind = 0
				GROUP BY ci.kind_id
			"#
			.to_owned(),
		))
		.await?;

	rows.into_iter()
		.map(|row| {
			Ok((
				row.try_get::<i32>("", "kind_id")?,
				row.try_get::<i64>("", "file_count")?,
			))
		})
		.collect()
}

impl ContentKindStatsQuery {
	pub fn new() -> Self {
		Self {
			input: ContentKindStatsInput {},
		}
	}
}

impl LibraryQuery for ContentKindStatsQuery {
	type Input = ContentKindStatsInput;
	type Output = ContentKindStatsOutput;

	fn from_input(input: Self::Input) -> QueryResult<Self> {
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

		let db = library.db();
		let counts = load_file_counts(db.conn()).await?;

		// Preserve the stable list and order, including zero-count kinds.
		let content_kinds = content_kind::Entity::find()
			.order_by(content_kind::Column::Id, Order::Asc)
			.all(db.conn())
			.await?;

		let mut stats = Vec::new();
		let mut total_files = 0i64;

		for ck in content_kinds {
			let kind = ContentKind::from_id(ck.id);
			let file_count = counts.get(&ck.id).copied().unwrap_or(0);
			total_files += file_count;

			stats.push(ContentKindStat {
				kind,
				name: ck.name,
				file_count,
			});
		}

		Ok(ContentKindStatsOutput { stats, total_files })
	}
}

// Register the query
crate::register_library_query!(ContentKindStatsQuery, "files.content_kind_stats");

#[cfg(test)]
mod tests {
	use super::*;
	use sea_orm::{ConnectionTrait, Database};

	#[tokio::test]
	async fn stats_ignore_orphaned_identities_and_directories() {
		let db = Database::connect("sqlite::memory:").await.unwrap();
		db.execute_unprepared(
			r#"
			CREATE TABLE content_identities (id INTEGER PRIMARY KEY, kind_id INTEGER NOT NULL);
			CREATE TABLE entries (id INTEGER PRIMARY KEY, kind INTEGER NOT NULL, content_id INTEGER);
			INSERT INTO content_identities VALUES (1, 1), (2, 1), (3, 2);
			INSERT INTO entries VALUES (1, 0, 1), (2, 1, 2);
			"#,
		)
		.await
		.unwrap();

		let counts = load_file_counts(&db).await.unwrap();

		assert_eq!(counts.get(&(ContentKind::Image as i32)), Some(&1));
		assert!(!counts.contains_key(&(ContentKind::Video as i32)));
	}
}
