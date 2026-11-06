use uuid::Uuid;
use wing_core::domain::tag::{PrivacyLevel, Tag, TagType};
use wing_core::infra::action::LibraryAction;
use wing_core::ops::tags::create::action::CreateTagAction;
use wing_core::ops::tags::create::input::CreateTagInput;
use wing_core::ops::tags::TagValidator;

fn tag(name: &str) -> Tag {
	Tag::new(name.to_string(), Uuid::nil())
}

#[test]
fn tag_validation_checks_names_aliases_colors_and_weights() {
	let mut value = tag("Café");
	assert!(TagValidator::validate_semantic_tag(&value).is_ok());
	value.aliases = vec!["Work".into(), "work".into()];
	assert!(TagValidator::validate_semantic_tag(&value).is_err());
	value.aliases.pop();
	value.color = Some("#ff0080".into());
	assert!(TagValidator::validate_semantic_tag(&value).is_ok());
	value.color = Some("#gg0080".into());
	assert!(TagValidator::validate_semantic_tag(&value).is_err());
	value.color = None;
	for weight in [-1, 1001] {
		value.search_weight = weight;
		assert!(TagValidator::validate_semantic_tag(&value).is_err());
	}
}

#[test]
fn tag_type_and_privacy_rules_are_enforced_at_boundaries() {
	let mut value = tag("private");
	value.tag_type = TagType::Organizational;
	assert!(TagValidator::validate_semantic_tag(&value).is_err());
	value.is_organizational_anchor = true;
	assert!(TagValidator::validate_semantic_tag(&value).is_ok());
	value.tag_type = TagType::Privacy;
	assert!(TagValidator::validate_semantic_tag(&value).is_err());
	value.privacy_level = PrivacyLevel::Hidden;
	value.search_weight = 50;
	assert!(TagValidator::validate_semantic_tag(&value).is_ok());
	value.search_weight = 51;
	assert!(TagValidator::validate_semantic_tag(&value).is_err());
	value.privacy_level = PrivacyLevel::Archive;
	value.search_weight = 200;
	assert!(TagValidator::validate_semantic_tag(&value).is_ok());
	value.search_weight = 201;
	assert!(TagValidator::validate_semantic_tag(&value).is_err());
}

#[test]
fn name_conflicts_are_scoped_to_namespace_and_include_aliases() {
	let mut existing = tag("Work");
	existing.namespace = Some("office".into());
	existing.aliases.push("Job".into());
	let mut candidate = tag("job");
	candidate.namespace = Some("office".into());
	assert!(TagValidator::validate_no_name_conflicts(&candidate, &[existing.clone()]).is_err());
	candidate.namespace = Some("personal".into());
	assert!(TagValidator::validate_no_name_conflicts(&candidate, &[existing]).is_ok());
}

#[test]
fn tag_name_limit_counts_unicode_characters() {
	assert!(TagValidator::validate_tag_name(&"é".repeat(128)).is_ok());
	assert!(TagValidator::validate_tag_name(&"é".repeat(255)).is_ok());
	assert!(TagValidator::validate_tag_name(&"é".repeat(256)).is_err());
}

#[test]
fn create_tag_action_validates_unicode_lengths_and_hex_color() {
	let mut input = CreateTagInput::simple("é".repeat(255));
	input.namespace = Some("é".repeat(100));
	input.color = Some("#123aEF".into());
	assert!(CreateTagAction::from_input(input.clone()).is_ok());
	input.canonical_name.push('é');
	assert!(CreateTagAction::from_input(input.clone()).is_err());
	input.canonical_name = "valid".into();
	input.namespace.as_mut().unwrap().push('é');
	assert!(CreateTagAction::from_input(input.clone()).is_err());
	input.namespace = None;
	input.color = Some("#zzzzzz".into());
	assert!(CreateTagAction::from_input(input).is_err());
}
