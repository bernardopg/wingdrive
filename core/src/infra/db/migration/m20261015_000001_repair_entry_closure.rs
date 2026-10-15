//! Restore `entry_closure` rows lost by batched indexing.
//!
//! The indexer built each entry's closure by copying its parent's rows, but the
//! parent's own depth-0 row was only written at the end of the batch. Entries
//! created in the same batch as their parent therefore lacked every row naming
//! that parent (and its in-batch ancestors) as an ancestor, so descendant
//! queries through the closure missed them. This rebuilds the missing rows from
//! `parent_id`; existing rows are kept.

use sea_orm_migration::prelude::*;

/// Inserts every (ancestor, descendant) pair implied by `parent_id` chains.
///
/// The depth bound stops a corrupt `parent_id` cycle from recursing forever.
pub(crate) const REPAIR_ENTRY_CLOSURE_SQL: &str = "
	WITH RECURSIVE chain(ancestor_id, descendant_id, depth) AS (
		SELECT id, id, 0 FROM entries
		UNION ALL
		SELECT e.parent_id, chain.descendant_id, chain.depth + 1
		FROM chain
		JOIN entries e ON e.id = chain.ancestor_id
		WHERE e.parent_id IS NOT NULL AND chain.depth < 4096
	)
	INSERT OR IGNORE INTO entry_closure (ancestor_id, descendant_id, depth)
	SELECT ancestor_id, descendant_id, depth FROM chain
";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.get_connection()
			.execute_unprepared(REPAIR_ENTRY_CLOSURE_SQL)
			.await?;

		Ok(())
	}

	async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
		// The restored rows are correct data; there is nothing to undo.
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use sea_orm::{ConnectionTrait, Database, DbBackend, Statement};

	#[tokio::test]
	async fn restores_rows_missing_for_entries_batched_with_their_parent() {
		let db = Database::connect("sqlite::memory:").await.unwrap();
		db.execute_unprepared(
			"CREATE TABLE entries (id INTEGER PRIMARY KEY, parent_id INTEGER);
			 CREATE TABLE entry_closure (
				ancestor_id INTEGER NOT NULL,
				descendant_id INTEGER NOT NULL,
				depth INTEGER NOT NULL,
				PRIMARY KEY (ancestor_id, descendant_id)
			 );
			 -- root(1) / a(2) / b(3) / file(4); a, b and file were batched together.
			 INSERT INTO entries VALUES (1, NULL), (2, 1), (3, 2), (4, 3);
			 INSERT INTO entry_closure VALUES
				(1, 1, 0), (2, 2, 0), (3, 3, 0), (4, 4, 0),
				(1, 2, 1), (1, 3, 2), (1, 4, 3);",
		)
		.await
		.unwrap();

		db.execute_unprepared(REPAIR_ENTRY_CLOSURE_SQL)
			.await
			.unwrap();

		let rows = db
			.query_all_raw(Statement::from_string(
				DbBackend::Sqlite,
				"SELECT ancestor_id, descendant_id, depth FROM entry_closure \
				 ORDER BY descendant_id, depth",
			))
			.await
			.unwrap()
			.into_iter()
			.map(|row| {
				(
					row.try_get::<i32>("", "ancestor_id").unwrap(),
					row.try_get::<i32>("", "descendant_id").unwrap(),
					row.try_get::<i32>("", "depth").unwrap(),
				)
			})
			.collect::<Vec<_>>();

		assert_eq!(
			rows,
			vec![
				(1, 1, 0),
				(2, 2, 0),
				(1, 2, 1),
				(3, 3, 0),
				(2, 3, 1),
				(1, 3, 2),
				(4, 4, 0),
				(3, 4, 1),
				(2, 4, 2),
				(1, 4, 3),
			]
		);
	}
}
