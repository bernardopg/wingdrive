use std::path::PathBuf;

use clap::{Args, Subcommand};
use uuid::Uuid;

use wing_core::ops::network::{
	pair::{
		cancel::input::PairCancelInput, generate::input::PairGenerateInput,
		join::input::PairJoinInput,
	},
	revoke::input::DeviceRevokeInput,
	wingdrop::WingdropSendInput,
};

use crate::domains::file::args::local_path;

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
		// Same rules as the file commands: relative paths mean the shell's
		// folder, not the daemon's, and `local://<device>/...` stays remote.
		let paths = args
			.paths
			.into_iter()
			.map(PathBuf::from)
			.map(local_path)
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

#[cfg(test)]
mod tests {
	use super::*;
	use clap::Parser;
	use wing_core::domain::addressing::WingPath;

	#[derive(Parser)]
	struct SendCli {
		#[command(flatten)]
		args: WingdropSendArgs,
	}
	#[derive(Parser)]
	struct RevokeCli {
		#[command(flatten)]
		args: RevokeArgs,
	}
	const DEVICE: &str = "550e8400-e29b-41d4-a716-446655440000";

	#[test]
	fn pairing_generate_only_converts_generate_command() {
		assert!(PairCmd::Generate {}.to_generate_input().is_some());
		assert!(PairCmd::Status.to_generate_input().is_none());
	}
	#[test]
	fn pairing_without_code_enters_interactive_mode() {
		assert!(PairCmd::Join {
			code: None,
			node_id: None
		}
		.to_join_input()
		.is_none());
	}
	#[test]
	fn pairing_plain_code_without_node_is_preserved() {
		let input = PairCmd::Join {
			code: Some("one two".into()),
			node_id: None,
		}
		.to_join_input()
		.unwrap();
		assert_eq!(input.code, "one two");
		assert!(input.node_id.is_none());
	}
	#[test]
	fn pairing_node_wraps_words_in_json() {
		let input = PairCmd::Join {
			code: Some("one two".into()),
			node_id: Some("node".into()),
		}
		.to_join_input()
		.unwrap();
		let json: serde_json::Value = serde_json::from_str(&input.code).unwrap();
		assert_eq!(json["version"], 2);
		assert_eq!(json["words"], "one two");
		assert_eq!(json["node_id"], "node");
		assert_eq!(input.node_id.as_deref(), Some("node"));
	}
	#[test]
	fn pairing_json_is_not_wrapped_twice() {
		let code = "  {\"version\":2}";
		let input = PairCmd::Join {
			code: Some(code.into()),
			node_id: Some("node".into()),
		}
		.to_join_input()
		.unwrap();
		assert_eq!(input.code, code);
	}
	#[test]
	fn pairing_cancel_preserves_session() {
		let id = Uuid::parse_str(DEVICE).unwrap();
		assert_eq!(
			PairCmd::Cancel { session_id: id }
				.to_cancel_input()
				.unwrap()
				.session_id,
			id
		);
		assert!(PairCmd::Status.to_cancel_input().is_none());
	}
	#[test]
	fn wingdrop_relative_path_is_resolved_before_sending() {
		let args = SendCli::try_parse_from([
			"wing",
			DEVICE,
			"relative-file",
			"local://other-device/folder/file",
		])
		.unwrap()
		.args;
		let input: WingdropSendInput = args.into();
		assert_eq!(input.device_id, Uuid::parse_str(DEVICE).unwrap());
		assert_eq!(
			input.paths[0].as_local_path().unwrap(),
			std::path::absolute("relative-file").unwrap()
		);
		assert!(
			matches!(&input.paths[1], WingPath::Physical { device_slug, .. } if device_slug == "other-device")
		);
	}
	#[test]
	fn wingdrop_requires_a_valid_device_and_path() {
		assert!(SendCli::try_parse_from(["wing", DEVICE]).is_err());
		assert!(SendCli::try_parse_from(["wing", "invalid", "file"]).is_err());
	}
	#[test]
	fn revoke_keeps_device_without_removing_library() {
		let args = RevokeCli::try_parse_from(["wing", DEVICE, "--yes"])
			.unwrap()
			.args;
		assert!(args.yes);
		let input: DeviceRevokeInput = args.into();
		assert_eq!(input.device_id, Uuid::parse_str(DEVICE).unwrap());
		assert!(!input.remove_from_library);
	}
	#[test]
	fn revoke_requires_valid_uuid() {
		assert!(RevokeCli::try_parse_from(["wing", "not-a-uuid"]).is_err());
	}
}
