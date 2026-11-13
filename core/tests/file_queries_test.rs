//! File queries against an indexed location: media listing, lookup by path and
//! by ID, alternate instances of the same content, and files by tag.

mod helpers;

use helpers::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::{
	path::{Path, PathBuf},
	sync::Arc,
};
use uuid::Uuid;
use wing_core::{
	domain::{addressing::WingPath, tag::RelationshipType, ContentKind, File},
	infra::{
		action::LibraryAction,
		api::SessionContext,
		db::entities::entry,
		query::{LibraryQuery, QueryError},
	},
	location::IndexMode,
	ops::{
		files::query::{
			alternate_instances::{AlternateInstancesInput, AlternateInstancesQuery},
			file_by_id::FileByIdQuery,
			file_by_path::FileByPathQuery,
			media_listing::{MediaListingQuery, MediaSortBy},
		},
		tags::{
			apply::{action::ApplyTagsAction, input::ApplyTagsInput},
			create::{action::CreateTagAction, input::CreateTagInput},
			files_by_tag::{GetFilesByTagInput, GetFilesByTagQuery},
			manager::TagManager,
		},
	},
};

/// A 1x1 PNG, so content identification classifies the file as an image.
const PNG: &[u8] = &[
	0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
	0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
	0x89, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
	0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
	0x42, 0x60, 0x82,
];

/// A PNG that differs from [`PNG`] only in trailing bytes, so it gets its own
/// content identity and a different size.
fn png_with_padding(padding: usize) -> Vec<u8> {
	let mut bytes = PNG.to_vec();
	bytes.extend(std::iter::repeat_n(0u8, padding));
	bytes
}

fn session(harness: &IndexingHarness) -> SessionContext {
	let mut session = SessionContext::device_session(
		wing_core::device::get_current_device_id(),
		wing_core::device::get_current_device_slug(),
	);
	session.current_library_id = Some(harness.library.id());
	session
}

async fn run<Q: LibraryQuery>(
	harness: &IndexingHarness,
	query: Q,
) -> Result<Q::Output, QueryError> {
	query
		.execute(harness.core.context.clone(), session(harness))
		.await
}

fn local(path: &Path) -> WingPath {
	WingPath::local(path.to_path_buf())
}

fn names(files: &[File]) -> Vec<String> {
	files.iter().map(|f| f.name.clone()).collect()
}

fn write(root: &Path, relative: &str, bytes: &[u8]) -> PathBuf {
	let path = root.join(relative);
	std::fs::create_dir_all(path.parent().unwrap()).unwrap();
	std::fs::write(&path, bytes).unwrap();
	path
}

async fn entry_by_name(harness: &IndexingHarness, name: &str) -> entry::Model {
	entry::Entity::find()
		.filter(entry::Column::Name.eq(name))
		.one(harness.library.db().conn())
		.await
		.unwrap()
		.unwrap_or_else(|| panic!("no entry named {name}"))
}

/// Indexes a location holding images in nested folders, two copies of the same
/// image, and a text file.
async fn indexed_media_library(test_name: &str) -> anyhow::Result<(IndexingHarness, PathBuf)> {
	let harness = IndexingHarnessBuilder::new(test_name)
		.disable_watcher()
		.build()
		.await?;
	let root = harness.temp_path().join("media");
	write(&root, "album/small.png", &png_with_padding(1));
	write(&root, "album/large.png", &png_with_padding(4096));
	write(&root, "album/notes.txt", b"not media");
	write(&root, "album/trip/deep.png", &png_with_padding(64));
	write(&root, "album/trip/copy.png", &png_with_padding(1));
	// `_` is a LIKE wildcard: this sibling must not leak into "album_x" listings.
	write(&root, "album_x/inside.png", &png_with_padding(8));
	write(&root, "albumYx/sub/outside.png", &png_with_padding(16));
	std::fs::create_dir_all(root.join("empty")).unwrap();
	harness
		.add_and_index_location(&root, "Media", IndexMode::Deep)
		.await?;
	Ok((harness, root))
}

#[tokio::test]
async fn media_listing_lists_direct_children_or_descendants() -> anyhow::Result<()> {
	let (harness, root) = indexed_media_library("media_listing").await?;
	let album = root.join("album");

	let direct = run(&harness, MediaListingQuery::new(local(&album))).await?;
	let mut direct_names = names(&direct.files);
	direct_names.sort();
	assert_eq!(direct_names, ["large", "small"]);
	assert_eq!(direct.total_count, 2);
	assert!(!direct.has_more);
	for file in &direct.files {
		assert_eq!(file.content_kind, ContentKind::Image);
		assert!(file.content_identity.is_some());
		let WingPath::Physical { path, .. } = &file.wing_path else {
			panic!("expected a physical path, got {:?}", file.wing_path);
		};
		assert_eq!(path.parent(), Some(album.as_path()));
	}

	let by_size = run(
		&harness,
		MediaListingQuery::with_options(local(&album), Some(true), None, None, MediaSortBy::Size),
	)
	.await?;
	// `small` and `copy` hold the same bytes, so their relative order is free.
	let by_size = names(&by_size.files);
	assert_eq!(by_size[..2], ["large", "deep"]);
	let mut tied = by_size[2..].to_vec();
	tied.sort();
	assert_eq!(tied, ["copy", "small"]);

	let by_name = run(
		&harness,
		MediaListingQuery::with_options(
			local(&album),
			Some(true),
			Some(vec![ContentKind::Image]),
			None,
			MediaSortBy::Name,
		),
	)
	.await?;
	assert_eq!(names(&by_name.files), ["copy", "deep", "large", "small"]);

	for sort in [
		MediaSortBy::Modified,
		MediaSortBy::Created,
		MediaSortBy::DateTaken,
	] {
		let listing = run(
			&harness,
			MediaListingQuery::with_options(local(&album), Some(true), None, None, sort),
		)
		.await?;
		assert_eq!(listing.files.len(), 4);
	}

	// Only videos: the album has none.
	let videos = run(
		&harness,
		MediaListingQuery::with_options(
			local(&album),
			Some(true),
			Some(vec![ContentKind::Video]),
			None,
			MediaSortBy::Name,
		),
	)
	.await?;
	assert!(videos.files.is_empty());
	assert_eq!(videos.total_count, 0);

	let empty = run(&harness, MediaListingQuery::new(local(&root.join("empty")))).await?;
	assert!(empty.files.is_empty());

	harness.shutdown().await
}

#[tokio::test]
async fn media_listing_reports_more_results_than_the_limit() -> anyhow::Result<()> {
	let (harness, root) = indexed_media_library("media_listing_limit").await?;
	let album = root.join("album");

	let limited = run(
		&harness,
		MediaListingQuery::with_options(
			local(&album),
			Some(true),
			None,
			Some(3),
			MediaSortBy::Name,
		),
	)
	.await?;
	assert_eq!(names(&limited.files), ["copy", "deep", "large"]);
	assert!(limited.has_more);

	let exact = run(
		&harness,
		MediaListingQuery::with_options(
			local(&album),
			Some(true),
			None,
			Some(4),
			MediaSortBy::Name,
		),
	)
	.await?;
	assert_eq!(exact.files.len(), 4);
	assert!(!exact.has_more);

	harness.shutdown().await
}

#[tokio::test]
async fn media_listing_matches_folder_names_literally() -> anyhow::Result<()> {
	let (harness, root) = indexed_media_library("media_listing_literal").await?;

	let listing = run(
		&harness,
		MediaListingQuery::with_options(
			local(&root.join("album_x")),
			Some(true),
			None,
			None,
			MediaSortBy::Name,
		),
	)
	.await?;
	assert_eq!(names(&listing.files), ["inside"]);

	harness.shutdown().await
}

#[tokio::test]
async fn media_listing_rejects_unindexed_and_unsupported_paths() -> anyhow::Result<()> {
	let (harness, root) = indexed_media_library("media_listing_errors").await?;

	let error = run(
		&harness,
		MediaListingQuery::new(local(&root.join("missing"))),
	)
	.await
	.unwrap_err();
	assert!(
		error.to_string().contains("has not been indexed yet"),
		"{error}"
	);

	let error = run(
		&harness,
		MediaListingQuery::new(WingPath::Content {
			content_id: Uuid::new_v4(),
		}),
	)
	.await
	.unwrap_err();
	assert!(error.to_string().contains("Content-addressed"), "{error}");

	let mut no_library = session(&harness);
	no_library.current_library_id = None;
	let error = MediaListingQuery::new(local(&root))
		.execute(harness.core.context.clone(), no_library)
		.await
		.unwrap_err();
	assert!(
		error.to_string().contains("No library in session"),
		"{error}"
	);

	harness.shutdown().await
}

#[tokio::test]
async fn files_are_found_by_path_and_by_id_with_their_other_copies() -> anyhow::Result<()> {
	let (harness, root) = indexed_media_library("file_lookup").await?;
	let small = root.join("album/small.png");
	let copy = root.join("album/trip/copy.png");

	let file = run(&harness, FileByPathQuery::new(small.clone()))
		.await?
		.expect("indexed file");
	assert_eq!(file.name, "small");
	assert_eq!(file.content_kind, ContentKind::Image);
	assert!(file.content_identity.is_some());
	let alternates: Vec<_> = file
		.alternate_paths
		.iter()
		.map(|p| match p {
			WingPath::Physical { path, .. } => path.to_path_buf(),
			other => panic!("unexpected alternate {other:?}"),
		})
		.collect();
	assert_eq!(alternates, std::slice::from_ref(&copy));

	let by_id = run(&harness, FileByIdQuery::new(file.id))
		.await?
		.expect("file by id");
	assert_eq!(by_id.name, "small");
	assert_eq!(by_id.alternate_paths.len(), 1);
	let WingPath::Physical { path, .. } = &by_id.wing_path else {
		panic!("expected a physical path, got {:?}", by_id.wing_path);
	};
	assert_eq!(path, &small);

	let instances = run(&harness, AlternateInstancesQuery::new(file.id)).await?;
	assert_eq!(instances.total_count, 2);
	let mut paths: Vec<_> = instances
		.instances
		.iter()
		.map(|f| match &f.wing_path {
			WingPath::Physical { path, .. } => path.to_path_buf(),
			other => panic!("unexpected instance {other:?}"),
		})
		.collect();
	paths.sort();
	let mut expected = vec![small.clone(), copy.clone()];
	expected.sort();
	assert_eq!(paths, expected);

	// A unique file has no other copies.
	let large = run(&harness, FileByPathQuery::new(root.join("album/large.png")))
		.await?
		.expect("indexed file");
	assert!(large.alternate_paths.is_empty());

	// Unknown paths and IDs are not errors.
	assert!(run(&harness, FileByPathQuery::new(root.join("nope.png")))
		.await?
		.is_none());
	assert!(run(&harness, FileByIdQuery::new(Uuid::new_v4()))
		.await?
		.is_none());

	let error = run(
		&harness,
		AlternateInstancesQuery::from_input(AlternateInstancesInput {
			entry_uuid: Uuid::new_v4(),
		})?,
	)
	.await
	.unwrap_err();
	assert!(error.to_string().contains("Entry not found"), "{error}");

	// A folder has no content identity.
	let album = entry_by_name(&harness, "album").await;
	let error = run(&harness, AlternateInstancesQuery::new(album.uuid.unwrap()))
		.await
		.unwrap_err();
	assert!(error.to_string().contains("no content identity"), "{error}");

	harness.shutdown().await
}

#[tokio::test]
async fn files_by_tag_follow_child_tags_and_confidence() -> anyhow::Result<()> {
	let (harness, _root) = indexed_media_library("files_by_tag").await?;
	let library_id = Some(harness.library.id());
	let actions = harness.core.context.get_action_manager().await.unwrap();

	let mut tags = Vec::new();
	for name in ["Travel", "Beach"] {
		let action = CreateTagAction::from_input(CreateTagInput::simple(name.to_string())).unwrap();
		tags.push(actions.dispatch_library(library_id, action).await?.tag_id);
	}
	let (parent, child) = (tags[0], tags[1]);
	TagManager::new(Arc::new(harness.library.db().conn().clone()))
		.create_relationship(parent, child, RelationshipType::ParentChild, None)
		.await?;

	let small = entry_by_name(&harness, "small").await;
	let deep = entry_by_name(&harness, "deep").await;
	actions
		.dispatch_library(
			library_id,
			ApplyTagsAction::from_input(ApplyTagsInput::user_tags_entry(
				vec![small.id],
				vec![parent],
			))
			.unwrap(),
		)
		.await?;
	let mut low_confidence = ApplyTagsInput::user_tags_entry(vec![deep.id], vec![child]);
	low_confidence.confidence = Some(0.4);
	actions
		.dispatch_library(
			library_id,
			ApplyTagsAction::from_input(low_confidence).unwrap(),
		)
		.await?;

	let query = |tag_id, include_children, min_confidence| GetFilesByTagQuery {
		input: GetFilesByTagInput {
			tag_id,
			include_children,
			min_confidence,
		},
	};

	let only_parent = run(&harness, query(parent, false, 0.0)).await?;
	assert_eq!(names(&only_parent.files), ["small"]);
	let tagged = &only_parent.files[0];
	assert_eq!(tagged.tags.len(), 1);
	assert_eq!(tagged.tags[0].id, parent);
	assert_eq!(tagged.content_kind, ContentKind::Image);

	let mut with_children = names(&run(&harness, query(parent, true, 0.0)).await?.files);
	with_children.sort();
	assert_eq!(with_children, ["deep", "small"]);

	let confident = run(&harness, query(parent, true, 0.5)).await?;
	assert_eq!(names(&confident.files), ["small"]);

	assert!(run(&harness, query(Uuid::new_v4(), true, 0.0))
		.await?
		.files
		.is_empty());

	harness.shutdown().await
}

#[tokio::test]
async fn paths_resolve_through_dotted_folders_and_sibling_locations() -> anyhow::Result<()> {
	let harness = IndexingHarnessBuilder::new("path_resolution")
		.disable_watcher()
		.build()
		.await?;
	let shelf = harness.temp_path().join("shelf");
	let shelf2 = harness.temp_path().join("shelf2");
	write(&shelf, "notes.txt", b"a file beside the folder");
	write(&shelf, "notes/inside.txt", b"inside");
	write(&shelf, "v1.2/README.MD", b"in a dotted folder");
	write(&shelf2, "other.txt", b"second location");
	harness
		.add_and_index_location(&shelf, "Shelf", IndexMode::Deep)
		.await?;
	harness
		.add_and_index_location(&shelf2, "Shelf 2", IndexMode::Deep)
		.await?;

	let lookup = |path: PathBuf| async { run(&harness, FileByPathQuery::new(path)).await };

	// A folder with a dot in its name, and a file inside it with an uppercase
	// extension, which the index keeps as written (DESK-096).
	let dotted = lookup(shelf.join("v1.2")).await?.expect("dotted folder");
	assert_eq!(
		(dotted.name.as_str(), dotted.extension.as_deref()),
		("v1.2", None)
	);
	let readme = lookup(shelf.join("v1.2/README.MD"))
		.await?
		.expect("file in dotted folder");
	assert_eq!(readme.name, "README");
	assert_eq!(readme.extension.as_deref(), Some("MD"));

	// The folder `notes` is not its sibling file `notes.txt`, and back.
	let folder = lookup(shelf.join("notes")).await?.expect("folder");
	assert_eq!(folder.extension, None);
	let file = lookup(shelf.join("notes.txt")).await?.expect("file");
	assert_eq!(file.extension.as_deref(), Some("txt"));
	assert!(lookup(shelf.join("notes/inside.txt")).await?.is_some());

	// `shelf` is a prefix of `shelf2` but does not contain it.
	let other = lookup(shelf2.join("other.txt"))
		.await?
		.expect("second location");
	assert_eq!(other.name, "other");

	// Clients without a registered device send the `local` placeholder, and a
	// location's volume is only filled in by its first index; neither may hide
	// an indexed file.
	let db = harness.library.db().conn();
	sea_orm::ConnectionTrait::execute_unprepared(db, "UPDATE locations SET volume_id = NULL")
		.await?;
	let placeholder = WingPath::Physical {
		device_slug: "local".to_string(),
		path: shelf.join("notes.txt"),
	};
	let entry = wing_core::ops::indexing::PathResolver::resolve_to_entry(db, &placeholder)
		.await?
		.expect("placeholder path in a location without a volume");
	assert_eq!(
		(entry.name.as_str(), entry.extension.as_deref()),
		("notes", Some("txt"))
	);

	harness.shutdown().await
}
