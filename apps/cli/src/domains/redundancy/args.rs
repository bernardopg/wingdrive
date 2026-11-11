use clap::Args;
use uuid::Uuid;

use wing_core::ops::redundancy::summary::RedundancySummaryInput;

#[derive(Args, Debug)]
pub struct SummaryArgs {
	/// Restrict summary to specific volume UUIDs (can be specified multiple times).
	/// Omit to summarize the entire library.
	#[arg(long = "volume", value_name = "VOLUME_UUID")]
	pub volumes: Option<Vec<Uuid>>,
}

impl From<SummaryArgs> for RedundancySummaryInput {
	fn from(args: SummaryArgs) -> Self {
		Self {
			volume_uuids: args.volumes,
		}
	}
}

#[derive(Args, Debug)]
pub struct AtRiskArgs {
	/// Only return files present on this volume (UUID)
	#[arg(long, value_name = "VOLUME_UUID")]
	pub volume: Option<Uuid>,

	/// Show redundant files (content on 2+ volumes) instead of at-risk (content on 1 volume)
	#[arg(long)]
	pub redundant: bool,

	/// Max number of files to show
	#[arg(long, default_value = "50")]
	pub limit: u32,

	/// Pagination offset
	#[arg(long, default_value = "0")]
	pub offset: u32,
}

#[derive(Args, Debug)]
pub struct CompareArgs {
	/// First volume UUID
	pub volume_a: Uuid,

	/// Second volume UUID
	pub volume_b: Uuid,

	/// What to compare
	#[arg(long, value_enum, default_value = "unique-a")]
	pub mode: CompareMode,

	/// Max number of files to show
	#[arg(long, default_value = "50")]
	pub limit: u32,

	/// Pagination offset
	#[arg(long, default_value = "0")]
	pub offset: u32,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum CompareMode {
	/// Files present on volume A but not on volume B
	UniqueA,
	/// Files present on volume B but not on volume A
	UniqueB,
	/// Files present on both volumes
	Shared,
}

#[cfg(test)]
mod tests {
	use super::*;
	use clap::Parser;
	#[derive(Parser)]
	struct Summary {
		#[command(flatten)]
		args: SummaryArgs,
	}
	#[derive(Parser)]
	struct Risk {
		#[command(flatten)]
		args: AtRiskArgs,
	}
	#[derive(Parser)]
	struct Compare {
		#[command(flatten)]
		args: CompareArgs,
	}
	const A: &str = "550e8400-e29b-41d4-a716-446655440000";
	const B: &str = "550e8400-e29b-41d4-a716-446655440001";

	#[test]
	fn summary_default_covers_library() {
		let input: RedundancySummaryInput = Summary::try_parse_from(["wing"]).unwrap().args.into();
		assert!(input.volume_uuids.is_none());
	}
	#[test]
	fn summary_preserves_multiple_volumes() {
		let input: RedundancySummaryInput =
			Summary::try_parse_from(["wing", "--volume", A, "--volume", B])
				.unwrap()
				.args
				.into();
		assert_eq!(
			input.volume_uuids.unwrap(),
			[Uuid::parse_str(A).unwrap(), Uuid::parse_str(B).unwrap()]
		);
	}
	#[test]
	fn summary_rejects_bad_uuid() {
		assert!(Summary::try_parse_from(["wing", "--volume", "bad"]).is_err());
	}
	#[test]
	fn risk_defaults_to_first_page() {
		let a = Risk::try_parse_from(["wing"]).unwrap().args;
		assert!(a.volume.is_none());
		assert!(!a.redundant);
		assert_eq!((a.limit, a.offset), (50, 0));
	}
	#[test]
	fn risk_preserves_filters() {
		let a = Risk::try_parse_from([
			"wing",
			"--volume",
			A,
			"--redundant",
			"--limit",
			"3",
			"--offset",
			"2",
		])
		.unwrap()
		.args;
		assert_eq!(a.volume, Some(Uuid::parse_str(A).unwrap()));
		assert!(a.redundant);
		assert_eq!((a.limit, a.offset), (3, 2));
	}
	#[test]
	fn risk_rejects_negative_pagination() {
		assert!(Risk::try_parse_from(["wing", "--limit", "-1"]).is_err());
	}
	#[test]
	fn compare_defaults_to_unique_a() {
		let a = Compare::try_parse_from(["wing", A, B]).unwrap().args;
		assert!(matches!(a.mode, CompareMode::UniqueA));
		assert_eq!((a.limit, a.offset), (50, 0));
	}
	#[test]
	fn compare_accepts_shared_mode_and_pagination() {
		let a = Compare::try_parse_from([
			"wing", A, B, "--mode", "shared", "--limit", "2", "--offset", "8",
		])
		.unwrap()
		.args;
		assert_eq!(a.volume_a, Uuid::parse_str(A).unwrap());
		assert_eq!(a.volume_b, Uuid::parse_str(B).unwrap());
		assert!(matches!(a.mode, CompareMode::Shared));
		assert_eq!((a.limit, a.offset), (2, 8));
	}
	#[test]
	fn compare_accepts_unique_b() {
		assert!(matches!(
			Compare::try_parse_from(["wing", A, B, "--mode", "unique-b"])
				.unwrap()
				.args
				.mode,
			CompareMode::UniqueB
		));
	}
	#[test]
	fn compare_rejects_missing_or_invalid_volumes_and_mode() {
		for args in [
			vec!["wing", A],
			vec!["wing", A, "bad"],
			vec!["wing", A, B, "--mode", "invalid"],
		] {
			assert!(Compare::try_parse_from(args).is_err());
		}
	}
}

#[cfg(test)]
#[path = "args/matrix.rs"]
mod matrix;
