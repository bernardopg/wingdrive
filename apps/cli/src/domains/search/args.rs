use chrono::{DateTime, Utc};
use clap::Args;
use uuid::Uuid;

use wing_core::domain::{addressing::WingPath, ContentKind};
use wing_core::ops::search::input::{
	DateField, DateRangeFilter, FileSearchInput, PaginationOptions, SearchFilters, SearchMode,
	SearchScope, SizeRangeFilter, SortDirection, SortField, SortOptions, TagFilter,
};

#[derive(Args, Debug)]
pub struct FileSearchArgs {
	/// Search query
	pub query: String,

	/// Search mode
	#[arg(long, value_enum, default_value = "normal")]
	pub mode: SearchModeArg,

	/// WingPath to narrow search to a specific directory
	#[arg(long, value_parser = WingPath::from_uri, conflicts_with = "location")]
	pub wing_path: Option<WingPath>,

	/// File type filter (can be specified multiple times)
	#[arg(long)]
	pub file_type: Option<Vec<String>>,

	/// Tag filter (can be specified multiple times)
	#[arg(long)]
	pub tags: Option<Vec<Uuid>>,

	/// Exclude tags (can be specified multiple times)
	#[arg(long)]
	pub exclude_tags: Option<Vec<Uuid>>,

	/// Location filter
	#[arg(long)]
	pub location: Option<Uuid>,

	/// Date field for filtering
	#[arg(long, value_enum, default_value = "modified")]
	pub date_field: DateFieldArg,

	/// Start date for filtering (ISO format)
	#[arg(long)]
	pub date_start: Option<DateTime<Utc>>,

	/// End date for filtering (ISO format)
	#[arg(long)]
	pub date_end: Option<DateTime<Utc>>,

	/// Minimum file size in bytes
	#[arg(long)]
	pub min_size: Option<u64>,

	/// Maximum file size in bytes
	#[arg(long)]
	pub max_size: Option<u64>,

	/// Content type filter
	#[arg(long, value_enum)]
	pub content_type: Option<Vec<ContentTypeArg>>,

	/// Sort field
	#[arg(long, value_enum, default_value = "relevance")]
	pub sort_field: SortFieldArg,

	/// Sort direction
	#[arg(long, value_enum, default_value = "desc")]
	pub sort_direction: SortDirectionArg,

	/// Limit number of results
	#[arg(long, default_value = "50")]
	pub limit: u32,

	/// Offset for pagination
	#[arg(long, default_value = "0")]
	pub offset: u32,

	/// Include hidden files
	#[arg(long)]
	pub include_hidden: bool,

	/// Include archived files
	#[arg(long)]
	pub include_archived: bool,
}

#[derive(clap::ValueEnum, Debug, Clone)]
pub enum SearchModeArg {
	Fast,
	Normal,
	Full,
}

#[derive(clap::ValueEnum, Debug, Clone)]
pub enum DateFieldArg {
	Created,
	Modified,
	Accessed,
}

#[derive(clap::ValueEnum, Debug, Clone)]
pub enum ContentTypeArg {
	Unknown,
	Image,
	Video,
	Audio,
	Document,
	Archive,
	Code,
	Text,
	Database,
	Book,
	Font,
	Mesh,
	Config,
	Encrypted,
	Key,
	Executable,
	Binary,
}

#[derive(clap::ValueEnum, Debug, Clone)]
pub enum SortFieldArg {
	Relevance,
	Name,
	Size,
	Modified,
	Created,
}

#[derive(clap::ValueEnum, Debug, Clone)]
pub enum SortDirectionArg {
	Asc,
	Desc,
}

impl From<FileSearchArgs> for FileSearchInput {
	fn from(args: FileSearchArgs) -> Self {
		let mode = match args.mode {
			SearchModeArg::Fast => SearchMode::Fast,
			SearchModeArg::Normal => SearchMode::Normal,
			SearchModeArg::Full => SearchMode::Full,
		};

		let scope = if let Some(path) = args.wing_path {
			SearchScope::Path { path }
		} else if let Some(location_id) = args.location {
			SearchScope::Location { location_id }
		} else {
			SearchScope::Library
		};

		let filters = SearchFilters {
			file_types: args.file_type,
			tags: if args.tags.is_some() || args.exclude_tags.is_some() {
				Some(TagFilter {
					include: args.tags.unwrap_or_default(),
					exclude: args.exclude_tags.unwrap_or_default(),
				})
			} else {
				None
			},
			date_range: if args.date_start.is_some() || args.date_end.is_some() {
				Some(DateRangeFilter {
					field: match args.date_field {
						DateFieldArg::Created => DateField::CreatedAt,
						DateFieldArg::Modified => DateField::ModifiedAt,
						DateFieldArg::Accessed => DateField::AccessedAt,
					},
					start: args.date_start,
					end: args.date_end,
				})
			} else {
				None
			},
			size_range: if args.min_size.is_some() || args.max_size.is_some() {
				Some(SizeRangeFilter {
					min: args.min_size,
					max: args.max_size,
				})
			} else {
				None
			},
			locations: None, // Not used in CLI for now
			content_types: args.content_type.map(|types| {
				types
					.into_iter()
					.map(|ct| match ct {
						ContentTypeArg::Unknown => ContentKind::Unknown,
						ContentTypeArg::Image => ContentKind::Image,
						ContentTypeArg::Video => ContentKind::Video,
						ContentTypeArg::Audio => ContentKind::Audio,
						ContentTypeArg::Document => ContentKind::Document,
						ContentTypeArg::Archive => ContentKind::Archive,
						ContentTypeArg::Code => ContentKind::Code,
						ContentTypeArg::Text => ContentKind::Text,
						ContentTypeArg::Database => ContentKind::Database,
						ContentTypeArg::Book => ContentKind::Book,
						ContentTypeArg::Font => ContentKind::Font,
						ContentTypeArg::Mesh => ContentKind::Mesh,
						ContentTypeArg::Config => ContentKind::Config,
						ContentTypeArg::Encrypted => ContentKind::Encrypted,
						ContentTypeArg::Key => ContentKind::Key,
						ContentTypeArg::Executable => ContentKind::Executable,
						ContentTypeArg::Binary => ContentKind::Binary,
					})
					.collect()
			}),
			favorite: None,
			include_hidden: Some(args.include_hidden),
			include_archived: Some(args.include_archived),
			at_risk: None,
			on_volumes: None,
			not_on_volumes: None,
			min_volume_count: None,
			max_volume_count: None,
		};

		let sort = SortOptions {
			field: match args.sort_field {
				SortFieldArg::Relevance => SortField::Relevance,
				SortFieldArg::Name => SortField::Name,
				SortFieldArg::Size => SortField::Size,
				SortFieldArg::Modified => SortField::ModifiedAt,
				SortFieldArg::Created => SortField::CreatedAt,
			},
			direction: match args.sort_direction {
				SortDirectionArg::Asc => SortDirection::Asc,
				SortDirectionArg::Desc => SortDirection::Desc,
			},
		};

		let pagination = PaginationOptions {
			limit: args.limit,
			offset: args.offset,
		};

		Self {
			query: args.query,
			scope,
			mode,
			filters,
			sort,
			pagination,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use clap::Parser;

	#[derive(Parser)]
	struct SearchCli {
		#[command(flatten)]
		search: FileSearchArgs,
	}

	#[test]
	fn default_search_keeps_library_scope_and_pagination() {
		let input: FileSearchInput = SearchCli::try_parse_from(["wing", "report"])
			.unwrap()
			.search
			.into();
		assert_eq!(input.query, "report");
		assert!(matches!(input.scope, SearchScope::Library));
		assert!(matches!(input.mode, SearchMode::Normal));
		assert_eq!(input.pagination.limit, 50);
		assert_eq!(input.pagination.offset, 0);
		assert!(input.filters.tags.is_none());
		assert!(input.filters.date_range.is_none());
		assert!(input.filters.size_range.is_none());
	}

	#[test]
	fn search_filters_and_pagination_reach_the_query() {
		let tag = "550e8400-e29b-41d4-a716-446655440000";
		let excluded = "550e8400-e29b-41d4-a716-446655440001";
		let input: FileSearchInput = SearchCli::try_parse_from([
			"wing",
			"report",
			"--mode",
			"full",
			"--tags",
			tag,
			"--exclude-tags",
			excluded,
			"--file-type",
			"pdf",
			"--file-type",
			"txt",
			"--min-size",
			"10",
			"--max-size",
			"100",
			"--date-field",
			"created",
			"--date-start",
			"2024-01-01T00:00:00Z",
			"--sort-field",
			"size",
			"--sort-direction",
			"asc",
			"--limit",
			"7",
			"--offset",
			"3",
			"--include-hidden",
			"--include-archived",
		])
		.unwrap()
		.search
		.into();
		assert!(matches!(input.mode, SearchMode::Full));
		assert_eq!(input.filters.file_types.unwrap(), ["pdf", "txt"]);
		let tags = input.filters.tags.unwrap();
		assert_eq!(tags.include, [Uuid::parse_str(tag).unwrap()]);
		assert_eq!(tags.exclude, [Uuid::parse_str(excluded).unwrap()]);
		let size = input.filters.size_range.unwrap();
		assert_eq!((size.min, size.max), (Some(10), Some(100)));
		let date = input.filters.date_range.unwrap();
		assert!(matches!(date.field, DateField::CreatedAt));
		assert_eq!(
			date.start.unwrap().to_rfc3339(),
			"2024-01-01T00:00:00+00:00"
		);
		assert!(date.end.is_none());
		assert!(matches!(input.sort.field, SortField::Size));
		assert!(matches!(input.sort.direction, SortDirection::Asc));
		assert_eq!((input.pagination.limit, input.pagination.offset), (7, 3));
		assert_eq!(input.filters.include_hidden, Some(true));
		assert_eq!(input.filters.include_archived, Some(true));
	}

	#[test]
	fn search_rejects_invalid_or_conflicting_scopes() {
		assert!(
			SearchCli::try_parse_from(["wing", "report", "--wing-path", "content://invalid"])
				.is_err()
		);
		assert!(SearchCli::try_parse_from([
			"wing",
			"report",
			"--wing-path",
			"/tmp",
			"--location",
			"550e8400-e29b-41d4-a716-446655440000",
		])
		.is_err());
		let input: FileSearchInput =
			SearchCli::try_parse_from(["wing", "report", "--wing-path", "/tmp"])
				.unwrap()
				.search
				.into();
		assert!(matches!(input.scope, SearchScope::Path { .. }));
	}

	#[test]
	fn search_rejects_invalid_filters() {
		for args in [
			vec!["wing", "q", "--tags", "bad-uuid"],
			vec!["wing", "q", "--date-start", "not-a-date"],
			vec!["wing", "q", "--min-size", "-1"],
			vec!["wing", "q", "--mode", "unknown"],
		] {
			assert!(SearchCli::try_parse_from(&args).is_err(), "{args:?}");
		}
	}
}

#[cfg(test)]
#[path = "args/matrix.rs"]
mod matrix;
