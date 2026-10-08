//! A listing of a small folder that is not indexed yet returns its entries in
//! the first response, so the explorer paints them without waiting for the
//! indexer's events and a refetch.

use tempfile::TempDir;
use wing_core::{
	domain::WingPath,
	infra::{api::SessionContext, query::LibraryQuery},
	ops::files::query::directory_listing::{
		DirectoryListingInput, DirectoryListingQuery, DirectorySortBy,
	},
	Core,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn first_listing_of_small_unindexed_folder_has_entries(
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	let temp_dir = TempDir::new()?;
	let core = Core::new(temp_dir.path().join("data")).await?;
	let library = core
		.libraries
		.create_library("Listing", None, core.context.clone())
		.await?;

	let folder = temp_dir.path().join("folder");
	std::fs::create_dir_all(&folder)?;
	for i in 0..20 {
		std::fs::write(folder.join(format!("file-{i:02}.txt")), "x")?;
	}

	let device = core.device.to_device()?;
	let session =
		SessionContext::device_session(device.id, device.name.clone()).with_library(library.id());
	let query = DirectoryListingQuery::from_input(DirectoryListingInput {
		path: WingPath::Physical {
			device_slug: device.slug.clone(),
			path: folder.clone(),
		},
		folders_first: Some(false),
		limit: None,
		include_hidden: Some(false),
		sort_by: DirectorySortBy::Name,
		sort_direction: None,
	})?;
	let listing = query.execute(core.context.clone(), session).await?;

	assert_eq!(
		listing.files.len(),
		20,
		"first response should list the folder"
	);
	assert_eq!(listing.files[0].name, "file-00");
	Ok(())
}
