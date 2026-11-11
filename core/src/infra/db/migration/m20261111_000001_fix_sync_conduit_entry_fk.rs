//! Point the `sync_conduit` foreign keys at `entries`.
//!
//! `m20251015_000002_create_sync_tables` declared both entry foreign keys
//! against a table named `entry`, which does not exist. SQLite checks a foreign
//! key against its parent table on every insert, so creating a sync conduit
//! always failed with "no such table: main.entry". SQLite cannot change a
//! foreign key in place, so this renames both tables, creates them again with
//! the corrected keys, copies the rows whose entries still exist and drops the
//! old tables.

use sea_orm_migration::{
	prelude::*,
	sea_orm::{ConnectionTrait, TransactionTrait},
};

use super::m20251015_000002_create_sync_tables::{
	sync_conduit_enabled_index, sync_conduit_table, sync_generation_conduit_index,
	sync_generation_table,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		let db = manager.get_connection();
		// One transaction, so a failure or crash midway leaves the original
		// tables in place and the migration can run again
		let txn = db.begin().await?;
		txn.execute_unprepared(
			"ALTER TABLE sync_generation RENAME TO sync_generation_old;
			 ALTER TABLE sync_conduit RENAME TO sync_conduit_old;
			 DROP INDEX IF EXISTS idx_sync_conduit_enabled;
			 DROP INDEX IF EXISTS idx_sync_generation_conduit;",
		)
		.await?;

		txn.execute(&sync_conduit_table()).await?;
		txn.execute(&sync_generation_table()).await?;

		// Both tables are created by the same statements as before, so their
		// columns are in the same order. Rows that name a missing entry or
		// conduit could never satisfy the corrected keys and are left behind.
		txn.execute_unprepared(
			"INSERT INTO sync_conduit SELECT * FROM sync_conduit_old
			 WHERE source_entry_id IN (SELECT id FROM entries)
			   AND target_entry_id IN (SELECT id FROM entries);
			 INSERT INTO sync_generation SELECT * FROM sync_generation_old
			 WHERE conduit_id IN (SELECT id FROM sync_conduit);
			 DROP TABLE sync_generation_old;
			 DROP TABLE sync_conduit_old;",
		)
		.await?;

		txn.execute(&sync_conduit_enabled_index()).await?;
		txn.execute(&sync_generation_conduit_index()).await?;

		txn.commit().await
	}

	async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
		// The corrected keys are what the tables were always meant to have.
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use sea_orm::{ConnectionTrait, Database, DbBackend, Statement};

	async fn column(db: &sea_orm::DatabaseConnection, sql: &str) -> Vec<String> {
		db.query_all_raw(Statement::from_string(DbBackend::Sqlite, sql))
			.await
			.unwrap()
			.iter()
			.map(|row| row.try_get_by_index::<String>(0).unwrap())
			.collect()
	}

	#[tokio::test]
	async fn conduits_reference_entries_and_keep_their_rows() {
		let db = Database::connect("sqlite::memory:").await.unwrap();
		db.execute_unprepared(
			"CREATE TABLE entries (id INTEGER PRIMARY KEY);
			 INSERT INTO entries (id) VALUES (1), (2);",
		)
		.await
		.unwrap();
		let manager = SchemaManager::new(&db);
		// The tables as the first migration created them, keys pointing at `entry`.
		for table in [sync_conduit_table(), sync_generation_table()] {
			let sql = DbBackend::Sqlite
				.build(&table)
				.to_string()
				.replace("\"entries\"", "\"entry\"");
			db.execute_unprepared(&sql).await.unwrap();
		}
		manager
			.create_index(sync_conduit_enabled_index())
			.await
			.unwrap();
		manager
			.create_index(sync_generation_conduit_index())
			.await
			.unwrap();

		// Rows could only exist with foreign keys off; one names a missing entry.
		db.execute_unprepared(
			"PRAGMA foreign_keys = OFF;
			 INSERT INTO sync_conduit (id, uuid, source_entry_id, target_entry_id, sync_mode,
				created_at, updated_at)
			 VALUES (1, x'01', 1, 2, 'mirror', '2026-01-01', '2026-01-01'),
				(2, x'02', 1, 99, 'mirror', '2026-01-01', '2026-01-01');
			 INSERT INTO sync_generation (conduit_id, generation, started_at)
			 VALUES (1, 1, '2026-01-01'), (2, 1, '2026-01-01');
			 PRAGMA foreign_keys = ON;",
		)
		.await
		.unwrap();

		Migration.up(&manager).await.unwrap();

		let parents = column(
			&db,
			"SELECT DISTINCT \"table\" FROM pragma_foreign_key_list('sync_conduit')",
		)
		.await;
		assert_eq!(parents, ["entries"]);
		let parents = column(
			&db,
			"SELECT DISTINCT \"table\" FROM pragma_foreign_key_list('sync_generation')",
		)
		.await;
		assert_eq!(parents, ["sync_conduit"]);
		let ids = column(&db, "SELECT CAST(id AS TEXT) FROM sync_conduit").await;
		assert_eq!(ids, ["1"]);
		let generations = column(&db, "SELECT CAST(conduit_id AS TEXT) FROM sync_generation").await;
		assert_eq!(generations, ["1"]);
		let indexes = column(
			&db,
			"SELECT name FROM sqlite_master WHERE type = 'index' AND name LIKE 'idx_sync_%' ORDER BY name",
		)
		.await;
		assert_eq!(
			indexes,
			["idx_sync_conduit_enabled", "idx_sync_generation_conduit"]
		);
		let leftovers = column(
			&db,
			"SELECT name FROM sqlite_master WHERE name LIKE '%_old'",
		)
		.await;
		assert!(leftovers.is_empty(), "{leftovers:?}");

		// A conduit between existing entries can now be inserted.
		db.execute_unprepared(
			"INSERT INTO sync_conduit (uuid, source_entry_id, target_entry_id, sync_mode,
				created_at, updated_at)
			 VALUES (x'03', 2, 1, 'mirror', '2026-01-01', '2026-01-01')",
		)
		.await
		.unwrap();
	}

	#[tokio::test]
	async fn a_failed_rebuild_keeps_the_original_tables() {
		let db = Database::connect("sqlite::memory:").await.unwrap();
		// No `entries` table, so copying the conduits fails after the renames.
		for table in [sync_conduit_table(), sync_generation_table()] {
			let sql = DbBackend::Sqlite
				.build(&table)
				.to_string()
				.replace("\"entries\"", "\"entry\"");
			db.execute_unprepared(&sql).await.unwrap();
		}

		assert!(Migration.up(&SchemaManager::new(&db)).await.is_err());

		let tables = column(
			&db,
			"SELECT name FROM sqlite_master WHERE type = 'table' AND name LIKE 'sync_%' ORDER BY name",
		)
		.await;
		assert_eq!(tables, ["sync_conduit", "sync_generation"]);
	}
}
