//! End-to-end tests of the Google integration against a local fake of the
//! Google APIs (`helpers/fake_google.rs`).
//!
//! `drive_backend_covers_every_volume_operation` drives `GoogleDriveBackend`
//! directly. `two_accounts_drive_photos_sync_edit_and_removal` runs the whole
//! product flow through `Core`: browser sign-in of two accounts, Drive mounted
//! as a volume with a sidebar location, live folder listing, uploads,
//! downloads, renames through the copy job, cloud sync, Photos upload and
//! picker import, editing a cloud file locally, and removing an account.

#[path = "helpers/fake_google.rs"]
mod fake_google;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use fake_google::{FakeGoogle, FakeUser, FOLDER};
use tempfile::TempDir;
use wing_core::{
	domain::addressing::{WingPath, WingPathBatch},
	infra::{api::SessionContext, query::LibraryQuery},
	ops::{
		cloud_sync::engine::{self, Manifest, SyncMode, SyncTarget},
		files::{
			copy::job::FileCopyJob,
			query::directory_listing::{
				DirectoryListingInput, DirectoryListingQuery, DirectorySortBy,
			},
		},
		integrations::photos::{PhotosImportJob, PhotosUploadJob},
	},
	service::integrations::{
		cloud_edit::CloudEditOpen,
		google::{
			oauth::GoogleService, GoogleApi, GoogleDriveBackend, GoogleEndpoints,
			GoogleTokenSource, OAuthClient,
		},
		ConnectStatus,
	},
	volume::{backend::CloudServiceType, VolumeBackend},
	Core,
};

type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

fn users() -> Vec<FakeUser> {
	vec![
		FakeUser {
			code: "code-ana".into(),
			sub: "1001".into(),
			email: "Ana@Example.com".into(),
			name: "Ana".into(),
		},
		FakeUser {
			code: "code-bruno".into(),
			sub: "2002".into(),
			email: "bruno@example.com".into(),
			name: "Bruno".into(),
		},
	]
}

fn client() -> OAuthClient {
	OAuthClient {
		client_id: "test.apps.googleusercontent.com".into(),
		client_secret: "test-secret".into(),
	}
}

fn backend_for(fake: &FakeGoogle, sub: &str) -> GoogleDriveBackend {
	fake.state
		.lock()
		.unwrap()
		.refresh_tokens
		.insert(format!("rt-{sub}"), sub.to_string());
	let endpoints = GoogleEndpoints::for_base_url(&fake.base_url);
	let http = reqwest::Client::new();
	let tokens = Arc::new(GoogleTokenSource::new(
		http.clone(),
		endpoints.clone(),
		client(),
		format!("rt-{sub}"),
	));
	GoogleDriveBackend::new(GoogleApi::new(http, endpoints, tokens), None)
}

fn names(entries: &[wing_core::volume::backend::RawDirEntry]) -> Vec<String> {
	let mut names: Vec<String> = entries.iter().map(|e| e.name.clone()).collect();
	names.sort();
	names
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn drive_backend_covers_every_volume_operation() -> TestResult {
	let fake = FakeGoogle::start(users()).await;
	let docs = {
		let mut s = fake.state.lock().unwrap();
		let docs = s.insert("Docs", FOLDER, "root", b"");
		s.insert("a.txt", "text/plain", &docs, b"hello");
		s.insert("dup.txt", "text/plain", "root", b"first");
		s.insert("dup.txt", "text/plain", "root", b"second");
		s.insert(
			"Plan",
			"application/vnd.google-apps.document",
			"root",
			b"doc body",
		);
		s.insert("Survey", "application/vnd.google-apps.form", "root", b"");
		docs
	};
	let drive = backend_for(&fake, "1001");

	// Listing: duplicates get numbered, Docs get an export extension, forms are hidden.
	let root = drive.read_dir(Path::new("")).await?;
	assert_eq!(
		names(&root),
		vec!["Docs", "Plan.docx", "dup (2).txt", "dup.txt"]
	);
	assert_eq!(drive.read(Path::new("dup.txt")).await?.as_ref(), b"first");
	assert_eq!(
		drive.read(Path::new("dup (2).txt")).await?.as_ref(),
		b"second"
	);
	assert_eq!(
		drive.read(Path::new("/Docs/a.txt")).await?.as_ref(),
		b"hello"
	);
	assert_eq!(
		drive
			.read_range(Path::new("Docs/a.txt"), 1..4)
			.await?
			.as_ref(),
		b"ell"
	);

	// Google Docs read as an export and only edit in the browser.
	let exported = drive.read(Path::new("Plan.docx")).await?;
	assert!(String::from_utf8_lossy(&exported).starts_with(
		"EXPORT(application/vnd.openxmlformats-officedocument.wordprocessingml.document):doc body"
	));
	assert!(drive.edits_in_browser(Path::new("Plan.docx")).await?);
	assert!(!drive.edits_in_browser(Path::new("dup.txt")).await?);
	assert!(drive
		.web_link(Path::new("Plan.docx"))
		.await?
		.unwrap()
		.contains("drive.google.com"));
	assert!(drive
		.write(Path::new("Plan.docx"), bytes::Bytes::from_static(b"x"))
		.await
		.is_err());

	// A 9 MiB upload takes two resumable chunks and downloads back intact.
	let temp = TempDir::new()?;
	let big: Vec<u8> = (0..9 * 1024 * 1024).map(|i| (i % 251) as u8).collect();
	std::fs::write(temp.path().join("big.bin"), &big)?;
	let before = fake.state.lock().unwrap().upload_chunks;
	assert_eq!(
		drive
			.write_from_file(Path::new("Docs/big.bin"), &temp.path().join("big.bin"))
			.await?,
		big.len() as u64
	);
	assert_eq!(fake.state.lock().unwrap().upload_chunks - before, 2);
	drive
		.read_to_file(Path::new("Docs/big.bin"), &temp.path().join("back.bin"))
		.await?;
	assert_eq!(std::fs::read(temp.path().join("back.bin"))?, big);

	// Overwriting updates the same file and changes its revision.
	drive
		.write(Path::new("Docs/new.txt"), bytes::Bytes::from_static(b"v1"))
		.await?;
	let first = drive.metadata(Path::new("Docs/new.txt")).await?.revision;
	drive
		.write(Path::new("Docs/new.txt"), bytes::Bytes::from_static(b"v2!"))
		.await?;
	let second = drive.metadata(Path::new("Docs/new.txt")).await?;
	assert_eq!(second.size, 3);
	assert_ne!(first, second.revision);
	assert_eq!(
		fake.state
			.lock()
			.unwrap()
			.files
			.values()
			.filter(|f| f.name == "new.txt")
			.count(),
		1
	);

	// Folders, rename across folders, server-side copy, trash and delete.
	drive.create_directory(Path::new("A/B/C"), true).await?;
	assert!(drive.exists(Path::new("A/B/C")).await?);
	drive
		.rename(Path::new("Docs/new.txt"), Path::new("A/B/renamed.txt"))
		.await?;
	assert!(!drive.exists(Path::new("Docs/new.txt")).await?);
	assert_eq!(
		drive.read(Path::new("A/B/renamed.txt")).await?.as_ref(),
		b"v2!"
	);
	drive
		.copy(Path::new("A/B/renamed.txt"), Path::new("Docs/copy.txt"))
		.await?;
	assert_eq!(
		drive.read(Path::new("Docs/copy.txt")).await?.as_ref(),
		b"v2!"
	);
	drive.trash(Path::new("Docs/copy.txt")).await?;
	assert!(!drive.exists(Path::new("Docs/copy.txt")).await?);
	assert!(fake
		.state
		.lock()
		.unwrap()
		.files
		.values()
		.any(|f| f.name == "copy.txt" && f.trashed));
	drive.delete(Path::new("A")).await?;
	assert!(!drive.exists(Path::new("A")).await?);
	let missing = drive.read_dir(Path::new("missing")).await.unwrap_err();
	assert!(
		matches!(missing, wing_core::volume::error::VolumeError::Io(ref e) if e.kind() == std::io::ErrorKind::NotFound)
	);

	// Pagination and a forced 401 that the client recovers from with a refresh.
	{
		let mut s = fake.state.lock().unwrap();
		for i in 0..5 {
			s.insert(&format!("page-{i}.txt"), "text/plain", &docs, b"p");
		}
		s.page_size = 2;
		s.fail_next_with_401 = true;
	}
	let refreshes = fake.state.lock().unwrap().refresh_requests;
	let fresh = backend_for(&fake, "1001");
	let listed = fresh.read_dir(Path::new("Docs")).await?;
	assert_eq!(
		listed
			.iter()
			.filter(|e| e.name.starts_with("page-"))
			.count(),
		5
	);
	assert!(fake.state.lock().unwrap().refresh_requests >= refreshes + 2);

	assert_eq!(
		drive.capacity().await?,
		Some((16106127360, 16106127360 - 1073741824))
	);
	Ok(())
}

async fn connect(
	core: &Core,
	library: &Arc<wing_core::library::Library>,
	code: &str,
) -> Result<ConnectStatus, Box<dyn std::error::Error + Send + Sync>> {
	let integrations = core.context.integrations.clone();
	let (session_id, auth_url) = integrations
		.start_google_connect(
			library.clone(),
			vec![GoogleService::Drive, GoogleService::Photos],
			None,
		)
		.await?;
	let url = url::Url::parse(&auth_url)?;
	let query: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
	assert_eq!(query["code_challenge_method"], "S256");
	assert_eq!(query["access_type"], "offline");
	// The browser follows Google's redirect to the daemon's loopback listener.
	let callback = format!(
		"{}?code={code}&state={}",
		query["redirect_uri"], query["state"]
	);
	let page = reqwest::get(&callback).await?.text().await?;
	assert!(page.contains("WingDrive"));
	for _ in 0..100 {
		let status = integrations
			.connect_status(library.id(), session_id)
			.await
			.unwrap();
		if status != ConnectStatus::Pending {
			return Ok(status);
		}
		tokio::time::sleep(Duration::from_millis(50)).await;
	}
	Err("sign-in did not finish".into())
}

fn gdrive(identifier: &str, path: &str) -> WingPath {
	WingPath::Cloud {
		service: CloudServiceType::GoogleDrive,
		identifier: identifier.into(),
		path: path.into(),
	}
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn two_accounts_drive_photos_sync_edit_and_removal() -> TestResult {
	let fake = FakeGoogle::start(users()).await;
	// Set before Core starts: the integration service reads them once.
	std::env::set_var("WINGDRIVE_GOOGLE_ENDPOINTS_BASE", &fake.base_url);
	std::env::set_var(
		"WINGDRIVE_GOOGLE_CLIENT_ID",
		"test.apps.googleusercontent.com",
	);
	std::env::set_var("WINGDRIVE_GOOGLE_CLIENT_SECRET", "test-secret");

	let temp = TempDir::new()?;
	let core = Core::new(temp.path().join("data")).await?;
	let library = core
		.libraries
		.create_library("Google", None, core.context.clone())
		.await?;
	let integrations = core.context.integrations.clone();

	// Two accounts in one library.
	let ConnectStatus::Completed { account: ana } = connect(&core, &library, "code-ana").await?
	else {
		panic!("Ana's sign-in failed");
	};
	assert_eq!(ana.email, "ana@example.com", "emails are stored lowercase");
	assert_eq!(
		ana.services,
		vec![GoogleService::Drive, GoogleService::Photos]
	);
	assert!(ana.drive.as_ref().is_some_and(|d| d.mounted));
	let ConnectStatus::Completed { account: bruno } =
		connect(&core, &library, "code-bruno").await?
	else {
		panic!("Bruno's sign-in failed");
	};
	assert_eq!(integrations.list_accounts(&library).await?.len(), 2);
	assert_ne!(
		ana.drive.as_ref().unwrap().fingerprint,
		bruno.drive.as_ref().unwrap().fingerprint
	);
	let failed = connect(&core, &library, "wrong-code").await?;
	assert!(matches!(failed, ConnectStatus::Failed { .. }));

	// Each Drive is a volume with a sidebar location that browses live.
	let volumes = core.context.volume_manager.clone();
	assert!(volumes
		.find_cloud_volume(CloudServiceType::GoogleDrive, "ana@example.com")
		.await
		.is_some());
	assert!(volumes
		.find_cloud_volume(CloudServiceType::GoogleDrive, "bruno@example.com")
		.await
		.is_some());
	let locations = wing_core::location::LocationManager::new(core.events.as_ref().clone())
		.list_locations(&library)
		.await?;
	assert!(locations
		.iter()
		.any(|l| l.path.to_string_lossy() == "gdrive://ana@example.com/"));

	// Upload through the copy job, then list the folder through the query.
	let local = TempDir::new()?;
	std::fs::create_dir_all(local.path().join("trip"))?;
	std::fs::write(local.path().join("trip/beach.jpg"), b"jpeg bytes")?;
	std::fs::write(local.path().join("trip/notes.txt"), b"day one")?;
	library
		.jobs()
		.dispatch(FileCopyJob::new(
			WingPathBatch::new(vec![WingPath::local(local.path().join("trip"))]),
			gdrive("ana@example.com", ""),
		))
		.await?
		.wait()
		.await?;
	let device = core.device.to_device()?;
	let session =
		SessionContext::device_session(device.id, device.name.clone()).with_library(library.id());
	let listing = DirectoryListingQuery::from_input(DirectoryListingInput {
		path: gdrive("ana@example.com", "trip"),
		folders_first: Some(true),
		limit: None,
		include_hidden: Some(false),
		sort_by: DirectorySortBy::Name,
		sort_direction: None,
	})?
	.execute(core.context.clone(), session.clone())
	.await?;
	let listed: Vec<String> = listing.files.iter().map(|f| f.name.clone()).collect();
	assert_eq!(listing.files.len(), 2, "{listed:?}");
	assert!(listing
		.files
		.iter()
		.all(|f| matches!(f.wing_path, WingPath::Cloud { .. })));

	// Rename inside Drive and download back to this device.
	library
		.jobs()
		.dispatch(FileCopyJob::new_rename(
			gdrive("ana@example.com", "trip/notes.txt"),
			"diary.txt".into(),
		))
		.await?
		.wait()
		.await?;
	let downloads = local.path().join("downloads");
	std::fs::create_dir_all(&downloads)?;
	library
		.jobs()
		.dispatch(FileCopyJob::new(
			WingPathBatch::new(vec![gdrive("ana@example.com", "trip/diary.txt")]),
			WingPath::local(downloads.clone()),
		))
		.await?
		.wait()
		.await?;
	assert_eq!(std::fs::read(downloads.join("diary.txt"))?, b"day one");

	// Two-way sync between a local folder and Bruno's Drive.
	let (backend, remote_root) = wing_core::ops::cloud_sync::resolve_remote(
		&core.context,
		&gdrive("bruno@example.com", "Backup"),
	)
	.await?;
	let sync_dir = local.path().join("sync");
	std::fs::create_dir_all(sync_dir.join("docs"))?;
	std::fs::write(sync_dir.join("docs/cv.pdf"), b"pdf")?;
	let target = SyncTarget {
		local_root: sync_dir.clone(),
		backend: backend.clone(),
		remote_root,
		use_system_trash: false,
	};
	let noop = |_: usize, _: usize, _: &str| {};
	let (report, manifest) =
		engine::run(&target, SyncMode::TwoWay, true, &Manifest::default(), &noop).await?;
	assert!(report.errors.is_empty(), "{:?}", report.errors);
	assert_eq!(report.uploaded, 1);
	assert_eq!(
		backend
			.read(Path::new("Backup/docs/cv.pdf"))
			.await?
			.as_ref(),
		b"pdf"
	);
	backend
		.write(
			Path::new("Backup/docs/letter.txt"),
			bytes::Bytes::from_static(b"hi"),
		)
		.await?;
	let (report, _) = engine::run(&target, SyncMode::TwoWay, true, &manifest, &noop).await?;
	assert_eq!(report.downloaded, 1);
	assert_eq!(std::fs::read(sync_dir.join("docs/letter.txt"))?, b"hi");

	// Photos: upload into a new album, then import what the picker selected.
	let photos = integrations.photos_client(&library, &ana.id).await?;
	let album = photos.create_album("Trip").await?;
	library
		.jobs()
		.dispatch(PhotosUploadJob {
			account_id: ana.id.clone(),
			paths: vec![local.path().join("trip")],
			album_id: Some(album.id.clone()),
			done: Vec::new(),
		})
		.await?
		.wait()
		.await?;
	{
		let s = fake.state.lock().unwrap();
		assert_eq!(
			s.media_items.len(),
			1,
			"only the photo is uploaded, not the text file"
		);
		assert_eq!(s.media_items[0].1, "beach.jpg");
		assert_eq!(s.media_items[0].2.as_deref(), Some(album.id.as_str()));
	}
	assert_eq!(photos.list_albums().await?.len(), 1);
	let picker = photos.create_picker_session().await?;
	assert!(!photos.get_picker_session(&picker.id).await?.media_items_set);
	{
		let mut s = fake.state.lock().unwrap();
		s.picker_ready = true;
		s.picked = vec![
			(
				"p1".into(),
				"IMG_1.jpg".into(),
				"image/jpeg".into(),
				b"photo".to_vec(),
			),
			(
				"p2".into(),
				"clip.mp4".into(),
				"video/mp4".into(),
				b"video".to_vec(),
			),
		];
	}
	assert!(photos.get_picker_session(&picker.id).await?.media_items_set);
	let imports = local.path().join("imports");
	library
		.jobs()
		.dispatch(PhotosImportJob {
			account_id: ana.id.clone(),
			session_id: picker.id.clone(),
			destination: imports.clone(),
			done: Vec::new(),
		})
		.await?
		.wait()
		.await?;
	assert_eq!(std::fs::read(imports.join("IMG_1.jpg"))?, b"photo");
	assert_eq!(std::fs::read(imports.join("clip.mp4"))?, b"video");
	assert!(fake.state.lock().unwrap().picker_deleted);

	// Edit a Drive file locally and close the session to upload it.
	let ana_volume = volumes
		.find_cloud_volume(CloudServiceType::GoogleDrive, "ana@example.com")
		.await
		.unwrap();
	let CloudEditOpen::Local { session: edit } = integrations
		.cloud_edits
		.open(
			gdrive("ana@example.com", "trip/diary.txt"),
			ana_volume.backend.clone().unwrap(),
			PathBuf::from("trip/diary.txt"),
		)
		.await?
	else {
		panic!("expected a local copy");
	};
	std::fs::write(&edit.local_path, b"day one, edited")?;
	integrations.cloud_edits.close(edit.id).await?;
	assert_eq!(
		ana_volume
			.backend
			.unwrap()
			.read(Path::new("trip/diary.txt"))
			.await?
			.as_ref(),
		b"day one, edited"
	);

	// Removing Ana revokes her grant and unmounts only her Drive.
	integrations
		.remove_google_account(&library, &ana.id, true)
		.await?;
	assert!(fake
		.state
		.lock()
		.unwrap()
		.revoked
		.contains(&"rt-1001".to_string()));
	assert!(volumes
		.find_cloud_volume(CloudServiceType::GoogleDrive, "ana@example.com")
		.await
		.is_none());
	assert!(volumes
		.find_cloud_volume(CloudServiceType::GoogleDrive, "bruno@example.com")
		.await
		.is_some());
	let remaining = integrations.list_accounts(&library).await?;
	assert_eq!(remaining.len(), 1);
	assert_eq!(remaining[0].email, "bruno@example.com");
	let locations = wing_core::location::LocationManager::new(core.events.as_ref().clone())
		.list_locations(&library)
		.await?;
	assert!(!locations.iter().any(|l| l
		.path
		.to_string_lossy()
		.starts_with("gdrive://ana@example.com")));

	core.shutdown().await?;
	Ok(())
}
