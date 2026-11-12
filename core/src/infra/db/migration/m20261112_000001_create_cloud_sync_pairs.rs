//! Create `cloud_sync_pairs`, which pairs a local folder with a folder on a
//! cloud volume and stores the manifest of the last successful sync run.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(CloudSyncPairs::Table)
					.if_not_exists()
					.col(
						ColumnDef::new(CloudSyncPairs::Id)
							.integer()
							.not_null()
							.auto_increment()
							.primary_key(),
					)
					.col(
						ColumnDef::new(CloudSyncPairs::Uuid)
							.uuid()
							.not_null()
							.unique_key(),
					)
					.col(ColumnDef::new(CloudSyncPairs::Name).string().not_null())
					.col(
						ColumnDef::new(CloudSyncPairs::LocalPath)
							.string()
							.not_null(),
					)
					.col(
						ColumnDef::new(CloudSyncPairs::RemoteService)
							.string()
							.not_null(),
					)
					.col(
						ColumnDef::new(CloudSyncPairs::RemoteIdentifier)
							.string()
							.not_null(),
					)
					.col(
						ColumnDef::new(CloudSyncPairs::RemotePath)
							.string()
							.not_null(),
					)
					.col(ColumnDef::new(CloudSyncPairs::Mode).string().not_null())
					.col(
						ColumnDef::new(CloudSyncPairs::PropagateDeletes)
							.boolean()
							.not_null()
							.default(false),
					)
					.col(
						ColumnDef::new(CloudSyncPairs::IntervalMinutes)
							.integer()
							.null(),
					)
					.col(
						ColumnDef::new(CloudSyncPairs::Enabled)
							.boolean()
							.not_null()
							.default(true),
					)
					.col(
						ColumnDef::new(CloudSyncPairs::LastRunAt)
							.timestamp_with_time_zone()
							.null(),
					)
					.col(ColumnDef::new(CloudSyncPairs::LastStatus).string().null())
					.col(ColumnDef::new(CloudSyncPairs::LastError).text().null())
					.col(ColumnDef::new(CloudSyncPairs::LastReport).text().null())
					.col(ColumnDef::new(CloudSyncPairs::Manifest).text().null())
					.col(
						ColumnDef::new(CloudSyncPairs::CreatedAt)
							.timestamp_with_time_zone()
							.not_null()
							.default(Expr::current_timestamp()),
					)
					.col(
						ColumnDef::new(CloudSyncPairs::UpdatedAt)
							.timestamp_with_time_zone()
							.not_null()
							.default(Expr::current_timestamp()),
					)
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(CloudSyncPairs::Table).to_owned())
			.await
	}
}

#[derive(DeriveIden)]
enum CloudSyncPairs {
	Table,
	Id,
	Uuid,
	Name,
	LocalPath,
	RemoteService,
	RemoteIdentifier,
	RemotePath,
	Mode,
	PropagateDeletes,
	IntervalMinutes,
	Enabled,
	LastRunAt,
	LastStatus,
	LastError,
	LastReport,
	Manifest,
	CreatedAt,
	UpdatedAt,
}
