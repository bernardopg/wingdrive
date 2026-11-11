use clap::Args;
use uuid::Uuid;

use wing_core::ops::tags::{
	apply::input::{ApplyTagsInput, TagTargets},
	create::input::CreateTagInput,
	search::input::SearchTagsInput,
};

#[derive(Args, Debug)]
pub struct TagCreateArgs {
	/// Canonical name for the tag
	pub name: String,
	/// Optional namespace
	#[arg(long)]
	pub namespace: Option<String>,
}

impl From<TagCreateArgs> for CreateTagInput {
	fn from(args: TagCreateArgs) -> Self {
		let mut input = CreateTagInput::simple(args.name);
		input.namespace = args.namespace;
		input
	}
}

#[derive(Args, Debug)]
pub struct TagApplyArgs {
	/// Entry IDs to tag (space-separated)
	#[arg(required = true)]
	pub entries: Vec<i32>,
	/// Tag IDs to apply (space-separated UUIDs)
	#[arg(long, required = true)]
	pub tags: Vec<Uuid>,
}

impl From<TagApplyArgs> for ApplyTagsInput {
	fn from(args: TagApplyArgs) -> Self {
		ApplyTagsInput::user_tags_entry(args.entries, args.tags)
	}
}

#[derive(Args, Debug)]
pub struct TagSearchArgs {
	/// Query text
	pub query: String,
	/// Optional namespace
	#[arg(long)]
	pub namespace: Option<String>,
	/// Include archived tags
	#[arg(long)]
	pub include_archived: bool,
	/// Limit number of results
	#[arg(long)]
	pub limit: Option<usize>,
}

impl From<TagSearchArgs> for SearchTagsInput {
	fn from(args: TagSearchArgs) -> Self {
		SearchTagsInput {
			query: args.query,
			namespace: args.namespace,
			tag_type: None,
			include_archived: Some(args.include_archived),
			limit: args.limit.or(Some(50)),
			resolve_ambiguous: Some(false),
			context_tag_ids: None,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use clap::Parser;

	#[derive(Parser)]
	struct CreateCli {
		#[command(flatten)]
		args: TagCreateArgs,
	}
	#[derive(Parser)]
	struct ApplyCli {
		#[command(flatten)]
		args: TagApplyArgs,
	}
	#[derive(Parser)]
	struct SearchCli {
		#[command(flatten)]
		args: TagSearchArgs,
	}

	#[test]
	fn create_preserves_name_and_namespace() {
		let input: CreateTagInput =
			CreateCli::try_parse_from(["wing", "report", "--namespace", "work"])
				.unwrap()
				.args
				.into();
		assert_eq!(input.canonical_name, "report");
		assert_eq!(input.namespace.as_deref(), Some("work"));
	}

	#[test]
	fn apply_preserves_entries_and_tag_ids() {
		let id = "550e8400-e29b-41d4-a716-446655440000";
		let input: ApplyTagsInput = ApplyCli::try_parse_from(["wing", "1", "2", "--tags", id])
			.unwrap()
			.args
			.into();
		assert!(matches!(input.targets, TagTargets::Entry(entries) if entries == [1, 2]));
		assert_eq!(input.tag_ids, [Uuid::parse_str(id).unwrap()]);
		assert_eq!(input.confidence, Some(1.0));
	}

	#[test]
	fn apply_rejects_missing_targets_and_invalid_tag_ids() {
		assert!(ApplyCli::try_parse_from([
			"wing",
			"--tags",
			"550e8400-e29b-41d4-a716-446655440000"
		])
		.is_err());
		assert!(ApplyCli::try_parse_from(["wing", "1"]).is_err());
		assert!(ApplyCli::try_parse_from(["wing", "1", "--tags", "bad"]).is_err());
	}

	#[test]
	fn search_defaults_to_fifty_and_excludes_archived() {
		let input: SearchTagsInput = SearchCli::try_parse_from(["wing", "project"])
			.unwrap()
			.args
			.into();
		assert_eq!(input.query, "project");
		assert_eq!(input.limit, Some(50));
		assert_eq!(input.include_archived, Some(false));
		assert_eq!(input.resolve_ambiguous, Some(false));
		assert!(input.namespace.is_none());
	}

	#[test]
	fn search_preserves_namespace_limit_and_archived_flag() {
		let input: SearchTagsInput = SearchCli::try_parse_from([
			"wing",
			"project",
			"--namespace",
			"work",
			"--limit",
			"5",
			"--include-archived",
		])
		.unwrap()
		.args
		.into();
		assert_eq!(input.namespace.as_deref(), Some("work"));
		assert_eq!(input.limit, Some(5));
		assert_eq!(input.include_archived, Some(true));
	}
}

#[cfg(test)]
#[path = "args/matrix.rs"]
mod matrix;
