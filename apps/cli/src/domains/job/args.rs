use clap::Args;
use uuid::Uuid;

use wing_core::{
	infra::job::types::JobStatus,
	ops::jobs::{info::query::JobInfoQueryInput, list::query::JobListInput},
};

fn parse_status(value: &str) -> Result<String, String> {
	value
		.parse::<JobStatus>()
		.map(|_| value.to_owned())
		.map_err(|_| "expected queued, running, paused, completed, failed, or cancelled".to_owned())
}

#[derive(Args, Debug)]
pub struct JobListArgs {
	/// Filter by job status: queued, running, paused, completed, failed or cancelled
	#[arg(long, value_parser = parse_status)]
	pub status: Option<String>,
}

impl JobListArgs {
	pub fn to_input(&self, _library_id: Uuid) -> JobListInput {
		JobListInput {
			status: self
				.status
				.as_deref()
				.and_then(|s| s.parse::<JobStatus>().ok()),
		}
	}
}

#[derive(Args, Debug)]
pub struct JobInfoArgs {
	pub job_id: Uuid,
}

impl JobInfoArgs {
	pub fn to_input(&self) -> JobInfoQueryInput {
		JobInfoQueryInput {
			job_id: self.job_id,
		}
	}
}

#[derive(Args, Debug, Clone)]
pub struct JobMonitorArgs {
	/// Monitor a specific job by ID
	#[arg(long)]
	pub job_id: Option<Uuid>,

	/// Filter by job status: queued, running, paused, completed, failed or cancelled
	#[arg(long, value_parser = parse_status)]
	pub status: Option<String>,

	/// Refresh interval in seconds
	#[arg(long, default_value = "1")]
	pub refresh: u64,

	/// Use simple progress bars instead of TUI
	#[arg(long)]
	pub simple: bool,
}

#[derive(Args, Debug)]
pub struct JobControlArgs {
	/// Job ID to control
	pub job_id: Uuid,
}

#[cfg(test)]
mod tests {
	use super::*;
	use clap::Parser;
	#[derive(Parser)]
	struct List {
		#[command(flatten)]
		args: JobListArgs,
	}
	#[derive(Parser)]
	struct Monitor {
		#[command(flatten)]
		args: JobMonitorArgs,
	}
	#[test]
	fn job_list_rejects_unknown_status() {
		assert!(List::try_parse_from(["wing", "--status", "runnnning"]).is_err());
	}
	#[test]
	fn job_monitor_rejects_unknown_status() {
		assert!(Monitor::try_parse_from(["wing", "--status", "runnnning"]).is_err());
	}

	#[test]
	fn job_filters_preserve_all_supported_statuses() {
		for status in [
			"queued",
			"running",
			"paused",
			"completed",
			"failed",
			"cancelled",
		] {
			let args = List::try_parse_from(["wing", "--status", status])
				.unwrap()
				.args;
			assert_eq!(
				args.to_input(Uuid::nil()).status,
				Some(status.parse().unwrap())
			);
			assert_eq!(
				Monitor::try_parse_from(["wing", "--status", status])
					.unwrap()
					.args
					.status
					.as_deref(),
				Some(status)
			);
		}
		assert!(List::try_parse_from(["wing"])
			.unwrap()
			.args
			.to_input(Uuid::nil())
			.status
			.is_none());
	}
}
