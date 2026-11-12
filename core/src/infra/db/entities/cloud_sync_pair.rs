//! Cloud sync pair entity: a local folder kept in step with a cloud folder.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "cloud_sync_pairs")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub id: i32,
	#[sea_orm(unique)]
	pub uuid: Uuid,
	pub name: String,
	pub local_path: String,
	/// URI scheme of the cloud service, such as "gdrive".
	pub remote_service: String,
	pub remote_identifier: String,
	pub remote_path: String,
	/// "two_way", "upload" or "download".
	pub mode: String,
	pub propagate_deletes: bool,
	pub interval_minutes: Option<i32>,
	pub enabled: bool,
	pub last_run_at: Option<DateTimeUtc>,
	/// "success", "partial" or "failed".
	pub last_status: Option<String>,
	pub last_error: Option<String>,
	/// JSON `SyncReport` of the last run.
	pub last_report: Option<String>,
	/// JSON `Manifest` of the last run.
	pub manifest: Option<String>,
	pub created_at: DateTimeUtc,
	pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
