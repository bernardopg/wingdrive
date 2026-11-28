//! `wing adapter`: tools for writing an adapter. They run locally and need no daemon.

use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use clap::{Args, Subcommand};

use wing_archive::harness::{self, CaseReport};

#[derive(Subcommand, Debug)]
pub enum AdapterCmd {
	/// Run an adapter against its fixtures/<case> folders, as the daemon and the registry do
	Test(TestArgs),
}

#[derive(Args, Debug)]
pub struct TestArgs {
	/// Adapter folder, the one that holds adapter.toml
	#[arg(default_value = ".")]
	pub dir: PathBuf,
	/// Run only this case
	#[arg(long)]
	pub case: Option<String>,
}

pub async fn run(cmd: AdapterCmd) -> Result<()> {
	match cmd {
		AdapterCmd::Test(args) => test(&args).await,
	}
}

async fn test(args: &TestArgs) -> Result<()> {
	let reports = run_cases(&args.dir, args.case.as_deref()).await?;
	print!("{}", format_reports(&reports));
	let failed = reports.iter().filter(|report| !report.passed()).count();
	if failed > 0 {
		bail!("{failed} of {} cases failed", reports.len());
	}
	Ok(())
}

async fn run_cases(dir: &Path, case: Option<&str>) -> Result<Vec<CaseReport>> {
	let Some(case) = case else {
		return Ok(harness::test_adapter(dir).await?);
	};
	let case_dir = dir.join(harness::FIXTURES_DIR).join(case);
	if !case_dir.join("expected.json").is_file() {
		bail!(
			"no case {case}: {} has no expected.json",
			case_dir.display()
		);
	}
	Ok(vec![harness::run_case(dir, &case_dir).await?])
}

/// One line per case, then the failures of the cases that failed.
fn format_reports(reports: &[CaseReport]) -> String {
	let mut text = String::new();
	for report in reports {
		let runs = report.runs.len();
		let plural = if runs == 1 { "run" } else { "runs" };
		let status = if report.passed() { "PASS" } else { "FAIL" };
		text.push_str(&format!("{status} {} ({runs} {plural})\n", report.name));
		for failure in &report.failures {
			text.push_str(&format!("  - {failure}\n"));
		}
	}
	let passed = reports.iter().filter(|report| report.passed()).count();
	text.push_str(&format!("{passed} of {} cases passed\n", reports.len()));
	text
}

#[cfg(test)]
mod tests {
	use super::*;
	use wing_archive::SyncReport;

	fn report(name: &str, runs: usize, failures: &[&str]) -> CaseReport {
		CaseReport {
			name: name.to_string(),
			runs: vec![SyncReport::default(); runs],
			failures: failures.iter().map(|failure| failure.to_string()).collect(),
		}
	}

	#[test]
	fn reports_list_each_case_and_its_failures() {
		let text = format_reports(&[
			report("missing-folder", 1, &[]),
			report("two-notes", 2, &["note: 2 records, expected 3"]),
		]);
		assert_eq!(
			text,
			"PASS missing-folder (1 run)\nFAIL two-notes (2 runs)\n  - note: 2 records, expected 3\n1 of 2 cases passed\n"
		);
	}

	#[tokio::test]
	async fn an_unknown_case_is_an_error() {
		let tmp = tempfile::tempdir().unwrap();
		let error = run_cases(tmp.path(), Some("basic")).await.unwrap_err();
		assert!(error.to_string().starts_with("no case basic"), "{error}");
	}
}
