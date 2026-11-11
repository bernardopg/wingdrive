use clap::{Args, Subcommand, ValueEnum};

#[derive(Debug, Subcommand)]
pub enum EventsCmd {
	/// Monitor events in real-time
	Monitor(EventsMonitorArgs),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
	/// Human-readable summary (default)
	Human,
	/// Compact JSON output
	Json,
	/// Pretty-printed JSON output
	JsonPretty,
}

#[derive(Debug, Args)]
pub struct EventsMonitorArgs {
	/// Filter by event type(s) (comma-separated, e.g., "JobProgress,JobCompleted")
	/// Leave empty to see all events
	#[arg(short = 't', long)]
	pub event_type: Option<String>,

	/// Filter by library ID
	#[arg(short = 'l', long)]
	pub library_id: Option<uuid::Uuid>,

	/// Filter by job ID
	#[arg(short = 'j', long)]
	pub job_id: Option<String>,

	/// Filter by device ID
	#[arg(short = 'd', long)]
	pub device_id: Option<uuid::Uuid>,

	/// Show timestamps
	#[arg(long)]
	pub timestamps: bool,

	/// Output format
	#[arg(short = 'f', long, value_enum, default_value = "human")]
	pub format: OutputFormat,

	/// Show full event JSON (verbose mode)
	#[arg(short = 'v', long)]
	pub verbose: bool,

	/// Pretty print JSON output
	#[arg(short = 'p', long)]
	pub pretty: bool,
}

#[cfg(test)]
mod tests {
	use super::*;
	use clap::Parser;
	#[derive(Parser)]
	struct Monitor {
		#[command(flatten)]
		args: EventsMonitorArgs,
	}
	const ID: &str = "550e8400-e29b-41d4-a716-446655440000";
	#[test]
	fn monitor_defaults_to_human_without_filters() {
		let a = Monitor::try_parse_from(["wing"]).unwrap().args;
		assert!(matches!(a.format, OutputFormat::Human));
		assert!(a.event_type.is_none());
		assert!(a.library_id.is_none());
		assert!(a.device_id.is_none());
		assert!(a.job_id.is_none());
	}
	#[test]
	fn monitor_defaults_disable_optional_output_flags() {
		let a = Monitor::try_parse_from(["wing"]).unwrap().args;
		assert!(!a.timestamps && !a.verbose && !a.pretty);
	}
	#[test]
	fn monitor_preserves_event_type_list() {
		let a = Monitor::try_parse_from(["wing", "--event-type", "JobProgress,JobCompleted"])
			.unwrap()
			.args;
		assert_eq!(a.event_type.as_deref(), Some("JobProgress,JobCompleted"));
	}
	#[test]
	fn monitor_preserves_library_filter() {
		let a = Monitor::try_parse_from(["wing", "--library-id", ID])
			.unwrap()
			.args;
		assert_eq!(a.library_id, Some(uuid::Uuid::parse_str(ID).unwrap()));
	}
	#[test]
	fn monitor_preserves_device_filter() {
		let a = Monitor::try_parse_from(["wing", "--device-id", ID])
			.unwrap()
			.args;
		assert_eq!(a.device_id, Some(uuid::Uuid::parse_str(ID).unwrap()));
	}
	#[test]
	fn monitor_preserves_job_filter() {
		let a = Monitor::try_parse_from(["wing", "--job-id", "job-123"])
			.unwrap()
			.args;
		assert_eq!(a.job_id.as_deref(), Some("job-123"));
	}
	#[test]
	fn monitor_accepts_json_format() {
		assert!(matches!(
			Monitor::try_parse_from(["wing", "--format", "json"])
				.unwrap()
				.args
				.format,
			OutputFormat::Json
		));
	}
	#[test]
	fn monitor_accepts_pretty_json_format() {
		assert!(matches!(
			Monitor::try_parse_from(["wing", "--format", "json-pretty"])
				.unwrap()
				.args
				.format,
			OutputFormat::JsonPretty
		));
	}
	#[test]
	fn monitor_preserves_display_flags() {
		let a = Monitor::try_parse_from(["wing", "--timestamps", "--verbose", "--pretty"])
			.unwrap()
			.args;
		assert!(a.timestamps && a.verbose && a.pretty);
	}
	#[test]
	fn monitor_rejects_invalid_formats_and_uuid_filters() {
		for args in [
			vec!["wing", "--format", "xml"],
			vec!["wing", "--library-id", "bad"],
			vec!["wing", "--device-id", "bad"],
		] {
			assert!(Monitor::try_parse_from(args).is_err());
		}
	}
}

#[cfg(test)]
#[path = "args/matrix.rs"]
mod matrix;
