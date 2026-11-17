//! The Google Picker flow for accounts on WingDrive's own client, which only
//! hold `drive.file`, against the local fake of the Google APIs.
//!
//! Covers the whole product path: sign-in with a `drive.file` grant, the
//! Drive volume root listing granted items whose parent is not granted, the
//! Picker page served on loopback, a pick refreshing the volume, cancel and
//! session isolation, the API key gate, and full-Drive accounts never getting
//! the Picker.

#[path = "helpers/fake_google.rs"]
mod fake_google;

use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use fake_google::{FakeGoogle, FakeUser, FOLDER};
use tempfile::TempDir;
use uuid::Uuid;
use wing_core::{
	library::Library,
	service::integrations::{
		google::{oauth::DriveAccess, oauth::GoogleService, DrivePickerState},
		ConnectStatus, ConnectionMode, IntegrationAccountInfo, IntegrationError,
	},
	volume::backend::CloudServiceType,
	Core,
};

type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

const CLIENT_ID: &str = "123456789-test.apps.googleusercontent.com";

/// What the token endpoint grants to WingDrive's own client since 3.5.0.
const APP_SCOPES: &str = "openid https://www.googleapis.com/auth/userinfo.email https://www.googleapis.com/auth/userinfo.profile https://www.googleapis.com/auth/drive.file https://www.googleapis.com/auth/photoslibrary.appendonly https://www.googleapis.com/auth/photoslibrary.readonly.appcreateddata https://www.googleapis.com/auth/photospicker.mediaitems.readonly";

fn users() -> Vec<FakeUser> {
	vec![
		FakeUser {
			code: "code-ana".into(),
			sub: "1001".into(),
			email: "ana@example.com".into(),
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

async fn connect(
	core: &Core,
	library: &Arc<Library>,
	code: &str,
) -> Result<IntegrationAccountInfo, Box<dyn std::error::Error + Send + Sync>> {
	let integrations = core.context.integrations.clone();
	let (session_id, auth_url) = integrations
		.start_google_connect(
			library.clone(),
			ConnectionMode::App,
			vec![GoogleService::Drive, GoogleService::Photos],
			None,
		)
		.await?;
	let url = url::Url::parse(&auth_url)?;
	let query: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
	assert!(
		query["scope"].contains("drive.file") && !query["scope"].contains("/auth/drive "),
		"WingDrive's client asks for drive.file only: {}",
		query["scope"]
	);
	assert!(
		!query["scope"].contains("edit.appcreateddata"),
		"no feature edits app-created Photos media"
	);
	reqwest::get(format!(
		"{}?code={code}&state={}",
		query["redirect_uri"], query["state"]
	))
	.await?;
	for _ in 0..100 {
		match integrations.connect_status(library.id(), session_id).await {
			Some(ConnectStatus::Completed { account }) => return Ok(account),
			Some(ConnectStatus::Pending) => tokio::time::sleep(Duration::from_millis(50)).await,
			other => return Err(format!("sign-in failed: {other:?}").into()),
		}
	}
	Err("sign-in did not finish".into())
}

fn names(entries: &[wing_core::volume::backend::RawDirEntry]) -> Vec<String> {
	let mut names: Vec<String> = entries.iter().map(|e| e.name.clone()).collect();
	names.sort();
	names
}

async fn wait_for_state(
	core: &Core,
	library_id: Uuid,
	session_id: Uuid,
	want: DrivePickerState,
) -> TestResult {
	for _ in 0..100 {
		if core
			.context
			.integrations
			.drive_picker_status(library_id, session_id)
			.await
			== Some(want)
		{
			return Ok(());
		}
		tokio::time::sleep(Duration::from_millis(20)).await;
	}
	Err(format!("picker never reached {want:?}").into())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn drive_file_accounts_add_existing_files_with_the_picker() -> TestResult {
	let fake = FakeGoogle::start(users()).await;
	let (report, old) = {
		let mut s = fake.state.lock().unwrap();
		let work = s.insert("Work", FOLDER, "root", b"");
		let report = s.insert("report.pdf", "application/pdf", &work, b"pdf bytes");
		let old = s.insert("old.txt", "text/plain", "root", b"old");
		// Nothing is granted yet: a fresh drive.file grant sees no file.
		s.granted_files = Some(HashSet::new());
		s.granted_scope = Some(APP_SCOPES.to_string());
		(report, old)
	};
	// Set before Core starts: the integration service reads them once.
	std::env::set_var("WINGDRIVE_GOOGLE_ENDPOINTS_BASE", &fake.base_url);
	std::env::set_var("WINGDRIVE_GOOGLE_CLIENT_ID", CLIENT_ID);
	std::env::set_var("WINGDRIVE_GOOGLE_CLIENT_SECRET", "test-secret");
	std::env::remove_var("WINGDRIVE_GOOGLE_API_KEY");

	let temp = TempDir::new()?;
	let core = Core::new(temp.path().join("data")).await?;
	let library = core
		.libraries
		.create_library("Picker", None, core.context.clone())
		.await?;
	let integrations = core.context.integrations.clone();

	let ana = connect(&core, &library, "code-ana").await?;
	assert_eq!(ana.drive_access, Some(DriveAccess::AppFiles));
	assert_eq!(
		ana.services,
		vec![GoogleService::Drive, GoogleService::Photos],
		"three Photos scopes are enough for the Photos service"
	);

	// Without an API key the build cannot show the Picker.
	assert!(!ana.drive_picker);
	assert!(matches!(
		integrations
			.start_drive_picker(&library, &ana.id, None)
			.await,
		Err(IntegrationError::DrivePickerUnavailable(_))
	));
	std::env::set_var("WINGDRIVE_GOOGLE_API_KEY", "test-api-key");
	let accounts = integrations.list_accounts(&library).await?;
	assert!(accounts.iter().any(|a| a.id == ana.id && a.drive_picker));

	// The volume root lists only granted items, and nothing is granted yet.
	let drive = core
		.context
		.volume_manager
		.find_cloud_volume(CloudServiceType::GoogleDrive, "ana@example.com")
		.await
		.and_then(|v| v.backend.clone())
		.expect("Ana's Drive is mounted");
	assert!(drive.read_dir(Path::new("")).await?.is_empty());

	// The Picker page carries the key, the project number and the language.
	let (session, url) = integrations
		.start_drive_picker(&library, &ana.id, Some("pt-BR".into()))
		.await?;
	assert!(url.starts_with("http://127.0.0.1:"));
	let page = reqwest::get(&url).await?.text().await?;
	assert!(page.contains(r#"const KEY = "test-api-key";"#));
	assert!(page.contains(r#"const APP_ID = "123456789";"#));
	assert!(page.contains(r#"const LOCALE = "pt-BR";"#));
	assert!(
		page.contains("const TOKEN = \"at-"),
		"page carries an access token"
	);
	let wrong_state = url.replace("state=", "state=x");
	assert_eq!(reqwest::get(&wrong_state).await?.status(), 404);
	assert_eq!(
		integrations
			.drive_picker_status(library.id(), session)
			.await,
		Some(DrivePickerState::Waiting)
	);

	// The user picks a file inside a folder the grant does not cover.
	fake.state
		.lock()
		.unwrap()
		.granted_files
		.as_mut()
		.unwrap()
		.insert(report.clone());
	let done = url.replace("/drive-picker?", "/drive-picker/done?") + "&count=1";
	assert!(reqwest::get(&done).await?.status().is_success());
	wait_for_state(
		&core,
		library.id(),
		session,
		DrivePickerState::Picked { count: 1 },
	)
	.await?;
	// The pick refreshed the cached root listing.
	assert_eq!(names(&drive.read_dir(Path::new("")).await?), ["report.pdf"]);
	assert_eq!(
		drive.read(Path::new("report.pdf")).await?.as_ref(),
		b"pdf bytes"
	);
	// Files WingDrive creates join the grant and list at the root too.
	drive
		.write(Path::new("notes.txt"), bytes::Bytes::from_static(b"mine"))
		.await?;
	assert_eq!(
		names(&drive.read_dir(Path::new("")).await?),
		["notes.txt", "report.pdf"]
	);
	assert!(
		!names(&drive.read_dir(Path::new("")).await?).contains(&"old.txt".to_string()),
		"{old} was never picked"
	);

	// Sessions belong to their library, and unknown ones read as missing.
	assert_eq!(
		integrations
			.drive_picker_status(Uuid::new_v4(), session)
			.await,
		None
	);
	assert_eq!(
		integrations
			.drive_picker_status(library.id(), Uuid::new_v4())
			.await,
		None
	);

	// Closing the Picker without picking cancels the session.
	let (cancelled, url) = integrations
		.start_drive_picker(&library, &ana.id, None)
		.await?;
	let page = reqwest::get(&url).await?.text().await?;
	assert!(
		page.contains(r#"const LOCALE = "en";"#),
		"English by default"
	);
	let cancel = url.replace("/drive-picker?", "/drive-picker/cancel?");
	assert!(reqwest::get(&cancel).await?.status().is_success());
	wait_for_state(&core, library.id(), cancelled, DrivePickerState::Cancelled).await?;

	// An account with full Drive access never needs the Picker.
	fake.state.lock().unwrap().granted_scope = None;
	let bruno = connect(&core, &library, "code-bruno").await?;
	assert_eq!(bruno.drive_access, Some(DriveAccess::Full));
	assert!(!bruno.drive_picker);
	assert!(matches!(
		integrations
			.start_drive_picker(&library, &bruno.id, None)
			.await,
		Err(IntegrationError::DrivePickerUnavailable(_))
	));
	assert!(matches!(
		integrations
			.start_drive_picker(&library, "no-such-account", None)
			.await,
		Err(IntegrationError::AccountNotFound(_))
	));

	core.shutdown().await?;
	Ok(())
}
