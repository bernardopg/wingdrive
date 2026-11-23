use super::*;
use clap::error::ErrorKind;
use crate::domains::network::PairCmd;

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
		assert!(matches!(cli.command, Commands::Wings(WingsCmd::List)));
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

#[test]
fn command_tree_parses_core_domain_subcommands() {
	let cases: Vec<Vec<&str>> = vec![
		vec!["wing", "status"],
		vec!["wing", "update"],
		vec!["wing", "cloud"],
		vec!["wing", "config", "show"],
		vec!["wing", "config", "get", "update.repo"],
		vec!["wing", "config", "set", "update.channel", "beta"],
		vec!["wing", "daemon", "install"],
		vec!["wing", "daemon", "uninstall"],
		vec!["wing", "daemon", "status"],
		vec!["wing", "library", "list"],
		vec!["wing", "library", "switch"],
		vec!["wing", "location", "list"],
		vec!["wing", "volume", "list"],
		vec!["wing", "volume", "scan"],
		vec!["wing", "job", "remote"],
		vec!["wing", "network", "status"],
		vec!["wing", "network", "devices"],
		vec!["wing", "network", "devices", "--connected"],
		vec!["wing", "wings", "list"],
		vec!["wing", "spaces", "layout", "my-wing"],
		vec!["wing", "index", "reset-cache"],
		vec!["wing", "redundancy", "summary"],
		vec!["wing", "redundancy", "at-risk"],
	];
	for args in cases {
		Cli::try_parse_from(&args).unwrap_or_else(|e| panic!("{args:?}: {e}"));
	}
}

#[test]
fn command_tree_rejects_missing_required_arguments() {
	let cases: Vec<Vec<&str>> = vec![
		vec!["wing", "config", "get"],
		vec!["wing", "config", "set", "update.repo"],
		vec!["wing", "library", "create"],
		vec!["wing", "library", "delete"],
		vec!["wing", "location", "remove"],
		vec!["wing", "location", "rescan"],
		vec!["wing", "file", "undo", "not-a-uuid"],
		vec!["wing", "job", "info"],
		vec!["wing", "job", "pause"],
		vec!["wing", "job", "resume"],
		vec!["wing", "job", "cancel"],
		vec!["wing", "network", "revoke"],
		vec!["wing", "tag", "create"],
		vec!["wing", "tag", "apply"],
		vec!["wing", "tag", "search"],
		vec!["wing", "volume", "add-cloud"],
		vec!["wing", "volume", "remove-cloud"],
		vec!["wing", "redundancy", "compare"],
	];
	for args in cases {
		assert!(
			Cli::try_parse_from(&args).is_err(),
			"expected parse failure for {args:?}"
		);
	}
}

#[test]
fn library_create_parses_name() {
	let cli = Cli::try_parse_from(["wing", "library", "create", "My Library"]).unwrap();
	let Commands::Library(LibraryCmd::Create(args)) = cli.command else {
		panic!("expected library create");
	};
	assert_eq!(args.name, "My Library");
}

#[test]
fn job_control_commands_parse_job_ids() {
	let id = "550e8400-e29b-41d4-a716-446655440000";
	for (verb, expect_pause, expect_resume, expect_cancel) in [
		("pause", true, false, false),
		("resume", false, true, false),
		("cancel", false, false, true),
	] {
		let cli = Cli::try_parse_from(["wing", "job", verb, id]).unwrap();
		match cli.command {
			Commands::Job(JobCmd::Pause(args)) => {
				assert!(expect_pause);
				assert_eq!(args.job_id.to_string(), id);
			}
			Commands::Job(JobCmd::Resume(args)) => {
				assert!(expect_resume);
				assert_eq!(args.job_id.to_string(), id);
			}
			Commands::Job(JobCmd::Cancel(args)) => {
				assert!(expect_cancel);
				assert_eq!(args.job_id.to_string(), id);
			}
			other => panic!("unexpected command for {verb}: {other:?}"),
		}
	}
}

#[test]
fn network_pair_and_devices_parse() {
	let cli = Cli::try_parse_from(["wing", "network", "pair", "generate"]).unwrap();
	assert!(matches!(
		cli.command,
		Commands::Network(NetworkCmd::Pair(PairCmd::Generate {}))
	));

	let cli = Cli::try_parse_from(["wing", "network", "pair", "join", "CODE"]).unwrap();
	match cli.command {
		Commands::Network(NetworkCmd::Pair(PairCmd::Join { code, .. })) => {
			assert_eq!(code.as_deref(), Some("CODE"));
		}
		other => panic!("expected pair join, got {other:?}"),
	}

	let cli = Cli::try_parse_from(["wing", "network", "devices", "--connected"]).unwrap();
	match cli.command {
		Commands::Network(NetworkCmd::Devices { connected }) => assert!(connected),
		other => panic!("expected devices, got {other:?}"),
	}
}

#[test]
fn volume_add_cloud_rejects_missing_service_specific_fields() {
	let err = Cli::try_parse_from([
		"wing",
		"volume",
		"add-cloud",
		"my-s3",
		"--service",
		"s3",
		"--bucket",
		"b",
	])
	.unwrap();
	let Commands::Volume(VolumeCmd::AddCloud(args)) = err.command else {
		panic!("expected add-cloud");
	};
	let message = args.validate_and_build().unwrap_err();
	assert!(message.contains("--region"));
}

#[test]
fn logs_show_and_follow_parse_through_cli() {
	let cli = Cli::try_parse_from(["wing", "logs", "show", "-n", "5", "--level", "info"]).unwrap();
	let Commands::Logs(LogsCmd::Show(args)) = cli.command else {
		panic!("expected logs show");
	};
	assert_eq!(args.lines, 5);
	assert_eq!(args.level.as_deref(), Some("info"));

	let cli = Cli::try_parse_from(["wing", "logs", "follow", "--component", "sync"]).unwrap();
	let Commands::Logs(LogsCmd::Follow(args)) = cli.command else {
		panic!("expected logs follow");
	};
	assert_eq!(args.component.as_deref(), Some("sync"));
}

#[test]
fn config_commands_parse_through_top_level_cli() {
	let cli = Cli::try_parse_from(["wing", "config", "set", "update.repo", "o/r"]).unwrap();
	match cli.command {
		Commands::Config(ConfigCmd::Set { key, value }) => {
			assert_eq!(key, "update.repo");
			assert_eq!(value, "o/r");
		}
		other => panic!("expected config set, got {other:?}"),
	}
}

#[test]
fn file_info_list_and_history_parse() {
	let cli = Cli::try_parse_from(["wing", "file", "info", "/tmp/file.txt"]).unwrap();
	match cli.command {
		Commands::File(FileCmd::Info(args)) => {
			assert_eq!(args.path, std::path::PathBuf::from("/tmp/file.txt"));
		}
		other => panic!("expected file info, got {other:?}"),
	}

	let cli = Cli::try_parse_from([
		"wing",
		"file",
		"list",
		"/tmp",
		"--limit",
		"10",
		"--include-hidden",
	])
	.unwrap();
	match cli.command {
		Commands::File(FileCmd::List(args)) => {
			assert_eq!(args.path, std::path::PathBuf::from("/tmp"));
			assert_eq!(args.limit, Some(10));
			assert!(args.include_hidden);
		}
		other => panic!("expected file list, got {other:?}"),
	}

	let cli = Cli::try_parse_from(["wing", "file", "history", "--limit", "3"]).unwrap();
	match cli.command {
		Commands::File(FileCmd::History(args)) => assert_eq!(args.limit, 3),
		other => panic!("expected file history, got {other:?}"),
	}
}

#[test]
fn devices_and_job_list_parse() {
	let cli =
		Cli::try_parse_from(["wing", "devices", "list", "--show-paired", "--detailed"]).unwrap();
	match cli.command {
		Commands::Devices(DevicesCmd::List(args)) => {
			assert!(args.show_paired);
			assert!(args.detailed);
			assert!(args.include_offline);
		}
		other => panic!("expected devices list, got {other:?}"),
	}

	let cli = Cli::try_parse_from(["wing", "job", "list", "--status", "running"]).unwrap();
	match cli.command {
		Commands::Job(JobCmd::List(args)) => {
			assert_eq!(args.status.as_deref(), Some("running"));
		}
		other => panic!("expected job list, got {other:?}"),
	}

	let id = "550e8400-e29b-41d4-a716-446655440000";
	let cli = Cli::try_parse_from(["wing", "job", "info", id]).unwrap();
	match cli.command {
		Commands::Job(JobCmd::Info(args)) => assert_eq!(args.job_id.to_string(), id),
		other => panic!("expected job info, got {other:?}"),
	}
}

#[test]
fn network_pair_status_and_cancel_parse() {
	let cli = Cli::try_parse_from(["wing", "network", "pair", "status"]).unwrap();
	assert!(matches!(
		cli.command,
		Commands::Network(NetworkCmd::Pair(PairCmd::Status))
	));

	let id = "550e8400-e29b-41d4-a716-446655440000";
	let cli = Cli::try_parse_from(["wing", "network", "pair", "cancel", id]).unwrap();
	match cli.command {
		Commands::Network(NetworkCmd::Pair(PairCmd::Cancel { session_id })) => {
			assert_eq!(session_id.to_string(), id);
		}
		other => panic!("expected pair cancel, got {other:?}"),
	}
}

#[test]
fn integrations_and_redundancy_commands_parse() {
	let cli = Cli::try_parse_from(["wing", "integrations", "list"]).unwrap();
	assert!(matches!(
		cli.command,
		Commands::Integrations(IntegrationsCmd::List)
	));

	let a = "550e8400-e29b-41d4-a716-446655440000";
	let b = "00000000-0000-0000-0000-000000000001";
	let cli = Cli::try_parse_from(["wing", "redundancy", "compare", a, b]).unwrap();
	match cli.command {
		Commands::Redundancy(RedundancyCmd::Compare(args)) => {
			assert_eq!(args.volume_a.to_string(), a);
			assert_eq!(args.volume_b.to_string(), b);
		}
		other => panic!("expected redundancy compare, got {other:?}"),
	}
}
