//! Add index on directory_paths(path) to back prefix scans during ephemeral
//! UUID reconciliation.
//!
//! `extract_persistent_uuids_for_path` resolves entries by an indexed
//! prefix range (`path >= lower AND path < upper`). The index avoids a full
//! scan of unrelated directories in the library.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.get_connection()
			.execute_unprepared(
				"CREATE INDEX IF NOT EXISTS idx_directory_paths_path \
				 ON directory_paths(path)",
			)
			.await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.get_connection()
			.execute_unprepared("DROP INDEX IF EXISTS idx_directory_paths_path")
			.await?;

		Ok(())
	}
}
