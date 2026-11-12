use super::*;
use clap::error::ErrorKind;

macro_rules! bytes_case {
	($name:ident, $input:expr, $expected:expr) => {
		#[test]
		fn $name() {
			assert_eq!(format_bytes($input), $expected);
		}
	};
}

bytes_case!(bytes_zero, 0, "0 B");
bytes_case!(bytes_one, 1, "1 B");
bytes_case!(bytes_below_kib, 1023, "1023 B");
bytes_case!(bytes_one_kib, 1024, "1.0 KB");
bytes_case!(bytes_fractional_kib, 1536, "1.5 KB");
bytes_case!(bytes_one_mib, 1048576, "1.0 MB");
bytes_case!(bytes_fractional_mib, 1572864, "1.5 MB");
bytes_case!(bytes_one_gib, 1073741824, "1.0 GB");
bytes_case!(bytes_one_tib, 1099511627776, "1.0 TB");
bytes_case!(bytes_above_largest_unit, 2199023255552, "2.0 TB");

macro_rules! invalid_args {
	($name:ident, $kind:expr, [$($arg:expr),+ $(,)?]) => {
		#[test]
		fn $name() {
			let error = Cli::try_parse_from([$($arg),+]).unwrap_err();
			assert_eq!(error.kind(), $kind, "{error}");
		}
	};
}

invalid_args!(
	missing_command,
	ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand,
	["wing"]
);
invalid_args!(
	unknown_command,
	ErrorKind::InvalidSubcommand,
	["wing", "unrecognized"]
);
invalid_args!(
	unknown_format,
	ErrorKind::InvalidValue,
	["wing", "--format", "yaml", "status"]
);
invalid_args!(
	missing_format,
	ErrorKind::InvalidValue,
	["wing", "--format", "status"]
);
invalid_args!(
	file_without_command,
	ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand,
	["wing", "file"]
);
invalid_args!(
	undo_rejects_invalid_uuid,
	ErrorKind::ValueValidation,
	["wing", "file", "undo", "invalid"]
);
invalid_args!(
	history_rejects_negative_limit,
	ErrorKind::UnknownArgument,
	["wing", "file", "history", "--limit", "-1"]
);
invalid_args!(
	history_rejects_non_numeric_limit,
	ErrorKind::ValueValidation,
	["wing", "file", "history", "--limit", "many"]
);
invalid_args!(
	logs_rejects_invalid_line_count,
	ErrorKind::ValueValidation,
	["wing", "logs", "show", "--lines", "many"]
);
invalid_args!(
	discover_requires_device,
	ErrorKind::MissingRequiredArgument,
	["wing", "library", "sync-setup", "discover"]
);
invalid_args!(
	discover_rejects_invalid_device,
	ErrorKind::ValueValidation,
	["wing", "library", "sync-setup", "discover", "nope"]
);
invalid_args!(
	sync_setup_rejects_unknown_action,
	ErrorKind::InvalidValue,
	[
		"wing",
		"library",
		"sync-setup",
		"setup",
		"--action",
		"unknown"
	]
);
invalid_args!(
	config_get_requires_key,
	ErrorKind::MissingRequiredArgument,
	["wing", "config", "get"]
);
invalid_args!(
	config_set_requires_value,
	ErrorKind::MissingRequiredArgument,
	["wing", "config", "set", "update.repo"]
);
invalid_args!(
	restart_rejects_unknown_flag,
	ErrorKind::UnknownArgument,
	["wing", "restart", "--unknown"]
);

#[test]
fn undo_without_job_selects_latest() {
	let cli = Cli::try_parse_from(["wing", "file", "undo"]).unwrap();
	let Commands::File(FileCmd::Undo(args)) = cli.command else {
		panic!("expected undo")
	};
	assert_eq!(args.job_id, None);
}

#[test]
fn undo_with_job_preserves_uuid() {
	let id = "550e8400-e29b-41d4-a716-446655440000";
	let cli = Cli::try_parse_from(["wing", "file", "undo", id]).unwrap();
	let Commands::File(FileCmd::Undo(args)) = cli.command else {
		panic!("expected undo")
	};
	assert_eq!(args.job_id.unwrap().to_string(), id);
}

#[test]
fn history_default_limit_is_twenty() {
	let cli = Cli::try_parse_from(["wing", "file", "history"]).unwrap();
	let Commands::File(FileCmd::History(args)) = cli.command else {
		panic!("expected history")
	};
	assert_eq!(args.limit, 20);
}

#[test]
fn history_explicit_limit_is_preserved() {
	let cli = Cli::try_parse_from(["wing", "file", "history", "--limit", "7"]).unwrap();
	let Commands::File(FileCmd::History(args)) = cli.command else {
		panic!("expected history")
	};
	assert_eq!(args.limit, 7);
}

#[test]
fn logs_show_defaults() {
	let cli = Cli::try_parse_from(["wing", "logs", "show"]).unwrap();
	let Commands::Logs(LogsCmd::Show(args)) = cli.command else {
		panic!("expected logs show")
	};
	assert_eq!(args.lines, 100);
	assert!(args.level.is_none() && args.component.is_none() && args.job_id.is_none());
	assert!(!args.timestamps && !args.verbose);
}

#[test]
fn logs_show_preserves_all_filters() {
	let cli = Cli::try_parse_from([
		"wing",
		"logs",
		"show",
		"-n",
		"4",
		"--level",
		"warn",
		"--component",
		"index",
		"--job-id",
		"job",
		"-t",
		"-v",
	])
	.unwrap();
	let Commands::Logs(LogsCmd::Show(args)) = cli.command else {
		panic!("expected logs show")
	};
	assert_eq!(args.lines, 4);
	assert_eq!(args.level.as_deref(), Some("warn"));
	assert_eq!(args.component.as_deref(), Some("index"));
	assert_eq!(args.job_id.as_deref(), Some("job"));
	assert!(args.timestamps && args.verbose);
}

#[test]
fn logs_follow_preserves_display_and_filter_flags() {
	let cli = Cli::try_parse_from([
		"wing",
		"logs",
		"follow",
		"--level",
		"error",
		"--component",
		"sync",
		"--job-id",
		"job",
		"--show-job-id",
		"--show-library-id",
		"-t",
		"-v",
	])
	.unwrap();
	let Commands::Logs(LogsCmd::Follow(args)) = cli.command else {
		panic!("expected logs follow")
	};
	assert_eq!(args.level.as_deref(), Some("error"));
	assert_eq!(args.component.as_deref(), Some("sync"));
	assert_eq!(args.job_id.as_deref(), Some("job"));
	assert!(args.show_job_id && args.show_library_id && args.timestamps && args.verbose);
}

#[test]
fn start_foreground_is_opt_in() {
	let cli = Cli::try_parse_from(["wing", "start", "--foreground"]).unwrap();
	assert!(matches!(cli.command, Commands::Start { foreground: true }));
}

#[test]
fn stop_reset_is_opt_in() {
	let cli = Cli::try_parse_from(["wing", "stop", "--reset"]).unwrap();
	assert!(matches!(cli.command, Commands::Stop { reset: true }));
}

#[test]
fn spaces_alias_and_wings_select_same_command() {
	for command in ["spaces", "wings"] {
		let cli = Cli::try_parse_from(["wing", command, "list"]).unwrap();
		assert!(matches!(cli.command, Commands::Spaces(SpacesCmd::List)));
	}
}

#[test]
fn explicit_instance_name_errors_are_specific() {
	assert!(validate_instance_name("").unwrap_err().contains("empty"));
	assert!(validate_instance_name(&"a".repeat(65))
		.unwrap_err()
		.contains("too long"));
	assert!(validate_instance_name("../unsafe")
		.unwrap_err()
		.contains("invalid characters"));
}

#[test]
fn named_instance_data_directory_does_not_change_cli_config_path() {
	let root = tempfile::tempdir().unwrap();
	let base = root.path().to_path_buf();
	let named = daemon_data_dir(&base, Some("fixture"));
	assert_eq!(named, base.join("instances/fixture"));
	assert_eq!(CliConfig::config_path(&base), base.join("cli.json"));
}
