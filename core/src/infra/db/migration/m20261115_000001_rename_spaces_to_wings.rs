//! Rename the sidebar tables from their Space names to Wings (BRAND-003).
//!
//! `spaces`, `space_groups` and `space_items` become `wings`, `wing_groups` and
//! `wing_items`, and the `space_id` column of the last two becomes `wing_id`. SQLite
//! renames keep the rows, the indexes and the foreign keys that point at the
//! renamed tables. Sync still sends the old model and field names, so nothing
//! changes for other devices.

use sea_orm_migration::{
	prelude::*,
	sea_orm::{ConnectionTrait, TransactionTrait},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

const UP: &str = "ALTER TABLE spaces RENAME TO wings;
	ALTER TABLE space_groups RENAME TO wing_groups;
	ALTER TABLE space_items RENAME TO wing_items;
	ALTER TABLE wing_groups RENAME COLUMN space_id TO wing_id;
	ALTER TABLE wing_items RENAME COLUMN space_id TO wing_id;";

const DOWN: &str = "ALTER TABLE wing_items RENAME COLUMN wing_id TO space_id;
	ALTER TABLE wing_groups RENAME COLUMN wing_id TO space_id;
	ALTER TABLE wing_items RENAME TO space_items;
	ALTER TABLE wing_groups RENAME TO space_groups;
	ALTER TABLE wings RENAME TO spaces;";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		// One transaction, so a failure midway leaves every table under its old name
		let txn = manager.get_connection().begin().await?;
		txn.execute_unprepared(UP).await?;
		txn.commit().await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		let txn = manager.get_connection().begin().await?;
		txn.execute_unprepared(DOWN).await?;
		txn.commit().await
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::infra::db::migration::Migrator;
	use sea_orm::{Database, DatabaseConnection, DbBackend, Statement};

	async fn strings(db: &DatabaseConnection, sql: &str) -> Vec<String> {
		db.query_all_raw(Statement::from_string(DbBackend::Sqlite, sql))
			.await
			.unwrap()
			.iter()
			.map(|row| row.try_get_by_index::<String>(0).unwrap())
			.collect()
	}

	async fn tables(db: &DatabaseConnection) -> Vec<String> {
		strings(
			db,
			"SELECT name FROM sqlite_master WHERE type = 'table'
			 AND (name LIKE 'space%' OR name LIKE 'wing%') ORDER BY name",
		)
		.await
	}

	#[tokio::test]
	async fn the_sidebar_keeps_its_rows_and_links_under_the_wing_names() {
		let db = Database::connect("sqlite::memory:").await.unwrap();
		Migrator::up(&db, None).await.unwrap();
		// Back to the schema of 3.3.1, then a sidebar as that version saved it.
		Migrator::down(&db, Some(2)).await.unwrap();
		assert_eq!(tables(&db).await, ["space_groups", "space_items", "spaces"]);
		db.execute_unprepared(
			"INSERT INTO spaces (id, uuid, name, icon, color, \"order\", created_at, updated_at)
			 VALUES (7, x'01', 'Work', 'Briefcase', '#3B82F6', 0, '2026-01-01', '2026-01-01');
			 INSERT INTO space_groups (id, uuid, space_id, name, group_type, is_collapsed,
				\"order\", created_at)
			 VALUES (3, x'02', 7, 'Projects', '\"Custom\"', 0, 0, '2026-01-01');
			 INSERT INTO space_items (id, uuid, space_id, group_id, item_type, \"order\", created_at)
			 VALUES (5, x'03', 7, 3, '\"Recents\"', 0, '2026-01-01'),
				(6, x'04', 7, NULL, '\"Overview\"', 1, '2026-01-01');",
		)
		.await
		.unwrap();

		Migrator::up(&db, None).await.unwrap();

		assert_eq!(tables(&db).await, ["wing_groups", "wing_items", "wings"]);
		assert_eq!(strings(&db, "SELECT name FROM wings").await, ["Work"]);
		let items = strings(
			&db,
			"SELECT i.item_type || ' ' || w.name || ' ' || IFNULL(g.name, '-')
			 FROM wing_items i JOIN wings w ON w.id = i.wing_id
			 LEFT JOIN wing_groups g ON g.id = i.group_id ORDER BY i.id",
		)
		.await;
		assert_eq!(items, ["\"Recents\" Work Projects", "\"Overview\" Work -"]);
		for table in ["wing_groups", "wing_items"] {
			let parents = strings(
				&db,
				&format!(
					"SELECT DISTINCT \"table\" FROM pragma_foreign_key_list('{table}') ORDER BY 1"
				),
			)
			.await;
			assert!(
				parents.contains(&"wings".to_string()),
				"{table}: {parents:?}"
			);
			assert!(
				!parents.iter().any(|p| p.starts_with("space")),
				"{table}: {parents:?}"
			);
		}
		let check = strings(&db, "SELECT \"table\" FROM pragma_foreign_key_check").await;
		assert!(check.is_empty(), "{check:?}");
	}

	#[tokio::test]
	async fn a_failed_rename_keeps_the_old_tables() {
		let db = Database::connect("sqlite::memory:").await.unwrap();
		Migrator::up(&db, None).await.unwrap();
		Migrator::down(&db, Some(2)).await.unwrap();
		// A table already under the new name makes the second rename fail.
		db.execute_unprepared("CREATE TABLE wing_groups (id INTEGER PRIMARY KEY)")
			.await
			.unwrap();

		assert!(Migration.up(&SchemaManager::new(&db)).await.is_err());

		assert_eq!(
			tables(&db).await,
			["space_groups", "space_items", "spaces", "wing_groups"]
		);
	}
}
