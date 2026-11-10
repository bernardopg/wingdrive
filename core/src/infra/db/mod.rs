//! Database infrastructure using SeaORM

use sea_orm::{
	ConnectOptions, Database as SeaDatabase, DatabaseConnection, DatabaseTransaction, DbErr,
	SqliteTransactionMode, TransactionOptions, TransactionTrait,
};
use sea_orm_migration::MigratorTrait;
use sqlx::sqlite::SqliteConnectOptions;
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;
use tracing::info;

pub mod entities;
pub mod migration;

/// Begins a transaction that takes SQLite's write lock up front.
///
/// A deferred transaction that reads first fails its first write with
/// SQLITE_BUSY_SNAPSHOT when another connection wrote meanwhile, and
/// `busy_timeout` does not wait on that, so the whole write was lost.
/// Waiting for the lock here is covered by `busy_timeout`, retried a few times
/// while a long write from another task holds it.
pub async fn begin_write(db: &DatabaseConnection) -> Result<DatabaseTransaction, DbErr> {
	let options = TransactionOptions {
		sqlite_transaction_mode: Some(SqliteTransactionMode::Immediate),
		..Default::default()
	};
	let mut attempt = 0;
	loop {
		match db.begin_with_options(options).await {
			Err(e) if attempt < 4 && e.to_string().contains("database is locked") => attempt += 1,
			result => return result,
		}
	}
}

/// Database wrapper for WingDrive
pub struct Database {
	/// SeaORM database connection
	conn: DatabaseConnection,
}

impl AsRef<DatabaseConnection> for Database {
	fn as_ref(&self) -> &DatabaseConnection {
		&self.conn
	}
}

/// Build `SqliteConnectOptions` with PRAGMAs applied to every pooled connection.
fn sqlite_connect_options(url: &str) -> Result<SqliteConnectOptions, DbErr> {
	let opts = SqliteConnectOptions::from_str(url)
		.map_err(|e| DbErr::Custom(format!("Invalid SQLite URL: {}", e)))?
		.busy_timeout(Duration::from_millis(5000))
		.journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
		.synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
		.pragma("temp_store", "MEMORY")
		.pragma("cache_size", "-20000")
		.pragma("mmap_size", "67108864");
	Ok(opts)
}

/// Build a SeaORM `DatabaseConnection` from sqlx `SqliteConnectOptions`,
/// ensuring all PRAGMAs are applied to every connection in the pool.
///
/// Every SQLite file WingDrive opens goes through here. SQLite's defaults
/// (rollback journal, `synchronous=FULL`) flush and recreate the journal on
/// every commit, which costs tens of milliseconds per commit on Windows and
/// made the sync log the bottleneck of each synced record.
pub(crate) async fn connect_sqlite(url: &str, pool_size: u32) -> Result<DatabaseConnection, DbErr> {
	let opts = sqlite_connect_options(url)?;

	let pool_size = pool_size.max(1);
	let pool = sqlx::pool::PoolOptions::<sqlx::Sqlite>::new()
		.max_connections(pool_size)
		.min_connections(pool_size.min(5))
		.acquire_timeout(Duration::from_secs(30))
		.idle_timeout(Duration::from_secs(30))
		.max_lifetime(Duration::from_secs(30))
		.connect_with(opts)
		.await
		.map_err(|e| DbErr::Custom(format!("Failed to connect: {}", e)))?;

	Ok(sea_orm::SqlxSqliteConnector::from_sqlx_sqlite_pool(pool))
}

impl Database {
	/// Create a new database at the specified path
	pub async fn create(path: &Path) -> Result<Self, DbErr> {
		let db_url = if path.as_os_str() == ":memory:" {
			"sqlite::memory:".to_string()
		} else {
			// Ensure parent directory exists
			if let Some(parent) = path.parent() {
				std::fs::create_dir_all(parent)
					.map_err(|e| DbErr::Custom(format!("Failed to create directory: {}", e)))?;
			}
			format!("sqlite://{}?mode=rwc", path.display())
		};

		let pool_size = std::env::var("WINGDRIVE_DB_POOL_SIZE")
			.or_else(|_| std::env::var("WINGDRIVE_DB_POOL_SIZE"))
			.ok()
			.and_then(|s| s.parse().ok())
			.unwrap_or(30);

		let conn = connect_sqlite(&db_url, pool_size).await?;

		info!("Created new database at {:?}", path);

		Ok(Self { conn })
	}

	/// Open an existing database
	pub async fn open(path: &Path) -> Result<Self, DbErr> {
		if !path.exists() {
			return Err(DbErr::Custom(format!(
				"Database does not exist: {}",
				path.display()
			)));
		}

		let db_url = format!("sqlite://{}", path.display());

		let pool_size = std::env::var("WINGDRIVE_DB_POOL_SIZE")
			.or_else(|_| std::env::var("WINGDRIVE_DB_POOL_SIZE"))
			.ok()
			.and_then(|s| s.parse().ok())
			.unwrap_or(30);

		let conn = connect_sqlite(&db_url, pool_size).await?;

		info!("Opened database at {:?}", path);

		Ok(Self { conn })
	}

	/// Run migrations on one pinned connection.
	///
	/// sea-orm-migration only wraps the run in a transaction for Postgres. On
	/// SQLite each statement would otherwise go to the pool, and the pool hands
	/// statements round-robin to connections that each cache their own copy of
	/// the schema. Most statements recover from a stale cache (SQLite re-prepares
	/// on SQLITE_SCHEMA), but ALTER TABLE DROP COLUMN resolves the column at
	/// prepare time against the cache and fails with "no such column" for good.
	/// A transaction holds a single connection for the whole run, so every
	/// statement sees the one it came after. Everything lands or nothing does.
	pub async fn migrate(&self) -> Result<(), DbErr> {
		let txn = self.conn.begin().await?;
		migration::Migrator::up(&txn, None).await?;
		txn.commit().await?;
		info!("Database migrations completed successfully");
		Ok(())
	}

	/// Get the database connection
	pub fn conn(&self) -> &DatabaseConnection {
		&self.conn
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn sqlite_includes_wal_page_size_fix() {
		use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};

		let db = connect_sqlite("sqlite::memory:", 1).await.unwrap();
		let row = db
			.query_one_raw(Statement::from_string(
				DatabaseBackend::Sqlite,
				"SELECT sqlite_version() AS version".to_string(),
			))
			.await
			.unwrap()
			.unwrap();
		let version: String = row.try_get("", "version").unwrap();
		let parts: Vec<u32> = version
			.split('.')
			.map(|part| part.parse().unwrap())
			.collect();
		assert!(
			parts.as_slice() >= [3, 54, 0].as_slice(),
			"SQLite {version} is vulnerable to CVE-2026-119375"
		);
	}

	/// Every connection of a 30-connection pool opens before the first
	/// statement. The migrator rotates through them, and the connection that
	/// loaded the schema just before `m20251226` adds `entries.device_id` is
	/// the one handed `m20260104`'s `DROP COLUMN device_id` fourteen statements
	/// later, with a cache that never had the column. Running the migrations on
	/// the pool failed this every time; pinned to one transaction it passes.
	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn migrate_fresh_database_on_eagerly_filled_pool() {
		let dir = tempfile::tempdir().unwrap();
		let url = format!(
			"sqlite://{}?mode=rwc",
			dir.path().join("library.db").display()
		);
		let pool = sqlx::pool::PoolOptions::<sqlx::Sqlite>::new()
			.max_connections(30)
			.min_connections(30)
			.connect_with(sqlite_connect_options(&url).unwrap())
			.await
			.unwrap();
		let db = Database {
			conn: sea_orm::SqlxSqliteConnector::from_sqlx_sqlite_pool(pool),
		};

		db.migrate()
			.await
			.expect("migrations on a stale pooled connection");
	}

	async fn scalar(db: &impl sea_orm::ConnectionTrait, sql: &str) -> i64 {
		let row = db
			.query_one_raw(sea_orm::Statement::from_string(
				sea_orm::DatabaseBackend::Sqlite,
				sql.to_string(),
			))
			.await
			.unwrap()
			.unwrap();
		row.try_get_by_index(0).unwrap()
	}

	#[tokio::test]
	async fn migrated_database_reopens_without_losing_rows_or_reapplying_migrations() {
		use sea_orm::ConnectionTrait;

		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("nested/library.db");
		let db = Database::create(&path).await.unwrap();
		db.migrate().await.unwrap();
		let applied = scalar(db.conn(), "SELECT COUNT(*) FROM seaql_migrations").await;
		assert!(applied > 0);
		db.conn().execute_unprepared("CREATE TABLE coverage_probe (value INTEGER NOT NULL); INSERT INTO coverage_probe VALUES (42)").await.unwrap();
		db.conn.close().await.unwrap();
		let reopened = Database::open(&path).await.unwrap();
		reopened.migrate().await.unwrap();
		assert_eq!(
			scalar(reopened.conn(), "SELECT COUNT(*) FROM seaql_migrations").await,
			applied
		);
		assert_eq!(
			scalar(reopened.conn(), "SELECT value FROM coverage_probe").await,
			42
		);
		assert_eq!(scalar(reopened.conn(), "PRAGMA synchronous").await, 1);
		reopened.conn.close().await.unwrap();
	}

	#[tokio::test]
	async fn rollback_discards_schema_and_rows_while_commit_survives_reopening() {
		use sea_orm::ConnectionTrait;

		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("transactions.db");
		let db = Database::create(&path).await.unwrap();
		db.conn()
			.execute_unprepared(
				"CREATE TABLE counter (value INTEGER NOT NULL); INSERT INTO counter VALUES (1)",
			)
			.await
			.unwrap();
		let transaction = begin_write(db.conn()).await.unwrap();
		transaction
			.execute_unprepared(
				"UPDATE counter SET value = 9; CREATE TABLE rolled_back (id INTEGER)",
			)
			.await
			.unwrap();
		transaction.rollback().await.unwrap();
		assert_eq!(scalar(db.conn(), "SELECT value FROM counter").await, 1);
		assert_eq!(
			scalar(
				db.conn(),
				"SELECT COUNT(*) FROM sqlite_master WHERE name = 'rolled_back'"
			)
			.await,
			0
		);
		let transaction = begin_write(db.conn()).await.unwrap();
		transaction
			.execute_unprepared("UPDATE counter SET value = 2")
			.await
			.unwrap();
		transaction.commit().await.unwrap();
		db.conn.close().await.unwrap();
		let reopened = Database::open(&path).await.unwrap();
		assert_eq!(
			scalar(reopened.conn(), "SELECT value FROM counter").await,
			2
		);
		reopened.conn.close().await.unwrap();
	}

	#[tokio::test]
	async fn failed_migration_rolls_back_earlier_schema_changes_and_tracking_rows() {
		use sea_orm::ConnectionTrait;

		let dir = tempfile::tempdir().unwrap();
		let db = Database::create(&dir.path().join("failed.db"))
			.await
			.unwrap();
		db.conn()
			.execute_unprepared("CREATE TABLE entries (incompatible INTEGER)")
			.await
			.unwrap();
		assert!(db.migrate().await.is_err());
		assert_eq!(
			scalar(
				db.conn(),
				"SELECT COUNT(*) FROM sqlite_master WHERE name = 'devices'"
			)
			.await,
			0
		);
		assert_eq!(
			scalar(
				db.conn(),
				"SELECT COUNT(*) FROM sqlite_master WHERE name = 'seaql_migrations'"
			)
			.await,
			0
		);
		db.conn()
			.execute_unprepared("DROP TABLE entries")
			.await
			.unwrap();
		db.migrate().await.unwrap();
		assert!(scalar(db.conn(), "SELECT COUNT(*) FROM seaql_migrations").await > 0);
		db.conn.close().await.unwrap();
	}

	#[tokio::test]
	async fn queued_writer_observes_the_committed_value_before_its_read() {
		use sea_orm::ConnectionTrait;

		let dir = tempfile::tempdir().unwrap();
		let url = format!(
			"sqlite://{}?mode=rwc",
			dir.path().join("writers.db").display()
		);
		let db = connect_sqlite(&url, 2).await.unwrap();
		db.execute_unprepared(
			"CREATE TABLE counter (value INTEGER NOT NULL); INSERT INTO counter VALUES (0)",
		)
		.await
		.unwrap();
		let first = begin_write(&db).await.unwrap();
		first
			.execute_unprepared("UPDATE counter SET value = 1")
			.await
			.unwrap();
		let other_db = db.clone();
		let (started, ready) = tokio::sync::oneshot::channel();
		let writer = tokio::spawn(async move {
			started.send(()).unwrap();
			let transaction = begin_write(&other_db).await.unwrap();
			let observed = scalar(&transaction, "SELECT value FROM counter").await;
			transaction
				.execute_unprepared("UPDATE counter SET value = value + 1")
				.await
				.unwrap();
			transaction.commit().await.unwrap();
			observed
		});
		ready.await.unwrap();
		first.commit().await.unwrap();
		assert_eq!(
			tokio::time::timeout(Duration::from_secs(10), writer)
				.await
				.unwrap()
				.unwrap(),
			1
		);
		assert_eq!(scalar(&db, "SELECT value FROM counter").await, 2);
		db.close().await.unwrap();
	}

	#[tokio::test]
	async fn opening_a_missing_database_does_not_create_it() {
		let dir = tempfile::tempdir().unwrap();
		let missing = dir.path().join("missing.db");
		assert!(Database::open(&missing).await.is_err());
		assert!(!missing.exists());
		assert!(connect_sqlite("sqlite://?mode=invalid", 1).await.is_err());
	}
}
