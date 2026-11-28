//! `wing adapter`: tools for writing an adapter. They run locally and need no daemon.
//!
//! `test` runs the contract harness, which lives in `wing-archive`. `validate` runs the review
//! checks Wingbot posts on a registry pull request. Those checks are Python (`tools/wingreg` in
//! bernardopg/wingdrive-adapters) and parse adapter code with Python's own `ast`, so the CLI runs
//! them from a copy of the registry instead of keeping a Rust port that would drift from what the
//! review enforces.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::{Args, Subcommand};
use tokio::process::Command;

use wing_archive::harness;

const REGISTRY_URL: &str = "https://github.com/bernardopg/wingdrive-adapters";

#[derive(Subcommand, Debug)]
pub enum AdapterCmd {
	/// Run an adapter against its fixtures/<case> folders, as the daemon and the registry do
	Test(TestArgs),
	/// Run the registry's review checks on an adapter folder, as Wingbot does on a pull request
	Validate(ValidateArgs),
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

#[derive(Args, Debug)]
pub struct ValidateArgs {
	/// Adapter folder, the one that holds adapter.toml
	#[arg(default_value = ".")]
	pub dir: PathBuf,
	/// A checkout of bernardopg/wingdrive-adapters to take the checks from. Without it, a copy in
	/// the cache folder is cloned on first use and updated on each run. WINGDRIVE_ADAPTER_REGISTRY
	/// sets the same
	#[arg(long)]
	pub registry: Option<PathBuf>,
	/// Only the static checks: skip the adapter's tests and the contract harness
	#[arg(long = "static")]
	pub static_only: bool,
}

pub async fn run(cmd: AdapterCmd) -> Result<()> {
	match cmd {
		AdapterCmd::Test(args) => test(&args).await,
		AdapterCmd::Validate(args) => validate(&args).await,
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

async fn validate(args: &ValidateArgs) -> Result<()> {
	let dir = std::fs::canonicalize(&args.dir)
		.with_context(|| format!("{} is not a folder", args.dir.display()))?;
	if !dir.join("adapter.toml").is_file() {
		bail!("{} has no adapter.toml", dir.display());
	}
	let registry = match args
		.registry
		.clone()
		.or_else(|| std::env::var_os("WINGDRIVE_ADAPTER_REGISTRY").map(PathBuf::from))
	{
		Some(path) => path,
		None => cached_registry().await?,
	};
	let tools = registry.join("tools");
	if !tools.join("wingreg").join("__main__.py").is_file() {
		bail!(
			"{} is not a checkout of {REGISTRY_URL} (tools/wingreg is missing)",
			registry.display()
		);
	}

	let mut command = python_command().await?;
	command.args(["-m", "wingreg", "check"]).arg(&dir);
	if !args.static_only {
		command.arg("--run-code");
	}
	// The harness is this same binary, so the checks run against the contract this CLI speaks.
	if let Ok(exe) = std::env::current_exe() {
		command.env("WING_ADAPTER_TEST", exe);
	}
	let status = command
		.current_dir(&tools)
		.status()
		.await
		.context("could not start the registry checks")?;
	if !status.success() {
		bail!("the adapter does not pass the registry checks");
	}
	Ok(())
}

/// The registry copy in the cache folder, cloned or brought up to date with `main`.
///
/// Offline, an existing copy is used as it is, with a warning; without one there is nothing to
/// check against, so that is an error.
async fn cached_registry() -> Result<PathBuf> {
	let root = dirs::cache_dir()
		.context("no cache folder on this system; pass --registry")?
		.join("wingdrive")
		.join("adapter-registry");
	if root.join(".git").is_dir() {
		let fetched = git(
			&root,
			&["fetch", "--quiet", "--depth", "1", "origin", "main"],
		)
		.await;
		match fetched {
			Ok(()) => git(&root, &["reset", "--quiet", "--hard", "FETCH_HEAD"]).await?,
			Err(error) => eprintln!(
				"Could not update the registry checks ({error}); using the copy in {}",
				root.display()
			),
		}
		return Ok(root);
	}
	if let Some(parent) = root.parent() {
		std::fs::create_dir_all(parent)?;
	}
	let target = root.to_string_lossy().into_owned();
	git(
		Path::new("."),
		&["clone", "--quiet", "--depth", "1", REGISTRY_URL, &target],
	)
	.await
	.context("could not clone the registry; pass --registry with a local checkout")?;
	Ok(root)
}

async fn git(dir: &Path, args: &[&str]) -> Result<()> {
	let status = Command::new("git")
		.args(args)
		.current_dir(dir)
		.status()
		.await
		.context("git is not installed")?;
	if !status.success() {
		bail!("git {} failed", args.first().copied().unwrap_or_default());
	}
	Ok(())
}

/// A Python that can run the checks: `uv` with Python 3.12 and jsonschema, or a Python 3.11+ that
/// already has jsonschema (the checks use `tomllib` and `sys.stdlib_module_names`).
async fn python_command() -> Result<Command> {
	if found(Command::new("uv").arg("--version")).await {
		let mut command = Command::new("uv");
		command.args([
			"run",
			"--quiet",
			"--python",
			"3.12",
			"--with",
			"jsonschema",
			"python",
		]);
		return Ok(command);
	}
	let probe = "import sys, jsonschema; sys.exit(sys.version_info < (3, 11))";
	for python in ["python3", "python"] {
		if found(Command::new(python).args(["-c", probe])).await {
			return Ok(Command::new(python));
		}
	}
	bail!(
		"the registry checks need uv (https://docs.astral.sh/uv/) or Python 3.11+ with jsonschema \
		 (pip install jsonschema)"
	)
}

async fn found(command: &mut Command) -> bool {
	command
		.stdout(std::process::Stdio::null())
		.stderr(std::process::Stdio::null())
		.status()
		.await
		.map(|status| status.success())
		.unwrap_or(false)
}
