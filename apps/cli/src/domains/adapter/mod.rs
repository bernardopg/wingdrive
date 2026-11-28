//! `wing adapter`: tools for writing an adapter. They run locally and need no daemon.

use std::path::PathBuf;

use anyhow::{bail, Result};
use clap::{Args, Subcommand};

use wing_archive::harness;

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
	let reports = harness::test_cases(&args.dir, args.case.as_deref()).await?;
	print!("{}", harness::format_reports(&reports));
	let failed = reports.iter().filter(|report| !report.passed()).count();
	if failed > 0 {
		bail!("{failed} of {} cases failed", reports.len());
	}
	Ok(())
}
