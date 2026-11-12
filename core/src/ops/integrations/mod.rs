//! # Integration operations
//!
//! Wire operations for connected accounts. Accounts belong to the library in
//! the session, so every operation here is a library action or query.
//!
//! - `integrations.accounts.list`: connected accounts and their services.
//! - `integrations.accounts.remove`: disconnect, revoke and forget an account.
//! - `integrations.google.client` and `integrations.google.set_client`: which
//!   OAuth client signs users in, and saving a custom one.
//! - `integrations.google.connect`, `integrations.google.connect_status` and
//!   `integrations.google.connect_cancel`: the browser sign-in.
//! - `integrations.google.drive.mount`: mount or unmount an account's Drive.
//! - `integrations.google.photos.*`: albums, upload and picker import.
//! - `files.cloud_edit.*`: open cloud files in local apps and write edits back.

pub mod accounts;
pub mod cloud_edit;
pub mod google;
pub mod photos;

use std::sync::Arc;

use crate::context::CoreContext;
use crate::infra::query::{QueryError, QueryResult};
use crate::library::Library;

/// Library of the session, for queries.
pub(crate) async fn session_library(
	context: &Arc<CoreContext>,
	session: &crate::infra::api::SessionContext,
) -> QueryResult<Arc<Library>> {
	let library_id = session
		.current_library_id
		.ok_or_else(|| QueryError::Internal("No library in session".to_string()))?;
	context
		.get_library(library_id)
		.await
		.ok_or_else(|| QueryError::Internal("Library not found".to_string()))
}
