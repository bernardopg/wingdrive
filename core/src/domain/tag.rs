//! Semantic Tag domain model
//!
//! Implementation of the advanced semantic tagging architecture described in the whitepaper.
//! This replaces the simple tag model with a sophisticated graph-based system that supports
//! polymorphic naming, contextual resolution, and compositional attributes.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use uuid::Uuid;

use super::resource::Identifiable;

/// A tag with advanced capabilities for contextual organization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
pub struct Tag {
	/// Unique identifier
	pub id: Uuid,

	/// Core identity
	pub canonical_name: String,
	pub display_name: Option<String>,

	/// Semantic variants for flexible access
	pub formal_name: Option<String>,
	pub abbreviation: Option<String>,
	pub aliases: Vec<String>,

	/// Context and categorization
	pub namespace: Option<String>,
	pub tag_type: TagType,

	/// Visual and behavioral properties
	pub color: Option<String>,
	pub icon: Option<String>,
	pub description: Option<String>,

	/// Advanced capabilities
	pub is_organizational_anchor: bool,
	pub privacy_level: PrivacyLevel,
	pub search_weight: i32,

	/// Compositional attributes
	pub attributes: HashMap<String, serde_json::Value>,
	pub composition_rules: Vec<CompositionRule>,

	/// Metadata
	pub created_at: DateTime<Utc>,
	pub updated_at: DateTime<Utc>,
	pub created_by_device: Uuid,
}

/// Types of semantic tags with different behaviors
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
pub enum TagType {
	/// Standard user-created tag
	Standard,
	/// Creates visual hierarchies in the interface
	Organizational,
	/// Controls search and display visibility
	Privacy,
	/// System-generated tag (AI, import, etc.)
	System,
}

impl TagType {
	pub fn as_str(&self) -> &'static str {
		match self {
			TagType::Standard => "standard",
			TagType::Organizational => "organizational",
			TagType::Privacy => "privacy",
			TagType::System => "system",
		}
	}

	pub fn from_str(s: &str) -> Option<Self> {
		match s {
			"standard" => Some(TagType::Standard),
			"organizational" => Some(TagType::Organizational),
			"privacy" => Some(TagType::Privacy),
			"system" => Some(TagType::System),
			_ => None,
		}
	}
}

/// Privacy levels for tag visibility control
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
pub enum PrivacyLevel {
	/// Standard visibility in all contexts
	Normal,
	/// Hidden from normal searches but accessible via direct query
	Archive,
	/// Completely hidden from standard UI
	Hidden,
}

impl PrivacyLevel {
	pub fn as_str(&self) -> &'static str {
		match self {
			PrivacyLevel::Normal => "normal",
			PrivacyLevel::Archive => "archive",
			PrivacyLevel::Hidden => "hidden",
		}
	}

	pub fn from_str(s: &str) -> Option<Self> {
		match s {
			"normal" => Some(PrivacyLevel::Normal),
			"archive" => Some(PrivacyLevel::Archive),
			"hidden" => Some(PrivacyLevel::Hidden),
			_ => None,
		}
	}
}

/// Relationship between two tags in the semantic graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
pub struct TagRelationship {
	pub related_tag_id: Uuid,
	pub relationship_type: RelationshipType,
	pub strength: f32,
	pub created_at: DateTime<Utc>,
}

/// Types of relationships between tags
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
pub enum RelationshipType {
	/// Hierarchical parent-child relationship
	ParentChild,
	/// Synonym or alias relationship
	Synonym,
	/// General semantic relatedness
	Related,
}

impl RelationshipType {
	pub fn as_str(&self) -> &'static str {
		match self {
			RelationshipType::ParentChild => "parent_child",
			RelationshipType::Synonym => "synonym",
			RelationshipType::Related => "related",
		}
	}

	pub fn from_str(s: &str) -> Option<Self> {
		match s {
			"parent_child" => Some(RelationshipType::ParentChild),
			"synonym" => Some(RelationshipType::Synonym),
			"related" => Some(RelationshipType::Related),
			_ => None,
		}
	}
}

/// Rules for composing attributes from multiple tags
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
pub struct CompositionRule {
	pub operator: CompositionOperator,
	pub operands: Vec<String>,
	pub result_attribute: String,
}

/// Operators for combining tag attributes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
pub enum CompositionOperator {
	/// All conditions must be true
	And,
	/// Any condition must be true
	Or,
	/// Must have this property
	With,
	/// Must not have this property
	Without,
}

/// Context-aware application of a tag to content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
pub struct TagApplication {
	pub tag_id: Uuid,
	/// Context when the tag was applied (e.g., "geography", "technology")
	pub applied_context: Option<String>,
	/// Which variant name was used when applying
	pub applied_variant: Option<String>,
	/// Confidence level (0.0-1.0, useful for AI-applied tags)
	pub confidence: f32,
	/// Source of the tag application
	pub source: TagSource,
	/// Attributes specific to this particular application
	pub instance_attributes: HashMap<String, serde_json::Value>,
	/// When this application was created
	pub created_at: DateTime<Utc>,
	/// Which device applied this tag
	pub device_uuid: Uuid,
}

/// Source of tag application
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
pub enum TagSource {
	/// Manually applied by user
	User,
	/// Applied by AI analysis
	AI,
	/// Imported from external source
	Import,
	/// Synchronized from another device
	Sync,
}

/// Result of merging tag applications during sync
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TagMergeResult {
	pub merged_applications: Vec<TagApplication>,
	pub conflicts: Vec<TagConflict>,
	pub merge_summary: String,
}

/// Conflict that occurred during tag merging
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TagConflict {
	pub tag_id: Uuid,
	pub conflict_type: ConflictType,
	pub local_value: serde_json::Value,
	pub remote_value: serde_json::Value,
	pub resolution: ConflictResolution,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum ConflictType {
	AttributeValue,
	Context,
	Confidence,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum ConflictResolution {
	UseLocal,
	UseRemote,
	Merge,
	RequiresUserInput,
}

/// Pattern discovered through usage analysis
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct OrganizationalPattern {
	pub pattern_type: PatternType,
	pub tags_involved: Vec<Uuid>,
	pub confidence: f32,
	pub suggestion: String,
	pub discovered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum PatternType {
	FrequentCoOccurrence,
	HierarchicalRelationship,
	SemanticSimilarity,
	ContextualGrouping,
}

impl Tag {
	/// Create a new semantic tag with default values
	pub fn new(canonical_name: String, created_by_device: Uuid) -> Self {
		let now = Utc::now();

		Self {
			id: Uuid::new_v4(),
			canonical_name: canonical_name.clone(),
			display_name: None,
			formal_name: None,
			abbreviation: None,
			aliases: Vec::new(),
			namespace: None,
			tag_type: TagType::Standard,
			color: None,
			icon: None,
			description: None,
			is_organizational_anchor: false,
			privacy_level: PrivacyLevel::Normal,
			search_weight: 100,
			attributes: HashMap::new(),
			composition_rules: Vec::new(),
			created_at: now,
			updated_at: now,
			created_by_device,
		}
	}

	/// Get the best display name for this tag in the given context
	pub fn get_display_name(&self, context: Option<&str>) -> &str {
		// If we have a context-specific display name, use it
		if let Some(display) = &self.display_name {
			return display;
		}

		// Otherwise use canonical name
		&self.canonical_name
	}

	/// Get all possible names this tag can be accessed by
	pub fn get_all_names(&self) -> Vec<&str> {
		let mut names = vec![self.canonical_name.as_str()];

		if let Some(formal) = &self.formal_name {
			names.push(formal);
		}

		if let Some(abbrev) = &self.abbreviation {
			names.push(abbrev);
		}

		for alias in &self.aliases {
			names.push(alias);
		}

		names
	}

	/// Check if this tag matches the given name in any variant
	pub fn matches_name(&self, name: &str) -> bool {
		self.get_all_names()
			.iter()
			.any(|&n| n.eq_ignore_ascii_case(name))
	}

	/// Add an alias to this tag
	pub fn add_alias(&mut self, alias: String) {
		if !self.aliases.contains(&alias) {
			self.aliases.push(alias);
			self.updated_at = Utc::now();
		}
	}

	/// Set an attribute value
	pub fn set_attribute<T: Serialize>(
		&mut self,
		key: String,
		value: T,
	) -> Result<(), serde_json::Error> {
		let json_value = serde_json::to_value(value)?;
		self.attributes.insert(key, json_value);
		self.updated_at = Utc::now();
		Ok(())
	}

	/// Get an attribute value
	pub fn get_attribute<T: for<'de> Deserialize<'de>>(
		&self,
		key: &str,
	) -> Result<Option<T>, serde_json::Error> {
		match self.attributes.get(key) {
			Some(value) => Ok(Some(serde_json::from_value(value.clone())?)),
			None => Ok(None),
		}
	}

	/// Check if this tag should be hidden from normal search results
	pub fn is_searchable(&self) -> bool {
		match self.privacy_level {
			PrivacyLevel::Normal => true,
			PrivacyLevel::Archive | PrivacyLevel::Hidden => false,
		}
	}

	/// Get the fully qualified name including namespace
	pub fn get_qualified_name(&self) -> String {
		match &self.namespace {
			Some(ns) => format!("{}::{}", ns, self.canonical_name),
			None => self.canonical_name.clone(),
		}
	}
}

impl TagApplication {
	/// Create a new tag application
	pub fn new(tag_id: Uuid, source: TagSource, device_uuid: Uuid) -> Self {
		Self {
			tag_id,
			applied_context: None,
			applied_variant: None,
			confidence: 1.0,
			source,
			instance_attributes: HashMap::new(),
			created_at: Utc::now(),
			device_uuid,
		}
	}

	/// Create a user-applied tag application
	pub fn user_applied(tag_id: Uuid, device_uuid: Uuid) -> Self {
		Self::new(tag_id, TagSource::User, device_uuid)
	}

	/// Create an AI-applied tag application with confidence
	pub fn ai_applied(tag_id: Uuid, confidence: f32, device_uuid: Uuid) -> Self {
		let mut app = Self::new(tag_id, TagSource::AI, device_uuid);
		app.confidence = confidence;
		app
	}

	/// Set an instance-specific attribute
	pub fn set_instance_attribute<T: Serialize>(
		&mut self,
		key: String,
		value: T,
	) -> Result<(), serde_json::Error> {
		let json_value = serde_json::to_value(value)?;
		self.instance_attributes.insert(key, json_value);
		Ok(())
	}

	/// Check if this application has high confidence
	pub fn is_high_confidence(&self) -> bool {
		self.confidence >= 0.8
	}
}

/// Error types for semantic tag operations
#[derive(Debug, thiserror::Error)]
pub enum TagError {
	#[error("Tag not found")]
	TagNotFound,

	#[error("Invalid tag relationship: {0}")]
	InvalidRelationship(String),

	#[error("Circular reference detected")]
	CircularReference,

	#[error("Conflicting tag names in namespace: {0}")]
	NameConflict(String),

	#[error("Invalid composition rule: {0}")]
	InvalidCompositionRule(String),

	#[error("Serialization error: {0}")]
	SerializationError(#[from] serde_json::Error),

	#[error("Database error: {0}")]
	DatabaseError(String),
}

// Implement Identifiable for normalized cache support
impl Identifiable for Tag {
	fn id(&self) -> Uuid {
		self.id
	}

	fn resource_type() -> &'static str {
		"tag"
	}

	fn sync_dependencies() -> &'static [&'static str] {
		&[] // Tags are a simple resource backed by the tags table
	}

	async fn from_ids(
		db: &sea_orm::DatabaseConnection,
		ids: &[Uuid],
	) -> crate::common::errors::Result<Vec<Self>>
	where
		Self: Sized,
	{
		use crate::infra::db::entities::tag;
		use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

		let tag_models = tag::Entity::find()
			.filter(tag::Column::Uuid.is_in(ids.to_vec()))
			.all(db)
			.await?;

		Ok(tag_models.into_iter().map(Tag::from_db_model).collect())
	}
}

impl Tag {
	/// Build Tag from database model
	pub fn from_db_model(model: crate::infra::db::entities::tag::Model) -> Self {
		let aliases: Vec<String> = model
			.aliases
			.as_ref()
			.and_then(|json| serde_json::from_value(json.clone()).ok())
			.unwrap_or_default();

		let tag_type = TagType::from_str(&model.tag_type).unwrap_or(TagType::Standard);

		let privacy_level =
			PrivacyLevel::from_str(&model.privacy_level).unwrap_or(PrivacyLevel::Normal);

		let attributes: HashMap<String, serde_json::Value> = model
			.attributes
			.as_ref()
			.and_then(|json| serde_json::from_value(json.clone()).ok())
			.unwrap_or_default();

		let composition_rules: Vec<CompositionRule> = model
			.composition_rules
			.as_ref()
			.and_then(|json| serde_json::from_value(json.clone()).ok())
			.unwrap_or_default();

		Self {
			id: model.uuid,
			canonical_name: model.canonical_name,
			display_name: model.display_name,
			formal_name: model.formal_name,
			abbreviation: model.abbreviation,
			aliases,
			namespace: model.namespace,
			tag_type,
			color: model.color,
			icon: model.icon,
			description: model.description,
			is_organizational_anchor: model.is_organizational_anchor,
			privacy_level,
			search_weight: model.search_weight,
			attributes,
			composition_rules,
			created_at: model.created_at.into(),
			updated_at: model.updated_at.into(),
			created_by_device: model.created_by_device.unwrap_or_else(Uuid::nil),
		}
	}
}

// Register Tag as a simple resource
crate::register_resource!(Tag);

#[cfg(test)]
mod tests {
	use super::*;
	use chrono::Utc;
	use std::collections::HashMap;

	fn device_id() -> Uuid {
		Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
	}

	fn make_db_model() -> crate::infra::db::entities::tag::Model {
		let now = Utc::now();
		crate::infra::db::entities::tag::Model {
			id: 1,
			uuid: Uuid::parse_str("00000000-0000-0000-0000-000000000099").unwrap(),
			canonical_name: "Rust".to_string(),
			display_name: Some("Rust Lang".to_string()),
			formal_name: Some("Rust Programming Language".to_string()),
			abbreviation: Some("rs".to_string()),
			aliases: Some(serde_json::json!(["rustlang", "rust-lang"])),
			namespace: Some("tech".to_string()),
			tag_type: "organizational".to_string(),
			color: Some("#ff0000".to_string()),
			icon: Some("rust".to_string()),
			description: Some("Systems language".to_string()),
			is_organizational_anchor: true,
			privacy_level: "archive".to_string(),
			search_weight: 42,
			attributes: Some(serde_json::json!({"kind": "language", "year": 2015})),
			composition_rules: Some(serde_json::json!([{
				"operator": "And",
				"operands": ["systems", "fast"],
				"result_attribute": "tier"
			}])),
			created_at: now,
			updated_at: now,
			created_by_device: Some(device_id()),
		}
	}

	#[test]
	fn tag_type_roundtrip() {
		for (variant, name) in [
			(TagType::Standard, "standard"),
			(TagType::Organizational, "organizational"),
			(TagType::Privacy, "privacy"),
			(TagType::System, "system"),
		] {
			assert_eq!(variant.as_str(), name);
			assert_eq!(TagType::from_str(name), Some(variant));
		}
		assert_eq!(TagType::from_str("unknown"), None);
	}

	#[test]
	fn privacy_level_roundtrip() {
		for (variant, name) in [
			(PrivacyLevel::Normal, "normal"),
			(PrivacyLevel::Archive, "archive"),
			(PrivacyLevel::Hidden, "hidden"),
		] {
			assert_eq!(variant.as_str(), name);
			assert_eq!(PrivacyLevel::from_str(name), Some(variant));
		}
		assert_eq!(PrivacyLevel::from_str("secret"), None);
	}

	#[test]
	fn relationship_type_roundtrip() {
		for (variant, name) in [
			(RelationshipType::ParentChild, "parent_child"),
			(RelationshipType::Synonym, "synonym"),
			(RelationshipType::Related, "related"),
		] {
			assert_eq!(variant.as_str(), name);
			assert_eq!(RelationshipType::from_str(name), Some(variant));
		}
		assert_eq!(RelationshipType::from_str("other"), None);
	}

	#[test]
	fn new_tag_has_defaults() {
		let dev = device_id();
		let tag = Tag::new("MyTag".to_string(), dev);

		assert_eq!(tag.canonical_name, "MyTag");
		assert_eq!(tag.display_name, None);
		assert_eq!(tag.formal_name, None);
		assert_eq!(tag.abbreviation, None);
		assert!(tag.aliases.is_empty());
		assert_eq!(tag.namespace, None);
		assert_eq!(tag.tag_type, TagType::Standard);
		assert_eq!(tag.color, None);
		assert_eq!(tag.icon, None);
		assert_eq!(tag.description, None);
		assert!(!tag.is_organizational_anchor);
		assert_eq!(tag.privacy_level, PrivacyLevel::Normal);
		assert_eq!(tag.search_weight, 100);
		assert!(tag.attributes.is_empty());
		assert!(tag.composition_rules.is_empty());
		assert_eq!(tag.created_by_device, dev);
		assert_eq!(tag.created_at, tag.updated_at);
	}

	#[test]
	fn display_name_prefers_explicit_display() {
		let mut tag = Tag::new("canon".to_string(), device_id());
		assert_eq!(tag.get_display_name(None), "canon");

		tag.display_name = Some("Display".to_string());
		assert_eq!(tag.get_display_name(None), "Display");
		assert_eq!(tag.get_display_name(Some("any")), "Display");
	}

	#[test]
	fn get_all_names_and_matches_case_insensitive() {
		let mut tag = Tag::new("Canon".to_string(), device_id());
		tag.formal_name = Some("Formal Name".to_string());
		tag.abbreviation = Some("CN".to_string());
		tag.aliases = vec!["alias1".to_string(), "alias2".to_string()];

		let names = tag.get_all_names();
		assert_eq!(
			names,
			vec!["Canon", "Formal Name", "CN", "alias1", "alias2"]
		);

		assert!(tag.matches_name("canon"));
		assert!(tag.matches_name("FORMAL NAME"));
		assert!(tag.matches_name("cn"));
		assert!(tag.matches_name("ALIAS2"));
		assert!(!tag.matches_name("missing"));
	}

	#[test]
	fn add_alias_deduplicates_and_updates_timestamp() {
		let mut tag = Tag::new("t".to_string(), device_id());
		let before = tag.updated_at;

		tag.add_alias("a".to_string());
		assert_eq!(tag.aliases, vec!["a"]);
		assert!(tag.updated_at >= before);

		let after_first = tag.updated_at;
		tag.add_alias("a".to_string());
		assert_eq!(tag.aliases, vec!["a"]);
		assert_eq!(tag.updated_at, after_first);
	}

	#[test]
	fn set_and_get_attribute() {
		let mut tag = Tag::new("t".to_string(), device_id());
		tag.set_attribute("count".to_string(), 3_i32).unwrap();
		tag.set_attribute("name".to_string(), "value").unwrap();

		let count: Option<i32> = tag.get_attribute("count").unwrap();
		let name: Option<String> = tag.get_attribute("name").unwrap();
		let missing: Option<i32> = tag.get_attribute("missing").unwrap();

		assert_eq!(count, Some(3));
		assert_eq!(name, Some("value".to_string()));
		assert_eq!(missing, None);
	}

	#[test]
	fn is_searchable_reflects_privacy_level() {
		let mut tag = Tag::new("t".to_string(), device_id());
		assert!(tag.is_searchable());

		tag.privacy_level = PrivacyLevel::Archive;
		assert!(!tag.is_searchable());

		tag.privacy_level = PrivacyLevel::Hidden;
		assert!(!tag.is_searchable());
	}

	#[test]
	fn get_qualified_name_includes_namespace() {
		let mut tag = Tag::new("name".to_string(), device_id());
		assert_eq!(tag.get_qualified_name(), "name");

		tag.namespace = Some("ns".to_string());
		assert_eq!(tag.get_qualified_name(), "ns::name");
	}

	#[test]
	fn tag_application_constructors() {
		let tag_id = Uuid::new_v4();
		let dev = device_id();

		let user = TagApplication::user_applied(tag_id, dev);
		assert_eq!(user.tag_id, tag_id);
		assert_eq!(user.source, TagSource::User);
		assert_eq!(user.confidence, 1.0);
		assert_eq!(user.device_uuid, dev);
		assert!(user.applied_context.is_none());
		assert!(user.applied_variant.is_none());
		assert!(user.instance_attributes.is_empty());
		assert!(user.is_high_confidence());

		let ai = TagApplication::ai_applied(tag_id, 0.5, dev);
		assert_eq!(ai.source, TagSource::AI);
		assert_eq!(ai.confidence, 0.5);
		assert!(!ai.is_high_confidence());

		let ai_high = TagApplication::ai_applied(tag_id, 0.8, dev);
		assert!(ai_high.is_high_confidence());
	}

	#[test]
	fn tag_application_instance_attributes() {
		let mut app = TagApplication::user_applied(Uuid::new_v4(), device_id());
		app.set_instance_attribute("k".to_string(), "v").unwrap();
		assert_eq!(
			app.instance_attributes.get("k"),
			Some(&serde_json::json!("v"))
		);
	}

	#[test]
	fn from_db_model_maps_fields_and_defaults() {
		let model = make_db_model();
		let tag = Tag::from_db_model(model.clone());

		assert_eq!(tag.id, model.uuid);
		assert_eq!(tag.canonical_name, "Rust");
		assert_eq!(tag.display_name.as_deref(), Some("Rust Lang"));
		assert_eq!(
			tag.formal_name.as_deref(),
			Some("Rust Programming Language")
		);
		assert_eq!(tag.abbreviation.as_deref(), Some("rs"));
		assert_eq!(tag.aliases, vec!["rustlang", "rust-lang"]);
		assert_eq!(tag.namespace.as_deref(), Some("tech"));
		assert_eq!(tag.tag_type, TagType::Organizational);
		assert_eq!(tag.color.as_deref(), Some("#ff0000"));
		assert_eq!(tag.icon.as_deref(), Some("rust"));
		assert_eq!(tag.description.as_deref(), Some("Systems language"));
		assert!(tag.is_organizational_anchor);
		assert_eq!(tag.privacy_level, PrivacyLevel::Archive);
		assert_eq!(tag.search_weight, 42);
		assert_eq!(
			tag.attributes.get("kind"),
			Some(&serde_json::json!("language"))
		);
		assert_eq!(tag.attributes.get("year"), Some(&serde_json::json!(2015)));
		assert_eq!(tag.composition_rules.len(), 1);
		assert_eq!(tag.composition_rules[0].result_attribute, "tier");
		assert_eq!(tag.created_by_device, device_id());
	}

	#[test]
	fn from_db_model_handles_invalid_or_missing_json() {
		let now = Utc::now();
		let model = crate::infra::db::entities::tag::Model {
			id: 2,
			uuid: Uuid::new_v4(),
			canonical_name: "x".to_string(),
			display_name: None,
			formal_name: None,
			abbreviation: None,
			aliases: Some(serde_json::json!("not-an-array")),
			namespace: None,
			tag_type: "bogus".to_string(),
			color: None,
			icon: None,
			description: None,
			is_organizational_anchor: false,
			privacy_level: "bogus".to_string(),
			search_weight: 1,
			attributes: Some(serde_json::json!("nope")),
			composition_rules: Some(serde_json::json!(123)),
			created_at: now,
			updated_at: now,
			created_by_device: None,
		};

		let tag = Tag::from_db_model(model);
		assert!(tag.aliases.is_empty());
		assert_eq!(tag.tag_type, TagType::Standard);
		assert_eq!(tag.privacy_level, PrivacyLevel::Normal);
		assert!(tag.attributes.is_empty());
		assert!(tag.composition_rules.is_empty());
		assert_eq!(tag.created_by_device, Uuid::nil());
	}

	#[test]
	fn tag_error_display() {
		assert_eq!(TagError::TagNotFound.to_string(), "Tag not found");
		assert_eq!(
			TagError::InvalidRelationship("bad".to_string()).to_string(),
			"Invalid tag relationship: bad"
		);
		assert_eq!(
			TagError::CircularReference.to_string(),
			"Circular reference detected"
		);
		assert_eq!(
			TagError::NameConflict("ns".to_string()).to_string(),
			"Conflicting tag names in namespace: ns"
		);
		assert_eq!(
			TagError::InvalidCompositionRule("r".to_string()).to_string(),
			"Invalid composition rule: r"
		);
	}
}
