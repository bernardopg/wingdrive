use clap::{Args, Subcommand};
use uuid::Uuid;

use wing_core::{
	domain::addressing::WingPath,
	ops::network::{
		pair::{
			cancel::input::PairCancelInput, generate::input::PairGenerateInput,
			join::input::PairJoinInput,
		},
		revoke::input::DeviceRevokeInput,
		wingdrop::WingdropSendInput,
	},
};

#[derive(Subcommand, Debug)]
pub enum PairCmd {
	/// Generate a pairing code (initiator)
	Generate {},
	/// Join using a pairing code (joiner)
	Join {
		/// Pairing code (12 words or JSON). If not provided, enters interactive mode.
		code: Option<String>,
		/// Node ID for remote pairing via pkarr (optional - enables relay path)
		#[arg(long)]
		node_id: Option<String>,
	},
	/// Show pairing sessions
	Status,
	/// Cancel a pairing session
	Cancel { session_id: Uuid },
}

impl PairCmd {
	pub fn to_generate_input(&self) -> Option<PairGenerateInput> {
		match self {
			Self::Generate {} => Some(PairGenerateInput {}),
			_ => None,
		}
	}

	pub fn to_join_input(&self) -> Option<PairJoinInput> {
		match self {
			Self::Join { code, node_id } => {
				// Code is required for non-interactive mode
				let code = code.as_ref()?.clone();

				// If node_id provided via CLI, wrap in QR JSON format for remote pairing
				let code = if let Some(node_id) = node_id {
					if code.trim().starts_with('{') {
						// Already JSON, just use it
						code
					} else {
						// Plain words + node_id - construct QR JSON format (v2)
						serde_json::json!({
							"version": 2,
							"words": code,
							"node_id": node_id,
						})
						.to_string()
					}
				} else {
					code
				};

				Some(PairJoinInput {
					code,
					node_id: node_id.clone(),
				})
			}
			_ => None,
		}
	}

	pub fn to_cancel_input(&self) -> Option<PairCancelInput> {
		match self {
			Self::Cancel { session_id } => Some(PairCancelInput {
				session_id: *session_id,
			}),
			_ => None,
		}
	}
}

#[derive(Subcommand, Debug)]
pub enum WingdropCmd {
	/// Send files or folders to a paired device; it has to accept them
	Send(WingdropSendArgs),
	/// List offers waiting for an answer on this device
	Pending,
	/// Accept an offer; files land in the Wingdrop folder
	Accept { drop_id: Uuid },
	/// Decline an offer
	Decline { drop_id: Uuid },
	/// Stop a drop this device is receiving
	Cancel { drop_id: Uuid },
}

#[derive(Args, Debug, Clone)]
pub struct WingdropSendArgs {
	/// Paired device ID (see `wing network devices`)
	pub device_id: Uuid,
	/// Files or folders to send
	#[arg(required = true)]
	pub paths: Vec<String>,
}

impl From<WingdropSendArgs> for WingdropSendInput {
	fn from(args: WingdropSendArgs) -> Self {
		let paths = args
			.paths
			.iter()
			.map(|s| {
				WingPath::from_uri(s).unwrap_or_else(|_| {
					// Relative paths mean the shell's folder, not the daemon's.
					let path = std::path::Path::new(s);
					WingPath::local(
						std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf()),
					)
				})
			})
			.collect();
		Self {
			device_id: args.device_id,
			paths,
		}
	}
}

#[derive(Args, Debug)]
pub struct RevokeArgs {
	pub device_id: Uuid,
	#[arg(long, short = 'y', default_value_t = false)]
	pub yes: bool,
}

impl From<RevokeArgs> for DeviceRevokeInput {
	fn from(args: RevokeArgs) -> Self {
		Self {
			device_id: args.device_id,
			remove_from_library: false,
		}
	}
}
