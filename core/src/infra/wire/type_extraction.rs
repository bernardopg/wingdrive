//! rspc-inspired trait-based type extraction for automatic API generation
//!
//! This module implements the core trait system that allows automatic discovery
//! and extraction of Input/Output types from registered operations at compile-time.

use serde::{de::DeserializeOwned, Serialize};
use specta::{DataType, Type, TypeCollection};

/// Operation scope - automatically determined by registration macro
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationScope {
	Core,
	Library,
}

/// Query scope - automatically determined by registration macro
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryScope {
	Core,
	Library,
}

/// Core trait that provides compile-time type information for operations
///
/// This is inspired by rspc's resolver trait system and enables automatic
/// type extraction without runtime iteration over inventory data.
pub trait OperationTypeInfo {
	/// The input type for this operation
	type Input: Type + Serialize + DeserializeOwned + 'static;

	/// The output type for this operation
	type Output: Type + Serialize + DeserializeOwned + 'static;

	/// The operation identifier (e.g., "files.copy", "libraries.create")
	fn identifier() -> &'static str;

	/// The operation scope (Core or Library) - automatically determined by registration macro
	fn scope() -> OperationScope;

	/// Generate the wire method string for this operation
	fn wire_method() -> String;

	/// Extract type metadata and register with Specta's TypeCollection
	/// This is the key method that enables automatic type discovery
	fn extract_types(collection: &mut TypeCollection) -> OperationMetadata {
		// Register the types with Specta and get their DataType definitions
		let input_type = Self::Input::definition(collection);
		let output_type = Self::Output::definition(collection);

		let input_raw = std::any::type_name::<Self::Input>();
		let output_raw = std::any::type_name::<Self::Output>();

		OperationMetadata {
			identifier: Self::identifier(),
			wire_method: Self::wire_method(),
			input_type,
			output_type,
			input_type_name: extract_type_name_swift(input_raw),
			output_type_name: extract_type_name_swift(output_raw),
			input_type_name_ts: extract_type_name_typescript(input_raw),
			output_type_name_ts: extract_type_name_typescript(output_raw),
			scope: Self::scope(),
		}
	}
}

/// Similar trait for query operations
pub trait QueryTypeInfo {
	/// Query input type (often () for queries with no parameters)
	type Input: Type + Serialize + DeserializeOwned + 'static;

	/// Query output type
	type Output: Type + Serialize + DeserializeOwned + 'static;

	/// Query identifier (e.g., "jobs.list", "libraries.list")
	fn identifier() -> &'static str;

	/// The query scope (Core or Library) - automatically determined by registration macro
	fn scope() -> QueryScope;

	/// Generate wire method for queries
	fn wire_method() -> String;

	/// Extract query type metadata
	fn extract_types(collection: &mut TypeCollection) -> QueryMetadata {
		// Register the types with Specta and get their DataType definitions
		let input_type = Self::Input::definition(collection);
		let output_type = Self::Output::definition(collection);

		let input_raw = std::any::type_name::<Self::Input>();
		let output_raw = std::any::type_name::<Self::Output>();

		QueryMetadata {
			identifier: Self::identifier(),
			wire_method: Self::wire_method(),
			input_type,
			output_type,
			input_type_name: extract_type_name_swift(input_raw),
			output_type_name: extract_type_name_swift(output_raw),
			input_type_name_ts: extract_type_name_typescript(input_raw),
			output_type_name_ts: extract_type_name_typescript(output_raw),
			scope: Self::scope(),
		}
	}
}

/// Metadata extracted from an operation
#[derive(Debug, Clone)]
pub struct OperationMetadata {
	pub identifier: &'static str,
	pub wire_method: String,
	pub input_type: DataType,
	pub output_type: DataType,
	pub input_type_name: String,
	pub output_type_name: String,
	/// Same references formatted for TypeScript, where arrays need `T[]` syntax
	pub input_type_name_ts: String,
	pub output_type_name_ts: String,
	pub scope: OperationScope,
}

/// Metadata extracted from a query
#[derive(Debug, Clone)]
pub struct QueryMetadata {
	pub identifier: &'static str,
	pub wire_method: String,
	pub input_type: DataType,
	pub output_type: DataType,
	pub input_type_name: String,
	pub output_type_name: String,
	/// Same references formatted for TypeScript, where arrays need `T[]` syntax
	pub input_type_name_ts: String,
	pub output_type_name_ts: String,
	pub scope: QueryScope,
}

/// Entry for collecting type extractors via inventory
/// This is the key that makes compile-time collection possible
pub struct TypeExtractorEntry {
	/// Function that extracts operation metadata and registers types
	pub extractor: fn(&mut TypeCollection) -> OperationMetadata,
	pub identifier: &'static str,
}

/// Entry for collecting query type extractors
pub struct QueryExtractorEntry {
	/// Function that extracts query metadata and registers types
	pub extractor: fn(&mut TypeCollection) -> QueryMetadata,
	pub identifier: &'static str,
}

// Collect type extractors via inventory - this enables compile-time discovery
inventory::collect!(TypeExtractorEntry);
inventory::collect!(QueryExtractorEntry);

/// Generate complete API metadata by running all collected type extractors
///
/// This is the rspc-inspired magic: we iterate over compile-time registered
/// extractors rather than runtime data, solving the timeline problem.
pub fn generate_spacedrive_api() -> (Vec<OperationMetadata>, Vec<QueryMetadata>, TypeCollection) {
	let mut collection = TypeCollection::default();
	let mut operations = Vec::new();
	let mut queries = Vec::new();

	// Extract all operations - this works because extractors are registered at compile-time
	for entry in inventory::iter::<TypeExtractorEntry>() {
		let metadata = (entry.extractor)(&mut collection);
		operations.push(metadata);
	}

	// Extract all queries
	for entry in inventory::iter::<QueryExtractorEntry>() {
		let metadata = (entry.extractor)(&mut collection);
		queries.push(metadata);
	}

	// Register event types in the same collection to avoid duplicates
	collection.register_mut::<crate::infra::event::Event>();
	collection.register_mut::<crate::infra::event::FsRawEventKind>();
	collection.register_mut::<crate::infra::event::FileOperation>();

	(operations, queries, collection)
}

/// Generate the complete WingDrive API structure as a Specta-compatible type
///
/// This creates a runtime representation of our API structure that Specta can export.
/// Similar to rspc's approach with TypesOrType, but tailored for WingDrive's needs.
pub fn create_spacedrive_api_structure(
	operations: &[OperationMetadata],
	queries: &[QueryMetadata],
) -> SpacedriveApiStructure {
	let mut core_actions = Vec::new();
	let mut library_actions = Vec::new();
	let mut core_queries = Vec::new();
	let mut library_queries = Vec::new();

	// Group operations by scope - preserve the actual DataType objects!
	for op in operations {
		match op.scope {
			OperationScope::Core => {
				core_actions.push(ApiOperationType {
					identifier: op.identifier.to_string(),
					wire_method: op.wire_method.clone(),
					input_type: op.input_type.clone(),
					output_type: op.output_type.clone(),
					input_type_name: op.input_type_name.clone(),
					output_type_name: op.output_type_name.clone(),
					input_type_name_ts: op.input_type_name_ts.clone(),
					output_type_name_ts: op.output_type_name_ts.clone(),
				});
			}
			OperationScope::Library => {
				library_actions.push(ApiOperationType {
					identifier: op.identifier.to_string(),
					wire_method: op.wire_method.clone(),
					input_type: op.input_type.clone(),
					output_type: op.output_type.clone(),
					input_type_name: op.input_type_name.clone(),
					output_type_name: op.output_type_name.clone(),
					input_type_name_ts: op.input_type_name_ts.clone(),
					output_type_name_ts: op.output_type_name_ts.clone(),
				});
			}
		}
	}

	// Group queries by scope - preserve the actual DataType objects!
	for query in queries {
		match query.scope {
			QueryScope::Core => {
				core_queries.push(ApiQueryType {
					identifier: query.identifier.to_string(),
					wire_method: query.wire_method.clone(),
					input_type: query.input_type.clone(),
					output_type: query.output_type.clone(),
					input_type_name: query.input_type_name.clone(),
					output_type_name: query.output_type_name.clone(),
					input_type_name_ts: query.input_type_name_ts.clone(),
					output_type_name_ts: query.output_type_name_ts.clone(),
				});
			}
			QueryScope::Library => {
				library_queries.push(ApiQueryType {
					identifier: query.identifier.to_string(),
					wire_method: query.wire_method.clone(),
					input_type: query.input_type.clone(),
					output_type: query.output_type.clone(),
					input_type_name: query.input_type_name.clone(),
					output_type_name: query.output_type_name.clone(),
					input_type_name_ts: query.input_type_name_ts.clone(),
					output_type_name_ts: query.output_type_name_ts.clone(),
				});
			}
		}
	}

	SpacedriveApiStructure {
		core_actions,
		library_actions,
		core_queries,
		library_queries,
	}
}

/// Represents the complete WingDrive API structure for code generation
pub struct SpacedriveApiStructure {
	pub core_actions: Vec<ApiOperationType>,
	pub library_actions: Vec<ApiOperationType>,
	pub core_queries: Vec<ApiQueryType>,
	pub library_queries: Vec<ApiQueryType>,
}

/// Represents a single API operation with actual type information
#[derive(Clone)]
pub struct ApiOperationType {
	pub identifier: String,
	pub wire_method: String,
	pub input_type: specta::datatype::DataType,
	pub output_type: specta::datatype::DataType,
	pub input_type_name: String,
	pub output_type_name: String,
	/// Same references formatted for TypeScript, where arrays need `T[]` syntax
	pub input_type_name_ts: String,
	pub output_type_name_ts: String,
}

/// Represents a single API query with actual type information
#[derive(Clone)]
pub struct ApiQueryType {
	pub identifier: String,
	pub wire_method: String,
	pub input_type: specta::datatype::DataType,
	pub output_type: specta::datatype::DataType,
	pub input_type_name: String,
	pub output_type_name: String,
	/// Same references formatted for TypeScript, where arrays need `T[]` syntax
	pub input_type_name_ts: String,
	pub output_type_name_ts: String,
}

/// Intermediate struct to hold API function metadata for Swift code generation
/// This is used to organize operations and queries into namespaces and methods
#[derive(Debug, Clone)]
pub struct ApiFunction {
	/// The namespace this function belongs to (e.g., "core", "libraries", "jobs")
	pub namespace: String,
	/// The method name within the namespace (e.g., "create", "list", "start")
	pub method_name: String,
	/// The full identifier (e.g., "libraries.create", "jobs.list")
	pub identifier: String,
	/// The wire method string (e.g., "action:libraries.create.input")
	pub wire_method: String,
	/// Whether this is an action (true) or query (false)
	pub is_action: bool,
	/// The scope (Core or Library)
	pub scope: String,
	/// Input type name for Swift generation
	pub input_type_name: String,
	/// Output type name for Swift generation
	pub output_type_name: String,
}

/// Extract API functions from the collected metadata
/// This organizes operations and queries into a flat list of functions with namespace information
pub fn extract_api_functions(
	operations: &[OperationMetadata],
	queries: &[QueryMetadata],
) -> Vec<ApiFunction> {
	let mut functions = Vec::new();

	// Process operations (actions)
	for op in operations {
		let namespace = extract_namespace(&op.identifier);
		let method_name = extract_method_name(&op.identifier);
		let scope = match op.scope {
			OperationScope::Core => "Core",
			OperationScope::Library => "Library",
		};

		functions.push(ApiFunction {
			namespace,
			method_name,
			identifier: op.identifier.to_string(),
			wire_method: op.wire_method.clone(),
			is_action: true,
			scope: scope.to_string(),
			input_type_name: op.input_type_name.clone(),
			output_type_name: op.output_type_name.clone(),
		});
	}

	// Process queries
	for query in queries {
		let namespace = extract_namespace(&query.identifier);
		let method_name = extract_method_name(&query.identifier);
		let scope = match query.scope {
			QueryScope::Core => "Core",
			QueryScope::Library => "Library",
		};

		functions.push(ApiFunction {
			namespace,
			method_name,
			identifier: query.identifier.to_string(),
			wire_method: query.wire_method.clone(),
			is_action: false,
			scope: scope.to_string(),
			input_type_name: query.input_type_name.clone(),
			output_type_name: query.output_type_name.clone(),
		});
	}

	functions
}

/// Extract namespace from identifier (e.g., "libraries.create" -> "libraries")
fn extract_namespace(identifier: &str) -> String {
	identifier.split('.').next().unwrap_or("core").to_string()
}

/// Extract method name from identifier (e.g., "libraries.create" -> "create")
fn extract_method_name(identifier: &str) -> String {
	identifier.split('.').skip(1).collect::<Vec<_>>().join("_")
}

/// Convert snake_case to PascalCase for Swift type names
fn to_pascal_case(s: &str) -> String {
	s.split(&['.', '_'][..])
		.map(|word| {
			let mut chars = word.chars();
			match chars.next() {
				None => String::new(),
				Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str(),
			}
		})
		.collect::<Vec<_>>()
		.join("")
}

/// Target language a generated type reference is rendered for.
///
/// The Rust type path is language-neutral; the only formatting difference is
/// arrays: Swift writes `[T]` while TypeScript reserves `[T]` for one-element
/// tuples and requires `T[]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TargetLang {
	Swift,
	TypeScript,
}

/// Extract the generator-facing type reference from a full Rust type path.
///
/// Only the outermost generic constructor drives formatting: `Option<T>`
/// unwraps to `T` because both clients treat missing values as nullable
/// output, and `Vec<T>` formats as an array per `lang`. Nested generics
/// recurse, so `Option<Vec<T>>` still produces the array form.
fn format_type_name(full_type_name: &str, lang: TargetLang) -> String {
	if full_type_name == "()" {
		return "Empty".to_string();
	}

	if let Some((ctor, args)) = split_outer_generic(full_type_name) {
		let ctor_name = ctor.rsplit("::").next().unwrap_or(ctor);
		return match ctor_name {
			"Option" => format_type_name(args, lang),
			"Vec" => {
				let inner = format_type_name(args, lang);
				match lang {
					TargetLang::Swift => format!("[{inner}]"),
					TargetLang::TypeScript => format!("{inner}[]"),
				}
			}
			// Other generics keep only the type name; paths like
			// `std::collections::HashMap` are not valid references in either
			// target language.
			_ => ctor_name.to_string(),
		};
	}

	full_type_name
		.rsplit("::")
		.next()
		.unwrap_or(full_type_name)
		.to_string()
}

/// Split `path::Ctor<args>` into `("path::Ctor", "args")`, or `None` when the
/// type has no outer generic. Matching on the constructor (text before the
/// first `<`) keeps nested occurrences of `Option`/`Vec` from hijacking the
/// parse.
fn split_outer_generic(full_type_name: &str) -> Option<(&str, &str)> {
	let open = full_type_name.find('<')?;
	if !full_type_name.ends_with('>') {
		return None;
	}
	Some((
		&full_type_name[..open],
		&full_type_name[open + 1..full_type_name.len() - 1],
	))
}

/// Type reference in Swift syntax, e.g. `[SourceInfo]`.
pub fn extract_type_name_swift(full_type_name: &str) -> String {
	format_type_name(full_type_name, TargetLang::Swift)
}

/// Type reference in TypeScript syntax, e.g. `SourceInfo[]`.
pub fn extract_type_name_typescript(full_type_name: &str) -> String {
	format_type_name(full_type_name, TargetLang::TypeScript)
}

/// Convert snake_case to camelCase for Swift method names
fn to_camel_case(s: &str) -> String {
	let mut words = s.split('_');
	let first_word = words.next().unwrap_or("");
	let rest_words: String = words
		.map(|word| {
			let mut chars = word.chars();
			match chars.next() {
				None => String::new(),
				Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str(),
			}
		})
		.collect();
	first_word.to_lowercase() + &rest_words
}

/// Generate Swift code for API namespace structs and their methods
pub fn generate_swift_api_code(functions: &[ApiFunction]) -> String {
	let mut swift_code = String::new();

	// Add import statement for Foundation (needed for async/await)
	swift_code.push_str("import Foundation\n\n");

	// Group functions by namespace
	let mut namespaces: std::collections::HashMap<String, Vec<&ApiFunction>> =
		std::collections::HashMap::new();
	for func in functions {
		namespaces
			.entry(func.namespace.clone())
			.or_default()
			.push(func);
	}

	// Generate code for each namespace
	for (namespace, funcs) in namespaces {
		let namespace_struct_name = format!("{}API", to_pascal_case(&namespace));

		swift_code.push_str(&format!("/// {} operations\n", to_pascal_case(&namespace)));
		swift_code.push_str(&format!("public struct {} {{\n", namespace_struct_name));
		swift_code.push_str("    private let client: SpacedriveClient\n");
		swift_code.push_str("\n");
		swift_code.push_str("    init(client: SpacedriveClient) {\n");
		swift_code.push_str("        self.client = client\n");
		swift_code.push_str("    }\n");
		swift_code.push_str("\n");

		// Generate methods for each function in this namespace
		for func in funcs {
			swift_code.push_str(&generate_swift_method(func));
			swift_code.push_str("\n");
		}

		swift_code.push_str("}\n\n");
	}

	swift_code
}

/// Generate Swift method code for a single API function
fn generate_swift_method(func: &ApiFunction) -> String {
	let method_name = to_camel_case(&func.method_name);
	let input_type = &func.input_type_name;
	let output_type = &func.output_type_name;
	let wire_method = &func.wire_method;

	// Determine if this is an action or query for documentation
	let operation_type = if func.is_action { "action" } else { "query" };

	let mut method_code = String::new();

	// Add documentation comment
	method_code.push_str(&format!(
		"    /// Execute {}: {}\n",
		operation_type, func.identifier
	));

	// Generate method signature
	if input_type == "EmptyInput" {
		// For operations with no input, use Empty struct
		method_code.push_str(&format!(
			"    public func {}() async throws -> {} {{\n",
			method_name, output_type
		));
		method_code.push_str("        let input = Empty()\n");
	} else {
		// For operations with input, take the input as parameter
		method_code.push_str(&format!(
			"    public func {}(_ input: {}) async throws -> {} {{\n",
			method_name, input_type, output_type
		));
	}

	// Generate method body
	method_code.push_str(&format!("        return try await client.execute(\n"));
	method_code.push_str("            input,\n");
	method_code.push_str(&format!("            method: \"{}\",\n", wire_method));
	method_code.push_str(&format!("            responseType: {}.self\n", output_type));
	method_code.push_str("        )\n");
	method_code.push_str("    }\n");

	method_code
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_type_name_swift_arrays_use_bracket_syntax() {
		assert_eq!(
			extract_type_name_swift("alloc::vec::Vec<core::sources::SourceInfo>"),
			"[SourceInfo]"
		);
		assert_eq!(
			extract_type_name_swift("core::sources::list::query::ListSourcesQuery"),
			"ListSourcesQuery"
		);
	}

	#[test]
	fn test_type_name_typescript_arrays_use_suffix_syntax() {
		assert_eq!(
			extract_type_name_typescript("alloc::vec::Vec<core::sources::SourceInfo>"),
			"SourceInfo[]"
		);
		// `[T]` is a one-element tuple in TypeScript, never an array.
		assert!(!extract_type_name_typescript("alloc::vec::Vec<u8>").starts_with('['));
	}

	#[test]
	fn test_type_name_nested_generics() {
		assert_eq!(
			extract_type_name_typescript(
				"alloc::vec::Vec<core::option::Option<alloc::string::String>>"
			),
			"String[]"
		);
		assert_eq!(
			extract_type_name_typescript(
				"core::option::Option<alloc::vec::Vec<core::domain::Device>>"
			),
			"Device[]"
		);
		assert_eq!(
			extract_type_name_swift("core::option::Option<alloc::vec::Vec<core::domain::Device>>"),
			"[Device]"
		);
		// The constructor decision must come from the outermost generic only;
		// nested `Option` inside `Vec` used to corrupt the parse.
		assert_eq!(
			extract_type_name_typescript("alloc::vec::Vec<core::option::Option<u8>>"),
			"u8[]"
		);
	}

	#[test]
	fn test_type_name_unit_and_generics() {
		assert_eq!(extract_type_name_typescript("()"), "Empty");
		assert_eq!(extract_type_name_swift("()"), "Empty");
		assert_eq!(
			extract_type_name_typescript("std::collections::HashMap<alloc::string::String, u8>"),
			"HashMap"
		);
	}

	#[test]
	fn test_type_extraction_system() {
		let (operations, queries, collection) = generate_spacedrive_api();

		println!(
			"Discovered {} operations and {} queries",
			operations.len(),
			queries.len()
		);
		println!("Type collection has {} types", collection.len());

		// Should have some operations if the system is working
		if !operations.is_empty() {
			println!("Type extraction system is working!");

			// Show some examples with scope information
			for op in operations.iter().take(3) {
				println!(
					"   Operation: {} -> wire: {} -> scope: {:?}",
					op.identifier, op.wire_method, op.scope
				);
			}
		}

		if !queries.is_empty() {
			for query in queries.iter().take(3) {
				println!(
					"   Query: {} -> wire: {} -> scope: {:?}",
					query.identifier, query.wire_method, query.scope
				);
			}
		}
	}

	#[test]
	fn test_api_functions_extraction() {
		let (operations, queries, _collection) = generate_spacedrive_api();
		let functions = extract_api_functions(&operations, &queries);

		println!("Extracted {} API functions", functions.len());

		// Group functions by namespace to show organization
		let mut namespaces: std::collections::HashMap<String, Vec<&ApiFunction>> =
			std::collections::HashMap::new();
		for func in &functions {
			namespaces
				.entry(func.namespace.clone())
				.or_default()
				.push(func);
		}

		for (namespace, funcs) in namespaces {
			println!("Namespace '{}': {} functions", namespace, funcs.len());
			for func in funcs.iter().take(3) {
				println!(
					"   {}: {} -> {} ({})",
					func.method_name,
					func.input_type_name,
					func.output_type_name,
					if func.is_action { "action" } else { "query" }
				);
			}
		}

		// Verify some basic properties
		assert!(
			!functions.is_empty(),
			"Should have extracted some API functions"
		);

		// Check that namespaces are properly extracted
		let has_libraries = functions.iter().any(|f| f.namespace == "libraries");
		let has_jobs = functions.iter().any(|f| f.namespace == "jobs");
		println!("Found libraries namespace: {}", has_libraries);
		println!("Found jobs namespace: {}", has_jobs);
	}

	#[test]
	fn test_swift_code_generation() {
		let (operations, queries, _collection) = generate_spacedrive_api();
		let functions = extract_api_functions(&operations, &queries);
		let swift_code = generate_swift_api_code(&functions);

		println!("Generated Swift code (first 1000 chars):");
		println!("{}", &swift_code[..swift_code.len().min(1000)]);

		// Verify basic structure
		assert!(swift_code.contains("public struct LibrariesAPI"));
		assert!(swift_code.contains("public struct JobsAPI"));
		assert!(swift_code.contains("public struct NetworkAPI"));

		// Verify method generation
		assert!(swift_code.contains("public func create("));
		assert!(swift_code.contains("public func list("));
		assert!(swift_code.contains("public func start("));

		// Verify method calls to client.execute
		assert!(swift_code.contains("client.execute("));
		assert!(swift_code.contains("responseType:"));

		// Verify wire method strings are included
		assert!(swift_code.contains("action:libraries.create.input"));
		assert!(swift_code.contains("query:jobs.list"));

		println!("Swift code generation test passed!");
	}
}
