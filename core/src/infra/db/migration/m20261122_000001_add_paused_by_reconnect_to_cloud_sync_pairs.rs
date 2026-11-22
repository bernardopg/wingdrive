//! Record whether a sync pair was paused by an account reconnect so it can be
//! resumed after the user signs in again.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(
				Table::alter()
					.table(CloudSyncPairs::Table)
					.add_column(
						ColumnDef::new(CloudSyncPairs::PausedByReconnect)
							.boolean()
							.not_null()
							.default(false),
					)
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.alter_table(
				Table::alter()
					.table(CloudSyncPairs::Table)
					.drop_column(CloudSyncPairs::PausedByReconnect)
					.to_owned(),
			)
			.await
	}
}

#[derive(Iden)]
enum CloudSyncPairs {
	Table,
	PausedByReconnect,
}
