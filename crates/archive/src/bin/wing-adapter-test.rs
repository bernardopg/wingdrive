//! # wing-adapter-test
//!
//! The contract harness as a small program: `wing-adapter-test <adapter folder> [--case <name>]`
//! does what `wing adapter test` does. It exists for the adapter registry's CI, which builds it
//! from a pinned WingDrive commit in a few minutes because it needs only this crate, while the
//! `wing` CLI pulls in the whole core.

use std::path::PathBuf;
use std::process::ExitCode;

use wing_archive::harness;

const USAGE: &str = "usage: wing-adapter-test <adapter folder> [--case <name>]";

fn main() -> ExitCode {
	let mut dir = None;
	let mut case = None;
	let mut args = std::env::args().skip(1);
	while let Some(arg) = args.next() {
		match arg.as_str() {
			"-h" | "--help" => {
				println!("{USAGE}");
				return ExitCode::SUCCESS;
			}
			"--case" => match args.next() {
				Some(name) => case = Some(name),
				None => return usage_error("--case needs a name"),
			},
			_ if dir.is_none() && !arg.starts_with('-') => dir = Some(PathBuf::from(arg)),
			_ => return usage_error(&format!("unexpected argument {arg}")),
		}
	}
	let dir = dir.unwrap_or_else(|| PathBuf::from("."));

	let runtime = match tokio::runtime::Builder::new_current_thread()
		.enable_all()
		.build()
	{
		Ok(runtime) => runtime,
		Err(error) => {
			eprintln!("error: {error}");
			return ExitCode::FAILURE;
		}
	};
	match runtime.block_on(harness::test_cases(&dir, case.as_deref())) {
		Ok(reports) => {
			print!("{}", harness::format_reports(&reports));
			if reports.iter().all(|report| report.passed()) {
				ExitCode::SUCCESS
			} else {
				ExitCode::FAILURE
			}
		}
		Err(error) => {
			eprintln!("error: {error}");
			ExitCode::FAILURE
		}
	}
}

fn usage_error(message: &str) -> ExitCode {
	eprintln!("error: {message}\n{USAGE}");
	ExitCode::from(2)
}
