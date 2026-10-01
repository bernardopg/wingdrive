//! Make directory entries unique per parent.
//!
//! `idx_entries_unique_file` only covers files, so two watcher Create events
//! for the same new folder could both insert it and leave a duplicate row.
//! This migration merges existing duplicates into the oldest row and adds the
//! matching unique index for directories.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		let db = manager.get_connection();

		// Map every duplicate directory to the oldest row with the same parent
		// and name. Rows that point at a duplicate are repointed to the keeper,
		// then the duplicates are deleted (closure and path rows cascade).
		db.execute_unprepared(
			r#"
			CREATE TEMP TABLE dup_dirs AS
			SELECT e.id AS dup_id, k.keep_id
			FROM entries e
			JOIN (
				SELECT parent_id, name, MIN(id) AS keep_id
				FROM entries
				WHERE kind = 1 AND parent_id IS NOT NULL
				GROUP BY parent_id, name
				HAVING COUNT(*) > 1
			) k ON e.parent_id = k.parent_id AND e.name = k.name
			WHERE e.kind = 1 AND e.id <> k.keep_id;

			UPDATE entries SET parent_id = (SELECT keep_id FROM dup_dirs WHERE dup_id = entries.parent_id)
			WHERE parent_id IN (SELECT dup_id FROM dup_dirs);

			UPDATE OR IGNORE collection_entry SET entry_id = (SELECT keep_id FROM dup_dirs WHERE dup_id = collection_entry.entry_id)
			WHERE entry_id IN (SELECT dup_id FROM dup_dirs);

			UPDATE sidecar SET source_entry_id = (SELECT keep_id FROM dup_dirs WHERE dup_id = sidecar.source_entry_id)
			WHERE source_entry_id IN (SELECT dup_id FROM dup_dirs);

			UPDATE locations SET entry_id = (SELECT keep_id FROM dup_dirs WHERE dup_id = locations.entry_id)
			WHERE entry_id IN (SELECT dup_id FROM dup_dirs);

			-- Move a duplicate's metadata (favorite, notes, tags) to the keeper
			-- unless the keeper already has its own; that copy is left as is.
			UPDATE user_metadata SET entry_uuid = (
				SELECT k.uuid FROM dup_dirs d
				JOIN entries x ON x.id = d.dup_id
				JOIN entries k ON k.id = d.keep_id
				WHERE x.uuid = user_metadata.entry_uuid
			)
			WHERE entry_uuid IN (
				SELECT x.uuid FROM dup_dirs d JOIN entries x ON x.id = d.dup_id
				JOIN entries k ON k.id = d.keep_id
				WHERE NOT EXISTS (SELECT 1 FROM user_metadata m WHERE m.entry_uuid = k.uuid)
			);

			DELETE FROM entries WHERE id IN (SELECT dup_id FROM dup_dirs);

			DROP TABLE dup_dirs;

			CREATE UNIQUE INDEX IF NOT EXISTS idx_entries_unique_dir
			ON entries(parent_id, name)
			WHERE kind = 1 AND parent_id IS NOT NULL;
			"#,
		)
		.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.get_connection()
			.execute_unprepared("DROP INDEX IF EXISTS idx_entries_unique_dir")
			.await?;
		Ok(())
	}
}
