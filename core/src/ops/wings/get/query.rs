use super::output::WingGetOutput;
use crate::domain::Wing;
use crate::infra::query::{QueryError, QueryResult};
use crate::{context::CoreContext, infra::query::LibraryQuery};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WingGetQueryInput {
	pub wing_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WingGetQuery {
	wing_id: Uuid,
}

impl LibraryQuery for WingGetQuery {
	type Input = WingGetQueryInput;
	type Output = WingGetOutput;

	fn from_input(input: Self::Input) -> QueryResult<Self> {
		Ok(Self {
			wing_id: input.wing_id,
		})
	}

	async fn execute(
		self,
		context: Arc<CoreContext>,
		session: crate::infra::api::SessionContext,
	) -> QueryResult<Self::Output> {
		let library_id = session
			.current_library_id
			.ok_or_else(|| QueryError::Internal("No library selected".to_string()))?;

		let library = context
			.libraries()
			.await
			.get_library(library_id)
			.await
			.ok_or_else(|| QueryError::Internal("Library not found".to_string()))?;

		let db = library.db().conn();

		let wing_model = crate::infra::db::entities::wing::Entity::find()
			.filter(crate::infra::db::entities::wing::Column::Uuid.eq(self.wing_id))
			.one(db)
			.await?
			.ok_or_else(|| QueryError::Internal(format!("Wing {} not found", self.wing_id)))?;

		let wing = Wing {
			id: wing_model.uuid,
			name: wing_model.name,
			icon: wing_model.icon,
			color: wing_model.color,
			order: wing_model.order,
			created_at: wing_model.created_at.into(),
			updated_at: wing_model.updated_at.into(),
		};

		Ok(WingGetOutput { wing })
	}
}

crate::register_library_query!(WingGetQuery, "wings.get");
