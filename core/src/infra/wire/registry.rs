//! Minimal action/query registry (action-centric) using `inventory`.
//!
//! Goals:
//! - Tiny, action-centric API: register Actions, decode their associated Inputs
//! - No conversion traits on inputs; Actions declare `type Input` and `from_input(..)`
//! - Single place that resolves `library_id` and dispatches

use futures::future::{FutureExt, LocalBoxFuture};
use once_cell::sync::Lazy;
use serde::de::DeserializeOwned;
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

/// Registry handler for library queries - thin wrapper calling business logic
pub fn handle_library_query<Q>(
	context: Arc<crate::context::CoreContext>,
	session: crate::infra::api::SessionContext,
	payload: serde_json::Value,
) -> std::pin::Pin<
	Box<dyn std::future::Future<Output = Result<serde_json::Value, String>> + Send + 'static>,
>
where
	Q: crate::infra::query::LibraryQuery + 'static,
	Q::Input: serde::de::DeserializeOwned + std::fmt::Debug + 'static,
	Q::Output: serde::Serialize + std::fmt::Debug + 'static,
{
	Box::pin(async move {
		// Create dispatcher
		let dispatcher = crate::infra::api::dispatcher::ApiDispatcher::new(context.clone());

		// Deserialize input
		let input: Q::Input = serde_json::from_value(payload).map_err(|e| e.to_string())?;

		// Call business logic method
		let output = dispatcher
			.execute_library_query::<Q>(input, session)
			.await
			.map_err(|e| e.to_string())?;

		// Serialize output
		serde_json::to_value(output).map_err(|e| e.to_string())
	})
}

/// Registry handler for core queries - thin wrapper calling business logic
pub fn handle_core_query<Q>(
	context: Arc<crate::context::CoreContext>,
	session: crate::infra::api::SessionContext,
	payload: serde_json::Value,
) -> std::pin::Pin<
	Box<dyn std::future::Future<Output = Result<serde_json::Value, String>> + Send + 'static>,
>
where
	Q: crate::infra::query::CoreQuery + 'static,
	Q::Input: serde::de::DeserializeOwned + std::fmt::Debug + 'static,
	Q::Output: serde::Serialize + std::fmt::Debug + 'static,
{
	Box::pin(async move {
		// Create dispatcher
		let dispatcher = crate::infra::api::dispatcher::ApiDispatcher::new(context.clone());

		// Deserialize input
		let input: Q::Input = serde_json::from_value(payload).map_err(|e| e.to_string())?;

		// Call business logic method
		let output = dispatcher
			.execute_core_query::<Q>(input, session)
			.await
			.map_err(|e| e.to_string())?;

		// Serialize output
		serde_json::to_value(output).map_err(|e| e.to_string())
	})
}

/// Registry handler for library actions - thin wrapper calling business logic
pub fn handle_library_action<A>(
	context: Arc<crate::context::CoreContext>,
	session: crate::infra::api::SessionContext,
	payload: serde_json::Value,
) -> std::pin::Pin<
	Box<dyn std::future::Future<Output = Result<serde_json::Value, String>> + Send + 'static>,
>
where
	A: crate::infra::action::LibraryAction + 'static,
	A::Input: serde::de::DeserializeOwned + std::fmt::Debug + 'static,
	A::Output: serde::Serialize + std::fmt::Debug + 'static,
{
	Box::pin(async move {
		// Create dispatcher
		let dispatcher = crate::infra::api::dispatcher::ApiDispatcher::new(context.clone());

		// Deserialize input
		let input: A::Input = serde_json::from_value(payload).map_err(|e| e.to_string())?;

		// Call business logic method
		let output = dispatcher
			.execute_library_action::<A>(input, session)
			.await
			.map_err(|e| e.to_string())?;

		// Serialize output
		serde_json::to_value(output).map_err(|e| e.to_string())
	})
}

/// Registry handler for core actions - thin wrapper calling business logic
pub fn handle_core_action<A>(
	context: Arc<crate::context::CoreContext>,
	payload: serde_json::Value,
) -> std::pin::Pin<
	Box<dyn std::future::Future<Output = Result<serde_json::Value, String>> + Send + 'static>,
>
where
	A: crate::infra::action::CoreAction + 'static,
	A::Input: serde::de::DeserializeOwned + std::fmt::Debug + 'static,
	A::Output: serde::Serialize + std::fmt::Debug + 'static,
{
	Box::pin(async move {
		// Create dispatcher
		let dispatcher = crate::infra::api::dispatcher::ApiDispatcher::new(context.clone());

		// Create base session
		let session = dispatcher
			.create_base_session()
			.map_err(|e| e.to_string())?;

		// Deserialize input
		let input: A::Input = serde_json::from_value(payload).map_err(|e| e.to_string())?;

		// Call business logic method
		let output = dispatcher
			.execute_core_action::<A>(input, session)
			.await
			.map_err(|e| e.to_string())?;

		// Serialize output
		serde_json::to_value(output).map_err(|e| e.to_string())
	})
}

/// Handler function signature for library queries.
pub type LibraryQueryHandlerFn = fn(
	Arc<crate::context::CoreContext>,
	crate::infra::api::SessionContext, // session with library context
	serde_json::Value,                 // payload with Q::Input as JSON
) -> std::pin::Pin<
	Box<dyn std::future::Future<Output = Result<serde_json::Value, String>> + Send + 'static>,
>;

/// Handler function signature for core queries.
pub type CoreQueryHandlerFn = fn(
	Arc<crate::context::CoreContext>,
	crate::infra::api::SessionContext, // session context
	serde_json::Value,                 // payload with Q::Input as JSON
) -> std::pin::Pin<
	Box<dyn std::future::Future<Output = Result<serde_json::Value, String>> + Send + 'static>,
>;

/// Handler function signature for library actions.
pub type LibraryActionHandlerFn = fn(
	Arc<crate::context::CoreContext>,
	crate::infra::api::SessionContext, // session with library context
	serde_json::Value,                 // payload with A::Input as JSON
) -> std::pin::Pin<
	Box<dyn std::future::Future<Output = Result<serde_json::Value, String>> + Send + 'static>,
>;

/// Handler function signature for core actions.
pub type CoreActionHandlerFn = fn(
	Arc<crate::context::CoreContext>,
	serde_json::Value, // payload with A::Input as JSON
) -> std::pin::Pin<
	Box<dyn std::future::Future<Output = Result<serde_json::Value, String>> + Send + 'static>,
>;

/// Registry entry for a library query operation.
pub struct LibraryQueryEntry {
	pub method: &'static str,
	pub handler: LibraryQueryHandlerFn,
}

/// Registry entry for a core query operation.
pub struct CoreQueryEntry {
	pub method: &'static str,
	pub handler: CoreQueryHandlerFn,
}

/// Registry entry for a library action operation.
pub struct LibraryActionEntry {
	pub method: &'static str,
	pub handler: LibraryActionHandlerFn,
}

/// Registry entry for a core action operation.
pub struct CoreActionEntry {
	pub method: &'static str,
	pub handler: CoreActionHandlerFn,
}

inventory::collect!(LibraryQueryEntry);
inventory::collect!(CoreQueryEntry);
inventory::collect!(LibraryActionEntry);
inventory::collect!(CoreActionEntry);

pub static LIBRARY_QUERIES: Lazy<HashMap<&'static str, LibraryQueryHandlerFn>> = Lazy::new(|| {
	let mut map = HashMap::new();
	for entry in inventory::iter::<LibraryQueryEntry>() {
		map.insert(entry.method, entry.handler);
	}
	map
});

pub static CORE_QUERIES: Lazy<HashMap<&'static str, CoreQueryHandlerFn>> = Lazy::new(|| {
	let mut map = HashMap::new();
	for entry in inventory::iter::<CoreQueryEntry>() {
		map.insert(entry.method, entry.handler);
	}
	map
});

pub static LIBRARY_ACTIONS: Lazy<HashMap<&'static str, LibraryActionHandlerFn>> = Lazy::new(|| {
	let mut map = HashMap::new();
	for entry in inventory::iter::<LibraryActionEntry>() {
		map.insert(entry.method, entry.handler);
	}
	map
});

pub static CORE_ACTIONS: Lazy<HashMap<&'static str, CoreActionHandlerFn>> = Lazy::new(|| {
	let mut map = HashMap::new();
	for entry in inventory::iter::<CoreActionEntry>() {
		map.insert(entry.method, entry.handler);
	}
	map
});

#[cfg(test)]
mod tests {
	use std::collections::HashSet;

	use super::*;

	#[test]
	fn registered_methods_are_unique_well_formed_and_never_lost_in_maps() {
		let queries: Vec<_> = inventory::iter::<CoreQueryEntry>()
			.map(|e| e.method)
			.chain(inventory::iter::<LibraryQueryEntry>().map(|e| e.method))
			.collect();
		let actions: Vec<_> = inventory::iter::<CoreActionEntry>()
			.map(|e| e.method)
			.chain(inventory::iter::<LibraryActionEntry>().map(|e| e.method))
			.collect();
		assert!(!queries.is_empty());
		assert!(!actions.is_empty());
		let mut seen = HashSet::new();
		for (methods, prefix, suffix) in [(&queries, "query:", ""), (&actions, "action:", ".input")]
		{
			for method in methods {
				assert!(seen.insert(*method), "duplicate registry method: {method}");
				let identifier = method
					.strip_prefix(prefix)
					.unwrap()
					.strip_suffix(suffix)
					.unwrap();
				assert!(identifier.contains('.'), "missing namespace: {method}");
				for segment in identifier.split('.') {
					assert!(!segment.is_empty(), "empty method segment: {method}");
					assert!(
						segment
							.bytes()
							.all(|b| b.is_ascii_alphanumeric() || b == b'_'),
						"invalid method segment: {method}"
					);
				}
			}
		}
		assert_eq!(queries.len(), CORE_QUERIES.len() + LIBRARY_QUERIES.len());
		assert_eq!(actions.len(), CORE_ACTIONS.len() + LIBRARY_ACTIONS.len());
	}

	#[test]
	fn extracted_types_match_every_registered_method_and_scope() {
		use crate::infra::wire::type_extraction::{
			create_wingdrive_api_structure, generate_wingdrive_api, OperationScope, QueryScope,
		};

		let (operations, queries, _) = generate_wingdrive_api();
		let mut methods = HashSet::new();
		for op in &operations {
			assert!(
				methods.insert(op.wire_method.as_str()),
				"duplicate metadata: {}",
				op.identifier
			);
			assert_eq!(op.wire_method, format!("action:{}.input", op.identifier));
			let registered = match op.scope {
				OperationScope::Core => CORE_ACTIONS.contains_key(op.wire_method.as_str()),
				OperationScope::Library => LIBRARY_ACTIONS.contains_key(op.wire_method.as_str()),
			};
			assert!(registered, "{}", op.wire_method);
			assert!(!op.input_type_name_ts.is_empty());
			assert!(!op.output_type_name_ts.is_empty());
		}
		for query in &queries {
			assert!(
				methods.insert(query.wire_method.as_str()),
				"duplicate metadata: {}",
				query.identifier
			);
			assert_eq!(query.wire_method, format!("query:{}", query.identifier));
			let registry = match query.scope {
				QueryScope::Core => &*CORE_QUERIES,
				QueryScope::Library => &*LIBRARY_QUERIES,
			};
			assert!(
				registry.contains_key(query.wire_method.as_str()),
				"{}",
				query.wire_method
			);
		}
		assert_eq!(operations.len(), CORE_ACTIONS.len() + LIBRARY_ACTIONS.len());
		assert_eq!(queries.len(), CORE_QUERIES.len() + LIBRARY_QUERIES.len());
		let api = create_wingdrive_api_structure(&operations, &queries);
		for (group, registry) in [
			(
				&api.core_actions,
				CORE_ACTIONS.keys().copied().collect::<HashSet<_>>(),
			),
			(
				&api.library_actions,
				LIBRARY_ACTIONS.keys().copied().collect::<HashSet<_>>(),
			),
		] {
			assert_eq!(group.len(), registry.len());
			for op in group {
				assert!(registry.contains(op.wire_method.as_str()));
				let original = operations
					.iter()
					.find(|m| m.identifier == op.identifier)
					.unwrap();
				assert_eq!(op.input_type_name_ts, original.input_type_name_ts);
				assert_eq!(op.output_type_name_ts, original.output_type_name_ts);
			}
		}
		assert_eq!(api.core_queries.len(), CORE_QUERIES.len());
		assert_eq!(api.library_queries.len(), LIBRARY_QUERIES.len());
	}

	#[test]
	fn list_registered_ops() {
		// Collect and display registered actions
		let mut action_methods: Vec<&'static str> = crate::infra::wire::registry::CORE_ACTIONS
			.keys()
			.cloned()
			.collect();
		action_methods.sort();
		println!("Registered actions ({}):", action_methods.len());
		for method in &action_methods {
			println!("  {}", method);
		}

		let mut library_action_methods: Vec<&'static str> =
			crate::infra::wire::registry::LIBRARY_ACTIONS
				.keys()
				.cloned()
				.collect();
		library_action_methods.sort();
		println!(
			"Registered library actions ({}):",
			library_action_methods.len()
		);
		for method in &library_action_methods {
			println!("  {}", method);
		}

		// Collect and display registered queries
		let mut query_methods: Vec<&'static str> = crate::infra::wire::registry::CORE_QUERIES
			.keys()
			.cloned()
			.collect();
		query_methods.sort();
		println!("Registered queries ({}):", query_methods.len());
		for method in &query_methods {
			println!("  {}", method);
		}

		let mut library_query_methods: Vec<&'static str> =
			crate::infra::wire::registry::LIBRARY_QUERIES
				.keys()
				.cloned()
				.collect();
		library_query_methods.sort();
		println!(
			"Registered library queries ({}):",
			library_query_methods.len()
		);
		for method in &library_query_methods {
			println!("  {}", method);
		}

		// Ensure we have at least one action or query registered
		assert!(
			!action_methods.is_empty()
				|| !query_methods.is_empty()
				|| !library_action_methods.is_empty()
				|| !library_query_methods.is_empty(),
			"No actions or queries registered"
		);
	}
}

/// Helper: construct action method string from a short name like "files.copy"
#[macro_export]
macro_rules! action_method {
	($name:literal) => {
		concat!("action:", $name, ".input")
	};
}

/// Helper: construct query method string from a short name like "core.status"
#[macro_export]
macro_rules! query_method {
	($name:literal) => {
		concat!("query:", $name, "")
	};
}

/// Register a library query `Q` by short name; binds method to `Q::Input` and handler to `handle_library_query::<Q>`.
/// Implements QueryTypeInfo trait for automatic type extraction
#[macro_export]
macro_rules! register_library_query {
	($query:ty, $name:literal) => {
		impl $crate::client::Wire for <$query as $crate::infra::query::LibraryQuery>::Input {
			const METHOD: &'static str = $crate::query_method!($name);
		}
		inventory::submit! {
			$crate::infra::wire::registry::LibraryQueryEntry {
				method: <<$query as $crate::infra::query::LibraryQuery>::Input as $crate::client::Wire>::METHOD,
				handler: $crate::infra::wire::registry::handle_library_query::<$query>,
			}
		}

		// Automatic QueryTypeInfo implementation for type extraction
		impl $crate::infra::wire::type_extraction::QueryTypeInfo for $query {
			type Input = <$query as $crate::infra::query::LibraryQuery>::Input;
			type Output = <$query as $crate::infra::query::LibraryQuery>::Output;

			fn identifier() -> &'static str {
				$name
			}

			fn scope() -> $crate::infra::wire::type_extraction::QueryScope {
				$crate::infra::wire::type_extraction::QueryScope::Library
			}

			fn wire_method() -> String {
				$crate::query_method!($name).to_string()
			}
		}

		// Submit query type extractor to inventory
		inventory::submit! {
			$crate::infra::wire::type_extraction::QueryExtractorEntry {
				extractor: <$query as $crate::infra::wire::type_extraction::QueryTypeInfo>::extract_types,
				identifier: $name,
			}
		}
	};
}

/// Register a core query `Q` by short name; binds method to `Q::Input` and handler to `handle_core_query::<Q>`.
/// Implements QueryTypeInfo trait for automatic type extraction
#[macro_export]
macro_rules! register_core_query {
	($query:ty, $name:literal) => {
		impl $crate::client::Wire for <$query as $crate::infra::query::CoreQuery>::Input {
			const METHOD: &'static str = $crate::query_method!($name);
		}
		inventory::submit! {
			$crate::infra::wire::registry::CoreQueryEntry {
				method: <<$query as $crate::infra::query::CoreQuery>::Input as $crate::client::Wire>::METHOD,
				handler: $crate::infra::wire::registry::handle_core_query::<$query>,
			}
		}

		// Automatic QueryTypeInfo implementation for type extraction
		impl $crate::infra::wire::type_extraction::QueryTypeInfo for $query {
			type Input = <$query as $crate::infra::query::CoreQuery>::Input;
			type Output = <$query as $crate::infra::query::CoreQuery>::Output;

			fn identifier() -> &'static str {
				$name
			}

			fn scope() -> $crate::infra::wire::type_extraction::QueryScope {
				$crate::infra::wire::type_extraction::QueryScope::Core
			}

			fn wire_method() -> String {
				$crate::query_method!($name).to_string()
			}
		}

		// Submit query type extractor to inventory
		inventory::submit! {
			$crate::infra::wire::type_extraction::QueryExtractorEntry {
				extractor: <$query as $crate::infra::wire::type_extraction::QueryTypeInfo>::extract_types,
				identifier: $name,
			}
		}
	};
}

/// Register a library action `A` by short name; binds method to `A::Input` and handler to `handle_library_action::<A>`.
/// Implements OperationTypeInfo trait for automatic type extraction
#[macro_export]
macro_rules! register_library_action {
	($action:ty, $name:literal) => {
		impl $crate::client::Wire for <$action as $crate::infra::action::LibraryAction>::Input {
			const METHOD: &'static str = $crate::action_method!($name);
		}
		inventory::submit! {
			$crate::infra::wire::registry::LibraryActionEntry {
				method: <<$action as $crate::infra::action::LibraryAction>::Input as $crate::client::Wire>::METHOD,
				handler: $crate::infra::wire::registry::handle_library_action::<$action>,
			}
		}

		// Automatic OperationTypeInfo implementation for type extraction
		impl $crate::infra::wire::type_extraction::OperationTypeInfo for $action {
			type Input = <$action as $crate::infra::action::LibraryAction>::Input;
			type Output = <$action as $crate::infra::action::LibraryAction>::Output;

			fn identifier() -> &'static str {
				$name
			}

			fn scope() -> $crate::infra::wire::type_extraction::OperationScope {
				$crate::infra::wire::type_extraction::OperationScope::Library
			}

			fn wire_method() -> String {
				$crate::action_method!($name).to_string()
			}
		}

		// Submit type extractor to inventory for compile-time collection
		inventory::submit! {
			$crate::infra::wire::type_extraction::TypeExtractorEntry {
				extractor: <$action as $crate::infra::wire::type_extraction::OperationTypeInfo>::extract_types,
				identifier: $name,
			}
		}
	};
}

/// Register a core action `A` similarly.
/// Implements OperationTypeInfo trait for automatic type extraction
#[macro_export]
macro_rules! register_core_action {
	($action:ty, $name:literal) => {
		impl $crate::client::Wire for <$action as $crate::infra::action::CoreAction>::Input {
			const METHOD: &'static str = $crate::action_method!($name);
		}
		inventory::submit! {
			$crate::infra::wire::registry::CoreActionEntry {
				method: <<$action as $crate::infra::action::CoreAction>::Input as $crate::client::Wire>::METHOD,
				handler: $crate::infra::wire::registry::handle_core_action::<$action>,
			}
		}

		// Automatic OperationTypeInfo implementation for core actions
		impl $crate::infra::wire::type_extraction::OperationTypeInfo for $action {
			type Input = <$action as $crate::infra::action::CoreAction>::Input;
			type Output = <$action as $crate::infra::action::CoreAction>::Output;

			fn identifier() -> &'static str {
				$name
			}

			fn scope() -> $crate::infra::wire::type_extraction::OperationScope {
				$crate::infra::wire::type_extraction::OperationScope::Core
			}

			fn wire_method() -> String {
				$crate::action_method!($name).to_string()
			}
		}

		// Submit type extractor to inventory for compile-time collection
		inventory::submit! {
			$crate::infra::wire::type_extraction::TypeExtractorEntry {
				extractor: <$action as $crate::infra::wire::type_extraction::OperationTypeInfo>::extract_types,
				identifier: $name,
			}
		}
	};
}
