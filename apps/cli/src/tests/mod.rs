use super::*;

use crate::config::CliConfig;
use crate::util::error::{improve_core_error, CliError};

#[test]
fn clap_command_tree_is_consistent() {
	use clap::CommandFactory;
	Cli::command().debug_assert();
}

#[test]
fn defaults_select_human_output_without_an_instance_override() {
	let cli = Cli::try_parse_from(["wing", "status"]).unwrap();
	assert!(matches!(cli.command, Commands::Status));
	assert!(matches!(cli.format, OutputFormat::Human));
	assert!(cli.instance.is_none());
	assert!(cli.data_dir.is_none());
}

#[test]
fn explicit_json_instance_and_data_directory_are_preserved() {
	let cli = Cli::try_parse_from([
		"wing",
		"--format",
		"json",
		"--instance",
		"test-01",
		"--data-dir",
		"scratch",
		"status",
	])
	.unwrap();
	assert!(matches!(cli.format, OutputFormat::Json));
	assert_eq!(cli.instance.as_deref(), Some("test-01"));
	assert_eq!(cli.data_dir.as_deref(), Some(Path::new("scratch")));
}

#[test]
fn invalid_output_formats_and_unknown_commands_are_rejected() {
	for args in [
		vec!["wing", "--format", "xml", "status"],
		vec!["wing", "unknown-command"],
		vec!["wing", "file", "delete"],
		vec!["wing", "file", "rename", "only-one-argument"],
	] {
		assert!(Cli::try_parse_from(&args).is_err(), "{args:?}");
	}
}

#[test]
fn search_scope_errors_are_rejected_by_the_actual_command_tree() {
	for args in [
		vec![
			"wing",
			"search",
			"files",
			"report",
			"--wing-path",
			"content://invalid",
		],
		vec![
			"wing",
			"search",
			"files",
			"report",
			"--wing-path",
			"/tmp",
			"--location",
			"550e8400-e29b-41d4-a716-446655440000",
		],
	] {
		assert!(Cli::try_parse_from(&args).is_err(), "{args:?}");
	}
}

#[test]
fn deletion_defaults_to_trash_and_confirmation() {
	let cli = Cli::try_parse_from(["wing", "file", "delete", "first", "second"]).unwrap();
	let Commands::File(FileCmd::Delete(args)) = cli.command else {
		panic!("expected delete")
	};
	assert!(!args.permanent);
	assert!(!args.yes);
	assert_eq!(
		args.paths,
		vec![std::path::PathBuf::from("first"), "second".into()]
	);
}

#[test]
fn permanent_deletion_requires_explicit_flags() {
	let cli =
		Cli::try_parse_from(["wing", "file", "delete", "--permanent", "-y", "first"]).unwrap();
	let Commands::File(FileCmd::Delete(args)) = cli.command else {
		panic!("expected delete")
	};
	assert!(args.permanent);
	assert!(args.yes);
}

#[test]
fn file_names_starting_with_a_dash_are_accepted_after_the_separator() {
	let cli = Cli::try_parse_from(["wing", "file", "delete", "--", "--permanent"]).unwrap();
	let Commands::File(FileCmd::Delete(args)) = cli.command else {
		panic!("expected delete")
	};
	assert!(!args.permanent);
	assert_eq!(args.paths, vec![std::path::PathBuf::from("--permanent")]);
}

#[test]
fn reset_and_foreground_are_opt_in_for_restart() {
	let cli = Cli::try_parse_from(["wing", "restart"]).unwrap();
	assert!(matches!(
		cli.command,
		Commands::Restart {
			reset: false,
			foreground: false
		}
	));
	let cli = Cli::try_parse_from(["wing", "restart", "--reset", "--foreground"]).unwrap();
	assert!(matches!(
		cli.command,
		Commands::Restart {
			reset: true,
			foreground: true
		}
	));
}

#[test]
fn traversal_and_socket_path_injection_are_rejected_in_instance_names() {
	for invalid in [
		"",
		"..",
		"../other",
		"a/b",
		"a\\b",
		"a b",
		"a\n",
		"a\0b",
		"name.sock",
	] {
		assert!(validate_instance_name(invalid).is_err(), "{invalid:?}");
	}
	assert!(validate_instance_name(&"a".repeat(65)).is_err());
	for valid in ["default", "test-01", "test_01", &"a".repeat(64)] {
		assert!(validate_instance_name(valid).is_ok(), "{valid}");
	}
}

#[test]
fn new_config_is_persisted_in_a_nested_data_directory() {
	let root = tempfile::tempdir().unwrap();
	let data = root.path().join("nested/data");
	let config = CliConfig::load(&data).unwrap();
	assert!(config.current_library_id.is_none());
	assert_eq!(config.update.repo, "bernardopg/wingdrive");
	assert_eq!(config.update.channel, "stable");
	let saved: serde_json::Value =
		serde_json::from_slice(&std::fs::read(data.join("cli.json")).unwrap()).unwrap();
	assert!(saved["current_library_id"].is_null());
	assert_eq!(saved["update"]["repo"], config.update.repo);
}

#[test]
fn legacy_config_gets_update_defaults_without_losing_the_library() {
	let root = tempfile::tempdir().unwrap();
	let data = root.path().to_path_buf();
	let library = uuid::Uuid::new_v4();
	std::fs::write(
		data.join("cli.json"),
		serde_json::json!({"current_library_id": library}).to_string(),
	)
	.unwrap();
	let config = CliConfig::load(&data).unwrap();
	assert_eq!(config.current_library_id, Some(library));
	assert_eq!(config.update.repo, "bernardopg/wingdrive");
	assert_eq!(config.update.channel, "stable");
}

#[test]
fn malformed_config_is_reported_and_preserved() {
	let root = tempfile::tempdir().unwrap();
	let data = root.path().to_path_buf();
	for invalid in [
		"not JSON",
		r#"{"current_library_id":"invalid-uuid"}"#,
		r#"{"update":{}}"#,
	] {
		std::fs::write(data.join("cli.json"), invalid).unwrap();
		assert!(CliConfig::load(&data).is_err());
		assert_eq!(
			std::fs::read_to_string(data.join("cli.json")).unwrap(),
			invalid
		);
	}
}

#[test]
fn library_and_update_settings_survive_reload_and_clear() {
	let root = tempfile::tempdir().unwrap();
	let data = root.path().to_path_buf();
	let library = uuid::Uuid::new_v4();
	let mut config = CliConfig::load(&data).unwrap();
	config.set_current_library(library, &data).unwrap();
	config
		.set_update_repo("example/releases".to_owned(), &data)
		.unwrap();
	config.set_update_channel("beta".to_owned(), &data).unwrap();
	let mut reloaded = CliConfig::load(&data).unwrap();
	assert_eq!(reloaded.current_library_id, Some(library));
	assert_eq!(reloaded.update.repo, "example/releases");
	assert_eq!(reloaded.update.channel, "beta");
	reloaded.clear_current_library(&data).unwrap();
	let cleared = CliConfig::load(&data).unwrap();
	assert!(cleared.current_library_id.is_none());
	assert_eq!(cleared.update.channel, "beta");
	assert_eq!(cleared.update.repo, "example/releases");
}

#[test]
fn saving_config_reports_an_unwritable_directory_without_touching_existing_data() {
	let root = tempfile::tempdir().unwrap();
	let data = root.path().join("file-not-directory");
	std::fs::write(&data, b"preserve").unwrap();
	assert!(CliConfig::default().save(&data).is_err());
	assert_eq!(std::fs::read(&data).unwrap(), b"preserve");
}

#[test]
fn reset_removes_only_known_v2_data_and_preserves_other_files() {
	let root = tempfile::tempdir().unwrap();
	for name in ["libraries", "logs", "job_logs", "backup"] {
		std::fs::create_dir(root.path().join(name)).unwrap();
		std::fs::write(root.path().join(name).join("keep-or-remove"), b"data").unwrap();
	}
	for name in ["wingdrive.json", "device.json", "cli.json", "personal.txt"] {
		std::fs::write(root.path().join(name), b"data").unwrap();
	}
	reset_wingdrive_v2_data(root.path()).unwrap();
	for name in [
		"libraries",
		"logs",
		"job_logs",
		"wingdrive.json",
		"device.json",
	] {
		assert!(!root.path().join(name).exists(), "{name}");
	}
	for name in ["backup/keep-or-remove", "cli.json", "personal.txt"] {
		assert_eq!(
			std::fs::read(root.path().join(name)).unwrap(),
			b"data",
			"{name}"
		);
	}
	reset_wingdrive_v2_data(root.path()).unwrap();
	assert_eq!(
		std::fs::read(root.path().join("personal.txt")).unwrap(),
		b"data"
	);
}

#[test]
fn connection_errors_become_actionable_errors_and_domain_errors_keep_their_message() {
	for message in [
		"Connection refused",
		"Failed to connect to daemon socket",
		"No such file or directory",
	] {
		let error = improve_core_error(message.to_owned());
		assert!(matches!(error, CliError::DaemonNotRunning));
		assert!(error.to_string().contains("wing start"));
	}
	let error = improve_core_error("Permission denied copying destination".to_owned());
	assert!(matches!(error, CliError::CoreError(_)));
	assert!(error
		.to_string()
		.contains("Permission denied copying destination"));
}

#[test]
fn named_instances_keep_daemon_state_under_their_own_directory() {
	let base = Path::new("data");
	assert_eq!(daemon_data_dir(base, None), base);
	assert_eq!(
		daemon_data_dir(base, Some("test-01")),
		base.join("instances").join("test-01")
	);
}

mod rpc;

mod coverage;
