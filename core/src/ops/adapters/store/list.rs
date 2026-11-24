//! `adapters.store.list`: the store from the last accepted index, without touching the network.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use specta::Type;

use super::{listing, open, StoreListing};
use crate::{
	context::CoreContext,
	infra::query::{CoreQuery, QueryError, QueryResult},
};

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct ListStoreInput {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListStoreQuery;

impl CoreQuery for ListStoreQuery {
	type Input = ListStoreInput;
	type Output = StoreListing;

	fn from_input(_input: Self::Input) -> QueryResult<Self> {
		Ok(Self)
	}

	async fn execute(
		self,
		context: Arc<CoreContext>,
		_session: crate::infra::api::SessionContext,
	) -> QueryResult<Self::Output> {
		let store = open(&context).map_err(QueryError::Internal)?;
		let index = store
			.cached_index()
			.map_err(|e| QueryError::Internal(e.to_string()))?;
		listing(&store, index.as_ref()).map_err(QueryError::Internal)
	}
}

crate::register_core_query!(ListStoreQuery, "adapters.store.list");
