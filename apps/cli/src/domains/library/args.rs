use clap::{Args, Subcommand};
use uuid::Uuid;

use wing_core::ops::libraries::{
	create::input::LibraryCreateInput, delete::input::LibraryDeleteInput,
	info::query::LibraryInfoQueryInput,
};
use wing_core::ops::network::sync_setup::{
	discovery::query::DiscoverRemoteLibrariesInput, input::LibrarySyncAction,
	input::LibrarySyncSetupInput,
};

#[derive(Args, Debug)]
pub struct LibraryCreateArgs {
	pub name: String,
}

impl From<LibraryCreateArgs> for LibraryCreateInput {
	fn from(args: LibraryCreateArgs) -> Self {
		Self::new(args.name)
	}
}

#[derive(Args, Debug)]
pub struct LibraryDeleteArgs {
	pub library_id: Uuid,
	#[arg(long, short = 'y', default_value_t = false)]
	pub yes: bool,
	#[arg(long, default_value_t = false)]
	pub delete_data: bool,
}

impl From<LibraryDeleteArgs> for LibraryDeleteInput {
	fn from(args: LibraryDeleteArgs) -> Self {
		Self {
			library_id: args.library_id,
			delete_data: args.delete_data,
		}
	}
}

#[derive(Args, Debug)]
pub struct LibraryInfoArgs {
	/// Library ID to get information about (optional, defaults to current library)
	pub library_id: Option<Uuid>,
}

impl LibraryInfoArgs {
	/// Create an input for the specified library ID or current library
	pub fn to_input(
		&self,
		current_library_id: Option<Uuid>,
	) -> anyhow::Result<LibraryInfoQueryInput> {
		let _library_id = self
			.library_id
			.or(current_library_id)
			.ok_or_else(|| anyhow::anyhow!("No library specified and no current library set"))?;

		Ok(LibraryInfoQueryInput {})
	}
}

#[derive(Args, Debug)]
pub struct LibrarySwitchArgs {
	/// Library ID to switch to
	#[arg(long)]
	pub library_id: Option<Uuid>,
	/// Library name to switch to
	#[arg(long)]
	pub name: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum SyncSetupCmd {
	/// Discover libraries on a paired device
	Discover(DiscoverArgs),
	/// Setup library sync between devices
	Setup(SetupArgs),
}

#[derive(Args, Debug)]
pub struct DiscoverArgs {
	/// Device ID to discover libraries from
	pub device_id: Uuid,
}

impl From<DiscoverArgs> for DiscoverRemoteLibrariesInput {
	fn from(args: DiscoverArgs) -> Self {
		Self {
			device_id: args.device_id,
		}
	}
}

/// Sync action accepted by `wing library sync-setup setup --action`.
#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyncActionArg {
	/// Share the local library with the remote device (needs --name)
	ShareLocal,
	/// Join a library the remote device already has (needs --remote-library and --name)
	JoinRemote,
	/// Merge two libraries (not implemented in the core yet)
	Merge,
}

#[derive(Args, Debug)]
pub struct SetupArgs {
	/// Local library ID
	#[arg(long)]
	pub local_library: Option<Uuid>,

	/// Remote device ID (paired device)
	#[arg(long)]
	pub remote_device: Option<Uuid>,

	/// Remote library ID (required for join-remote and merge)
	#[arg(long)]
	pub remote_library: Option<Uuid>,

	/// Sync action
	#[arg(long, value_enum, default_value_t = SyncActionArg::ShareLocal)]
	pub action: SyncActionArg,

	/// Library name: the shared name for share-local, the remote library's name for
	/// join-remote, the merged name for merge
	#[arg(long)]
	pub name: Option<String>,

	/// DEPRECATED: Leader device: "local" or "remote"
	#[arg(long)]
	pub leader: Option<String>,

	/// Local device ID (optional, asks the daemon if not specified)
	#[arg(long)]
	pub local_device: Option<Uuid>,

	/// Use interactive mode (default if no arguments provided)
	#[arg(short, long)]
	pub interactive: bool,
}

impl SetupArgs {
	pub fn is_interactive(&self) -> bool {
		self.interactive || (self.local_library.is_none() && self.remote_device.is_none())
	}

	/// Build the action input. `daemon_device_id` is the device ID the daemon reports
	/// for the selected instance; `--local-device` overrides it.
	pub fn to_input(&self, daemon_device_id: Uuid) -> anyhow::Result<LibrarySyncSetupInput> {
		let local_library = self
			.local_library
			.ok_or_else(|| anyhow::anyhow!("--local-library is required"))?;
		let remote_device = self
			.remote_device
			.ok_or_else(|| anyhow::anyhow!("--remote-device is required"))?;
		let local_device_id = self.local_device.unwrap_or(daemon_device_id);

		// Determine leader device ID
		let leader = self.leader.as_deref().unwrap_or("local");
		let leader_device_id = match leader {
			"local" => local_device_id,
			"remote" => remote_device,
			_ => anyhow::bail!("Leader must be 'local' or 'remote'"),
		};

		let action =
			match self.action {
				SyncActionArg::ShareLocal => {
					let name = self.name.clone().ok_or_else(|| {
						anyhow::anyhow!("--name is required for share-local action")
					})?;
					LibrarySyncAction::ShareLocalLibrary { library_name: name }
				}
				SyncActionArg::JoinRemote => {
					let remote_library_id = self.remote_library.ok_or_else(|| {
						anyhow::anyhow!("--remote-library is required for join-remote action")
					})?;
					let name = self.name.clone().ok_or_else(|| {
						anyhow::anyhow!("--name is required for join-remote action")
					})?;
					LibrarySyncAction::JoinRemoteLibrary {
						remote_library_id,
						remote_library_name: name,
					}
				}
				SyncActionArg::Merge => {
					let remote_library_id = self.remote_library.ok_or_else(|| {
						anyhow::anyhow!("--remote-library is required for merge action")
					})?;
					let name = self
						.name
						.clone()
						.ok_or_else(|| anyhow::anyhow!("--name is required for merge action"))?;
					LibrarySyncAction::MergeLibraries {
						local_library_id: local_library,
						remote_library_id,
						merged_name: name,
					}
				}
			};

		Ok(LibrarySyncSetupInput {
			local_device_id,
			remote_device_id: remote_device,
			local_library_id: local_library,
			remote_library_id: self.remote_library,
			action,
			leader_device_id,
		})
	}
}

#[cfg(test)]
mod tests {
	use clap::Parser;

	use super::*;

	#[derive(Parser, Debug)]
	struct Harness {
		#[command(flatten)]
		setup: SetupArgs,
	}

	fn parse(args: &[&str]) -> Result<SetupArgs, clap::Error> {
		let mut argv = vec!["setup"];
		argv.extend_from_slice(args);
		Harness::try_parse_from(argv).map(|harness| harness.setup)
	}

	const LIBRARY: &str = "3f8cb26f-de79-4d87-88dd-01be5f024041";
	const REMOTE_DEVICE: &str = "550e8400-e29b-41d4-a716-446655440000";
	const REMOTE_LIBRARY: &str = "7a9c2d1e-5f84-4b23-a567-1234567890ab";

	#[test]
	fn help_lists_exactly_the_actions_the_parser_accepts() {
		use clap::CommandFactory;

		let help = Harness::command().render_long_help().to_string();
		for action in ["share-local", "join-remote", "merge"] {
			assert!(help.contains(action), "help is missing {action}:\n{help}");
			assert!(
				parse(&["--action", action]).is_ok(),
				"{action} is documented but rejected"
			);
		}
		for stale in ["register-only", "create-shared"] {
			assert!(!help.contains(stale), "help still documents {stale}");
			assert!(parse(&["--action", stale]).is_err(), "{stale} was accepted");
		}
	}

	#[test]
	fn local_device_comes_from_the_daemon_unless_overridden() {
		let daemon_device = Uuid::new_v4();
		let args = parse(&[
			"--local-library",
			LIBRARY,
			"--remote-device",
			REMOTE_DEVICE,
			"--name",
			"Photos",
		])
		.unwrap();
		let input = args.to_input(daemon_device).unwrap();
		assert_eq!(input.local_device_id, daemon_device);
		assert_eq!(input.leader_device_id, daemon_device);
		assert!(matches!(
			input.action,
			LibrarySyncAction::ShareLocalLibrary { ref library_name } if library_name == "Photos"
		));

		let explicit = Uuid::new_v4();
		let explicit_text = explicit.to_string();
		let args = parse(&[
			"--local-library",
			LIBRARY,
			"--remote-device",
			REMOTE_DEVICE,
			"--name",
			"Photos",
			"--local-device",
			&explicit_text,
			"--leader",
			"remote",
		])
		.unwrap();
		let input = args.to_input(daemon_device).unwrap();
		assert_eq!(input.local_device_id, explicit);
		assert_eq!(input.leader_device_id.to_string(), REMOTE_DEVICE);
	}

	#[test]
	fn each_action_reports_its_missing_arguments() {
		let base = ["--local-library", LIBRARY, "--remote-device", REMOTE_DEVICE];
		let device = Uuid::new_v4();

		let error = parse(&base).unwrap().to_input(device).unwrap_err();
		assert!(error
			.to_string()
			.contains("--name is required for share-local"));

		let mut join = base.to_vec();
		join.extend(["--action", "join-remote", "--name", "Theirs"]);
		let error = parse(&join).unwrap().to_input(device).unwrap_err();
		assert!(error
			.to_string()
			.contains("--remote-library is required for join-remote"));

		join.extend(["--remote-library", REMOTE_LIBRARY]);
		let input = parse(&join).unwrap().to_input(device).unwrap();
		assert!(matches!(
			input.action,
			LibrarySyncAction::JoinRemoteLibrary { remote_library_id, .. }
				if remote_library_id.to_string() == REMOTE_LIBRARY
		));

		let mut merge = base.to_vec();
		merge.extend(["--action", "merge", "--remote-library", REMOTE_LIBRARY]);
		let error = parse(&merge).unwrap().to_input(device).unwrap_err();
		assert!(error.to_string().contains("--name is required for merge"));

		let mut bad_leader = base.to_vec();
		bad_leader.extend(["--name", "Photos", "--leader", "both"]);
		assert!(parse(&bad_leader).unwrap().to_input(device).is_err());
	}

	#[test]
	fn no_library_or_device_means_interactive() {
		assert!(parse(&[]).unwrap().is_interactive());
		assert!(!parse(&["--local-library", LIBRARY])
			.unwrap()
			.is_interactive());
		assert!(parse(&["--local-library", LIBRARY, "-i"])
			.unwrap()
			.is_interactive());
	}
}
