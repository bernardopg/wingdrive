//! Elevated helper for WingDrive administrator retries.
//!
//! The desktop app starts this binary with the Windows `runas` verb. It reads the
//! requested operation from a file and writes the per-item result to another file.
//! Operations never come from the command line, and file paths are passed to
//! filesystem APIs as data, not through a shell.

use std::path::PathBuf;
use std::process::ExitCode;

use wing_elevate::{run_request_file, AdminResponse, write_response};

fn main() -> ExitCode {
	let mut request: Option<PathBuf> = None;
	let mut response: Option<PathBuf> = None;

	let mut args = std::env::args_os().skip(1);
	while let Some(arg) = args.next() {
		if arg == "--request" {
			request = args.next().map(PathBuf::from);
		} else if arg == "--response" {
			response = args.next().map(PathBuf::from);
		} else {
			// Unknown arguments are ignored so the helper stays strict about data.
		}
	}

	let (Some(request), Some(response)) = (request, response) else {
		eprintln!("usage: wing-elevate --request <file> --response <file>");
		return ExitCode::from(2);
	};

	if let Err(message) = run_request_file(&request, &response) {
		let _ = write_response(&response, &AdminResponse::failed(message));
		return ExitCode::from(1);
	}

	ExitCode::SUCCESS
}
