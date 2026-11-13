//! Integrations CLI domain
//!
//! Connects third-party accounts and uses their services:
//! - `list`, `connect google|onedrive`, `remove`: connected accounts
//! - `google-client show|set|clear`: your own Google Cloud OAuth client
//! - `microsoft-client show|set|clear`: your own Microsoft Entra app
//! - `drive mount|unmount`: an account's Google Drive or OneDrive volume
//! - `photos albums|upload|import`: Google Photos
//! - `cloud-sync list|add|run|update|remove`: local folders synced with cloud folders

use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use clap::{Args, Subcommand, ValueEnum};
use comfy_table::{presets::UTF8_BORDERS_ONLY, Cell, Table};
use uuid::Uuid;

use crate::context::Context;
use crate::util::prelude::*;
use wing_core::domain::addressing::WingPath;
use wing_core::infra::job::handle::JobReceipt;
use wing_core::ops::cloud_sync::ops::{
	CloudSyncCreateInput, CloudSyncDeleteInput, CloudSyncDeleteOutput, CloudSyncListInput,
	CloudSyncRunInput, CloudSyncUpdateInput,
};
use wing_core::ops::cloud_sync::{CloudSyncPairInfo, SyncMode};
use wing_core::ops::integrations::accounts::{
	IntegrationAccountRemoveInput, IntegrationAccountRemoveOutput, IntegrationAccountsListInput,
};
use wing_core::ops::integrations::google::{
	GoogleClientInfoInput, GoogleConnectInput, GoogleConnectOutput, GoogleConnectStatusInput,
	GoogleDriveMountInput, GoogleDriveMountOutput, GoogleSetClientInput,
};
use wing_core::ops::integrations::microsoft::{
	MicrosoftClientInfoInput, MicrosoftConnectInput, MicrosoftConnectOutput,
	MicrosoftSetClientInput, OneDriveMountInput, OneDriveMountOutput,
};
use wing_core::ops::integrations::photos::{
	PhotosAlbumsInput, PhotosPickerImportInput, PhotosPickerStartInput, PhotosPickerStartOutput,
	PhotosPickerStatus, PhotosPickerStatusInput, PhotosUploadInput,
};
use wing_core::service::integrations::google::oauth::GoogleService;
use wing_core::service::integrations::google::photos::PhotosAlbum;
use wing_core::service::integrations::{
	ConnectStatus, ConnectionMode, IntegrationAccountInfo, IntegrationProvider, OAuthClientsInfo,
};

#[derive(Subcommand, Debug)]
pub enum IntegrationsCmd {
	/// List connected accounts
	List,
	/// Connect an account through the browser
	#[command(subcommand)]
	Connect(ConnectCmd),
	/// Disconnect an account and revoke WingDrive's access
	Remove(RemoveArgs),
	/// Show or change your own Google Cloud OAuth client
	#[command(subcommand)]
	GoogleClient(GoogleClientCmd),
	/// Show or change your own Microsoft Entra app registration
	#[command(subcommand)]
	MicrosoftClient(MicrosoftClientCmd),
	/// Mount or unmount an account's Google Drive or OneDrive
	#[command(subcommand)]
	Drive(DriveCmd),
	/// Google Photos albums, upload and import
	#[command(subcommand)]
	Photos(PhotosCmd),
	/// Keep local folders in sync with cloud folders
	#[command(subcommand)]
	CloudSync(CloudSyncCmd),
}

#[derive(Subcommand, Debug)]
pub enum ConnectCmd {
	/// Connect a Google account (Drive and Photos unless limited)
	Google(ConnectGoogleArgs),
	/// Connect a Microsoft account and mount its OneDrive
	Onedrive(ConnectOneDriveArgs),
}

#[derive(Args, Debug)]
pub struct ConnectOneDriveArgs {
	/// Sign in with your own Microsoft Entra app instead of WingDrive's
	#[arg(long)]
	pub own_client: bool,
	/// Preselect this account on Microsoft's page
	#[arg(long)]
	pub email: Option<String>,
	/// Do not try to open the browser; only print the link
	#[arg(long)]
	pub no_browser: bool,
}

#[derive(Args, Debug)]
pub struct ConnectGoogleArgs {
	/// Request only Google Drive access
	#[arg(long, conflicts_with = "photos_only")]
	pub drive_only: bool,
	/// Request only Google Photos access
	#[arg(long)]
	pub photos_only: bool,
	/// Sign in with your own Google Cloud client instead of WingDrive's
	#[arg(long)]
	pub own_client: bool,
	/// Preselect this account on Google's page
	#[arg(long)]
	pub email: Option<String>,
	/// Do not try to open the browser; only print the link
	#[arg(long)]
	pub no_browser: bool,
}

#[derive(Args, Debug)]
pub struct RemoveArgs {
	/// Account ID from `wing integrations list`
	pub account_id: String,
	/// Keep the grant at Google (when the account stays connected in another library)
	#[arg(long)]
	pub keep_grant: bool,
	/// Skip confirmation
	#[arg(long, short = 'y')]
	pub yes: bool,
}

#[derive(Subcommand, Debug)]
pub enum GoogleClientCmd {
	/// Show WingDrive's client and your own
	Show,
	/// Save your own "Desktop app" OAuth client from Google Cloud Console
	Set {
		#[arg(long)]
		client_id: String,
		#[arg(long)]
		client_secret: String,
	},
	/// Forget your own client
	Clear,
}

#[derive(Subcommand, Debug)]
pub enum MicrosoftClientCmd {
	/// Show WingDrive's app registration and your own
	Show,
	/// Save your own public client app registration from Microsoft Entra
	Set {
		/// The application (client) ID
		#[arg(long)]
		client_id: String,
	},
	/// Forget your own app registration
	Clear,
}

#[derive(Subcommand, Debug)]
pub enum DriveCmd {
	/// Mount the account's Google Drive or OneDrive as a volume and sidebar location
	Mount { account_id: String },
	/// Remove the volume; files in the cloud are not touched
	Unmount { account_id: String },
}

#[derive(Subcommand, Debug)]
pub enum PhotosCmd {
	/// List the albums WingDrive created
	Albums { account_id: String },
	/// Upload photos and videos (folders are walked)
	Upload {
		account_id: String,
		#[arg(required = true)]
		paths: Vec<PathBuf>,
		/// Add to an existing WingDrive album
		#[arg(long, conflicts_with = "new_album")]
		album_id: Option<String>,
		/// Create an album with this title
		#[arg(long)]
		new_album: Option<String>,
	},
	/// Pick photos in the browser and download them to a folder
	Import {
		account_id: String,
		destination: PathBuf,
		#[arg(long)]
		no_browser: bool,
	},
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum ModeArg {
	TwoWay,
	Upload,
	Download,
}

impl From<ModeArg> for SyncMode {
	fn from(mode: ModeArg) -> Self {
		match mode {
			ModeArg::TwoWay => SyncMode::TwoWay,
			ModeArg::Upload => SyncMode::Upload,
			ModeArg::Download => SyncMode::Download,
		}
	}
}

#[derive(Subcommand, Debug)]
pub enum CloudSyncCmd {
	/// List sync pairs and their last result
	List,
	/// Pair a local folder with a cloud folder
	Add {
		/// Name shown in lists
		name: String,
		/// Local folder (created if missing)
		local: PathBuf,
		/// Cloud folder URI, such as gdrive://me@gmail.com/Backup
		remote: String,
		#[arg(long, value_enum, default_value = "two-way")]
		mode: ModeArg,
		/// Propagate deletions (to the trash) between sides
		#[arg(long)]
		propagate_deletes: bool,
		/// Run automatically every N minutes (5 or more)
		#[arg(long)]
		every: Option<u32>,
		/// Do not start the first run now
		#[arg(long)]
		no_run: bool,
	},
	/// Run a pair now
	Run { id: Uuid },
	/// Change a pair
	Update {
		id: Uuid,
		#[arg(long, value_enum)]
		mode: Option<ModeArg>,
		#[arg(long)]
		propagate_deletes: Option<bool>,
		/// Minutes between runs; 0 turns automatic runs off
		#[arg(long)]
		every: Option<u32>,
		#[arg(long)]
		enabled: Option<bool>,
	},
	/// Delete a pair; files stay where they are
	Remove {
		id: Uuid,
		#[arg(long, short = 'y')]
		yes: bool,
	},
}

pub async fn run(ctx: &Context, cmd: IntegrationsCmd) -> Result<()> {
	match cmd {
		IntegrationsCmd::List => list(ctx).await,
		IntegrationsCmd::Connect(ConnectCmd::Google(args)) => connect_google(ctx, args).await,
		IntegrationsCmd::Connect(ConnectCmd::Onedrive(args)) => connect_onedrive(ctx, args).await,
		IntegrationsCmd::Remove(args) => remove(ctx, args).await,
		IntegrationsCmd::GoogleClient(cmd) => google_client(ctx, cmd).await,
		IntegrationsCmd::MicrosoftClient(cmd) => microsoft_client(ctx, cmd).await,
		IntegrationsCmd::Drive(cmd) => drive(ctx, cmd).await,
		IntegrationsCmd::Photos(cmd) => photos(ctx, cmd).await,
		IntegrationsCmd::CloudSync(cmd) => cloud_sync(ctx, cmd).await,
	}
}

/// Opens `url` in the default browser; failures only mean the user copies the link.
fn open_browser(url: &str) {
	#[cfg(target_os = "linux")]
	let opener = "xdg-open";
	#[cfg(target_os = "macos")]
	let opener = "open";
	#[cfg(target_os = "windows")]
	let opener = "explorer";
	#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
	let _ = std::process::Command::new(opener)
		.arg(url)
		.stdout(std::process::Stdio::null())
		.stderr(std::process::Stdio::null())
		.spawn();
}

fn services_label(services: &[GoogleService]) -> String {
	services
		.iter()
		.map(|s| match s {
			GoogleService::Drive => "Drive",
			GoogleService::Photos => "Photos",
		})
		.collect::<Vec<_>>()
		.join(", ")
}

/// Services of an account; a Microsoft account's only service is OneDrive.
fn account_services(account: &IntegrationAccountInfo) -> String {
	match account.provider {
		IntegrationProvider::Microsoft => "OneDrive".to_string(),
		_ => services_label(&account.services),
	}
}

fn drive_root(account: &IntegrationAccountInfo) -> Option<String> {
	let scheme = match account.provider {
		IntegrationProvider::Microsoft => "onedrive",
		_ => "gdrive",
	};
	account
		.drive
		.as_ref()
		.map(|d| format!("{scheme}://{}/", d.identifier))
}

fn mode_label(mode: ConnectionMode) -> &'static str {
	match mode {
		ConnectionMode::App => "WingDrive app",
		ConnectionMode::Custom => "own client",
	}
}

fn print_account(account: &IntegrationAccountInfo) {
	println!("Connected {} ({})", account.email, account.id);
	println!("  Services: {}", account_services(account));
	println!("  Signed in with: {}", mode_label(account.mode));
	if let Some(root) = drive_root(account) {
		println!("  Drive: {root}");
	}
}

async fn list(ctx: &Context) -> Result<()> {
	let accounts: Vec<IntegrationAccountInfo> =
		execute_query!(ctx, IntegrationAccountsListInput {});
	print_output!(ctx, &accounts, |accounts: &Vec<IntegrationAccountInfo>| {
		if accounts.is_empty() {
			println!(
				"No accounts connected. Run `wing integrations connect google` or `wing integrations connect onedrive`."
			);
			return;
		}
		let mut table = Table::new();
		table.load_style(UTF8_BORDERS_ONLY);
		table.set_header(vec!["ID", "Provider", "Email", "Services", "Sign-in", "Drive"]);
		for account in accounts {
			table.add_row(vec![
				Cell::new(&account.id),
				Cell::new(format!("{:?}", account.provider)),
				Cell::new(&account.email),
				Cell::new(account_services(account)),
				Cell::new(mode_label(account.mode)),
				Cell::new(drive_root(account).unwrap_or_else(|| "-".to_string())),
			]);
		}
		println!("{table}");
	});
	Ok(())
}

pub async fn connect_google(ctx: &Context, args: ConnectGoogleArgs) -> Result<()> {
	let services = if args.drive_only {
		vec![GoogleService::Drive]
	} else if args.photos_only {
		vec![GoogleService::Photos]
	} else {
		vec![GoogleService::Drive, GoogleService::Photos]
	};
	let started: GoogleConnectOutput = execute_action!(
		ctx,
		GoogleConnectInput {
			mode: if args.own_client {
				ConnectionMode::Custom
			} else {
				ConnectionMode::App
			},
			services,
			login_hint: args.email,
		}
	);
	println!(
		"Sign in to Google in your browser:\n\n  {}\n",
		started.auth_url
	);
	if !args.no_browser {
		open_browser(&started.auth_url);
	}
	wait_for_sign_in(ctx, started.session_id).await
}

pub async fn connect_onedrive(ctx: &Context, args: ConnectOneDriveArgs) -> Result<()> {
	let started: MicrosoftConnectOutput = execute_action!(
		ctx,
		MicrosoftConnectInput {
			mode: if args.own_client {
				ConnectionMode::Custom
			} else {
				ConnectionMode::App
			},
			login_hint: args.email,
		}
	);
	println!(
		"Sign in to Microsoft in your browser:\n\n  {}\n",
		started.auth_url
	);
	if !args.no_browser {
		open_browser(&started.auth_url);
	}
	wait_for_sign_in(ctx, started.session_id).await
}

/// Polls a browser sign-in of either provider until it ends.
async fn wait_for_sign_in(ctx: &Context, session_id: Uuid) -> Result<()> {
	println!("Waiting for the sign-in to finish (Ctrl+C to stop)...");
	loop {
		tokio::time::sleep(Duration::from_secs(1)).await;
		let status: ConnectStatus = execute_query!(
			ctx,
			GoogleConnectStatusInput { session_id }
		);
		match status {
			ConnectStatus::Pending => continue,
			ConnectStatus::Completed { account } => {
				print_output!(ctx, &account, print_account);
				return Ok(());
			}
			ConnectStatus::Failed { message } => anyhow::bail!(message),
			ConnectStatus::Cancelled => anyhow::bail!("Sign-in cancelled"),
		}
	}
}

async fn remove(ctx: &Context, args: RemoveArgs) -> Result<()> {
	confirm_or_abort(
		&format!(
			"Disconnect account {}? Its Drive volume and sync pairs that use it stop working.",
			args.account_id
		),
		args.yes,
	)?;
	let out: IntegrationAccountRemoveOutput = execute_action!(
		ctx,
		IntegrationAccountRemoveInput {
			account_id: args.account_id,
			revoke: !args.keep_grant,
		}
	);
	print_output!(ctx, &out, |o: &IntegrationAccountRemoveOutput| {
		println!("Disconnected account {}", o.account_id);
	});
	Ok(())
}

fn print_clients(provider: &str, setup: &str, info: &OAuthClientsInfo) {
	match &info.app {
		Some(app) => println!(
			"Sign in with your account: WingDrive's client {} ({:?})",
			app.client_id, app.source
		),
		None => println!("Sign in with your account: not available in this build"),
	}
	match &info.custom_client_id {
		Some(id) => println!("Your own {provider} client: {id}"),
		None => println!("Your own {provider} client: none. {setup}"),
	}
}

async fn microsoft_client(ctx: &Context, cmd: MicrosoftClientCmd) -> Result<()> {
	let info: OAuthClientsInfo = match cmd {
		MicrosoftClientCmd::Show => execute_query!(ctx, MicrosoftClientInfoInput {}),
		MicrosoftClientCmd::Set { client_id } => execute_action!(
			ctx,
			MicrosoftSetClientInput {
				client_id: Some(client_id),
			}
		),
		MicrosoftClientCmd::Clear => {
			execute_action!(ctx, MicrosoftSetClientInput { client_id: None })
		}
	};
	print_output!(ctx, &info, |info: &OAuthClientsInfo| {
		print_clients(
			"Microsoft Entra",
			"Register a public client app with the redirect URI http://localhost/oauth/callback and run `wing integrations microsoft-client set`.",
			info,
		)
	});
	Ok(())
}

async fn google_client(ctx: &Context, cmd: GoogleClientCmd) -> Result<()> {
	let info: OAuthClientsInfo = match cmd {
		GoogleClientCmd::Show => execute_query!(ctx, GoogleClientInfoInput {}),
		GoogleClientCmd::Set {
			client_id,
			client_secret,
		} => execute_action!(
			ctx,
			GoogleSetClientInput {
				client_id: Some(client_id),
				client_secret: Some(client_secret),
			}
		),
		GoogleClientCmd::Clear => execute_action!(
			ctx,
			GoogleSetClientInput {
				client_id: None,
				client_secret: None,
			}
		),
	};
	print_output!(ctx, &info, |info: &OAuthClientsInfo| {
		print_clients(
			"Google Cloud",
			"Create a \"Desktop app\" client in Google Cloud Console and run `wing integrations google-client set`.",
			info,
		)
	});
	Ok(())
}

async fn drive(ctx: &Context, cmd: DriveCmd) -> Result<()> {
	let (account_id, mounted) = match cmd {
		DriveCmd::Mount { account_id } => (account_id, true),
		DriveCmd::Unmount { account_id } => (account_id, false),
	};
	let accounts: Vec<IntegrationAccountInfo> =
		execute_query!(ctx, IntegrationAccountsListInput {});
	let microsoft = accounts
		.iter()
		.any(|a| a.id == account_id && a.provider == IntegrationProvider::Microsoft);
	if microsoft {
		let out: OneDriveMountOutput = execute_action!(
			ctx,
			OneDriveMountInput {
				account_id,
				mounted
			}
		);
		print_output!(ctx, &out, |o: &OneDriveMountOutput| match &o.drive {
			Some(drive) => println!("OneDrive mounted at onedrive://{}/", drive.identifier),
			None => println!("OneDrive unmounted"),
		});
		return Ok(());
	}
	let out: GoogleDriveMountOutput = execute_action!(
		ctx,
		GoogleDriveMountInput {
			account_id,
			mounted
		}
	);
	print_output!(ctx, &out, |o: &GoogleDriveMountOutput| match &o.drive {
		Some(drive) => println!("Google Drive mounted at gdrive://{}/", drive.identifier),
		None => println!("Google Drive unmounted"),
	});
	Ok(())
}

fn local_path(path: PathBuf) -> Result<WingPath> {
	let path = if path.is_absolute() {
		path
	} else {
		std::env::current_dir()?.join(path)
	};
	Ok(WingPath::local(path))
}

async fn photos(ctx: &Context, cmd: PhotosCmd) -> Result<()> {
	match cmd {
		PhotosCmd::Albums { account_id } => {
			let albums: Vec<PhotosAlbum> = execute_query!(ctx, PhotosAlbumsInput { account_id });
			print_output!(ctx, &albums, |albums: &Vec<PhotosAlbum>| {
				if albums.is_empty() {
					println!("WingDrive has not created any albums in this account.");
				}
				for album in albums {
					println!("{}  {}", album.id, album.title);
				}
			});
		}
		PhotosCmd::Upload {
			account_id,
			paths,
			album_id,
			new_album,
		} => {
			let paths = paths
				.into_iter()
				.map(local_path)
				.collect::<Result<Vec<_>>>()?;
			let receipt: JobReceipt = execute_action!(
				ctx,
				PhotosUploadInput {
					account_id,
					paths,
					album_id,
					new_album_title: new_album,
				}
			);
			print_output!(ctx, &receipt, |r: &JobReceipt| {
				println!("Upload started as job {}", r.id);
			});
		}
		PhotosCmd::Import {
			account_id,
			destination,
			no_browser,
		} => {
			let started: PhotosPickerStartOutput = execute_action!(
				ctx,
				PhotosPickerStartInput {
					account_id: account_id.clone()
				}
			);
			println!(
				"Select photos in your browser, then press Done:\n\n  {}\n",
				started.picker_uri
			);
			if !no_browser {
				open_browser(&started.picker_uri);
			}
			loop {
				tokio::time::sleep(Duration::from_secs(
					started.poll_interval_seconds.max(1) as u64
				))
				.await;
				let status: PhotosPickerStatus = execute_query!(
					ctx,
					PhotosPickerStatusInput {
						account_id: account_id.clone(),
						session_id: started.session_id.clone(),
					}
				);
				if status.ready {
					break;
				}
			}
			let receipt: JobReceipt = execute_action!(
				ctx,
				PhotosPickerImportInput {
					account_id,
					session_id: started.session_id,
					destination: local_path(destination)?,
				}
			);
			print_output!(ctx, &receipt, |r: &JobReceipt| {
				println!("Download started as job {}", r.id);
			});
		}
	}
	Ok(())
}

fn print_pairs(pairs: &Vec<CloudSyncPairInfo>) {
	if pairs.is_empty() {
		println!("No sync pairs. Add one with `wing integrations cloud-sync add`.");
		return;
	}
	let mut table = Table::new();
	table.load_style(UTF8_BORDERS_ONLY);
	table.set_header(vec![
		"ID", "Name", "Local", "Cloud", "Mode", "Every", "Last run",
	]);
	for pair in pairs {
		table.add_row(vec![
			Cell::new(pair.id),
			Cell::new(&pair.name),
			Cell::new(&pair.local_path),
			Cell::new(pair.remote.display()),
			Cell::new(format!("{:?}", pair.mode)),
			Cell::new(
				pair.interval_minutes
					.map(|m| format!("{m} min"))
					.unwrap_or_else(|| "manual".into()),
			),
			Cell::new(match (&pair.last_status, pair.running) {
				(_, true) => "running".to_string(),
				(Some(status), _) => status.clone(),
				(None, _) => "never".to_string(),
			}),
		]);
	}
	println!("{table}");
}

async fn cloud_sync(ctx: &Context, cmd: CloudSyncCmd) -> Result<()> {
	match cmd {
		CloudSyncCmd::List => {
			let pairs: Vec<CloudSyncPairInfo> = execute_query!(ctx, CloudSyncListInput {});
			print_output!(ctx, &pairs, print_pairs);
		}
		CloudSyncCmd::Add {
			name,
			local,
			remote,
			mode,
			propagate_deletes,
			every,
			no_run,
		} => {
			let remote = WingPath::from_uri(&remote)
				.map_err(|e| anyhow::anyhow!("Invalid cloud folder URI: {e}"))?;
			let local = match local_path(local)? {
				WingPath::Physical { path, .. } => path.to_string_lossy().into_owned(),
				_ => unreachable!("local_path builds a physical path"),
			};
			let pair: CloudSyncPairInfo = execute_action!(
				ctx,
				CloudSyncCreateInput {
					name,
					local_path: local,
					remote,
					mode: mode.into(),
					propagate_deletes,
					interval_minutes: every,
					run_now: !no_run,
				}
			);
			print_output!(ctx, &pair, |p: &CloudSyncPairInfo| {
				println!("Created sync pair {} ({})", p.name, p.id);
			});
		}
		CloudSyncCmd::Run { id } => {
			let receipt: JobReceipt = execute_action!(ctx, CloudSyncRunInput { id });
			print_output!(ctx, &receipt, |r: &JobReceipt| {
				println!("Sync started as job {}", r.id);
			});
		}
		CloudSyncCmd::Update {
			id,
			mode,
			propagate_deletes,
			every,
			enabled,
		} => {
			let pair: CloudSyncPairInfo = execute_action!(
				ctx,
				CloudSyncUpdateInput {
					id,
					name: None,
					mode: mode.map(Into::into),
					propagate_deletes,
					interval_minutes: every.map(|m| (m > 0).then_some(m)),
					enabled,
				}
			);
			print_output!(ctx, &vec![pair], print_pairs);
		}
		CloudSyncCmd::Remove { id, yes } => {
			confirm_or_abort(
				&format!("Delete sync pair {id}? Files on both sides stay where they are."),
				yes,
			)?;
			let out: CloudSyncDeleteOutput = execute_action!(ctx, CloudSyncDeleteInput { id });
			print_output!(ctx, &out, |o: &CloudSyncDeleteOutput| {
				println!("Deleted sync pair {}", o.id);
			});
		}
	}
	Ok(())
}
