//! Permission and authorization layer
//!
//! This module handles authentication and authorization for all API operations.
//! It provides fine-grained control over what operations each session can execute.

use super::{error::ApiError, session::SessionContext};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

pub use super::session::AuthLevel;

/// Permission checking and enforcement
#[derive(Clone)]
pub struct PermissionLayer {
	/// Permission policies and rules
	policies: PermissionPolicies,
}

/// Permission policies configuration
#[derive(Debug, Clone)]
pub struct PermissionPolicies {
	/// Whether to enforce permissions (can disable for development)
	pub enforce: bool,

	/// Default permissions for device-level access
	pub device_defaults: PermissionSet,

	/// Library-specific permission overrides
	pub library_overrides: HashMap<Uuid, LibraryPermissions>,
}

/// Complete permission set for a session
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PermissionSet {
	/// Core system permissions
	pub core: CorePermissions,

	/// Library operation permissions
	pub library: LibraryPermissions,

	/// Network operation permissions
	pub network: NetworkPermissions,

	/// Job management permissions
	pub jobs: JobPermissions,
}

/// Core system permissions
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CorePermissions {
	pub can_read_status: bool,
	pub can_manage_libraries: bool,
	pub can_modify_settings: bool,
	pub can_manage_devices: bool,
}

/// Library operation permissions
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct LibraryPermissions {
	pub can_read: bool,
	pub can_write: bool,
	pub can_delete: bool,
	pub can_manage_locations: bool,
	pub can_manage_tags: bool,
	pub can_search: bool,
	pub can_index: bool,
}

/// Network operation permissions
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct NetworkPermissions {
	pub can_start_stop: bool,
	pub can_pair_devices: bool,
	pub can_send_spacedrop: bool,
	pub can_manage_devices: bool,
}

/// Job management permissions
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct JobPermissions {
	pub can_list: bool,
	pub can_pause_resume: bool,
	pub can_cancel: bool,
	pub can_view_details: bool,
}

/// Permission-related errors
#[derive(Debug, Error, Serialize, Deserialize, Type)]
pub enum PermissionError {
	#[error("Authentication required")]
	Unauthenticated,

	#[error("Insufficient privileges for this operation")]
	InsufficientPrivileges,

	#[error("Library access denied: {library_id}")]
	LibraryAccessDenied { library_id: Uuid },

	#[error("Operation not allowed: {operation}")]
	OperationNotAllowed { operation: String },

	#[error("Rate limit exceeded")]
	RateLimitExceeded,
}

impl PermissionLayer {
	/// Create a new permission layer with default policies
	pub fn new() -> Self {
		Self {
			policies: PermissionPolicies {
				enforce: true,
				device_defaults: PermissionSet::device_default(),
				library_overrides: HashMap::new(),
			},
		}
	}

	/// Create a permissive layer for development
	pub fn permissive() -> Self {
		Self {
			policies: PermissionPolicies {
				enforce: false,
				device_defaults: PermissionSet::admin_all(),
				library_overrides: HashMap::new(),
			},
		}
	}

	/// Check if session can execute a library action
	pub async fn check_library_action<A>(
		&self,
		session: &SessionContext,
		_action_type: std::marker::PhantomData<A>,
	) -> Result<(), PermissionError>
	where
		A: crate::infra::action::LibraryAction,
	{
		if !self.policies.enforce {
			return Ok(());
		}

		// Check basic authentication
		if session.auth.authentication_level == AuthLevel::None {
			return Err(PermissionError::Unauthenticated);
		}

		// Check library access
		if session.current_library_id.is_none() {
			return Err(PermissionError::OperationNotAllowed {
				operation: "Library action requires library context".to_string(),
			});
		}

		// Check library permissions
		if !session.permissions.library.can_write {
			return Err(PermissionError::InsufficientPrivileges);
		}

		// Future: More fine-grained checks based on action type
		// match std::any::type_name::<A>() {
		//     "FileCopyAction" => check_file_permissions(),
		//     "IndexingAction" => check_indexing_permissions(),
		//     _ => Ok(()),
		// }

		Ok(())
	}

	/// Check if session can execute a core action
	pub async fn check_core_action<A>(
		&self,
		session: &SessionContext,
		_action_type: std::marker::PhantomData<A>,
	) -> Result<(), PermissionError>
	where
		A: crate::infra::action::CoreAction,
	{
		if !self.policies.enforce {
			return Ok(());
		}

		// Core actions typically need higher privileges
		match session.auth.authentication_level {
			AuthLevel::Device | AuthLevel::User(_) | AuthLevel::Admin(_) => {
				if session.permissions.core.can_manage_libraries {
					Ok(())
				} else {
					Err(PermissionError::InsufficientPrivileges)
				}
			}
			AuthLevel::None => Err(PermissionError::Unauthenticated),
		}
	}

	/// Check if session can execute a library query
	pub async fn check_library_query<Q>(
		&self,
		session: &SessionContext,
		_query_type: std::marker::PhantomData<Q>,
	) -> Result<(), PermissionError>
	where
		Q: crate::infra::query::LibraryQuery,
	{
		if !self.policies.enforce {
			return Ok(());
		}

		// Queries typically need read permissions
		if session.auth.authentication_level == AuthLevel::None {
			return Err(PermissionError::Unauthenticated);
		}

		if !session.permissions.library.can_read {
			return Err(PermissionError::InsufficientPrivileges);
		}

		Ok(())
	}

	/// Check if session can execute a core query
	pub async fn check_core_query<Q>(
		&self,
		session: &SessionContext,
		_query_type: std::marker::PhantomData<Q>,
	) -> Result<(), PermissionError>
	where
		Q: crate::infra::query::CoreQuery,
	{
		if !self.policies.enforce {
			return Ok(());
		}

		// Core queries need basic authentication
		if session.auth.authentication_level == AuthLevel::None {
			return Err(PermissionError::Unauthenticated);
		}

		if !session.permissions.core.can_read_status {
			return Err(PermissionError::InsufficientPrivileges);
		}

		Ok(())
	}
}

impl PermissionSet {
	/// Full admin permissions
	pub fn admin_all() -> Self {
		Self {
			core: CorePermissions {
				can_read_status: true,
				can_manage_libraries: true,
				can_modify_settings: true,
				can_manage_devices: true,
			},
			library: LibraryPermissions {
				can_read: true,
				can_write: true,
				can_delete: true,
				can_manage_locations: true,
				can_manage_tags: true,
				can_search: true,
				can_index: true,
			},
			network: NetworkPermissions {
				can_start_stop: true,
				can_pair_devices: true,
				can_send_spacedrop: true,
				can_manage_devices: true,
			},
			jobs: JobPermissions {
				can_list: true,
				can_pause_resume: true,
				can_cancel: true,
				can_view_details: true,
			},
		}
	}

	/// Default device permissions (current behavior)
	pub fn device_default() -> Self {
		Self::admin_all() // For now, maintain current permissive behavior
	}

	/// Read-only permissions
	pub fn read_only() -> Self {
		Self {
			core: CorePermissions {
				can_read_status: true,
				can_manage_libraries: false,
				can_modify_settings: false,
				can_manage_devices: false,
			},
			library: LibraryPermissions {
				can_read: true,
				can_write: false,
				can_delete: false,
				can_manage_locations: false,
				can_manage_tags: false,
				can_search: true,
				can_index: false,
			},
			network: NetworkPermissions {
				can_start_stop: false,
				can_pair_devices: false,
				can_send_spacedrop: false,
				can_manage_devices: false,
			},
			jobs: JobPermissions {
				can_list: true,
				can_pause_resume: false,
				can_cancel: false,
				can_view_details: true,
			},
		}
	}
}

/// Convert permission errors to API errors
impl From<PermissionError> for ApiError {
	fn from(err: PermissionError) -> Self {
		match err {
			PermissionError::Unauthenticated => Self::Unauthenticated,
			PermissionError::InsufficientPrivileges => Self::InsufficientPermissions {
				reason: "Operation not allowed".to_string(),
			},
			PermissionError::LibraryAccessDenied { library_id } => Self::LibraryNotFound {
				library_id: library_id.to_string(),
			},
			PermissionError::OperationNotAllowed { operation } => {
				Self::InsufficientPermissions { reason: operation }
			}
			PermissionError::RateLimitExceeded => Self::RateLimitExceeded {
				retry_after_seconds: 60,
			},
		}
	}
}

#[cfg(test)]
mod tests {
	use std::marker::PhantomData;

	use super::*;
	use crate::ops::{
		core::status::query::CoreStatusQuery,
		files::{create_folder::action::CreateFolderAction, query::file_by_id::FileByIdQuery},
		libraries::create::LibraryCreateAction,
	};

	#[tokio::test]
	async fn unauthenticated_sessions_cannot_execute_any_operation_kind() {
		let mut session = SessionContext::device_session(Uuid::nil(), "test".into())
			.with_library(Uuid::from_u128(1));
		session.auth.authentication_level = AuthLevel::None;
		let layer = PermissionLayer::new();
		let results = [
			layer
				.check_core_action(&session, PhantomData::<LibraryCreateAction>)
				.await,
			layer
				.check_library_action(&session, PhantomData::<CreateFolderAction>)
				.await,
			layer
				.check_core_query(&session, PhantomData::<CoreStatusQuery>)
				.await,
			layer
				.check_library_query(&session, PhantomData::<FileByIdQuery>)
				.await,
		];
		for result in results {
			assert!(matches!(result, Err(PermissionError::Unauthenticated)));
		}
	}

	#[tokio::test]
	async fn read_only_sessions_can_query_but_cannot_mutate_core_or_library() {
		let mut session = SessionContext::device_session(Uuid::nil(), "test".into())
			.with_library(Uuid::from_u128(1));
		session.permissions = PermissionSet::read_only();
		let layer = PermissionLayer::new();
		layer
			.check_core_query(&session, PhantomData::<CoreStatusQuery>)
			.await
			.unwrap();
		layer
			.check_library_query(&session, PhantomData::<FileByIdQuery>)
			.await
			.unwrap();
		assert!(matches!(
			layer
				.check_core_action(&session, PhantomData::<LibraryCreateAction>)
				.await,
			Err(PermissionError::InsufficientPrivileges)
		));
		assert!(matches!(
			layer
				.check_library_action(&session, PhantomData::<CreateFolderAction>)
				.await,
			Err(PermissionError::InsufficientPrivileges)
		));
		session.permissions.library.can_read = false;
		session.permissions.core.can_read_status = false;
		assert!(matches!(
			layer
				.check_library_query(&session, PhantomData::<FileByIdQuery>)
				.await,
			Err(PermissionError::InsufficientPrivileges)
		));
		assert!(matches!(
			layer
				.check_core_query(&session, PhantomData::<CoreStatusQuery>)
				.await,
			Err(PermissionError::InsufficientPrivileges)
		));
	}

	#[tokio::test]
	async fn authenticated_levels_require_library_context_and_explicit_write_permission() {
		let layer = PermissionLayer::new();
		for level in [
			AuthLevel::Device,
			AuthLevel::User(Uuid::nil()),
			AuthLevel::Admin(Uuid::nil()),
		] {
			let mut session = SessionContext::device_session(Uuid::nil(), "test".into());
			session.auth.authentication_level = level;
			layer
				.check_core_action(&session, PhantomData::<LibraryCreateAction>)
				.await
				.unwrap();
			assert!(matches!(
				layer
					.check_library_action(&session, PhantomData::<CreateFolderAction>)
					.await,
				Err(PermissionError::OperationNotAllowed { .. })
			));
			session.current_library_id = Some(Uuid::from_u128(1));
			layer
				.check_library_action(&session, PhantomData::<CreateFolderAction>)
				.await
				.unwrap();
			session.permissions.library.can_write = false;
			assert!(matches!(
				layer
					.check_library_action(&session, PhantomData::<CreateFolderAction>)
					.await,
				Err(PermissionError::InsufficientPrivileges)
			));
		}
	}

	#[tokio::test]
	async fn permissive_layer_bypasses_authentication_context_and_permission_checks() {
		let mut session = SessionContext::device_session(Uuid::nil(), "test".into());
		session.auth.authentication_level = AuthLevel::None;
		session.permissions = PermissionSet::read_only();
		session.permissions.library.can_read = false;
		session.permissions.core.can_read_status = false;
		let layer = PermissionLayer::permissive();
		layer
			.check_core_action(&session, PhantomData::<LibraryCreateAction>)
			.await
			.unwrap();
		layer
			.check_library_action(&session, PhantomData::<CreateFolderAction>)
			.await
			.unwrap();
		layer
			.check_core_query(&session, PhantomData::<CoreStatusQuery>)
			.await
			.unwrap();
		layer
			.check_library_query(&session, PhantomData::<FileByIdQuery>)
			.await
			.unwrap();
	}

	#[test]
	fn permission_errors_preserve_public_status_and_payload() {
		for (error, status, payload) in [
			(
				PermissionError::Unauthenticated,
				401,
				serde_json::json!("Unauthenticated"),
			),
			(
				PermissionError::InsufficientPrivileges,
				403,
				serde_json::json!({"InsufficientPermissions": {"reason": "Operation not allowed"}}),
			),
			(
				PermissionError::LibraryAccessDenied {
					library_id: Uuid::nil(),
				},
				404,
				serde_json::json!({"LibraryNotFound": {"library_id": Uuid::nil().to_string()}}),
			),
			(
				PermissionError::OperationNotAllowed {
					operation: "blocked".into(),
				},
				403,
				serde_json::json!({"InsufficientPermissions": {"reason": "blocked"}}),
			),
			(
				PermissionError::RateLimitExceeded,
				429,
				serde_json::json!({"RateLimitExceeded": {"retry_after_seconds": 60}}),
			),
		] {
			let api: ApiError = error.into();
			assert_eq!(api.status_code(), status);
			assert!(api.is_client_error());
			assert_eq!(serde_json::to_value(api).unwrap(), payload);
		}
	}
}
