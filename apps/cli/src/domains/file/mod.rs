pub(crate) mod args;

use anyhow::Result;
use clap::Subcommand;
use comfy_table::presets::UTF8_BORDERS_ONLY;

use crate::format_bytes;
use crate::util::prelude::*;

use crate::context::Context;
use wing_core::infra::job::handle::JobReceipt;
use wing_core::infra::query::LibraryQuery;

use self::args::*;
use wing_core::ops::files::{
	copy::action::FileConflictResolution,
	plan::PlanDecision,
	query::preflight::{FilePreflightInput, FilePreflightOutput, PreflightOperation},
};

#[derive(Subcommand, Debug)]
pub enum FileCmd {
	/// Copy files
	Copy(FileCopyArgs),
	/// Get file information
	Info(FileInfoArgs),
	/// List directory contents
	List(FileListArgs),
	/// Rename a file or directory
	Rename(FileRenameArgs),
	/// Move files to trash, or delete them permanently with --permanent
	Delete(FileDeleteArgs),
	/// Create a folder
	Mkdir(FileMkdirArgs),
	/// Undo a copy, move, rename or trash
	Undo(FileUndoArgs),
	/// List recent file operations and what undo can still reverse
	History(FileHistoryArgs),
}

pub async fn run(ctx: &Context, cmd: FileCmd) -> Result<()> {
	match cmd {
		FileCmd::Copy(args) => {
			let dry_run = args.dry_run;
			let input: wing_core::ops::files::copy::input::FileCopyInput = args.into();
			if let Err(errors) = input.validate() {
				anyhow::bail!(errors.join("; "))
			}
			let plan = preflight(
				ctx,
				FilePreflightInput {
					operation: if input.move_files {
						PreflightOperation::Move
					} else {
						PreflightOperation::Copy
					},
					sources: input.sources.paths.clone(),
					destination: Some(input.destination.clone()),
					on_conflict: input.overwrite.then_some(FileConflictResolution::Overwrite),
				},
			)
			.await?;
			if dry_run {
				print_output!(ctx, &plan, print_plan);
				return Ok(());
			}
			print_plan(&plan);

			// Handle confirmation for file copy operations
			let receipt = run_copy_with_confirmation(ctx, input).await?;
			print_output!(ctx, &receipt, |receipt: &JobReceipt| {
				println!("Dispatched copy job {}", receipt.id);
			});
		}
		FileCmd::Rename(args) => {
			let input: wing_core::ops::files::rename::input::FileRenameInput = args.into();
			let receipt: JobReceipt = execute_action!(ctx, input);
			print_output!(ctx, &receipt, |receipt: &JobReceipt| {
				println!("Dispatched rename job {}", receipt.id);
			});
		}
		FileCmd::Delete(args) => {
			let (permanent, yes, count, dry_run) =
				(args.permanent, args.yes, args.paths.len(), args.dry_run);
			let input: wing_core::ops::files::delete::input::FileDeleteInput = args.into();
			if let Err(errors) = input.validate() {
				anyhow::bail!(errors.join("; "))
			}
			let plan = preflight(
				ctx,
				FilePreflightInput {
					operation: PreflightOperation::Delete,
					sources: input.targets.paths.clone(),
					destination: None,
					on_conflict: None,
				},
			)
			.await?;
			if dry_run {
				print_output!(ctx, &plan, print_plan);
				return Ok(());
			}
			print_plan(&plan);
			if permanent {
				crate::util::confirm::confirm_or_abort(
					&format!("Permanently delete {count} item(s)? This cannot be undone."),
					yes,
				)?;
			}
			let receipt: JobReceipt = execute_action!(ctx, input);
			print_output!(ctx, &receipt, |receipt: &JobReceipt| {
				let verb = if permanent { "delete" } else { "trash" };
				println!("Dispatched {verb} job {}", receipt.id);
			});
		}
		FileCmd::Mkdir(args) => {
			let input: wing_core::ops::files::create_folder::input::CreateFolderInput = args.into();
			let output: wing_core::ops::files::create_folder::output::CreateFolderOutput =
				execute_action!(ctx, input);
			print_output!(
				ctx,
				&output,
				|output: &wing_core::ops::files::create_folder::output::CreateFolderOutput| {
					println!("Created {}", output.folder_path);
				}
			);
		}
		FileCmd::Undo(args) => {
			let input = wing_core::ops::files::undo::FileUndoInput {
				job_id: args.job_id,
			};
			let receipt: JobReceipt = execute_action!(ctx, input);
			print_output!(ctx, &receipt, |receipt: &JobReceipt| {
				println!("Dispatched undo job {}", receipt.id);
			});
		}
		FileCmd::History(args) => {
			use wing_core::ops::files::undo::{UndoHistoryInput, UndoHistoryOutput};
			let output: UndoHistoryOutput = execute_query!(
				ctx,
				UndoHistoryInput {
					limit: Some(args.limit)
				}
			);
			print_output!(ctx, &output, |output: &UndoHistoryOutput| {
				if output.operations.is_empty() {
					println!("No file operations recorded");
					return;
				}
				let mut table = comfy_table::Table::new();
				table.load_style(UTF8_BORDERS_ONLY);
				table.set_header(vec!["Job", "Operation", "When", "Undoable", "Example"]);
				for operation in &output.operations {
					let when = chrono::DateTime::from_timestamp(operation.started_at, 0)
						.map(|time| {
							time.with_timezone(&chrono::Local)
								.format("%Y-%m-%d %H:%M")
								.to_string()
						})
						.unwrap_or_default();
					table.add_row(vec![
						operation.job_id.to_string(),
						operation.operation.clone(),
						when,
						format!("{} of {}", operation.pending, operation.effects),
						operation.example.display().to_string(),
					]);
				}
				println!("{table}");
				println!(
					"Records and replaced files are kept for {} days.",
					output.retention_days
				);
			});
		}
		FileCmd::Info(args) => {
			let file_info = get_file_info(ctx, &args.path).await?;
			print_output!(ctx, &file_info, |info: &Option<wing_core::domain::File>| {
				match info {
					Some(file) => {
						println!("{}", serde_json::to_string_pretty(file).unwrap());
					}
					None => {
						println!("File not found or not indexed in WingDrive");
					}
				}
			});
		}
		FileCmd::List(args) => {
			let sort_by = match args.sort_by.to_lowercase().as_str() {
				"name" => wing_core::ops::files::query::DirectorySortBy::Name,
				"modified" => wing_core::ops::files::query::DirectorySortBy::Modified,
				"size" => wing_core::ops::files::query::DirectorySortBy::Size,
				"type" => wing_core::ops::files::query::DirectorySortBy::Type,
				_ => {
					anyhow::bail!(
						"Invalid sort option: {}. Valid options are: name, modified, size, type",
						args.sort_by
					);
				}
			};
			let directory_listing =
				list_directory(ctx, &args.path, args.limit, args.include_hidden, sort_by).await?;
			print_output!(
				ctx,
				&directory_listing,
				|listing: &wing_core::ops::files::query::DirectoryListingOutput| {
					println!("Directory: {}", args.path.display());
					println!("Found {} items:", listing.files.len());
					println!();

					// Create a table to display the results
					let mut table = comfy_table::Table::new();
					table.load_style(UTF8_BORDERS_ONLY);
					table.set_header(vec!["Name", "Type", "Size", "Modified"]);

					for file in &listing.files {
						use wing_core::domain::file::EntryKind;

						let file_type = match file.kind {
							EntryKind::File => "File",
							EntryKind::Directory => "Directory",
							EntryKind::Symlink => "Symlink",
						};
						let size_str = match file.kind {
							EntryKind::Directory => "-".to_string(),
							_ => format_bytes(file.size),
						};
						// `name` excludes the extension
						let name = match &file.extension {
							Some(ext) if !ext.is_empty() => format!("{}.{ext}", file.name),
							_ => file.name.clone(),
						};

						table.add_row(vec![
							name,
							file_type.to_string(),
							size_str,
							file.modified_at.format("%Y-%m-%d %H:%M:%S").to_string(),
						]);
					}

					println!("{}", table);
				}
			);
		}
	}
	Ok(())
}

/// Asks the daemon what a copy, move or delete would do, without doing it.
async fn preflight(ctx: &Context, input: FilePreflightInput) -> Result<FilePreflightOutput> {
	let plan: FilePreflightOutput = execute_query!(ctx, input);
	Ok(plan)
}

/// Prints a preflight plan: totals, conflicts and, for a delete, last copies.
fn print_plan(plan: &FilePreflightOutput) {
	let at_least = if plan.truncated { "at least " } else { "" };
	println!(
		"Plan: {at_least}{} file(s), {}",
		plan.files,
		format_bytes(plan.bytes)
	);
	if plan.conflicts > 0 {
		println!("  {} destination(s) already exist:", plan.conflicts);
		for item in plan
			.items
			.iter()
			.filter(|item| !matches!(item.decision, None | Some(PlanDecision::Write)))
		{
			let target = item
				.destination
				.as_ref()
				.map(|d| d.display())
				.unwrap_or_default();
			let effect = match &item.decision {
				Some(PlanDecision::Overwrite) => "overwrite".to_string(),
				Some(PlanDecision::Skip) => "skip".to_string(),
				Some(PlanDecision::Rename(path)) => format!("rename to {}", path.display()),
				Some(PlanDecision::Abort) => "abort".to_string(),
				_ => "ask".to_string(),
			};
			println!("    {target}: {effect}");
		}
	}
	if let Some(last) = plan.last_copies.filter(|n| *n > 0) {
		println!("  {last} file(s) are the last copy of their content in this library");
	}
	if let Some(unknown) = plan.unknown_files.filter(|n| *n > 0) {
		println!(
			"  {unknown} file(s) are not indexed with their content; other copies are unknown"
		);
	}
}

/// Run file copy with confirmation handling
async fn run_copy_with_confirmation(
	ctx: &Context,
	mut input: wing_core::ops::files::copy::input::FileCopyInput,
) -> Result<JobReceipt> {
	use crate::util::confirm::prompt_for_choice;
	use wing_core::infra::action::LibraryAction;
	use wing_core::ops::files::copy::action::FileCopyAction;

	// Build the action from input for validation purposes
	let action = FileCopyAction::from_input(input.clone())
		.map_err(|e| anyhow::anyhow!("Failed to build action: {}", e))?;

	// Use the action's validation method to check for conflicts
	// For CLI validation, we'll use a simplified approach since we don't have full library context
	// In a production system, you'd want to pass the actual library context

	// Simple conflict detection - check if destination exists and overwrite is not enabled
	if !input.overwrite {
		let has_conflict = check_for_simple_conflicts(&action).await?;
		if has_conflict {
			use wing_core::infra::action::ConfirmationRequest;

			let request = ConfirmationRequest {
				message: "Destination file(s) already exist. What would you like to do?"
					.to_string(),
				choices: vec![
					"Overwrite the existing file(s)".to_string(),
					"Rename the new file(s) (e.g., file.txt -> file (1).txt)".to_string(),
					"Abort this copy operation".to_string(),
				],
				metadata: None,
			};

			let choice_index = prompt_for_choice(request)?;

			// Apply the user's choice to the input
			match choice_index {
				0 => {
					// Overwrite: set conflict resolution in input
					use wing_core::ops::files::copy::action::FileConflictResolution;
					input.on_conflict = Some(FileConflictResolution::Overwrite);
				}
				1 => {
					// Auto-rename: set conflict resolution in input
					use wing_core::ops::files::copy::action::FileConflictResolution;
					input.on_conflict = Some(FileConflictResolution::AutoModifyName);
				}
				2 => {
					// Abort
					anyhow::bail!("Operation aborted by user");
				}
				_ => {
					anyhow::bail!("Invalid choice selected");
				}
			}
		}
	}

	// Execute the action using the input
	let receipt: JobReceipt = execute_action!(ctx, input);
	Ok(receipt)
}

/// Simple conflict detection for CLI
async fn check_for_simple_conflicts(
	action: &wing_core::ops::files::copy::action::FileCopyAction,
) -> Result<bool> {
	let Some(destination) = action.destination.as_local_path() else {
		return Ok(false);
	};
	let is_directory = match tokio::fs::metadata(destination).await {
		Ok(metadata) => metadata.is_dir(),
		Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
		Err(error) => return Err(error.into()),
	};
	for source in &action.sources.paths {
		let Some(source) = source.as_local_path() else {
			continue;
		};
		let target = if is_directory || action.sources.paths.len() > 1 {
			let Some(name) = source.file_name() else {
				continue;
			};
			destination.join(name)
		} else {
			destination.to_path_buf()
		};
		match tokio::fs::symlink_metadata(target).await {
			Ok(_) => return Ok(true),
			Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
			Err(error) => return Err(error.into()),
		}
	}
	Ok(false)
}

/// Get file information using the FileByPathQuery
async fn get_file_info(
	ctx: &Context,
	path: &std::path::Path,
) -> Result<Option<wing_core::domain::File>> {
	use wing_core::ops::files::query::FileByPathQuery;

	// Create the query with the local path
	let query =
		FileByPathQuery::new(std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf()));

	// Execute the query using the core client
	let json_response = ctx.core.query(&query, ctx.library_id).await?;
	let result: Option<wing_core::domain::File> = serde_json::from_value(json_response)?;

	Ok(result)
}

/// List directory contents using the DirectoryListingQuery
async fn list_directory(
	ctx: &Context,
	path: &std::path::Path,
	limit: Option<u32>,
	include_hidden: bool,
	sort_by: wing_core::ops::files::query::DirectorySortBy,
) -> Result<wing_core::ops::files::query::DirectoryListingOutput> {
	use wing_core::ops::files::query::DirectoryListingQuery;

	// Create the WingPath for the directory
	let wing_path = self::args::local_path(path.to_path_buf());

	// Create the query input
	let input = wing_core::ops::files::query::DirectoryListingInput {
		path: wing_path,
		limit,
		include_hidden: Some(include_hidden),
		sort_by,
		folders_first: None,
		sort_direction: None,
	};

	// Execute the query using the core client
	let json_response = ctx.core.query(&input, ctx.library_id).await?;
	let result: wing_core::ops::files::query::DirectoryListingOutput =
		serde_json::from_value(json_response)?;

	Ok(result)
}
